//! Erreurs du module Trésorerie (messages compréhensibles par un non-comptable).

use thiserror::Error;

/// Erreurs du module Trésorerie.
#[derive(Debug, Error)]
pub enum ErreurTresorerie {
    /// Type de compte invalide.
    #[error("type de compte invalide : « {0} » (attendu : BANQUE, CAISSE ou MOBILE_MONEY)")]
    TypeCompteInvalide(String),
    /// Libellé obligatoire.
    #[error("le libellé du compte est obligatoire")]
    LibelleVide,
    /// Numéro de compte ou de téléphone manquant pour ce type de compte.
    #[error("un numéro de compte ou de téléphone est obligatoire pour ce type de compte")]
    NumeroObligatoire,
    /// Nom de l'opérateur Mobile Money manquant.
    #[error("l'opérateur Mobile Money est obligatoire")]
    OperateurObligatoire,
    /// Opérateur non présent dans `tresorerie_operateur`.
    #[error("l'opérateur « {0} » n'existe pas dans la table tresorerie_operateur")]
    OperateurInconnu(String),
    /// Compte comptable déjà utilisé.
    #[error("le compte comptable « {0} » est déjà utilisé")]
    CompteComptableExistant(String),
    /// Caisse avec solde d'ouverture négatif (interdit).
    #[error("une caisse ne peut pas avoir un solde négatif (solde d'ouverture {0})")]
    CaisseSoldeNegatif(i64),
    /// Identifiant de compte inconnu.
    #[error("aucun compte de trésorerie avec l'identifiant {0}")]
    CompteInconnu(i64),
    /// Code de journal invalide.
    #[error("code de journal invalide : « {0} » (attendu : 2 à 4 majuscules)")]
    JournalCodeInvalide(String),
    /// Erreur du moteur comptable (A1).
    #[error(transparent)]
    Compta(#[from] ac_compta::ErreurCompta),
    /// Erreur de la couche données (A2).
    #[error(transparent)]
    Donnees(#[from] ac_donnees::ErreurDonnees),
    /// Erreur de la couche noyau (A1).
    #[error(transparent)]
    Noyau(#[from] ac_noyau::ErreurNoyau),
}

impl ErreurTresorerie {
    /// Code stable pour l'interface (`ErreurCommande`).
    pub fn code(&self) -> &'static str {
        match self {
            ErreurTresorerie::TypeCompteInvalide(_) => "TYPE_COMPTE_INVALIDE",
            ErreurTresorerie::LibelleVide => "LIBELLE_VIDE",
            ErreurTresorerie::NumeroObligatoire => "NUMERO_OBLIGATOIRE",
            ErreurTresorerie::OperateurObligatoire => "OPERATEUR_OBLIGATOIRE",
            ErreurTresorerie::OperateurInconnu(_) => "OPERATEUR_INCONNU",
            ErreurTresorerie::CompteComptableExistant(_) => "COMPTE_COMPTABLE_EXISTANT",
            ErreurTresorerie::CaisseSoldeNegatif(_) => "CAISSE_SOLDE_NEGATIF",
            ErreurTresorerie::CompteInconnu(_) => "COMPTE_INCONNU",
            ErreurTresorerie::JournalCodeInvalide(_) => "JOURNAL_CODE_INVALIDE",
            ErreurTresorerie::Compta(_) => "ERREUR_COMPTA",
            ErreurTresorerie::Donnees(_) => "ERREUR_DONNEES",
            ErreurTresorerie::Noyau(_) => "ERREUR_NOYAU",
        }
    }
}

impl From<ac_donnees::rusqlite::Error> for ErreurTresorerie {
    fn from(e: ac_donnees::rusqlite::Error) -> Self {
        ErreurTresorerie::Donnees(e.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes_stables() {
        assert_eq!(ErreurTresorerie::LibelleVide.code(), "LIBELLE_VIDE");
        assert_eq!(
            ErreurTresorerie::TypeCompteInvalide("XX".into()).code(),
            "TYPE_COMPTE_INVALIDE"
        );
    }
}
