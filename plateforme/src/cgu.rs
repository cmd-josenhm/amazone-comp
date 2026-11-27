//! Conditions générales d'utilisation versionnées (ADR 0004).
//!
//! La version courante est publiée par le propriétaire (console d'administration, lot 7) ;
//! l'acceptation de chaque version est enregistrée par compte. Le premier lancement du
//! logiciel affiche les CGU et enregistre l'acceptation (`compte_accepter_cgu`, lot 3).

use axum::{
    extract::{rejection::JsonRejection, Json, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::json;

use crate::{erreur::ErreurApi, Etat};

/// Demande d'acceptation des CGU.
#[derive(Debug, Deserialize)]
pub struct AcceptationCgu {
    /// Version des conditions acceptées (la version courante).
    pub version: i32,
}

/// Routes `/v1` des conditions générales.
pub fn routes() -> Router<Etat> {
    Router::new()
        .route("/v1/cgu/courante", get(cgu_courante))
        .route("/v1/cgu/acceptations", post(accepter_cgu))
}

async fn cgu_courante(State(etat): State<Etat>) -> Result<Response, ErreurApi> {
    let ligne: Option<(i32, String, String, DateTime<Utc>)> = sqlx::query_as(
        "SELECT version, titre, texte, publiee_le FROM cgu_versions ORDER BY version DESC LIMIT 1",
    )
    .fetch_optional(&etat.bdd)
    .await
    .map_err(crate::erreur_bdd)?;
    match ligne {
        Some((version, titre, texte, publiee_le)) => Ok(Json(json!({
            "version": version,
            "titre": titre,
            "texte": texte,
            "publiee_le": publiee_le,
        }))
        .into_response()),
        None => Err(ErreurApi::introuvable(
            "Aucune version des conditions générales n'est encore publiée.",
        )),
    }
}

async fn accepter_cgu(
    State(etat): State<Etat>,
    en_tetes: axum::http::HeaderMap,
    corps: Result<Json<AcceptationCgu>, JsonRejection>,
) -> Result<Response, ErreurApi> {
    let compte = crate::authentifier(&etat, &en_tetes).await?;
    let corps = crate::corps_json(corps)?;
    let courante: Option<(i32,)> =
        sqlx::query_as("SELECT version FROM cgu_versions ORDER BY version DESC LIMIT 1")
            .fetch_optional(&etat.bdd)
            .await
            .map_err(crate::erreur_bdd)?;
    match courante.map(|(v,)| v) {
        Some(version) if version == corps.version => {}
        Some(version) => {
            return Err(ErreurApi::conflit(
                "CGU_NON_A_JOUR",
                format!(
                    "Les conditions ont changé (version {version}) : rechargez la version courante avant de l'accepter."
                ),
            ));
        }
        None => {
            return Err(ErreurApi::introuvable(
                "Aucune version des conditions générales n'est encore publiée.",
            ));
        }
    }
    sqlx::query(
        "INSERT INTO cgu_acceptations (compte_id, cgu_version) VALUES ($1, $2)
         ON CONFLICT (compte_id, cgu_version) DO NOTHING",
    )
    .bind(compte.compte_id)
    .bind(corps.version)
    .execute(&etat.bdd)
    .await
    .map_err(crate::erreur_bdd)?;
    Ok((
        StatusCode::CREATED,
        Json(json!({
            "message": "Conditions générales acceptées.",
            "version": corps.version,
        })),
    )
        .into_response())
}
