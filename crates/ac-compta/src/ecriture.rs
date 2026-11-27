//! Écritures comptables : brouillon, contrôle, enregistrement.

use ac_donnees::audit::{journaliser, Evenement};
use ac_donnees::{Permission, Session, Transaction};
use ac_noyau::{Date, Montant, Taux};
use serde::{Deserialize, Serialize};

use crate::{CodeFiscal, ErreurCompta, NumeroCompte, Resultat};

/// Identifiant d'une écriture enregistrée.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EcritureId(pub i64);

/// Module à l'origine d'une écriture (traçabilité document → écriture).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ModuleOrigine {
    /// Saisie manuelle (opérations diverses).
    Saisie,
    /// Ventes (factures, avoirs).
    Ventes,
    /// Achats (factures fournisseurs).
    Achats,
    /// Stocks (variations, inventaire).
    Stocks,
    /// Trésorerie (encaissements, décaissements, virements).
    Tresorerie,
    /// Paie.
    Paie,
    /// Fiscalité (liquidation TVA, IS…).
    Fiscal,
    /// Reprise de données (à-nouveaux importés).
    Reprise,
    /// Clôture / à-nouveaux.
    Cloture,
}

impl ModuleOrigine {
    fn code(self) -> &'static str {
        match self {
            ModuleOrigine::Saisie => "SAISIE",
            ModuleOrigine::Ventes => "VENTES",
            ModuleOrigine::Achats => "ACHATS",
            ModuleOrigine::Stocks => "STOCKS",
            ModuleOrigine::Tresorerie => "TRESORERIE",
            ModuleOrigine::Paie => "PAIE",
            ModuleOrigine::Fiscal => "FISCAL",
            ModuleOrigine::Reprise => "REPRISE",
            ModuleOrigine::Cloture => "CLOTURE",
        }
    }
}

/// Ligne d'une écriture à passer. Exactement **un** de `debit` / `credit` est non nul et positif.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LigneBrouillon {
    /// Compte mouvementé.
    pub compte: NumeroCompte,
    /// Libellé de la ligne (sinon celui de l'écriture).
    pub libelle: Option<String>,
    /// Montant au débit.
    pub debit: Montant,
    /// Montant au crédit.
    pub credit: Montant,
    /// Code du tiers (client, fournisseur, salarié) pour les comptes de classe 4.
    pub tiers: Option<String>,
    /// Étiquette fiscale (voir [`CodeFiscal`]).
    pub code_fiscal: Option<CodeFiscal>,
    /// Base imposable concernée (HT) quand `code_fiscal` est renseigné.
    pub base_fiscale: Option<Montant>,
    /// Taux appliqué quand `code_fiscal` est renseigné.
    pub taux: Option<Taux>,
}

impl LigneBrouillon {
    /// Ligne simple au débit.
    pub fn debit(compte: NumeroCompte, montant: Montant) -> Self {
        Self::simple(compte, montant, Montant::ZERO)
    }

    /// Ligne simple au crédit.
    pub fn credit(compte: NumeroCompte, montant: Montant) -> Self {
        Self::simple(compte, Montant::ZERO, montant)
    }

    fn simple(compte: NumeroCompte, debit: Montant, credit: Montant) -> Self {
        LigneBrouillon {
            compte,
            libelle: None,
            debit,
            credit,
            tiers: None,
            code_fiscal: None,
            base_fiscale: None,
            taux: None,
        }
    }
}

/// Écriture à passer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BrouillonEcriture {
    /// Code journal : `VE` ventes, `AC` achats, `BQ` banque, `CA` caisse, `MM` mobile money,
    /// `PA` paie, `OD` opérations diverses, `AN` à-nouveaux.
    pub journal: String,
    /// Date comptable.
    pub date: Date,
    /// Libellé général.
    pub libelle: String,
    /// Référence de la pièce justificative (n° facture…).
    pub piece: Option<String>,
    /// Module d'origine.
    pub origine: ModuleOrigine,
    /// Référence du document d'origine dans son module (ex. id de facture).
    pub reference_origine: Option<String>,
    /// Lignes (au moins deux).
    pub lignes: Vec<LigneBrouillon>,
}

/// Contrôles sans base de données : lignes valides et partie double équilibrée (règle R4).
pub fn verifier_brouillon(b: &BrouillonEcriture) -> Resultat<()> {
    if b.libelle.trim().is_empty() {
        return Err(ErreurCompta::LibelleVide);
    }
    if b.lignes.len() < 2 {
        return Err(ErreurCompta::TropPeuDeLignes);
    }
    let mut debit = Montant::ZERO;
    let mut credit = Montant::ZERO;
    for (i, l) in b.lignes.iter().enumerate() {
        let un_seul = l.debit.est_zero() != l.credit.est_zero();
        if !un_seul || l.debit.est_negatif() || l.credit.est_negatif() {
            return Err(ErreurCompta::LigneInvalide(i + 1));
        }
        debit = debit.ajouter(l.debit)?;
        credit = credit.ajouter(l.credit)?;
    }
    if debit != credit {
        return Err(ErreurCompta::Desequilibre { debit, credit });
    }
    Ok(())
}

