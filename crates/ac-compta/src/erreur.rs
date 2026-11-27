//! Erreurs du moteur comptable (messages compréhensibles par un non-comptable).

use ac_noyau::Montant;
use thiserror::Error;

/// Erreurs du moteur comptable.
#[derive(Debug, Error)]
pub enum ErreurCompta {
    /// Débit total ≠ crédit total.
    #[error("l'écriture n'est pas équilibrée : débit {debit}, crédit {credit}")]
    Desequilibre {
        /// Total débit.
        debit: Montant,
        /// Total crédit.
        credit: Montant,
    },
    /// Moins de deux lignes.
    #[error("une écriture doit comporter au moins deux lignes")]
    TropPeuDeLignes,
    /// Ligne avec débit et crédit à la fois, ou aucun des deux, ou négatif.
    #[error("ligne {0} : saisir soit un débit, soit un crédit, positif")]
    LigneInvalide(usize),
    /// Numéro de compte mal formé.
    #[error("numéro de compte invalide : « {0} »")]
    CompteInvalide(String),
    /// Compte absent du plan de comptes du dossier.
    #[error("le compte {0} n'existe pas dans le plan de comptes")]
    CompteInconnu(String),
    /// Période clôturée ou hors exercice.
    #[error("la date {0} est dans une période clôturée ou hors exercice")]
    PeriodeFermee(ac_noyau::Date),
    /// Libellé vide.
    #[error("le libellé de l'écriture est obligatoire")]
    LibelleVide,
    /// Erreur de la couche de données.
    #[error(transparent)]
    Donnees(#[from] ac_donnees::ErreurDonnees),
    /// Erreur de calcul.
    #[error(transparent)]
    Noyau(#[from] ac_noyau::ErreurNoyau),
}

impl From<ac_donnees::rusqlite::Error> for ErreurCompta {
    fn from(e: ac_donnees::rusqlite::Error) -> Self {
        ErreurCompta::Donnees(ac_donnees::ErreurDonnees::Sql(e))
    }
}
