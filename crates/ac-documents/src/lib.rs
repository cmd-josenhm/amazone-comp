//! # ac-documents
//!
//! Rendu commun des documents. **Propriétaire : agent A1.**
//!
//! Objectif : une seule façon de produire des PDF (factures, bulletins, déclarations, états) et des
//! classeurs Excel `.xlsx` (déclarations au format DGI, exports), en noir et blanc, sans police
//! téléchargée, fonctionnant hors ligne.
//!
//! État : squelette (contrat seulement). A1 choisit les bibliothèques par ADR (pistes :
//! `rust_xlsxwriter` pour Excel, `printpdf` ou `genpdf` pour PDF) en respectant la règle R12.

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Erreurs de rendu.
#[derive(Debug, Error)]
pub enum ErreurDocument {
    /// Fonction prévue mais pas encore livrée.
    #[error("rendu non encore disponible : {0}")]
    NonImplemente(&'static str),
    /// Erreur d'écriture du fichier.
    #[error("impossible d'écrire le document : {0}")]
    Ecriture(String),
}

/// Bloc de contenu indépendant du format de sortie.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Bloc {
    /// Titre de section.
    Titre(String),
    /// Paragraphe de texte.
    Paragraphe(String),
    /// Tableau : en-têtes + lignes (cellules déjà formatées en texte français).
    Tableau {
        /// En-têtes de colonnes.
        entetes: Vec<String>,
        /// Lignes.
        lignes: Vec<Vec<Cellule>>,
    },
    /// Saut de page (PDF).
    SautDePage,
}

/// Cellule de tableau typée (pour garder les nombres en nombres dans Excel).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Cellule {
    /// Texte.
    Texte(String),
    /// Montant en FCFA (entier).
    Montant(ac_noyau::Montant),
    /// Date.
    Date(ac_noyau::Date),
    /// Vide.
    Vide,
}

/// Document à produire.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Document {
    /// Titre (en-tête de chaque page).
    pub titre: String,
    /// Raison sociale + IFU affichés en en-tête.
    pub entete_entreprise: String,
    /// Contenu.
    pub blocs: Vec<Bloc>,
    /// Mention de bas de page (ex. « SIMULATION — NON VALABLE FISCALEMENT »).
    pub pied_de_page: Option<String>,
}

/// Produit un PDF.
pub fn rendre_pdf(_doc: &Document) -> Result<Vec<u8>, ErreurDocument> {
    Err(ErreurDocument::NonImplemente("rendu PDF (A1)"))
}

/// Produit un classeur Excel `.xlsx` (une feuille par tableau).
pub fn rendre_xlsx(_doc: &Document) -> Result<Vec<u8>, ErreurDocument> {
    Err(ErreurDocument::NonImplemente("rendu Excel (A1)"))
}
