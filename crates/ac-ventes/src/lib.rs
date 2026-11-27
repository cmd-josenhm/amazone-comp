//! # ac-ventes
//!
//! Ventes : clients, devis, factures, avoirs, facture normalisée. **Propriétaire : agent A4.**
//!
//! - [`clients`] : fichier clients (codes `CLI-…`, recherche, désactivation) — Lot 1.
//! - [`normalisation`] : **contrat** de l'adaptateur MECeF / e-MECeF (ADR 0006) et simulateur
//!   utilisable sans accès à l'API de la DGI.
//!
//! État : Lot 1 « Clients » livré. À venir : voir `PROMPT_A4.md` (factures, avoirs, écrans).

#![forbid(unsafe_code)]

pub mod clients;
pub mod erreur;
pub mod normalisation;

pub use erreur::ErreurVentes;

/// Résultat standard du module des ventes.
pub type Resultat<T> = Result<T, ErreurVentes>;

/// Migrations de `ac-ventes` (plage 2000–2999).
pub fn migrations() -> &'static [ac_donnees::Migration] {
    &[ac_donnees::Migration {
        version: 2000,
        nom: "ventes_clients",
        sql: include_str!("../migrations/2000_ventes_clients.sql"),
    }]
}
