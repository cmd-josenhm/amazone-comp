//! # ac-fiscal
//!
//! Fiscalité béninoise. **Propriétaire : agent A8.**
//!
//! - [`parametres`] : paramètres annuels embarqués (`parametres-fiscaux/<année>/`), typés.
//! - Calculs purs (TVA, AIB, minimum de perception, TPS…) : fonctions sans base de données, testées
//!   avec des exemples chiffrés.
//! - Déclarations : agrégation des lignes d'écriture **par étiquette fiscale** (`ac_compta::CodeFiscal`)
//!   puis rendu PDF / Excel via `ac-documents`.
//!
//! État : squelette. À livrer par A8 : voir `PROMPT_A8.md`.

#![forbid(unsafe_code)]

pub mod parametres;
pub mod regime;
pub mod tva;

pub use parametres::{fichiers_embarques, ParametresAnnee};
pub use regime::Regime;

use thiserror::Error;

/// Erreurs fiscales.
#[derive(Debug, Error)]
pub enum ErreurFiscale {
    /// Aucune donnée pour l'année demandée.
    #[error("aucun paramètre fiscal pour l'année {0} : mettre à jour le logiciel")]
    AnneeInconnue(i32),
    /// Erreur de lecture des paramètres.
    #[error(transparent)]
    Noyau(#[from] ac_noyau::ErreurNoyau),
    /// Fonction prévue mais pas encore livrée.
    #[error("fonction fiscale non encore disponible : {0}")]
    NonImplemente(&'static str),
}

/// Résultat standard du module fiscal.
pub type Resultat<T> = Result<T, ErreurFiscale>;
