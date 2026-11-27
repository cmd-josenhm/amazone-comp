//! # ac-achats
//!
//! Achats et stocks : fournisseurs, commandes, réceptions, factures fournisseurs, articles, inventaires. **Propriétaire : agent A5.**
//!
//! État : squelette vide. Lire `PROMPT_A5.md` avant de commencer.

#![forbid(unsafe_code)]

/// Migrations de `ac-achats` (plage 3000–3999).
pub fn migrations() -> &'static [ac_donnees::Migration] {
    &[]
}
