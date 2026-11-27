//! Régimes d'imposition.

use serde::{Deserialize, Serialize};

/// Régime fiscal du dossier (choisi dans l'assistant de démarrage, modifiable par l'administrateur).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Regime {
    /// Réel normal : TVA, IS/IBA, AIB, DSF complète (système normal SYSCOHADA).
    ReelNormal,
    /// Réel simplifié : obligations allégées (à préciser par A8 selon le CGI 2026).
    ReelSimplifie,
    /// Taxe professionnelle synthétique : petites entreprises (CA ≤ seuil), pas de TVA facturée.
    Tps,
}

impl Regime {
    /// Vrai si l'entreprise facture et déclare la TVA.
    pub fn collecte_tva(self) -> bool {
        !matches!(self, Regime::Tps)
    }
}
