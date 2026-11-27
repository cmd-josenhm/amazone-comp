# Politique de sécurité

## Signaler une vulnérabilité

**Ne pas ouvrir d'issue publique.** Utiliser l'onglet *Security → Report a vulnerability* du dépôt GitHub
(signalement privé), ou écrire à : **A COMPLETER** (adresse de sécurité du propriétaire).

Indiquer : version, étapes de reproduction, impact estimé. Réponse sous 7 jours, correctif visé sous 30 jours
pour une faille critique. Les auteurs de signalements sérieux sont remerciés dans le CHANGELOG (s'ils le souhaitent).

## Versions maintenues

Seule la dernière version publiée reçoit des correctifs de sécurité.

## Principes de sécurité du produit

Détaillés dans [docs/securite.md](docs/securite.md) :

- base locale **chiffrée** (SQLCipher, AES-256) ; clé protégée par le mot de passe (Argon2id) + clé de secours imprimable ;
- **sauvegardes chiffrées côté client** avant envoi (la plateforme ne peut pas lire les données comptables) ;
- **journal d'audit chaîné** (SHA-256) pour détecter toute altération ;
- licence signée **Ed25519** vérifiée hors ligne ; mises à jour **signées** (Tauri updater, minisign) ;
- aucune télémétrie, aucun appel réseau non nécessaire ;
- contrôle des rôles côté Rust (jamais seulement dans l'interface) ;
- dépendances auditées en CI (`cargo deny`, `npm audit`).
