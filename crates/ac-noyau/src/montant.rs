//! Montants en francs CFA (XOF).
//!
//! Le franc CFA n'a pas de subdivision utilisée en pratique : un montant est un **entier de francs**.
//! Les calculs intermédiaires (taux, prorata) se font en décimal exact puis sont arrondis au franc
//! selon une règle [`Arrondi`] explicite.

use std::fmt;
use std::iter::Sum;
use std::ops::{Add, Neg, Sub};

use rust_decimal::prelude::ToPrimitive;
use rust_decimal::{Decimal, RoundingStrategy};
use serde::{Deserialize, Serialize};

use crate::{ErreurNoyau, Resultat, Taux};

/// Règle d'arrondi au franc.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Arrondi {
    /// Au franc le plus proche, 0,5 arrondi en s'éloignant de zéro (règle par défaut).
    PlusProche,
    /// Au franc inférieur (troncature vers zéro).
    Inferieur,
    /// Au franc supérieur (en s'éloignant de zéro).
    Superieur,
}

/// Somme d'argent en francs CFA entiers. Peut être négative (avoir, solde créditeur…).
///
/// Bornée à ± [`Montant::MAX`] pour que les additions ne puissent pas déborder en pratique.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize,
)]
#[serde(try_from = "i64", into = "i64")]
pub struct Montant(i64);

impl Montant {
    /// Zéro franc.
    pub const ZERO: Montant = Montant(0);
    /// Borne absolue : 10^15 FCFA (un million de milliards).
    pub const MAX: i64 = 1_000_000_000_000_000;

    /// Crée un montant après contrôle des bornes.
    pub fn new(francs: i64) -> Resultat<Self> {
        if francs.unsigned_abs() > Self::MAX as u64 {
            return Err(ErreurNoyau::MontantHorsLimites(i128::from(francs)));
        }
        Ok(Montant(francs))
    }

    /// Valeur en francs.
    pub const fn francs(self) -> i64 {
        self.0
    }

    /// Vrai si le montant vaut zéro.
    pub const fn est_zero(self) -> bool {
        self.0 == 0
    }

    /// Vrai si le montant est strictement négatif.
    pub const fn est_negatif(self) -> bool {
        self.0 < 0
    }

    /// Valeur absolue.
    pub const fn abs(self) -> Self {
        Montant(self.0.abs())
    }

    /// Addition contrôlée.
    pub fn ajouter(self, autre: Montant) -> Resultat<Self> {
        self.0
            .checked_add(autre.0)
            .ok_or(ErreurNoyau::Depassement)
            .and_then(Montant::new)
    }

    /// Soustraction contrôlée.
    pub fn soustraire(self, autre: Montant) -> Resultat<Self> {
        self.0
            .checked_sub(autre.0)
            .ok_or(ErreurNoyau::Depassement)
            .and_then(Montant::new)
    }

    /// Multiplie par une quantité entière (ex. prix unitaire × quantité).
    pub fn multiplier(self, quantite: i64) -> Resultat<Self> {
        self.0
            .checked_mul(quantite)
            .ok_or(ErreurNoyau::Depassement)
            .and_then(Montant::new)
    }

    /// Applique un taux puis arrondit au franc. Exemple : 10 000 × 18 % = 1 800.
    pub fn appliquer_taux(self, taux: Taux, arrondi: Arrondi) -> Resultat<Self> {
        let brut = Decimal::from(self.0)
            .checked_mul(taux.valeur())
            .ok_or(ErreurNoyau::Depassement)?;
        Self::depuis_decimal(brut, arrondi)
    }

    /// Convertit un décimal en montant selon la règle d'arrondi.
    pub fn depuis_decimal(valeur: Decimal, arrondi: Arrondi) -> Resultat<Self> {
        let strategie = match arrondi {
            Arrondi::PlusProche => RoundingStrategy::MidpointAwayFromZero,
            Arrondi::Inferieur => RoundingStrategy::ToZero,
            Arrondi::Superieur => RoundingStrategy::AwayFromZero,
        };
        let arrondi = valeur.round_dp_with_strategy(0, strategie);
        let entier = arrondi.to_i64().ok_or(ErreurNoyau::Depassement)?;
        Montant::new(entier)
    }

