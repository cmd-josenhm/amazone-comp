//! Balance comptable fournie en entrée du moteur d'états financiers.
//!
//! Tant que les fonctions de lecture d'`ac-compta` (lot 5 de A1) ne sont pas livrées,
//! le moteur d'états travaille sur des fonctions pures prenant une [`Balance`] en mémoire.

use std::cmp::Ordering;
use std::collections::BTreeSet;

use ac_compta::NumeroCompte;
use ac_noyau::{Montant, Periode};
use serde::{Deserialize, Serialize};

use crate::erreur::{ErreurEtats, Resultat};

/// Sens du solde net d'un compte.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SensSolde {
    /// Solde débiteur strict (`total_debit > total_credit`).
    Debiteur,
    /// Solde créditeur strict (`total_credit > total_debit`).
    Crediteur,
    /// Solde nul (`total_debit == total_credit`).
    Nul,
}

/// Ligne d'une balance comptable pour un compte donné.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LigneBalance {
    /// Numéro de compte SYSCOHADA.
    pub compte: NumeroCompte,
    /// Libellé du compte.
    pub libelle: String,
    /// Cumul des mouvements au débit (positif ou nul).
    pub total_debit: Montant,
    /// Cumul des mouvements au crédit (positif ou nul).
    pub total_credit: Montant,
}

impl LigneBalance {
    /// Crée une ligne de balance en vérifiant que les totaux débit et crédit sont positifs ou nuls.
    pub fn new(
        compte: NumeroCompte,
        libelle: impl Into<String>,
        total_debit: Montant,
        total_credit: Montant,
    ) -> Resultat<Self> {
        let ligne = Self {
            compte,
            libelle: libelle.into(),
            total_debit,
            total_credit,
        };
        ligne.verifier()?;
        Ok(ligne)
    }

    /// Crée une ligne de balance portant uniquement un solde débiteur.
    pub fn solde_debiteur(
        compte: NumeroCompte,
        libelle: impl Into<String>,
        montant: Montant,
    ) -> Resultat<Self> {
        Self::new(compte, libelle, montant, Montant::ZERO)
    }

    /// Crée une ligne de balance portant uniquement un solde créditeur.
    pub fn solde_crediteur(
        compte: NumeroCompte,
        libelle: impl Into<String>,
        montant: Montant,
    ) -> Resultat<Self> {
        Self::new(compte, libelle, Montant::ZERO, montant)
    }

    /// Vérifie que les montants débit et crédit sont positifs ou nuls.
    pub fn verifier(&self) -> Resultat<()> {
        if self.total_debit.est_negatif() || self.total_credit.est_negatif() {
            return Err(ErreurEtats::MontantNegatifBalance {
                compte: self.compte.to_string(),
            });
        }
        Ok(())
    }

    /// Sens du solde net (`total_debit` comparé à `total_credit`).
    pub fn sens_solde(&self) -> SensSolde {
        match self.total_debit.cmp(&self.total_credit) {
            Ordering::Greater => SensSolde::Debiteur,
            Ordering::Less => SensSolde::Crediteur,
            Ordering::Equal => SensSolde::Nul,
        }
    }

    /// Solde algébrique orienté débit (`total_debit - total_credit`).
    pub fn solde_algebrique_debiteur(&self) -> Resultat<Montant> {
        Ok(self.total_debit.soustraire(self.total_credit)?)
    }

    /// Solde algébrique orienté crédit (`total_credit - total_debit`).
    pub fn solde_algebrique_crediteur(&self) -> Resultat<Montant> {
        Ok(self.total_credit.soustraire(self.total_debit)?)
    }
}

/// Balance comptable d'un exercice N, avec comparatif éventuel de l'exercice N-1.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Balance {
    /// Période de l'exercice N (facultative).
    pub periode_n: Option<Periode>,
    /// Lignes de la balance de l'exercice N.
    pub lignes_n: Vec<LigneBalance>,
    /// Période de l'exercice N-1 (facultative).
    pub periode_n1: Option<Periode>,
    /// Lignes de la balance de l'exercice N-1 (vide si premier exercice).
    pub lignes_n1: Vec<LigneBalance>,
}

impl Balance {
    /// Construit une balance pour un seul exercice (sans comparatif N-1).
    pub fn exercice(lignes_n: Vec<LigneBalance>) -> Self {
        Self {
            periode_n: None,
            lignes_n,
            periode_n1: None,
            lignes_n1: Vec::new(),
        }
    }

    /// Construit une balance avec ses colonnes comparatives N et N-1.
    pub fn avec_comparatif(
        periode_n: Option<Periode>,
        lignes_n: Vec<LigneBalance>,
        periode_n1: Option<Periode>,
        lignes_n1: Vec<LigneBalance>,
    ) -> Self {
        Self {
            periode_n,
            lignes_n,
            periode_n1,
            lignes_n1,
        }
    }

