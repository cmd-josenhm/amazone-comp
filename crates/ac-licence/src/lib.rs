//! # ac-licence
//!
//! Compte plateforme, licence signée de 6 mois, sauvegarde distante, avis et mises à jour
//! (côté application). **Propriétaire : agent A10.** Voir ADR 0004 et `PROMPT_A10.md`.
//!
//! État : squelette — types du contrat et règle de calcul de l'état de licence (sans signature).
//! À livrer par A10 : vérification Ed25519, empreinte du poste, client HTTPS de la plateforme,
//! renouvellement, sauvegarde chiffrée distante, envoi des avis.

#![forbid(unsafe_code)]

use ac_noyau::Date;
use serde::{Deserialize, Serialize};

/// Durée de validité d'une licence gratuite, en mois.
pub const DUREE_MOIS: u32 = 6;
/// Jours avant expiration à partir desquels on renouvelle automatiquement si Internet est là.
pub const FENETRE_RENOUVELLEMENT_JOURS: i64 = 30;
/// Jours de grâce après expiration avant le mode consultation.
pub const GRACE_JOURS: i64 = 15;

/// Contenu signé de la licence (charge utile du jeton).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Licence {
    /// Version du format de jeton.
    pub format: u32,
    /// Identifiant du compte sur la plateforme.
    pub compte_id: String,
    /// IFU de l'entreprise.
    pub ifu: String,
    /// Empreinte (hash) du poste — jamais d'identifiant matériel en clair.
    pub empreinte_poste: String,
    /// Date d'émission.
    pub emise_le: Date,
    /// Date d'expiration (émission + 6 mois).
    pub expire_le: Date,
    /// Offre : « GRATUITE » en V1.
    pub offre: String,
}

/// État de la licence à une date donnée.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EtatLicence {
    /// Valide, rien à faire.
    Valide {
        /// Jours restants.
        jours_restants: i64,
    },
    /// Valide mais à renouveler (moins de 30 jours) : renouvellement automatique si connecté.
    ARenouveler {
        /// Jours restants.
        jours_restants: i64,
    },
    /// Expirée depuis moins de 15 jours : tout fonctionne, bandeau d'alerte permanent.
    Grace {
        /// Jours de grâce restants.
        jours_restants: i64,
    },
    /// Mode consultation : lecture, exports et sauvegarde locale seulement. Jamais de blocage des données.
    Consultation,
}

impl EtatLicence {
    /// Vrai si la saisie est autorisée.
    pub fn saisie_autorisee(self) -> bool {
        !matches!(self, EtatLicence::Consultation)
    }
}

/// Calcule l'état d'une licence **dont la signature a déjà été vérifiée**.
pub fn etat(licence: &Licence, aujourd_hui: Date) -> EtatLicence {
    let restants = (licence.expire_le - aujourd_hui).num_days();
    if restants > FENETRE_RENOUVELLEMENT_JOURS {
        EtatLicence::Valide {
            jours_restants: restants,
        }
    } else if restants >= 0 {
        EtatLicence::ARenouveler {
            jours_restants: restants,
        }
    } else if -restants <= GRACE_JOURS {
        EtatLicence::Grace {
            jours_restants: GRACE_JOURS + restants,
        }
    } else {
        EtatLicence::Consultation
    }
}

/// Migrations de `ac-licence` (plage 8000–8999).
pub fn migrations() -> &'static [ac_donnees::Migration] {
    &[]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(a: i32, m: u32, j: u32) -> Date {
        Date::from_ymd_opt(a, m, j).unwrap()
    }

    #[test]
    fn cycle_de_vie() {
        let l = Licence {
            format: 1,
            compte_id: "c1".into(),
            ifu: "3201912345678".into(),
            empreinte_poste: "x".into(),
            emise_le: d(2026, 1, 1),
            expire_le: d(2026, 7, 1),
            offre: "GRATUITE".into(),
        };
        assert!(matches!(
            etat(&l, d(2026, 3, 1)),
            EtatLicence::Valide { .. }
        ));
        assert_eq!(
            etat(&l, d(2026, 6, 21)),
            EtatLicence::ARenouveler { jours_restants: 10 }
        );
        assert_eq!(
            etat(&l, d(2026, 7, 1)),
            EtatLicence::ARenouveler { jours_restants: 0 }
        );
        assert_eq!(
            etat(&l, d(2026, 7, 6)),
            EtatLicence::Grace { jours_restants: 10 }
        );
        assert_eq!(etat(&l, d(2026, 7, 17)), EtatLicence::Consultation);
        assert!(!EtatLicence::Consultation.saisie_autorisee());
    }
}
