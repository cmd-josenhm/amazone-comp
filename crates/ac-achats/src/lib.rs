//! # ac-achats
//!
//! Achats, catalogue d'articles, stocks, immobilisations et amortissements. **Propriétaire : agent A5.**

#![forbid(unsafe_code)]

pub mod catalogue;
pub mod erreur;
pub mod stock;

pub use erreur::ErreurAchats;

/// Résultat standard des opérations d'achats.
pub type Resultat<T> = Result<T, ErreurAchats>;

/// Migrations de `ac-achats` (plage 3000–3999).
pub fn migrations() -> &'static [ac_donnees::Migration] {
    &[ac_donnees::Migration {
        version: 3000,
        nom: "achats_catalogue_stock",
        sql: include_str!("../migrations/3000_catalogue_stock.sql"),
    }]
}
