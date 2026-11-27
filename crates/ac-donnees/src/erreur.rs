//! Erreurs de la couche de données.

use thiserror::Error;

use crate::Permission;

/// Erreurs de la couche de données.
#[derive(Debug, Error)]
pub enum ErreurDonnees {
    /// Erreur SQLite.
    #[error("erreur de base de données : {0}")]
    Sql(#[from] rusqlite::Error),
    /// La clé fournie ne permet pas de lire le schéma du dossier.
    #[error("clé du dossier incorrecte (vérifiez le mot de passe ou la clé de secours)")]
    MotDePasseIncorrect,
    /// La bibliothèque SQLCipher n'a pas pu appliquer la clé au dossier.
    #[error("impossible d'initialiser le chiffrement du dossier")]
    ChiffrementIndisponible,
    /// Deux migrations ont le même numéro ou un numéro hors de la plage de leur crate.
    #[error("migration invalide : {0}")]
    MigrationInvalide(String),
    /// L'utilisateur n'a pas le droit demandé.
    #[error("action non autorisée pour votre rôle ({0:?})")]
    NonAutorise(Permission),
    /// Le journal d'audit a été altéré.
    #[error("journal d'audit altéré à l'entrée n° {0}")]
    AuditAltere(i64),
    /// Fonction prévue mais pas encore livrée.
    #[error("fonction non encore disponible : {0}")]
    NonImplemente(&'static str),
}
