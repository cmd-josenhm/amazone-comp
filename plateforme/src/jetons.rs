//! Jetons opaques, codes à usage unique et jetons de licence signés Ed25519.
//!
//! Format du jeton de licence (ADR 0004) :
//! `base64url(charge_utile_json) + "." + base64url(signature_ed25519)`.
//! La signature porte sur les **octets JSON de la charge utile** (avant encodage
//! base64url). La charge utile est le type [`ac_licence::Licence`].

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use ed25519_dalek::{Signer, SigningKey, VerifyingKey};
use rand::{thread_rng, Rng, RngCore};
use sha2::{Digest, Sha256};
use thiserror::Error;

use ac_licence::Licence;

use crate::config::hex_depuis_octets;

/// Erreur de manipulation d'un jeton ou d'un code.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ErreurJeton {
    /// Le jeton ne contient pas les deux parties attendues.
    #[error("Jeton mal formé.")]
    Format,
    /// La signature ne correspond pas à la charge utile.
    #[error("Signature du jeton invalide.")]
    Signature,
    /// La charge utile n'est pas un JSON lisible en licence.
    #[error("Contenu du jeton illisible.")]
    ChargeUtile,
}

/// Signataire Ed25519 des jetons de licence.
pub struct SignataireLicences {
    cle: SigningKey,
}

impl SignataireLicences {
    /// Crée un signataire à partir de la clé privée (32 octets).
    pub fn nouveau(cle_privee: [u8; 32]) -> Self {
        SignataireLicences {
            cle: SigningKey::from_bytes(&cle_privee),
        }
    }

    /// Clé publique correspondante, en hexadécimal (à embarquer dans l'application).
    pub fn cle_publique_hex(&self) -> String {
        hex_depuis_octets(&self.cle.verifying_key().to_bytes())
    }

    /// Signe une licence et renvoie le jeton `charge_utile.signature`.
    pub fn signer_licence(&self, licence: &Licence) -> Result<String, ErreurJeton> {
        let charge = serde_json::to_string(licence).map_err(|_| ErreurJeton::ChargeUtile)?;
        let signature = self.cle.sign(charge.as_bytes());
        Ok(format!(
            "{}.{}",
            URL_SAFE_NO_PAD.encode(charge.as_bytes()),
            URL_SAFE_NO_PAD.encode(signature.to_bytes())
        ))
    }
}

/// Vérifie un jeton de licence avec une clé publique hexadécimale, puis renvoie la charge utile.
///
/// C'est exactement la vérification **hors ligne** effectuée par l'application (lot 2) :
/// un octet modifié, une signature étrangère ou une charge utile illisible sont refusés.
pub fn verifier_jeton(jeton: &str, cle_publique_hex: &str) -> Result<Licence, ErreurJeton> {
    let mut parties = jeton.split('.');
    let (Some(charge_b64), Some(signature_b64), None) =
        (parties.next(), parties.next(), parties.next())
    else {
        return Err(ErreurJeton::Format);
    };
    let charge = URL_SAFE_NO_PAD
        .decode(charge_b64)
        .map_err(|_| ErreurJeton::Format)?;
    let signature_octets = URL_SAFE_NO_PAD
        .decode(signature_b64)
        .map_err(|_| ErreurJeton::Format)?;
    let signature_tableau: [u8; 64] = signature_octets
        .as_slice()
        .try_into()
        .map_err(|_| ErreurJeton::Format)?;
    let cle_octets =
        crate::config::octets_depuis_hex(cle_publique_hex).ok_or(ErreurJeton::Signature)?;
    let cle_publique =
        VerifyingKey::from_bytes(&cle_octets).map_err(|_| ErreurJeton::Signature)?;
    let signature = ed25519_dalek::Signature::from_bytes(&signature_tableau);
    cle_publique
        .verify_strict(&charge, &signature)
        .map_err(|_| ErreurJeton::Signature)?;
    serde_json::from_slice(&charge).map_err(|_| ErreurJeton::ChargeUtile)
}

/// Nombre de chiffres d'un code de vérification.
pub const LONGUEUR_CODE: usize = 6;

