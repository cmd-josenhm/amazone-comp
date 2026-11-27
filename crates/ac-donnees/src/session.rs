//! Utilisateurs, rôles et permissions.
//!
//! Les droits sont **toujours** vérifiés côté Rust, au début de chaque commande Tauri, par
//! [`Session::exiger`]. L'interface peut masquer des boutons, mais ce n'est jamais une protection.

use serde::{Deserialize, Serialize};

use crate::{ErreurDonnees, Resultat};

/// Rôles disponibles en V1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Role {
    /// Tous les droits, y compris utilisateurs, clôture, paramètres, restauration.
    Administrateur,
    /// Saisie, validation, déclarations, états. Pas de gestion des utilisateurs.
    Comptable,
    /// Saisie de brouillons (factures, achats, encaissements). Pas de validation.
    Saisie,
    /// Paie uniquement (salaires confidentiels), plus la consultation.
    Paie,
    /// Consultation et exports uniquement.
    Lecture,
}

/// Permissions élémentaires.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Permission {
    /// Lire les données comptables et commerciales.
    Consulter,
    /// Créer / modifier des brouillons.
    Saisir,
    /// Valider des documents et écritures (irréversible, contre-passation seulement).
    Valider,
    /// Accéder à la paie (salaires).
    GererPaie,
    /// Préparer et exporter les déclarations fiscales.
    Declarer,
    /// Clôturer une période ou un exercice.
    Cloturer,
    /// Gérer utilisateurs, paramètres du dossier, restauration de sauvegarde.
    Administrer,
}

impl Role {
    /// Vrai si le rôle accorde la permission.
    pub fn autorise(self, p: Permission) -> bool {
        use Permission as P;
        match self {
            Role::Administrateur => true,
            Role::Comptable => !matches!(p, P::Administrer | P::GererPaie),
            Role::Saisie => matches!(p, P::Consulter | P::Saisir),
            Role::Paie => matches!(p, P::Consulter | P::GererPaie),
            Role::Lecture => matches!(p, P::Consulter),
        }
    }
}

/// Utilisateur connecté.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Session {
    /// Identifiant en base.
    pub utilisateur_id: i64,
    /// Rôle.
    pub role: Role,
}

impl Session {
    /// Erreur si l'utilisateur n'a pas la permission.
    pub fn exiger(&self, p: Permission) -> Resultat<()> {
        if self.role.autorise(p) {
            Ok(())
        } else {
            Err(ErreurDonnees::NonAutorise(p))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matrice() {
        assert!(Role::Administrateur.autorise(Permission::Administrer));
        assert!(!Role::Comptable.autorise(Permission::Administrer));
        assert!(!Role::Comptable.autorise(Permission::GererPaie));
        assert!(Role::Comptable.autorise(Permission::Valider));
        assert!(!Role::Saisie.autorise(Permission::Valider));
        assert!(Role::Paie.autorise(Permission::GererPaie));
        assert!(!Role::Lecture.autorise(Permission::Saisir));
        let s = Session {
            utilisateur_id: 1,
            role: Role::Lecture,
        };
        assert!(s.exiger(Permission::Consulter).is_ok());
        assert!(s.exiger(Permission::Valider).is_err());
    }
}
