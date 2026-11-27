# Sécurité — modèle de menaces et mesures

## Ce que l'on protège

1. **Les données comptables et la paie** de l'entreprise (confidentialité, intégrité).
2. **L'intégrité de la comptabilité** (écritures non falsifiables a posteriori : exigence légale de fiabilité).
3. **La licence** et la distribution officielle (anti-piratage, mises à jour authentiques).
4. **La plateforme** (comptes utilisateurs, sauvegardes).

## Menaces et réponses

| Menace | Réponse | Responsable |
|---|---|---|
| Vol ou perte du PC | Base SQLCipher AES-256 ; clé enveloppée par Argon2id(mot de passe) ; verrouillage après inactivité | A2 |
| Oubli du mot de passe administrateur | Clé de secours (24 mots / code imprimable) générée à la création, jamais envoyée en ligne | A2, A3 |
| Employé qui modifie une écriture passée | Écritures immuables (déclencheurs SQL), contre-passation, journal d'audit chaîné, contrôle de chaîne dans l'interface | A1, A2 |
| Employé qui consulte la paie | Rôle `Paie` séparé, contrôle côté Rust (`Session::exiger`) | A2, A7 |
| Rançongiciel / disque mort | Sauvegardes locales automatiques rotatives + sauvegarde distante chiffrée côté client | A2, A10 |
| Fuite sur la plateforme | Sauvegardes chiffrées **avant** envoi (la plateforme stocke des blobs opaques) ; mots de passe Argon2id ; HTTPS obligatoire | A10 |
| Fausse mise à jour (malware) | Paquets signés (minisign / Tauri updater) ; clé publique embarquée ; HTTPS ; à terme signature Authenticode | A10 |
| Licence piratée / partagée | Jeton Ed25519 signé par le serveur, lié à l'IFU et à l'empreinte du poste ; révocation côté serveur ; mises à jour et sauvegarde distante réservées aux licences valides | A10 |
| Injection via fichier importé (CSV/Excel) | Analyse stricte, taille maximale, aucune formule exécutée, aperçu avant import, requêtes SQL paramétrées uniquement | A6, A9 |
| XSS dans l'interface | CSP stricte (`tauri.conf.json`), pas de `dangerouslySetInnerHTML`, permissions Tauri minimales | A3, A10 |
| Dépendance compromise | `cargo deny`, `npm audit`, Dependabot, nouvelles dépendances justifiées (R12), `package-lock.json` et `Cargo.lock` commités | Chef de projet |

## Anti-piratage : ce qui est réaliste avec un code ouvert

Le code étant public, une personne compétente peut toujours retirer la vérification de licence d'une copie
qu'elle compile elle-même. On ne cherche donc pas l'impossible ; on rend la version officielle **plus
intéressante** et la contrefaçon **inutile ou visible** :

- seule la version officielle (signée) reçoit les **mises à jour fiscales** (indispensables chaque année avec la loi de finances) ;
- la **sauvegarde en ligne** et la restauration exigent une licence valide vérifiée par le serveur ;
- la marque « AMAZONE COMP » et le logo ne sont pas couverts par l'AGPL : une copie modifiée ne peut pas utiliser le nom ;
- l'AGPL oblige toute personne qui redistribue une version modifiée à publier son code source.

## Données personnelles

Conformité visée : loi béninoise n° 2017-20 portant Code du numérique (protection des données personnelles, APDP).
Collecte minimale sur la plateforme : identité du compte, raison sociale, IFU, empreinte du poste. Aucune
donnée comptable en clair. Politique de confidentialité et CGU : **A COMPLETER** (à faire rédiger par un juriste).
