//! # ac-tresorerie
//!
//! Trésorerie : banques, caisse, Mobile Money (MTN MoMo, Moov Money, Celtiis), rapprochement, import de relevés. **Propriétaire : agent A6.**
//!
//! État : squelette vide. Lire `PROMPT_A6.md` avant de commencer.

#![forbid(unsafe_code)]

/// Migrations de `ac-tresorerie` (plage 4000–4999).
pub fn migrations() -> &'static [ac_donnees::Migration] {
    &[]
}
