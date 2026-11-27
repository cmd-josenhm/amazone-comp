//! Comptes : inscription, vérification du courriel (code à 6 chiffres), sessions,
//! mot de passe oublié (ADR 0004).
//!
//! Sécurité : mots de passe hachés en **Argon2id** (jamais stockés en clair), jetons de
//! session opaques (seule leur empreinte SHA-256 est stockée), codes à usage unique
//! valables [`courriel::DUREE_CODE_MINUTES`] minutes, verrouillage temporaire après
//! [`SEUIL_VERROUILLAGE`] mots de passe faux.

use std::{sync::Mutex, time::Duration};

use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use axum::{
    extract::rejection::JsonRejection,
    extract::{Json, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{delete, post},
    Router,
};
use chrono::Utc;
use rand::{thread_rng, RngCore};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{courriel, erreur::ErreurApi, jetons, Etat};

/// Nombre de mots de passe faux avant le verrouillage temporaire du compte.
pub const SEUIL_VERROUILLAGE: u64 = 10;

const DUREE_VERROU: Duration = Duration::from_secs(15 * 60);
const LONGUEUR_MIN_MOT_DE_PASSE: usize = 10;
const LONGUEUR_MAX_MOT_DE_PASSE: usize = 200;
const CODE_INCORRECT: &str = "Code de vérification incorrect. Vérifiez les chiffres saisis.";

/// Compteur d'échecs de connexion d'un compte (verrouillage temporaire, en mémoire).
#[derive(Debug, Clone, Copy)]
pub struct EchecsConnexion {
    /// Nombre d'échecs consécutifs.
    pub echecs: u64,
    /// Instant du dernier échec.
    pub dernier: std::time::Instant,
    /// Verrouillé jusqu'à cet instant, s'il y a lieu.
    pub verrouille_jusqu: Option<std::time::Instant>,
}

/// Demande de création de compte.
#[derive(Debug, Deserialize)]
pub struct CreationCompte {
    /// Adresse électronique (identifiant de connexion).
    pub courriel: String,
    /// Mot de passe (au moins 10 caractères).
    pub mot_de_passe: String,
    /// Nom de l'utilisateur.
    pub nom: String,
}

/// Demande de vérification du courriel.
#[derive(Debug, Deserialize)]
pub struct VerificationCourriel {
    /// Adresse électronique du compte.
    pub courriel: String,
    /// Code à 6 chiffres reçu par courriel.
    pub code: String,
}

/// Demande simple identifiée par une adresse électronique (renvoi de code, mot de passe oublié).
#[derive(Debug, Deserialize)]
pub struct DemandeCourriel {
    /// Adresse électronique du compte.
    pub courriel: String,
}

/// Demande de connexion.
#[derive(Debug, Deserialize)]
pub struct Connexion {
    /// Adresse électronique du compte.
    pub courriel: String,
    /// Mot de passe.
    pub mot_de_passe: String,
}

/// Demande de réinitialisation du mot de passe (mot de passe oublié).
#[derive(Debug, Deserialize)]
pub struct Reinitialisation {
    /// Adresse électronique du compte.
    pub courriel: String,
    /// Code à 6 chiffres reçu par courriel.
    pub code: String,
    /// Nouveau mot de passe.
    pub nouveau_mot_de_passe: String,
}

/// Informations sur le compte connecté, renvoyées à la connexion.
#[derive(Debug, Serialize)]
pub struct CompteResume {
    /// Identifiant du compte.
    pub id: i64,
    /// Adresse électronique.
    pub courriel: String,
    /// Nom de l'utilisateur.
    pub nom: String,
}

/// Routes `/v1` des comptes et sessions.
pub fn routes() -> Router<crate::Etat> {
    Router::new()
        .route("/v1/comptes", post(creer_compte))
        .route("/v1/comptes/verifications", post(verifier_courriel))
        .route("/v1/comptes/codes", post(renvoyer_code))
        .route("/v1/sessions", post(ouvrir_session))
        .route("/v1/sessions/courante", delete(fermer_session))
        .route("/v1/mots-de-passe/oublie", post(mot_de_passe_oublie))
        .route(
            "/v1/mots-de-passe/reinitialiser",
            post(reinitialiser_mot_de_passe),
        )
}

/// Contrôle la politique de mots de passe.
pub fn mot_de_passe_valide(mot_de_passe: &str) -> Result<(), ErreurApi> {
    let longueur = mot_de_passe.chars().count();
    if longueur < LONGUEUR_MIN_MOT_DE_PASSE {
        return Err(ErreurApi::invalide(
            "Le mot de passe doit contenir au moins 10 caractères.",
        ));
    }
    if longueur > LONGUEUR_MAX_MOT_DE_PASSE {
        return Err(ErreurApi::invalide(
            "Le mot de passe ne peut pas dépasser 200 caractères.",
        ));
    }
    Ok(())
}

/// Hache un mot de passe en Argon2id (sel aléatoire de 16 octets).
pub fn hacher_mot_de_passe(mot_de_passe: &str) -> Result<String, ErreurApi> {
    let mut sel_octets = [0u8; 16];
    thread_rng().fill_bytes(&mut sel_octets);
    let sel = SaltString::encode_b64(&sel_octets).map_err(|_| ErreurApi::interne())?;
    Argon2::default()
        .hash_password(mot_de_passe.as_bytes(), &sel)
        .map(|h| h.to_string())
        .map_err(|_| ErreurApi::interne())
}

/// Vérifie un mot de passe contre son hachage Argon2id.
pub fn verifier_mot_de_passe(mot_de_passe: &str, hash: &str) -> bool {
    match PasswordHash::new(hash) {
        Ok(hache) => Argon2::default()
            .verify_password(mot_de_passe.as_bytes(), &hache)
            .is_ok(),
        Err(_) => false,
    }
}

/// Vrai si le compte est temporairement verrouillé (trop de mots de passe faux).
pub fn connexion_verrouillee(etat: &Etat, courriel: &str) -> bool {
    let maintenant = std::time::Instant::now();
    let map = verrouiller_map(&etat.echecs_connexion);
    map.get(courriel)
        .is_some_and(|e| e.verrouille_jusqu.is_some_and(|j| maintenant < j))
}

/// Enregistre un mot de passe faux et verrouille le compte à partir du seuil.
pub fn enregistrer_echec_connexion(etat: &Etat, courriel: &str) {
    let maintenant = std::time::Instant::now();
    let mut map = verrouiller_map(&etat.echecs_connexion);
    let entree = map.entry(courriel.to_owned()).or_insert(EchecsConnexion {
        echecs: 0,
        dernier: maintenant,
        verrouille_jusqu: None,
    });
    if entree.verrouille_jusqu.is_some_and(|j| maintenant >= j) {
        entree.echecs = 0;
        entree.verrouille_jusqu = None;
    }
    if maintenant.duration_since(entree.dernier) > DUREE_VERROU {
        entree.echecs = 0;
        entree.verrouille_jusqu = None;
    }
    entree.dernier = maintenant;
    entree.echecs += 1;
    if entree.echecs >= SEUIL_VERROUILLAGE {
        entree.verrouille_jusqu = Some(maintenant + DUREE_VERROU);
    }
}

/// Oublie les échecs d'un compte (connexion réussie).
pub fn reinitialiser_echecs_connexion(etat: &Etat, courriel: &str) {
    let mut map = verrouiller_map(&etat.echecs_connexion);
    map.remove(courriel);
}

fn verrouiller_map(
    m: &Mutex<std::collections::HashMap<String, EchecsConnexion>>,
) -> std::sync::MutexGuard<'_, std::collections::HashMap<String, EchecsConnexion>> {
    match m.lock() {
        Ok(g) => g,
        Err(p) => p.into_inner(),
    }
}

