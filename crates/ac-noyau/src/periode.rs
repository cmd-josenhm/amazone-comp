//! Périodes (mois, trimestre, exercice).

use chrono::{Datelike, NaiveDate};
use serde::{Deserialize, Serialize};

use crate::{ErreurNoyau, Resultat};

/// Intervalle de dates **fermé** `[debut, fin]`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Periode {
    debut: NaiveDate,
    fin: NaiveDate,
}

impl Periode {
    /// Crée une période ; erreur si `debut > fin`.
    pub fn new(debut: NaiveDate, fin: NaiveDate) -> Resultat<Self> {
        if debut > fin {
            return Err(ErreurNoyau::PeriodeInvalide { debut, fin });
        }
        Ok(Periode { debut, fin })
    }

    /// Mois civil complet.
    pub fn mois(annee: i32, mois: u32) -> Option<Self> {
        let debut = NaiveDate::from_ymd_opt(annee, mois, 1)?;
        let suivant = if mois == 12 {
            NaiveDate::from_ymd_opt(annee + 1, 1, 1)?
        } else {
            NaiveDate::from_ymd_opt(annee, mois + 1, 1)?
        };
        Some(Periode {
            debut,
            fin: suivant.pred_opt()?,
        })
    }

    /// Année civile complète (exercice standard au Bénin : 1er janvier – 31 décembre).
    pub fn annee(annee: i32) -> Option<Self> {
        Some(Periode {
            debut: NaiveDate::from_ymd_opt(annee, 1, 1)?,
            fin: NaiveDate::from_ymd_opt(annee, 12, 31)?,
        })
    }

    /// Premier jour.
    pub fn debut(&self) -> NaiveDate {
        self.debut
    }

    /// Dernier jour.
    pub fn fin(&self) -> NaiveDate {
        self.fin
    }

    /// Vrai si la date est dans la période.
    pub fn contient(&self, d: NaiveDate) -> bool {
        self.debut <= d && d <= self.fin
    }

    /// Année de la date de fin (année fiscale de rattachement).
    pub fn annee_fiscale(&self) -> i32 {
        self.fin.year()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fevrier_bissextile() {
        let p = Periode::mois(2028, 2).unwrap();
        assert_eq!(p.fin(), NaiveDate::from_ymd_opt(2028, 2, 29).unwrap());
        let p = Periode::mois(2026, 12).unwrap();
        assert_eq!(p.fin(), NaiveDate::from_ymd_opt(2026, 12, 31).unwrap());
    }

    #[test]
    fn invalide() {
        let a = NaiveDate::from_ymd_opt(2026, 5, 1).unwrap();
        let b = NaiveDate::from_ymd_opt(2026, 4, 1).unwrap();
        assert!(Periode::new(a, b).is_err());
        assert!(Periode::new(b, a).unwrap().contient(b));
    }
}
