//! Paramètres sociaux typés : `parametres-fiscaux/<année>/social.toml` (ITS, VPS, CNSS, SMIG,
//! redevance audiovisuelle sur salaires).
//!
//! - Règle R2 : aucun taux, seuil ni barème n'est écrit dans le Rust. Tout vient du fichier.
//! - Règle R14 : chaque valeur porte sa source légale. Une valeur incertaine reste
//!   `verifie = false` et renvoie vers une issue `fiscal-a-verifier`.
//! - Contrat C10 : l'année applicable est celle de la **date de l'opération** (date de paie),
//!   pas la date du jour. Les fichiers sont embarqués dans l'exécutable.

use ac_noyau::parametres::{lire_toml, EnTete, Valeur};
use ac_noyau::{Date, Montant, Taux};
use chrono::Datelike;
use serde::Deserialize;

use crate::erreur::ErreurPaie;
use crate::its::Tranche;

/// Contenu embarqué de `social.toml` pour une année, s'il existe.
fn fichier_embarque(annee: i32) -> Option<&'static str> {
    match annee {
        2026 => Some(include_str!("../../../parametres-fiscaux/2026/social.toml")),
        _ => None,
    }
}

/// Base imposable de l'ITS. [`BaseIts::NonTranchee`] tant que l'issue #44 n'est pas close.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BaseIts {
    /// Salaire brut.
    Brut,
    /// Salaire brut diminué de la cotisation CNSS salarié.
    BrutMoinsCnssSalarie,
    /// Pas encore tranchée par une source écrite : aucun calcul définitif.
    NonTranchee,
}

/// Périodicité de déclaration et de paiement de la CNSS.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Periodicite {
    /// Déclaration et paiement chaque mois.
    Mensuelle,
    /// Déclaration et paiement chaque trimestre.
    Trimestrielle,
    /// Pas encore tranchée : la périodicité dépend de l'effectif (issue #47).
    NonTranchee,
}

/// Montant de la redevance audiovisuelle retenue sur salaires pour un mois donné.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub struct EcheanceRedevance {
    /// Mois de la retenue (1 = janvier … 12 = décembre).
    pub mois: u32,
    /// Montant retenu, en FCFA.
    pub montant: Montant,
}

/// Majoration d'heures supplémentaires (issue #49 : liste encore vide).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Majoration {
    /// Catégorie d'heures (jour, nuit, dimanche ou jour férié…).
    pub categorie: String,
    /// Taux de majoration.
    pub taux: Taux,
}

/// Paramètres sociaux d'une année, tels que déclarés dans `social.toml`.
///
/// `deny_unknown_fields` : toute clé du fichier absente de cette struct est refusée au
/// chargement. La couverture complète du fichier est donc garantie (voir les tests).
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ParametresSociaux {
    /// En-tête du fichier : année d'application, textes de référence, mise à jour.
    pub en_tete: EnTete,
    /// Barème mensuel progressif de l'ITS.
    pub its_bareme_mensuel: Valeur<Vec<Tranche>>,
    /// Base imposable de l'ITS (issue #44).
    pub its_base_imposable: Valeur<BaseIts>,
    /// Redevance audiovisuelle retenue sur salaires, par mois (issue #46).
    pub its_redevance_audiovisuelle: Valeur<Vec<EcheanceRedevance>>,
    /// Taux de la VPS (versement patronal sur les salaires).
    pub vps_taux: Valeur<Taux>,
    /// Taux de la VPS pour les établissements privés d'enseignement.
    pub vps_taux_enseignement_prive: Valeur<Taux>,
    /// `true` si les entreprises soumises à la TPS sont dispensées de VPS.
    pub vps_exonere_tps: Valeur<bool>,
    /// Prestations familiales, à la charge de l'employeur.
    pub cnss_prestations_familiales_employeur: Valeur<Taux>,
    /// Pension vieillesse, part employeur.
    pub cnss_pension_employeur: Valeur<Taux>,
    /// Pension vieillesse, part salarié.
    pub cnss_pension_salarie: Valeur<Taux>,
    /// Risques professionnels : taux minimum.
    pub cnss_risques_professionnels_min: Valeur<Taux>,
    /// Risques professionnels : taux maximum.
    pub cnss_risques_professionnels_max: Valeur<Taux>,
    /// Plafond mensuel de l'assiette CNSS. `0` = pas de plafond (issue #45).
    pub cnss_plafond_mensuel: Valeur<Montant>,
    /// Périodicité de déclaration et de paiement CNSS (issue #47).
    pub cnss_periodicite: Valeur<Periodicite>,
    /// Majoration de retard par mois, en cas de paiement tardif.
    pub cnss_majoration_retard_mensuelle: Valeur<Taux>,
    /// Majorations des heures supplémentaires (issue #49). Liste vide = non renseignées.
    pub heures_supplementaires_majorations: Valeur<Vec<Majoration>>,
    /// Salaire minimum interprofessionnel garanti, mensuel (issue #48).
    pub smig_mensuel: Valeur<Montant>,
}

