//! Tables de correspondance comptes SYSCOHADA révisé → rubriques des états financiers.
//!
//! Source : Acte uniforme OHADA relatif au droit comptable et à l'information financière
//! (AUDCIF, 2017), modèle des états financiers du Système normal.

use ac_compta::NumeroCompte;
use serde::{Deserialize, Serialize};

use crate::balance::SensSolde;
use crate::erreur::{ErreurEtats, Resultat};

const CSV_SYSCOHADA: &str = include_str!("../donnees/correspondances_syscohada.csv");

/// État financier cible d'une règle de correspondance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EtatFinancier {
    /// Bilan — Actif.
    BilanActif,
    /// Bilan — Passif.
    BilanPassif,
    /// Compte de résultat.
    CompteResultat,
}

/// Colonne ou section de destination dans l'état financier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ColonneBilan {
    /// Colonne Brut de l'actif.
    Brut,
    /// Colonne Amortissements et dépréciations de l'actif (déduite du brut).
    AmortDeprec,
    /// Colonne Net du passif.
    Passif,
    /// Rubrique du compte de résultat.
    Resultat,
}

/// Sens de solde exigé pour appliquer la règle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SensAttendu {
    /// Solde débiteur uniquement.
    Debiteur,
    /// Solde créditeur uniquement.
    Crediteur,
    /// Tout solde (débiteur ou créditeur).
    Mixte,
}

/// Signe conventionnel de la rubrique dans les états SYSCOHADA.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SigneRubrique {
    /// Rubrique positive (`+`).
    Plus,
    /// Rubrique soustractive (`-`).
    Moins,
    /// Rubrique à variation positive ou négative (`+/-`).
    PlusOuMoins,
}

/// Motif de rattachement : préfixe de compte ou plage inclusive de préfixes.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MotifCompte {
    /// Préfixe exact (ex. `"601"` pour tous les comptes `601*`).
    Prefixe(String),
    /// Plage inclusive de préfixes de même longueur (ex. `"101"`..=`"104"`).
    Plage {
        /// Borne basse inclusive.
        debut: String,
        /// Borne haute inclusive.
        fin: String,
    },
}

impl MotifCompte {
    /// Nombre de chiffres du préfixe (sert de niveau de spécificité).
    pub fn longueur(&self) -> usize {
        match self {
            MotifCompte::Prefixe(p) => p.len(),
            MotifCompte::Plage { debut, .. } => debut.len(),
        }
    }

    /// Vrai si le numéro de compte correspond à ce motif.
    pub fn correspond(&self, compte: &NumeroCompte) -> bool {
        let num = compte.as_str();
        match self {
            MotifCompte::Prefixe(p) => num.starts_with(p),
            MotifCompte::Plage { debut, fin } => {
                let Some(prefixe) = num.get(..debut.len()) else {
                    return false;
                };
                (debut.as_str()..=fin.as_str()).contains(&prefixe)
            }
        }
    }
}

/// Règle unitaire de correspondance entre un compte (ou plage) et une rubrique SYSCOHADA.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegleCorrespondance {
    /// Compte ou plage de comptes concerné.
    pub motif: MotifCompte,
    /// État financier cible.
    pub etat: EtatFinancier,
    /// Colonne cible.
    pub colonne: ColonneBilan,
    /// Code référence de la rubrique (`AE`, `BI`, `CA`, `TA`, `RA`…).
    pub rubrique: String,
    /// Libellé officiel de la rubrique.
    pub libelle_rubrique: String,
    /// Condition sur le sens du solde du compte.
    pub sens_solde: SensAttendu,
    /// Signe officiel de la rubrique.
    pub signe: SigneRubrique,
}

/// Table complète des correspondances comptes → rubriques.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Correspondances {
    regles: Vec<RegleCorrespondance>,
}

impl Correspondances {
    /// Charge la table officielle embarquée du Système normal SYSCOHADA révisé (AUDCIF 2017).
    pub fn syscohada() -> Resultat<Self> {
        Self::depuis_csv(CSV_SYSCOHADA)
    }

