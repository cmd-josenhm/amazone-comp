//! # ac-ventes
//!
//! Ventes : clients, devis, factures, avoirs, facture normalisée. **Propriétaire : agent A4.**
//!
//! Le module [`normalisation`] contient le **contrat** de l'adaptateur MECeF / e-MECeF (ADR 0006)
//! et un simulateur utilisable sans accès à l'API de la DGI.
//!
//! État : squelette. À livrer par A4 : voir `PROMPT_A4.md`.

#![forbid(unsafe_code)]

pub mod normalisation;

/// Migrations de `ac-ventes` (plage 2000–2999). A4 ajoute ici ses tables `ventes_*`.
pub fn migrations() -> &'static [ac_donnees::Migration] {
    &[]
}
