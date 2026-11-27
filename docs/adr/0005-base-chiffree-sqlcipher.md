# ADR 0005 — Base locale chiffrée (SQLCipher) et gestion des clés

- Statut : **Accepté** (chef de projet, 2026-11-27) — implémentation : A2

## Décision
- `rusqlite` avec la fonctionnalité `bundled-sqlcipher-vendored-openssl` (remplace `bundled` actuel du squelette). A2 vérifie que la compilation fonctionne sur `windows-latest` en CI et mesure l'impact sur la taille de l'installateur.
- **Clé de base** : 256 bits aléatoires (`getrandom`), jamais stockée en clair.
- **Enveloppes** : pour chaque utilisateur, clé de base chiffrée (XChaCha20-Poly1305 ou AES-256-GCM) par une clé dérivée du mot de passe (**Argon2id**, paramètres calibrés pour ~0,5 s sur un PC 4 Go). Une enveloppe supplémentaire pour la **clé de secours** imprimée à la création du dossier.
- Fichier : `%APPDATA%\AmazoneComp\dossier.acdb` ; sauvegardes locales : `%APPDATA%\AmazoneComp\sauvegardes\` (rotation : 7 quotidiennes, 4 hebdomadaires, 12 mensuelles), chacune déjà chiffrée.
- Effacement des secrets en mémoire avec `zeroize`.

## Conséquences
Perte du mot de passe **et** de la clé de secours = perte des données (sauf sauvegarde distante restaurable avec la clé de secours). L'interface doit le dire clairement à la création.
