//! Moteur de calcul et de contrôle des états financiers SYSCOHADA révisé (Système normal).
//!
//! Génère le [`Bilan`] et le [`CompteResultat`] avec colonnes N et N-1 à partir d'une
//! [`Balance`], en appliquant les règles de [`Correspondances`].

use std::collections::BTreeMap;

use ac_noyau::{Montant, Periode};
use serde::{Deserialize, Serialize};

use crate::balance::{Balance, LigneBalance};
use crate::correspondances::{ColonneBilan, Correspondances, EtatFinancier, SigneRubrique};
use crate::{ErreurEtats, Resultat};

/// Nature hiérarchique d'une ligne d'état financier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TypeLigne {
    /// Rubrique élémentaire alimentée par les comptes.
    Detail,
    /// Sous-total intermédiaire.
    SousTotal,
    /// Total général de l'état.
    TotalGeneral,
}

/// Ligne du bilan — Actif (exercice N en Brut / Amort-Dépréc / Net, et Net N-1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LigneBilanActif {
    /// Code référence officiel SYSCOHADA (`AD`..`BZ`).
    pub reference: String,
    /// Libellé officiel de la rubrique.
    pub libelle: String,
    /// Nature de la ligne.
    pub type_ligne: TypeLigne,
    /// Montant brut de l'exercice N.
    pub brut_n: Montant,
    /// Amortissements et dépréciations de l'exercice N.
    pub amort_deprec_n: Montant,
    /// Montant net de l'exercice N (`brut_n - amort_deprec_n`).
    pub net_n: Montant,
    /// Montant net de l'exercice N-1.
    pub net_n1: Montant,
}

/// Ligne du bilan — Passif (Net N et Net N-1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LigneBilanPassif {
    /// Code référence officiel SYSCOHADA (`CA`..`DZ`).
    pub reference: String,
    /// Libellé officiel de la rubrique.
    pub libelle: String,
    /// Nature de la ligne.
    pub type_ligne: TypeLigne,
    /// Montant net de l'exercice N.
    pub net_n: Montant,
    /// Montant net de l'exercice N-1.
    pub net_n1: Montant,
}

/// Bilan comptable du Système normal SYSCOHADA révisé (N et N-1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Bilan {
    /// Période de l'exercice N.
    pub periode_n: Option<Periode>,
    /// Période de l'exercice N-1.
    pub periode_n1: Option<Periode>,
    /// Lignes de l'actif dans l'ordre réglementaire (`AD`..`BZ`).
    pub actif: Vec<LigneBilanActif>,
    /// Lignes du passif dans l'ordre réglementaire (`CA`..`DZ`).
    pub passif: Vec<LigneBilanPassif>,
    /// Total brut de l'actif N.
    pub total_actif_brut_n: Montant,
    /// Total des amortissements et dépréciations N.
    pub total_amort_deprec_n: Montant,
    /// Total général de l'actif net N (`BZ`).
    pub total_actif_n: Montant,
    /// Total général du passif N (`DZ`).
    pub total_passif_n: Montant,
    /// Résultat net de l'exercice N figurant au passif (`CJ`).
    pub resultat_net_n: Montant,
    /// Total général de l'actif net N-1 (`BZ`).
    pub total_actif_n1: Montant,
    /// Total général du passif N-1 (`DZ`).
    pub total_passif_n1: Montant,
    /// Résultat net de l'exercice N-1 figurant au passif (`CJ`).
    pub resultat_net_n1: Montant,
    /// Vrai si la balance comportait un comparatif N-1.
    pub possede_comparatif: bool,
}

impl Bilan {
    /// Recherche une rubrique de l'actif par son code référence (`AD`..`BZ`).
    pub fn rubrique_actif(&self, reference: &str) -> Option<&LigneBilanActif> {
        self.actif.iter().find(|l| l.reference == reference)
    }

    /// Recherche une rubrique du passif par son code référence (`CA`..`DZ`).
    pub fn rubrique_passif(&self, reference: &str) -> Option<&LigneBilanPassif> {
        self.passif.iter().find(|l| l.reference == reference)
    }
}

/// Ligne du compte de résultat SYSCOHADA révisé (N et N-1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LigneCompteResultat {
    /// Code référence officiel (`TA`..`XI`).
    pub reference: String,
    /// Libellé officiel de la rubrique ou du solde intermédiaire.
    pub libelle: String,
    /// Nature de la ligne.
    pub type_ligne: TypeLigne,
    /// Signe officiel (`+`, `-`, `+/-`) pour les rubriques de détail.
    pub signe: Option<SigneRubrique>,
    /// Montant de l'exercice N (positif pour une charge sur une ligne `-`).
    pub montant_n: Montant,
    /// Montant de l'exercice N-1.
    pub montant_n1: Montant,
}

impl LigneCompteResultat {
    /// Contribution algébrique de la ligne au résultat net sur l'exercice N.
    pub fn contribution_n(&self) -> Montant {
        match self.signe {
            Some(SigneRubrique::Moins) => -self.montant_n,
            _ => self.montant_n,
        }
    }

    /// Contribution algébrique de la ligne au résultat net sur l'exercice N-1.
    pub fn contribution_n1(&self) -> Montant {
        match self.signe {
            Some(SigneRubrique::Moins) => -self.montant_n1,
            _ => self.montant_n1,
        }
    }
}

/// Compte de résultat en liste du Système normal SYSCOHADA révisé (N et N-1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompteResultat {
    /// Période de l'exercice N.
    pub periode_n: Option<Periode>,
    /// Période de l'exercice N-1.
    pub periode_n1: Option<Periode>,
    /// Lignes du compte de résultat dans l'ordre réglementaire (`TA`..`XI`).
    pub lignes: Vec<LigneCompteResultat>,
    /// Chiffre d'affaires N (`TA + TB + TC + TD`).
    pub chiffre_affaires_n: Montant,
    /// Marge commerciale N (`XA`).
    pub marge_commerciale_n: Montant,
    /// Valeur ajoutée N (`XB`).
    pub valeur_ajoutee_n: Montant,
    /// Excédent brut d'exploitation N (`XC`).
    pub excedent_brut_exploitation_n: Montant,
    /// Résultat d'exploitation N (`XD`).
    pub resultat_exploitation_n: Montant,
    /// Résultat financier N (`XE`).
    pub resultat_financier_n: Montant,
    /// Résultat des activités ordinaires N (`XF`).
    pub resultat_activites_ordinaires_n: Montant,
    /// Résultat hors activités ordinaires N (`XG`).
    pub resultat_hao_n: Montant,
    /// Résultat net N (`XI`).
    pub resultat_net_n: Montant,
    /// Chiffre d'affaires N-1 (`TA + TB + TC + TD`).
    pub chiffre_affaires_n1: Montant,
    /// Marge commerciale N-1 (`XA`).
    pub marge_commerciale_n1: Montant,
    /// Valeur ajoutée N-1 (`XB`).
    pub valeur_ajoutee_n1: Montant,
    /// Excédent brut d'exploitation N-1 (`XC`).
    pub excedent_brut_exploitation_n1: Montant,
    /// Résultat d'exploitation N-1 (`XD`).
    pub resultat_exploitation_n1: Montant,
    /// Résultat financier N-1 (`XE`).
    pub resultat_financier_n1: Montant,
    /// Résultat des activités ordinaires N-1 (`XF`).
    pub resultat_activites_ordinaires_n1: Montant,
    /// Résultat hors activités ordinaires N-1 (`XG`).
    pub resultat_hao_n1: Montant,
    /// Résultat net N-1 (`XI`).
    pub resultat_net_n1: Montant,
    /// Vrai si la balance comportait un comparatif N-1.
    pub possede_comparatif: bool,
}

