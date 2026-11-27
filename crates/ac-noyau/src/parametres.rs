//! Paramètres réglementaires versionnés par année.
//!
//! Arborescence : `parametres-fiscaux/<année>/<fichier>.toml`. Chaque valeur réglementaire est
//! un [`Valeur`] : la donnée + sa **source légale** + l'indicateur `verifie` (validation par un
//! expert-comptable). Règle R2 : aucun taux n'est codé en dur ailleurs.
//!
//! Les fichiers sont **embarqués dans l'exécutable** au moment de la compilation (voir
//! `ac-fiscal`), mais peuvent être relus depuis le disque en développement.

use std::path::Path;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::{ErreurNoyau, Resultat};

/// Valeur réglementaire accompagnée de sa source.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Valeur<T> {
    /// La valeur elle-même (taux, montant, date…).
    pub valeur: T,
    /// Référence légale : « CGI 2026, art. 229 », « LF 2026 (loi n° …), art. … ».
    pub source: String,
    /// `true` seulement après validation par un expert-comptable (décision du chef de projet).
    pub verifie: bool,
    /// Remarque libre (point d'attention, contradiction entre sources…).
    #[serde(default)]
    pub note: Option<String>,
}

/// En-tête obligatoire de chaque fichier de paramètres.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnTete {
    /// Année d'application (1er janvier → 31 décembre).
    pub annee: i32,
    /// Textes de référence de l'année.
    pub textes: Vec<String>,
    /// Date de dernière mise à jour du fichier (AAAA-MM-JJ).
    pub mis_a_jour_le: String,
}

/// Désérialise un fichier TOML de paramètres déjà chargé en mémoire.
pub fn lire_toml<T: DeserializeOwned>(nom_fichier: &str, contenu: &str) -> Resultat<T> {
    toml::from_str(contenu).map_err(|e| ErreurNoyau::ParametresInvalides {
        fichier: nom_fichier.to_owned(),
        detail: e.to_string(),
    })
}

/// Lit `racine/<annee>/<fichier>` depuis le disque (développement, tests).
pub fn charger_fichier<T: DeserializeOwned>(
    racine: &Path,
    annee: i32,
    fichier: &str,
) -> Resultat<T> {
    let chemin = racine.join(annee.to_string()).join(fichier);
    let contenu = std::fs::read_to_string(&chemin)
        .map_err(|e| ErreurNoyau::ParametresIntrouvables(format!("{} ({e})", chemin.display())))?;
    lire_toml(&chemin.display().to_string(), &contenu)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Taux;

    #[derive(Deserialize)]
    struct Exemple {
        en_tete: EnTete,
        taux: Valeur<Taux>,
    }

    #[test]
    fn lecture() {
        let t = r#"
            [en_tete]
            annee = 2026
            textes = ["CGI 2026"]
            mis_a_jour_le = "2026-11-27"

            [taux]
            valeur = "0.18"
            source = "CGI 2026"
            verifie = false
        "#;
        let e: Exemple = lire_toml("exemple.toml", t).unwrap();
        assert_eq!(e.en_tete.annee, 2026);
        assert_eq!(e.taux.valeur, Taux::pourcentage("18").unwrap());
        assert!(!e.taux.verifie);
    }

    #[test]
    fn erreur_lisible() {
        let r: Resultat<Exemple> = lire_toml("x.toml", "taux = 3");
        assert!(matches!(r, Err(ErreurNoyau::ParametresInvalides { .. })));
    }
}
