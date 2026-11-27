# Module « Plateforme en ligne » — propriétaire : A10

Service unique **Rust (axum) + PostgreSQL** déployé en conteneurs sur un VPS (ADR 0004, 0008,
0009). Il gère les comptes, l'inscription des entreprises, les CGU versionnées et les licences
signées Ed25519. Les lots suivants ajouteront les sauvegardes distantes, les avis, les mises à
jour, la console d'administration (`web-admin/`) et la messagerie ASSISTANCE (ADR 0010).

Code : `plateforme/` (crate binaire **séparé** du workspace, son propre `Cargo.toml` et
`Cargo.lock`). Il dépend de `ac-noyau` et `ac-licence` **par chemin** :

- la fiche entreprise est validée par `ac_noyau::entreprise::FicheEntreprise::valider()` —
  **la même règle que l'application**, jamais réécrite ;
- la charge utile du jeton de licence est le type `ac_licence::Licence` — un seul format pour
  l'émetteur (plateforme) et le vérificateur (application, lot 2).

## 1. API `/v1` (JSON, UTF-8)

Sauf mention contraire, les réponses d'erreur ont la forme
`{ "code": "STABLE", "message": "phrase compréhensible en français" }`.
Les routes notées **(auth)** exigent l'en-tête `Authorization: Bearer <jeton>` (jeton de session).
Toutes les adresses électroniques sont normalisées (minuscules, espaces retirés).

### Comptes et sessions

