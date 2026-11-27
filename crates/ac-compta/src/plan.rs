//! Plan de comptes SYSCOHADA révisé.
//!
//! Classes : 1 ressources durables, 2 actif immobilisé, 3 stocks, 4 tiers, 5 trésorerie,
//! 6 charges des activités ordinaires, 7 produits des activités ordinaires,
//! 8 autres charges et produits (HAO, impôt sur le résultat), 9 comptabilité analytique / engagements.
//!
//! TODO(A1) : charger le plan complet (`plan_syscohada.csv`) avec libellés officiels, et les comptes
//! spécifiques utiles aux PME béninoises (TVA, AIB, Mobile Money 55x…), après validation expert.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::{ErreurCompta, Resultat};

/// Numéro de compte : 2 à 12 chiffres, la première chiffre est la classe (1 à 9).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct NumeroCompte(String);

impl NumeroCompte {
    /// Valide un numéro de compte.
    pub fn new(texte: &str) -> Resultat<Self> {
        let t = texte.trim();
        let valide = (2..=12).contains(&t.len())
            && t.chars().all(|c| c.is_ascii_digit())
            && !t.starts_with('0');
        if valide {
            Ok(NumeroCompte(t.to_owned()))
        } else {
            Err(ErreurCompta::CompteInvalide(texte.to_owned()))
        }
    }

    /// Classe SYSCOHADA (1 à 9).
    pub fn classe(&self) -> u8 {
        self.0.as_bytes().first().map_or(0, |b| b - b'0')
    }

    /// Vrai si ce compte est `racine` ou un sous-compte de `racine` (ex. 4111 ⊂ 411).
    pub fn est_sous_compte_de(&self, racine: &str) -> bool {
        self.0.starts_with(racine)
    }

    /// Valeur textuelle.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for NumeroCompte {
    type Error = ErreurCompta;
    fn try_from(s: String) -> Resultat<Self> {
        NumeroCompte::new(&s)
    }
}

impl From<NumeroCompte> for String {
    fn from(n: NumeroCompte) -> String {
        n.0
    }
}

impl fmt::Display for NumeroCompte {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numeros() {
        let c = NumeroCompte::new("4111").unwrap();
        assert_eq!(c.classe(), 4);
        assert!(c.est_sous_compte_de("411"));
        assert!(!c.est_sous_compte_de("401"));
        assert!(NumeroCompte::new("0123").is_err());
        assert!(NumeroCompte::new("4").is_err());
        assert!(NumeroCompte::new("41A").is_err());
    }
}
