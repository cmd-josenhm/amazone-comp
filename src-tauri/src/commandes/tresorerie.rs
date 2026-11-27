//! Commandes Trésorerie (A6). — Propriétaire : A6.
//!
//! Règles (C8 + AGENTS.md) : nom `module_action` snake_case ; première instruction
//! `session.exiger(Permission::…)` ; erreurs via `ErreurCommande` (code + message).

use std::sync::MutexGuard;

use ac_donnees::{Base, Permission, Session};
use ac_tresorerie::{
    CreerCompte, CreerOperateur, TresorerieCompte, TresorerieConfiguration, TresorerieOperateur,
    TresorerieSolde,
};
use tauri::State;

use super::ErreurCommande;
use crate::etat::EtatApplication;

/// Ouvre la session et verrouille la base pour une commande Trésorerie (C2, C8).
fn ouvrir<'r>(
    etat: &State<'r, EtatApplication>,
    permission: Permission,
) -> Result<(Session, MutexGuard<'r, Option<Base>>), ErreurCommande> {
    let etat = etat.inner();
    let session = etat
        .session
        .lock()
        .map_err(|_| ErreurCommande::new("ETAT_BLOQUE", "Verrou momentané sur la session"))?
        .as_ref()
        .cloned()
        .ok_or_else(|| {
            ErreurCommande::new(
                "AUCUNE_SESSION",
                "Aucune session connectée : se connecter d'abord",
            )
        })?;
    session
        .exiger(permission)
        .map_err(|erreur| ErreurCommande::new("PERMISSION_REFUSEE", erreur.to_string()))?;
    let base = etat
        .base
        .lock()
        .map_err(|_| ErreurCommande::new("ETAT_BLOQUE", "Verrou momentané sur la base"))?;
    Ok((session, base))
}

/// Mappe une erreur Trésorerie à `ErreurCommande`.
fn erreur_tresorerie(erreur: ac_tresorerie::ErreurTresorerie) -> ErreurCommande {
    ErreurCommande::new(erreur.code(), erreur.to_string())
}

/// Mappe une erreur de la couche données (A2) à `ErreurCommande`.
fn erreur_donnees(erreur: ac_donnees::ErreurDonnees) -> ErreurCommande {
    ErreurCommande::new("ERREUR_DONNEES", erreur.to_string())
}

/// Liste les comptes de trésorerie actifs, avec les numéros masqués.
#[tauri::command]
pub fn tresorerie_comptes(
    etat: State<'_, EtatApplication>,
) -> Result<Vec<TresorerieCompte>, ErreurCommande> {
    let (_session, mut garde_base) = ouvrir(&etat, Permission::Consulter)?;
    let base = garde_base
        .as_mut()
        .ok_or_else(|| {
            ErreurCommande::new(
                "AUCUNE_BASE",
                "Aucun dossier ouvert : ouvrir un dossier d'abord",
            )
        })?;
    let tx = base.transaction().map_err(erreur_donnees)?;
    ac_tresorerie::compte::comptes(&tx).map_err(erreur_tresorerie)
}

/// Crée un compte et, si son solde d'ouverture est non nul, son écriture AN.
#[tauri::command]
pub fn tresorerie_compte_creer(
    etat: State<'_, EtatApplication>,
    params: CreerCompte,
) -> Result<TresorerieCompte, ErreurCommande> {
    // Un solde d'ouverture produit une écriture validée et immuable.
    let (session, mut garde_base) = ouvrir(&etat, Permission::Valider)?;
    let base = garde_base
        .as_mut()
        .ok_or_else(|| {
            ErreurCommande::new(
                "AUCUNE_BASE",
                "Aucun dossier ouvert : ouvrir un dossier d'abord",
            )
        })?;
    let tx = base.transaction().map_err(erreur_donnees)?;
    let compte = ac_tresorerie::compte::creer_compte(&tx, &session, &params)
        .map_err(erreur_tresorerie)?;
    tx.commit().map_err(erreur_donnees)?;
    Ok(compte)
}

/// Lit un compte de trésorerie par identifiant, avec le numéro masqué.
#[tauri::command]
pub fn tresorerie_compte_afficher(
    etat: State<'_, EtatApplication>,
    id: i64,
) -> Result<TresorerieCompte, ErreurCommande> {
    let (_session, mut garde_base) = ouvrir(&etat, Permission::Consulter)?;
    let base = garde_base
        .as_mut()
        .ok_or_else(|| {
            ErreurCommande::new(
                "AUCUNE_BASE",
                "Aucun dossier ouvert : ouvrir un dossier d'abord",
            )
        })?;
    let tx = base.transaction().map_err(erreur_donnees)?;
    ac_tresorerie::compte::compte_par_id(&tx, id).map_err(erreur_tresorerie)
}

