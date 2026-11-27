# PROMPT A10 — Plateforme en ligne, compte et licence, anti-piratage, sauvegarde distante, mises à jour, installateur

> **Lis d'abord, dans cet ordre :** `AGENTS.md` (règles R1 à R14, **non négociables**), ce fichier, `docs/architecture.md` (§1, §6),
> `docs/securite.md`, `docs/adr/0004-compte-licence-anti-piratage.md`, `docs/adr/0007-mises-a-jour-signature.md`, `docs/adr/0008-plateforme-en-ligne.md`,
> **`docs/adr/0009-inscription-des-entreprises.md`**, **`docs/adr/0010-console-admin-et-assistance.md`**, `docs/contrats.md` (C11, C12).
> Tu ne connais pas le projet : **ne devine rien**. Question : issue GitHub `agent:a10` + `question`.

## 1. Le projet en une minute

AMAZONE COMP est un logiciel de comptabilité **gratuit et open source (AGPL-3.0)** pour les **PME du Bénin**, Windows 10+ 64 bits (PC de 4 Go),
**hors ligne d'abord**. Pile : **Tauri 2 + Rust** + **React/TypeScript** + **SQLite chiffrée**. Le propriétaire exige : **acceptation de ses conditions**,
**compte obligatoire** sur sa plateforme, **licence gratuite renouvelée tous les 6 mois** (nécessaire pour les mises à jour), **protection contre le piratage**,
possibilité de devenir **payant plus tard**. **Seul coût accepté : l'hébergement** (un VPS) — aucune API payante. Dix agents en parallèle ; le **chef de projet** valide chaque PR.

## 2. Ta mission

Tu construis **tout ce qui touche Internet et la distribution** : la **plateforme** (serveur), le **client** dans l'application (`ac-licence`),
la **glue `src-tauri`** (démarrage, état, commandes système de session/dossier, plugins), l'**écran Compte**, la **sauvegarde distante chiffrée**,
l'envoi des **avis**, les **mises à jour** automatiques/manuelles et l'**installateur** Windows.
Tu construis aussi l'**inscription des entreprises** (IFU, RCCM, identité — ADR 0009), la **console d'administration web** du propriétaire
(`plateforme/web-admin/`) et la messagerie **ASSISTANCE** entre les utilisateurs et le propriétaire (module logiciel + console — ADR 0010).

## 3. Ton périmètre

