//! Quantités exactes en millièmes d'unité.
//!
//! Une quantité non entière (1,5 kg de riz, 3,25 heures de prestation…) est représentée par un
//! entier de **millièmes** (`i64`), conformément à l'ADR 0002 : jamais de flottant pour une valeur
//! qui entre dans un calcul monétaire (règle R1). Le produit « prix unitaire × quantité » se fait
//! via [`Montant::multiplier_quantite`](crate::Montant::multiplier_quantite), qui arrondit au
//! franc selon une règle [`Arrondi`](crate::Arrondi) explicite.

use std::fmt;
use std::iter::Sum;
use std::ops::{Add, Neg, Sub};

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::{ErreurNoyau, Resultat};

/// Quantité exacte exprimée en **millièmes d'unité** (`i64`).
///
/// 1 000 millièmes valent 1 unité. Bornée à ± [`Quantite::MAX`] (10^15 millièmes, soit 10^12
/// unités) pour que la multiplication par un [`Montant`](crate::Montant) ne puisse pas déborder.
///
/// Sérialisée en entier JSON (millièmes), comme [`Montant`](crate::Montant) : l'interface ne fait
/// aucun calcul, elle transmet des entiers (contrat C8).
///
/// # Exemple
///
/// ```
/// use ac_noyau::Quantite;
///
/// let q = Quantite::depuis_texte("1,5").unwrap();
/// assert_eq!(q.milliemes(), 1_500);
/// assert_eq!(q.to_string(), "1,5");
/// ```
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize,
)]
#[serde(try_from = "i64", into = "i64")]
pub struct Quantite(i64);

impl Quantite {
    /// Quantité nulle.
    pub const ZERO: Quantite = Quantite(0);

    /// Une unité exacte (1 000 millièmes).
    pub const UN: Quantite = Quantite(1_000);

    /// Borne absolue : 10^15 millièmes (10^12 unités).
    pub const MAX: i64 = 1_000_000_000_000_000;

    /// Crée une quantité depuis des millièmes d'unité, après contrôle des bornes.
    pub fn depuis_milliemes(milliemes: i64) -> Resultat<Self> {
        if milliemes.unsigned_abs() > Self::MAX as u64 {
            return Err(ErreurNoyau::QuantiteHorsLimites(i128::from(milliemes)));
        }
        Ok(Quantite(milliemes))
    }

    /// Crée une quantité depuis un nombre entier d'unités (ex. `2` → 2,000).
    pub fn depuis_unites(unites: i64) -> Resultat<Self> {
        let milliemes = unites
            .checked_mul(1_000)
            .ok_or(ErreurNoyau::QuantiteHorsLimites(i128::from(unites) * 1_000))?;
        Self::depuis_milliemes(milliemes)
    }

    /// Crée une quantité depuis un texte saisi en français : `3,5`, `3.5`, `2`, `-1,25`…
    ///
    /// Le séparateur décimal peut être la virgule ou le point ; au plus **3 décimales** (la
    /// précision est la millième, on ne tronque jamais en silence). Tout autre texte est refusé :
    /// pas d'unité, pas d'espaces intérieurs, pas de notation scientifique.
    ///
    /// # Exemple
    ///
    /// ```
    /// use ac_noyau::Quantite;
    ///
    /// assert_eq!(Quantite::depuis_texte("0,333").unwrap().milliemes(), 333);
    /// assert!(Quantite::depuis_texte("1,5555").is_err());
    /// ```
    pub fn depuis_texte(texte: &str) -> Resultat<Self> {
        let corps = texte.trim();
        let (negatif, chiffres) = match corps.strip_prefix('-') {
            Some(reste) => (true, reste),
            None => (false, corps.strip_prefix('+').unwrap_or(corps)),
        };
        let (entiere, fraction) = match chiffres.split_once([',', '.']) {
            Some((e, f)) => (e, f),
            None => (chiffres, ""),
        };
        if entiere.is_empty() && fraction.is_empty() {
            return Err(ErreurNoyau::QuantiteInvalide(texte.to_owned()));
        }
        let chiffres_valides = |s: &str| s.chars().all(|c| c.is_ascii_digit());
        if !chiffres_valides(entiere) || !chiffres_valides(fraction) {
            return Err(ErreurNoyau::QuantiteInvalide(texte.to_owned()));
        }
        if fraction.len() > 3 {
            return Err(ErreurNoyau::QuantiteInvalide(texte.to_owned()));
        }
        let unites: i128 = if entiere.is_empty() {
            0
        } else {
            entiere
                .parse()
                .map_err(|_| ErreurNoyau::QuantiteInvalide(texte.to_owned()))?
        };
        // « 5 » → 500, « 25 » → 250, « 333 » → 333 : la fraction est alignée sur 3 décimales.
        let milliemes_fraction: i128 = if fraction.is_empty() {
            0
        } else {
            let valeur: i128 = fraction
                .parse()
                .map_err(|_| ErreurNoyau::QuantiteInvalide(texte.to_owned()))?;
            valeur * [100i128, 10, 1][fraction.len() - 1]
        };
        let milliemes_abs = unites * 1_000 + milliemes_fraction;
        if milliemes_abs > i128::from(Self::MAX) {
            return Err(ErreurNoyau::QuantiteHorsLimites(milliemes_abs));
        }
        let signe = if negatif { -1i128 } else { 1i128 };
        let milliemes = i64::try_from(signe * milliemes_abs)
            .map_err(|_| ErreurNoyau::QuantiteHorsLimites(milliemes_abs))?;
        Self::depuis_milliemes(milliemes)
    }