    /// Charge une table de correspondances à partir d'un texte CSV (séparateur `;`).
    pub fn depuis_csv(contenu: &str) -> Resultat<Self> {
        let mut regles = Vec::new();
        let mut entete_lue = false;

        for (index, ligne_brute) in contenu.lines().enumerate() {
            let num_ligne = index + 1;
            let ligne = ligne_brute.trim();
            if ligne.is_empty() || ligne.starts_with('#') {
                continue;
            }
            if !entete_lue && ligne.starts_with("compte_ou_plage;") {
                entete_lue = true;
                continue;
            }
            entete_lue = true;
            regles.push(analyser_ligne_csv(ligne, num_ligne)?);
        }

        if regles.is_empty() {
            return Err(ErreurEtats::CorrespondanceInvalide {
                ligne: 1,
                detail: "aucune règle de correspondance trouvée".to_owned(),
            });
        }

        Ok(Self { regles })
    }

    /// Retourne toutes les règles chargées.
    pub fn regles(&self) -> &[RegleCorrespondance] {
        &self.regles
    }

    /// Résout la rubrique applicable à un compte selon le sens de son solde net.
    ///
    /// Applique d'abord le préfixe le plus spécifique, puis sélectionne la règle compatible avec
    /// `sens` (ex. banque créditrice → découvert en Trésorerie-Passif `DR`).
    pub fn resoudre(
        &self,
        compte: &NumeroCompte,
        sens: SensSolde,
    ) -> Resultat<&RegleCorrespondance> {
        let mut candidates = Vec::new();
        let mut max_longueur = 0;
        for r in &self.regles {
            if r.motif.correspond(compte) {
                max_longueur = max_longueur.max(r.motif.longueur());
                candidates.push(r);
            }
        }
        if candidates.is_empty() {
            return Err(ErreurEtats::CompteNonRattache {
                compte: compte.to_string(),
            });
        }

        let specifiques: Vec<&RegleCorrespondance> = candidates
            .into_iter()
            .filter(|r| r.motif.longueur() == max_longueur)
            .collect();

        choisir_par_sens(&specifiques, sens).ok_or_else(|| ErreurEtats::CompteNonRattache {
            compte: compte.to_string(),
        })
    }
}

fn choisir_par_sens<'a>(
    specifiques: &[&'a RegleCorrespondance],
    sens: SensSolde,
) -> Option<&'a RegleCorrespondance> {
    match sens {
        SensSolde::Debiteur => trouver_sens(specifiques, SensAttendu::Debiteur)
            .or_else(|| trouver_sens(specifiques, SensAttendu::Mixte)),
        SensSolde::Crediteur => trouver_sens(specifiques, SensAttendu::Crediteur)
            .or_else(|| trouver_sens(specifiques, SensAttendu::Mixte)),
        SensSolde::Nul => trouver_sens(specifiques, SensAttendu::Mixte)
            .or_else(|| trouver_sens(specifiques, SensAttendu::Debiteur))
            .or_else(|| trouver_sens(specifiques, SensAttendu::Crediteur)),
    }
}

fn trouver_sens<'a>(
    specifiques: &[&'a RegleCorrespondance],
    attendu: SensAttendu,
) -> Option<&'a RegleCorrespondance> {
    specifiques
        .iter()
        .copied()
        .find(|r| r.sens_solde == attendu)
}

fn analyser_ligne_csv(ligne: &str, num_ligne: usize) -> Resultat<RegleCorrespondance> {
    let champs: Vec<&str> = ligne.split(';').collect();
    if champs.len() != 7 {
        let nb = champs.len();
        return Err(ErreurEtats::CorrespondanceInvalide {
            ligne: num_ligne,
            detail: format!("7 colonnes attendues, {nb} trouvée(s)"),
        });
    }

    let motif = analyser_motif(champs[0].trim(), num_ligne)?;
    let etat = analyser_etat(champs[1].trim(), num_ligne)?;
    let colonne = analyser_colonne(champs[2].trim(), etat, num_ligne)?;
    let rubrique = champs[3].trim();
    if rubrique.is_empty() || !rubrique_detail_valide(etat, rubrique) {
        let c_etat = champs[1];
        return Err(ErreurEtats::CorrespondanceInvalide {
            ligne: num_ligne,
            detail: format!("rubrique « {rubrique} » inconnue pour {c_etat}"),
        });
    }
    let libelle_rubrique = champs[4].trim();
    if libelle_rubrique.is_empty() {
        return Err(ErreurEtats::CorrespondanceInvalide {
            ligne: num_ligne,
            detail: "libellé de rubrique vide".to_owned(),
        });
    }
    let sens_solde = analyser_sens(champs[5].trim(), num_ligne)?;
    let signe = analyser_signe(champs[6].trim(), num_ligne)?;

    Ok(RegleCorrespondance {
        motif,
        etat,
        colonne,
        rubrique: rubrique.to_owned(),
        libelle_rubrique: libelle_rubrique.to_owned(),
        sens_solde,
        signe,
    })
}

