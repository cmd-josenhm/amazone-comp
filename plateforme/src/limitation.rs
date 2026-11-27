//! Limitation de débit : seau de jetons par adresse IP et par classe de route,
//! appliqué par une couche [`tower`] (issue n° 12).
//!
//! Comme la plateforme tourne en une instance derrière Caddy (ADR 0008), les compteurs
//! vivent en mémoire : c'est simple, sans dépendance supplémentaire, et largement
//! suffisant pour un VPS unique. La clé est l'adresse IP du client (en-tête
//! `X-Forwarded-For` posé par Caddy, sinon la connexion TCP).

use std::{
    collections::HashMap,
    convert::Infallible,
    future::Future,
    net::SocketAddr,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll},
    time::{Duration, Instant},
};

use axum::{
    extract::{ConnectInfo, Request},
    response::{IntoResponse, Response},
};
use tower::{Layer, Service};

/// Classe de route : chaque classe a son propre budget de requêtes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ClasseDebit {
    /// Création de compte, envoi de codes.
    Inscription,
    /// Connexion et réinitialisation du mot de passe.
    Connexion,
    /// Émission, renouvellement et transfert de licences.
    Licence,
    /// Toutes les autres routes.
    Generale,
}

impl ClasseDebit {
    /// Budget maximal de requêtes dans la fenêtre.
    pub fn maximum(self) -> u64 {
        match self {
            ClasseDebit::Inscription => 20,
            ClasseDebit::Connexion => 60,
            ClasseDebit::Licence => 60,
            ClasseDebit::Generale => 300,
        }
    }

    /// Durée de la fenêtre glissante.
    pub fn fenetre(self) -> Duration {
        match self {
            ClasseDebit::Inscription => Duration::from_secs(3600),
            ClasseDebit::Connexion | ClasseDebit::Licence | ClasseDebit::Generale => {
                Duration::from_secs(300)
            }
        }
    }

    /// Classe d'une route, d'après son chemin (`GET` et `POST` confondus).
    pub fn pour_chemin(chemin: &str) -> ClasseDebit {
        if chemin.starts_with("/v1/comptes") || chemin.starts_with("/v1/mots-de-passe") {
            ClasseDebit::Inscription
        } else if chemin.starts_with("/v1/sessions") {
            ClasseDebit::Connexion
        } else if chemin.starts_with("/v1/licences") {
            ClasseDebit::Licence
        } else {
            ClasseDebit::Generale
        }
    }
}

/// Compteur d'une clé sur une fenêtre glissante (comptage entier, sans flottant).
#[derive(Debug, Clone, Copy)]
struct Compteur {
    debut_fenetre: Instant,
    requetes: u64,
}

/// Limiteur de débit en mémoire, partagé par toutes les connexions.
#[derive(Debug, Default)]
pub struct Limiteur {
    compteurs: Mutex<HashMap<(String, ClasseDebit), Compteur>>,
}

impl Limiteur {
    /// Crée un limiteur vide.
    pub fn nouveau() -> Self {
        Self::default()
    }

    /// Vrai si la requête est autorisée (consomme un jeton).
    pub fn autoriser(&self, cle: &str, classe: ClasseDebit, maintenant: Instant) -> bool {
        let mut compteurs = match self.compteurs.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };
        // Nettoyage ponctuel des entrées inactives pour ne pas grossir indéfiniment.
        if compteurs.len() > 4096 {
            compteurs.retain(|_, c| maintenant.duration_since(c.debut_fenetre) < classe.fenetre());
        }
        let entree = compteurs
            .entry((cle.to_owned(), classe))
            .or_insert(Compteur {
                debut_fenetre: maintenant,
                requetes: 0,
            });
        if maintenant.duration_since(entree.debut_fenetre) >= classe.fenetre() {
            entree.debut_fenetre = maintenant;
            entree.requetes = 0;
        }
        if entree.requetes >= classe.maximum() {
            return false;
        }
        entree.requetes += 1;
        true
    }
}

/// Adresse IP du client : `X-Forwarded-For` (proxy Caddy) ou connexion TCP.
pub fn adresse_client(requete: &Request) -> String {
    if let Some(xff) = requete
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
    {
        if let Some(premier) = xff.split(',').next() {
            let adresse = premier.trim();
            if !adresse.is_empty() {
                return adresse.to_owned();
            }
        }
    }
    if let Some(ConnectInfo(adresse)) = requete.extensions().get::<ConnectInfo<SocketAddr>>() {
        return adresse.ip().to_string();
    }
    "inconnu".to_owned()
}