    /// Valeur en millièmes d'unité.
    pub const fn milliemes(self) -> i64 {
        self.0
    }

    /// Vrai si la quantité vaut zéro.
    pub const fn est_zero(self) -> bool {
        self.0 == 0
    }

    /// Vrai si la quantité est strictement négative.
    pub const fn est_negatif(self) -> bool {
        self.0 < 0
    }

    /// Valeur absolue.
    pub const fn abs(self) -> Self {
        Quantite(self.0.abs())
    }

    /// Valeur décimale exacte (pour les calculs intermédiaires) : 1 500 millièmes → 1,5.
    pub fn en_decimal(self) -> Decimal {
        Decimal::new(self.0, 3)
    }
}

impl TryFrom<i64> for Quantite {
    type Error = ErreurNoyau;
    fn try_from(milliemes: i64) -> Resultat<Self> {
        Quantite::depuis_milliemes(milliemes)
    }
}

impl From<Quantite> for i64 {
    fn from(q: Quantite) -> i64 {
        q.0
    }
}

// Les opérateurs restent sûrs : chaque opérande est borné à 10^15 millièmes, leur somme ne peut
// donc pas déborder un i64 (9,2 × 10^18). Pour enchaîner des calculs, préférer les méthodes
// contrôlées de `Montant`.
impl Add for Quantite {
    type Output = Quantite;
    fn add(self, autre: Quantite) -> Quantite {
        Quantite(self.0 + autre.0)
    }
}

impl Sub for Quantite {
    type Output = Quantite;
    fn sub(self, autre: Quantite) -> Quantite {
        Quantite(self.0 - autre.0)
    }
}

impl Neg for Quantite {
    type Output = Quantite;
    fn neg(self) -> Quantite {
        Quantite(-self.0)
    }
}

impl Sum for Quantite {
    fn sum<I: Iterator<Item = Quantite>>(iter: I) -> Quantite {
        iter.fold(Quantite::ZERO, |acc, q| acc + q)
    }
}

