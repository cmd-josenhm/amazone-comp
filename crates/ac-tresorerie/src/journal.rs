//! Journaux comptables déclarés par les comptes de trésorerie (table `compta_journal`, migration 4000).
//!
//! `A1` fournit [`ac_compta::journal::creer`] dans `ac-compta` (contrat C3) ; pour le Lot 1,
//! les journaux sont enregistrés ici via [`journal_creer`] et alloués via [`journal_prochant_code`]
//! (codes standards `BQn` et `MMn` du contrat C3).

use ac_donnees::Transaction;

use crate::{ErreurTresorerie, Resultat};

/// Code de journal valide : 2 à 4 caractères ASCII majuscules ou chiffres, au moins une lettre
/// (ex. `BQ1`, `MM2`, `CA`).
fn code_valide(code: &str) -> bool {
    (2..=4).contains(&code.len())
        && code
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
        && code.chars().any(|c| c.is_ascii_alphabetic())
}

/// Enregistre un journal dans `compta_journal` (idempotent : PK sur le code).
pub fn journal_creer(tx: &Transaction, code: &str, libelle: &str) -> Resultat<()> {
    if !code_valide(code) {
        return Err(ErreurTresorerie::JournalCodeInvalide(code.to_owned()));
    }
    tx.execute(
        "INSERT OR IGNORE INTO compta_journal (code, libelle) VALUES (?1, ?2)",
        [code, libelle],
    )?;
    Ok(())
}

/// Prochain code libre pour un préfixe donné : `BQ` → `BQ1`, `BQ2`, …
/// (lecture des codes déjà présents dans `compta_journal`).
pub fn journal_prochant_code(tx: &Transaction, prefixe: &str) -> Resultat<String> {
    let mut stmt = tx.prepare("SELECT code FROM compta_journal ORDER BY code")?;
    let mut codes: Vec<String> = Vec::new();
    let mut lignes = stmt.query([])?;
    while let Some(ligne) = lignes.next()? {
        let code: String = ligne.get(0)?;
        if code.starts_with(prefixe) {
            codes.push(code);
        }
    }
    let mut max = 0;
    for code in codes {
        if let Ok(n) = code.trim_start_matches(prefixe).parse::<i64>() {
            max = max.max(n);
        }
    }
    Ok(format!("{prefixe}{}", max + 1))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ac_donnees::{Base, Role, Session};

    fn base_test() -> Base {
        let mut base = Base::en_memoire().unwrap();
        base.migrer(&[ac_donnees::base::migrations(), crate::migrations()])
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
        assert!(!code_valide("bqq1"));
    }

    #[test]
    fn journal_prochant_code_a_completion() {
        let base = base_test();
        let tx = base.transaction().unwrap();
        assert_eq!(journal_prochant_code(&tx, "BQ").unwrap(), "BQ1");
    }

    #[test]
    fn journal_prochant_code_incremente() {
        let base = base_test();
        let tx = base.transaction().unwrap();
        journal_creer(&tx, "BQ1", "Banque OBA").unwrap();
        assert_eq!(journal_prochant_code(&tx, "BQ").unwrap(), "BQ2");
    }

    #[test]
    fn journal_creer_rejette_code_invalide() {
        let base = base_test();
        let tx = base.transaction().unwrap();
        let e = journal_creer(&tx, "bq1", "Banque OBA").unwrap_err();
        assert_eq!(e.code(), "JOURNAL_CODE_INVALIDE");
    }
}
