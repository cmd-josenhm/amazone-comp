//! Adaptateur de facture normalisée (MECeF / e-MECeF) — ADR 0006.
//!
//! Le flux e-MECeF de la DGI est en **deux temps** : la facture est d'abord *soumise* (la DGI
//! renvoie un identifiant et ses totaux), puis *confirmée* ou *annulée* ; à la confirmation la DGI
//! renvoie les éléments de sécurité (code MECeF/DGI, compteurs, NIM, QR code, date/heure).
//! Ce trait reproduit ce flux pour que le reste du logiciel n'ait pas à changer quand l'API réelle
//! sera branchée.
//!
//! Implémentations :
//! - [`Simulateur`] : disponible maintenant ; toutes ses factures sont marquées
//!   « SIMULATION — NON VALABLE FISCALEMENT ».
//! - `EMecefApi` : à écrire par A4 dès que l'accès API (jeton, URL) est obtenu auprès de la DGI.

use ac_noyau::{Ifu, Montant};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Type de facture e-MECeF.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TypeFacture {
    /// Facture de vente.
    FV,
    /// Facture d'avoir.
    FA,
    /// Facture de vente à l'exportation.
    EV,
    /// Facture d'avoir à l'exportation.
    EA,
}

/// Groupe de taxation e-MECeF (codes A à F, voir `parametres-fiscaux/<année>/emecef.toml`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GroupeTaxation {
    /// Exonéré.
    A,
    /// Taxable TVA.
    B,
    /// Exportation.
    C,
    /// Régime d'exception.
    D,
    /// Régime TPS.
    E,
    /// Réservé.
    F,
}

/// Ligne d'article transmise à la DGI.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArticleNormalise {
    /// Désignation.
    pub nom: String,
    /// Prix unitaire **TTC** en FCFA.
    pub prix_unitaire_ttc: Montant,
    /// Quantité (en millièmes d'unité pour permettre 1,5 kg : 1500 = 1,5).
    pub quantite_milliemes: i64,
    /// Groupe de taxation.
    pub groupe: GroupeTaxation,
    /// Taxe spécifique éventuelle (TTC).
    pub taxe_specifique: Option<Montant>,
}

/// Moyen de paiement déclaré sur la facture.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaiementNormalise {
    /// Code e-MECeF : ESPECES, VIREMENT, CARTEBANCAIRE, MOBILEMONEY, CHEQUES, CREDIT, AUTRE.
    pub type_paiement: String,
    /// Montant.
    pub montant: Montant,
}

/// Client tel que transmis à la DGI.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClientNormalise {
    /// IFU du client s'il en a un.
    pub ifu: Option<Ifu>,
    /// Nom ou raison sociale.
    pub nom: String,
    /// Téléphone ou courriel.
    pub contact: Option<String>,
    /// Adresse.
    pub adresse: Option<String>,
}

/// Demande de normalisation (une facture validée par l'utilisateur).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DemandeNormalisation {
    /// IFU du vendeur.
    pub ifu_vendeur: Ifu,
    /// Type de facture.
    pub type_facture: TypeFacture,
    /// Opérateur (nom de l'utilisateur qui émet).
    pub operateur: String,
    /// Client.
    pub client: ClientNormalise,
    /// Articles.
    pub articles: Vec<ArticleNormalise>,
    /// Paiements.
    pub paiements: Vec<PaiementNormalise>,
    /// Code AIB e-MECeF éventuel (« A », « B »).
    pub code_aib: Option<String>,
    /// Pour un avoir : code MECeF/DGI de la facture d'origine.
    pub reference_origine: Option<String>,
}

/// Réponse intermédiaire après soumission.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FactureSoumise {
    /// Identifiant attribué par la DGI (ou le simulateur).
    pub uid: String,
    /// Total TTC calculé par la DGI (à comparer au total calculé localement).
    pub total_ttc: Montant,
    /// TVA totale calculée par la DGI.
    pub total_tva: Montant,
    /// AIB calculé par la DGI.
    pub total_aib: Montant,
}

/// Décision après soumission.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Decision {
    /// Confirmer : la facture devient définitive.
    Confirmer,
    /// Annuler : la facture n'existe pas fiscalement.
    Annuler,
}

/// Éléments de sécurité à imprimer sur la facture.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FactureNormalisee {
    /// Code MECeF / DGI.
    pub code_mecef: String,
    /// Numéro d'identification de la machine / du compte.
    pub nim: String,
    /// Compteurs (ex. « 12/15 FV »).
    pub compteurs: String,
    /// Date et heure DGI (ISO-8601).
    pub date_heure: String,
    /// Contenu du QR code.
    pub qr_code: String,
    /// Vrai si produit par le simulateur (mention obligatoire sur le document).
    pub simulation: bool,
}