impl fmt::Display for Quantite {
    /// Format français : `1,5`, `1,25`, `2`, `0` (zéros inutiles de la fraction supprimés).
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let signe = if self.0 < 0 { "-" } else { "" };
        let abs = self.0.unsigned_abs();
        let unites = abs / 1_000;
        let fraction = abs % 1_000;
        if fraction == 0 {
            write!(f, "{signe}{unites}")
        } else {
            let mut chiffres = format!("{fraction:03}");
            while chiffres.ends_with('0') {
                chiffres.pop();
            }
            write!(f, "{signe}{unites},{chiffres}")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn q(milliemes: i64) -> Quantite {
        Quantite::depuis_milliemes(milliemes).unwrap()
    }

    #[test]
    fn construction_par_milliemes_et_unites() {
        assert_eq!(q(1_500).milliemes(), 1_500);
        assert_eq!(q(0), Quantite::ZERO);
        assert_eq!(Quantite::depuis_unites(2).unwrap(), q(2_000));
        assert_eq!(Quantite::depuis_unites(0).unwrap(), Quantite::ZERO);
        // 10^12 unités = 10^15 millièmes : à la borne exacte
        assert_eq!(
            Quantite::depuis_unites(1_000_000_000_000).unwrap(),
            q(Quantite::MAX)
        );
        assert!(Quantite::depuis_unites(1_000_000_000_001).is_err());
        assert!(Quantite::depuis_unites(i64::MAX).is_err());
        assert!(Quantite::depuis_milliemes(Quantite::MAX).is_ok());
        assert!(Quantite::depuis_milliemes(Quantite::MAX + 1).is_err());
        assert!(Quantite::depuis_milliemes(-Quantite::MAX - 1).is_err());
    }

    #[test]
    fn construction_par_texte_exemples() {
        assert_eq!(Quantite::depuis_texte("3,5").unwrap(), q(3_500));
        assert_eq!(Quantite::depuis_texte("3.5").unwrap(), q(3_500));
        assert_eq!(Quantite::depuis_texte("0,333").unwrap(), q(333));
        assert_eq!(Quantite::depuis_texte("2").unwrap(), q(2_000));
        assert_eq!(Quantite::depuis_texte("-1,25").unwrap(), q(-1_250));
        assert_eq!(Quantite::depuis_texte("+2").unwrap(), q(2_000));
        assert_eq!(Quantite::depuis_texte(" 2 ").unwrap(), q(2_000));
        assert_eq!(Quantite::depuis_texte(".5").unwrap(), q(500));
        assert_eq!(Quantite::depuis_texte("5.").unwrap(), q(5_000));
        assert_eq!(Quantite::depuis_texte("0").unwrap(), Quantite::ZERO);
        assert_eq!(Quantite::depuis_texte("-0,001").unwrap(), q(-1));
    }

    #[test]
    fn construction_par_texte_refus() {
        for texte in [
            "",
            "   ",
            ",",
            ".",
            "-",
            "+",
            "3,5,5",
            "3,5555",
            "abc",
            "1 2",
            "1e3",
            "--2",
            "0x10",
            "1,234 5",
            "99999999999999999999999999",
        ] {
            assert!(
                Quantite::depuis_texte(texte).is_err(),
                "le texte « {texte} » aurait dû être refusé"
            );
        }
    }

    #[test]
    fn texte_trop_grand_hors_limites() {
        // 10^12 unités + 1 millième : dépasse la borne de 10^15 millièmes
        assert!(Quantite::depuis_texte("1000000000000,001").is_err());
        assert_eq!(
            Quantite::depuis_texte("1000000000000").unwrap(),
            q(Quantite::MAX)
        );
    }

    #[test]
    fn affichage_francais() {
        assert_eq!(q(1_500).to_string(), "1,5");
        assert_eq!(q(1_250).to_string(), "1,25");
        assert_eq!(q(2_000).to_string(), "2");
        assert_eq!(q(0).to_string(), "0");
        assert_eq!(q(-3_500).to_string(), "-3,5");
        assert_eq!(q(3_333).to_string(), "3,333");
        assert_eq!(q(50).to_string(), "0,05");
        assert_eq!(q(5).to_string(), "0,005");
        assert_eq!(q(-5).to_string(), "-0,005");
    }

    #[test]
    fn arithmetique() {
        assert_eq!(q(1_500) + q(500), q(2_000));
        assert_eq!(q(1_500) - q(2_000), q(-500));
        assert_eq!(-q(1_500), q(-1_500));
        let somme: Quantite = [q(1_500), q(500), q(-1_000)].into_iter().sum();
        assert_eq!(somme, q(1_000));
    }

    #[test]
    fn comparaisons_et_conversions() {
        assert!(!q(1_500).est_negatif());
        assert!(q(-1).est_negatif());
        assert!(q(0).est_zero());
        assert_eq!(q(-3_500).abs(), q(3_500));
        assert_eq!(Quantite::try_from(2_000i64).unwrap(), q(2_000));
        assert!(Quantite::try_from(Quantite::MAX + 1i64).is_err());
        assert_eq!(i64::from(q(-42)), -42);
    }

    #[test]
    fn conversion_decimale_exacte() {
        assert_eq!(q(1_500).en_decimal().to_string(), "1.500");
        assert_eq!(q(-1_250).en_decimal().to_string(), "-1.250");
        assert_eq!(q(0).en_decimal().to_string(), "0.000");
    }

    #[test]
    fn serde_entier() {
        assert_eq!(serde_json::to_string(&q(1_500)).unwrap(), "1500");
        let lu: Quantite = serde_json::from_str("-42").unwrap();
        assert_eq!(lu, q(-42));
        assert!(Quantite::try_from(1_000_000_000_000_001i64).is_err());
        let hors_bornes = serde_json::from_str::<Quantite>("2000000000000000");
        assert!(hors_bornes.is_err());
        assert!(serde_json::from_str::<Quantite>("12.5").is_err());
    }
}
