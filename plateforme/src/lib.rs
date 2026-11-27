//! # amazone-plateforme
//!
//! Plateforme en ligne AMAZONE COMP (ADR 0004, 0008, 0009) : comptes utilisateurs,
//! vérification du courriel, conditions générales versionnées, inscription des
//! entreprises (fiche [`ac_noyau::entreprise::FicheEntreprise`], validation unique
//! [`ac_noyau::entreprise::FicheEntreprise::valider`]) et licences signées Ed25519.
//!
//! Projet **séparé** du workspace de l'application (son propre `Cargo.toml`) : il
//! dépend de `ac-noyau` et `ac-licence` par chemin pour ne jamais dupliquer la
//! validation de la fiche entreprise ni le format de la charge utile de licence.

#![forbid(unsafe_code)]

pub mod cgu;
pub mod comptes;
pub mod config;
pub mod courriel;
pub mod entreprises;
pub mod erreur;
pub mod jetons;
pub mod licences;
pub mod limitation;

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use axum::{
    extract::{rejection::JsonRejection, Json},
    http::HeaderMap,
    Router,
};
use chrono::Utc;
use serde::de::DeserializeOwned;
use sqlx::postgres::PgPool;
use thiserror::Error;

pub use config::Config;
pub use courriel::EnvoyeurCourriel;
pub use erreur::ErreurApi;
pub use jetons::SignataireLicences;

use comptes::EchecsConnexion;

/// État partagé de la plateforme, cloné à chaque requête.
#[derive(Clone)]
pub struct Etat {
    /// Pool de connexions PostgreSQL.
    pub bdd: PgPool,
    /// Signataire des jetons de licence.
    pub signataire: Arc<SignataireLicences>,
    /// Transport de courriels (codes de vérification, réinitialisation).
    pub courriel: Arc<dyn EnvoyeurCourriel>,
    /// Adresse d'expéditeur des courriels.
    pub courriel_expediteur: String,
    /// Transferts de poste autorisés par an et par entreprise.
    pub transferts_poste_par_an: i64,
    /// Échecs de connexion par compte (verrouillage temporaire, en mémoire).
    pub echecs_connexion: Arc<Mutex<HashMap<String, EchecsConnexion>>>,
}

