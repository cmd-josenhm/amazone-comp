//! Erreurs du module paie.

use thiserror::Error;

/// Erreurs produites par la paie.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ErreurPaie {
    /// Aucun paramètre social embarqué pour l'année de l'opération.
    #[error("aucun paramètre social pour l'année {0} : mettre à jour le logiciel")]
    AnneeInconnue(i32),
    /// Le fichier social lu ne correspond pas à l'année demandée.
    #[error("le fichier social de l'année {fichier} a été lu pour l'année {demandee}")]
    AnneeIncoherente {
        /// Année demandée par l'appelant.
        demandee: i32,
        /// Année déclarée dans `en_tete.annee`.
        fichier: i32,
    },
    /// Valeur présente dans le fichier mais incohérente (bornes, mois, ordre des taux…).
    #[error("paramètre « {cle} » incohérent : {detail}")]
    ParametreIncoherent {
        /// Clé TOML concernée.
        cle: &'static str,
        /// Détail lisible.
        detail: String,
    },
    /// Erreur du noyau : fichier mal formé, montant ou taux hors bornes.
    #[error(transparent)]
    Noyau(#[from] ac_noyau::ErreurNoyau),
}
