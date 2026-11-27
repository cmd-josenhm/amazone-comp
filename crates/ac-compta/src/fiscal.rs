//! Étiquettes fiscales portées par les lignes d'écriture.
//!
//! C'est le **contrat** entre les modules qui génèrent des écritures (ventes, achats, trésorerie,
//! paie) et `ac-fiscal` qui prépare les déclarations : `ac-fiscal` additionne les lignes par
//! étiquette et par période, sans lire les tables des autres modules (architecture §3).
//!
//! Ajouter une variante = ADR + accord de A8 (fiscal) et du chef de projet.

use serde::{Deserialize, Serialize};

/// Nature fiscale d'une ligne d'écriture.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CodeFiscal {
    // --- TVA ---
    /// TVA collectée sur ventes / prestations taxables (base = HT).
    TvaCollectee,
    /// TVA collectée retenue à la source par le client (État, sociétés désignées).
    TvaRetenueParClient,
    /// TVA déductible sur achats de biens et marchandises.
    TvaDeductibleBiens,
    /// TVA déductible sur services.
    TvaDeductibleServices,
    /// TVA déductible sur immobilisations.
    TvaDeductibleImmobilisations,
    /// TVA retenue à la source par l'entreprise sur ses fournisseurs (si elle est désignée).
    TvaRetenueSurFournisseur,
    /// Chiffre d'affaires exonéré (base seulement).
    CaExonere,
    /// Chiffre d'affaires à l'exportation (base seulement).
    CaExport,
    // --- AIB ---
    /// AIB prélevé par l'entreprise sur ses clients (à reverser).
    AibCollecte,
    /// AIB supporté par l'entreprise sur ses achats (imputable sur IS / IBA / TPS).
    AibSupporte,
    // --- Retenues à la source ---
    /// Retenue sur loyers versés.
    RetenueLoyer,
    /// Retenue sur prestations de non-résidents.
    RetenueNonResident,
    /// Retenue sur revenus de capitaux mobiliers (IRCM).
    RetenueIrcm,
    // --- Salaires ---
    /// Impôt sur traitements et salaires retenu.
    Its,
    /// Versement patronal sur salaires.
    Vps,
    /// Cotisation CNSS part salarié.
    CnssSalarie,
    /// Cotisation CNSS part employeur.
    CnssEmployeur,
    /// Redevance audiovisuelle (ORTB / SRTV) retenue sur salaires, si applicable.
    RedevanceAudiovisuelle,
    // --- Impôts sur le résultat ---
    /// Impôt sur les sociétés / IBA de l'exercice.
    ImpotResultat,
    /// Acompte d'IS / IBA versé.
    AcompteImpotResultat,
    /// Taxe professionnelle synthétique.
    Tps,
}
