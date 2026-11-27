//! Régimes d'imposition.
//!
//! Le type est défini dans `ac-noyau` (il fait partie de la fiche entreprise, ADR 0009) et ré-exporté ici
//! pour que le code fiscal continue d'écrire `ac_fiscal::Regime`.

pub use ac_noyau::entreprise::Regime;
