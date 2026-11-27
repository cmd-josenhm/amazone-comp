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
    /// # Exemple
    ///
    /// ```
    /// use ac_noyau::Montant;
    ///
    /// let parts = Montant::new(100).unwrap().repartir(&[1, 1, 1]).unwrap();
    /// let francs: Vec<i64> = parts.iter().map(|m| m.francs()).collect();
    /// assert_eq!(francs, vec![34, 33, 33]);
    /// ```
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
        let somme_parts = parts.iter().try_fold(0i128, |acc, &p| {
            acc.checked_add(i128::from(p))
                .ok_or(ErreurNoyau::Depassement)
        })?;
        if somme_parts == 0 {
            return Err(ErreurNoyau::RepartitionImpossible(format!(
                "la somme des parts est nulle : impossible de répartir {self}"
            )));
        }
        let total_abs = i128::from(total).abs();
        // Proportion de chaque part, arrondie au franc le plus proche (0,5 s'éloignant de zéro).
        // Produit exact en i128 : total ≤ 10^15 et part ≤ 9,2 × 10^18 donnent un produit
        // ≤ 9,2 × 10^33, en dessous du maximum de i128 ; le doublage du reste est contrôlé.
        let mut parts_calculees: Vec<i128> = parts
            .iter()
            .map(|&part| {
                let produit = total_abs * i128::from(part);
                let quotient = produit / somme_parts;
                let reste = produit % somme_parts;
                let double = reste.checked_mul(2).ok_or(ErreurNoyau::Depassement)?;
                if double >= somme_parts {
                    Ok(quotient + 1)
                } else {
                    Ok(quotient)
                }
            })
            .collect::<Resultat<Vec<i128>>>()?;
        // Ordre décroissant des parts d'origine ; le tri stable conserve l'ordre fourni à parts
        // égales, donc le premier indice est celui de la première part maximale.
        let mut ordre: Vec<usize> = (0..parts.len()).collect();
        ordre.sort_by(|&a, &b| parts[b].cmp(&parts[a]));
        let somme_arrondie = parts_calculees.iter().try_fold(0i128, |acc, &p| {
            acc.checked_add(p).ok_or(ErreurNoyau::Depassement)
        })?;
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
        let signe = if total < 0 { -1i128 } else { 1i128 };
        parts_calculees
            .into_iter()
            .map(|part| {
                // La somme des parts vaut le total et chaque part est positive : aucune part ne
                // peut dépasser le montant réparti, donc rester hors des bornes de `Montant`.
                let valeur = signe * part;
                let francs = i64::try_from(valeur)
                    .map_err(|_| ErreurNoyau::MontantHorsLimites(valeur))?;
                Montant::new(francs)
            })
            .collect()
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

    #[test]
    fn multiplication_par_quantite_exemples() {
        // 1 250 FCFA × 3,5 = 4 375 FCFA (exemple de l'issue n° 9)
        let prix = m(1_250);
        let trois_demis = Quantite::depuis_texte("3,5").unwrap();
        for arrondi in [Arrondi::PlusProche, Arrondi::Inferieur, Arrondi::Superieur] {
            assert_eq!(
                prix.multiplier_quantite(trois_demis, arrondi).unwrap(),
                m(4_375)
            );
        }
        // 0,333 × 1 001 FCFA = 333,333 → 333 FCFA au plus proche (exemple de l'issue n° 9)
        let tiers = Quantite::depuis_texte("0,333").unwrap();
        assert_eq!(
            m(1_001)
                .multiplier_quantite(tiers, Arrondi::PlusProche)
                .unwrap(),
            m(333)
        );
        assert_eq!(
            m(1_001)
                .multiplier_quantite(tiers, Arrondi::Inferieur)
                .unwrap(),
            m(333)
        );
        assert_eq!(
            m(1_001)
                .multiplier_quantite(tiers, Arrondi::Superieur)
                .unwrap(),
            m(334)
        );
        // Cas zéro, dans les deux sens
        assert_eq!(
            prix
                .multiplier_quantite(Quantite::ZERO, Arrondi::PlusProche)
                .unwrap(),
            Montant::ZERO
        );
        assert_eq!(
            Montant::ZERO
                .multiplier_quantite(trois_demis, Arrondi::PlusProche)
                .unwrap(),
            Montant::ZERO
        );
        // Cas d'arrondi exact à 0,5 : le demi-franc s'éloigne de zéro
        let demi = Quantite::depuis_texte("0,5").unwrap();
        assert_eq!(
            m(1).multiplier_quantite(demi, Arrondi::PlusProche).unwrap(),
            m(1)
        );
        assert_eq!(
            m(1).multiplier_quantite(demi, Arrondi::Inferieur).unwrap(),
            m(0)
        );
        assert_eq!(
            m(1).multiplier_quantite(demi, Arrondi::Superieur).unwrap(),
            m(1)
        );
        // Symétrie pour les avoirs (montants négatifs)
        assert_eq!(
            m(-1_250)
                .multiplier_quantite(trois_demis, Arrondi::PlusProche)
                .unwrap(),
            m(-4_375)
        );
        assert_eq!(
            m(-1)
                .multiplier_quantite(demi, Arrondi::PlusProche)
                .unwrap(),
            m(-1)
        );
        assert_eq!(
            m(-1).multiplier_quantite(demi, Arrondi::Inferieur).unwrap(),
            m(0)
        );
        // Multiplication par une unité exacte et dépassement des bornes
        assert_eq!(
            prix
                .multiplier_quantite(Quantite::UN, Arrondi::PlusProche)
                .unwrap(),
            prix
        );
        assert!(m(Montant::MAX)
            .multiplier_quantite(Quantite::depuis_texte("1,001").unwrap(), Arrondi::PlusProche)
            .is_err());
    }

    #[test]
    fn repartition_exemples() {
        // 100 FCFA en 3 parts égales → 34 / 33 / 33 (exemple de l'issue n° 9)
        let parts = m(100).repartir(&[1, 1, 1]).unwrap();
        assert_eq!(
            parts.iter().map(|p| p.francs()).collect::<Vec<_>>(),
            vec![34, 33, 33]
        );
        // Proportions exactes, sans écart d'arrondi : 100 en parts 1/1/2 → 25 / 25 / 50
        let parts = m(100).repartir(&[1, 1, 2]).unwrap();
        assert_eq!(
            parts.iter().map(|p| p.francs()).collect::<Vec<_>>(),
            vec![25, 25, 50]
        );
        // Écart d'arrondi négatif absorbé par la plus grande part : 102 en parts 1/1/2
        // (quotients 25,5 / 25,5 / 51 → 26 / 26 / 51 = 103 ; l'écart −1 va à la part de poids 2)
        let parts = m(102).repartir(&[1, 1, 2]).unwrap();
        assert_eq!(
            parts.iter().map(|p| p.francs()).collect::<Vec<_>>(),
            vec![26, 26, 50]
        );
        // Cas zéro : répartir zéro donne des parts nulles, même avec des parts nulles
        assert_eq!(m(0).repartir(&[1, 1, 1]).unwrap(), vec![Montant::ZERO; 3]);
        assert_eq!(m(0).repartir(&[0, 0]).unwrap(), vec![Montant::ZERO; 2]);
        // Une seule part : tout y va
        assert_eq!(m(118_000).repartir(&[1]).unwrap(), vec![m(118_000)]);
        // Une part nulle dans la répartition : rien pour elle
        let parts = m(100).repartir(&[0, 1]).unwrap();
        assert_eq!(
            parts.iter().map(|p| p.francs()).collect::<Vec<_>>(),
            vec![0, 100]
        );
        // Avoir négatif : −100 en 3 parts égales → −34 / −33 / −33
        let parts = m(-100).repartir(&[1, 1, 1]).unwrap();
        assert_eq!(
            parts.iter().map(|p| p.francs()).collect::<Vec<_>>(),
            vec![-34, -33, -33]
        );
        // Garde anti-négatif : 5 en 8 parts égales (quotients 0,625 → 1 chacun = 8)
        // → l'écart −3 réduit les trois premières parts sans jamais en rendre une négative
        let parts = m(5).repartir(&[1, 1, 1, 1, 1, 1, 1, 1]).unwrap();
        assert_eq!(
            parts.iter().map(|p| p.francs()).collect::<Vec<_>>(),
            vec![0, 0, 0, 1, 1, 1, 1, 1]
        );
        // Cas limite 0,5 exact : 6 en 4 parts égales (quotients 1,5 → 2 chacun = 8)
        let parts = m(6).repartir(&[1, 1, 1, 1]).unwrap();
        assert_eq!(
            parts.iter().map(|p| p.francs()).collect::<Vec<_>>(),
            vec![0, 2, 2, 2]
        );
        // Montant maximal : la répartition reste dans les bornes et conserve le total
        let parts = m(Montant::MAX).repartir(&[1, 1, 1]).unwrap();
        assert_eq!(
            parts.iter().map(|p| p.francs()).collect::<Vec<_>>(),
            vec![333_333_333_333_334, 333_333_333_333_333, 333_333_333_333_333]
        );
        // Erreurs : parts vides, part négative, somme de parts nulle pour un montant non nul
        assert!(m(100).repartir(&[]).is_err());
        assert!(m(100).repartir(&[1, -1]).is_err());
        assert!(m(100).repartir(&[0, 0]).is_err());
        let erreur = m(1_000).repartir(&[0, 0, 0]).unwrap_err();
        // Le montant est affiché avec des espaces fines insécables (voir `Display` de `Montant`).
        assert_eq!(
            erreur.to_string(),
            "répartition impossible : la somme des parts est nulle : \
             impossible de répartir 1\u{202F}000 FCFA"
        );
    }
}