| Méthode et chemin | Corps | Réponses |
|---|---|---|
| `POST /v1/comptes` | `{courriel, mot_de_passe, nom}` | **201** `{compte_id, message}` + code à 6 chiffres envoyé ; **400** mot de passe < 10 caractères ou courriel invalide ; **409** `COMPTE_EXISTANT` |
| `POST /v1/comptes/verifications` | `{courriel, code}` | **200** vérifié ; **400** code incorrect / expiré / épuisé (5 tentatives) |
| `POST /v1/comptes/codes` | `{courriel}` | **202** toujours (renvoi du code ; aucune indication sur l'existence du compte) |
| `POST /v1/sessions` | `{courriel, mot_de_passe}` | **200** `{jeton, expire_le, compte: {id, courriel, nom}}` (jeton d'accès **24 h**) ; **401** identifiants incorrects ; **403** `COURRIEL_NON_VERIFIE` ; **429** `TROP_DE_TENTATIVES` (10 mots de passe faux → verrouillage 15 min) |
| `DELETE /v1/sessions/courante` **(auth)** | — | **204** |
| `POST /v1/mots-de-passe/oublie` | `{courriel}` | **202** toujours (code de réinitialisation envoyé si le compte existe) |
| `POST /v1/mots-de-passe/reinitialiser` | `{courriel, code, nouveau_mot_de_passe}` | **200** (toutes les sessions du compte sont fermées) ; **400** code invalide |

Les mots de passe sont hachés en **Argon2id** (sel aléatoire) ; les codes et jetons ne sont
stockés que sous forme d'**empreinte SHA-256**.

### Conditions générales (CGU)

| Méthode et chemin | Corps | Réponses |
|---|---|---|
| `GET /v1/cgu/courante` | — | **200** `{version, titre, texte, publiee_le}` ; **404** aucune version publiée |
| `POST /v1/cgu/acceptations` **(auth)** | `{version}` | **201** acceptée (idempotent) ; **409** `CGU_NON_A_JOUR` si la version n'est plus la courante |

La version courante est publiée dans la table `cgu_versions` (lot 1 : brouillon « A COMPLETER »
inséré par la migration ; la console publiera les versions suivantes, lot 7). L'acceptation de
chaque version est enregistrée par compte (`cgu_acceptations`), conformément à la loi béninoise
n° 2017-20. L'inscription d'une entreprise exige l'acceptation de la version courante.

### Entreprises (ADR 0009)

| Méthode et chemin | Corps | Réponses |
|---|---|---|
| `POST /v1/entreprises` **(auth)** | `{fiche}` — fiche complète `FicheEntreprise` (C11) | **201** `{entreprise_id, ifu, statut: "EN_ATTENTE_VERIFICATION", avertissements}` ; **422** rapport de `valider()` **exact** (`{erreurs, avertissements}`) ; **409** `IFU_DEJA_INSCRIT` ; **403** `CGU_NON_ACCEPTEES` |
| `PUT /v1/entreprises/{id}` **(auth)** | `{fiche}` | **200** `{statut, avertissements, demande_changement_ifu}` ; **422** fiche invalide ; **403** / **404** |
| `GET /v1/entreprises/{id}/statut` **(auth)** | — | **200** `{entreprise_id, ifu, statut, motif, statut_change_le, demandes_ifu_en_attente}` |
| `POST /v1/entreprises/{id}/justificatifs` **(auth)** | `{categorie, nom_fichier, mime, contenu_base64}` | **201** ; **413** > 5 Mo ; **415** hors PDF/JPG/PNG ; **400** catégorie ou base64 invalide |

- La fiche est stockée **en entier** (JSONB, tous les champs C11) : elle sera réutilisée par la
  comptabilité (en-têtes, déclarations).
- IFU normalisé (13 chiffres, espaces retirés), **unique sur toute la plateforme**.
- Statuts : `EN_ATTENTE_VERIFICATION` → `VERIFIEE` | `REJETEE` (motif) | `SUSPENDUE` (motif) —
  les changements de statut viennent de la console (lot 7).
- **Changement d'IFU** : `PUT` avec un IFU différent enregistre une `demandes_changement_ifu`
  (validation par l'administrateur, lot 7) et conserve l'IFU actuel ; les autres champs sont
  modifiés immédiatement.

### Licences (ADR 0004)

| Méthode et chemin | Corps | Réponses |
|---|---|---|
| `POST /v1/licences` **(auth)** | `{ifu, empreinte_poste}` | **201** `{jeton, emise_le, expire_le, offre}` ; **409** licence déjà active (même poste → renouvellement ; autre poste → transfert) |
| `POST /v1/licences/renouveler` **(auth)** | `{ifu, empreinte_poste}` | **200** nouveau jeton ; **404** aucune licence à renouveler |
| `POST /v1/licences/transferer` **(auth)** | `{ifu, ancienne_empreinte, nouvelle_empreinte}` | **200** `{jeton, …, transferts_restants}` ; **403** `LIMITE_TRANSFERTS` (2 par an par défaut, paramétrable) |

- `empreinte_poste` : hash SHA-256 du poste calculé par l'application (hexadécimal ou base64url,
  200 caractères au plus) — **jamais un identifiant matériel en clair**.
- Entreprise `SUSPENDUE` ou `REJETEE` → émission et renouvellement **refusés**.
- Réglage « vérification préalable » (table `parametres`) activé → licence délivrée seulement
  après passage en `VERIFIEE`. Par défaut, la licence est délivrée immédiatement.
- Durée : `ac_licence::DUREE_MOIS` (6 mois). Offre : `GRATUITE` (champ prévu pour une future
  offre payante).

### Format du jeton de licence

```
jeton = base64url(charge_utile_json) + "." + base64url(signature_ed25519)
```

- **charge_utile_json** : sérialisation JSON de `ac_licence::Licence`
  (`{format, compte_id, ifu, empreinte_poste, emise_le, expire_le, offre}`, dates en `AAAA-MM-JJ`) ;
- **signature** : Ed25519 par la clé privée du serveur, portant sur les **octets JSON de la
  charge utile** (avant encodage base64url, sans remplissage `=`).
- Vérification côté application (hors ligne) : décoder la première partie, vérifier la signature
  avec la clé publique embarquée, puis relire la charge utile. Un octet modifié, une signature
  étrangère ou un IFU / une empreinte différents → licence invalide.

## 2. Limitation de débit et sécurité

- Couche `tower` (module `limitation.rs`) : seau par adresse IP (`X-Forwarded-For` posé par
  Caddy) et par classe de route — inscription 20/h, connexion 60/5 min, licences 60/5 min,
  général 300/5 min. Réponse **429** `{code: "TROP_DE_REQUETES"}`.
- Verrouillage d'un compte après **10** mots de passe faux (15 min), en mémoire.
- Aucun secret dans le dépôt (règle R7) : clé privée en variable d'environnement ou fichier hors
  dépôt ; `.env.exemple` sans valeur réelle ; `plateforme/secrets/` et `.env` ignorés par Git.
- Pas de télémétrie, pas d'analytics, pas de service tiers payant (règle R6/R12).
- Les journaux du serveur ne contiennent ni mot de passe, ni clé privée, ni donnée comptable.

## 3. Déploiement sur un VPS

Prérequis : un VPS (2 vCPU / 2 Go suffisent), Docker + plugin compose, un domaine DNS pointant
vers le VPS (ex. `amazone.votre-domaine.bj`).

1. **Récupérer le code** : `git clone https://github.com/cmd-josenhm/amazone-comp` puis
   `cd amazone-comp/plateforme`.
2. **Configurer** : `cp .env.exemple .env` puis remplir :
   - `BDD_MOT_DE_PASSE` : mot de passe PostgreSQL fort ;
   - `PLATEFORME_CLE_PRIVEE_LICENCE` : `python3 -c "import secrets; print(secrets.token_hex(32))"`
     (ou `PLATEFORME_FICHIER_CLE_PRIVEE` vers un fichier hors dépôt, ex. `/etc/amazone/cle_privee.hex`) ;
   - `PLATEFORME_COURRIEL_EXPEDITEUR` : adresse d'expéditeur.
3. **Adapter `Caddyfile`** : domaine réel + adresse e-mail (certificat Let's Encrypt gratuit).
4. **Démarrer** : `docker compose up -d --build`. Vérifier :
   `curl https://amazone.votre-domaine.bj/v1/cgu/courante`.
5. **Noter la clé publique** affichée dans les journaux (`docker compose logs plateforme`) :
   c'est celle à embarquer dans l'application (lot 2).
6. **Sauvegarde PostgreSQL** (obligatoire) — exemple cron quotidien chiffré hors site :

   ```bash
   # /etc/cron.d/amazone-sauvegarde — 3 h 30 chaque matin
   30 3 * * * root docker compose -f /opt/amazone-comp/plateforme/docker-compose.yml exec -T bdd \
     pg_dump -U amazone amazone | gzip | age -r /etc/amazone/cle-sauvegarde.pub \
     > /sauvegardes/amazone-$(date +\%F).sql.gz.age
   ```

   Conserver au moins 14 jours de sauvegardes **hors du VPS**. Restauration :
   `gunzip -c sauvegarde.sql.gz | docker compose exec -T bdd psql -U amazone amazone`.

Courriels : le transport par défaut (**journal**, `docker compose logs plateforme`) affiche les
codes de vérification dans les journaux — c'est le mode **développement**. Pour la production,
brancher le SMTP de l'hébergeur sur le trait `EnvoyeurCourriel` (module `courriel.rs`) : le choix
dépend de l'hébergeur retenu et sera documenté ici au déploiement réel (ADR 0008).

## 4. Rotation des clés de signature

La clé privée signe les licences ; sa rotation n'invalidise **pas** les licences déjà émises
(l'application vérifie avec la clé publique embarquée dans sa version) :

1. Générer une nouvelle clé : `python3 -c "import secrets; print(secrets.token_hex(32))"`.
2. Mettre à jour `PLATEFORME_CLE_PRIVEE_LICENCE` (ou le fichier désigné) et
   `docker compose up -d plateforme` ; noter la nouvelle clé publique affichée au démarrage.
3. Publier une **nouvelle version de l'application** embarquant la nouvelle clé publique (lot 5/6).
   Les anciennes versions continuent de vérifier leurs licences avec l'ancienne clé.
4. Conserver l'ancienne clé publique dans les versions d'application qui l'utilisent ; l'ancienne
   clé **privée** peut être détruite une fois la période de transition terminée (après 6 mois,
   durée de validité maximale d'une licence).

En cas de **compromission** de la clé privée : la remplacer immédiatement (étapes 1–3), puis
demander au propriétaire de révoquer les licences suspectes (console, lot 7).

## 5. Tests et intégration continue

- Tests d'intégration (`plateforme/tests/parcours.rs`) : parcours complet inscription →
  vérification → connexion → entreprise → licence ; 10 mots de passe faux → limitation ; IFU
  déjà inscrit → refus ; transfert au-delà de la limite → refus ; fiche invalide → 422 avec la
  même liste `erreurs` que `valider()` ; statuts et vérification préalable ; mot de passe oublié ;
  accès croisé entre comptes refusé ; justificatifs contrôlés.
- Exécution locale : un PostgreSQL est requis (`DATABASE_URL` pointant vers une base de test) ;
  chaque test reçoit sa propre base :
  `cd plateforme && cargo test`. Format et lint : `cargo fmt --check`,
  `cargo clippy --all-targets -- -D warnings`.
- Intégration continue : `.github/workflows/plateforme.yml` (service PostgreSQL, format, clippy,
  tests, couverture ≥ 80 %, construction de l'image Docker).

## 6. Dépendances (justification, règle R12)

| Crate | Usage | Licence | Justification |
|---|---|---|---|
| `axum` | serveur HTTP | MIT | imposé par l'ADR 0008 ; équipe Tokio, très maintenu |
| `sqlx` + `tokio` | PostgreSQL, migrations | Apache-2.0 / MIT | imposés par l'ADR 0008 |
| `argon2` | hachage des mots de passe (Argon2id) | Apache-2.0 / MIT | référence OWASP, RustCrypto |
| `ed25519-dalek` | signature des jetons de licence | BSD-3-Clause | imposé par l'ADR 0004 |
| `tower` | limitation de débit (couche) | MIT | imposé par l'issue n° 12 ; socle de l'écosystème axum |
| `base64` | format du jeton (base64url) | MIT / Apache-2.0 | minuscule, sans dépendance |
| `rand` | codes, jetons, sel Argon2id | MIT / Apache-2.0 | génération aléatoire cryptographique |
| `sha2` | empreintes des jetons et codes stockés | Apache-2.0 / MIT | RustCrypto, déjà présent dans le workspace |
| `serde` / `serde_json` | JSON des API | MIT / Apache-2.0 | standard du projet |
| `chrono` | dates et horodatages | MIT / Apache-2.0 | déjà présent via `ac-noyau` |
| `thiserror` | erreurs typées (règle R8) | MIT | déjà présent dans le workspace |

Aucune licence non libre, aucune dépendance abandonnée, aucun service tiers.

## 7. Suites (lots suivants de `PROMPT_A10.md`)

- Lot 2 : client `ac-licence` (vérification hors ligne, empreinte du poste, horloge reculée).
- Lot 3 : glue `src-tauri`, commandes `compte_*` / `dossier_*` / `session_*`.
- Lot 4 : sauvegardes distantes chiffrées et avis.
- Lot 5 : mises à jour signées (`GET /v1/mises-a-jour/windows/{version}`).
- Lot 6 : installateur Windows et publication.
- Lot 7 : console d'administration (`web-admin/`), API `/admin/v1`, TOTP, audit.
- Lot 8 : ASSISTANCE (conversations, SSE).
- Lot 9 : annonces et publications (CGU).
- Transport SMTP des courriels, dès le choix de l'hébergeur.
