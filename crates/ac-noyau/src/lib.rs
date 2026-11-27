//! # ac-noyau
//!
//! Types communs à tout AMAZONE COMP. **Propriétaire : agent A1.**
//!
//! - [`Montant`] : somme d'argent en **francs CFA entiers** (XOF). Jamais de flottant (règle R1).
//! - [`Quantite`] : quantité exacte en **millièmes** (1,5 kg = 1 500 millièmes). Jamais de flottant.
//! - [`Taux`] : taux exact (décimal), par exemple 18 % = `Taux::pourcentage("18")`.
//! - [`Arrondi`] : règle d'arrondi au franc.
//! - [`Periode`] : intervalle de dates fermé.
//! - [`Ifu`] : Identifiant fiscal unique béninois.
//! - [`entreprise`] : fiche légale de l'entreprise (IFU, RCCM, adresse…) et sa validation (ADR 0009).
//! - [`parametres`] : lecture des paramètres versionnés (`parametres-fiscaux/<année>/*.toml`).

#![forbid(unsafe_code)]

pub mod entreprise;
pub mod erreur;
pub mod identifiants;
pub mod montant;
pub mod parametres;
pub mod periode;
pub mod quantite;
pub mod taux;

pub use entreprise::{FicheEntreprise, RapportValidation, Regime};
pub use erreur::ErreurNoyau;
pub use identifiants::Ifu;
pub use montant::{Arrondi, Montant};
pub use periode::Periode;
pub use quantite::Quantite;
pub use taux::Taux;

/// Date civile (sans heure ni fuseau). Le Bénin est en UTC+1 sans heure d'été.
pub type Date = chrono::NaiveDate;

/// Résultat standard des fonctions du noyau.
pub type Resultat<T> = Result<T, ErreurNoyau>;
