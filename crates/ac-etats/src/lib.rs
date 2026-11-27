//! # ac-etats
//!
//! États financiers SYSCOHADA, liasse / DSF, tableaux de bord, reprise de données. **Propriétaire : agent A9.**
//!
//! État : squelette vide. Lire `PROMPT_A9.md` avant de commencer.

#![forbid(unsafe_code)]

/// Migrations de `ac-etats` (plage 7000–7999).
pub fn migrations() -> &'static [ac_donnees::Migration] {
    &[]
}
