//! Calculs de TVA (fonctions pures).

use ac_noyau::{Arrondi, Montant, Taux};

use crate::Resultat;

/// Ventilation HT / TVA / TTC d'un montant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ventilation {
    /// Hors taxes.
    pub ht: Montant,
    /// TVA.
    pub tva: Montant,
    /// Toutes taxes comprises.
    pub ttc: Montant,
}

/// À partir d'un montant HT. La TVA est arrondie au franc le plus proche.
///
/// TODO(A8) : confirmer la règle d'arrondi officielle (par ligne ou sur le total de la facture,
/// cohérence avec le calcul e-MECeF) et la paramétrer.
pub fn depuis_ht(ht: Montant, taux: Taux) -> Resultat<Ventilation> {
    let tva = ht.appliquer_taux(taux, Arrondi::PlusProche)?;
    Ok(Ventilation {
        ht,
        tva,
        ttc: ht.ajouter(tva)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn m(v: i64) -> Montant {
        Montant::new(v).unwrap()
    }

    #[test]
    fn exemples_chiffres() {
        let t = Taux::pourcentage("18").unwrap();
        assert_eq!(
            depuis_ht(m(100_000), t).unwrap(),
            Ventilation {
                ht: m(100_000),
                tva: m(18_000),
                ttc: m(118_000)
            }
        );
        // 1 234 × 18 % = 222,12 → 222
        assert_eq!(depuis_ht(m(1_234), t).unwrap().tva, m(222));
        assert_eq!(depuis_ht(m(0), t).unwrap().ttc, m(0));
    }
}
