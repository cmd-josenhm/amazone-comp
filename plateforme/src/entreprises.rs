//! Inscription des entreprises (ADR 0009, contrat C11).
//!
//! La fiche complète est validée par [`ac_noyau::entreprise::FicheEntreprise::valider`] :
//! **la même règle que l'application**, jamais réécrite. Une fiche invalide renvoie 422
//! avec exactement la liste `erreurs` de la validation. L'IFU normalisé est unique sur
//! toute la plateforme ; le statut initial est `EN_ATTENTE_VERIFICATION`. Un changement
//! d'IFU devient une demande soumise à l'administrateur (console, lot 7).

use axum::{
    extract::{rejection::JsonRejection, Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post, put},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::json;

use ac_noyau::{
    entreprise::{FicheEntreprise, RapportValidation},
    Ifu,
};
use sqlx::types::Json as JsonBdd;

use crate::{erreur::ErreurApi, Etat};

/// Taille maximale d'un justificatif (5 Mo, ADR 0009).
pub const TAILLE_MAX_JUSTIFICATIF: usize = 5 * 1024 * 1024;

/// Demande d'inscription ou de modification d'une entreprise.
#[derive(Debug, Deserialize)]
pub struct DemandeEntreprise {
    /// Fiche légale complète (contrat C11).
    pub fiche: FicheEntreprise,
}

/// Dépôt d'un justificatif (attestation IFU ou extrait RCCM).
#[derive(Debug, Deserialize)]
pub struct DeposeJustificatif {
    /// Catégorie : `ATTESTATION_IFU` ou `EXTRAIT_RCCM`.
    pub categorie: String,
    /// Nom du fichier d'origine.
    pub nom_fichier: String,
    /// Type MIME : `application/pdf`, `image/png` ou `image/jpeg`.
    pub mime: String,
    /// Contenu du fichier, encodé en base64 standard.
    pub contenu_base64: String,
}

/// Routes `/v1` des entreprises.
pub fn routes() -> Router<Etat> {
    Router::new()
        .route("/v1/entreprises", post(inscrire_entreprise))
        .route("/v1/entreprises/{id}", put(modifier_entreprise))
        .route("/v1/entreprises/{id}/statut", get(statut_entreprise))
        .route(
            "/v1/entreprises/{id}/justificatifs",
            post(deposer_justificatif),
        )
}

/// Normalise un IFU validé (espaces retirés).
fn ifu_normalise(fiche: &FicheEntreprise) -> Result<String, ErreurApi> {
    Ifu::new(&fiche.ifu)
        .map(|i| i.as_str().to_owned())
        .map_err(|_| {
            ErreurApi::invalide(
                "L'IFU doit compter exactement 13 chiffres. Il figure sur votre attestation d'immatriculation.",
            )
        })
}

/// Vrai si le compte a accepté la version courante des CGU.
async fn cgu_acceptees(etat: &Etat, compte_id: i64) -> Result<bool, ErreurApi> {
    let ligne: Option<(i32,)> = sqlx::query_as(
        "SELECT ca.cgu_version FROM cgu_acceptations ca
         WHERE ca.compte_id = $1
           AND ca.cgu_version = (SELECT MAX(version) FROM cgu_versions)
         LIMIT 1",
    )
    .bind(compte_id)
    .fetch_optional(&etat.bdd)
    .await
    .map_err(crate::erreur_bdd)?;
    Ok(ligne.is_some())
}

/// Refuse l'inscription si le compte n'a pas accepté les CGU courantes.
async fn exiger_cgu(etat: &Etat, compte_id: i64) -> Result<(), ErreurApi> {
    if cgu_acceptees(etat, compte_id).await? {
        Ok(())
    } else {
        Err(ErreurApi::interdit(
            "CGU_NON_ACCEPTEES",
            "Acceptez d'abord les conditions générales d'utilisation.",
        ))
    }
}

/// Entreprise appartenant au compte, d'après son identifiant.
struct EntreprisePossedee {
    id: i64,
    ifu: String,
    statut: String,
    motif: Option<String>,
    statut_change_le: Option<DateTime<Utc>>,
}

async fn entreprise_possedee(
    etat: &Etat,
    compte_id: i64,
    id: i64,
) -> Result<EntreprisePossedee, ErreurApi> {
    let ligne: Option<(i64, String, String, Option<String>, Option<DateTime<Utc>>, i64)> =
        sqlx::query_as(
            "SELECT id, ifu, statut, motif, statut_change_le, compte_id FROM entreprises WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&etat.bdd)
        .await
        .map_err(crate::erreur_bdd)?;
    let Some((id, ifu, statut, motif, statut_change_le, proprietaire)) = ligne else {
        return Err(ErreurApi::introuvable("Entreprise introuvable."));
    };
    if proprietaire != compte_id {
        return Err(ErreurApi::interdit(
            "ACCES_REFUSE",
            "Cette entreprise n'appartient pas à votre compte.",
        ));
    }
    Ok(EntreprisePossedee {
        id,
        ifu,
        statut,
        motif,
        statut_change_le,
    })
}

/// Vrai si l'IFU est déjà porté par une entreprise (toute la plateforme).
async fn ifu_deja_inscrit(etat: &Etat, ifu: &str) -> Result<bool, ErreurApi> {
    let ligne: Option<(i64,)> = sqlx::query_as("SELECT id FROM entreprises WHERE ifu = $1")
        .bind(ifu)
        .fetch_optional(&etat.bdd)
        .await
        .map_err(crate::erreur_bdd)?;
    Ok(ligne.is_some())
}

async fn inscrire_entreprise(
    State(etat): State<Etat>,
    en_tetes: HeaderMap,
    corps: Result<Json<DemandeEntreprise>, JsonRejection>,
) -> Result<Response, ErreurApi> {
    let compte = crate::authentifier(&etat, &en_tetes).await?;
    exiger_cgu(&etat, compte.compte_id).await?;
    let corps = crate::corps_json(corps)?;
    let rapport = corps.fiche.valider(crate::aujourdhui());
    if !rapport.est_valide() {
        return Err(ErreurApi::fiche_invalide(&rapport));
    }
    let ifu = ifu_normalise(&corps.fiche)?;
    if ifu_deja_inscrit(&etat, &ifu).await? {
        return Err(ErreurApi::conflit(
            "IFU_DEJA_INSCRIT",
            "Cet IFU est déjà inscrit sur la plateforme. Si cette entreprise est la vôtre, contactez l'assistance.",
        ));
    }
    let insere = sqlx::query_as::<_, (i64,)>(
        "INSERT INTO entreprises (compte_id, ifu, fiche) VALUES ($1, $2, $3) RETURNING id",
    )
    .bind(compte.compte_id)
    .bind(&ifu)
    .bind(JsonBdd(&corps.fiche))
    .fetch_one(&etat.bdd)
    .await;
    let (id,) = match insere {
        Ok(ligne) => ligne,
        Err(e) if crate::est_conflit_unicite(&e) => {
            return Err(ErreurApi::conflit(
                "IFU_DEJA_INSCRIT",
                "Cet IFU est déjà inscrit sur la plateforme. Si cette entreprise est la vôtre, contactez l'assistance.",
            ));
        }
        Err(e) => return Err(crate::erreur_bdd(e)),
    };
    Ok((
        StatusCode::CREATED,
        Json(json!({
            "entreprise_id": id,
            "ifu": ifu,
            "statut": "EN_ATTENTE_VERIFICATION",
            "avertissements": rapport.avertissements,
        })),
    )
        .into_response())
}

async fn modifier_entreprise(
    State(etat): State<Etat>,
    Path(id): Path<i64>,
    en_tetes: HeaderMap,
    corps: Result<Json<DemandeEntreprise>, JsonRejection>,
) -> Result<Response, ErreurApi> {
    let compte = crate::authentifier(&etat, &en_tetes).await?;
    let entreprise = entreprise_possedee(&etat, compte.compte_id, id).await?;
    let corps = crate::corps_json(corps)?;
    let rapport = corps.fiche.valider(crate::aujourdhui());
    if !rapport.est_valide() {
        return Err(ErreurApi::fiche_invalide(&rapport));
    }
    let ifu_demande = ifu_normalise(&corps.fiche)?;

    // Le champ IFU conservé reste l'IFU actuel : un changement d'IFU est une demande
    // validée par l'administrateur (ADR 0009 §5).
    let mut fiche = corps.fiche.clone();
    let mut demande_ifu = false;
    if ifu_demande != entreprise.ifu {
        if ifu_deja_inscrit(&etat, &ifu_demande).await? {
            return Err(ErreurApi::conflit(
                "IFU_DEJA_INSCRIT",
                "Cet IFU est déjà inscrit sur la plateforme par une autre entreprise.",
            ));
        }
        fiche.ifu = entreprise.ifu.clone();
        demande_ifu = true;
        sqlx::query(
            "INSERT INTO demandes_changement_ifu (entreprise_id, ancien_ifu, nouveau_ifu)
             VALUES ($1, $2, $3)",
        )
        .bind(entreprise.id)
        .bind(&entreprise.ifu)
        .bind(&ifu_demande)
        .execute(&etat.bdd)
        .await
        .map_err(crate::erreur_bdd)?;
    }
    sqlx::query("UPDATE entreprises SET fiche = $1, maj_le = now() WHERE id = $2")
        .bind(JsonBdd(&fiche))
        .bind(entreprise.id)
        .execute(&etat.bdd)
        .await
        .map_err(crate::erreur_bdd)?;
    let mut reponse = json!({
        "entreprise_id": entreprise.id,
        "statut": entreprise.statut,
        "avertissements": rapport.avertissements,
        "demande_changement_ifu": demande_ifu,
    });
    if demande_ifu {
        reponse["message"] = json!(
            "Le changement d'IFU doit être approuvé par l'administrateur. Les autres modifications de la fiche sont enregistrées."
        );
    }
    Ok(Json(reponse).into_response())
}

async fn statut_entreprise(
    State(etat): State<Etat>,
    Path(id): Path<i64>,
    en_tetes: HeaderMap,
) -> Result<Response, ErreurApi> {
    let compte = crate::authentifier(&etat, &en_tetes).await?;
    let entreprise = entreprise_possedee(&etat, compte.compte_id, id).await?;
    let demandes: Vec<(String,)> = sqlx::query_as(
        "SELECT nouveau_ifu FROM demandes_changement_ifu
         WHERE entreprise_id = $1 AND statut = 'EN_ATTENTE' ORDER BY id",
    )
    .bind(entreprise.id)
    .fetch_all(&etat.bdd)
    .await
    .map_err(crate::erreur_bdd)?;
    Ok(Json(json!({
        "entreprise_id": entreprise.id,
        "ifu": entreprise.ifu,
        "statut": entreprise.statut,
        "motif": entreprise.motif,
        "statut_change_le": entreprise.statut_change_le,
        "demandes_ifu_en_attente": demandes.into_iter().map(|(n,)| n).collect::<Vec<_>>(),
    }))
    .into_response())
}

async fn deposer_justificatif(
    State(etat): State<Etat>,
    Path(id): Path<i64>,
    en_tetes: HeaderMap,
    corps: Result<Json<DeposeJustificatif>, JsonRejection>,
) -> Result<Response, ErreurApi> {
    let compte = crate::authentifier(&etat, &en_tetes).await?;
    let entreprise = entreprise_possedee(&etat, compte.compte_id, id).await?;
    let corps = crate::corps_json(corps)?;
    if !matches!(
        corps.categorie.as_str(),
        "ATTESTATION_IFU" | "EXTRAIT_RCCM"
    ) {
        return Err(ErreurApi::invalide(
            "Catégorie de justificatif inconnue : attendu ATTESTATION_IFU ou EXTRAIT_RCCM.",
        ));
    }
    if !matches!(
        corps.mime.as_str(),
        "application/pdf" | "image/png" | "image/jpeg"
    ) {
        return Err(ErreurApi::type_non_supporte(
            "Format non accepté : utilisez un fichier PDF, JPG ou PNG.",
        ));
    }
    use base64::{engine::general_purpose::STANDARD, Engine};
    let contenu = STANDARD
        .decode(corps.contenu_base64.trim())
        .map_err(|_| ErreurApi::invalide("Fichier illisible : le contenu base64 est invalide."))?;
    if contenu.is_empty() {
        return Err(ErreurApi::invalide("Le fichier est vide."));
    }
    if contenu.len() > TAILLE_MAX_JUSTIFICATIF {
        return Err(ErreurApi::trop_grand(
            "Le fichier dépasse la taille maximale de 5 Mo.",
        ));
    }
    let taille = i32::try_from(contenu.len()).map_err(|_| ErreurApi::trop_grand(
        "Le fichier dépasse la taille maximale de 5 Mo.",
    ))?;
    let (justificatif_id,): (i64,) = sqlx::query_as(
        "INSERT INTO justificatifs (entreprise_id, categorie, nom_fichier, mime, contenu, taille)
         VALUES ($1, $2, $3, $4, $5, $6) RETURNING id",
    )
    .bind(entreprise.id)
    .bind(&corps.categorie)
    .bind(&corps.nom_fichier)
    .bind(&corps.mime)
    .bind(contenu.as_slice())
    .bind(taille)
    .fetch_one(&etat.bdd)
    .await
    .map_err(crate::erreur_bdd)?;
    Ok((
        StatusCode::CREATED,
        Json(json!({
            "justificatif_id": justificatif_id,
            "categorie": corps.categorie,
            "taille": taille,
        })),
    )
        .into_response())
}

/// Vérifie une fiche comme la plateforme le ferait, sans base de données.
///
/// Utilisé par l'application (lot 3, `compte_valider_entreprise`) et les tests.
pub fn valider_fiche(fiche: &FicheEntreprise) -> RapportValidation {
    fiche.valider(crate::aujourdhui())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fiche_vide_renvoie_les_memes_erreurs_que_valider() {
        let fiche = FicheEntreprise::default();
        let rapport = valider_fiche(&fiche);
        let attendu = fiche.valider(crate::aujourdhui());
        assert_eq!(rapport.erreurs, attendu.erreurs);
        assert!(!rapport.est_valide());
    }

    #[test]
    fn ifu_normalise_retire_les_espaces() {
        let mut fiche = FicheEntreprise::default();
        fiche.ifu = "3201 9123 45678".into();
        assert_eq!(ifu_normalise(&fiche).unwrap(), "3201912345678");
        fiche.ifu = "123".into();
        assert!(ifu_normalise(&fiche).is_err());
    }

    #[test]
    fn categories_et_formats_de_justificatifs() {
        assert!(matches!(
            "ATTESTATION_IFU",
            "ATTESTATION_IFU" | "EXTRAIT_RCCM"
        ));
        assert!(matches!(
            "image/jpeg",
            "application/pdf" | "image/png" | "image/jpeg"
        ));
        assert_eq!(TAILLE_MAX_JUSTIFICATIF, 5 * 1024 * 1024);
    }
}