/// Enregistre une écriture **validée** (donc immuable) dans la transaction de l'appelant.
///
/// TODO(A1) : contrôle de période ouverte, numérotation continue par journal et exercice,
/// contrôle des comptes mouvementables, tiers obligatoire sur 401/411/421…
pub fn passer_ecriture(
    tx: &Transaction<'_>,
    session: &Session,
    b: &BrouillonEcriture,
) -> Resultat<EcritureId> {
    session.exiger(Permission::Valider)?;
    verifier_brouillon(b)?;
    for l in &b.lignes {
        let existe: bool = tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM compta_compte WHERE numero = ?1)",
            [l.compte.as_str()],
            |r| r.get(0),
        )?;
        if !existe {
            return Err(ErreurCompta::CompteInconnu(l.compte.to_string()));
        }
    }
    tx.execute(
        "INSERT INTO compta_ecriture (journal, date, libelle, piece, origine, reference_origine, cree_par)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        ac_donnees::rusqlite::params![
            b.journal,
            b.date.to_string(),
            b.libelle,
            b.piece,
            b.origine.code(),
            b.reference_origine,
            session.utilisateur_id
        ],
    )?;
    let id = tx.last_insert_rowid();
    for (rang, l) in b.lignes.iter().enumerate() {
        tx.execute(
            "INSERT INTO compta_ligne
                (ecriture_id, rang, compte, libelle, debit, credit, tiers, code_fiscal, base_fiscale, taux)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            ac_donnees::rusqlite::params![
                id,
                rang as i64 + 1,
                l.compte.as_str(),
                l.libelle,
                l.debit.francs(),
                l.credit.francs(),
                l.tiers,
                l.code_fiscal
                    .and_then(|c| serde_json::to_value(c).ok())
                    .and_then(|v| v.as_str().map(str::to_owned)),
                l.base_fiscale.map(Montant::francs),
                l.taux.map(String::from),
            ],
        )?;
    }
    journaliser(
        tx,
        &Evenement {
            utilisateur_id: Some(session.utilisateur_id),
            action: "VALIDER",
            objet_type: "compta_ecriture",
            objet_id: &id.to_string(),
            details: &format!(
                "{{\"journal\":\"{}\",\"lignes\":{}}}",
                b.journal,
                b.lignes.len()
            ),
        },
    )?;
    Ok(EcritureId(id))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ac_donnees::{Base, Role};

    fn c(n: &str) -> NumeroCompte {
        NumeroCompte::new(n).unwrap()
    }
    fn m(v: i64) -> Montant {
        Montant::new(v).unwrap()
    }

    /// Facture de vente 100 000 HT + TVA 18 % = 118 000 TTC.
    fn facture() -> BrouillonEcriture {
        let mut tva = LigneBrouillon::credit(c("4431"), m(18_000));
        tva.code_fiscal = Some(CodeFiscal::TvaCollectee);
        tva.base_fiscale = Some(m(100_000));
        tva.taux = Some(Taux::pourcentage("18").unwrap());
        BrouillonEcriture {
            journal: "VE".into(),
            date: Date::from_ymd_opt(2026, 3, 15).unwrap(),
            libelle: "Facture F2026-0001".into(),
            piece: Some("F2026-0001".into()),
            origine: ModuleOrigine::Ventes,
            reference_origine: Some("1".into()),
            lignes: vec![
                LigneBrouillon::debit(c("4111"), m(118_000)),
                LigneBrouillon::credit(c("7011"), m(100_000)),
                tva,
            ],
        }
    }

    #[test]
    fn equilibre() {
        assert!(verifier_brouillon(&facture()).is_ok());
        let mut b = facture();
        b.lignes[0].debit = m(117_999);
        assert!(matches!(
            verifier_brouillon(&b),
            Err(ErreurCompta::Desequilibre { .. })
        ));
    }

    #[test]
    fn ligne_double_sens_refusee() {
        let mut b = facture();
        b.lignes[1].debit = m(1);
        assert!(matches!(
            verifier_brouillon(&b),
            Err(ErreurCompta::LigneInvalide(2))
        ));
    }

    #[test]
    fn enregistrement_et_audit() {
        let mut base = Base::en_memoire().unwrap();
        base.migrer(&[ac_donnees::base::migrations(), crate::migrations()])
            .unwrap();
        let admin = Session {
            utilisateur_id: 1,
            role: Role::Comptable,
        };
        let tx = base.transaction().unwrap();
        for n in ["4111", "7011", "4431"] {
            tx.execute(
                "INSERT INTO compta_compte (numero, libelle) VALUES (?1, ?1)",
                [n],
            )
            .unwrap();
        }
        let id = passer_ecriture(&tx, &admin, &facture()).unwrap();
        tx.commit().unwrap();
        assert_eq!(id, EcritureId(1));
        assert_eq!(
            ac_donnees::audit::verifier_chaine(base.connexion()).unwrap(),
            1
        );

        // Immuabilité : la ligne ne peut plus être modifiée.
        assert!(base
            .connexion()
            .execute(
                "UPDATE compta_ligne SET debit = 0 WHERE ecriture_id = 1",
                []
            )
            .is_err());

        // Un utilisateur « Saisie » ne peut pas valider.
        let saisie = Session {
            utilisateur_id: 2,
            role: Role::Saisie,
        };
        let tx = base.transaction().unwrap();
        assert!(passer_ecriture(&tx, &saisie, &facture()).is_err());
    }
}
