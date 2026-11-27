//! Conteneur opaque de la clé brute SQLCipher.

use zeroize::{Zeroize, Zeroizing};

/// Clé brute de la base locale (256 bits).
///
/// Les octets sont effacés de la mémoire à la destruction. Ce type n'implémente volontairement ni
/// [`Debug`] ni [`std::fmt::Display`] afin qu'une clé ne puisse pas être affichée par accident.
/// La génération aléatoire et les enveloppes sont livrées dans le lot suivant.
pub struct CleBase(Zeroizing<[u8; 32]>);

impl CleBase {
    /// Prend possession de 32 octets fournis par un générateur cryptographiquement sûr.
    ///
    /// La copie locale est effacée avant le retour. L'appelant doit aussi effacer toute autre copie
    /// qu'il aurait conservée ; la génération intégrée sera fournie dans le Lot 2.
    pub fn depuis_octets(mut octets: [u8; 32]) -> Self {
        let cle = Self(Zeroizing::new(octets));
        octets.zeroize();
        cle
    }

    /// Accès réservé à la couche de données pour configurer SQLCipher.
    pub(crate) fn octets(&self) -> &[u8; 32] {
        &self.0
    }
}