fn normaliser_courriel(courriel: &str) -> String {
    courriel.trim().to_lowercase()
}

async fn creer_compte(
    State(etat): State<crate::Etat>,
    corps: Result<Json<CreationCompte>, JsonRejection>,
) -> Result<Response, ErreurApi> {
    let corps = crate::corps_json(corps)?;
    mot_de_passe_valide(&corps.mot_de_passe)?;
    let courriel = normaliser_courriel(&corps.courriel);
    if !ac_noyau::entreprise::courriel_valide(&courriel) {
        return Err(ErreurApi::invalide(
            "Adresse électronique invalide. Exemple attendu : contact@entreprise.bj.",
        ));
    }
    let nom = corps.nom.trim();
    if nom.is_empty() {
        return Err(ErreurApi::invalide("Indiquez votre nom."));
    }
    let hash = hacher_mot_de_passe(&corps.mot_de_passe)?;
    let insere = sqlx::query_as::<_, (i64,)>(
        "INSERT INTO comptes (courriel, nom, mot_de_passe_hash) VALUES ($1, $2, $3) RETURNING id",
    )
    .bind(&courriel)
    .bind(nom)
    .bind(&hash)
    .fetch_one(&etat.bdd)
    .await;
    let (compte_id,) = match insere {
        Ok(ligne) => ligne,
        Err(e) if crate::est_conflit_unicite(&e) => {
            return Err(ErreurApi::conflit(
                "COMPTE_EXISTANT",
                "Un compte existe déjà avec cette adresse électronique. Connectez-vous ou utilisez « Mot de passe oublié ».",
            ));
        }
        Err(e) => return Err(crate::erreur_bdd(e)),
    };
    envoyer_code_verification(&etat, compte_id, &courriel).await?;
    Ok((
        StatusCode::CREATED,
        Json(json!({
            "compte_id": compte_id,
            "message": "Compte créé. Un code de vérification à 6 chiffres vient de vous être envoyé par courriel."
        })),
    )
        .into_response())
}

