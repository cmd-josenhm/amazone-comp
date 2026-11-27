//! Commandes système (propriété de A3 pour l'affichage, A10 pour la licence).

use serde::Serialize;

use super::ErreurCommande;

/// Informations affichées dans « À propos » et la barre d'état.
#[derive(Debug, Serialize)]
pub struct InfosSysteme {
    /// Version du logiciel.
    pub version: String,
    /// Année des paramètres fiscaux embarqués.
    pub annee_fiscale: i32,
    /// Nombre de valeurs fiscales non encore validées par un expert.
    pub valeurs_non_verifiees: usize,
}

/// Renvoie les informations système.
#[tauri::command]
pub fn systeme_infos() -> Result<InfosSysteme, ErreurCommande> {
    let annee = 2026;
    Ok(InfosSysteme {
        version: env!("CARGO_PKG_VERSION").to_owned(),
        annee_fiscale: annee,
        valeurs_non_verifiees: ac_fiscal::ParametresAnnee::nombre_non_verifiees(annee),
    })
}