impl Etat {
    /// Assemble un état à partir d'un pool de connexions déjà prêt (tests, outils).
    pub fn nouveau(
        bdd: PgPool,
        signataire: SignataireLicences,
        courriel: Arc<dyn EnvoyeurCourriel>,
        courriel_expediteur: String,
        transferts_poste_par_an: i64,
    ) -> Self {
        Etat {
            bdd,
            signataire: Arc::new(signataire),
            courriel,
            courriel_expediteur,
            transferts_poste_par_an,
            echecs_connexion: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Connecte la base, exécute les migrations et charge la clé de signature.
    pub async fn creer(config: &Config) -> Result<Etat, ErreurDemarrage> {
        let bdd = PgPool::connect(&config.bdd_url).await?;
        sqlx::migrate!("./migrations").run(&bdd).await?;
        Ok(Etat {
            bdd,
            signataire: Arc::new(SignataireLicences::nouveau(config.cle_privee)),
            courriel: Arc::new(courriel::EnvoyeurJournal),
            courriel_expediteur: config.courriel_expediteur.clone(),
            transferts_poste_par_an: config.transferts_poste_par_an,
            echecs_connexion: Arc::new(Mutex::new(HashMap::new())),
        })
    }
}

/// Erreur au démarrage du service.
#[derive(Debug, Error)]
pub enum ErreurDemarrage {
    /// Configuration incorrecte.
    #[error(transparent)]
    Configuration(#[from] config::ErreurConfig),
    /// Connexion PostgreSQL impossible.
    #[error("Connexion à PostgreSQL impossible : {0}")]
    BaseDeDonnees(#[from] sqlx::Error),
    /// Migrations impossibles.
    #[error("Migrations de la base de données impossibles : {0}")]
    Migrations(#[from] sqlx::migrate::MigrateError),
    /// Erreur d'écoute ou de service HTTP.
    #[error("Erreur du serveur HTTP : {0}")]
    Serveur(String),
}

/// Compte authentifié par un jeton d'accès (`Authorization: Bearer …`).
#[derive(Debug, Clone)]
pub struct CompteConnecte {
    /// Identifiant du compte.
    pub compte_id: i64,
    /// Adresse électronique du compte.
    pub courriel: String,
    /// Empreinte SHA-256 du jeton présenté (permet de fermer la session).
    pub jeton_hash: String,
}

/// Extrait le compte depuis l'en-tête `Authorization`, ou refuse la requête.
pub async fn authentifier(etat: &Etat, en_tetes: &HeaderMap) -> Result<CompteConnecte, ErreurApi> {
    let jeton = en_tetes
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(str::trim)
        .filter(|j| !j.is_empty())
        .ok_or_else(ErreurApi::non_autorise)?;
    let hash = jetons::empreinte_sha256(jeton);
    let ligne: Option<(i64, String)> = sqlx::query_as(
        "SELECT c.id, c.courriel FROM sessions s
         JOIN comptes c ON c.id = s.compte_id
         WHERE s.jeton_hash = $1 AND s.expire_le > now() AND c.actif",
    )
    .bind(&hash)
    .fetch_optional(&etat.bdd)
    .await
    .map_err(erreur_bdd)?;
    let (compte_id, courriel) = ligne.ok_or_else(ErreurApi::non_autorise)?;
    Ok(CompteConnecte {
        compte_id,
        courriel,
        jeton_hash: hash,
    })
}

/// Extrait le corps JSON d'une requête, avec message d'erreur français en cas de rejet.
pub fn corps_json<T: DeserializeOwned>(
    corps: Result<Json<T>, JsonRejection>,
) -> Result<T, ErreurApi> {
    corps.map(|Json(v)| v).map_err(|_| {
        ErreurApi::invalide("Le corps de la requête est absent ou n'est pas du JSON valide.")
    })
}

/// Erreur de base de données : le détail technique reste dans les journaux du serveur.
pub fn erreur_bdd(e: sqlx::Error) -> ErreurApi {
    eprintln!("[erreur] base de données : {e}");
    ErreurApi::interne()
}

/// Vrai si l'erreur SQL est un conflit d'unicité (code PostgreSQL 23505).
pub fn est_conflit_unicite(e: &sqlx::Error) -> bool {
    e.as_database_error()
        .and_then(|b| b.code())
        .is_some_and(|code| code.as_ref() == "23505")
}

/// Date du jour au Bénin (UTC+1), pour la validation des fiches entreprise.
pub fn aujourdhui() -> ac_noyau::Date {
    let decalage = chrono::FixedOffset::east_opt(3600).unwrap_or_else(Utc::fix);
    Utc::now().with_timezone(&decalage).date_naive()
}

/// Construit le routeur complet de l'API `/v1` (limitation de débit comprise).
pub fn application(etat: Etat) -> Router {
    Router::new()
        .merge(comptes::routes())
        .merge(cgu::routes())
        .merge(entreprises::routes())
        .merge(licences::routes())
        .layer(axum::extract::DefaultBodyLimit::max(10 * 1024 * 1024))
        .layer(limitation::CoucheLimitation::defaut())
        .with_state(etat)
}

/// État sans base de données, pour les tests unitaires sans SQL.
#[cfg(test)]
pub(crate) fn etat_de_test() -> Etat {
    let bdd = PgPool::connect_lazy("postgres://test@localhost/test").expect("URL de test invalide");
    Etat::nouveau(
        bdd,
        SignataireLicences::nouveau([7u8; 32]),
        Arc::new(courriel::EnvoyeurMemoire::nouveau()),
        "test@localhost".to_owned(),
        2,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn date_au_benin() {
        // La date du Bénin (UTC+1) ne peut pas être en retard de plus d'un jour
        // sur la date UTC du test.
        let d = aujourdhui();
        assert!(d >= chrono::Utc::now().date_naive());
    }
}
