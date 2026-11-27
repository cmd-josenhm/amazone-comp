//! # ac-donnees
//!
//! Couche de données locale. **Propriétaire : agent A2.**
//!
//! - [`Base`] : ouverture du dossier SQLCipher (ADR 0005), transactions, moteur de [`Migration`]s.
//! - [`CleBase`] : conteneur de clé mémoire effacé à la destruction et volontairement non formatable.
//! - [`audit`] : journal d'audit chaîné SHA-256 (règle R5).
//! - [`session`] : utilisateurs, rôles, permissions vérifiées côté Rust.
//!
//! Le Lot 1 livre l'ouverture SQLCipher ; génération/enveloppes, utilisateurs et sauvegardes sont
//! livrés par les lots suivants (voir `PROMPT_A2.md`).

#![forbid(unsafe_code)]

pub mod audit;
pub mod base;
pub mod cle;
pub mod erreur;
pub mod session;

pub use base::{Base, Migration};
pub use cle::CleBase;
pub use erreur::ErreurDonnees;
/// Ré-export de rusqlite pour que les autres crates utilisent exactement la même version.
pub use rusqlite;
pub use session::{Permission, Role, Session};

/// Transaction SQL. Tous les modules écrivent **dans une transaction** fournie par l'appelant.
pub type Transaction<'a> = rusqlite::Transaction<'a>;

/// Connexion en lecture (requêtes de consultation).
pub type Connexion = rusqlite::Connection;

/// Résultat standard de la couche de données.
pub type Resultat<T> = Result<T, ErreurDonnees>;