/// Clé d'un paramètre, avec sa source et son indicateur de vérification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceValeur<'a> {
    /// Clé TOML du paramètre.
    pub cle: &'static str,
    /// Référence légale.
    pub source: &'a str,
    /// `true` seulement après validation par un expert-comptable.
    pub verifie: bool,
}

fn source<'a, T>(cle: &'static str, valeur: &'a Valeur<T>) -> SourceValeur<'a> {
    SourceValeur {
        cle,
        source: valeur.source.as_str(),
        verifie: valeur.verifie,
    }
}

fn incoherent(cle: &'static str, detail: &str) -> ErreurPaie {
    ErreurPaie::ParametreIncoherent {
        cle,
        detail: detail.to_owned(),
    }
}

/// Le barème doit être non vide, avec des bornes strictement croissantes, la seule tranche sans
/// borne étant la dernière.
fn valider_bareme(bareme: &[Tranche]) -> Result<(), ErreurPaie> {
    const CLE: &str = "its_bareme_mensuel";
    if bareme.is_empty() {
        return Err(incoherent(CLE, "barème vide"));
    }
    let derniere = bareme.len().saturating_sub(1);
    let mut borne = Montant::ZERO;
    for (i, tranche) in bareme.iter().enumerate() {
        match tranche.jusqu_a {
            Some(haut) if i < derniere => {
                if haut <= borne {
                    return Err(incoherent(CLE, "bornes non strictement croissantes"));
                }
                borne = haut;
            }
            Some(_) => return Err(incoherent(CLE, "la dernière tranche doit être sans borne")),
            None if i == derniere => {}
            None => return Err(incoherent(CLE, "seule la dernière tranche peut être sans borne")),
        }
    }
    Ok(())
}

fn valider_redevance(echeances: &[EcheanceRedevance]) -> Result<(), ErreurPaie> {
    for echeance in echeances {
        if !(1..=12).contains(&echeance.mois) {
            return Err(incoherent(
                "its_redevance_audiovisuelle",
                "mois hors de l'intervalle 1 à 12",
            ));
        }
    }
    Ok(())
}

impl ParametresSociaux {
    /// Charge les paramètres sociaux embarqués d'une année.
    pub fn charger(annee: i32) -> Result<Self, ErreurPaie> {
        let contenu = fichier_embarque(annee).ok_or(ErreurPaie::AnneeInconnue(annee))?;
        let parametres = Self::depuis_toml(contenu)?;
        if parametres.en_tete.annee != annee {
            return Err(ErreurPaie::AnneeIncoherente {
                demandee: annee,
                fichier: parametres.en_tete.annee,
            });
        }
        Ok(parametres)
    }

    /// Paramètres applicables à une opération datée du `date` (date de paie) : ceux de l'année
    /// civile de cette date.
    pub fn pour_date(date: Date) -> Result<Self, ErreurPaie> {
        Self::charger(date.year())
    }

    /// Lit et valide le contenu d'un `social.toml`.
    pub fn depuis_toml(contenu: &str) -> Result<Self, ErreurPaie> {
        let parametres: ParametresSociaux = lire_toml("social.toml", contenu)?;
        parametres.valider()?;
        Ok(parametres)
    }

    fn valider(&self) -> Result<(), ErreurPaie> {
        valider_bareme(&self.its_bareme_mensuel.valeur)?;
        valider_redevance(&self.its_redevance_audiovisuelle.valeur)?;
        let (min, max) = (
            self.cnss_risques_professionnels_min.valeur,
            self.cnss_risques_professionnels_max.valeur,
        );
        if min > max {
            return Err(incoherent(
                "cnss_risques_professionnels_min",
                "supérieur au taux maximum",
            ));
        }
        if self.smig_mensuel.valeur.francs() <= 0 {
            return Err(incoherent("smig_mensuel", "doit être strictement positif"));
        }
        Ok(())
    }

    /// Tranches du barème mensuel de l'ITS.
    pub fn bareme_its(&self) -> &[Tranche] {
        &self.its_bareme_mensuel.valeur
    }

    /// Base imposable de l'ITS (tant que l'issue #44 n'est pas close : [`BaseIts::NonTranchee`]).
    pub fn base_its(&self) -> BaseIts {
        self.its_base_imposable.valeur
    }

    /// Périodicité CNSS (tant que l'issue #47 n'est pas close : [`Periodicite::NonTranchee`]).
    pub fn periodicite_cnss(&self) -> Periodicite {
        self.cnss_periodicite.valeur
    }

