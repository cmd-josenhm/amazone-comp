//! # ac-tresorerie
//!
//! Trésorerie : banques, caisse, Mobile Money (MTN MoMo, Moov Money, Celtiis Cash), rapprochement,
//! import de relevés. **Propriétaire : agent A6.**
//!
//! Lot 1 de `PROMPT_A6.md` : comptes de trésorerie — tables `tresorerie_compte`, `tresorerie_operateur`
//! et journaux déclarés (`compta_journal`) ; solde d'ouverture → écriture AN via
//! `ac_compta::passer_ecriture` (A1).

#![forbid(unsafe_code)]

pub mod compte;
pub mod erreur;
pub mod journal;

pub use compte::{
    CreerCompte, TresorerieCompte, TresorerieOperateur, TresorerieSolde, TypeCompte, masquer,
};
pub use erreur::ErreurTresorerie;

/// Résultat standard du module Trésorerie.
pub type Resultat<T> = Result<T, ErreurTresorerie>;

/// Migrations de `ac-tresorerie` (plage 4000–4999).
pub fn migrations() -> &'static [ac_donnees::Migration] {
    &[ac_donnees::Migration {
        version: 4000,
        nom: "tresorerie_comptes",
        sql: include_str!("../migrations/4000_tresorerie_comptes.sql"),
    }]
}
