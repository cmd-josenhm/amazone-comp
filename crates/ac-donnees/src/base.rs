//! Ouverture de la base et migrations.

use std::collections::BTreeMap;
use std::path::Path;

use rusqlite::Connection;
use zeroize::Zeroizing;

use crate::{CleBase, ErreurDonnees, Resultat, Transaction};

const HEXADECIMAL: &[u8; 16] = b"0123456789abcdef";

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
    /// Ouvre (ou crée) le fichier chiffré du dossier avec sa clé brute.
    ///
    /// Applique la clé SQLCipher sous forme hexadécimale brute puis lit immédiatement
    /// `sqlite_master`. Les valeurs de compatibilité ne sont pas remplacées : SQLCipher applique
    /// ses paramètres par défaut de version 4. Une clé incorrecte est détectée avant toute
    /// configuration susceptible d'écrire dans le fichier.
    pub fn ouvrir(chemin: &Path, cle: &CleBase) -> Resultat<Self> {
        let conn = Connection::open(chemin)?;
        appliquer_cle(&conn, cle)?;
        verifier_cle(&conn)?;
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

fn appliquer_cle(conn: &Connection, cle: &CleBase) -> Resultat<()> {
    let mut hex = Zeroizing::new(String::with_capacity(64));
    for octet in cle.octets() {
        hex.push(char::from(HEXADECIMAL[usize::from(*octet >> 4)]));
        hex.push(char::from(HEXADECIMAL[usize::from(*octet & 0x0f)]));
    }

    // Ne jamais retourner l'erreur SQLite brute : SqlInputError peut inclure le SQL fourni,
    // qui contient ici la clé. Les deux buffers intermédiaires sont effacés à leur destruction.
    let pragma = Zeroizing::new(format!("PRAGMA key = \"x'{}'\";", hex.as_str()));
    conn.execute_batch(pragma.as_str())
        .map_err(|_| ErreurDonnees::ChiffrementIndisponible)
}

fn verifier_cle(conn: &Connection) -> Resultat<()> {
    conn.query_row("SELECT count(*) FROM sqlite_master", [], |ligne| {
        ligne.get::<_, i64>(0)
    })
    .map(|_| ())
    .map_err(|_| ErreurDonnees::MotDePasseIncorrect)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::*;

    struct FichierTemporaire(PathBuf);

    impl FichierTemporaire {
        fn nouveau() -> Self {
            static SEQUENCE: AtomicUsize = AtomicUsize::new(0);
            let numero = SEQUENCE.fetch_add(1, Ordering::Relaxed);
            let chemin = std::env::temp_dir().join(format!(
                "Amazone Comp Sèna SQLCipher {} {numero}.acdb",
                std::process::id()
            ));
            let fichier = Self(chemin);
            fichier.supprimer();
            fichier
        }

        fn supprimer(&self) {
            let _ = fs::remove_file(&self.0);
            for suffixe in ["-wal", "-shm"] {
                let mut chemin = self.0.as_os_str().to_os_string();
                chemin.push(suffixe);
                let _ = fs::remove_file(PathBuf::from(chemin));
            }
        }
    }

    impl Drop for FichierTemporaire {
        fn drop(&mut self) {
            self.supprimer();
        }
    }

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

    #[test]
    fn fichier_sur_disque_est_chiffre() {
        let fichier = FichierTemporaire::nouveau();
        let cle = CleBase::depuis_octets([0x42; 32]);
        {
            let mut base = Base::ouvrir(&fichier.0, &cle).unwrap();
            assert_eq!(base.migrer(&[migrations()]).unwrap(), 1);
        }

        let contenu = fs::read(&fichier.0).unwrap();
        assert!(contenu.len() >= 16);
        assert_ne!(&contenu[..16], b"SQLite format 3\0");

        let base = Base::ouvrir(&fichier.0, &cle).unwrap();
        let migrations_enregistrees: i64 = base
            .connexion()
            .query_row("SELECT count(*) FROM donnees_migration", [], |ligne| {
                ligne.get(0)
            })
            .unwrap();
        assert_eq!(migrations_enregistrees, 1);
    }

    #[test]
    fn mauvaise_cle_refusee_sans_modifier_le_fichier() {
        let fichier = FichierTemporaire::nouveau();
        let bonne_cle = CleBase::depuis_octets([0x42; 32]);
        {
            let mut base = Base::ouvrir(&fichier.0, &bonne_cle).unwrap();
            base.migrer(&[migrations()]).unwrap();
        }
        let contenu_avant = fs::read(&fichier.0).unwrap();

        assert!(matches!(
            Base::ouvrir(&fichier.0, &CleBase::depuis_octets([0x24; 32])),
            Err(ErreurDonnees::MotDePasseIncorrect)
        ));

        let contenu_apres = fs::read(&fichier.0).unwrap();
        assert_eq!(contenu_apres, contenu_avant);
    }
}
