//! État partagé de l'application (dossier ouvert, session).

use std::sync::Mutex;

use ac_donnees::{Base, Session};

/// État global géré par Tauri.
#[derive(Default)]
pub struct EtatApplication {
    /// Dossier ouvert (None tant que l'utilisateur ne s'est pas connecté).
    pub base: Mutex<Option<Base>>,
    /// Utilisateur connecté.
    pub session: Mutex<Option<Session>>,
}

/// Toutes les migrations de toutes les crates, dans l'ordre des plages.
pub fn toutes_les_migrations() -> Vec<&'static [ac_donnees::Migration]> {
    vec![
        ac_donnees::base::migrations(),
        ac_compta::migrations(),
        ac_ventes::migrations(),
        ac_achats::migrations(),
        ac_tresorerie::migrations(),
        ac_paie::migrations(),
        ac_etats::migrations(),
        ac_licence::migrations(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toutes_les_migrations_s_appliquent() {
        let mut b = Base::en_memoire().unwrap();
        b.migrer(&toutes_les_migrations()).unwrap();
    }
}