fn analyser_motif(texte: &str, ligne: usize) -> Resultat<MotifCompte> {
    if let Some((debut, fin)) = texte.split_once('-') {
        let d = debut.trim();
        let f = fin.trim();
        if !prefixe_valide(d) || !prefixe_valide(f) || d.len() != f.len() || d > f {
            return Err(ErreurEtats::CorrespondanceInvalide {
                ligne,
                detail: format!("plage de comptes invalide : « {texte} »"),
            });
        }
        return Ok(MotifCompte::Plage {
            debut: d.to_owned(),
            fin: f.to_owned(),
        });
    }
    if !prefixe_valide(texte) {
        return Err(ErreurEtats::CorrespondanceInvalide {
            ligne,
            detail: format!("préfixe de compte invalide : « {texte} »"),
        });
    }
    Ok(MotifCompte::Prefixe(texte.to_owned()))
}

fn prefixe_valide(texte: &str) -> bool {
    (1..=12).contains(&texte.len())
        && texte.chars().all(|c| c.is_ascii_digit())
        && !texte.starts_with('0')
}

fn analyser_etat(texte: &str, ligne: usize) -> Resultat<EtatFinancier> {
    match texte {
        "BILAN_ACTIF" => Ok(EtatFinancier::BilanActif),
        "BILAN_PASSIF" => Ok(EtatFinancier::BilanPassif),
        "COMPTE_RESULTAT" => Ok(EtatFinancier::CompteResultat),
        _ => Err(ErreurEtats::CorrespondanceInvalide {
            ligne,
            detail: format!("état financier inconnu : « {texte} »"),
        }),
    }
}

fn analyser_colonne(texte: &str, etat: EtatFinancier, ligne: usize) -> Resultat<ColonneBilan> {
    let col = match texte {
        "BRUT" => ColonneBilan::Brut,
        "AMORT_DEPREC" => ColonneBilan::AmortDeprec,
        "PASSIF" => ColonneBilan::Passif,
        "RESULTAT" => ColonneBilan::Resultat,
        _ => {
            return Err(ErreurEtats::CorrespondanceInvalide {
                ligne,
                detail: format!("colonne inconnue : « {texte} »"),
            });
        }
    };
    let compatible = matches!(
        (etat, col),
        (
            EtatFinancier::BilanActif,
            ColonneBilan::Brut | ColonneBilan::AmortDeprec
        ) | (EtatFinancier::BilanPassif, ColonneBilan::Passif)
            | (EtatFinancier::CompteResultat, ColonneBilan::Resultat)
    );
    if !compatible {
        return Err(ErreurEtats::CorrespondanceInvalide {
            ligne,
            detail: format!("colonne « {texte} » incompatible avec l'état ciblé"),
        });
    }
    Ok(col)
}

fn analyser_sens(texte: &str, ligne: usize) -> Resultat<SensAttendu> {
    match texte {
        "D" => Ok(SensAttendu::Debiteur),
        "C" => Ok(SensAttendu::Crediteur),
        "M" => Ok(SensAttendu::Mixte),
        _ => Err(ErreurEtats::CorrespondanceInvalide {
            ligne,
            detail: format!("sens de solde inconnu : « {texte} »"),
        }),
    }
}

fn analyser_signe(texte: &str, ligne: usize) -> Resultat<SigneRubrique> {
    match texte {
        "+" => Ok(SigneRubrique::Plus),
        "-" => Ok(SigneRubrique::Moins),
        "+/-" => Ok(SigneRubrique::PlusOuMoins),
        _ => Err(ErreurEtats::CorrespondanceInvalide {
            ligne,
            detail: format!("signe inconnu : « {texte} »"),
        }),
    }
}

