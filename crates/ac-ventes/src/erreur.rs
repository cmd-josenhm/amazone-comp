//! Erreurs du module des ventes (messages compréhensibles par un non-comptable).

use thiserror::Error;

/// Erreurs du module des ventes.
#[derive(Debug, Error)]
pub enum ErreurVentes {
    /// Le nom (ou la raison sociale) du client est vide.
    #[error("le nom du client est obligatoire")]
    NomObligatoire,
    /// Nom de client trop long.
    #[error("le nom du client ne peut pas dépasser {0} caractères")]
    NomTropLong(usize),
    /// Client exonéré d'AIB sans motif.
    #[error("indiquer le motif d'exonération d'AIB, ou décocher l'exonération")]
    MotifExonerationObligatoire,
    /// Courriel mal formé.
    #[error("adresse e-mail invalide : « {0} » (ex. attendu : nom@exemple.bj)")]
    CourrielInvalide(String),
    /// Client introuvable.
    #[error("client introuvable (n° {0})")]
    ClientInconnu(i64),
    /// Le client « Comptant » est obligatoire et protégé.
    #[error("le client « Comptant » est obligatoire : il ne peut pas être modifié ni désactivé")]
    ClientComptantProtege,
    /// Client déjà désactivé.
    #[error("le client {0} est déjà désactivé")]
    ClientDejaInactif(String),
    /// Client déjà actif.
    #[error("le client {0} est déjà actif")]
    ClientDejaActif(String),
    /// Un autre client porte déjà cet IFU.
    #[error("un autre client porte déjà l'IFU {0}")]
    IfuDejaUtilise(String),
    /// Valeur inattendue dans la base (base corrompue ou requête erronée).
    #[error("donnée inattendue dans le dossier : {0}")]
    DonneeIncoherente(String),
    /// Erreur de la couche de données.
    #[error(transparent)]
    Donnees(#[from] ac_donnees::ErreurDonnees),
    /// Erreur des types de base (IFU, RCCM, téléphone, montants…).
    #[error(transparent)]
    Noyau(#[from] ac_noyau::ErreurNoyau),
}

impl From<ac_donnees::rusqlite::Error> for ErreurVentes {
    fn from(e: ac_donnees::rusqlite::Error) -> Self {
        ErreurVentes::Donnees(ac_donnees::ErreurDonnees::Sql(e))
    }
}
