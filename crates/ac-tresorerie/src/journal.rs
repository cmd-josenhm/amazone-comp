//! Journaux comptables déclarés par les comptes de trésorerie.
//!
//! La branche principale ne fournit pas encore `ac_compta::journal::creer` ; ces deux fonctions
//! restent privées au crate et utilisent la structure `compta_journal` prévue par le contrat C3.

use ac_donnees::Transaction;

use crate::{ErreurTresorerie, Resultat};

/// Code de journal valide : 2 à 4 caractères ASCII majuscules ou chiffres, avec une lettre.
fn code_valide(code: &str) -> bool {
    (2..=4).contains(&code.len())
        && code
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
        && code.chars().any(|c| c.is_ascii_alphabetic())
}

/// Enregistre un journal de trésorerie (idempotent : clé primaire sur le code).
pub(crate) fn journal_creer(
    tx: &Transaction<'_>,
    code: &str,
    libelle: &str,
    compte_tresorerie: Option<&str>,
) -> Resultat<()> {
    if !code_valide(code) {
        return Err(ErreurTresorerie::JournalCodeInvalide(code.to_owned()));
    }
    if libelle.trim().is_empty() {
        return Err(ErreurTresorerie::JournalLibelleVide);
    }
    tx.execute(
        "INSERT OR IGNORE INTO compta_journal (code, libelle, type, compte_tresorerie)
         VALUES (?1, ?2, 'TRESORERIE', ?3)",
        ac_donnees::rusqlite::params![code, libelle, compte_tresorerie],
    )?;
    Ok(())
}

/// Prochain code libre : `BQ` → `BQ1`, `BQ2`, … ; `MM` → `MM1`, `MM2`, …
pub(crate) fn journal_prochain_code(
    tx: &Transaction<'_>,
    prefixe: &str,
) -> Resultat<String> {
    for numero in 1..=99 {
        let code = format!("{prefixe}{numero}");
        if !code_valide(&code) {
            return Err(ErreurTresorerie::JournalCodeInvalide(code));
        }
        let existe: bool = tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM compta_journal WHERE code = ?1)",
            [&code],
            |ligne| ligne.get(0),
        )?;
        if !existe {
            return Ok(code);
        }
    }
    Err(ErreurTresorerie::JournalIndisponible(
        prefixe.to_owned(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ac_donnees::Base;

    fn base_test() -> Base {
        let mut base = Base::en_memoire().unwrap();
        base.migrer(&[
            ac_donnees::base::migrations(),
            ac_compta::migrations(),
            crate::migrations(),
        ])
        .unwrap();
        base
    }

    #[test]
    fn code_valide_ok() {
        assert!(code_valide("BQ"));
        assert!(code_valide("BQ1"));
        assert!(code_valide("MM2"));
        assert!(!code_valide("Bq"));
        assert!(!code_valide("B"));
        assert!(!code_valide("1234"));
        assert!(!code_valide("bqq1"));
    }

    #[test]
    fn journal_prochain_code_commence_a_un() {
        let mut base = base_test();
        let tx = base.transaction().unwrap();
        assert_eq!(journal_prochain_code(&tx, "BQ").unwrap(), "BQ1");
    }

    #[test]
    fn journal_prochain_code_choisit_le_premier_libre() {
        let mut base = base_test();
        let tx = base.transaction().unwrap();
        journal_creer(&tx, "BQ1", "Banque OBA", None).unwrap();
        journal_creer(&tx, "BQ3", "Banque de l'Est", None).unwrap();
        assert_eq!(journal_prochain_code(&tx, "BQ").unwrap(), "BQ2");
    }

    #[test]
    fn journal_prochain_code_refuse_si_tous_utilises() {
        let mut base = base_test();
        let tx = base.transaction().unwrap();
        for numero in 1..=99 {
            let code = format!("BQ{numero}");
            journal_creer(&tx, &code, "Banque", None).unwrap();
        }
        assert!(matches!(
            journal_prochain_code(&tx, "BQ"),
            Err(ErreurTresorerie::JournalIndisponible(_))
        ));
    }

    #[test]
    fn journal_creer_rejette_code_invalide() {
        let mut base = base_test();
        let tx = base.transaction().unwrap();
        let erreur = journal_creer(&tx, "bq1", "Banque OBA", None).unwrap_err();
        assert_eq!(erreur.code(), "JOURNAL_CODE_INVALIDE");
    }

    #[test]
    fn journal_tresorerie_est_enregistre_dans_la_meme_base() {
        let mut base = base_test();
        let tx = base.transaction().unwrap();
        journal_creer(&tx, "CA", "Journal de caisse", None).unwrap();
        let type_journal: String = tx
            .query_row(
                "SELECT type FROM compta_journal WHERE code = 'CA'",
                [],
                |ligne| ligne.get(0),
            )
            .unwrap();
        assert_eq!(type_journal, "TRESORERIE");
    }
}