    /// Valeur décimale exacte (pour les calculs intermédiaires).
    pub fn en_decimal(self) -> Decimal {
        Decimal::from(self.0)
    }
}

impl TryFrom<i64> for Montant {
    type Error = ErreurNoyau;
    fn try_from(francs: i64) -> Resultat<Self> {
        Montant::new(francs)
    }
}

impl From<Montant> for i64 {
    fn from(m: Montant) -> i64 {
        m.0
    }
}

// Les opérateurs restent sûrs : chaque opérande est borné à 10^15, la somme de deux montants
// ne peut donc pas déborder un i64 (9,2 × 10^18). Pour enchaîner des calculs, préférer `ajouter`.
impl Add for Montant {
    type Output = Montant;
    fn add(self, autre: Montant) -> Montant {
        Montant(self.0 + autre.0)
    }
}

impl Sub for Montant {
    type Output = Montant;
    fn sub(self, autre: Montant) -> Montant {
        Montant(self.0 - autre.0)
    }
}

impl Neg for Montant {
    type Output = Montant;
    fn neg(self) -> Montant {
        Montant(-self.0)
    }
}

impl Sum for Montant {
    fn sum<I: Iterator<Item = Montant>>(iter: I) -> Montant {
        iter.fold(Montant::ZERO, |acc, m| acc + m)
    }
}

impl fmt::Display for Montant {
    /// Format français : `1 234 567 FCFA` (espace fine insécable comme séparateur de milliers).
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let chiffres = self.0.unsigned_abs().to_string();
        let mut groupe = String::with_capacity(chiffres.len() + chiffres.len() / 3 * 3);
        for (i, c) in chiffres.chars().enumerate() {
            if i > 0 && (chiffres.len() - i).is_multiple_of(3) {
                groupe.push('\u{202F}');
            }
            groupe.push(c);
        }
        let signe = if self.0 < 0 { "-" } else { "" };
        write!(f, "{signe}{groupe} FCFA")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn m(v: i64) -> Montant {
        Montant::new(v).unwrap()
    }

    #[test]
    fn tva_18_pourcent_sur_10000() {
        let tva = m(10_000)
            .appliquer_taux(Taux::pourcentage("18").unwrap(), Arrondi::PlusProche)
            .unwrap();
        assert_eq!(tva, m(1_800));
    }

    #[test]
    fn arrondi_demi_franc() {
        // 25 × 18 % = 4,5 → 5 au plus proche, 4 à l'inférieur, 5 au supérieur
        let t = Taux::pourcentage("18").unwrap();
        assert_eq!(m(25).appliquer_taux(t, Arrondi::PlusProche).unwrap(), m(5));
        assert_eq!(m(25).appliquer_taux(t, Arrondi::Inferieur).unwrap(), m(4));
        assert_eq!(m(25).appliquer_taux(t, Arrondi::Superieur).unwrap(), m(5));
        // Montants négatifs (avoirs) : symétrie
        assert_eq!(
            m(-25).appliquer_taux(t, Arrondi::PlusProche).unwrap(),
            m(-5)
        );
    }

    #[test]
    fn bornes() {
        assert!(Montant::new(Montant::MAX).is_ok());
        assert!(Montant::new(Montant::MAX + 1).is_err());
        assert!(Montant::new(-Montant::MAX - 1).is_err());
        assert!(m(Montant::MAX).ajouter(m(1)).is_err());
    }

    #[test]
    fn affichage_francais() {
        assert_eq!(m(1_234_567).to_string(), "1\u{202F}234\u{202F}567 FCFA");
        assert_eq!(m(-500).to_string(), "-500 FCFA");
        assert_eq!(m(0).to_string(), "0 FCFA");
    }

    #[test]
    fn somme() {
        let total: Montant = [m(100), m(250), m(-50)].into_iter().sum();
        assert_eq!(total, m(300));
    }

    #[test]
    fn serde_entier() {
        assert_eq!(serde_json::to_string(&m(1500)).unwrap(), "1500");
        let lu: Montant = serde_json::from_str("-42").unwrap();
        assert_eq!(lu, m(-42));
        assert!(serde_json::from_str::<Montant>("1000000000000001").is_err());
        assert!(serde_json::from_str::<Montant>("12.5").is_err());
    }
}
