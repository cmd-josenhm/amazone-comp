//! Erreurs du moteur d'états financiers (messages compréhensibles par un non-comptable).

use ac_noyau::Montant;
use thiserror::Error;

/// Erreurs produites lors du contrôle des correspondances ou de la génération des états.
#[derive(Debug, Error)]
pub enum ErreurEtats {
    /// Un compte de la balance ne correspond à aucune rubrique SYSCOHADA connue.
    #[error("Le compte {compte} n'est rattaché à aucune rubrique")]
    CompteNonRattache {
        /// Numéro du compte non rattaché.
        compte: String,
    },
    /// Total Actif ≠ Total Passif (aucun poste fourre-tout n'est créé pour masquer un écart).
    #[error("Bilan {exercice} déséquilibré : actif {actif}, passif {passif} (écart {ecart})")]
    BilanDesequilibre {
        /// Exercice concerné (`"N"` ou `"N-1"`).
        exercice: &'static str,
        /// Total général de l'actif net.
        actif: Montant,
        /// Total général du passif.
        passif: Montant,
        /// Écart absolu entre l'actif et le passif.
        ecart: Montant,
    },
    /// Le résultat net calculé au bilan diffère de celui du compte de résultat.
    #[error("Résultat {exercice} incohérent : {bilan} ≠ {compte_resultat} ({ecart})")]
    ResultatsIncoherents {
        /// Exercice concerné (`"N"` ou `"N-1"`).
        exercice: &'static str,
        /// Résultat net figurant au passif du bilan (`CJ`).
        bilan: Montant,
        /// Résultat net calculé au compte de résultat (`XI`).
        compte_resultat: Montant,
        /// Écart absolu entre les deux résultats.
        ecart: Montant,
    },
    /// Un compte apparaît plusieurs fois dans la balance d'un même exercice.
    #[error("Le compte {compte} apparaît plusieurs fois dans la balance ({exercice})")]
    DoublonCompteBalance {
        /// Numéro du compte en doublon.
        compte: String,
        /// Exercice concerné (`"N"` ou `"N-1"`).
        exercice: &'static str,
    },
    /// Une ligne de balance contient un montant négatif sur un total débit ou crédit.
    #[error("Montant négatif interdit sur {compte} (débit et crédit doivent être >= 0)")]
    MontantNegatifBalance {
        /// Numéro du compte concerné.
        compte: String,
    },
    /// La table CSV de correspondances SYSCOHADA est invalide.
    #[error("Table de correspondances SYSCOHADA invalide à la ligne {ligne} : {detail}")]
    CorrespondanceInvalide {
        /// Numéro de ligne (1-based) dans le fichier CSV.
        ligne: usize,
        /// Description de l'anomalie.
        detail: String,
    },
    /// Erreur du moteur comptable.
    #[error(transparent)]
    Compta(#[from] ac_compta::ErreurCompta),
    /// Erreur de calcul ou de bornes du noyau.
    #[error(transparent)]
    Noyau(#[from] ac_noyau::ErreurNoyau),
}

/// Résultat standard du module d'états financiers.
pub type Resultat<T> = Result<T, ErreurEtats>;