async fn envoyer_code_verification(
    etat: &Etat,
    compte_id: i64,
    courriel: &str,
) -> Result<(), ErreurApi> {
    let code = creer_code(etat, compte_id, "VERIFICATION_COURRIEL").await?;
    let message = courriel::courriel_verification(courriel, &etat.courriel_expediteur, &code);
    etat.courriel
        .envoyer(&message)
        .map_err(|_| ErreurApi::interne())
}

async fn verifier_courriel(
    State(etat): State<crate::Etat>,
    corps: Result<Json<VerificationCourriel>, JsonRejection>,
) -> Result<Response, ErreurApi> {
    let corps = crate::corps_json(corps)?;
    let courriel = normaliser_courriel(&corps.courriel);
    let compte = compte_par_courriel(&etat, &courriel).await?;
    let Some((compte_id, _hash, _verifie)) = compte else {
        return Err(ErreurApi::invalide(CODE_INCORRECT));
    };
    verifier_code(&etat, compte_id, "VERIFICATION_COURRIEL", &corps.code).await?;
    sqlx::query("UPDATE comptes SET courriel_verifie_le = now() WHERE id = $1")
        .bind(compte_id)
        .execute(&etat.bdd)
        .await
        .map_err(crate::erreur_bdd)?;
    Ok(Json(json!({ "message": "Adresse électronique vérifiée. Vous pouvez maintenant vous connecter." })).into_response())
}

