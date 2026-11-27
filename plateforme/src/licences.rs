//! Licences signées Ed25519 (ADR 0004) : émission, renouvellement, transfert de poste.
//!
//! Charge utile : [`ac_licence::Licence`] (le type exact lu par l'application).
//! Format du jeton : `base64url(charge_utile_json) + "." + base64url(signature)` — voir
//! [`crate::jetons`]. La clé privée ne quitte jamais le serveur ; la clé publique est
//! embarquée dans l'application.
//!
//! Statuts d'entreprise (ADR 0009 §4) : `SUSPENDUE` ou `REJETEE` → émission et
//! renouvellement refusés ; si le réglage « vérification préalable » est actif, la
//! licence n'est délivrée qu'à une entreprise `VERIFIEE`.

use axum::{
    extract::{rejection::JsonRejection, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use serde::Deserialize;
use serde_json::json;

use ac_licence::Licence;

use crate::{erreur::ErreurApi, Etat};

/// Offre de licence en V1.
pub const OFFRE_GRATUITE: &str = "GRATUITE";

/// Demande d'émission ou de renouvellement de licence.
#[derive(Debug, Deserialize)]
pub struct DemandeLicence {
    /// IFU de l'entreprise (doit appartenir au compte connecté).
    pub ifu: String,
    /// Empreinte du poste (hash SHA-256 calculé par l'application, jamais un identifiant brut).
    pub empreinte_poste: String,
}

/// Demande de transfert de licence vers un nouveau poste.
#[derive(Debug, Deserialize)]
pub struct DemandeTransfert {
    /// IFU de l'entreprise.
    pub ifu: String,
    /// Empreinte du poste quitté.
    pub ancienne_empreinte: String,
    /// Empreinte du nouveau poste.
    pub nouvelle_empreinte: String,
}

/// Routes `/v1` des licences.
pub fn routes() -> Router<Etat> {
    Router::new()
        .route("/v1/licences", post(emettre_licence))
        .route("/v1/licences/renouveler", post(renouveler_licence))
        .route("/v1/licences/transferer", post(transferer_licence))
}

/// Vrai si l'empreinte d'un poste est bien formée (hash en base64url ou hexadécimal).
pub fn empreinte_valide(empreinte: &str) -> bool {
    !empreinte.is_empty()
        && empreinte.len() <= 200
        && empreinte
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_'))
}

struct Entreprise {
    id: i64,
    ifu: String,
    statut: String,
    motif: Option<String>,
}

async fn entreprise_par_ifu(
    etat: &Etat,
    compte_id: i64,
    ifu: &str,
) -> Result<Entreprise, ErreurApi> {
    let ligne: Option<(i64, String, String, Option<String>)> = sqlx::query_as(
        "SELECT id, ifu, statut, motif FROM entreprises WHERE ifu = $1 AND compte_id = $2",
    )
    .bind(ifu)
    .bind(compte_id)
    .fetch_optional(&etat.bdd)
    .await
    .map_err(crate::erreur_bdd)?;
    let Some((id, ifu, statut, motif)) = ligne else {
        return Err(ErreurApi::introuvable(
            "Aucune entreprise de votre compte n'est inscrite avec cet IFU.",
        ));
    };
    Ok(Entreprise {
        id,
        ifu,
        statut,
        motif,
    })
}

/// Refuse l'émission et le renouvellement pour une entreprise suspendue ou rejetée.
fn exiger_statut_licence(entreprise: &Entreprise) -> Result<(), ErreurApi> {
    match entreprise.statut.as_str() {
        "SUSPENDUE" => Err(ErreurApi::interdit(
            "ENTREPRISE_SUSPENDUE",
            format!(
                "Votre entreprise est suspendue : la licence ne peut pas être délivrée ni renouvelée. Motif : {}",
                entreprise.motif.clone().unwrap_or_else(|| "non précisé".to_owned())
            ),
        )),
        "REJETEE" => Err(ErreurApi::interdit(
            "ENTREPRISE_REJETEE",
            format!(
                "Votre entreprise est rejetée : la licence ne peut pas être délivrée ni renouvelée. Motif : {}",
                entreprise.motif.clone().unwrap_or_else(|| "non précisé".to_owned())
            ),
        )),
        _ => Ok(()),
    }
}

/// Vrai si le réglage « vérification préalable » est actif (défaut : non).
async fn verification_prealable(etat: &Etat) -> Result<bool, ErreurApi> {
    let ligne: Option<(String,)> =
        sqlx::query_as("SELECT valeur FROM parametres WHERE cle = 'verification_prealable'")
            .fetch_optional(&etat.bdd)
            .await
            .map_err(crate::erreur_bdd)?;
    Ok(ligne.is_some_and(|(v,)| v == "true"))
}

/// Refuse la tant que l'entreprise n'est pas `VERIFIEE` si la vérification préalable est active.
async fn exiger_verification(etat: &Etat, entreprise: &Entreprise) -> Result<(), ErreurApi> {
    if entreprise.statut == "VERIFIEE" || !verification_prealable(etat).await? {
        Ok(())
    } else {
        Err(ErreurApi::interdit(
            "VERIFICATION_EN_ATTENTE",
            "Votre entreprise est en attente de vérification : la licence sera délivrée après validation par l'administrateur.",
        ))
    }
}

async fn licence_active(
    etat: &Etat,
    entreprise_id: i64,
    empreinte: &str,
) -> Result<Option<i64>, ErreurApi> {
    let ligne: Option<(i64,)> = sqlx::query_as(
        "SELECT id FROM licences
         WHERE entreprise_id = $1 AND empreinte_poste = $2 AND statut = 'ACTIVE'
         ORDER BY id DESC LIMIT 1",
    )
    .bind(entreprise_id)
    .bind(empreinte)
    .fetch_optional(&etat.bdd)
    .await
    .map_err(crate::erreur_bdd)?;
    Ok(ligne.map(|(id,)| id))
}

async fn licence_active_autre_poste(
    etat: &Etat,
    entreprise_id: i64,
    empreinte: &str,
) -> Result<Option<String>, ErreurApi> {
    let ligne: Option<(String,)> = sqlx::query_as(
        "SELECT empreinte_poste FROM licences
         WHERE entreprise_id = $1 AND empreinte_poste <> $2 AND statut = 'ACTIVE'
         LIMIT 1",
    )
    .bind(entreprise_id)
    .bind(empreinte)
    .fetch_optional(&etat.bdd)
    .await
    .map_err(crate::erreur_bdd)?;
    Ok(ligne.map(|(e,)| e))
}

/// Insère une nouvelle licence active et signe son jeton.
async fn creer_licence_signee(
    etat: &Etat,
    compte_id: i64,
    entreprise: &Entreprise,
    empreinte: &str,
) -> Result<serde_json::Value, ErreurApi> {
    let emise_le = crate::aujourdhui();
    let expire_le = emise_le
        .checked_add_months(chrono::Months::new(ac_licence::DUREE_MOIS))
        .ok_or_else(ErreurApi::interne)?;
    sqlx::query(
        "INSERT INTO licences (entreprise_id, empreinte_poste, emise_le, expire_le, offre)
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(entreprise.id)
    .bind(empreinte)
    .bind(emise_le)
    .bind(expire_le)
    .bind(OFFRE_GRATUITE)
    .execute(&etat.bdd)
    .await
    .map_err(crate::erreur_bdd)?;
    let charge = Licence {
        format: 1,
        compte_id: compte_id.to_string(),
        ifu: entreprise.ifu.clone(),
        empreinte_poste: empreinte.to_owned(),
        emise_le,
        expire_le,
        offre: OFFRE_GRATUITE.to_owned(),
    };
    let jeton = etat
        .signataire
        .signer_licence(&charge)
        .map_err(|_| ErreurApi::interne())?;
    Ok(json!({
        "jeton": jeton,
        "emise_le": emise_le,
        "expire_le": expire_le,
        "offre": OFFRE_GRATUITE,
    }))
}

fn controler_empreintes(empreintes: &[&str]) -> Result<(), ErreurApi> {
    for e in empreintes {
        if !empreinte_valide(e) {
            return Err(ErreurApi::invalide(
                "Empreinte du poste invalide : elle doit être un hash (hexadécimal ou base64url) de 200 caractères au plus.",
            ));
        }
    }
    Ok(())
}

async fn emettre_licence(
    State(etat): State<Etat>,
    en_tetes: HeaderMap,
    corps: Result<Json<DemandeLicence>, JsonRejection>,
) -> Result<Response, ErreurApi> {
    let compte = crate::authentifier(&etat, &en_tetes).await?;
    let corps = crate::corps_json(corps)?;
    controler_empreintes(&[&corps.empreinte_poste])?;
    let entreprise = entreprise_par_ifu(&etat, compte.compte_id, corps.ifu.trim()).await?;
    exiger_statut_licence(&entreprise)?;
    exiger_verification(&etat, &entreprise).await?;
    if licence_active(&etat, entreprise.id, &corps.empreinte_poste)
        .await?
        .is_some()
    {
        return Err(ErreurApi::conflit(
            "LICENCE_DEJA_ACTIVE",
            "Une licence est déjà active pour ce poste. Utilisez le renouvellement.",
        ));
    }
    if licence_active_autre_poste(&etat, entreprise.id, &corps.empreinte_poste)
        .await?
        .is_some()
    {
        return Err(ErreurApi::conflit(
            "AUTRE_POSTE_ACTIF",
            "Une licence est déjà active sur un autre poste. Utilisez le transfert de poste.",
        ));
    }
    let reponse =
        creer_licence_signee(&etat, compte.compte_id, &entreprise, &corps.empreinte_poste).await?;
    Ok((StatusCode::CREATED, Json(reponse)).into_response())
}

async fn renouveler_licence(
    State(etat): State<Etat>,
    en_tetes: HeaderMap,
    corps: Result<Json<DemandeLicence>, JsonRejection>,
) -> Result<Response, ErreurApi> {
    let compte = crate::authentifier(&etat, &en_tetes).await?;
    let corps = crate::corps_json(corps)?;
    controler_empreintes(&[&corps.empreinte_poste])?;
    let entreprise = entreprise_par_ifu(&etat, compte.compte_id, corps.ifu.trim()).await?;
    exiger_statut_licence(&entreprise)?;
    exiger_verification(&etat, &entreprise).await?;
    let existante = sqlx::query_as::<_, (i64,)>(
        "SELECT id FROM licences WHERE entreprise_id = $1 AND empreinte_poste = $2
         ORDER BY id DESC LIMIT 1",
    )
    .bind(entreprise.id)
    .bind(&corps.empreinte_poste)
    .fetch_optional(&etat.bdd)
    .await
    .map_err(crate::erreur_bdd)?;
    if existante.is_none() {
        return Err(ErreurApi::introuvable(
            "Aucune licence à renouveler pour ce poste. Émettez d'abord une licence.",
        ));
    }
    if licence_active_autre_poste(&etat, entreprise.id, &corps.empreinte_poste)
        .await?
        .is_some()
    {
        return Err(ErreurApi::conflit(
            "AUTRE_POSTE_ACTIF",
            "Une licence est déjà active sur un autre poste. Utilisez le transfert de poste.",
        ));
    }
    sqlx::query(
        "UPDATE licences SET statut = 'REMPLACEE', motif = 'renouvellement'
         WHERE entreprise_id = $1 AND empreinte_poste = $2 AND statut = 'ACTIVE'",
    )
    .bind(entreprise.id)
    .bind(&corps.empreinte_poste)
    .execute(&etat.bdd)
    .await
    .map_err(crate::erreur_bdd)?;
    let reponse =
        creer_licence_signee(&etat, compte.compte_id, &entreprise, &corps.empreinte_poste).await?;
    Ok(Json(reponse).into_response())
}

async fn transferer_licence(
    State(etat): State<Etat>,
    en_tetes: HeaderMap,
    corps: Result<Json<DemandeTransfert>, JsonRejection>,
) -> Result<Response, ErreurApi> {
    let compte = crate::authentifier(&etat, &en_tetes).await?;
    let corps = crate::corps_json(corps)?;
    controler_empreintes(&[&corps.ancienne_empreinte, &corps.nouvelle_empreinte])?;
    if corps.ancienne_empreinte == corps.nouvelle_empreinte {
        return Err(ErreurApi::invalide(
            "La nouvelle empreinte doit être différente de l'ancienne.",
        ));
    }
    let entreprise = entreprise_par_ifu(&etat, compte.compte_id, corps.ifu.trim()).await?;
    exiger_statut_licence(&entreprise)?;
    exiger_verification(&etat, &entreprise).await?;
    if licence_active(&etat, entreprise.id, &corps.ancienne_empreinte)
        .await?
        .is_none()
    {
        return Err(ErreurApi::introuvable(
            "Aucune licence active sur l'ancien poste : rien à transférer.",
        ));
    }

    // Limite de transferts par an (paramétrable, 2 par défaut).
    let annee = crate::aujourdhui().year();
    let (deja_faits,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM transferts_poste WHERE entreprise_id = $1 AND annee = $2",
    )
    .bind(entreprise.id)
    .bind(annee)
    .fetch_one(&etat.bdd)
    .await
    .map_err(crate::erreur_bdd)?;
    let ligne: Option<(String,)> =
        sqlx::query_as("SELECT valeur FROM parametres WHERE cle = 'transferts_poste_par_an'")
            .fetch_optional(&etat.bdd)
            .await
            .map_err(crate::erreur_bdd)?;
    let limite: i64 = ligne
        .and_then(|(v,)| v.parse::<i64>().ok())
        .filter(|l| *l >= 1)
        .unwrap_or(etat.transferts_poste_par_an);
    if deja_faits >= limite {
        return Err(ErreurApi::interdit(
            "LIMITE_TRANSFERTS",
            format!(
                "Limite de transferts atteinte pour cette année ({limite} par an). Contactez l'assistance si vous avez besoin d'un transfert supplémentaire."
            ),
        ));
    }

    sqlx::query(
        "INSERT INTO transferts_poste (entreprise_id, ancienne_empreinte, nouvelle_empreinte, annee)
         VALUES ($1, $2, $3, $4)",
    )
    .bind(entreprise.id)
    .bind(&corps.ancienne_empreinte)
    .bind(&corps.nouvelle_empreinte)
    .bind(annee)
    .execute(&etat.bdd)
    .await
    .map_err(crate::erreur_bdd)?;
    sqlx::query(
        "UPDATE licences SET statut = 'REMPLACEE', motif = 'transfert de poste'
         WHERE entreprise_id = $1 AND empreinte_poste = $2 AND statut = 'ACTIVE'",
    )
    .bind(entreprise.id)
    .bind(&corps.ancienne_empreinte)
    .execute(&etat.bdd)
    .await
    .map_err(crate::erreur_bdd)?;
    let mut reponse =
        creer_licence_signee(&etat, compte.compte_id, &entreprise, &corps.nouvelle_empreinte)
            .await?;
    reponse["transferts_restants"] = json!(limite - deja_faits - 1);
    Ok(Json(reponse).into_response())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empreintes_poste() {
        assert!(empreinte_valide("a1b2c3d4e5f6"));
        assert!(empreinte_valide(&"f".repeat(200))); // limite exacte
        assert!(empreinte_valide("base64_url-avec_tirets"));
        assert!(!empreinte_valide(""));
        assert!(!empreinte_valide(&"f".repeat(201)));
        assert!(!empreinte_valide("avec espace"));
        assert!(!empreinte_valide("accentsé"));
    }
}
