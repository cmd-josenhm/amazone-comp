//! Configuration de la plateforme : variables d'environnement et clé de signature.
//!
//! Aucun secret n'est stocké dans le dépôt (règle R7) : la clé privée de signature des
//! licences est lue depuis une variable d'environnement ou un fichier hors dépôt.

use thiserror::Error;

/// Nombre de transferts de poste autorisés par an si aucune variable n'est réglée.
pub const TRANSFERTS_PAR_AN_DEFAUT: i64 = 2;

/// Erreur de configuration au démarrage du service.
#[derive(Debug, Error)]
pub enum ErreurConfig {
    /// Une variable obligatoire manque ou est vide.
    #[error("Configuration incomplète : {0}. Vérifiez le fichier .env (modèle : .env.exemple).")]
    VariableManquante(String),
    /// La clé privée de signature est illisible ou invalide.
    #[error("Clé privée de licence illisible : {0}. Elle doit contenir 32 octets en hexadécimal (64 caractères).")]
    ClePriveeInvalide(String),
    /// Une valeur numérique est invalide.
    #[error("Valeur invalide pour {0} : {1}.")]
    ValeurInvalide(String, String),
}

/// Configuration chargée depuis l'environnement du processus.
#[derive(Debug, Clone)]
pub struct Config {
    /// URL de connexion PostgreSQL (`PLATEFORME_BDD_URL`).
    pub bdd_url: String,
    /// Adresse d'écoute HTTP (`PLATEFORME_ADRESSE`, défaut `0.0.0.0:8080`).
    pub adresse: String,
    /// Clé privée Ed25519 de signature des licences (32 octets).
    pub cle_privee: [u8; 32],
    /// Adresse d'expéditeur des courriels (`PLATEFORME_COURRIEL_EXPEDITEUR`).
    pub courriel_expediteur: String,
    /// Transferts de poste autorisés par an et par entreprise.
    pub transferts_poste_par_an: i64,
}

impl Config {
    /// Charge la configuration depuis les variables d'environnement.
    ///
    /// La clé privée est lue dans `PLATEFORME_CLE_PRIVEE_LICENCE` (hexadécimal) ou, à
    /// défaut, dans le fichier désigné par `PLATEFORME_FICHIER_CLE_PRIVEE` (hors dépôt).
    pub fn depuis_env() -> Result<Config, ErreurConfig> {
        let bdd_url = variable_obligatoire("PLATEFORME_BDD_URL")?;
        let adresse =
            std::env::var("PLATEFORME_ADRESSE").unwrap_or_else(|_| "0.0.0.0:8080".to_owned());
        let courriel_expediteur = std::env::var("PLATEFORME_COURRIEL_EXPEDITEUR")
            .unwrap_or_else(|_| "ne-pas-repondre@localhost".to_owned());
        let transferts_poste_par_an = match std::env::var("PLATEFORME_TRANSFERTS_PAR_AN") {
            Ok(texte) => i64::from_str_radix(&texte, 10).map_err(|_| {
                ErreurConfig::ValeurInvalide("PLATEFORME_TRANSFERTS_PAR_AN".into(), texte)
            })?,
            Err(_) => TRANSFERTS_PAR_AN_DEFAUT,
        };
        if transferts_poste_par_an < 1 {
            return Err(ErreurConfig::ValeurInvalide(
                "PLATEFORME_TRANSFERTS_PAR_AN".into(),
                "doit être supérieur ou égal à 1".into(),
            ));
        }
        let cle_privee = charger_cle_privee()?;
        Ok(Config {
            bdd_url,
            adresse,
            cle_privee,
            courriel_expediteur,
            transferts_poste_par_an,
        })
    }
}

fn variable_obligatoire(nom: &str) -> Result<String, ErreurConfig> {
    match std::env::var(nom) {
        Ok(v) if !v.trim().is_empty() => Ok(v),
        _ => Err(ErreurConfig::VariableManquante(nom.to_owned())),
    }
}

fn charger_cle_privee() -> Result<[u8; 32], ErreurConfig> {
    let (source, contenu) = if let Ok(hex) = std::env::var("PLATEFORME_CLE_PRIVEE_LICENCE") {
        ("PLATEFORME_CLE_PRIVEE_LICENCE", hex)
    } else if let Ok(chemin) = std::env::var("PLATEFORME_FICHIER_CLE_PRIVEE") {
        let texte = std::fs::read_to_string(&chemin).map_err(|e| {
            ErreurConfig::ClePriveeInvalide(format!("fichier « {chemin} » illisible ({e})"))
        })?;
        ("PLATEFORME_FICHIER_CLE_PRIVEE", texte)
    } else {
        return Err(ErreurConfig::VariableManquante(
            "PLATEFORME_CLE_PRIVEE_LICENCE (ou PLATEFORME_FICHIER_CLE_PRIVEE)".into(),
        ));
    };
    let hex = contenu.trim();
    octets_depuis_hex(hex).ok_or_else(|| ErreurConfig::ClePriveeInvalide(source.to_owned()))
}

/// Convertit 64 caractères hexadécimaux en 32 octets. Renvoie `None` si le texte est invalide.
pub fn octets_depuis_hex(texte: &str) -> Option<[u8; 32]> {
    if texte.len() != 64 || !texte.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    let mut octets = [0u8; 32];
    for (i, paire) in texte.as_bytes().chunks(2).enumerate() {
        let hi = valeur_hex(paire[0])?;
        let lo = valeur_hex(paire[1])?;
        octets[i] = hi * 16 + lo;
    }
    Some(octets)
}

/// Convertit 32 octets en 64 caractères hexadécimaux minuscules.
pub fn hex_depuis_octets(octets: &[u8]) -> String {
    const HEX: [u8; 16] = *b"0123456789abcdef";
    let mut sortie = String::with_capacity(octets.len() * 2);
    for o in octets {
        sortie.push(char::from(HEX[usize::from(o / 16)]));
        sortie.push(char::from(HEX[usize::from(o % 16)]));
    }
    sortie
}

fn valeur_hex(c: u8) -> Option<u8> {
    match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'a'..=b'f' => Some(c - b'a' + 10),
        b'A'..=b'F' => Some(c - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_aller_retour() {
        let octets = [7u8; 32];
        let texte = hex_depuis_octets(&octets);
        assert_eq!(texte.len(), 64);
        assert_eq!(octets_depuis_hex(&texte), Some(octets));
    }

    #[test]
    fn hex_majuscules_acceptees() {
        let mut octets = [0u8; 32];
        octets[0] = 0xAB;
        octets[31] = 0xCD;
        assert_eq!(
            octets_depuis_hex(&hex_depuis_octets(&octets).to_uppercase()),
            Some(octets)
        );
    }

    #[test]
    fn hex_invalides() {
        assert_eq!(octets_depuis_hex(""), None);
        assert_eq!(octets_depuis_hex("00"), None); // trop court
        assert_eq!(octets_depuis_hex(&"0".repeat(64)), Some([0u8; 32]));
        assert_eq!(octets_depuis_hex(&"g".repeat(64)), None);
        assert_eq!(octets_depuis_hex(&"00".repeat(33)), None); // trop long
    }

    #[test]
    fn conversion_octets_hex() {
        assert_eq!(hex_depuis_octets(&[0x00, 0x0F, 0xFF]), "000fff");
        assert_eq!(valeur_hex(b'z'), None);
    }
}
