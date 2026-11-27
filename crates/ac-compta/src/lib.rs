//! # ac-compta
//!
//! Moteur comptable **SYSCOHADA révisé**. **Propriétaire : agent A1.**
//!
//! C'est le **seul** point d'entrée pour créer des écritures (architecture §3) :
//! les autres modules construisent un [`BrouillonEcriture`] et appellent [`passer_ecriture`]
//! dans **leur** transaction.
//!
//! État : squelette — types du contrat, contrôle d'équilibre et enregistrement minimal.
//! À livrer par A1 : plan de comptes complet, exercices et périodes, verrouillage, contre-passation,
//! lettrage, balance, grand livre, clôture et à-nouveaux (voir `PROMPT_A1.md`).

#![forbid(unsafe_code)]

pub mod ecriture;
pub mod erreur;
pub mod fiscal;
pub mod plan;

pub use ecriture::{
    passer_ecriture, verifier_brouillon, BrouillonEcriture, EcritureId, LigneBrouillon,
    ModuleOrigine,
};
pub use erreur::ErreurCompta;
pub use fiscal::CodeFiscal;
pub use plan::NumeroCompte;

/// Résultat standard du moteur comptable.
pub type Resultat<T> = Result<T, ErreurCompta>;

/// Migrations de `ac-compta` (plage 1000–1999).
pub fn migrations() -> &'static [ac_donnees::Migration] {
    &[ac_donnees::Migration {
        version: 1000,
        nom: "compta_socle",
        sql: include_str!("../migrations/1000_compta_socle.sql"),
    }]
}
