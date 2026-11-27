//! Commandes IPC du module Ventes — Propriétaire : agent A4 (contrat C8).
//!
//! Jamais de logique métier ici : chaque commande vérifie la permission puis appelle
//! `ac_ventes::clients`. Les montants transmis à l'interface sont des entiers FCFA.

use ac_donnees::{Permission, Session};
use ac_ventes::clients::{self, Client, InformationsClient, RECHERCHE_LIMITE_DEFAUT};
use ac_ventes::ErreurVentes;
use serde::{Deserialize, Serialize};
use tauri::State;

use super::ErreurCommande;
use crate::etat::EtatApplication;

/// Critères de recherche de clients (commande `ventes_clients_rechercher`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CritereRecherche {
    /// Texte cherché dans le nom, le code, l'IFU et le téléphone (vide = tout lister).
    pub texte: String,
    /// Inclure les clients désactivés.
    pub inclure_inactifs: bool,
    /// Nombre maximal de résultats (50 par défaut, 500 au plus).
    pub limite: Option<u32>,
}

impl From<ErreurVentes> for ErreurCommande {
    fn from(e: ErreurVentes) -> Self {
        let code = match &e {
            ErreurVentes::NomObligatoire
            | ErreurVentes::NomTropLong(_)
            | ErreurVentes::MotifExonerationObligatoire
            | ErreurVentes::CourrielInvalide(_)
            | ErreurVentes::Noyau(_) => "VALIDATION",
            ErreurVentes::ClientInconnu(_) => "CLIENT_INCONNU",
            ErreurVentes::ClientComptantProtege => "CLIENT_COMPTANT_PROTEGE",
            ErreurVentes::ClientDejaInactif(_) | ErreurVentes::ClientDejaActif(_) => "ETAT_CLIENT",
            ErreurVentes::IfuDejaUtilise(_) => "IFU_DEJA_UTILISE",
            ErreurVentes::DonneeIncoherente(_) => "DONNEES",
            ErreurVentes::Donnees(ac_donnees::ErreurDonnees::NonAutorise(_)) => "NON_AUTORISE",
            ErreurVentes::Donnees(_) => "DONNEES",
        };
        ErreurCommande::new(code, e.to_string())
    }
}

/// Exécute `action` sur le dossier ouvert avec la session de l'utilisateur.
///
/// Avant l'action, la permission demandée par `action` est vérifiée dans `ac-ventes` (ou ici
/// pour les lectures) : l'interface ne protège jamais rien.
fn avec_dossier<T>(
    etat: &EtatApplication,
    action: impl FnOnce(&mut ac_donnees::Base, &Session) -> ac_ventes::Resultat<T>,
) -> Result<T, ErreurCommande> {
    let message_verrou = "L'application est dans un état imprévu. Fermez-la puis relancez-la.";
    let mut garde_base = etat
        .base
        .lock()
        .map_err(|_| ErreurCommande::new("ETAT_VERROUILLE", message_verrou))?;
    let garde_session = etat
        .session
        .lock()
        .map_err(|_| ErreurCommande::new("ETAT_VERROUILLE", message_verrou))?;
    let (Some(base), Some(session)) = (garde_base.as_mut(), garde_session.as_ref()) else {
        return Err(ErreurCommande::new(
            "SESSION_REQUISE",
            "Connectez-vous à un dossier d'entreprise pour gérer les clients.",
        ));
    };
    action(base, session).map_err(ErreurCommande::from)
}

/// Crée un client (code `CLI-…` attribué automatiquement).
#[tauri::command]
pub fn ventes_client_creer(
    etat: State<'_, EtatApplication>,
    informations: InformationsClient,
) -> Result<Client, ErreurCommande> {
    avec_dossier(&etat, |base, session| {
        session.exiger(Permission::Saisir)?;
        let tx = base.transaction()?;
        let client = clients::creer(&tx, session, &informations)?;
        tx.commit()?;
        Ok(client)
    })
}

/// Modifie un client (le client « Comptant » est protégé).
#[tauri::command]
pub fn ventes_client_modifier(
    etat: State<'_, EtatApplication>,
    id: i64,
    informations: InformationsClient,
) -> Result<Client, ErreurCommande> {
    avec_dossier(&etat, |base, session| {
        session.exiger(Permission::Saisir)?;
        let tx = base.transaction()?;
        let client = clients::modifier(&tx, session, id, &informations)?;
        tx.commit()?;
        Ok(client)
    })
}

/// Lit un client par son identifiant interne.
#[tauri::command]
pub fn ventes_client_obtenir(
    etat: State<'_, EtatApplication>,
    id: i64,
) -> Result<Client, ErreurCommande> {
    avec_dossier(&etat, |base, session| {
        session.exiger(Permission::Consulter)?;
        clients::obtenir(base.connexion(), id)?
            .ok_or(ErreurVentes::ClientInconnu(id))
    })
}

/// Recherche des clients (nom, code, IFU, téléphone).
#[tauri::command]
pub fn ventes_clients_rechercher(
    etat: State<'_, EtatApplication>,
    recherche: CritereRecherche,
) -> Result<Vec<Client>, ErreurCommande> {
    avec_dossier(&etat, |base, session| {
        session.exiger(Permission::Consulter)?;
        clients::rechercher(
            base.connexion(),
            &recherche.texte,
            recherche.inclure_inactifs,
            recherche.limite.unwrap_or(RECHERCHE_LIMITE_DEFAUT),
        )
    })
}

/// Désactive un client (jamais de suppression d'un client facturé).
#[tauri::command]
pub fn ventes_client_desactiver(
    etat: State<'_, EtatApplication>,
    id: i64,
) -> Result<(), ErreurCommande> {
    avec_dossier(&etat, |base, session| {
        session.exiger(Permission::Saisir)?;
        let tx = base.transaction()?;
        clients::desactiver(&tx, session, id)?;
        tx.commit()?;
        Ok(())
    })
}

/// Réactive un client désactivé.
#[tauri::command]
pub fn ventes_client_reactiver(
    etat: State<'_, EtatApplication>,
    id: i64,
) -> Result<(), ErreurCommande> {
    avec_dossier(&etat, |base, session| {
        session.exiger(Permission::Saisir)?;
        let tx = base.transaction()?;
        clients::reactiver(&tx, session, id)?;
        tx.commit()?;
        Ok(())
    })
}
