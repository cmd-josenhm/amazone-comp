//! # ac-paie
//!
//! Paie béninoise. **Propriétaire : agent A7.**
//!
//! État : squelette — calcul de l'ITS par barème progressif à partir de `social.toml`.
//! À livrer par A7 : salariés, contrats, éléments variables, bulletins, VPS, CNSS, écritures de
//! paie, déclarations mensuelles (voir `PROMPT_A7.md`).

#![forbid(unsafe_code)]

pub mod its;

/// Migrations de `ac-paie` (plage 5000–5999).
pub fn migrations() -> &'static [ac_donnees::Migration] {
    &[]
}
