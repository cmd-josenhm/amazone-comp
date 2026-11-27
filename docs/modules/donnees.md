# Module Données — A2

## Lot 1 : base locale chiffrée SQLCipher

`ac-donnees` ouvre le fichier de l'entreprise avec SQLCipher compilé depuis ses sources et OpenSSL vendored. L'API est :

- `Base::ouvrir(chemin: &Path, cle: &CleBase) -> Resultat<Base>` ouvre ou crée le dossier chiffré ;
- `Base::en_memoire() -> Resultat<Base>` reste disponible pour les tests et les autres crates ;
- `Base::connexion()` fournit la connexion de lecture et `Base::transaction()` démarre une transaction d'écriture.

La clé de cette livraison est une clé brute de 256 bits. `CleBase::depuis_octets` prend possession d'un tableau de 32 octets provenant d'une source cryptographiquement sûre ; l'appelant efface toute copie qu'il conserve. `Base::ouvrir` encode les octets en hexadécimal et applique `PRAGMA key = "x'<64 chiffres hexadécimaux>'"`. Les chaînes temporaires sont effacées après usage. `CleBase` efface les octets qu'elle possède à sa destruction et ne fournit ni `Debug` ni `Display`. La génération aléatoire intégrée et les enveloppes de clé ne sont pas encore livrées : elles appartiennent au Lot 2.

La validation lit immédiatement `sqlite_master`. Une clé incorrecte renvoie `ErreurDonnees::MotDePasseIncorrect` avant d'activer le mode WAL ou d'appliquer d'autres réglages. La bibliothèque conserve les paramètres de chiffrement par défaut SQLCipher 4 ; les connexions configurent ensuite `foreign_keys = ON`, `journal_mode = WAL` et `synchronous = NORMAL`. Les erreurs de mise en place du chiffrement sont assainies et ne contiennent jamais la clé.

Un fichier correctement chiffré n'a pas l'en-tête SQLite en clair `SQLite format 3\0` : ses 16 premiers octets sont le sel SQLCipher. L'ouverture d'un fichier existant doit toujours recevoir la bonne clé. La comparaison avant/après dans le test de mauvaise clé garantit que cet échec ne modifie pas le fichier.

## Dépendances et sécurité de la compilation

La fonctionnalité `rusqlite/bundled-sqlcipher-vendored-openssl` compile SQLCipher et OpenSSL depuis les sources embarquées, sans dépendre de la SQLite système. `rusqlite 0.43.0` / `libsqlite3-sys 0.40.0` sélectionne `sqlcipher-src 420.0.0`, dont les métadonnées annoncent SQLCipher 4.20.0 et SQLite 3.54.1 ([dépendance épinglée](https://github.com/rusqlite/rusqlite/blob/43326c07c018323d9c0ed0b7cd68c35f5b9a3c28/libsqlite3-sys/Cargo.toml), [version du bundle](https://github.com/LucaCappelletti94/sqlcipher-src/blob/96648b56061e13e76b22d413307ca205b2d9476a/Cargo.toml)). RUSTSEC-2026-0459 concerne SQLite avant 3.54.0 ; le bundle 3.54.1 est donc postérieur à la version corrigée. L’avis signale comme vulnérable le bundle SQLCipher 419.0.6 / SQLite 3.53.4 utilisé par `libsqlite3-sys` 0.39.0–0.39.1, pas le bundle 420.0.0 choisi ici ([avis RustSec](https://github.com/RustSec/advisory-db/blob/c5279d0b0c9e8b70df666eb6d1edbabcfd1176bd/crates/libsqlite3-sys/RUSTSEC-2026-0459.md)).

## À livrer dans les lots suivants

- Lot 2 : génération aléatoire, enveloppes par mot de passe et clé de secours ;
- Lot 3 : fiche entreprise, utilisateurs et connexion ;
- Lots 4–6 : audit consultable, migrations robustes, sauvegardes locales et instantanés.