    /// Vrai si la balance comporte des données comparatives pour N-1.
    pub fn possede_comparatif(&self) -> bool {
        !self.lignes_n1.is_empty()
    }

    /// Valide les lignes de N et N-1 (montants non négatifs et absence de doublons).
    pub fn valider(&self) -> Resultat<()> {
        valider_liste(&self.lignes_n, "N")?;
        valider_liste(&self.lignes_n1, "N-1")?;
        Ok(())
    }

    /// Somme des débits et des crédits de l'exercice N.
    pub fn totaux_n(&self) -> Resultat<(Montant, Montant)> {
        calculer_totaux(&self.lignes_n)
    }

    /// Somme des débits et des crédits de l'exercice N-1.
    pub fn totaux_n1(&self) -> Resultat<(Montant, Montant)> {
        calculer_totaux(&self.lignes_n1)
    }
}

fn valider_liste(lignes: &[LigneBalance], exercice: &'static str) -> Resultat<()> {
    let mut vus = BTreeSet::new();
    for ligne in lignes {
        ligne.verifier()?;
        if !vus.insert(ligne.compte.as_str()) {
            return Err(ErreurEtats::DoublonCompteBalance {
                compte: ligne.compte.to_string(),
                exercice,
            });
        }
    }
    Ok(())
}

fn calculer_totaux(lignes: &[LigneBalance]) -> Resultat<(Montant, Montant)> {
    let mut debit = Montant::ZERO;
    let mut credit = Montant::ZERO;
    for ligne in lignes {
        debit = debit.ajouter(ligne.total_debit)?;
        credit = credit.ajouter(ligne.total_credit)?;
    }
    Ok((debit, credit))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn c(n: &str) -> NumeroCompte {
        NumeroCompte::new(n).unwrap()
    }

    fn m(v: i64) -> Montant {
        Montant::new(v).unwrap()
    }

    #[test]
    fn sens_et_soldes() {
        let d = LigneBalance::solde_debiteur(c("521"), "Banque", m(250_000)).unwrap();
        assert_eq!(d.sens_solde(), SensSolde::Debiteur);
        assert_eq!(d.solde_algebrique_debiteur().unwrap(), m(250_000));
        assert_eq!(d.solde_algebrique_crediteur().unwrap(), m(-250_000));

        let cr = LigneBalance::solde_crediteur(c("101"), "Capital", m(1_000_000)).unwrap();
        assert_eq!(cr.sens_solde(), SensSolde::Crediteur);
        assert_eq!(cr.solde_algebrique_crediteur().unwrap(), m(1_000_000));

        let nul = LigneBalance::new(c("411"), "Client", m(100_000), m(100_000)).unwrap();
        assert_eq!(nul.sens_solde(), SensSolde::Nul);
        assert_eq!(nul.solde_algebrique_debiteur().unwrap(), Montant::ZERO);
    }

    #[test]
    fn refus_montant_negatif_et_doublon() {
        let neg = LigneBalance::new(c("521"), "Banque", m(-1), Montant::ZERO);
        assert!(matches!(
            neg,
            Err(ErreurEtats::MontantNegatifBalance { .. })
        ));

        let b = Balance::exercice(vec![
            LigneBalance::solde_debiteur(c("521"), "Banque 1", m(100)).unwrap(),
            LigneBalance::solde_debiteur(c("521"), "Banque 2", m(200)).unwrap(),
        ]);
        if let Err(ErreurEtats::DoublonCompteBalance { exercice, .. }) = b.valider() {
            assert_eq!(exercice, "N");
        } else {
            panic!("DoublonCompteBalance attendu");
        }
    }

    #[test]
    fn totaux_et_comparatif() {
        let b = Balance::avec_comparatif(
            Periode::annee(2026),
            vec![
                LigneBalance::solde_debiteur(c("521"), "Banque", m(500_000)).unwrap(),
                LigneBalance::solde_crediteur(c("101"), "Capital", m(500_000)).unwrap(),
            ],
            Periode::annee(2025),
            vec![
                LigneBalance::solde_debiteur(c("521"), "Banque", m(300_000)).unwrap(),
                LigneBalance::solde_crediteur(c("101"), "Capital", m(300_000)).unwrap(),
            ],
        );
        assert!(b.possede_comparatif());
        assert!(b.valider().is_ok());
        assert_eq!(b.totaux_n().unwrap(), (m(500_000), m(500_000)));
        assert_eq!(b.totaux_n1().unwrap(), (m(300_000), m(300_000)));
    }
}
