//! Identifiants réglementaires béninois.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::{ErreurNoyau, Resultat};

/// Identifiant fiscal unique (IFU) délivré par la DGI du Bénin : **13 chiffres**.
///
/// À vérifier (issue `fiscal-a-verifier`) : existence d'une clé de contrôle officielle.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Ifu(String);

impl Ifu {
    /// Valide et crée un IFU (les espaces sont ignorés).
    pub fn new(texte: &str) -> Resultat<Self> {
        let nettoye: String = texte.chars().filter(|c| !c.is_whitespace()).collect();
        if nettoye.len() == 13 && nettoye.chars().all(|c| c.is_ascii_digit()) {
            Ok(Ifu(nettoye))
        } else {
            Err(ErreurNoyau::IfuInvalide(texte.to_owned()))
        }
    }

    /// Valeur textuelle.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for Ifu {
    type Error = ErreurNoyau;
    fn try_from(s: String) -> Resultat<Self> {
        Ifu::new(&s)
    }
}

impl From<Ifu> for String {
    fn from(i: Ifu) -> String {
        i.0
    }
}

impl fmt::Display for Ifu {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validation() {
        assert!(Ifu::new("3201912345678").is_ok());
        assert!(Ifu::new("3201 9123 45678").is_ok());
        assert!(Ifu::new("320191234567").is_err());
        assert!(Ifu::new("32019123456AB").is_err());
    }
}
