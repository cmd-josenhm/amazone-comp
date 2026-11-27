//! Application de bureau AMAZONE COMP (Tauri 2).
//!
//! **Propriétaire : agent A10** (démarrage, état, mise à jour, installateur), sauf
//! `commandes/<module>.rs` qui appartiennent à l'agent du module.
//!
//! Règles pour les commandes (tous agents) :
//! - nom `module_action` en snake_case (`ventes_creer_facture`) ;
//! - première instruction : `session.exiger(Permission::…)` ;
//! - jamais de logique métier ici : appeler la crate du module ;
//! - erreurs renvoyées via [`commandes::ErreurCommande`] (message français pour l'utilisateur).

#![forbid(unsafe_code)]

pub mod commandes;
pub mod etat;

/// Construit et lance l'application.
pub fn lancer() {
    let resultat = tauri::Builder::default()
        .manage(etat::EtatApplication::default())
        .invoke_handler(commandes::gestionnaire_commandes!())
        .run(tauri::generate_context!());
    if let Err(e) = resultat {
        // Pas de panic : on journalise et on quitte proprement.
        eprintln!("AMAZONE COMP n'a pas pu démarrer : {e}");
        std::process::exit(1);
    }
}
