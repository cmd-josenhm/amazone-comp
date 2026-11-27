//! Impôt sur les traitements et salaires (ITS) — barème mensuel progressif par tranches.

use ac_noyau::parametres::{lire_toml, Valeur};
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

#[derive(Deserialize)]
struct FichierSocial {
    its_bareme_mensuel: Valeur<Vec<Tranche>>,
}

/// Barème embarqué pour 2026.
pub fn bareme_2026() -> Result<Vec<Tranche>, ErreurNoyau> {
    let f: FichierSocial = lire_toml(
        "social.toml",
        include_str!("../../../parametres-fiscaux/2026/social.toml"),
    )?;
    Ok(f.its_bareme_mensuel.valeur)
}

/// ITS mensuel sur un brut imposable mensuel. Chaque tranche est arrondie au franc inférieur.
///
/// TODO(A7) : confirmer la base imposable (brut moins CNSS salarié ?), les avantages en nature,
/// l'arrondi officiel, et le traitement des primes non mensuelles.
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

    fn m(v: i64) -> Montant {
        Montant::new(v).unwrap()
    }

    #[test]
    fn exemples_bareme_2026_non_verifie() {
        let b = bareme_2026().unwrap();
        assert_eq!(calculer_its(m(0), &b).unwrap(), m(0));
        assert_eq!(calculer_its(m(60_000), &b).unwrap(), m(0));
        // 100 000 : (100 000 − 60 000) × 10 % = 4 000
        assert_eq!(calculer_its(m(100_000), &b).unwrap(), m(4_000));
        // 300 000 : 90 000×10 % + 100 000×15 % + 50 000×19 % = 9 000 + 15 000 + 9 500 = 33 500
        assert_eq!(calculer_its(m(300_000), &b).unwrap(), m(33_500));
        // 1 000 000 : 9 000 + 15 000 + 47 500 + 500 000×30 % = 221 500
        assert_eq!(calculer_its(m(1_000_000), &b).unwrap(), m(221_500));
    }
}