fn rubrique_detail_valide(etat: EtatFinancier, rubrique: &str) -> bool {
    crate::etats::rubrique_detail_existe(etat, rubrique)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn c(n: &str) -> NumeroCompte {
        NumeroCompte::new(n).unwrap()
    }

    #[test]
    fn table_syscohada_se_charge_sans_erreur() {
        let corr = Correspondances::syscohada().unwrap();
        assert!(!corr.regles().is_empty());
    }

    #[test]
    fn specificite_et_reclassement_solde_inverse() {
        let corr = Correspondances::syscohada().unwrap();

        // 245 (transport) prime sur 24 (autres matériels).
        let r_2451 = corr.resoudre(&c("2451"), SensSolde::Debiteur).unwrap();
        assert_eq!(r_2451.rubrique, "AN");
        let r_2441 = corr.resoudre(&c("2441"), SensSolde::Debiteur).unwrap();
        assert_eq!(r_2441.rubrique, "AM");

        // 521 débiteur → BS (Trésorerie-Actif) ; 521 créditeur → DR (Trésorerie-Passif).
        let bq_d = corr.resoudre(&c("5211"), SensSolde::Debiteur).unwrap();
        assert_eq!(bq_d.etat, EtatFinancier::BilanActif);
        assert_eq!(bq_d.rubrique, "BS");

        let bq_c = corr.resoudre(&c("5212"), SensSolde::Crediteur).unwrap();
        assert_eq!(bq_c.etat, EtatFinancier::BilanPassif);
        assert_eq!(bq_c.rubrique, "DR");

        // 411 créditeur → DI (Clients avances) ; 401 débiteur → BH (Fournisseurs avances).
        let r_4111 = corr.resoudre(&c("4111"), SensSolde::Crediteur).unwrap();
        assert_eq!(r_4111.rubrique, "DI");
        let r_4011 = corr.resoudre(&c("4011"), SensSolde::Debiteur).unwrap();
        assert_eq!(r_4011.rubrique, "BH");
    }

    #[test]
    fn compte_non_rattache_message_explicite() {
        let corr = Correspondances::syscohada().unwrap();
        let res_9011 = corr.resoudre(&c("9011"), SensSolde::Debiteur);
        let msg_9011 = "Le compte 9011 n'est rattaché à aucune rubrique";
        assert_eq!(res_9011.unwrap_err().to_string(), msg_9011);

        // Table personnalisée sans le compte 4711 → message exact exigé par le cahier des charges.
        let csv = "101;BILAN_PASSIF;PASSIF;CA;Capital;M;+\n";
        let partielle = Correspondances::depuis_csv(csv).unwrap();
        let res_4711 = partielle.resoudre(&c("4711"), SensSolde::Debiteur);
        let msg_4711 = "Le compte 4711 n'est rattaché à aucune rubrique";
        assert_eq!(res_4711.unwrap_err().to_string(), msg_4711);
    }

    fn refuser_csv(cas: &str) {
        assert!(
            matches!(
                Correspondances::depuis_csv(cas),
                Err(ErreurEtats::CorrespondanceInvalide { .. })
            ),
            "CorrespondanceInvalide attendu pour {cas}"
        );
    }

    #[test]
    fn csv_invalide_refuse() {
        refuser_csv("");
        refuser_csv("101;BILAN_PASSIF;PASSIF;CA;Capital;M");
        refuser_csv("01;BILAN_PASSIF;PASSIF;CA;Capital;M;+");
        refuser_csv("105-101;BILAN_PASSIF;PASSIF;CA;Capital;M;+");
        refuser_csv("101;INCONNU;PASSIF;CA;Capital;M;+");
        refuser_csv("101;BILAN_PASSIF;INCONNUE;CA;Capital;M;+");
        refuser_csv("101;BILAN_ACTIF;PASSIF;CA;Capital;M;+");
        refuser_csv("101;BILAN_PASSIF;PASSIF;ZZ;Capital;M;+");
        refuser_csv("101;BILAN_PASSIF;PASSIF;CA;;M;+");
        refuser_csv("101;BILAN_PASSIF;PASSIF;CA;Capital;X;+");
        refuser_csv("101;BILAN_PASSIF;PASSIF;CA;Capital;M;*");
    }

    #[test]
    fn solde_nul_sur_compte_mixte_accepte() {
        let corr = Correspondances::syscohada().unwrap();
        let r = corr.resoudre(&c("4111"), SensSolde::Nul).unwrap();
        assert_eq!(r.rubrique, "BI");
        let r_bq = corr.resoudre(&c("5211"), SensSolde::Nul).unwrap();
        assert_eq!(r_bq.rubrique, "BS");
        let csv_c = "512;BILAN_PASSIF;PASSIF;DR;Banque;C;+\n";
        let corr_c = Correspondances::depuis_csv(csv_c).unwrap();
        assert_eq!(
            corr_c.resoudre(&c("512"), SensSolde::Nul).unwrap().rubrique,
            "DR"
        );
    }
}
