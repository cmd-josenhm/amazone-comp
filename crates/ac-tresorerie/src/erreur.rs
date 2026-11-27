//! Erreurs du module Trésorerie (messages compréhensibles par un non-comptable).

use thiserror::Error;

/// Erreurs du module Trésorerie.
#[derive(Debug, Error)]
pub enum ErreurTresorerie {
    /// Type de compte invalide.
    #[error("type de compte invalide : « {0} » (attendu : BANQUE, CAISSE ou MOBILE_MONEY)")]
    TypeCompteInvalide(String),
    /// Module Mobile Money désactivé dans la configuration du dossier.
    #[error("le module Mobile Money est désactivé ; activez-le dans la configuration de trésorerie")]
    MobileMoneyDesactive,
    /// Libellé obligatoire.
    #[error("le libellé du compte est obligatoire")]
    LibelleVide,
    /// Libellé trop long.
    #[error("le libellé du compte ne peut pas dépasser 200 caractères")]
    LibelleTropLong,
    /// Nom de la banque ou de l'opérateur obligatoire.
    #[error("le nom de la banque ou de l'opérateur est obligatoire pour ce compte")]
    BanqueOperateurObligatoire,
    /// Numéro de compte ou de téléphone manquant pour ce type de compte.
    #[error("un numéro de compte ou de téléphone est obligatoire pour ce type de compte")]
    NumeroObligatoire,
    /// Numéro de téléphone Mobile Money invalide.
    #[error("le numéro Mobile Money doit être un numéro béninois valide")]
    TelephoneMobileMoneyInvalide,
    /// Nom de l'opérateur Mobile Money manquant.
    #[error("l'opérateur Mobile Money est obligatoire")]
    OperateurObligatoire,
    /// Opérateur non présent dans `tresorerie_operateur`.
    #[error("l'opérateur « {0} » n'existe pas dans la liste des opérateurs Mobile Money")]
    OperateurInconnu(String),
    /// Opérateur inactif.
    #[error("l'opérateur « {0} » est désactivé ; choisissez un opérateur actif")]
    OperateurInactif(String),
    /// Nom d'opérateur obligatoire.
    #[error("le nom de l'opérateur est obligatoire")]
    NomOperateurVide,
    /// Nom d'opérateur trop long.
    #[error("le nom de l'opérateur ne peut pas dépasser 80 caractères")]
    NomOperateurTropLong,
    /// Libellé d'opérateur trop long.
    #[error("le libellé de l'opérateur ne peut pas dépasser 120 caractères")]
    LibelleOperateurTropLong,
    /// Opérateur déjà enregistré.
    #[error("l'opérateur « {0} » est déjà enregistré")]
    OperateurDejaExistant(String),
    /// Référence de la pièce justificative d'ouverture manquante.
    #[error("indiquez la référence de la pièce justificative du solde d'ouverture")]
    PieceOuvertureObligatoire,
    /// Compte comptable de contrepartie d'ouverture manquant.
    #[error("choisissez un compte comptable existant en contrepartie du solde d'ouverture")]
    ContrepartieOuvertureObligatoire,
    /// Compte comptable de contrepartie absent ou inutilisable.
    #[error("le compte de contrepartie « {0} » n'existe pas ou ne peut pas être mouvementé")]
    ContrepartieInconnue(String),
    /// Numéro déjà utilisé pour ce type de banque ou d'opérateur.
    #[error("ce numéro est déjà utilisé pour cette banque ou cet opérateur")]
    NumeroDejaUtilise,
    /// Compte comptable racine ayant déjà des écritures.
    #[error("le compte comptable {0} contient déjà des écritures ; aucun sous-compte ne peut y être ajouté")]
    CompteRacineAvecMouvements(String),
    /// Aucun numéro de sous-compte disponible dans la limite de la longueur du plan comptable.
    #[error("aucun numéro de sous-compte disponible sous le compte {0}")]
    SousCompteIndisponible(String),
    /// Caisse avec solde d'ouverture négatif (interdit).
    #[error("une caisse ne peut pas avoir un solde négatif (solde d'ouverture {0} FCFA)")]
    CaisseSoldeNegatif(i64),
    /// Identifiant de compte inconnu.
    #[error("aucun compte de trésorerie avec l'identifiant {0}")]
    CompteInconnu(i64),
    /// Code de journal invalide.
    #[error("code de journal invalide : « {0} » (attendu : 2 à 4 majuscules ou chiffres)")]
    JournalCodeInvalide(String),
    /// Tous les codes de journal du préfixe sont utilisés.
    #[error("aucun code de journal disponible pour le préfixe {0}")]
    JournalIndisponible(String),
    /// Le libellé du journal est vide.
    #[error("le libellé du journal est obligatoire")]
    JournalLibelleVide,
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
            ErreurTresorerie::MobileMoneyDesactive => "MOBILE_MONEY_DESACTIVE",
            ErreurTresorerie::LibelleVide => "LIBELLE_VIDE",
            ErreurTresorerie::LibelleTropLong => "LIBELLE_TROP_LONG",
            ErreurTresorerie::BanqueOperateurObligatoire => "BANQUE_OPERATEUR_OBLIGATOIRE",
            ErreurTresorerie::NumeroObligatoire => "NUMERO_OBLIGATOIRE",
            ErreurTresorerie::TelephoneMobileMoneyInvalide => "TELEPHONE_MOBILE_MONEY_INVALIDE",
            ErreurTresorerie::OperateurObligatoire => "OPERATEUR_OBLIGATOIRE",
            ErreurTresorerie::OperateurInconnu(_) => "OPERATEUR_INCONNU",
            ErreurTresorerie::OperateurInactif(_) => "OPERATEUR_INACTIF",
            ErreurTresorerie::NomOperateurVide => "NOM_OPERATEUR_VIDE",
            ErreurTresorerie::NomOperateurTropLong => "NOM_OPERATEUR_TROP_LONG",
            ErreurTresorerie::LibelleOperateurTropLong => "LIBELLE_OPERATEUR_TROP_LONG",
            ErreurTresorerie::OperateurDejaExistant(_) => "OPERATEUR_DEJA_EXISTANT",
            ErreurTresorerie::PieceOuvertureObligatoire => "PIECE_OUVERTURE_OBLIGATOIRE",
            ErreurTresorerie::ContrepartieOuvertureObligatoire => {
                "CONTREPARTIE_OUVERTURE_OBLIGATOIRE"
            }
            ErreurTresorerie::ContrepartieInconnue(_) => "CONTREPARTIE_INCONNUE",
            ErreurTresorerie::NumeroDejaUtilise => "NUMERO_DEJA_UTILISE",
            ErreurTresorerie::CompteRacineAvecMouvements(_) => "COMPTE_RACINE_AVEC_MOUVEMENTS",
            ErreurTresorerie::SousCompteIndisponible(_) => "SOUS_COMPTE_INDISPONIBLE",
            ErreurTresorerie::CaisseSoldeNegatif(_) => "CAISSE_SOLDE_NEGATIF",
            ErreurTresorerie::CompteInconnu(_) => "COMPTE_INCONNU",
            ErreurTresorerie::JournalCodeInvalide(_) => "JOURNAL_CODE_INVALIDE",
            ErreurTresorerie::JournalIndisponible(_) => "JOURNAL_INDISPONIBLE",
            ErreurTresorerie::JournalLibelleVide => "JOURNAL_LIBELLE_VIDE",
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
        assert_eq!(
            ErreurTresorerie::ContrepartieOuvertureObligatoire.code(),
            "CONTREPARTIE_OUVERTURE_OBLIGATOIRE"
        );
    }
}