async fn renvoyer_code(
    State(etat): State<crate::Etat>,
    corps: Result<Json<DemandeCourriel>, JsonRejection>,
) -> Result<Response, ErreurApi> {
    let corps = crate::corps_json(corps)?;
    let courriel = normaliser_courriel(&corps.courriel);
    // Un code n'est renvoyé qu'à un compte non vérifié : la réponse reste la même dans
    // tous les cas, pour ne pas révéler si un compte existe.
    if let Some((compte_id, _hash, verifie)) = compte_par_courriel(&etat, &courriel).await? {
        if !verifie {
            envoyer_code_verification(&etat, compte_id, &courriel).await?;
        }
    }
    Ok((
        StatusCode::ACCEPTED,
        Json(json!({
            "message": "Si un compte non vérifié existe avec cette adresse, un nouveau code vient de lui être envoyé."
        })),
    )
        .into_response())
}

async fn ouvrir_session(
    State(etat): State<crate::Etat>,
    corps: Result<Json<Connexion>, JsonRejection>,
) -> Result<Response, ErreurApi> {
    let corps = crate::corps_json(corps)?;
    let courriel = normaliser_courriel(&corps.courriel);
    if connexion_verrouillee(&etat, &courriel) {
        return Err(ErreurApi::trop_de_tentatives(
            "TROP_DE_TENTATIVES",
            "Trop de tentatives de connexion. Pour protéger votre compte, réessayez dans 15 minutes.",
        ));
    }
    let Some((compte_id, hash, _verifie)) = compte_par_courriel(&etat, &courriel).await? else {
        return Err(ErreurApi::invalide(
            "Adresse électronique ou mot de passe incorrect.",
        ));
    };
    if !verifier_mot_de_passe(&corps.mot_de_passe, &hash) {
        enregistrer_echec_connexion(&etat, &courriel);
        return Err(ErreurApi::invalide(
            "Adresse électronique ou mot de passe incorrect.",
        ));
    }
    reinitialiser_echecs_connexion(&etat, &courriel);
    let ligne: Option<(i64, String, bool)> = sqlx::query_as(
        "SELECT id, nom, (courriel_verifie_le IS NOT NULL) FROM comptes WHERE id = $1 AND actif",
    )
    .bind(compte_id)
    .fetch_optional(&etat.bdd)
    .await
    .map_err(crate::erreur_bdd)?;
    let Some((id, nom, verifie)) = ligne else {
        return Err(ErreurApi::interdit(
            "COMPTE_DESACTIVE",
            "Ce compte est désactivé. Contactez l'assistance.",
        ));
    };
    if !verifie {
        return Err(ErreurApi::interdit(
            "COURRIEL_NON_VERIFIE",
            "Vérifiez d'abord votre adresse électronique : un code à 6 chiffres vous a été envoyé.",
        ));
    }
    let jeton = jetons::jeton_opaque();
    let hash_jeton = jetons::empreinte_sha256(&jeton);
    let expire_le = Utc::now() + chrono::Duration::hours(24);
    sqlx::query("INSERT INTO sessions (compte_id, jeton_hash, expire_le) VALUES ($1, $2, $3)")
        .bind(id)
        .bind(&hash_jeton)
        .bind(expire_le)
        .execute(&etat.bdd)
        .await
        .map_err(crate::erreur_bdd)?;
    Ok(Json(json!({
        "jeton": jeton,
        "expire_le": expire_le,
        "compte": CompteResume { id, courriel, nom }
    }))
    .into_response())
}

