//! Journal d'audit chaîné.
//!
//! Chaque entrée contient le hash de la précédente : `hash = SHA-256(hash_precedent | horodatage |
//! utilisateur | action | objet_type | objet_id | details)`. Toute modification d'une entrée passée
//! casse la chaîne et est détectée par [`verifier_chaine`].

use sha2::{Digest, Sha256};

use crate::{Connexion, ErreurDonnees, Resultat, Transaction};

/// Hash initial de la chaîne.
pub const GENESE: &str = "0000000000000000000000000000000000000000000000000000000000000000";

/// Événement à journaliser.
#[derive(Debug, Clone)]
pub struct Evenement<'a> {
    /// Utilisateur à l'origine (None = système).
    pub utilisateur_id: Option<i64>,
    /// Verbe : `CREER`, `VALIDER`, `CONTREPASSER`…
    pub action: &'a str,
    /// Table / type d'objet.
    pub objet_type: &'a str,
    /// Identifiant de l'objet.
    pub objet_id: &'a str,
    /// Détails au format JSON canonique (clés triées).
    pub details: &'a str,
}

fn calculer_hash(precedent: &str, horodatage: &str, e: &Evenement<'_>) -> String {
    let mut h = Sha256::new();
    for morceau in [
        precedent,
        horodatage,
        &e.utilisateur_id.map(|u| u.to_string()).unwrap_or_default(),
        e.action,
        e.objet_type,
        e.objet_id,
        e.details,
    ] {
        h.update(morceau.as_bytes());
        h.update([0x1f]);
    }
    h.finalize().iter().map(|b| format!("{b:02x}")).collect()
}

/// Ajoute un événement au journal, dans la transaction de l'opération journalisée.
pub fn journaliser(tx: &Transaction<'_>, e: &Evenement<'_>) -> Resultat<()> {
    let precedent: String = tx
        .query_row(
            "SELECT hash FROM donnees_audit ORDER BY id DESC LIMIT 1",
            [],
            |r| r.get(0),
        )
        .or_else(|err| match err {
            rusqlite::Error::QueryReturnedNoRows => Ok(GENESE.to_owned()),
            autre => Err(autre),
        })?;
    let horodatage = chrono::Utc::now()
        .format("%Y-%m-%dT%H:%M:%S%.3fZ")
        .to_string();
    let hash = calculer_hash(&precedent, &horodatage, e);
    tx.execute(
        "INSERT INTO donnees_audit
            (horodatage, utilisateur_id, action, objet_type, objet_id, details, hash_precedent, hash)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        rusqlite::params![
            horodatage,
            e.utilisateur_id,
            e.action,
            e.objet_type,
            e.objet_id,
            e.details,
            precedent,
            hash
        ],
    )?;
    Ok(())
}

/// Recalcule toute la chaîne. Renvoie le nombre d'entrées vérifiées.
pub fn verifier_chaine(conn: &Connexion) -> Resultat<i64> {
    let mut stmt = conn.prepare(
        "SELECT id, horodatage, utilisateur_id, action, objet_type, objet_id, details,
                hash_precedent, hash
         FROM donnees_audit ORDER BY id",
    )?;
    let mut lignes = stmt.query([])?;
    let mut attendu = GENESE.to_owned();
    let mut n = 0;
    while let Some(l) = lignes.next()? {
        let id: i64 = l.get(0)?;
        let horodatage: String = l.get(1)?;
        let utilisateur_id: Option<i64> = l.get(2)?;
        let action: String = l.get(3)?;
        let objet_type: String = l.get(4)?;
        let objet_id: String = l.get(5)?;
        let details: String = l.get(6)?;
        let precedent: String = l.get(7)?;
        let hash: String = l.get(8)?;
        let e = Evenement {
            utilisateur_id,
            action: &action,
            objet_type: &objet_type,
            objet_id: &objet_id,
            details: &details,
        };
        if precedent != attendu || calculer_hash(&precedent, &horodatage, &e) != hash {
            return Err(ErreurDonnees::AuditAltere(id));
        }
        attendu = hash;
        n += 1;
    }
    Ok(n)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Base;

    #[test]
    fn chaine_valide_puis_alteree() {
        let mut b = Base::en_memoire().unwrap();
        b.migrer(&[crate::base::migrations()]).unwrap();
        {
            let tx = b.transaction().unwrap();
            for i in 0..3 {
                let id = i.to_string();
                journaliser(
                    &tx,
                    &Evenement {
                        utilisateur_id: Some(1),
                        action: "CREER",
                        objet_type: "test",
                        objet_id: &id,
                        details: "{}",
                    },
                )
                .unwrap();
            }
            tx.commit().unwrap();
        }
        assert_eq!(verifier_chaine(b.connexion()).unwrap(), 3);

        // Les déclencheurs interdisent la modification…
        assert!(b
            .connexion()
            .execute(
                "UPDATE donnees_audit SET details = '{\"x\":1}' WHERE id = 2",
                []
            )
            .is_err());
        // …et une altération directe du fichier serait détectée par verifier_chaine.
    }
}