impl CompteResultat {
    /// Recherche une rubrique ou un solde intermédiaire par son code référence (`TA`..`XI`).
    pub fn rubrique(&self, reference: &str) -> Option<&LigneCompteResultat> {
        self.lignes.iter().find(|l| l.reference == reference)
    }
}

pub(crate) fn rubrique_detail_existe(etat: EtatFinancier, reference: &str) -> bool {
    match etat {
        EtatFinancier::BilanActif => STRUCTURE_ACTIF
            .iter()
            .any(|s| s.reference == reference && s.type_ligne == TypeLigne::Detail),
        EtatFinancier::BilanPassif => STRUCTURE_PASSIF
            .iter()
            .any(|s| s.reference == reference && s.type_ligne == TypeLigne::Detail),
        EtatFinancier::CompteResultat => STRUCTURE_RESULTAT
            .iter()
            .any(|s| s.reference == reference && s.type_ligne == TypeLigne::Detail),
    }
}

/// Calcule le bilan SYSCOHADA révisé (Système normal) à partir d'une balance.
pub fn bilan(balance: &Balance) -> Resultat<Bilan> {
    let correspondances = Correspondances::syscohada()?;
    bilan_avec_correspondances(balance, &correspondances)
}

/// Calcule le bilan avec une table de correspondances explicite.
pub fn bilan_avec_correspondances(
    balance: &Balance,
    correspondances: &Correspondances,
) -> Resultat<Bilan> {
    balance.valider()?;
    let cumuls_n = cumuler_exercice(&balance.lignes_n, correspondances)?;
    let cumuls_n1 = cumuler_exercice(&balance.lignes_n1, correspondances)?;

    let actif = construire_actif(&cumuls_n, &cumuls_n1)?;
    let passif = construire_passif(&cumuls_n, &cumuls_n1)?;

    let (total_actif_brut_n, total_amort_deprec_n, total_actif_n, total_actif_n1) =
        match actif.iter().find(|l| l.reference == "BZ") {
            Some(l) => (l.brut_n, l.amort_deprec_n, l.net_n, l.net_n1),
            None => (Montant::ZERO, Montant::ZERO, Montant::ZERO, Montant::ZERO),
        };

    let (total_passif_n, total_passif_n1) = match passif.iter().find(|l| l.reference == "DZ") {
        Some(l) => (l.net_n, l.net_n1),
        None => (Montant::ZERO, Montant::ZERO),
    };

    let (resultat_net_n, resultat_net_n1) = match passif.iter().find(|l| l.reference == "CJ") {
        Some(l) => (l.net_n, l.net_n1),
        None => (Montant::ZERO, Montant::ZERO),
    };

    if total_actif_n != total_passif_n {
        return Err(ErreurEtats::BilanDesequilibre {
            exercice: "N",
            actif: total_actif_n,
            passif: total_passif_n,
            ecart: total_actif_n.soustraire(total_passif_n)?.abs(),
        });
    }

    let possede_comparatif = balance.possede_comparatif();
    if possede_comparatif && total_actif_n1 != total_passif_n1 {
        return Err(ErreurEtats::BilanDesequilibre {
            exercice: "N-1",
            actif: total_actif_n1,
            passif: total_passif_n1,
            ecart: total_actif_n1.soustraire(total_passif_n1)?.abs(),
        });
    }

    Ok(Bilan {
        periode_n: balance.periode_n,
        periode_n1: balance.periode_n1,
        actif,
        passif,
        total_actif_brut_n,
        total_amort_deprec_n,
        total_actif_n,
        total_passif_n,
        resultat_net_n,
        total_actif_n1,
        total_passif_n1,
        resultat_net_n1,
        possede_comparatif,
    })
}

/// Calcule le compte de résultat SYSCOHADA révisé (Système normal) à partir d'une balance.
pub fn compte_resultat(balance: &Balance) -> Resultat<CompteResultat> {
    let correspondances = Correspondances::syscohada()?;
    compte_resultat_avec_correspondances(balance, &correspondances)
}

/// Calcule le compte de résultat avec une table de correspondances explicite.
pub fn compte_resultat_avec_correspondances(
    balance: &Balance,
    correspondances: &Correspondances,
) -> Resultat<CompteResultat> {
    balance.valider()?;
    let cumuls_n = cumuler_exercice(&balance.lignes_n, correspondances)?;
    let cumuls_n1 = cumuler_exercice(&balance.lignes_n1, correspondances)?;

    let synthese_n = calculer_soldes_resultat(&cumuls_n.resultat)?;
    let synthese_n1 = calculer_soldes_resultat(&cumuls_n1.resultat)?;
    let lignes = construire_lignes_resultat(
        &cumuls_n.resultat,
        &cumuls_n1.resultat,
        &synthese_n,
        &synthese_n1,
    );

    Ok(CompteResultat {
        periode_n: balance.periode_n,
        periode_n1: balance.periode_n1,
        lignes,
        chiffre_affaires_n: synthese_n.ca,
        marge_commerciale_n: synthese_n.xa,
        valeur_ajoutee_n: synthese_n.xb,
        excedent_brut_exploitation_n: synthese_n.xc,
        resultat_exploitation_n: synthese_n.xd,
        resultat_financier_n: synthese_n.xe,
        resultat_activites_ordinaires_n: synthese_n.xf,
        resultat_hao_n: synthese_n.xg,
        resultat_net_n: synthese_n.xi,
        chiffre_affaires_n1: synthese_n1.ca,
        marge_commerciale_n1: synthese_n1.xa,
        valeur_ajoutee_n1: synthese_n1.xb,
        excedent_brut_exploitation_n1: synthese_n1.xc,
        resultat_exploitation_n1: synthese_n1.xd,
        resultat_financier_n1: synthese_n1.xe,
        resultat_activites_ordinaires_n1: synthese_n1.xf,
        resultat_hao_n1: synthese_n1.xg,
        resultat_net_n1: synthese_n1.xi,
        possede_comparatif: balance.possede_comparatif(),
    })
}

/// Contrôle la cohérence entre le bilan et le compte de résultat (équilibre et résultat net).
pub fn verifier_coherence(bilan: &Bilan, resultat: &CompteResultat) -> Resultat<()> {
    if bilan.total_actif_n != bilan.total_passif_n {
        return Err(ErreurEtats::BilanDesequilibre {
            exercice: "N",
            actif: bilan.total_actif_n,
            passif: bilan.total_passif_n,
            ecart: bilan.total_actif_n.soustraire(bilan.total_passif_n)?.abs(),
        });
    }
    if bilan.resultat_net_n != resultat.resultat_net_n {
        return Err(ErreurEtats::ResultatsIncoherents {
            exercice: "N",
            bilan: bilan.resultat_net_n,
            compte_resultat: resultat.resultat_net_n,
            ecart: bilan.resultat_net_n.soustraire(resultat.resultat_net_n)?.abs(),
        });
    }
    if bilan.possede_comparatif || resultat.possede_comparatif {
        if bilan.total_actif_n1 != bilan.total_passif_n1 {
            return Err(ErreurEtats::BilanDesequilibre {
                exercice: "N-1",
                actif: bilan.total_actif_n1,
                passif: bilan.total_passif_n1,
                ecart: bilan.total_actif_n1.soustraire(bilan.total_passif_n1)?.abs(),
            });
        }
        if bilan.resultat_net_n1 != resultat.resultat_net_n1 {
            return Err(ErreurEtats::ResultatsIncoherents {
                exercice: "N-1",
                bilan: bilan.resultat_net_n1,
                compte_resultat: resultat.resultat_net_n1,
                ecart: bilan.resultat_net_n1.soustraire(resultat.resultat_net_n1)?.abs(),
            });
        }
    }
    Ok(())
}

