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

use crate::{ErreurNoyau, Quantite, Resultat, Taux};

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

    /// Multiplie par une quantité entière (ex. prix unitaire × quantité entière).
    pub fn multiplier(self, quantite: i64) -> Resultat<Self> {
        self.0
            .checked_mul(quantite)
            .ok_or(ErreurNoyau::Depassement)
            .and_then(Montant::new)
    }

    /// Multiplie par une quantité exacte (millièmes) puis arrondit au franc.
    ///
    /// Le produit « francs × millièmes » est calculé exactement en entier 128 bits puis divisé
    /// par 1 000 selon la règle [`Arrondi`] demandée. Le 0,5 exact s'éloigne de zéro, comme pour
    /// [`Montant::appliquer_taux`].
    ///
    /// # Exemples
    ///
    /// ```
    /// use ac_noyau::{Arrondi, Montant, Quantite};
    ///
    /// let prix = Montant::new(1250).unwrap();
    /// let quantite = Quantite::depuis_texte("3,5").unwrap();
    /// // 1 250 FCFA × 3,5 = 4 375 FCFA
    /// assert_eq!(
    ///     prix.multiplier_quantite(quantite, Arrondi::PlusProche).unwrap().francs(),
    ///     4_375
    /// );
    /// // 1 001 FCFA × 0,333 = 333,333 → 333 FCFA au plus proche
    /// let tiers = Quantite::depuis_texte("0,333").unwrap();
    /// assert_eq!(
    ///     Montant::new(1001).unwrap().multiplier_quantite(tiers, Arrondi::PlusProche).unwrap().francs(),
    ///     333
    /// );
    /// ```
    pub fn multiplier_quantite(self, quantite: Quantite, arrondi: Arrondi) -> Resultat<Self> {
        // Produit exact en i128 : |francs| ≤ 10^15 et |millièmes| ≤ 10^15 donnent un produit
        // ≤ 10^30, très en dessous du maximum de i128 (≈ 1,7 × 10^38) : aucun débordement possible.
        let produit_milliemes = i128::from(self.0) * i128::from(quantite.milliemes());
        // Division exacte par 1 000 avec arrondi explicite ; le reste suit le signe du dividende.
        let quotient = produit_milliemes / 1_000;
        let reste = produit_milliemes % 1_000;
        let arrondi_francs = match arrondi {
            Arrondi::Inferieur => quotient,
            Arrondi::PlusProche => {
                if reste.abs() * 2 >= 1_000 {
                    quotient + produit_milliemes.signum()
                } else {
                    quotient
                }
            }
            Arrondi::Superieur => {
                if reste != 0 {
                    quotient + produit_milliemes.signum()
                } else {
                    quotient
                }
            }
        };
        let francs = i64::try_from(arrondi_francs)
            .map_err(|_| ErreurNoyau::MontantHorsLimites(arrondi_francs))?;
        Montant::new(francs)
    }

    /// Applique un taux puis arrondit au franc. Exemple : 10 000 × 18 % = 1 800.
    pub fn appliquer_taux(self, taux: Taux, arrondi: Arrondi) -> Resultat<Self> {
        let brut = Decimal::from(self.0)
            .checked_mul(taux.valeur())
            .ok_or(ErreurNoyau::Depassement)?;
        Self::depuis_decimal(brut, arrondi)
    }

    /// Répartit le montant en parts proportionnelles, **sans perdre un seul franc**.
    ///
    /// Chaque part reçoit sa proportion du total (arrondie au franc le plus proche, 0,5
    /// s'éloignant de zéro) ; la différence d'arrondi restante est absorbée par la part la plus
    /// grande (à parts égales, la première fournie). La somme des parts renvoyées est toujours
    /// exactement égale au montant réparti. Exemple : 100 FCFA en 3 parts égales → 34 / 33 / 33.
    ///
    /// Si absorber l'écart rendrait la plus grande part négative (total très petit devant le
    /// nombre de parts), l'excédent est reporté sur les parts suivantes, toujours de la plus
    /// grande à la plus petite : aucune part ne devient négative pour un total positif. Un
    /// montant négatif (avoir) est réparti selon la même règle sur sa valeur absolue puis rendu
    /// négatif : −100 en 3 parts égales donne −34 / −33 / −33.
    ///
    /// Répartir zéro renvoie toujours des parts nulles, même si les parts sont nulles.
    ///
    /// # Erreurs
    ///
    /// - parts vides ou contenant une valeur négative ;
    /// - somme des parts nulle alors que le montant n'est pas nul.
    ///
    pub fn repartir(self, parts: &[i64]) -> Resultat<Vec<Self>> {
        if parts.is_empty() {
            return Err(ErreurNoyau::RepartitionImpossible(
                "aucune part fournie pour la répartition".to_owned(),
            ));
        }
        for (position, &part) in parts.iter().enumerate() {
            if part < 0 {
                return Err(ErreurNoyau::RepartitionImpossible(format!(
                    "la part n°{} est négative : {}",
                    position + 1,
                    part
                )));
            }
        }
        let total = self.0;
        if total == 0 {
            return Ok(vec![Montant::ZERO; parts.len()]);
        }
        // Somme des parts en i128 : avec des parts de i64, un débordement exigerait davantage de
        // parts qu'il n'y a d'octets adressables, mais il reste contrôlé proprement.
        let mut somme_parts = 0i128;
        for &part in parts {
            somme_parts = somme_parts
                .checked_add(i128::from(part))
                .ok_or(ErreurNoyau::Depassement)?;
        }
        if somme_parts == 0 {
            return Err(ErreurNoyau::RepartitionImpossible(format!(
                "la somme des parts est nulle : impossible de répartir {self}"
            )));
        }
        let total_abs = i128::from(total).abs();
        // Proportion de chaque part, arrondie au franc le plus proche (0,5 s'éloignant de zéro).
        // Produit exact en i128 : total ≤ 10^15 et part ≤ 9,2 × 10^18 donnent un produit
        // ≤ 9,2 × 10^33, en dessous du maximum de i128 ; le doublage du reste est contrôlé.
        let mut parts_calculees: Vec<i128> = Vec::with_capacity(parts.len());
        for &part in parts {
            let produit = total_abs * i128::from(part);
            let quotient = produit / somme_parts;
            let reste = produit % somme_parts;
            let double = reste.checked_mul(2).ok_or(ErreurNoyau::Depassement)?;
            if double >= somme_parts {
                parts_calculees.push(quotient + 1);
            } else {
                parts_calculees.push(quotient);
            }
        }
        // Ordre décroissant des parts d'origine ; le tri stable conserve l'ordre fourni à parts
        // égales, donc le premier indice est celui de la première part maximale.
        let mut ordre: Vec<usize> = (0..parts.len()).collect();
        ordre.sort_by(|&a, &b| parts[b].cmp(&parts[a]));
        let mut somme_arrondie = 0i128;
        for &part in &parts_calculees {
            somme_arrondie = somme_arrondie
                .checked_add(part)
                .ok_or(ErreurNoyau::Depassement)?;
        }
        // |écart| < nombre de parts : chaque arrondi diffère de son quotient exact de moins d'un.
        let mut ecart = total_abs - somme_arrondie;
        if ecart > 0 {
            // Toute la différence d'arrondi va à la plus grande part.
            parts_calculees[ordre[0]] += ecart;
        } else if ecart < 0 {
            // Réduire en commençant par les plus grandes parts, sans descendre sous zéro.
            for &indice in &ordre {
                if ecart == 0 {
                    break;
                }
                let absorbe = ecart.max(-parts_calculees[indice]);
                parts_calculees[indice] += absorbe;
                ecart -= absorbe;
            }
            if ecart != 0 {
                // Inatteignable : un écart négatif implique que la somme arrondie dépasse le
                // total, donc qu'au moins une part calculée est strictement positive.
                return Err(ErreurNoyau::RepartitionImpossible(
                    "écart d'arrondi impossible à absorber".to_owned(),
                ));
            }
        }
        let signe = 1i128;
        // La somme des parts vaut le total et chaque part est positive : aucune part ne peut
        // dépasser le montant réparti, donc rester hors des bornes de `Montant`.
        let mut resultats = Vec::with_capacity(parts_calculees.len());
        for &part in &parts_calculees {
            let valeur_de_la_part = signe * part;
            let francs = i64::try_from(valeur_de_la_part)
                .map_err(|_| ErreurNoyau::MontantHorsLimites(valeur_de_la_part))?;
            resultats.push(Montant::new(francs)?);
        }
        Ok(resultats)
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