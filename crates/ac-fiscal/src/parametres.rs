//! Paramètres fiscaux embarqués dans l'exécutable.

use ac_noyau::parametres::{lire_toml, EnTete, Valeur};
use ac_noyau::{Montant, Taux};
use serde::Deserialize;

use crate::{ErreurFiscale, Resultat};

/// Fichiers d'une année : `(nom, contenu)`.
pub fn fichiers_embarques(annee: i32) -> Option<&'static [(&'static str, &'static str)]> {
    match annee {
        2026 => Some(&ANNEE_2026),
        _ => None,
    }
}

const ANNEE_2026: [(&str, &str); 9] = [
    (
        "tva.toml",
        include_str!("../../../parametres-fiscaux/2026/tva.toml"),
    ),
    (
        "aib.toml",
        include_str!("../../../parametres-fiscaux/2026/aib.toml"),
    ),
    (
        "impot-resultat.toml",
        include_str!("../../../parametres-fiscaux/2026/impot-resultat.toml"),
    ),
    (
        "tps.toml",
        include_str!("../../../parametres-fiscaux/2026/tps.toml"),
    ),
    (
        "retenues.toml",
        include_str!("../../../parametres-fiscaux/2026/retenues.toml"),
    ),
    (
        "penalites.toml",
        include_str!("../../../parametres-fiscaux/2026/penalites.toml"),
    ),
    (
        "calendrier.toml",
        include_str!("../../../parametres-fiscaux/2026/calendrier.toml"),
    ),
    (
        "emecef.toml",
        include_str!("../../../parametres-fiscaux/2026/emecef.toml"),
    ),
    (
        "social.toml",
        include_str!("../../../parametres-fiscaux/2026/social.toml"),
    ),
];

/// Paramètres TVA (`tva.toml`).
#[derive(Debug, Clone, Deserialize)]
pub struct ParametresTva {
    /// En-tête.
    pub en_tete: EnTete,
    /// Taux normal.
    pub taux_normal: Valeur<Taux>,
    /// Taux à l'exportation.
    pub taux_exportation: Valeur<Taux>,
    /// Seuil de CA HT au-delà duquel la TVA est obligatoire.
    pub seuil_assujettissement_ca_ht: Valeur<Montant>,
    /// Jour limite de déclaration (du mois suivant).
    pub jour_limite_declaration: Valeur<u32>,
    /// Limite de déduction BTP / industrie.
    pub limite_deduction_btp_industrie: Valeur<Taux>,
    /// Retenue de TVA lorsque le fournisseur est à la TPS.
    pub retenue_tva_sur_fournisseur_tps: Valeur<Taux>,
    /// Retenue de TVA pour les autres fournisseurs.
    pub retenue_tva_sur_fournisseur_reel: Valeur<Taux>,
}

/// Paramètres TPS (`tps.toml`).
#[derive(Debug, Clone, Deserialize)]
pub struct ParametresTps {
    /// En-tête.
    pub en_tete: EnTete,
    /// Taux sur le CA.
    pub taux: Valeur<Taux>,
    /// Minimum annuel.
    pub minimum: Valeur<Montant>,
    /// Seuil de CA HT.
    pub seuil_ca_ht: Valeur<Montant>,
    /// Redevance audiovisuelle.
    pub redevance_audiovisuelle: Valeur<Montant>,
}

/// Ensemble des paramètres typés d'une année (A8 complète au fur et à mesure).
#[derive(Debug, Clone)]
pub struct ParametresAnnee {
    /// Année.
    pub annee: i32,
    /// TVA.
    pub tva: ParametresTva,
    /// TPS.
    pub tps: ParametresTps,
}

impl ParametresAnnee {
    /// Charge les paramètres embarqués d'une année.
    pub fn charger(annee: i32) -> Resultat<Self> {
        let fichiers = fichiers_embarques(annee).ok_or(ErreurFiscale::AnneeInconnue(annee))?;
        let lire = |nom: &str| {
            fichiers
                .iter()
                .find(|(n, _)| *n == nom)
                .map(|(_, c)| *c)
                .ok_or(ErreurFiscale::AnneeInconnue(annee))
        };
        Ok(ParametresAnnee {
            annee,
            tva: lire_toml("tva.toml", lire("tva.toml")?)?,
            tps: lire_toml("tps.toml", lire("tps.toml")?)?,
        })
    }

    /// Nombre de valeurs non encore validées par un expert (affiché dans l'interface).
    pub fn nombre_non_verifiees(annee: i32) -> usize {
        fichiers_embarques(annee)
            .map(|f| {
                f.iter()
                    .map(|(_, c)| c.matches("verifie = false").count())
                    .sum()
            })
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Chaque table (hors `en_tete`) et chaque élément de tableau de tables doit porter
    /// `source` (texte non vide) et `verifie` (booléen) : règle R2.
    #[test]
    fn toutes_les_valeurs_ont_une_source() {
        for (nom, contenu) in fichiers_embarques(2026).unwrap() {
            let table: toml::Table =
                toml::from_str(contenu).unwrap_or_else(|e| panic!("{nom} illisible : {e}"));
            assert_eq!(
                table["en_tete"]["annee"].as_integer(),
                Some(2026),
                "{nom} : en_tete.annee"
            );
            for (cle, val) in &table {
                if cle == "en_tete" {
                    continue;
                }
                let elements: Vec<&toml::Table> = match val {
                    toml::Value::Table(t) => vec![t],
                    toml::Value::Array(a) => a.iter().filter_map(|v| v.as_table()).collect(),
                    _ => panic!(
                        "{nom} : « {cle} » doit être une table {{ valeur, source, verifie }}"
                    ),
                };
                for t in elements {
                    let source = t.get("source").and_then(|s| s.as_str()).unwrap_or("");
                    assert!(!source.trim().is_empty(), "{nom} : « {cle} » sans source");
                    assert!(
                        t.get("verifie").and_then(|v| v.as_bool()).is_some(),
                        "{nom} : « {cle} » sans indicateur verifie"
                    );
                }
            }
        }
    }

    #[test]
    fn chargement_type_2026() {
        let p = ParametresAnnee::charger(2026).unwrap();
        assert_eq!(p.tva.taux_normal.valeur, Taux::pourcentage("18").unwrap());
        assert_eq!(p.tps.minimum.valeur, Montant::new(10_000).unwrap());
        assert!(ParametresAnnee::nombre_non_verifiees(2026) > 0);
        assert!(matches!(
            ParametresAnnee::charger(1999),
            Err(ErreurFiscale::AnneeInconnue(1999))
        ));
    }
}