/// Produit le bilan et le compte de résultat puis vérifie leur cohérence d'ensemble.
pub fn generer_etats_complets(balance: &Balance) -> Resultat<(Bilan, CompteResultat)> {
    let b = bilan(balance)?;
    let cr = compte_resultat(balance)?;
    verifier_coherence(&b, &cr)?;
    Ok((b, cr))
}

#[derive(Default)]
struct CumulExercice {
    brut: BTreeMap<String, Montant>,
    amort_deprec: BTreeMap<String, Montant>,
    passif: BTreeMap<String, Montant>,
    resultat: BTreeMap<String, Montant>,
    resultat_gestion: Montant,
}

fn cumuler_exercice(
    lignes: &[LigneBalance],
    correspondances: &Correspondances,
) -> Resultat<CumulExercice> {
    let mut cumul = CumulExercice::default();
    for ligne in lignes {
        let regle = correspondances.resoudre(&ligne.compte, ligne.sens_solde())?;
        match (regle.etat, regle.colonne) {
            (EtatFinancier::BilanActif, ColonneBilan::Brut) => {
                let montant = ligne.solde_algebrique_debiteur()?;
                accumuler(&mut cumul.brut, &regle.rubrique, montant)?;
            }
            (EtatFinancier::BilanActif, ColonneBilan::AmortDeprec) => {
                let montant = ligne.solde_algebrique_crediteur()?;
                accumuler(&mut cumul.amort_deprec, &regle.rubrique, montant)?;
            }
            (EtatFinancier::BilanPassif, ColonneBilan::Passif) => {
                let montant = ligne.solde_algebrique_crediteur()?;
                accumuler(&mut cumul.passif, &regle.rubrique, montant)?;
            }
            (EtatFinancier::CompteResultat, ColonneBilan::Resultat) => {
                let impact = ligne.solde_algebrique_crediteur()?;
                accumuler(&mut cumul.resultat, &regle.rubrique, impact)?;
                cumul.resultat_gestion = cumul.resultat_gestion.ajouter(impact)?;
            }
            _ => {}
        }
    }
    Ok(cumul)
}

fn accumuler(map: &mut BTreeMap<String, Montant>, cle: &str, delta: Montant) -> Resultat<()> {
    let courant = map.get(cle).copied().unwrap_or(Montant::ZERO);
    map.insert(cle.to_owned(), courant.ajouter(delta)?);
    Ok(())
}

fn lire(map: &BTreeMap<String, Montant>, cle: &str) -> Montant {
    map.get(cle).copied().unwrap_or(Montant::ZERO)
}

struct ActifSpec {
    reference: &'static str,
    libelle: &'static str,
    enfants: &'static [&'static str],
    type_ligne: TypeLigne,
}

const STRUCTURE_ACTIF: &[ActifSpec] = &[
    ActifSpec {
        reference: "AD",
        libelle: "Immobilisations incorporelles",
        enfants: &["AE", "AF", "AG", "AH"],
        type_ligne: TypeLigne::SousTotal,
    },
    ActifSpec {
        reference: "AE",
        libelle: "Frais de développement et de prospection",
        enfants: &[],
        type_ligne: TypeLigne::Detail,
    },
    ActifSpec {
        reference: "AF",
        libelle: "Brevets, licences, logiciels et droits similaires",
        enfants: &[],
        type_ligne: TypeLigne::Detail,
    },
    ActifSpec {
        reference: "AG",
        libelle: "Fonds commercial et droit au bail",
        enfants: &[],
        type_ligne: TypeLigne::Detail,
    },
    ActifSpec {
        reference: "AH",
        libelle: "Autres immobilisations incorporelles",
        enfants: &[],
        type_ligne: TypeLigne::Detail,
    },
    ActifSpec {
        reference: "AI",
        libelle: "Immobilisations corporelles",
        enfants: &["AJ", "AK", "AL", "AM", "AN"],
        type_ligne: TypeLigne::SousTotal,
    },
    ActifSpec {
        reference: "AJ",
        libelle: "Terrains",
        enfants: &[],
        type_ligne: TypeLigne::Detail,
    },
    ActifSpec {
        reference: "AK",
        libelle: "Bâtiments",
        enfants: &[],
        type_ligne: TypeLigne::Detail,
    },
    ActifSpec {
        reference: "AL",
        libelle: "Aménagements, agencements et installations",
        enfants: &[],
        type_ligne: TypeLigne::Detail,
    },
    ActifSpec {
        reference: "AM",
        libelle: "Matériel, mobilier et actifs biologiques",
        enfants: &[],
        type_ligne: TypeLigne::Detail,
    },
    ActifSpec {
        reference: "AN",
        libelle: "Matériel de transport",
        enfants: &[],
        type_ligne: TypeLigne::Detail,
    },
    ActifSpec {
        reference: "AP",
        libelle: "Avances et acomptes versés sur immobilisations",
        enfants: &[],
        type_ligne: TypeLigne::Detail,
    },
    ActifSpec {
        reference: "AQ",
        libelle: "Immobilisations financières",
        enfants: &["AR", "AS"],
        type_ligne: TypeLigne::SousTotal,
    },
    ActifSpec {
        reference: "AR",
        libelle: "Titres de participation",
        enfants: &[],
        type_ligne: TypeLigne::Detail,
    },
    ActifSpec {
        reference: "AS",
        libelle: "Autres immobilisations financières",
        enfants: &[],
        type_ligne: TypeLigne::Detail,
    },
    ActifSpec {
        reference: "AZ",
        libelle: "TOTAL ACTIF IMMOBILISÉ",
        enfants: &["AD", "AI", "AP", "AQ"],
        type_ligne: TypeLigne::SousTotal,
    },
    ActifSpec {
        reference: "BA",
        libelle: "Actif circulant HAO",
        enfants: &[],
        type_ligne: TypeLigne::Detail,
    },
    ActifSpec {
        reference: "BB",
        libelle: "Stocks et encours",
        enfants: &[],
        type_ligne: TypeLigne::Detail,
    },
    ActifSpec {
        reference: "BG",
        libelle: "Créances et emplois assimilés",
        enfants: &["BH", "BI", "BJ"],
        type_ligne: TypeLigne::SousTotal,
    },
    ActifSpec {
        reference: "BH",
        libelle: "Fournisseurs avances versées",
        enfants: &[],
        type_ligne: TypeLigne::Detail,
    },
    ActifSpec {
        reference: "BI",
        libelle: "Clients",
        enfants: &[],
        type_ligne: TypeLigne::Detail,
    },
    ActifSpec {
        reference: "BJ",
        libelle: "Autres créances",
        enfants: &[],
        type_ligne: TypeLigne::Detail,
    },
    ActifSpec {
        reference: "BK",
        libelle: "TOTAL ACTIF CIRCULANT",
        enfants: &["BA", "BB", "BG"],
        type_ligne: TypeLigne::SousTotal,
    },
    ActifSpec {
        reference: "BQ",
        libelle: "Titres de placement",
        enfants: &[],
        type_ligne: TypeLigne::Detail,
    },
    ActifSpec {
        reference: "BR",
        libelle: "Valeurs à encaisser",
        enfants: &[],
        type_ligne: TypeLigne::Detail,
    },
    ActifSpec {
        reference: "BS",
        libelle: "Banques, chèques postaux, caisse et assimilés",
        enfants: &[],
        type_ligne: TypeLigne::Detail,
    },
    ActifSpec {
        reference: "BT",
        libelle: "TOTAL TRÉSORERIE-ACTIF",
        enfants: &["BQ", "BR", "BS"],
        type_ligne: TypeLigne::SousTotal,
    },
    ActifSpec {
        reference: "BU",
        libelle: "Écart de conversion - Actif",
        enfants: &[],
        type_ligne: TypeLigne::Detail,
    },
    ActifSpec {
        reference: "BZ",
        libelle: "TOTAL GÉNÉRAL",
        enfants: &["AZ", "BK", "BT", "BU"],
        type_ligne: TypeLigne::TotalGeneral,
    },
];

