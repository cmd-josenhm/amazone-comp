# ADR 0010 — Console d'administration web et ASSISTANCE (messagerie avec les utilisateurs)

- Statut : **Accepté** (propriétaire, 2026-11-27) — implémentation : A10

## Contexte
Le propriétaire veut une **session administrateur** sur la partie web pour **gérer le logiciel**, **voir les utilisateurs**
et **discuter avec eux** via un espace **ASSISTANCE**. Contraintes : coût = hébergement seul, aucun service tiers payant.

## Décision

### 1. Console d'administration (web)
- Application web **React + TypeScript + Vite** dans `plateforme/web-admin/`, même design noir et blanc que le logiciel
  (copie des jetons CSS), servie en fichiers statiques par le serveur axum sous `https://admin.<domaine>` (sous-domaine distinct).
- Accès : comptes administrateurs **séparés** des comptes clients, rôle `PROPRIETAIRE` (tous droits) ou `SUPPORT` (assistance et lecture seule ; prévu pour une future équipe).
  **Double authentification TOTP obligatoire**, session de 8 h maximum, verrouillage après 5 échecs, journal d'audit de **toutes** les actions administrateur.
- Écrans :
  1. **Tableau de bord** : entreprises inscrites (total, nouvelles sur 7/30 jours), licences actives / à renouveler / expirées, versions du logiciel utilisées, conversations ouvertes, avis récents. Statistiques **agrégées**.
  2. **Entreprises et utilisateurs** : liste, recherche (raison sociale, IFU, RCCM, téléphone, courriel), filtres (statut, régime, département, secteur), fiche complète (inscription, justificatifs, licences, postes, historique), actions : **vérifier / rejeter / suspendre / réactiver** (motif obligatoire), **prolonger ou révoquer une licence**, autoriser un transfert de poste, valider un changement d'IFU, renvoyer l'e-mail de vérification.
  3. **ASSISTANCE** : boîte de réception des conversations (voir §2), réponse, statut, réponses types, fiche de l'entreprise affichée à côté.
  4. **Avis et critiques** : lecture, marquer comme traité, répondre (ouvre une conversation d'assistance).
  5. **Annonces** : message diffusé dans le logiciel à tous ou à un filtre (régime, version) — ex. « Nouvelle version », « Rappel : TVA avant le 10 ».
  6. **Publication** : versions du logiciel (notes, manifeste de mise à jour, retrait d'une version défectueuse), versions des CGU et de la politique de confidentialité.
  7. **Paramètres** : vérification préalable des inscriptions oui/non, justificatifs obligatoires oui/non, limites (transferts, quota de sauvegarde), administrateurs.
  8. **Journal d'audit administrateur**.

### 2. ASSISTANCE (conversations)
- **Côté logiciel** : module « Assistance » (`ui/src/modules/assistance/`) : nouvelle demande (catégorie : utilisation, problème technique,
  question comptable ou fiscale, compte et licence, suggestion), fil de messages, pièces jointes (PNG/JPG/PDF ≤ 5 Mo, 3 par message),
  option **« joindre le diagnostic technique »** (version, Windows, mémoire, journal d'erreurs **sans aucune donnée comptable**) avec accord explicite à chaque fois.
  Hors ligne : messages en file d'attente, envoyés au retour d'Internet. Nouvelles réponses récupérées toutes les **2 minutes** quand le logiciel est ouvert et connecté → badge dans le menu + notification.
- **Côté console** : réception en temps réel par **SSE** (Server-Sent Events, natif axum, gratuit), notification par e-mail au propriétaire pour un nouveau message (SMTP de l'hébergeur), statuts `OUVERTE` → `EN_COURS` → `EN_ATTENTE_CLIENT` → `RESOLUE` → `FERMEE` (fermeture automatique 7 jours après `RESOLUE` sans réponse), attribution à un administrateur, réponses types.
- Une conversation est rattachée à **une entreprise** et à l'utilisateur qui l'a ouverte. L'utilisateur voit le nom affiché de l'administrateur (« Équipe AMAZONE COMP » par défaut).
- Sécurité : messages et pièces jointes stockés sur le VPS, accès limité au compte concerné et aux administrateurs ; avertissement permanent « Ne communiquez jamais votre mot de passe ni votre clé de secours ». Conservation : conversations fermées supprimées après 24 mois (réglable).

### 3. Pas de service tiers
Pas de chat SaaS, pas de Firebase, pas de push payant : HTTP + SSE + polling, e-mail SMTP.

## Conséquences
Charge supplémentaire pour A10 (lots 7 à 9 de `PROMPT_A10.md`). La console est une surface d'attaque sensible : 2FA, sous-domaine séparé,
limitation de débit renforcée, en-têtes de sécurité (CSP, HSTS), sauvegarde quotidienne chiffrée de PostgreSQL et des pièces jointes.