/// Génère un code à 6 chiffres (100 000 à 999 999).
pub fn code_six_chiffres() -> String {
    let mut rng = thread_rng();
    format!("{:06}", rng.gen_range(100_000u32..=999_999u32))
}

/// Génère un jeton opaque (32 octets aléatoires, encodés en base64url) pour les sessions.
pub fn jeton_opaque() -> String {
    let mut octets = [0u8; 32];
    thread_rng().fill_bytes(&mut octets);
    URL_SAFE_NO_PAD.encode(octets)
}

/// Empreinte SHA-256 (base64url) : utilisée pour ne stocker que des jetons et codes hachés.
pub fn empreinte_sha256(texte: &str) -> String {
    let mut hash = Sha256::new();
    hash.update(texte.as_bytes());
    URL_SAFE_NO_PAD.encode(hash.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ac_noyau::Date;

    fn licence_exemple() -> Licence {
        Licence {
            format: 1,
            compte_id: "42".into(),
            ifu: "3201912345678".into(),
            empreinte_poste: "empreinte-abc".into(),
            emise_le: Date::from_ymd_opt(2026, 1, 1).unwrap(),
            expire_le: Date::from_ymd_opt(2026, 7, 1).unwrap(),
            offre: "GRATUITE".into(),
        }
    }

    #[test]
    fn jeton_valide_puis_un_octet_modifie() {
        let signataire = SignataireLicences::nouveau([9u8; 32]);
        let pub_hex = signataire.cle_publique_hex();
        let jeton = signataire.signer_licence(&licence_exemple()).unwrap();

        let relu = verifier_jeton(&jeton, &pub_hex).unwrap();
        assert_eq!(relu, licence_exemple());

        // Un octet modifié dans la charge utile → signature invalide.
        let mut parties: Vec<String> = jeton.split('.').map(str::to_owned).collect();
        let mut charge = URL_SAFE_NO_PAD.decode(&parties[0]).unwrap();
        charge[5] ^= 0x01;
        parties[0] = URL_SAFE_NO_PAD.encode(charge);
        let altere = parties.join(".");
        assert!(matches!(
            verifier_jeton(&altere, &pub_hex),
            Err(ErreurJeton::Signature)
        ));
    }

    #[test]
    fn jeton_avec_cle_publique_etrangere_refuse() {
        let signataire = SignataireLicences::nouveau([1u8; 32]);
        let autre = SignataireLicences::nouveau([2u8; 32]);
        let jeton = signataire.signer_licence(&licence_exemple()).unwrap();
        assert_eq!(
            verifier_jeton(&jeton, &autre.cle_publique_hex()),
            Err(ErreurJeton::Signature)
        );
    }

    #[test]
    fn jeton_mal_forme() {
        let signataire = SignataireLicences::nouveau([3u8; 32]);
        let pub_hex = signataire.cle_publique_hex();
        assert_eq!(verifier_jeton("sans-point", &pub_hex), Err(ErreurJeton::Format));
        assert_eq!(verifier_jeton("a.b.c", &pub_hex), Err(ErreurJeton::Format));
        assert_eq!(verifier_jeton("", &pub_hex), Err(ErreurJeton::Format));
    }

    #[test]
    fn codes_et_jetons_opaques() {
        for _ in 0..20 {
            let code = code_six_chiffres();
            assert_eq!(code.len(), LONGUEUR_CODE);
            assert!(code.chars().all(|c| c.is_ascii_digit()));
            assert!((100_000..=999_999).contains(&code.parse::<u32>().unwrap()));
        }
        let a = jeton_opaque();
        let b = jeton_opaque();
        assert_ne!(a, b);
        assert_eq!(a.len(), 43); // 32 octets en base64url sans remplissage
    }

    #[test]
    fn empreintes_stables_et_distinctes() {
        assert_eq!(empreinte_sha256("abc"), empreinte_sha256("abc"));
        assert_ne!(empreinte_sha256("abc"), empreinte_sha256("abd"));
        assert_eq!(empreinte_sha256("").len(), 43);
    }
}