fn construire_actif(n: &CumulExercice, n1: &CumulExercice) -> Resultat<Vec<LigneBilanActif>> {
    let mut valeurs: BTreeMap<&'static str, (Montant, Montant, Montant, Montant)> = BTreeMap::new();

    for spec in STRUCTURE_ACTIF {
        if spec.type_ligne == TypeLigne::Detail {
            let brut_n = lire(&n.brut, spec.reference);
            let amort_n = lire(&n.amort_deprec, spec.reference);
            let net_n = brut_n.soustraire(amort_n)?;
            let net_n1 = lire(&n1.brut, spec.reference)
                .soustraire(lire(&n1.amort_deprec, spec.reference))?;
            valeurs.insert(spec.reference, (brut_n, amort_n, net_n, net_n1));
        }
    }

    for spec in STRUCTURE_ACTIF {
        if spec.type_ligne != TypeLigne::Detail {
            let mut b_n = Montant::ZERO;
            let mut a_n = Montant::ZERO;
            let mut net_n = Montant::ZERO;
            let mut net_n1 = Montant::ZERO;
            for enfant in spec.enfants {
                if let Some(&(eb, ea, en, en1)) = valeurs.get(enfant) {
                    b_n = b_n.ajouter(eb)?;
                    a_n = a_n.ajouter(ea)?;
                    net_n = net_n.ajouter(en)?;
                    net_n1 = net_n1.ajouter(en1)?;
                }
            }
            valeurs.insert(spec.reference, (b_n, a_n, net_n, net_n1));
        }
    }

    let mut lignes = Vec::with_capacity(STRUCTURE_ACTIF.len());
    for spec in STRUCTURE_ACTIF {
        let (brut_n, amort_deprec_n, net_n, net_n1) = valeurs
            .get(spec.reference)
            .copied()
            .unwrap_or((Montant::ZERO, Montant::ZERO, Montant::ZERO, Montant::ZERO));
        lignes.push(LigneBilanActif {
            reference: spec.reference.to_owned(),
            libelle: spec.libelle.to_owned(),
            type_ligne: spec.type_ligne,
            brut_n,
            amort_deprec_n,
            net_n,
            net_n1,
        });
    }
    Ok(lignes)
}

struct PassifSpec {
    reference: &'static str,
    libelle: &'static str,
    enfants: &'static [&'static str],
    type_ligne: TypeLigne,
}

const STRUCTURE_PASSIF: &[PassifSpec] = &[
    PassifSpec {
        reference: "CA",
        libelle: "Capital",
        enfants: &[],
        type_ligne: TypeLigne::Detail,
    },
    PassifSpec {
        reference: "CB",
        libelle: "Apporteurs, capital non appelé (-)",
        enfants: &[],
        type_ligne: TypeLigne::Detail,
    },
    PassifSpec {
        reference: "CD",
        libelle: "Primes liées au capital social",
        enfants: &[],
        type_ligne: TypeLigne::Detail,
    },
    PassifSpec {
        reference: "CE",
        libelle: "Écarts de réévaluation",
        enfants: &[],
        type_ligne: TypeLigne::Detail,
    },
    PassifSpec {
        reference: "CF",
        libelle: "Réserves indisponibles",
        enfants: &[],
        type_ligne: TypeLigne::Detail,
    },
    PassifSpec {
        reference: "CG",
        libelle: "Réserves libres",
        enfants: &[],
        type_ligne: TypeLigne::Detail,
    },
    PassifSpec {
        reference: "CH",
        libelle: "Report à nouveau (+ ou -)",
        enfants: &[],
        type_ligne: TypeLigne::Detail,
    },
    PassifSpec {
        reference: "CJ",
        libelle: "Résultat net de l'exercice (bénéfice + ou perte -)",
        enfants: &[],
        type_ligne: TypeLigne::Detail,
    },
    PassifSpec {
        reference: "CL",
        libelle: "Subventions d'investissement",
        enfants: &[],
        type_ligne: TypeLigne::Detail,
    },
    PassifSpec {
        reference: "CM",
        libelle: "Provisions réglementées",
        enfants: &[],
        type_ligne: TypeLigne::Detail,
    },
    PassifSpec {
        reference: "CP",
        libelle: "TOTAL CAPITAUX PROPRES ET RESSOURCES ASSIMILÉES",
        enfants: &["CA", "CB", "CD", "CE", "CF", "CG", "CH", "CJ", "CL", "CM"],
        type_ligne: TypeLigne::SousTotal,
    },
    PassifSpec {
        reference: "DA",
        libelle: "Emprunts et dettes financières diverses",
        enfants: &[],
        type_ligne: TypeLigne::Detail,
    },
    PassifSpec {
        reference: "DB",
        libelle: "Dettes de location-acquisition",
        enfants: &[],
        type_ligne: TypeLigne::Detail,
    },
    PassifSpec {
        reference: "DC",
        libelle: "Provisions pour risques et charges",
        enfants: &[],
        type_ligne: TypeLigne::Detail,
    },
    PassifSpec {
        reference: "DD",
        libelle: "TOTAL DETTES FINANCIÈRES ET RESSOURCES ASSIMILÉES",
        enfants: &["DA", "DB", "DC"],
        type_ligne: TypeLigne::SousTotal,
    },
    PassifSpec {
        reference: "DF",
        libelle: "TOTAL RESSOURCES STABLES",
        enfants: &["CP", "DD"],
        type_ligne: TypeLigne::SousTotal,
    },
    PassifSpec {
        reference: "DH",
        libelle: "Dettes circulantes HAO",
        enfants: &[],
        type_ligne: TypeLigne::Detail,
    },
    PassifSpec {
        reference: "DI",
        libelle: "Clients, avances reçues",
        enfants: &[],
        type_ligne: TypeLigne::Detail,
    },
    PassifSpec {
        reference: "DJ",
        libelle: "Fournisseurs d'exploitation",
        enfants: &[],
        type_ligne: TypeLigne::Detail,
    },
    PassifSpec {
        reference: "DK",
        libelle: "Dettes fiscales et sociales",
        enfants: &[],
        type_ligne: TypeLigne::Detail,
    },
    PassifSpec {
        reference: "DM",
        libelle: "Autres dettes",
        enfants: &[],
        type_ligne: TypeLigne::Detail,
    },
    PassifSpec {
        reference: "DN",
        libelle: "Provisions pour risques à court terme",
        enfants: &[],
        type_ligne: TypeLigne::Detail,
    },
    PassifSpec {
        reference: "DP",
        libelle: "TOTAL PASSIF CIRCULANT",
        enfants: &["DH", "DI", "DJ", "DK", "DM", "DN"],
        type_ligne: TypeLigne::SousTotal,
    },
    PassifSpec {
        reference: "DQ",
        libelle: "Banques, crédits d'escompte",
        enfants: &[],
        type_ligne: TypeLigne::Detail,
    },
    PassifSpec {
        reference: "DR",
        libelle: "Banques, établissements financiers et crédits de trésorerie",
        enfants: &[],
        type_ligne: TypeLigne::Detail,
    },
    PassifSpec {
        reference: "DT",
        libelle: "TOTAL TRÉSORERIE-PASSIF",
        enfants: &["DQ", "DR"],
        type_ligne: TypeLigne::SousTotal,
    },
    PassifSpec {
        reference: "DV",
        libelle: "Écart de conversion - Passif",
        enfants: &[],
        type_ligne: TypeLigne::Detail,
    },
    PassifSpec {
        reference: "DZ",
        libelle: "TOTAL GÉNÉRAL",
        enfants: &["DF", "DP", "DT", "DV"],
        type_ligne: TypeLigne::TotalGeneral,
    },
];

