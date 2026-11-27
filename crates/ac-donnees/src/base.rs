//! Ouverture de la base et migrations.

use std::collections::BTreeMap;
use std::path::Path;

use rusqlite::Connection;

use crate::{ErreurDonnees, Resultat, Transaction};

/// Migration SQL d'une crate. Les numéros sont réservés par plage (voir `docs/architecture.md` §4) :
/// 0001–0999 ac-donnees, 1000–1999 ac-compta, 2000 ac-ventes, 3000 ac-achats, 4000 ac-tresorerie,
/// 5000 ac-paie, 6000 ac-fiscal, 7000 ac-etats, 8000 ac-licence.
///
/// **Une migration publiée n'est jamais modifiée.**
#[derive(Debug, Clone, Copy)]
pub struct Migration {
    /// Numéro global unique.
    pub version: u32,
    /// Nom court lisible (`"creer_ecritures"`).
    pub nom: &'static str,
    /// Script SQL (plusieurs instructions autorisées).
    pub sql: &'static str,
}

/// Migrations propres à `ac-donnees`.
pub fn migrations() -> &'static [Migration] {
    &[Migration {
        version: 1,
        nom: "socle",
        sql: include_str!("../migrations/0001_socle.sql"),
    }]
}

/// Dossier d'entreprise ouvert.
pub struct Base {
    conn: Connection,
}

impl Base {
    /// Ouvre (ou crée) le fichier du dossier.
    ///
    /// TODO(A2) : chiffrement SQLCipher (`PRAGMA key`), clé enveloppée par Argon2id (ADR 0005).
    pub fn ouvrir(chemin: &Path) -> Resultat<Self> {
        let conn = Connection::open(chemin)?;
        Self::configurer(conn)
    }

    /// Base en mémoire, pour les tests de toutes les crates.
    pub fn en_memoire() -> Resultat<Self> {
        Self::configurer(Connection::open_in_memory()?)
    }

    fn configurer(conn: Connection) -> Resultat<Self> {
        conn.execute_batch(
            "PRAGMA foreign_keys = ON;
             PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;",
        )?;
        Ok(Base { conn })
    }

    /// Applique dans l'ordre global toutes les migrations non encore appliquées.
    /// `lots` contient la liste de chaque crate (`&[ac_donnees::migrations(), ac_compta::migrations(), …]`).
    pub fn migrer(&mut self, lots: &[&[Migration]]) -> Resultat<u32> {
        let mut toutes: BTreeMap<u32, Migration> = BTreeMap::new();
        for lot in lots {
            for m in *lot {
                if toutes.insert(m.version, *m).is_some() {
                    return Err(ErreurDonnees::MigrationInvalide(format!(
                        "numéro {} utilisé deux fois",
                        m.version
                    )));
                }
            }
        }
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS donnees_migration (
                version INTEGER PRIMARY KEY,
                nom TEXT NOT NULL,
                appliquee_le TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
             );",
        )?;
        let mut appliquees = 0;
        for m in toutes.values() {
            let deja: bool = self.conn.query_row(
                "SELECT EXISTS(SELECT 1 FROM donnees_migration WHERE version = ?1)",
                [m.version],
                |r| r.get(0),
            )?;
            if deja {
                continue;
            }
            let tx = self.conn.transaction()?;
            tx.execute_batch(m.sql)?;
            tx.execute(
                "INSERT INTO donnees_migration (version, nom) VALUES (?1, ?2)",
                rusqlite::params![m.version, m.nom],
            )?;
            tx.commit()?;
            appliquees += 1;
        }
        Ok(appliquees)
    }

    /// Démarre une transaction d'écriture.
    pub fn transaction(&mut self) -> Resultat<Transaction<'_>> {
        Ok(self.conn.transaction()?)
    }

    /// Accès en lecture.
    pub fn connexion(&self) -> &Connection {
        &self.conn
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrations_idempotentes() {
        let mut b = Base::en_memoire().unwrap();
        assert_eq!(b.migrer(&[migrations()]).unwrap(), 1);
        assert_eq!(b.migrer(&[migrations()]).unwrap(), 0);
    }

    #[test]
    fn doublon_refuse() {
        let mut b = Base::en_memoire().unwrap();
        let r = b.migrer(&[migrations(), migrations()]);
        assert!(matches!(r, Err(ErreurDonnees::MigrationInvalide(_))));
    }

    const INSERER_ENTREPRISE: &str = "INSERT INTO donnees_entreprise (id, raison_sociale, forme_juridique,
        date_creation, ifu, rccm, regime_fiscal, centre_impots, secteur, activite_principale, departement,
        commune, adresse, telephone, courriel, responsable_nom, responsable_prenoms, responsable_fonction,
        responsable_telephone, responsable_courriel, systeme_comptable, debut_exercice)
        VALUES (1, 'Boutique Exemple SARL', 'SARL', '2021-03-15', ?1, 'RB/COT/21 B 12345', 'REEL_SIMPLIFIE',
        'CIME Cotonou', 'COMMERCE', 'Fournitures de bureau', 'LITTORAL', 'Cotonou', 'Ganhi',
        '+2290197123456', 'contact@exemple.bj', 'AHOUANSOU', 'Marie', 'Gérante', '+2290166001122',
        'marie@exemple.bj', 'NORMAL', '2026-01-01')";

    #[test]
    fn fiche_entreprise_contraintes() {
        let mut b = Base::en_memoire().unwrap();
        b.migrer(&[migrations()]).unwrap();
        let c = b.connexion();
        // IFU de 12 chiffres puis IFU avec une lettre : refusés par la base
        assert!(c.execute(INSERER_ENTREPRISE, ["320191234567"]).is_err());
        assert!(c.execute(INSERER_ENTREPRISE, ["320191234567A"]).is_err());
        assert_eq!(c.execute(INSERER_ENTREPRISE, ["3201912345678"]).unwrap(), 1);
        // un seul dossier par base
        assert!(c.execute(INSERER_ENTREPRISE, ["3201912345678"]).is_err());
    }
}
