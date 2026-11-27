//! Erreurs de l'API : un statut HTTP, un code stable et un message français compréhensible.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

use ac_noyau::entreprise::RapportValidation;

/// Erreur renvoyée par un gestionnaire de route.
#[derive(Debug)]
pub struct ErreurApi {
    statut: StatusCode,
    corps: serde_json::Value,
}

impl ErreurApi {
    /// Construit une erreur avec un code stable (`DEMANDE_INVALIDE`…) et un message.
    pub fn nouvelle(statut: StatusCode, code: &str, message: impl Into<String>) -> Self {
        ErreurApi {
            statut,
            corps: json!({ "code": code, "message": message.into() }),
        }
    }

    /// 400 — la requête est mal formée.
    pub fn invalide(message: impl Into<String>) -> Self {
        Self::nouvelle(StatusCode::BAD_REQUEST, "DEMANDE_INVALIDE", message)
    }

    /// 401 — jeton d'accès absent ou inconnu.
    pub fn non_autorise() -> Self {
        Self::nouvelle(
            StatusCode::UNAUTHORIZED,
            "NON_AUTORISE",
            "Vous devez être connecté pour effectuer cette action.",
        )
    }

    /// 403 — l'action est interdite à ce compte.
    pub fn interdit(code: &str, message: impl Into<String>) -> Self {
        Self::nouvelle(StatusCode::FORBIDDEN, code, message)
    }

    /// 404 — ressource introuvable.
    pub fn introuvable(message: impl Into<String>) -> Self {
        Self::nouvelle(StatusCode::NOT_FOUND, "INTROUVABLE", message)
    }

    /// 409 — conflit (ressource déjà utilisée, état incompatible).
    pub fn conflit(code: &str, message: impl Into<String>) -> Self {
        Self::nouvelle(StatusCode::CONFLICT, code, message)
    }

    /// 413 — fichier trop volumineux.
    pub fn trop_grand(message: impl Into<String>) -> Self {
        Self::nouvelle(StatusCode::PAYLOAD_TOO_LARGE, "FICHIER_TROP_GRAND", message)
    }

    /// 415 — type de fichier non accepté.
    pub fn type_non_supporte(message: impl Into<String>) -> Self {
        Self::nouvelle(
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "TYPE_NON_SUPPORTE",
            message,
        )
    }

    /// 422 — fiche entreprise invalide : le corps est **exactement** le rapport de
    /// [`ac_noyau::entreprise::FicheEntreprise::valider`] (même liste `erreurs`).
    pub fn fiche_invalide(rapport: &RapportValidation) -> Self {
        let corps = serde_json::to_value(rapport)
            .unwrap_or_else(|_| json!({ "erreurs": [], "avertissements": [] }));
        ErreurApi {
            statut: StatusCode::UNPROCESSABLE_ENTITY,
            corps,
        }
    }

    /// 429 — trop de tentatives ou de requêtes.
    pub fn trop_de_tentatives(code: &str, message: impl Into<String>) -> Self {
        Self::nouvelle(StatusCode::TOO_MANY_REQUESTS, code, message)
    }

    /// 500 — erreur interne (le détail technique reste dans les journaux du serveur).
    pub fn interne() -> Self {
        Self::nouvelle(
            StatusCode::INTERNAL_SERVER_ERROR,
            "ERREUR_INTERNE",
            "Une erreur interne s'est produite. Réessayez plus tard ou contactez l'assistance.",
        )
    }
}

impl IntoResponse for ErreurApi {
    fn into_response(self) -> Response {
        (self.statut, Json(self.corps)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ac_noyau::entreprise::ProblemeChamp;

    #[test]
    fn fiche_invalide_renvoie_le_rapport_exact() {
        let rapport = RapportValidation {
            erreurs: vec![ProblemeChamp {
                champ: "ifu".into(),
                message: "L'IFU doit compter exactement 13 chiffres.".into(),
            }],
            avertissements: vec![],
        };
        let reponse = ErreurApi::fiche_invalide(&rapport).into_response();
        assert_eq!(reponse.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[test]
    fn erreurs_standard_avec_code_et_message() {
        let e = ErreurApi::conflit("IFU_DEJA_INSCRIT", "Cet IFU est déjà inscrit.");
        assert_eq!(e.statut, StatusCode::CONFLICT);
        assert_eq!(e.corps["code"], "IFU_DEJA_INSCRIT");
        assert_eq!(e.corps["message"], "Cet IFU est déjà inscrit.");
    }
}