fn construire_passif(n: &CumulExercice, n1: &CumulExercice) -> Resultat<Vec<LigneBilanPassif>> {
    let mut valeurs: BTreeMap<&'static str, (Montant, Montant)> = BTreeMap::new();

    for spec in STRUCTURE_PASSIF {
        if spec.type_ligne == TypeLigne::Detail {
            let mut val_n = lire(&n.passif, spec.reference);
            let mut val_n1 = lire(&n1.passif, spec.reference);
            if spec.reference == "CJ" {
                val_n = val_n.ajouter(n.resultat_gestion)?;
                val_n1 = val_n1.ajouter(n1.resultat_gestion)?;
            }
            valeurs.insert(spec.reference, (val_n, val_n1));
        }
    }

    for spec in STRUCTURE_PASSIF {
        if spec.type_ligne != TypeLigne::Detail {
            let mut sum_n = Montant::ZERO;
            let mut sum_n1 = Montant::ZERO;
            for enfant in spec.enfants {
                if let Some(&(en, en1)) = valeurs.get(enfant) {
                    sum_n = sum_n.ajouter(en)?;
                    sum_n1 = sum_n1.ajouter(en1)?;
                }
            }
            valeurs.insert(spec.reference, (sum_n, sum_n1));
        }
    }

    let mut lignes = Vec::with_capacity(STRUCTURE_PASSIF.len());
    for spec in STRUCTURE_PASSIF {
        let (net_n, net_n1) = valeurs
            .get(spec.reference)
            .copied()
            .unwrap_or((Montant::ZERO, Montant::ZERO));
        lignes.push(LigneBilanPassif {
            reference: spec.reference.to_owned(),
            libelle: spec.libelle.to_owned(),
            type_ligne: spec.type_ligne,
            net_n,
            net_n1,
        });
    }
    Ok(lignes)
}

struct SoldesResultat {
    ca: Montant,
    xa: Montant,
    xb: Montant,
    xc: Montant,
    xd: Montant,
    xe: Montant,
    xf: Montant,
    xg: Montant,
    xi: Montant,
}

fn calculer_soldes_resultat(impacts: &BTreeMap<String, Montant>) -> Resultat<SoldesResultat> {
    let ca = somme_refs(impacts, &["TA", "TB", "TC", "TD"])?;
    let xa = somme_refs(impacts, &["TA", "RA", "RB"])?;
    let reste_xb = somme_refs(
        impacts,
        &[
            "TB", "TC", "TD", "TE", "TF", "TG", "TH", "TI", "RC", "RD", "RE", "RF", "RG", "RH",
            "RI", "RJ",
        ],
    )?;
    let xb = xa.ajouter(reste_xb)?;
    let xc = xb.ajouter(lire(impacts, "RK"))?;
    let xd = xc.ajouter(somme_refs(impacts, &["TJ", "RL"])?)?;
    let xe = somme_refs(impacts, &["TK", "TL", "TM", "RM", "RN"])?;
    let xf = xd.ajouter(xe)?;
    let xg = somme_refs(impacts, &["TN", "TO", "RO", "RP"])?;
    let xi = xf.ajouter(xg)?.ajouter(somme_refs(impacts, &["RQ", "RS"])?)?;

    Ok(SoldesResultat {
        ca,
        xa,
        xb,
        xc,
        xd,
        xe,
        xf,
        xg,
        xi,
    })
}

fn somme_refs(impacts: &BTreeMap<String, Montant>, refs: &[&str]) -> Resultat<Montant> {
    let mut acc = Montant::ZERO;
    for r in refs {
        acc = acc.ajouter(lire(impacts, r))?;
    }
    Ok(acc)
}

struct ResultatSpec {
    reference: &'static str,
    libelle: &'static str,
    type_ligne: TypeLigne,
    signe: Option<SigneRubrique>,
}