async fn fermer_session(
    State(etat): State<crate::Etat>,
    en_tetes: HeaderMap,
) -> Result<StatusCode, ErreurApi> {
    let compte = crate::authentifier(&etat, &en_tetes).await?;
    sqlx::query("DELETE FROM sessions WHERE jeton_hash = $1")
        .bind(&compte.jeton_hash)
        .execute(&etat.bdd)
        .await
        .map_err(crate::erreur_bdd)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn mot_de_passe_oublie(
    State(etat): State<crate::Etat>,
    corps: Result<Json<DemandeCourriel>, JsonRejection>,
) -> Result<Response, ErreurApi> {
    let corps = crate::corps_json(corps)?;
    let courriel = normaliser_courriel(&corps.courriel);
    if let Some((compte_id, _hash, _verifie)) = compte_par_courriel(&etat, &courriel).await? {
        let code = creer_code(&etat, compte_id, "REINITIALISATION_MOT_DE_PASSE").await?;
        let message =
            courriel::courriel_reinitialisation(&courriel, &etat.courriel_expediteur, &code);
        etat.courriel
            .envoyer(&message)
            .map_err(|_| ErreurApi::interne())?;
    }
    Ok((
        StatusCode::ACCEPTED,
        Json(json!({
            "message": "Si un compte existe avec cette adresse, un code de réinitialisation vient de lui être envoyé."
        })),
    )
        .into_response())
}

async fn reinitialiser_mot_de_passe(
    State(etat): State<crate::Etat>,
    corps: Result<Json<Reinitialisation>, JsonRejection>,
) -> Result<Response, ErreurApi> {
    let corps = crate::corps_json(corps)?;
    mot_de_passe_valide(&corps.nouveau_mot_de_passe)?;
    let courriel = normaliser_courriel(&corps.courriel);
    let Some((compte_id, _hash, _verifie)) = compte_par_courriel(&etat, &courriel).await? else {
        return Err(ErreurApi::invalide(CODE_INCORRECT));
    };
    verifier_code(
        &etat,
        compte_id,
        "REINITIALISATION_MOT_DE_PASSE",
        &corps.code,
    )
    .await?;
    let nouveau_hash = hacher_mot_de_passe(&corps.nouveau_mot_de_passe)?;
    sqlx::query("UPDATE comptes SET mot_de_passe_hash = $1 WHERE id = $2")
        .bind(&nouveau_hash)
        .bind(compte_id)
        .execute(&etat.bdd)
        .await
        .map_err(crate::erreur_bdd)?;
    // Toutes les sessions existantes sont fermées, par prudence.
    sqlx::query("DELETE FROM sessions WHERE compte_id = $1")
        .bind(compte_id)
        .execute(&etat.bdd)
        .await
        .map_err(crate::erreur_bdd)?;
    reinitialiser_echecs_connexion(&etat, &courriel);
    Ok(Json(json!({
        "message": "Mot de passe réinitialisé. Vous pouvez maintenant vous connecter."
    }))
    .into_response())
}

/// Identifiant, hachage du mot de passe et état de vérification d'un compte.
async fn compte_par_courriel(
    etat: &Etat,
    courriel: &str,
) -> Result<Option<(i64, String, bool)>, ErreurApi> {
    sqlx::query_as::<_, (i64, String, bool)>(
        "SELECT id, mot_de_passe_hash, (courriel_verifie_le IS NOT NULL) FROM comptes WHERE courriel = $1",
    )
    .bind(courriel)
    .fetch_optional(&etat.bdd)
    .await
    .map_err(crate::erreur_bdd)
}

async fn creer_code(etat: &Etat, compte_id: i64, categorie: &str) -> Result<String, ErreurApi> {
    let code = jetons::code_six_chiffres();
    let hash = jetons::empreinte_sha256(&code);
    let expire_le = Utc::now() + chrono::Duration::minutes(courriel::DUREE_CODE_MINUTES);
    sqlx::query(
        "UPDATE codes_verification SET utilise_le = now()
         WHERE compte_id = $1 AND categorie = $2 AND utilise_le IS NULL",
    )
    .bind(compte_id)
    .bind(categorie)
    .execute(&etat.bdd)
    .await
    .map_err(crate::erreur_bdd)?;
    sqlx::query(
        "INSERT INTO codes_verification (compte_id, categorie, code_hash, expire_le)
         VALUES ($1, $2, $3, $4)",
    )
    .bind(compte_id)
    .bind(categorie)
    .bind(&hash)
    .bind(expire_le)
    .execute(&etat.bdd)
    .await
    .map_err(crate::erreur_bdd)?;
    Ok(code)
}

async fn verifier_code(
    etat: &Etat,
    compte_id: i64,
    categorie: &str,
    code: &str,
) -> Result<(), ErreurApi> {
    let ligne: Option<(i64, String, chrono::DateTime<Utc>, i32)> = sqlx::query_as(
        "SELECT id, code_hash, expire_le, tentatives FROM codes_verification
         WHERE compte_id = $1 AND categorie = $2 AND utilise_le IS NULL
         ORDER BY id DESC LIMIT 1",
    )
    .bind(compte_id)
    .bind(categorie)
    .fetch_optional(&etat.bdd)
    .await
    .map_err(crate::erreur_bdd)?;
    let Some((id, code_hash, expire_le, tentatives)) = ligne else {
        return Err(ErreurApi::invalide(CODE_INCORRECT));
    };
    if tentatives >= 5 {
        return Err(ErreurApi::invalide(
            "Ce code a été annulé après plusieurs tentatives. Demandez-en un nouveau.",
        ));
    }
    if jetons::empreinte_sha256(code) != code_hash {
        sqlx::query("UPDATE codes_verification SET tentatives = tentatives + 1 WHERE id = $1")
            .bind(id)
            .execute(&etat.bdd)
            .await
            .map_err(crate::erreur_bdd)?;
        return Err(ErreurApi::invalide(CODE_INCORRECT));
    }
    if expire_le < Utc::now() {
        return Err(ErreurApi::invalide(
            "Ce code a expiré. Demandez-en un nouveau.",
        ));
    }
    sqlx::query("UPDATE codes_verification SET utilise_le = now() WHERE id = $1")
        .bind(id)
        .execute(&etat.bdd)
        .await
        .map_err(crate::erreur_bdd)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn politique_de_mot_de_passe() {
        assert!(mot_de_passe_valide("motdepasse10").is_ok());
        assert!(mot_de_passe_valide("0123456789").is_ok()); // 10 exactement
        assert!(mot_de_passe_valide("tropcourt9").is_err()); // 9
        assert!(mot_de_passe_valide(&"a".repeat(201)).is_err());
        assert!(mot_de_passe_valide(&"a".repeat(200)).is_ok()); // limite exacte
    }

    #[test]
    fn hachage_argon2id() {
        let hash = hacher_mot_de_passe("un-bon-mot-de-passe").unwrap();
        // Le préfixe garantit l'algorithme Argon2id (et non Argon2i / Argon2d).
        assert!(hash.starts_with("$argon2id$"));
        assert!(verifier_mot_de_passe("un-bon-mot-de-passe", &hash));
        assert!(!verifier_mot_de_passe("mauvais-mot-de-passe", &hash));
        assert!(!verifier_mot_de_passe("un-bon-mot-de-passe", "hash-casse"));
    }

    #[test]
    fn hachages_differents_pour_le_meme_mot_de_passe() {
        let a = hacher_mot_de_passe("mot-de-passe-long").unwrap();
        let b = hacher_mot_de_passe("mot-de-passe-long").unwrap();
        assert_ne!(a, b, "le sel aléatoire rend chaque hachage unique");
    }

    #[test]
    fn verrouillage_apres_dix_echecs() {
        let etat = crate::etat_de_test();
        assert!(!connexion_verrouillee(&etat, "a@b.bj"));
        for _ in 0..SEUIL_VERROUILLAGE {
            enregistrer_echec_connexion(&etat, "a@b.bj");
        }
        assert!(connexion_verrouillee(&etat, "a@b.bj"));
        assert!(!connexion_verrouillee(&etat, "autre@b.bj"));
        reinitialiser_echecs_connexion(&etat, "a@b.bj");
        assert!(!connexion_verrouillee(&etat, "a@b.bj"));
    }

    #[test]
    fn courriel_normalise() {
        assert_eq!(
            normaliser_courriel("  Marie@Exemple.BJ "),
            "marie@exemple.bj"
        );
    }
}
