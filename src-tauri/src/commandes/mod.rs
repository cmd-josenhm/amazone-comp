//! Commandes IPC appelées par l'interface.
//!
//! **Fichier PARTAGÉ** : chaque agent y ajoute uniquement (1) la ligne `pub mod <module>;` de son
//! module et (2) ses commandes dans `gestionnaire_commandes!`, dans SA section. Rien d'autre.
//! Le code des commandes vit dans `commandes/<module>.rs`, propriété de l'agent du module.

pub mod systeme;
// A1 : pub mod compta;
pub mod ventes;
// A5 : pub mod achats;
// A6 : pub mod tresorerie;
// A7 : pub mod paie;
// A8 : pub mod fiscal;
// A9 : pub mod etats;
// A10 : pub mod compte;

use serde::Serialize;

/// Liste de toutes les commandes exposées à l'interface.
macro_rules! gestionnaire_commandes {
    () => {
        tauri::generate_handler![
            // --- A3 / A10 : système ---
            crate::commandes::systeme::systeme_infos,
            // --- A1 : comptabilité ---
            // --- A4 : ventes ---
            crate::commandes::ventes::ventes_client_creer,
            crate::commandes::ventes::ventes_client_modifier,
            crate::commandes::ventes::ventes_client_obtenir,
            crate::commandes::ventes::ventes_clients_rechercher,
            crate::commandes::ventes::ventes_client_desactiver,
            crate::commandes::ventes::ventes_client_reactiver,
            // --- A5 : achats et stocks ---
            // --- A6 : trésorerie ---
            // --- A7 : paie ---
            // --- A8 : fiscalité ---
            // --- A9 : états et tableau de bord ---
            // --- A10 : compte, licence, sauvegarde, avis, mises à jour ---
        ]
    };
}
pub(crate) use gestionnaire_commandes;

/// Erreur renvoyée à l'interface : un message français compréhensible + un code stable.
#[derive(Debug, Serialize)]
pub struct ErreurCommande {
    /// Code stable pour l'interface (`NON_AUTORISE`, `DESEQUILIBRE`…).
    pub code: String,
    /// Message pour l'utilisateur.
    pub message: String,
}

impl ErreurCommande {
    /// Construit une erreur.
    pub fn new(code: &str, message: impl Into<String>) -> Self {
        ErreurCommande {
            code: code.to_owned(),
            message: message.into(),
        }
    }
}
