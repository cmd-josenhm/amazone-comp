//! # ac-etats
//!
//! États financiers **SYSCOHADA révisé** (Système normal, SMT), liasse / DSF, tableaux de bord
//! et reprise de données. **Propriétaire : agent A9.**
//!
//! Lot 1 : tables de correspondances (`correspondances_syscohada.csv`) et moteur pur de calcul
//! du [`Bilan`] et du [`CompteResultat`] à partir d'une [`Balance`] fournie en mémoire.

#![forbid(unsafe_code)]

pub mod balance;
pub mod correspondances;
pub mod erreur;
pub mod etats;

pub use balance::{Balance, LigneBalance, SensSolde};
pub use correspondances::{ColonneBilan, Correspondances, EtatFinancier, MotifCompte};
pub use correspondances::{RegleCorrespondance, SensAttendu, SigneRubrique};
pub use erreur::{ErreurEtats, Resultat};
pub use etats::{bilan, bilan_avec_correspondances, compte_resultat};
pub use etats::{compte_resultat_avec_correspondances, generer_etats_complets};
pub use etats::{verifier_coherence, Bilan, CompteResultat, LigneBilanActif};
pub use etats::{LigneBilanPassif, LigneCompteResultat, TypeLigne};

/// Migrations de `ac-etats` (plage 7000–7999).
pub fn migrations() -> &'static [ac_donnees::Migration] {
    &[]
}