const STRUCTURE_RESULTAT: &[ResultatSpec] = &[
    ResultatSpec {
        reference: "TA",
        libelle: "Ventes de marchandises",
        type_ligne: TypeLigne::Detail,
        signe: Some(SigneRubrique::Plus),
    },
    ResultatSpec {
        reference: "RA",
        libelle: "Achats de marchandises",
        type_ligne: TypeLigne::Detail,
        signe: Some(SigneRubrique::Moins),
    },
    ResultatSpec {
        reference: "RB",
        libelle: "Variation de stocks de marchandises",
        type_ligne: TypeLigne::Detail,
        signe: Some(SigneRubrique::PlusOuMoins),
    },
    ResultatSpec {
        reference: "XA",
        libelle: "MARGE COMMERCIALE (somme TA à RB)",
        type_ligne: TypeLigne::SousTotal,
        signe: None,
    },
    ResultatSpec {
        reference: "TB",
        libelle: "Ventes de produits fabriqués",
        type_ligne: TypeLigne::Detail,
        signe: Some(SigneRubrique::Plus),
    },
    ResultatSpec {
        reference: "TC",
        libelle: "Travaux, services vendus",
        type_ligne: TypeLigne::Detail,
        signe: Some(SigneRubrique::Plus),
    },
    ResultatSpec {
        reference: "TD",
        libelle: "Produits accessoires",
        type_ligne: TypeLigne::Detail,
        signe: Some(SigneRubrique::Plus),
    },
    ResultatSpec {
        reference: "TE",
        libelle: "Production stockée (ou déstockage)",
        type_ligne: TypeLigne::Detail,
        signe: Some(SigneRubrique::PlusOuMoins),
    },
    ResultatSpec {
        reference: "TF",
        libelle: "Production immobilisée",
        type_ligne: TypeLigne::Detail,
        signe: Some(SigneRubrique::Plus),
    },
    ResultatSpec {
        reference: "TG",
        libelle: "Subventions d'exploitation",
        type_ligne: TypeLigne::Detail,
        signe: Some(SigneRubrique::Plus),
    },
    ResultatSpec {
        reference: "TH",
        libelle: "Autres produits",
        type_ligne: TypeLigne::Detail,
        signe: Some(SigneRubrique::Plus),
    },
    ResultatSpec {
        reference: "TI",
        libelle: "Transferts de charges d'exploitation",
        type_ligne: TypeLigne::Detail,
        signe: Some(SigneRubrique::Plus),
    },
    ResultatSpec {
        reference: "RC",
        libelle: "Achats de matières premières et fournitures liées",
        type_ligne: TypeLigne::Detail,
        signe: Some(SigneRubrique::Moins),
    },
    ResultatSpec {
        reference: "RD",
        libelle: "Variation de stocks de matières premières et fournitures liées",
        type_ligne: TypeLigne::Detail,
        signe: Some(SigneRubrique::PlusOuMoins),
    },
    ResultatSpec {
        reference: "RE",
        libelle: "Autres achats",
        type_ligne: TypeLigne::Detail,
        signe: Some(SigneRubrique::Moins),
    },
    ResultatSpec {
        reference: "RF",
        libelle: "Variation de stocks d'autres approvisionnements",
        type_ligne: TypeLigne::Detail,
        signe: Some(SigneRubrique::PlusOuMoins),
    },
    ResultatSpec {
        reference: "RG",
        libelle: "Transports",
        type_ligne: TypeLigne::Detail,
        signe: Some(SigneRubrique::Moins),
    },
    ResultatSpec {
        reference: "RH",
        libelle: "Services extérieurs",
        type_ligne: TypeLigne::Detail,
        signe: Some(SigneRubrique::Moins),
    },
    ResultatSpec {
        reference: "RI",
        libelle: "Impôts et taxes",
        type_ligne: TypeLigne::Detail,
        signe: Some(SigneRubrique::Moins),
    },
    ResultatSpec {
        reference: "RJ",
        libelle: "Autres charges",
        type_ligne: TypeLigne::Detail,
        signe: Some(SigneRubrique::Moins),
    },
    ResultatSpec {
        reference: "XB",
        libelle: "VALEUR AJOUTÉE",
        type_ligne: TypeLigne::SousTotal,
        signe: None,
    },
    ResultatSpec {
        reference: "RK",
        libelle: "Charges de personnel",
        type_ligne: TypeLigne::Detail,
        signe: Some(SigneRubrique::Moins),
    },
    ResultatSpec {
        reference: "XC",
        libelle: "EXCÉDENT BRUT D'EXPLOITATION (EBE)",
        type_ligne: TypeLigne::SousTotal,
        signe: None,
    },
    ResultatSpec {
        reference: "TJ",
        libelle: "Reprises d'amortissements, provisions et dépréciations",
        type_ligne: TypeLigne::Detail,
        signe: Some(SigneRubrique::Plus),
    },
    ResultatSpec {
        reference: "RL",
        libelle: "Dotations aux amortissements, aux provisions et dépréciations",
        type_ligne: TypeLigne::Detail,
        signe: Some(SigneRubrique::Moins),
    },
    ResultatSpec {
        reference: "XD",
        libelle: "RÉSULTAT D'EXPLOITATION",
        type_ligne: TypeLigne::SousTotal,
        signe: None,
    },
    ResultatSpec {
        reference: "TK",
        libelle: "Revenus financiers et assimilés",
        type_ligne: TypeLigne::Detail,
        signe: Some(SigneRubrique::Plus),
    },
    ResultatSpec {
        reference: "TL",
        libelle: "Reprises de provisions et dépréciations financières",
        type_ligne: TypeLigne::Detail,
        signe: Some(SigneRubrique::Plus),
    },
    ResultatSpec {
        reference: "TM",
        libelle: "Transferts de charges financières",
        type_ligne: TypeLigne::Detail,
        signe: Some(SigneRubrique::Plus),
    },
    ResultatSpec {
        reference: "RM",
        libelle: "Frais financiers et charges assimilées",
        type_ligne: TypeLigne::Detail,
        signe: Some(SigneRubrique::Moins),
    },
    ResultatSpec {
        reference: "RN",
        libelle: "Dotations aux provisions et aux dépréciations financières",
        type_ligne: TypeLigne::Detail,
        signe: Some(SigneRubrique::Moins),
    },
    ResultatSpec {
        reference: "XE",
        libelle: "RÉSULTAT FINANCIER",
        type_ligne: TypeLigne::SousTotal,
        signe: None,
    },
    ResultatSpec {
        reference: "XF",
        libelle: "RÉSULTAT DES ACTIVITÉS ORDINAIRES",
        type_ligne: TypeLigne::SousTotal,
        signe: None,
    },
    ResultatSpec {
        reference: "TN",
        libelle: "Produits des cessions d'immobilisations",
        type_ligne: TypeLigne::Detail,
        signe: Some(SigneRubrique::Plus),
    },
    ResultatSpec {
        reference: "TO",
        libelle: "Autres produits HAO",
        type_ligne: TypeLigne::Detail,
        signe: Some(SigneRubrique::Plus),
    },
    ResultatSpec {
        reference: "RO",
        libelle: "Valeurs comptables des cessions d'immobilisations",
        type_ligne: TypeLigne::Detail,
        signe: Some(SigneRubrique::Moins),
    },
    ResultatSpec {
        reference: "RP",
        libelle: "Autres charges HAO",
        type_ligne: TypeLigne::Detail,
        signe: Some(SigneRubrique::Moins),
    },
    ResultatSpec {
        reference: "XG",
        libelle: "RÉSULTAT HORS ACTIVITÉS ORDINAIRES",
        type_ligne: TypeLigne::SousTotal,
        signe: None,
    },
    ResultatSpec {
        reference: "RQ",
        libelle: "Participation des travailleurs",
        type_ligne: TypeLigne::Detail,
        signe: Some(SigneRubrique::Moins),
    },
    ResultatSpec {
        reference: "RS",
        libelle: "Impôts sur le résultat",
        type_ligne: TypeLigne::Detail,
        signe: Some(SigneRubrique::Moins),
    },
    ResultatSpec {
        reference: "XI",
        libelle: "RÉSULTAT NET",
        type_ligne: TypeLigne::TotalGeneral,
        signe: None,
    },
];

fn construire_lignes_resultat(
    impacts_n: &BTreeMap<String, Montant>,
    impacts_n1: &BTreeMap<String, Montant>,
    synthese_n: &SoldesResultat,
    synthese_n1: &SoldesResultat,
) -> Vec<LigneCompteResultat> {
    let mut lignes = Vec::with_capacity(STRUCTURE_RESULTAT.len());
    for spec in STRUCTURE_RESULTAT {
        let (montant_n, montant_n1) = match spec.reference {
            "XA" => (synthese_n.xa, synthese_n1.xa),
            "XB" => (synthese_n.xb, synthese_n1.xb),
            "XC" => (synthese_n.xc, synthese_n1.xc),
            "XD" => (synthese_n.xd, synthese_n1.xd),
            "XE" => (synthese_n.xe, synthese_n1.xe),
            "XF" => (synthese_n.xf, synthese_n1.xf),
            "XG" => (synthese_n.xg, synthese_n1.xg),
            "XI" => (synthese_n.xi, synthese_n1.xi),
            code => {
                let raw_n = lire(impacts_n, code);
                let raw_n1 = lire(impacts_n1, code);
                match spec.signe {
                    Some(SigneRubrique::Moins) => (-raw_n, -raw_n1),
                    _ => (raw_n, raw_n1),
                }
            }
        };
        lignes.push(LigneCompteResultat {
            reference: spec.reference.to_owned(),
            libelle: spec.libelle.to_owned(),
            type_ligne: spec.type_ligne,
            signe: spec.signe,
            montant_n,
            montant_n1,
        });
    }
    lignes
}

