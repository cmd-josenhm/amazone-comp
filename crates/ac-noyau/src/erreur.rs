//! Erreurs du noyau.

use thiserror::Error;

/// Erreurs produites par les types de base.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ErreurNoyau {
    /// Montant hors des bornes autorisées (± [`crate::Montant::MAX`]).
    #[error("montant hors limites : {0} FCFA")]
    MontantHorsLimites(i128),
    /// Dépassement de capacité lors d'un calcul.
    #[error("dépassement de capacité lors d'un calcul de montant")]
    Depassement,
    /// Texte qui ne représente pas un taux valide.
    #[error("taux invalide : « {0} »")]
    TauxInvalide(String),
    /// Période dont la date de début est après la date de fin.
    #[error("période invalide : le {debut} est après le {fin}")]
    PeriodeInvalide {
        /// Début demandé.
        debut: chrono::NaiveDate,
        /// Fin demandée.
        fin: chrono::NaiveDate,
    },
    /// IFU mal formé.
    #[error("IFU invalide : « {0} » (13 chiffres attendus)")]
    IfuInvalide(String),
    /// Fichier de paramètres introuvable ou illisible.
    #[error("paramètres introuvables : {0}")]
    ParametresIntrouvables(String),
    /// Fichier de paramètres mal formé.
    #[error("paramètres mal formés dans {fichier} : {detail}")]
    ParametresInvalides {
        /// Fichier concerné.
        fichier: String,
        /// Détail de l'erreur de lecture.
        detail: String,
    },
}