/// Renvoie les soldes comptables des comptes de trésorerie actifs.
#[tauri::command]
pub fn tresorerie_soldes(
    etat: State<'_, EtatApplication>,
) -> Result<Vec<TresorerieSolde>, ErreurCommande> {
    let (_session, mut garde_base) = ouvrir(&etat, Permission::Consulter)?;
    let base = garde_base
        .as_mut()
        .ok_or_else(|| {
            ErreurCommande::new(
                "AUCUNE_BASE",
                "Aucun dossier ouvert : ouvrir un dossier d'abord",
            )
        })?;
    let tx = base.transaction().map_err(erreur_donnees)?;
    ac_tresorerie::compte::soldes_comptes(&tx).map_err(erreur_tresorerie)
}

/// Lit la configuration Mobile Money du dossier.
#[tauri::command]
pub fn tresorerie_configuration(
    etat: State<'_, EtatApplication>,
) -> Result<TresorerieConfiguration, ErreurCommande> {
    let (_session, mut garde_base) = ouvrir(&etat, Permission::Consulter)?;
    let base = garde_base
        .as_mut()
        .ok_or_else(|| {
            ErreurCommande::new(
                "AUCUNE_BASE",
                "Aucun dossier ouvert : ouvrir un dossier d'abord",
            )
        })?;
    let tx = base.transaction().map_err(erreur_donnees)?;
    ac_tresorerie::compte::configuration(&tx).map_err(erreur_tresorerie)
}

/// Active ou désactive Mobile Money pour le dossier (administrateur uniquement).
#[tauri::command]
pub fn tresorerie_mobile_money_configurer(
    etat: State<'_, EtatApplication>,
    actif: bool,
) -> Result<TresorerieConfiguration, ErreurCommande> {
    let (session, mut garde_base) = ouvrir(&etat, Permission::Administrer)?;
    let base = garde_base
        .as_mut()
        .ok_or_else(|| {
            ErreurCommande::new(
                "AUCUNE_BASE",
                "Aucun dossier ouvert : ouvrir un dossier d'abord",
            )
        })?;
    let tx = base.transaction().map_err(erreur_donnees)?;
    let configuration =
        ac_tresorerie::compte::configurer_mobile_money(&tx, &session, actif)
            .map_err(erreur_tresorerie)?;
    tx.commit().map_err(erreur_donnees)?;
    Ok(configuration)
}

/// Liste les opérateurs Mobile Money pré-remplis ou ajoutés.
#[tauri::command]
pub fn tresorerie_operateurs(
    etat: State<'_, EtatApplication>,
) -> Result<Vec<TresorerieOperateur>, ErreurCommande> {
    let (_session, mut garde_base) = ouvrir(&etat, Permission::Consulter)?;
    let base = garde_base
        .as_mut()
        .ok_or_else(|| {
            ErreurCommande::new(
                "AUCUNE_BASE",
                "Aucun dossier ouvert : ouvrir un dossier d'abord",
            )
        })?;
    let tx = base.transaction().map_err(erreur_donnees)?;
    ac_tresorerie::compte::operateurs(&tx).map_err(erreur_tresorerie)
}

/// Ajoute un opérateur Mobile Money (réservé à l'administrateur du dossier).
#[tauri::command]
pub fn tresorerie_operateur_creer(
    etat: State<'_, EtatApplication>,
    params: CreerOperateur,
) -> Result<TresorerieOperateur, ErreurCommande> {
    let (session, mut garde_base) = ouvrir(&etat, Permission::Administrer)?;
    let base = garde_base
        .as_mut()
        .ok_or_else(|| {
            ErreurCommande::new(
                "AUCUNE_BASE",
                "Aucun dossier ouvert : ouvrir un dossier d'abord",
            )
        })?;
    let tx = base.transaction().map_err(erreur_donnees)?;
    let operateur = ac_tresorerie::compte::creer_operateur(&tx, &session, &params)
        .map_err(erreur_tresorerie)?;
    tx.commit().map_err(erreur_donnees)?;
    Ok(operateur)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn erreur_tresorerie_cartographie_les_codes() {
        let erreur = erreur_tresorerie(ac_tresorerie::ErreurTresorerie::LibelleVide);
        assert_eq!(erreur.code, "LIBELLE_VIDE");
    }
}