#[cfg(test)]
mod tests {
    use ac_compta::NumeroCompte;

    use super::*;

    fn c(n: &str) -> NumeroCompte {
        NumeroCompte::new(n).unwrap()
    }

    fn m(v: i64) -> Montant {
        Montant::new(v).unwrap()
    }

    fn d(compte: &str, libelle: &str, montant: i64) -> LigneBalance {
        LigneBalance::solde_debiteur(c(compte), libelle, m(montant)).unwrap()
    }

    fn cr_ligne(compte: &str, libelle: &str, montant: i64) -> LigneBalance {
        LigneBalance::solde_crediteur(c(compte), libelle, m(montant)).unwrap()
    }

    #[test]
    fn exemple_chiffre_prompt_a9() {
        // Capital 1 000 000 (101 C), banque 1 300 000 (521 D), ventes 1 000 000 (701 C),
        // achats 700 000 (601 D) -> actif 1 300 000 = passif 1 000 000 + résultat 300 000.
        let balance = Balance::exercice(vec![
            cr_ligne("101", "Capital", 1_000_000),
            d("521", "Banque", 1_300_000),
            cr_ligne("701", "Ventes de marchandises", 1_000_000),
            d("601", "Achats de marchandises", 700_000),
        ]);

        let (b, cr) = generer_etats_complets(&balance).unwrap();
        assert_eq!(b.total_actif_n, m(1_300_000));
        assert_eq!(b.total_passif_n, m(1_300_000));
        assert_eq!(b.rubrique_actif("BS").unwrap().net_n, m(1_300_000));
        assert_eq!(b.rubrique_passif("CA").unwrap().net_n, m(1_000_000));
        assert_eq!(b.rubrique_passif("CJ").unwrap().net_n, m(300_000));
        assert_eq!(b.resultat_net_n, m(300_000));

        assert_eq!(cr.chiffre_affaires_n, m(1_000_000));
        assert_eq!(cr.marge_commerciale_n, m(300_000));
        assert_eq!(cr.resultat_net_n, m(300_000));
        assert_eq!(cr.rubrique("TA").unwrap().montant_n, m(1_000_000));
        assert_eq!(cr.rubrique("RA").unwrap().montant_n, m(700_000));
        assert_eq!(cr.rubrique("RA").unwrap().contribution_n(), m(-700_000));
        assert_eq!(cr.rubrique("XI").unwrap().montant_n, m(300_000));
    }

    #[test]
    fn exemple_complet_pme_beninoise_n_et_n1() {
        let lignes_n = vec![
            cr_ligne("1011", "Capital social", 5_000_000),
            cr_ligne("111", "Réserve légale", 500_000),
            d("129", "Report à nouveau débiteur", 200_000),
            cr_ligne("162", "Emprunts auprès des établissements", 2_000_000),
            d("213", "Logiciels", 600_000),
            cr_ligne("2813", "Amortissements logiciels", 200_000),
            d("231", "Bâtiments", 4_000_000),
            cr_ligne("2831", "Amortissements bâtiments", 400_000),
            d("245", "Matériel de transport", 3_000_000),
            cr_ligne("2845", "Amortissements matériel transport", 1_000_000),
            d("31", "Stocks de marchandises", 1_500_000),
            cr_ligne("391", "Dépréciations des stocks", 100_000),
            cr_ligne("4011", "Fournisseurs", 1_200_000),
            d("4091", "Fournisseurs avances versées", 150_000),
            d("4111", "Clients", 2_360_000),
            cr_ligne("491", "Dépréciations comptes clients", 60_000),
            cr_ligne("4191", "Clients avances reçues", 100_000),
            cr_ligne("422", "Personnel rémunérations dues", 350_000),
            cr_ligne("4441", "État TVA due", 180_000),
            d("4452", "État TVA récupérable", 80_000),
            d("5211", "Banque BOA", 1_100_000),
            cr_ligne("5212", "Banque Ecobank (découvert)", 400_000),
            d("571", "Caisse", 250_000),
            cr_ligne("701", "Ventes de marchandises", 12_000_000),
            cr_ligne("706", "Services vendus", 2_000_000),
            d("601", "Achats de marchandises", 7_500_000),
            cr_ligne("6031", "Variation stocks marchandises", 300_000),
            d("621", "Services extérieurs", 1_200_000),
            d("641", "Impôts et taxes", 300_000),
            d("661", "Rémunérations du personnel", 1_800_000),
            d("664", "Charges sociales", 400_000),
            d("681", "Dotations aux amortissements", 600_000),
            cr_ligne("791", "Reprises d'amortissements", 50_000),
            d("671", "Intérêts des emprunts", 200_000),
            cr_ligne("771", "Intérêts reçus", 50_000),
            cr_ligne("822", "Produits de cessions d'immobilisations", 400_000),
            d("812", "Valeurs comptables des cessions", 300_000),
            d("891", "Impôts sur les bénéfices", 750_000),
        ];

        let lignes_n1 = vec![
            cr_ligne("1011", "Capital social", 5_000_000),
            d("5211", "Banque BOA", 5_400_000),
            cr_ligne("701", "Ventes de marchandises", 2_000_000),
            d("601", "Achats de marchandises", 1_600_000),
        ];

        let balance = Balance::avec_comparatif(
            Periode::annee(2026),
            lignes_n,
            Periode::annee(2025),
            lignes_n1,
        );
        let (tot_d, tot_c) = balance.totaux_n().unwrap();
        assert_eq!(tot_d, tot_c);
        assert_eq!(tot_d, m(26_290_000));

        let (b, cr) = generer_etats_complets(&balance).unwrap();
        assert!(b.possede_comparatif);
        assert_eq!(b.total_actif_brut_n, m(13_040_000));
        assert_eq!(b.total_amort_deprec_n, m(1_760_000));
        assert_eq!(b.total_actif_n, m(11_280_000));
        assert_eq!(b.total_passif_n, m(11_280_000));
        assert_eq!(b.resultat_net_n, m(1_750_000));

        // Vérification du reclassement du découvert bancaire 5212 en DR.
        assert_eq!(b.rubrique_actif("BS").unwrap().net_n, m(1_350_000));
        assert_eq!(b.rubrique_passif("DR").unwrap().net_n, m(400_000));
        // Report à nouveau débiteur en CH négatif.
        assert_eq!(b.rubrique_passif("CH").unwrap().net_n, m(-200_000));

        // Vérification des soldes intermédiaires de gestion N.
        assert_eq!(cr.chiffre_affaires_n, m(14_000_000));
        assert_eq!(cr.marge_commerciale_n, m(4_800_000));
        assert_eq!(cr.valeur_ajoutee_n, m(5_300_000));
        assert_eq!(cr.excedent_brut_exploitation_n, m(3_100_000));
        assert_eq!(cr.resultat_exploitation_n, m(2_550_000));
        assert_eq!(cr.resultat_financier_n, m(-150_000));
        assert_eq!(cr.resultat_activites_ordinaires_n, m(2_400_000));
        assert_eq!(cr.resultat_hao_n, m(100_000));
        assert_eq!(cr.resultat_net_n, m(1_750_000));

        // Vérification du comparatif N-1.
        assert_eq!(b.total_actif_n1, m(5_400_000));
        assert_eq!(b.total_passif_n1, m(5_400_000));
        assert_eq!(b.resultat_net_n1, m(400_000));
        assert_eq!(cr.resultat_net_n1, m(400_000));
    }

