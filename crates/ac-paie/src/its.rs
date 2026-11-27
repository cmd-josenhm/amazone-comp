//! Impôt sur les traitements et salaires (ITS) — barème mensuel progressif par tranches.
//!
//! Le barème n'est jamais écrit ici : il vient de [`crate::ParametresSociaux`] (règle R2).

use ac_noyau::{Arrondi, ErreurNoyau, Montant, Taux};
use serde::Deserialize;

/// Tranche du barème : s'applique à la part du salaire comprise entre la borne précédente et
/// `jusqu_a` (inclus). `None` = tranche supérieure illimitée.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Tranche {
    /// Borne haute de la tranche (FCFA).
    pub jusqu_a: Option<Montant>,
    /// Taux marginal.
    pub taux: Taux,
}

/// ITS mensuel sur un brut imposable mensuel. Chaque tranche est arrondie au franc inférieur.
///
/// TODO(#44) : la base imposable (brut ou brut moins CNSS salarié) n'est pas tranchée. Ne pas
/// utiliser ce calcul pour un bulletin définitif avant confirmation. Les avantages en nature,
/// l'arrondi officiel et les primes non mensuelles restent à confirmer.
pub fn calculer_its(brut_imposable: Montant, bareme: &[Tranche]) -> Result<Montant, ErreurNoyau> {
    let mut impot = Montant::ZERO;
    let mut borne_basse = Montant::ZERO;
    for t in bareme {
        if brut_imposable <= borne_basse {
            break;
        }
        let haut = match t.jusqu_a {
            Some(h) if h < brut_imposable => h,
            _ => brut_imposable,
        };
        let part = haut.soustraire(borne_basse)?;
        impot = impot.ajouter(part.appliquer_taux(t.taux, Arrondi::Inferieur)?)?;
        match t.jusqu_a {
            Some(h) => borne_basse = h,
            None => break,
        }
    }
    Ok(impot)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ParametresSociaux;

    fn m(v: i64) -> Montant {
        Montant::new(v).unwrap()
    }

    fn bareme_2026() -> Vec<Tranche> {
        let parametres = ParametresSociaux::charger(2026).unwrap();
        parametres.bareme_its().to_vec()
    }

    #[test]
    fn zero_et_seuil_exact() {
        let b = bareme_2026();
        assert_eq!(calculer_its(m(0), &b).unwrap(), m(0));
        // Seuil exact : la tranche à 0 % s'arrête à 60 000 FCFA inclus.
        assert_eq!(calculer_its(m(60_000), &b).unwrap(), m(0));
        // 60 001 : 1 × 10 % = 0,1 → 0 (arrondi au franc inférieur).
        assert_eq!(calculer_its(m(60_001), &b).unwrap(), m(0));
        // 150 000 : (150 000 − 60 000) × 10 % = 9 000 (borne de la tranche à 10 %).
        assert_eq!(calculer_its(m(150_000), &b).unwrap(), m(9_000));
        // 150 001 : 9 000 + 1 × 15 % = 9 000,15 → 9 000.
        assert_eq!(calculer_its(m(150_001), &b).unwrap(), m(9_000));
    }

    #[test]
    fn exemples_bareme_2026() {
        let b = bareme_2026();
        // 100 000 : (100 000 − 60 000) × 10 % = 4 000
        assert_eq!(calculer_its(m(100_000), &b).unwrap(), m(4_000));
        // 300 000 : 9 000 + 15 000 + 50 000 × 19 % (9 500) = 33 500
        assert_eq!(calculer_its(m(300_000), &b).unwrap(), m(33_500));
        // 1 000 000 : 9 000 + 15 000 + 47 500 + 500 000 × 30 % (150 000) = 221 500
        assert_eq!(calculer_its(m(1_000_000), &b).unwrap(), m(221_500));
    }
}
