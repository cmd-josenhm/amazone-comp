# PROMPT A10 — Plateforme en ligne, compte et licence, anti-piratage, sauvegarde distante, mises à jour, installateur

> **Lis d'abord, dans cet ordre :** `AGENTS.md` (règles R1 à R14, **non négociables**), ce fichier, `docs/architecture.md` (§1, §6),
> `docs/securite.md`, `docs/adr/0004-compte-licence-anti-piratage.md`, `docs/adr/0007-mises-a-jour-signature.md`, `docs/adr/0008-plateforme-en-ligne.md`.
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

## 3. Ton périmètre

| Autorisé | Interdit |
|---|---|
| `plateforme/`, `crates/ac-licence/`, `src-tauri/` (hors `commandes/<module>.rs` des autres agents), `ui/src/modules/compte/`, `.github/workflows/release.yml`, `.github/workflows/plateforme.yml`, ta section de `commandes/mod.rs`, `docs/modules/plateforme.md`, ADR « Proposé » | Tout le reste. Le chiffrement local et les instantanés appartiennent à A2 (tu les utilises). Les écrans Paramètres / démarrage appartiennent à A3 (tu fournis les commandes). |

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

## 6. Travail à livrer (une PR par lot)

### Lot 1 — Plateforme : comptes, CGU, licences (≤ 8 jours) — **PRIORITÉ**
- `plateforme/` : crate binaire **hors workspace** de l'application (son propre `Cargo.toml` et `Cargo.lock`), axum + sqlx (PostgreSQL) + argon2 + ed25519-dalek + tower (limitation de débit).
- API `/v1` (JSON) : `POST comptes` (inscription : e-mail, mot de passe, nom), vérification e-mail (code à 6 chiffres), `POST sessions` (connexion → jeton d'accès court), mot de passe oublié,
  `GET cgu/courante`, `POST cgu/acceptations`, `POST entreprises` (raison sociale, IFU unique par compte), `POST licences` (émission : IFU + empreinte du poste → jeton signé 6 mois), `POST licences/renouveler`, `POST licences/transferer` (limite paramétrable, ex. 2 par an), révocation (admin).
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

### Lot 7 — Administration minimale (≤ 3 jours)
Page web d'administration protégée (compte propriétaire, 2FA TOTP) : nombre de comptes, licences actives/expirées, révocation, lecture des avis, statistiques **agrégées** uniquement.

## 7. Contrats

- **Tu fournis** : API `/v1`, commandes `dossier_*`, `session_*`, `compte_*`, `maj_*`, mode consultation.
- **Tu consommes** : A2 (`dossier`, `connexion`, `instantane`), A3 (écrans démarrage/paramètres appellent tes commandes).

## 8. Tests exigés

- Jeton valide → `Valide` ; un octet modifié → invalide ; IFU ou empreinte différents → invalide ; expiré +10 j → `Grace` ; +16 j → `Consultation` (déjà testé pour les dates).
- Plateforme : inscription → vérification → connexion → entreprise → licence ; mot de passe faux 10 fois → limitation ; IFU déjà utilisé par un autre compte → refus ; transfert au-delà de la limite → refus.
- Sauvegarde : envoi puis restauration → octets identiques ; la plateforme ne stocke aucun contenu lisible (test : le blob n'est pas un SQLite lisible).
- Horloge reculée de 3 jours → vérification en ligne exigée.
- Mode consultation : une commande d'écriture renvoie `LICENCE_EXPIREE`, une lecture fonctionne.

## 9. Pièges à éviter

- Aucun secret dans le dépôt (le CI le vérifie) ; `.env.exemple` sans valeurs réelles.
- Ne jamais empêcher l'utilisateur d'**exporter ou sauvegarder** ses données, même licence expirée.
- Pas d'analytics, pas de télémétrie, pas de service tiers payant.
- Messages d'erreur réseau compréhensibles : « Pas de connexion Internet. Vous pouvez continuer à travailler ; le renouvellement se fera plus tard. »

## 10. Définition de « terminé »

CI verte (y compris `plateforme.yml`), couverture ≥ 80 % sur `ac-licence` et la plateforme, `docs/modules/plateforme.md` (API, déploiement, procédure de rotation des clés), CHANGELOG, PR fusionnée.

## 11. Pour démarrer

```powershell
git checkout main; git pull
git checkout -b a10/feat-plateforme-comptes
cargo test -p ac-licence
```