    #[test]
    fn perte_nette_et_capital_non_appele() {
        let balance = Balance::exercice(vec![
            cr_ligne("101", "Capital", 1_000_000),
            d("109", "Apporteurs capital non appelé", 200_000),
            d("521", "Banque", 400_000),
            cr_ligne("701", "Ventes", 600_000),
            d("601", "Achats", 1_000_000),
        ]);

        let (b, cr) = generer_etats_complets(&balance).unwrap();
        assert_eq!(cr.resultat_net_n, m(-400_000));
        assert_eq!(b.rubrique_passif("CA").unwrap().net_n, m(1_000_000));
        assert_eq!(b.rubrique_passif("CB").unwrap().net_n, m(-200_000));
        assert_eq!(b.rubrique_passif("CJ").unwrap().net_n, m(-400_000));
        assert_eq!(b.rubrique_passif("CP").unwrap().net_n, m(400_000));
        assert_eq!(b.total_actif_n, m(400_000));
        assert_eq!(b.total_passif_n, m(400_000));
    }

    #[test]
    fn balance_apres_cloture_et_balance_vide() {
        let vide = Balance::default();
        let (bv, crv) = generer_etats_complets(&vide).unwrap();
        assert_eq!(bv.total_actif_n, Montant::ZERO);
        assert_eq!(crv.resultat_net_n, Montant::ZERO);

        let cloturee = Balance::exercice(vec![
            cr_ligne("101", "Capital", 1_000_000),
            cr_ligne("131", "Résultat net : bénéfice", 300_000),
            d("521", "Banque", 1_300_000),
        ]);
        let b = bilan(&cloturee).unwrap();
        assert_eq!(b.total_actif_n, m(1_300_000));
        assert_eq!(b.total_passif_n, m(1_300_000));
        assert_eq!(b.resultat_net_n, m(300_000));
    }

    #[test]
    fn erreurs_desequilibre_compte_non_rattache_et_coherence() {
        let desequilibree = Balance::exercice(vec![
            cr_ligne("101", "Capital", 1_000_000),
            d("521", "Banque", 1_000_001),
        ]);
        let err_bilan = bilan(&desequilibree).unwrap_err();
        assert!(matches!(
            err_bilan,
            ErreurEtats::BilanDesequilibre { exercice: "N", .. }
        ));

        // Compte non rattaché (message exact).
        let inconnue = Balance::exercice(vec![d("9011", "Engagement hors bilan", 100_000)]);
        let err_9011 = bilan(&inconnue).unwrap_err();
        assert_eq!(err_9011.to_string(), "Le compte 9011 n'est rattaché à aucune rubrique");

        let corr_partielle = Correspondances::depuis_csv(
            "compte_ou_plage;etat;colonne_bilan;rubrique;libelle_rubrique;sens_solde;signe\n\
             101;BILAN_PASSIF;PASSIF;CA;Capital;M;+\n",
        )
        .unwrap();
        let avec_4711 = Balance::exercice(vec![d("4711", "Compte d'attente", 50_000)]);
        let err_4711 = bilan_avec_correspondances(&avec_4711, &corr_partielle).unwrap_err();
        assert_eq!(err_4711.to_string(), "Le compte 4711 n'est rattaché à aucune rubrique");

        // Incohérence entre un bilan et un compte de résultat d'exercices distincts.
        let b1 = bilan(&Balance::exercice(vec![
            cr_ligne("101", "Capital", 1_000_000),
            d("521", "Banque", 1_200_000),
            cr_ligne("701", "Ventes", 200_000),
        ]))
        .unwrap();
        let cr2 = compte_resultat(&Balance::exercice(vec![
            cr_ligne("101", "Capital", 1_000_000),
            d("521", "Banque", 1_500_000),
            cr_ligne("701", "Ventes", 500_000),
        ]))
        .unwrap();
        assert!(matches!(
            verifier_coherence(&b1, &cr2),
            Err(ErreurEtats::ResultatsIncoherents { exercice: "N", .. })
        ));

        // Déséquilibre sur N-1 et incohérence sur N-1 détectés également.
        let n1_desequilibre = Balance::avec_comparatif(
            None,
            vec![
                cr_ligne("101", "Capital", 1_000_000),
                d("521", "Banque", 1_000_000),
            ],
            None,
            vec![
                cr_ligne("101", "Capital", 1_000_000),
                d("521", "Banque", 999_999),
            ],
        );
        assert!(matches!(
            bilan(&n1_desequilibre),
            Err(ErreurEtats::BilanDesequilibre { exercice: "N-1", .. })
        ));

        let mut b_faux = b1.clone();
        b_faux.total_actif_n = m(999_999);
        assert!(matches!(
            verifier_coherence(&b_faux, &cr2),
            Err(ErreurEtats::BilanDesequilibre { exercice: "N", .. })
        ));

        let mut b_faux_n1 = b1.clone();
        b_faux_n1.possede_comparatif = true;
        b_faux_n1.total_actif_n1 = m(500);
        assert!(matches!(
            verifier_coherence(&b_faux_n1, &cr2),
            Err(ErreurEtats::BilanDesequilibre { exercice: "N-1", .. })
        ));

        b_faux_n1.total_actif_n1 = Montant::ZERO;
        b_faux_n1.resultat_net_n1 = m(100);
        let cr_ok_n = compte_resultat(&Balance::exercice(vec![
            cr_ligne("101", "Capital", 1_000_000),
            d("521", "Banque", 1_200_000),
            cr_ligne("701", "Ventes", 200_000),
        ]))
        .unwrap();
        assert!(matches!(
            verifier_coherence(&b_faux_n1, &cr_ok_n),
            Err(ErreurEtats::ResultatsIncoherents { exercice: "N-1", .. })
        ));
    }

    #[test]
    fn variation_stock_destockage_et_contributions_n1() {
        let balance = Balance::avec_comparatif(
            None,
            vec![
                cr_ligne("101", "Capital", 1_000_000),
                d("521", "Banque", 1_100_000),
                cr_ligne("701", "Ventes", 1_000_000),
                d("601", "Achats", 700_000),
                d("6031", "Déstockage de marchandises", 200_000),
            ],
            None,
            vec![
                cr_ligne("101", "Capital", 1_000_000),
                d("521", "Banque", 1_200_000),
                cr_ligne("701", "Ventes", 800_000),
                d("601", "Achats", 600_000),
            ],
        );
        let (b, cr) = generer_etats_complets(&balance).unwrap();
        assert_eq!(cr.rubrique("RB").unwrap().montant_n, m(-200_000));
        assert_eq!(cr.marge_commerciale_n, m(100_000));
        assert_eq!(cr.resultat_net_n, m(100_000));
        assert_eq!(b.total_actif_n, m(1_100_000));
        assert_eq!(b.total_passif_n, m(1_100_000));
        assert_eq!(cr.rubrique("TA").unwrap().contribution_n1(), m(800_000));
        assert_eq!(cr.rubrique("RA").unwrap().contribution_n1(), m(-600_000));
        assert_eq!(cr.resultat_net_n1, m(200_000));
    }
}