/// Erreurs de normalisation.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ErreurNormalisation {
    /// Pas de connexion : la facture reste « en attente de normalisation ».
    #[error("service de facture normalisée injoignable : la facture reste en attente")]
    Injoignable,
    /// La DGI refuse la facture.
    #[error("facture refusée par la DGI : {0}")]
    Refusee(String),
    /// Écart entre les totaux locaux et ceux de la DGI.
    #[error("écart de totaux avec la DGI : local {local}, DGI {dgi}")]
    EcartTotaux {
        /// Total calculé localement.
        local: Montant,
        /// Total renvoyé par la DGI.
        dgi: Montant,
    },
    /// Demande incomplète.
    #[error("facture incomplète : {0}")]
    Invalide(String),
}

/// Contrat de tout fournisseur de normalisation.
pub trait Normaliseur: Send + Sync {
    /// Nom affiché (« Simulateur », « e-MECeF DGI »…).
    fn nom(&self) -> &'static str;
    /// Étape 1 : soumettre la facture.
    fn soumettre(
        &self,
        demande: &DemandeNormalisation,
    ) -> Result<FactureSoumise, ErreurNormalisation>;
    /// Étape 2 : confirmer ou annuler. Renvoie `None` si annulée.
    fn finaliser(
        &self,
        uid: &str,
        decision: Decision,
    ) -> Result<Option<FactureNormalisee>, ErreurNormalisation>;
}

/// Simulateur hors ligne. **Ne produit jamais une facture fiscalement valable.**
#[derive(Debug, Default)]
pub struct Simulateur;

impl Normaliseur for Simulateur {
    fn nom(&self) -> &'static str {
        "Simulateur (non valable fiscalement)"
    }

    fn soumettre(&self, d: &DemandeNormalisation) -> Result<FactureSoumise, ErreurNormalisation> {
        if d.articles.is_empty() {
            return Err(ErreurNormalisation::Invalide("aucun article".into()));
        }
        let mut total = Montant::ZERO;
        for a in &d.articles {
            let ligne = a
                .prix_unitaire_ttc
                .multiplier(a.quantite_milliemes)
                .map_err(|e| ErreurNormalisation::Invalide(e.to_string()))?;
            // Division entière par 1000 (quantités en millièmes). TODO(A4) : règle d'arrondi DGI.
            let ligne = Montant::new(ligne.francs() / 1000)
                .map_err(|e| ErreurNormalisation::Invalide(e.to_string()))?;
            total = total
                .ajouter(ligne)
                .map_err(|e| ErreurNormalisation::Invalide(e.to_string()))?;
        }
        Ok(FactureSoumise {
            uid: format!("SIM-{}", d.ifu_vendeur),
            total_ttc: total,
            total_tva: Montant::ZERO, // TODO(A4) : ventilation par groupe via ac-fiscal
            total_aib: Montant::ZERO,
        })
    }

    fn finaliser(
        &self,
        uid: &str,
        decision: Decision,
    ) -> Result<Option<FactureNormalisee>, ErreurNormalisation> {
        Ok(match decision {
            Decision::Annuler => None,
            Decision::Confirmer => Some(FactureNormalisee {
                code_mecef: format!("SIMU-{uid}"),
                nim: "SIMULATEUR".into(),
                compteurs: "0/0 FV".into(),
                date_heure: chrono::Utc::now().to_rfc3339(),
                qr_code: format!("SIMULATION;{uid}"),
                simulation: true,
            }),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simulateur_deux_temps() {
        let s = Simulateur;
        let d = DemandeNormalisation {
            ifu_vendeur: Ifu::new("3201912345678").unwrap(),
            type_facture: TypeFacture::FV,
            operateur: "Caisse 1".into(),
            client: ClientNormalise {
                ifu: None,
                nom: "Client comptant".into(),
                contact: None,
                adresse: None,
            },
            articles: vec![ArticleNormalise {
                nom: "Sac de ciment".into(),
                prix_unitaire_ttc: Montant::new(5_900).unwrap(),
                quantite_milliemes: 3_000,
                groupe: GroupeTaxation::B,
                taxe_specifique: None,
            }],
            paiements: vec![],
            code_aib: None,
            reference_origine: None,
        };
        let soumise = s.soumettre(&d).unwrap();
        assert_eq!(soumise.total_ttc, Montant::new(17_700).unwrap());
        let f = s
            .finaliser(&soumise.uid, Decision::Confirmer)
            .unwrap()
            .unwrap();
        assert!(f.simulation);
        assert!(s
            .finaliser(&soumise.uid, Decision::Annuler)
            .unwrap()
            .is_none());
    }
}