/// Couche [`tower`] de limitation de débit, à appliquer à tout le routeur.
#[derive(Clone)]
pub struct CoucheLimitation {
    limiteur: Arc<Limiteur>,
}

impl CoucheLimitation {
    /// Couche avec budgets par défaut ([`ClasseDebit`]).
    pub fn defaut() -> Self {
        CoucheLimitation {
            limiteur: Arc::new(Limiteur::nouveau()),
        }
    }
}

impl<S> Layer<S> for CoucheLimitation {
    type Service = ServiceLimitation<S>;
    fn layer(&self, interne: S) -> Self::Service {
        ServiceLimitation {
            interne,
            limiteur: Arc::clone(&self.limiteur),
        }
    }
}

/// Service enveloppé par [`CoucheLimitation`].
#[derive(Clone)]
pub struct ServiceLimitation<S> {
    interne: S,
    limiteur: Arc<Limiteur>,
}

impl<S> Service<Request> for ServiceLimitation<S>
where
    S: Service<Request, Response = Response, Error = Infallible> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = Response;
    type Error = Infallible;
    type Future = Pin<Box<dyn Future<Output = Result<Response, Infallible>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.interne.poll_ready(cx)
    }

    fn call(&mut self, requete: Request) -> Self::Future {
        let cle = adresse_client(&requete);
        let classe = ClasseDebit::pour_chemin(requete.uri().path());
        if !self.limiteur.autoriser(&cle, classe, Instant::now()) {
            return Box::pin(async {
                Ok(axum::Json(serde_json::json!({
                    "code": "TROP_DE_REQUETES",
                    "message": "Trop de requêtes. Réessayez dans quelques minutes."
                }))
                .into_response())
            });
        }
        let mut interne = self.interne.clone();
        Box::pin(async move { interne.call(requete).await })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classes_selon_le_chemin() {
        assert_eq!(
            ClasseDebit::pour_chemin("/v1/comptes"),
            ClasseDebit::Inscription
        );
        assert_eq!(
            ClasseDebit::pour_chemin("/v1/mots-de-passe/reinitialiser"),
            ClasseDebit::Inscription
        );
        assert_eq!(
            ClasseDebit::pour_chemin("/v1/sessions"),
            ClasseDebit::Connexion
        );
        assert_eq!(
            ClasseDebit::pour_chemin("/v1/licences/renouveler"),
            ClasseDebit::Licence
        );
        assert_eq!(
            ClasseDebit::pour_chemin("/v1/cgu/courante"),
            ClasseDebit::Generale
        );
    }

    #[test]
    fn fenetre_glissante_reautorise_apres_expiration() {
        let limiteur = Limiteur::nouveau();
        let debut = Instant::now();
        for _ in 0..ClasseDebit::Connexion.maximum() {
            assert!(limiteur.autoriser("ip1", ClasseDebit::Connexion, debut));
        }
        assert!(!limiteur.autoriser("ip1", ClasseDebit::Connexion, debut));
        // Une autre clé n'est pas affectée.
        assert!(limiteur.autoriser("ip2", ClasseDebit::Connexion, debut));
        // Après la fenêtre, le compteur repart à zéro.
        let plus_tard = debut + ClasseDebit::Connexion.fenetre();
        assert!(limiteur.autoriser("ip1", ClasseDebit::Connexion, plus_tard));
    }

    #[test]
    fn budgets_independants_par_classe() {
        let limiteur = Limiteur::nouveau();
        let t = Instant::now();
        for _ in 0..ClasseDebit::Licence.maximum() {
            assert!(limiteur.autoriser("ip", ClasseDebit::Licence, t));
        }
        assert!(!limiteur.autoriser("ip", ClasseDebit::Licence, t));
        assert!(limiteur.autoriser("ip", ClasseDebit::Generale, t));
    }

    #[test]
    fn adresse_depuis_x_forwarded_for() {
        let requete = Request::builder()
            .header("x-forwarded-for", "203.0.113.7, 10.0.0.1")
            .body(axum::body::Body::empty())
            .unwrap();
        assert_eq!(adresse_client(&requete), "203.0.113.7");
        let sans = Request::builder().body(axum::body::Body::empty()).unwrap();
        assert_eq!(adresse_client(&sans), "inconnu");
    }
}