#[cfg(test)]
mod tests_propristes {
    use super::*;
    use proptest::prelude::*;

    fn strategie_montant() -> impl Strategy<Value = Montant> {
        (-Montant::MAX..=Montant::MAX)
            .prop_map(|v| Montant::new(v).unwrap())
    }

    fn strategie_montant_moyen() -> impl Strategy<Value = Montant> {
        // |a| + |b| ≤ Montant::MAX : les méthodes contrôlées ne peuvent pas échouer aux bornes.
        (-Montant::MAX / 2..=Montant::MAX / 2)
            .prop_map(|v| Montant::new(v).unwrap())
    }

    proptest! {
        /// a + b − b == a, avec les opérateurs (les bornes rendent ces calculs exacts en i64).
        #[test]
        fn addition_et_soustraction_s_inversent(a in strategie_montant(), b in strategie_montant()) {
            prop_assert_eq!(a + b - b, a);
            prop_assert_eq!(b - a + a, b);
        }

        /// Idem avec les méthodes contrôlées, pour des montants restant dans les bornes.
        #[test]
        fn ajouter_soustraire_s_inversent(a in strategie_montant_moyen(), b in strategie_montant_moyen()) {
            prop_assert_eq!(a.ajouter(b).unwrap().soustraire(b).unwrap(), a);
            prop_assert_eq!(a.soustraire(b).unwrap().ajouter(b).unwrap(), a);
        }

        /// La répartition conserve toujours le total ; les parts gardent le signe du total et
        /// restent dans ses bornes. Le seul refus possible : total non nul et somme de parts nulle.
        #[test]
        fn repartir_conserve_le_total(
            total in strategie_montant(),
            parts in proptest::collection::vec(0i64..=1_000, 1..=8),
        ) {
            let somme_parts: i64 = parts.iter().sum();
            match total.repartir(&parts) {
                Ok(repartition) => {
                    for part in &repartition {
                        if total.est_negatif() {
                            prop_assert!(part.francs() <= 0);
                        } else {
                            prop_assert!(part.francs() >= 0);
                        }
                        prop_assert!(part.abs().francs() <= total.abs().francs());
                    }
                    prop_assert_eq!(repartition.into_iter().sum::<Montant>(), total);
                }
                Err(ErreurNoyau::RepartitionImpossible(_)) => {
                    prop_assert!(!total.est_zero());
                    prop_assert_eq!(somme_parts, 0);
                }
                Err(e) => prop_fail!("erreur inattendue : {e}"),
            }
        }

        /// Multiplier par une unité exacte ne change rien ; multiplier par zéro annule tout.
        #[test]
        fn multiplier_quantite_cas_triviaux(a in strategie_montant()) {
            for arrondi in [Arrondi::PlusProche, Arrondi::Inferieur, Arrondi::Superieur] {
                prop_assert_eq!(a.multiplier_quantite(Quantite::UN, arrondi).unwrap(), a);
                prop_assert_eq!(
                    a.multiplier_quantite(Quantite::ZERO, arrondi).unwrap(),
                    Montant::ZERO
                );
            }
        }

        /// La multiplication par une quantité positive conserve l'ordre, quel que soit l'arrondi.
        /// Quantité ≤ 1 unité : le résultat reste dans les bornes de `Montant`.
        #[test]
        fn multiplier_quantite_monotone(
            a in strategie_montant(),
            b in strategie_montant(),
            milliemes in 0i64..=1_000,
        ) {
            let q = Quantite::depuis_milliemes(milliemes).unwrap();
            for arrondi in [Arrondi::PlusProche, Arrondi::Inferieur, Arrondi::Superieur] {
                let pa = a.multiplier_quantite(q, arrondi).unwrap();
                let pb = b.multiplier_quantite(q, arrondi).unwrap();
                prop_assert_eq!(a <= b, pa <= pb);
            }
        }
    }
}
