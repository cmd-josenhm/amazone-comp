//! Taux exacts (TVA, AIB, ITS, CNSS…).

use std::fmt;
use std::str::FromStr;

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::{ErreurNoyau, Resultat};

/// Taux exprimé en fraction exacte : 18 % est stocké `0.18`.
///
/// Sérialisé en texte (`"0.18"`) pour ne jamais passer par un flottant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Taux(Decimal);

impl Taux {
    /// Taux nul.
    pub const ZERO: Taux = Taux(Decimal::ZERO);

    /// Crée un taux depuis une fraction, ex. `Taux::fraction("0.18")`. Doit être entre 0 et 1.
    pub fn fraction(texte: &str) -> Resultat<Self> {
        let d = Decimal::from_str(texte.trim())
            .map_err(|_| ErreurNoyau::TauxInvalide(texte.to_owned()))?;
        Self::depuis_decimal(d).ok_or_else(|| ErreurNoyau::TauxInvalide(texte.to_owned()))
    }

    /// Crée un taux depuis un pourcentage, ex. `Taux::pourcentage("18")` ou `"1.5"`.
    pub fn pourcentage(texte: &str) -> Resultat<Self> {
        let d = Decimal::from_str(texte.trim())
            .map_err(|_| ErreurNoyau::TauxInvalide(texte.to_owned()))?;
        Self::depuis_decimal(d / Decimal::ONE_HUNDRED)
            .ok_or_else(|| ErreurNoyau::TauxInvalide(texte.to_owned()))
    }

    fn depuis_decimal(d: Decimal) -> Option<Self> {
        (d >= Decimal::ZERO && d <= Decimal::ONE).then_some(Taux(d.normalize()))
    }

    /// Valeur décimale (fraction).
    pub fn valeur(self) -> Decimal {
        self.0
    }

    /// Valeur en pourcentage (18 % → 18).
    pub fn en_pourcentage(self) -> Decimal {
        (self.0 * Decimal::ONE_HUNDRED).normalize()
    }
}

impl TryFrom<String> for Taux {
    type Error = ErreurNoyau;
    fn try_from(s: String) -> Resultat<Self> {
        Taux::fraction(&s)
    }
}

impl From<Taux> for String {
    fn from(t: Taux) -> String {
        t.0.to_string()
    }
}

impl fmt::Display for Taux {
    /// Format français : `18 %`, `1,5 %`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let p = self.en_pourcentage().to_string().replace('.', ",");
        write!(f, "{p}\u{00A0}%")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creation() {
        assert_eq!(
            Taux::pourcentage("18").unwrap(),
            Taux::fraction("0.18").unwrap()
        );
        assert_eq!(
            Taux::pourcentage("1.5").unwrap().to_string(),
            "1,5\u{00A0}%"
        );
        assert!(Taux::pourcentage("101").is_err());
        assert!(Taux::pourcentage("-1").is_err());
        assert!(Taux::fraction("abc").is_err());
    }
}
