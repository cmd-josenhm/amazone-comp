# ADR 0004 — Compte obligatoire, licence gratuite de 6 mois, anti-piratage

- Statut : **Accepté** (propriétaire, 2026-11-27)

## Contexte
Le logiciel est gratuit et open source, mais le propriétaire exige : acceptation des conditions, **compte obligatoire** sur sa plateforme, **licence gratuite renouvelée tous les 6 mois**, nécessaire pour les mises à jour, protection contre le piratage, possibilité de rendre payant plus tard.

## Décision
1. **Premier lancement** (Internet requis une fois) : CGU (version acceptée enregistrée localement et sur la plateforme) → création de compte / connexion → déclaration de l'entreprise (raison sociale, IFU, régime) → émission de la licence.
2. **Jeton de licence** = charge utile JSON (`ac_licence::Licence`) signée **Ed25519** par la plateforme. La clé **publique** est embarquée dans l'application ; la clé privée reste sur le serveur (jamais dans le dépôt).
3. **Liaison** : IFU + **empreinte du poste** (hash SHA-256 salé d'identifiants machine : `MachineGuid` Windows + nom du volume système). Changement de PC = transfert depuis le compte (limité, ex. 2 par an).
4. **Vérification hors ligne** à chaque démarrage (signature + dates). Renouvellement automatique dans les 30 derniers jours si Internet est disponible.
5. **Expiration** : 15 jours de grâce (bandeau), puis **mode consultation** : lecture, exports, sauvegarde locale autorisés ; saisie bloquée. **Jamais de blocage d'accès aux données.**
6. **Mises à jour** et **sauvegarde distante** exigent une licence valide, vérifiée par le serveur.
7. Champ `offre` (« GRATUITE ») prévu pour une future offre payante, sans changement d'architecture.

## Conséquences
- L'anti-piratage est **économique** et non absolu (voir `docs/securite.md`) : une copie recompilée sans contrôle ne reçoit ni mises à jour fiscales ni sauvegarde en ligne, et ne peut pas porter la marque.
- Aucune donnée comptable n'est envoyée pour la licence.