| Autorisé | Interdit |
|---|---|
| `plateforme/` (dont `plateforme/web-admin/`), `crates/ac-licence/`, `src-tauri/` (hors `commandes/<module>.rs` des autres agents), `ui/src/modules/compte/`, `ui/src/modules/assistance/`, `.github/workflows/release.yml`, `.github/workflows/plateforme.yml`, ta section de `commandes/mod.rs`, `docs/modules/plateforme.md`, ADR « Proposé » | Tout le reste. Le chiffrement local et les instantanés appartiennent à A2 (tu les utilises). Les écrans Paramètres / démarrage appartiennent à A3 (tu fournis les commandes). `ac_noyau::entreprise` appartient à A1 (tu l'utilises, tu ne le modifies pas : besoin → issue). |

## 4. Connaissances indispensables

- Connexion Internet **intermittente et chère** au Bénin : requêtes petites, reprise sur erreur, **jamais** de blocage de l'utilisateur hors ligne (sauf la toute première activation).
- Le code est **public** : la clé **privée** de signature des licences et des mises à jour ne doit **jamais** apparaître dans le dépôt, les journaux ou l'application.
  L'anti-piratage est **économique** (mises à jour fiscales et sauvegarde réservées aux licences valides, marque protégée) et non absolu : lis `docs/securite.md`.
- **Jamais de données comptables en clair** vers la plateforme : les sauvegardes sont chiffrées par A2 **avant** envoi.
- Données personnelles : loi béninoise n° 2017-20 (Code du numérique, APDP) → collecte minimale, CGU et politique de confidentialité (textes **A COMPLETER** par le propriétaire, prévois leur affichage versionné).

## 5. Ce qui existe déjà

- `crates/ac-licence` : `Licence`, `EtatLicence`, `etat(licence, date)` avec constantes 6 mois / 30 jours / 15 jours de grâce (testé). Pas de signature, pas de réseau.
- `src-tauri` : application minimale (fenêtre, `EtatApplication`, `toutes_les_migrations`, commande `systeme_infos`, macro `gestionnaire_commandes!`), `tauri.conf.json` (NSIS + MSI, français, CSP stricte), icônes **provisoires**.
- `.github/workflows/release.yml` (squelette tauri-action), `plateforme/README.md` (arborescence attendue).
- `ac_noyau::entreprise` : `FicheEntreprise` (tous les champs de l'ADR 0009), `valider(aujourdhui) -> RapportValidation { erreurs, avertissements }`,
  `Rccm`, `Telephone`, `courriel_valide`, listes `FormeJuridique`, `Departement` (12), `SecteurActivite`, `Regime` — **testés**. La plateforme dépend de
  `ac-noyau` par chemin (`ac-noyau = { path = "../crates/ac-noyau" }`) : **une seule validation** pour l'application et le serveur.
- Table locale `donnees_entreprise` (migration A2 `0001_socle.sql`) avec tous les champs de la fiche + `plateforme_id`, `statut_plateforme`.
- `ui/src/modules/assistance/module.tsx` : écran provisoire déjà enregistré dans le menu (chemin `/assistance`).

## 6. Travail à livrer (une PR par lot)

### Lot 1 — Plateforme : comptes, CGU, licences (≤ 8 jours) — **PRIORITÉ**
- `plateforme/` : crate binaire **hors workspace** de l'application (son propre `Cargo.toml` et `Cargo.lock`), axum + sqlx (PostgreSQL) + argon2 + ed25519-dalek + tower (limitation de débit).
- API `/v1` (JSON) : `POST comptes` (inscription : e-mail, mot de passe, nom), vérification e-mail (code à 6 chiffres), `POST sessions` (connexion → jeton d'accès court), mot de passe oublié,
  `GET cgu/courante`, `POST cgu/acceptations`, `POST entreprises` (**fiche complète `FicheEntreprise`**, validée par `valider()` côté serveur ; IFU **unique sur toute la plateforme** ; statut initial `EN_ATTENTE_VERIFICATION` ; justificatifs IFU/RCCM facultatifs, PDF/JPG/PNG ≤ 5 Mo), `PUT entreprises/{id}` (modification ; changement d'IFU = demande soumise à l'administrateur), `GET entreprises/{id}/statut`, `POST licences` (émission : IFU + empreinte du poste → jeton signé 6 mois), `POST licences/renouveler`, `POST licences/transferer` (limite paramétrable, ex. 2 par an), révocation (admin).
- Format du jeton : `base64url(charge_utile_json) + "." + base64url(signature_ed25519)`. Clé privée chargée depuis une variable d'environnement / fichier hors dépôt.
- `Dockerfile`, `docker-compose.yml` (plateforme + postgres + caddy HTTPS), `.env.exemple`, guide de déploiement sur un VPS (`docs/modules/plateforme.md`), sauvegarde de PostgreSQL.
- Workflow `.github/workflows/plateforme.yml` : tests avec un service PostgreSQL.

### Lot 2 — `ac-licence` côté application (≤ 4 jours)
- Vérification Ed25519 hors ligne (clé publique embarquée en constante), refus si signature, IFU ou empreinte ne correspondent pas.
- **Empreinte du poste** : SHA-256(sel applicatif + `MachineGuid` du registre Windows + numéro de série du volume système) — crate `winreg` (justifier), jamais d'identifiant brut envoyé.
- **Horloge reculée** : mémoriser la date maximale vue ; si la date système recule de plus de 48 h → exiger une vérification en ligne au prochain accès Internet (sans bloquer les données).
- Client HTTPS (`ureq` ou `reqwest` avec rustls — justifier la taille), délais courts, reprise, aucune donnée comptable.
- Stockage local de la licence, de l'état des CGU et de la file d'avis (migrations 8000+ via `ac-donnees`).

### Lot 3 — Glue `src-tauri` et premier lancement (≤ 5 jours)
- Démarrage : licence → choix/création du dossier (fonctions A2) → connexion → application ; mode consultation si `EtatLicence::Consultation` (les commandes d'écriture renvoient `LICENCE_EXPIREE`).
- Commandes pour A3 : `dossier_creer`, `dossier_ouvrir`, `session_ouvrir`, `session_fermer`, `session_verrouiller`, `compte_creer`, `compte_connecter`, `compte_accepter_cgu`, `compte_activer_licence`, `compte_etat_licence`, `compte_renouveler`.
  Écris d'abord la liste des signatures dans `docs/modules/plateforme.md` et **préviens A3 par issue**.
- Plugins : instance unique (`tauri-plugin-single-instance`), état de fenêtre, updater. Permissions (`capabilities/`) minimales.

### Lot 4 — Sauvegarde distante et avis (≤ 5 jours)
- Plateforme : `PUT/GET sauvegardes` (blobs opaques, quota par compte, conservation des 10 dernières, licence valide exigée), `POST avis`.
- Application : envoi automatique de l'instantané chiffré (A2 `instantane::exporter`) selon la fréquence choisie, reprise d'envoi interrompu, restauration sur un nouveau poste (compte + clé de secours) ; file d'avis hors ligne (`compte_envoyer_avis`, `compte_lister_avis`).
- Écran `ui/src/modules/compte/` : compte, licence (jours restants, renouveler), postes, sauvegarde en ligne (activer, dernière sauvegarde, restaurer), CGU acceptées.

### Lot 5 — Mises à jour (≤ 4 jours)
- Plugin updater Tauri : clé publique minisign dans `tauri.conf.json`, manifeste servi par la plateforme (`GET /v1/mises-a-jour/windows/{version}`) **uniquement** pour une licence valide.
- Modes automatique / manuel (commandes `maj_regler`, `maj_verifier`, `maj_installer`), jamais pendant une saisie, notes de version en français.

### Lot 6 — Installateur et publication (≤ 4 jours)
- NSIS en français, installation par utilisateur (sans droits administrateur), WebView2 installé si absent, désinstallation qui **conserve** les données (`%APPDATA%\AmazoneComp`) sauf choix explicite.
- `release.yml` : build sur tag, signature updater, sommes SHA-256, brouillon de release. Préparer la demande **SignPath Foundation** (signature Authenticode gratuite pour l'open source) : dossier de candidature dans `docs/modules/plateforme.md`.
- Mesures dans la PR : taille de l'installateur (< 15 Mo visé), RAM au démarrage sur un PC 4 Go.

### Lot 7 — Console d'administration : socle, entreprises et licences (≤ 8 jours)
- `plateforme/web-admin/` : application **React + TypeScript strict + Vite**, même design noir et blanc que le logiciel (reprendre les jetons de
  `ui/src/design/jetons.css` par copie documentée), servie par axum sur le sous-domaine `admin.<domaine>` (Caddy). Aucune bibliothèque UI lourde, aucun CDN.
- API `/admin/v1/…` séparée de `/v1` : comptes administrateurs **distincts** des comptes utilisateurs, rôles `PROPRIETAIRE` (tout) et `SUPPORT`
  (assistance + lecture), **2FA TOTP obligatoire** (crate `totp-rs`, à justifier), session ≤ 8 h, verrouillage après 5 échecs, création du premier
  propriétaire par une commande en ligne (`amazone-plateforme admin creer`), jamais par une page publique.
- **Journal d'audit administrateur** (table en ajout seul) : qui, quoi, quand, avant/après, pour **chaque** action.
- Écrans : **Tableau de bord** (inscriptions par semaine, licences actives / en grâce / expirées, conversations ouvertes, versions installées — chiffres agrégés) ;
  **Entreprises** (recherche par raison sociale / IFU / RCCM / commune, filtres statut / régime / département, fiche complète, justificatifs, utilisateurs du compte,
  postes, licences, historique) avec les actions **vérifier, rejeter (motif), suspendre (motif), réactiver, prolonger ou révoquer une licence, approuver un transfert
  de poste ou un changement d'IFU** ; **Utilisateurs** (comptes, dernière connexion, réinitialisation forcée, désactivation) ; **Avis** ; **Journal d'audit** ;
  **Paramètres** (vérification préalable oui/non, vérification renforcée oui/non, limite de transferts par an, quota de sauvegarde, durée de conservation).
- Effet du statut (ADR 0009 §4) : `SUSPENDUE` ou `REJETEE` → refus de renouvellement et de manifeste de mise à jour ; « vérification préalable » activée → licence délivrée seulement après `VERIFIEE`.
- E-mails à l'utilisateur sur changement de statut (SMTP de l'hébergeur, gabarits en français).

### Lot 8 — ASSISTANCE : messagerie utilisateurs ↔ propriétaire (≤ 7 jours)
- Plateforme : tables `conversations` (entreprise, utilisateur, catégorie, sujet, statut, assignée à, dates) et `messages` (auteur utilisateur ou administrateur,
  texte ≤ 5 000 caractères, pièces jointes PNG/JPG/PDF ≤ 5 Mo et 3 par message, lu le). Statuts `OUVERTE → EN_COURS → EN_ATTENTE_CLIENT → RESOLUE → FERMEE`
  (fermeture automatique 7 jours après `RESOLUE`, réouverture si l'utilisateur répond). Conservation 24 mois puis suppression. API du contrat **C12**.
- Logiciel (`ui/src/modules/assistance/` + `src-tauri/src/commandes/assistance.rs`) : liste des conversations, nouvelle demande (catégories : licence et compte,
  installation, utilisation, fiscalité, facturation MECeF, sauvegarde, signaler un problème, suggestion), fil de messages, pièces jointes, **diagnostic technique
  facultatif** (version, Windows, RAM, état de licence, 50 dernières lignes de journal **sans aucune donnée comptable**, joint seulement après case cochée à chaque fois).
  Hors ligne : les messages partent dans une **file d'envoi** locale (migration 8000+) ; récupération des réponses toutes les 2 minutes quand l'application est
  ouverte et en ligne (requête `depuis=<horodatage>` légère) ; **pastille** du nombre de réponses non lues dans le menu.
- Console : boîte de réception (filtres statut / catégorie / assignée à moi), fil en **temps réel** par **SSE** (`/admin/v1/assistance/flux`, pas de WebSocket ni de service tiers),
  fiche de l'entreprise à côté du fil, réponses types modifiables, notes internes invisibles pour l'utilisateur, assignation, changement de statut,
  notification par e-mail au propriétaire pour toute nouvelle conversation.
- Avertissement permanent des deux côtés : « Ne communiquez jamais votre mot de passe ni votre clé de secours. »

### Lot 9 — Annonces et publications (≤ 3 jours)
- **Annonces** : message diffusé dans le logiciel (titre, texte, niveau information / important, cible : toutes les entreprises, un régime, un département,
  dates de début et de fin) ; affichage dans l'application au démarrage et dans « Compte », `annonces_lister` ; marquage lu local.
- **Publications** : depuis la console, publier une version (notes de version en français, fichiers déjà signés par `release.yml` — la console **ne signe rien**),
  publier une nouvelle version des CGU / de la politique de confidentialité (les utilisateurs doivent l'accepter au prochain démarrage connecté).

## 7. Contrats

- **Tu fournis** : API `/v1` et `/admin/v1`, commandes `dossier_*`, `session_*`, `compte_*` (dont `compte_valider_entreprise(fiche) -> RapportValidation` — fonctionne hors ligne —, `compte_inscrire_entreprise(fiche)` et `compte_modifier_entreprise(fiche)`), `maj_*`, `assistance_*`, `annonces_lister` (contrat C12), mode consultation.
- **Tu consommes** : A1 (`ac_noyau::entreprise::FicheEntreprise` — contrat C11), A2 (`dossier`, `connexion`, `instantane`), A3 (écrans démarrage/paramètres appellent tes commandes ; l'assistant de démarrage affiche la fiche et les messages de `valider()`).

## 8. Tests exigés

- Jeton valide → `Valide` ; un octet modifié → invalide ; IFU ou empreinte différents → invalide ; expiré +10 j → `Grace` ; +16 j → `Consultation` (déjà testé pour les dates).
- Plateforme : inscription → vérification → connexion → entreprise → licence ; mot de passe faux 10 fois → limitation ; IFU déjà utilisé par un autre compte → refus ; transfert au-delà de la limite → refus.
- Sauvegarde : envoi puis restauration → octets identiques ; la plateforme ne stocke aucun contenu lisible (test : le blob n'est pas un SQLite lisible).
- Horloge reculée de 3 jours → vérification en ligne exigée.
- Inscription : fiche invalide → 422 avec la **même** liste `erreurs` que `valider()` ; IFU déjà inscrit → refus ; avertissements renvoyés sans bloquer.
- Statuts : entreprise `SUSPENDUE` → renouvellement refusé ; « vérification préalable » activée → pas de licence avant `VERIFIEE`.
- Console : connexion sans TOTP → refus ; 5 échecs → verrouillage ; rôle `SUPPORT` → suspension refusée ; chaque action → une ligne d'audit.
- Assistance : un utilisateur ne voit **que** les conversations de son entreprise (test d'accès croisé) ; message hors ligne → envoyé à la reconnexion,
  une seule fois (idempotence par identifiant client) ; pièce jointe de 6 Mo ou `.exe` → refus ; fermeture automatique à J+7 après `RESOLUE`.
- Mode consultation : une commande d'écriture renvoie `LICENCE_EXPIREE`, une lecture fonctionne.

## 9. Pièges à éviter

- Aucun secret dans le dépôt (le CI le vérifie) ; `.env.exemple` sans valeurs réelles.
- Ne jamais empêcher l'utilisateur d'**exporter ou sauvegarder** ses données, même licence expirée.
- Pas d'analytics, pas de télémétrie, pas de service tiers payant (ni chat externe, ni service d'e-mail payant, ni WebSocket hébergé).
- La console d'administration est la cible n° 1 des attaquants : jamais d'accès sans TOTP, jamais de mot de passe administrateur dans le dépôt ou les journaux,
  aucune action destructive sans confirmation **et** ligne d'audit.
- Ne réécris pas la validation de la fiche entreprise : utilise `FicheEntreprise::valider()` (une seule règle, deux usages).
- Messages d'erreur réseau compréhensibles : « Pas de connexion Internet. Vous pouvez continuer à travailler ; le renouvellement se fera plus tard. »

## 10. Définition de « terminé »

CI verte (y compris `plateforme.yml`, avec lint et tests de `plateforme/web-admin/`), couverture ≥ 80 % sur `ac-licence` et la plateforme, `docs/modules/plateforme.md` (API, déploiement, procédure de rotation des clés), CHANGELOG, PR fusionnée.

## 11. Pour démarrer

```powershell
git checkout main; git pull
git checkout -b a10/feat-plateforme-comptes
cargo test -p ac-licence
```
