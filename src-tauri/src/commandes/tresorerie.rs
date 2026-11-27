//! Commandes Trésorerie (A6). — Propriétaire : A6.
//!
//! Règles (C8 + AGENTS.md) : nom `module_action` snake_case ; première instruction
//! `session.exiger(Permission::…)` ; erreurs via `ErreurCommande` (code + message).

use ac_donnees::{Permission, Session};
use ac_tresorerie::{CreerCompte, TresorerieCompte, TresorerieOperateur, TresorerieSolde};
use std::sync::MutexGuard;
use tauri::State;

use super::ErreurCommande;
use crate::etat::EtatApplication;

/// Ouvre la session et la base pour une commande Trésorerie (C2, C8).
fn ouvrir<'r>(
    etat: &State<'r, EtatApplication>,
    permission: Permission,
) -> Result<(Session, MutexGuard<'r, EtatApplication>), ErreurCommande> {
    let etat = etat.inner();
    let session: Session = etat
        .session
        .lock()
        .map_err(|_| ErreurCommande::new("ETAT_BLOQUE", "Verrou momentané sur la session"))?
        .as_ref()
        .cloned()
        .ok_or_else(|| ErreurCommande::new("AUCUNE_SESSION", "Aucune session connectée : se connecter d'abord"))?;
    session
        .exiger(permission)
        .map_err(|e| ErreurCommande::new("PERMISSION_REFUSEE", e.to_string()))?;
    let base = etat
        .base
        .lock()
        .map_err(|_| ErreurCommande::new("ETAT_BLOQUE", "Verrou momentané sur la base"))?;
    Ok((session, base))
}

/// Map une erreur Trésorerie à `ErreurCommande`.
fn erreur_tresorerie(e: ac_tresorerie::ErreurTresorerie) -> ErreurCommande {
    ErreurCommande::new(e.code(), e.to_string())
}

/// Map une erreur de la couche données (A2) à `ErreurCommande`.
fn erreur_donnees(e: ac_donnees::ErreurDonnees) -> ErreurCommande {
    ErreurCommande::new("ERREUR_DONNEES", e.to_string())
}

/// Liste les comptes de trésorerie actifs.
#[tauri::command]
pub fn tresorerie_comptes(etat: State<'_, EtatApplication>) -> Result<Vec<TresorerieCompte>, ErreurCommande> {
    let (_session, base) = ouvrir(&etat, Permission::Consulter)?;
    let base = base
        .as_ref()
        .ok_or_else(|| ErreurCommande::new("AUCUNE_BASE", "Aucun dossier ouvert : ouvrir un dossier d'abord"))?;
    let tx = base.transaction().map_err(erreur_donnees)?;
    ac_tresorerie::compte::comptes(&tx).map_err(erreur_tresorerie)
}

/// Crée un compte de trésorerie (BANQUE / CAISSE / MOBILE_MONEY).
#[tauri::command]
pub fn tresorerie_compte_creer(
    etat: State<'_, EtatApplication>,
    params: CreerCompte,
) -> Result<TresorerieCompte, ErreurCommande> {
    let (session, mut base) = ouvrir(&etat, Permission::Saisir)?;
    let base = base
        .as_mut()
        .ok_or_else(|| ErreurCommande::new("AUCUNE_BASE", "Aucun dossier ouvert : ouvrir un dossier d'abord"))?;
    let tx = base.transaction().map_err(erreur_donnees)?;
    let compte = ac_tresorerie::compte::creer_compte(&tx, &session, &params).map_err(erreur_tresorerie)?;
    tx.commit().map_err(erreur_donnees)?;
    Ok(compte)
}

/// Lit un compte de trésorerie par identifiant.
#[tauri::command]
pub fn tresorerie_compte_afficher(
    etat: State<'_, EtatApplication>,
    id: i64,
) -> Result<TresorerieCompte, ErreurCommande> {
    let (_session, base) = ouvrir(&etat, Permission::Consulter)?;
    let base = base
        .as_ref()
        .ok_or_else(|| ErreurCommande::new("AUCUNE_BASE", "Aucun dossier ouvert : ouvrir un dossier d'abord"))?;
    let tx = base.transaction().map_err(erreur_donnees)?;
    ac_tresorerie::compte::compte_par_id(&tx, id).map_err(erreur_tresorerie)
}

/// Soldes comptables des comptes de trésorerie.
#[tauri::command]
pub fn tresorerie_soldes(etat: State<'_, EtatApplication>) -> Result<Vec<TresorerieSolde>, ErreurCommande> {
    let (_session, base) = ouvrir(&etat, Permission::Consulter)?;
    let base = base
        .as_ref()
        .ok_or_else(|| ErreurCommande::new("AUCUNE_BASE", "Aucun dossier ouvert : ouvrir un dossier d'abord"))?;
    let tx = base.transaction().map_err(erreur_donnees)?;
    ac_tresorerie::compte::soldes_comptes(&tx).map_err(erreur_tresorerie)
}

/// Opérateurs de trésorerie (pré-remplis, désactivables — `tresorerie_operateurs`).
#[tauri::command]
pub fn tresorerie_operateurs(etat: State<'_, EtatApplication>) -> Result<Vec<TresorerieOperateur>, ErreurCommande> {
    let (_session, base) = ouvrir(&etat, Permission::Consulter)?;
    let base = base
        .as_ref()
        .ok_or_else(|| ErreurCommande::new("AUCUNE_BASE", "Aucun dossier ouvert : ouvrir un dossier d'abord"))?;
    let tx = base.transaction().map_err(erreur_donnees)?;
    ac_tresorerie::compte::operateurs(&tx).map_err(erreur_tresorerie)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn erreur_tresorerie_cartographie_les_codes() {
        let e = erreur_tresorerie(ac_tresorerie::ErreurTresorerie::LibelleVide);
        assert_eq!(e.code, "LIBELLE_VIDE");
    }
}
