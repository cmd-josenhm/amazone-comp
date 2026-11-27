//! Point d'entrée du service « amazone-plateforme ».
//!
//! La configuration vient des variables d'environnement (voir `.env.exemple`) ; la clé
//! **privée** de signature des licences ne provient jamais du dépôt (règle R7). La clé
//! **publique** correspondante est affichée au démarrage : c'est celle à embarquer dans
//! l'application (lot 2).

#![forbid(unsafe_code)]

use std::net::SocketAddr;

use amazone_plateforme::{application, Config, ErreurDemarrage, Etat};

#[tokio::main]
async fn main() -> Result<(), ErreurDemarrage> {
    let config = Config::depuis_env()?;
    let etat = Etat::creer(&config).await?;
    println!(
        "Clé publique de licence (à embarquer dans l'application) : {}",
        etat.signataire.cle_publique_hex()
    );
    let application = application(etat);
    let adresse: SocketAddr = config.adresse.parse().map_err(|_| {
        ErreurDemarrage::Serveur(format!("PLATEFORME_ADRESSE invalide : {}", config.adresse))
    })?;
    let ecouteur = tokio::net::TcpListener::bind(adresse).await.map_err(|e| {
        ErreurDemarrage::Serveur(format!("impossible d'écouter sur {adresse} : {e}"))
    })?;
    println!("Plateforme AMAZONE COMP à l'écoute sur {adresse}");
    axum::serve(
        ecouteur,
        application.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(arret())
    .await
    .map_err(|e| ErreurDemarrage::Serveur(format!("arrêt inattendu du serveur : {e}")))?;
    Ok(())
}

/// Attend l'arrêt demandé par l'administrateur (Ctrl+C).
async fn arret() {
    if tokio::signal::ctrl_c().await.is_err() {
        eprintln!("[avertissement] écoute du signal d'arrêt impossible ; le serveur continue.");
    }
}