    /// Plafond mensuel de l'assiette CNSS, ou `None` quand le fichier indique « pas de
    /// plafond » (valeur `0`, provisoire : issue #45).
    pub fn plafond_cnss_mensuel(&self) -> Option<Montant> {
        let plafond = self.cnss_plafond_mensuel.valeur;
        (!plafond.est_zero()).then_some(plafond)
    }

    /// Montant de la redevance audiovisuelle retenue pour un mois (1 à 12), s'il y en a une.
    pub fn redevance_audiovisuelle(&self, mois: u32) -> Option<Montant> {
        self.its_redevance_audiovisuelle
            .valeur
            .iter()
            .find(|echeance| echeance.mois == mois)
            .map(|echeance| echeance.montant)
    }

    /// Majorations d'heures supplémentaires. Liste vide tant que l'issue #49 n'est pas close.
    pub fn majorations(&self) -> &[Majoration] {
        &self.heures_supplementaires_majorations.valeur
    }

    /// Source et indicateur `verifie` de chaque paramètre (hors en-tête).
    pub fn sources(&self) -> Vec<SourceValeur<'_>> {
        vec![
            source("its_bareme_mensuel", &self.its_bareme_mensuel),
            source("its_base_imposable", &self.its_base_imposable),
            source("its_redevance_audiovisuelle", &self.its_redevance_audiovisuelle),
            source("vps_taux", &self.vps_taux),
            source("vps_taux_enseignement_prive", &self.vps_taux_enseignement_prive),
            source("vps_exonere_tps", &self.vps_exonere_tps),
            source(
                "cnss_prestations_familiales_employeur",
                &self.cnss_prestations_familiales_employeur,
            ),
            source("cnss_pension_employeur", &self.cnss_pension_employeur),
            source("cnss_pension_salarie", &self.cnss_pension_salarie),
            source(
                "cnss_risques_professionnels_min",
                &self.cnss_risques_professionnels_min,
            ),
            source(
                "cnss_risques_professionnels_max",
                &self.cnss_risques_professionnels_max,
            ),
            source("cnss_plafond_mensuel", &self.cnss_plafond_mensuel),
            source("cnss_periodicite", &self.cnss_periodicite),
            source(
                "cnss_majoration_retard_mensuelle",
                &self.cnss_majoration_retard_mensuelle,
            ),
            source(
                "heures_supplementaires_majorations",
                &self.heures_supplementaires_majorations,
            ),
            source("smig_mensuel", &self.smig_mensuel),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ac_noyau::{Arrondi, ErreurNoyau};
    use chrono::NaiveDate;

    fn m(v: i64) -> Montant {
        Montant::new(v).unwrap()
    }

    fn taux(pourcentage: &str) -> Taux {
        Taux::pourcentage(pourcentage).unwrap()
    }

    fn charger_2026() -> ParametresSociaux {
        ParametresSociaux::charger(2026).unwrap()
    }

    #[test]
    fn chaque_valeur_a_une_source() {
        for s in charger_2026().sources() {
            assert!(!s.source.trim().is_empty(), "« {} » sans source", s.cle);
        }
    }

    #[test]
    fn chaque_cle_du_fichier_est_couverte() {
        let contenu = fichier_embarque(2026).unwrap();
        let table: toml::Table = toml::from_str(contenu).unwrap();
        let parametres = charger_2026();
        let sources = parametres.sources();
        for cle in table.keys().filter(|c| c.as_str() != "en_tete") {
            assert!(
                sources.iter().any(|s| s.cle == cle.as_str()),
                "« {cle} » absent de ParametresSociaux::sources()"
            );
        }
        assert_eq!(sources.len() + 1, table.len());
    }

    #[test]
    fn cle_inconnue_refusee() {
        let contenu = format!(
            "{}\n[cle_inconnue]\nvaleur = 1\nsource = \"test\"\nverifie = false\n",
            fichier_embarque(2026).unwrap()
        );
        assert!(ParametresSociaux::depuis_toml(&contenu).is_err());
    }

    #[test]
    fn annee_selon_la_date_de_l_operation() {
        let fin_2026 = NaiveDate::from_ymd_opt(2026, 12, 31).unwrap();
        assert!(ParametresSociaux::pour_date(fin_2026).is_ok());
        let debut_2027 = NaiveDate::from_ymd_opt(2027, 1, 1).unwrap();
        assert_eq!(
            ParametresSociaux::pour_date(debut_2027).unwrap_err(),
            ErreurPaie::AnneeInconnue(2027)
        );
    }

    #[test]
    fn annee_inconnue() {
        assert_eq!(
            ParametresSociaux::charger(1999).unwrap_err(),
            ErreurPaie::AnneeInconnue(1999)
        );
    }

    #[test]
    fn valeurs_2026_chiffrees() {
        let p = charger_2026();
        assert_eq!(p.smig_mensuel.valeur, m(52_000));
        assert_eq!(p.vps_taux.valeur, taux("4"));
        assert_eq!(p.vps_taux_enseignement_prive.valeur, taux("2"));
        assert!(p.vps_exonere_tps.valeur);
        assert_eq!(p.cnss_prestations_familiales_employeur.valeur, taux("9"));
        assert_eq!(p.cnss_pension_employeur.valeur, taux("6.4"));
        assert_eq!(p.cnss_pension_salarie.valeur, taux("3.6"));
        assert_eq!(p.cnss_risques_professionnels_min.valeur, taux("1"));
        assert_eq!(p.cnss_risques_professionnels_max.valeur, taux("4"));
        assert_eq!(p.cnss_majoration_retard_mensuelle.valeur, taux("1.5"));
    }

    /// Salaire brut 300 000 FCFA : chaque cotisation calculée à la main.
    #[test]
    fn cotisations_sur_300_000() {
        let p = charger_2026();
        let brut = m(300_000);
        let calcul = |t: Taux| brut.appliquer_taux(t, Arrondi::PlusProche).unwrap();
        // 3,6 % de 300 000 = 10 800
        assert_eq!(calcul(p.cnss_pension_salarie.valeur), m(10_800));
        // 9 % de 300 000 = 27 000
        assert_eq!(calcul(p.cnss_prestations_familiales_employeur.valeur), m(27_000));
        // 6,4 % de 300 000 = 19 200
        assert_eq!(calcul(p.cnss_pension_employeur.valeur), m(19_200));
        // 1 % de 300 000 = 3 000 (taux minimum de risques professionnels)
        assert_eq!(calcul(p.cnss_risques_professionnels_min.valeur), m(3_000));
        // 4 % de 300 000 = 12 000 (VPS)
        assert_eq!(calcul(p.vps_taux.valeur), m(12_000));
        // 2 % de 300 000 = 6 000 (VPS, enseignement privé)
        assert_eq!(calcul(p.vps_taux_enseignement_prive.valeur), m(6_000));
    }

    /// Salaire nul : aucune cotisation, et pas de plafond quand le fichier vaut 0.
    #[test]
    fn salaire_nul_et_pas_de_plafond() {
        let p = charger_2026();
        assert_eq!(
            m(0).appliquer_taux(p.cnss_pension_salarie.valeur, Arrondi::PlusProche),
            Ok(m(0))
        );
        assert_eq!(p.plafond_cnss_mensuel(), None);
    }

    #[test]
    fn redevance_par_mois() {
        let p = charger_2026();
        assert_eq!(p.redevance_audiovisuelle(3), Some(m(1_000)));
        assert_eq!(p.redevance_audiovisuelle(6), Some(m(3_000)));
        assert_eq!(p.redevance_audiovisuelle(1), None);
    }

    #[test]
    fn valeurs_non_tranchees_restent_explicites() {
        let p = charger_2026();
        assert_eq!(p.base_its(), BaseIts::NonTranchee);
        assert_eq!(p.periodicite_cnss(), Periodicite::NonTranchee);
        assert!(p.majorations().is_empty());
    }

    #[test]
    fn bareme_bien_forme() {
        let tranche = |haut: Option<i64>| Tranche {
            jusqu_a: haut.map(m),
            taux: Taux::ZERO,
        };
        assert!(valider_bareme(&[tranche(Some(100)), tranche(None)]).is_ok());
        assert!(valider_bareme(&[]).is_err());
        // bornes décroissantes
        assert!(valider_bareme(&[tranche(Some(100)), tranche(Some(50)), tranche(None)]).is_err());
        // dernière tranche bornée
        assert!(valider_bareme(&[tranche(Some(100))]).is_err());
        // tranche sans borne au milieu
        assert!(valider_bareme(&[tranche(None), tranche(Some(100))]).is_err());
    }

    #[test]
    fn redevance_mois_valides() {
        let echeance = |mois: u32| EcheanceRedevance {
            mois,
            montant: m(1_000),
        };
        assert!(valider_redevance(&[echeance(1), echeance(12)]).is_ok());
        assert!(valider_redevance(&[echeance(0)]).is_err());
        assert!(valider_redevance(&[echeance(13)]).is_err());
    }

    #[test]
    fn erreur_noyau_remontee() {
        let r = ParametresSociaux::depuis_toml("n_est_pas_du_toml = [");
        assert!(matches!(
            r,
            Err(ErreurPaie::Noyau(ErreurNoyau::ParametresInvalides { .. }))
        ));
    }
}
