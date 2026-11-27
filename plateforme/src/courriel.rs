//! Envoi des courriels de la plateforme (codes de vérification, réinitialisation du mot de passe).
//!
//! ADR 0008 : « E-mails : SMTP de l'hébergeur ou d'un fournisseur avec offre gratuite
//! suffisante (choix documenté par A10) ». Le transport [`EnvoyeurJournal`] (affichage
//! dans les journaux du serveur) est le transport par défaut, adapté au développement
//! et aux tests ; le transport SMTP sera branché sur l'hébergeur retenu, sans changer
//! les appels. Aucun service tiers payant (règle R12).

use std::sync::Mutex;

/// Message électronique prêt à être envoyé.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageCourriel {
    /// Destinataire.
    pub dest: String,
    /// Sujet, en français.
    pub sujet: String,
    /// Corps du message, en français.
    pub corps: String,
    /// Code à usage unique contenu dans le message (pratique pour les tests).
    pub code: String,
}

/// Erreur d'envoi d'un courriel.
#[derive(Debug, thiserror::Error)]
#[error("Impossible d'envoyer le courriel : {0}")]
pub struct ErreurCourriel(String);

/// Transport de courriels.
pub trait EnvoyeurCourriel: Send + Sync {
    /// Envoie un message. En cas d'échec, l'erreur ne contient aucune donnée personnelle.
    fn envoyer(&self, message: &MessageCourriel) -> Result<(), ErreurCourriel>;
}

/// Transport par journal : affiche le message dans les journaux du serveur (`docker logs`).
///
/// Destiné au développement, aux tests et au démarrage du service avant le branchement
/// SMTP. En production, utilisez le transport SMTP de l'hébergeur (voir
/// `docs/modules/plateforme.md`).
pub struct EnvoyeurJournal;

impl EnvoyeurCourriel for EnvoyeurJournal {
    fn envoyer(&self, message: &MessageCourriel) -> Result<(), ErreurCourriel> {
        eprintln!(
            "[courriel] À : {} | Sujet : {} | {}",
            message.dest, message.sujet, message.corps
        );
        Ok(())
    }
}

/// Transport en mémoire : conserve les messages envoyés. Destiné **aux tests**.
#[derive(Default)]
pub struct EnvoyeurMemoire {
    boite: Mutex<Vec<MessageCourriel>>,
}

impl EnvoyeurMemoire {
    /// Crée une boîte vide.
    pub fn nouveau() -> Self {
        Self::default()
    }

    /// Tous les messages envoyés, dans l'ordre.
    pub fn messages(&self) -> Vec<MessageCourriel> {
        match self.boite.lock() {
            Ok(garde) => garde.clone(),
            Err(poison) => poison.into_inner().clone(),
        }
    }

    /// Le dernier code envoyé à une adresse, s'il existe.
    pub fn dernier_code(&self, dest: &str) -> Option<String> {
        self.messages()
            .into_iter()
            .rev()
            .find(|m| m.dest == dest)
            .map(|m| m.code)
    }
}

impl EnvoyeurCourriel for EnvoyeurMemoire {
    fn envoyer(&self, message: &MessageCourriel) -> Result<(), ErreurCourriel> {
        match self.boite.lock() {
            Ok(mut garde) => {
                garde.push(message.clone());
                Ok(())
            }
            Err(poison) => {
                poison.into_inner().push(message.clone());
                Ok(())
            }
        }
    }
}

/// Durée de validité d'un code à usage unique, en minutes.
pub const DUREE_CODE_MINUTES: i64 = 15;

/// Courriel de vérification de l'adresse électronique.
pub fn courriel_verification(dest: &str, expediteur: &str, code: &str) -> MessageCourriel {
    MessageCourriel {
        dest: dest.to_owned(),
        sujet: "AMAZONE COMP — vérification de votre adresse électronique".to_owned(),
        corps: format!(
            "Bonjour,\n\nVotre code de vérification est : {code}.\n\
             Il est valable {DUREE_CODE_MINUTES} minutes. Saisissez-le dans l'application pour activer votre compte.\n\n\
             Si vous n'êtes pas à l'origine de cette demande, ignorez ce message.\n\n\
             L'équipe AMAZONE COMP\n(expéditeur : {expediteur})"
        ),
        code: code.to_owned(),
    }
}

/// Courriel de réinitialisation du mot de passe.
pub fn courriel_reinitialisation(dest: &str, expediteur: &str, code: &str) -> MessageCourriel {
    MessageCourriel {
        dest: dest.to_owned(),
        sujet: "AMAZONE COMP — réinitialisation de votre mot de passe".to_owned(),
        corps: format!(
            "Bonjour,\n\nVotre code de réinitialisation est : {code}.\n\
             Il est valable {DUREE_CODE_MINUTES} minutes. Ne le communiquez jamais.\n\n\
             Si vous n'êtes pas à l'origine de cette demande, ignorez ce message.\n\n\
             L'équipe AMAZONE COMP\n(expéditeur : {expediteur})"
        ),
        code: code.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gabarits_contiennent_le_code_et_la_duree() {
        let m = courriel_verification("marie@exemple.bj", "ne-pas-repondre@amazone.bj", "123456");
        assert!(m.corps.contains("123456"));
        assert!(m.corps.contains("15 minutes"));
        assert_eq!(m.code, "123456");
        let r =
            courriel_reinitialisation("marie@exemple.bj", "ne-pas-repondre@amazone.bj", "654321");
        assert!(r.corps.contains("Ne le communiquez jamais"));
        assert_eq!(r.code, "654321");
    }

    #[test]
    fn transport_memoire_conserve_les_messages() {
        let env = EnvoyeurMemoire::nouveau();
        env.envoyer(&courriel_verification("a@b.bj", "x@y.bj", "111111"))
            .unwrap();
        env.envoyer(&courriel_verification("a@b.bj", "x@y.bj", "222222"))
            .unwrap();
        env.envoyer(&courriel_verification("c@d.bj", "x@y.bj", "333333"))
            .unwrap();
        assert_eq!(env.messages().len(), 3);
        assert_eq!(env.dernier_code("a@b.bj").as_deref(), Some("222222"));
        assert_eq!(env.dernier_code("c@d.bj").as_deref(), Some("333333"));
        assert_eq!(env.dernier_code("inconnu@b.bj"), None);
    }
}
