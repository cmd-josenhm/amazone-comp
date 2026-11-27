//! # ac-paie
//!
//! Paie béninoise. **Propriétaire : agent A7.**
//!
//! État : Lot 1 livré — paramètres sociaux typés ([`ParametresSociaux`]) et ITS sur barème.
//! Calculs purs (CNSS, VPS, redevance, net), salariés, bulletins et écritures : lots suivants
//! (voir `PROMPT_A7.md`). Documentation : `docs/modules/paie.md`.

#![forbid(unsafe_code)]

pub mod erreur;
pub mod its;
pub mod parametres;

pub use erreur::ErreurPaie;
pub use parametres::ParametresSociaux;

/// Migrations de `ac-paie` (plage 5000–5999).
pub fn migrations() -> &'static [ac_donnees::Migration] {
    &[]
}
