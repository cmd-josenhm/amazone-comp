# AGENTS.md — Règles communes OBLIGATOIRES

Ce fichier s'applique à **tous** les contributeurs d'AMAZONE COMP : les 10 agents IA (A1 à A10) et les humains.
Chaque agent lit, **dans cet ordre**, avant d'écrire une seule ligne :

1. ce fichier `AGENTS.md` ;
2. son prompt `PROMPT_AN.md` (à la racine) ;
3. `docs/architecture.md`, `docs/contrats.md` et `docs/fiscalite/regles-fiscales-benin.md` ;
4. les ADR de `docs/adr/` qui concernent son module.

Le **chef de projet** (le mainteneur) garde le contrôle : il relit, teste et valide chaque Pull Request.
Une règle de ce fichier ne se négocie pas dans une PR : pour la changer, ouvrir une issue `rfc:` adressée au chef de projet.

---

## 1. Le produit en 10 lignes

- Logiciel de comptabilité **Windows 10+ 64 bits** pour **PME du Bénin**, utilisable **sans comptable**.
- **Hors ligne d'abord** ; Internet sert uniquement au compte, à la licence, aux mises à jour et à la sauvegarde.
- **Un poste, un dossier d'entreprise**, plusieurs utilisateurs avec rôles.
- **FCFA uniquement**, **français uniquement**, **Bénin uniquement**.
- Référentiel **SYSCOHADA révisé** ; fiscalité **béninoise** (CGI + loi de finances de l'année).
- Régimes : **réel normal, réel simplifié, TPS**.
- Le logiciel **prépare** les déclarations (PDF + Excel) ; il ne dépose rien lui-même.
- Facturation normalisée **MECeF / e-MECeF** via un **adaptateur** (l'API réelle viendra plus tard ; aujourd'hui : simulateur).
- Licence d'utilisation gratuite mais **obligatoire**, valable **6 mois**, liée à un compte sur la plateforme.
- Doit tourner sur un PC **4 Go de RAM** : chaque dépendance doit se justifier.

## 2. Règles d'or (violation = PR refusée sans discussion)

| N° | Règle |
|---|---|
| R1 | **Jamais de `f32`/`f64` pour un montant, une quantité valorisée ou un taux.** Montants = `ac_noyau::Montant` (entier FCFA, `i64`). Taux = `ac_noyau::Taux` (décimal exact). En TypeScript : montants en `number` **entiers** uniquement, calculs faits côté Rust. |
| R2 | **Aucun taux, seuil, barème ou date fiscale codé en dur.** Tout vient de `parametres-fiscaux/<année>/*.toml`, chaque valeur avec sa **source légale** (article CGI / loi de finances). |
| R3 | **Une écriture validée est immuable.** On ne la modifie ni ne la supprime : on la contre-passe. Les périodes clôturées sont verrouillées. |
| R4 | **Partie double toujours équilibrée** : somme des débits = somme des crédits, vérifiée dans le code **et** par une contrainte en base. |
| R5 | **Toute action qui modifie des données passe par le journal d'audit** chaîné (hash SHA-256), via `ac-donnees`. |
| R6 | **Pas de donnée personnelle ou comptable envoyée sur le réseau** sauf : sauvegarde chiffrée côté client, appels licence/compte, appels MECeF/e-MECeF. Pas de télémétrie, pas d'analytics, pas de CDN. |
| R7 | **Pas de secret dans le dépôt** (clés, mots de passe, jetons). La clé publique de licence est la seule clé autorisée en clair. |
| R8 | **Pas de `unwrap()`, `expect()`, `panic!` dans le code de production Rust** (autorisé dans les tests). Erreurs typées avec `thiserror`. Pas de `unsafe`. |
| R9 | **Tests obligatoires** : toute fonction de calcul a des tests avec des **exemples chiffrés** écrits à la main, dont au moins un cas limite (zéro, seuil exact, arrondi). Couverture minimale **80 %** des crates métier. |
| R10 | **Tu ne modifies que ton périmètre** (voir §4). Un besoin hors périmètre = issue adressée à l'agent propriétaire + au chef de projet. Le script `scripts/verifier-perimetre.mjs` bloque la PR sinon. |
| R11 | **Les contrats (`docs/contrats.md`, traits et types publics) ne changent pas sans ADR** validé par le chef de projet. |
| R12 | **Aucune nouvelle dépendance** (crate ou paquet npm) sans la justifier dans la PR (taille, licence compatible AGPL, maintenance). Interdites : licences non libres, dépendances abandonnées, gros frameworks UI. |
| R13 | **Français partout dans le produit** : textes UI, messages d'erreur visibles, documents générés. Le code (identifiants) est en **français sans accents** (`montant_ht`, `ecriture`), commentaires en français. |
| R14 | **Ne jamais inventer une règle fiscale.** Si un point de loi est incertain : implémenter le paramètre, marquer `verifie = false` dans le TOML, et ouvrir une issue `fiscal-a-verifier`. |

## 3. Workflow Git (strict)

1. Branche par tâche, depuis `main` à jour : `aN/<type>-<sujet-court>` (ex. `a4/feat-facture-avoir`). N = ton numéro d'agent.
2. Commits **Conventional Commits** en français : `feat(ventes): ajoute les avoirs`, `fix(fiscal): …`, `test(paie): …`, `docs: …`. Types : `feat fix test docs refactor perf chore ci build`.
3. **Une PR = un sujet**, idéalement < 600 lignes modifiées (hors tests et fichiers générés). Remplir le modèle de PR en entier.
4. Avant d'ouvrir la PR, en local, tout doit passer :
   ```powershell
   cargo fmt --all -- --check
   cargo clippy --workspace --all-targets -- -D warnings
   cargo test --workspace
   npm run lint
   npm run test
   node scripts/verifier-perimetre.mjs aN
   ```
5. Interdit : `push --force` sur `main`, fusion de sa propre PR, désactivation d'un test ou d'une règle de lint pour « faire passer », `#[allow(...)]` sans commentaire justificatif.
6. Seul le chef de projet fusionne. Une PR refusée se corrige dans la même branche.

## 4. Périmètres (qui a le droit de modifier quoi)

La source de vérité est `.github/perimetres.json` (lu par `scripts/verifier-perimetre.mjs`).

| Agent | Mission | Dossiers possédés |
|---|---|---|
| **A1** | Noyau, moteur comptable SYSCOHADA, rendu PDF/Excel commun | `crates/ac-noyau/`, `crates/ac-compta/`, `crates/ac-documents/`, `ui/src/modules/comptabilite/`, `src-tauri/src/commandes/compta.rs` |
| **A2** | Données locales : SQLite chiffrée, migrations, audit, sauvegardes, utilisateurs & rôles | `crates/ac-donnees/` |
| **A3** | Interface : coque, design system, assistant de démarrage, paramètres, « À propos » & avis | `ui/` (sauf les dossiers `ui/src/modules/<module>/` et `proprietaire.ts`), `src-tauri/src/commandes/systeme.rs` |
| **A4** | Ventes : clients, devis, factures, avoirs, MECeF / e-MECeF | `crates/ac-ventes/`, `ui/src/modules/ventes/`, `src-tauri/src/commandes/ventes.rs` |
| **A5** | Achats, catalogue d'articles, stocks, immobilisations & amortissements | `crates/ac-achats/`, `ui/src/modules/achats/`, `src-tauri/src/commandes/achats.rs` |
| **A6** | Trésorerie : banque, caisse, Mobile Money, rapprochement, import de relevés | `crates/ac-tresorerie/`, `ui/src/modules/tresorerie/`, `src-tauri/src/commandes/tresorerie.rs` |
| **A7** | Paie : ITS, VPS, CNSS, bulletins | `crates/ac-paie/`, `ui/src/modules/paie/`, `src-tauri/src/commandes/paie.rs`, `parametres-fiscaux/*/social.toml` |
| **A8** | Fiscalité : TVA, AIB, IS/IBA, TPS, retenues, calendrier, déclarations PDF/Excel | `crates/ac-fiscal/`, `ui/src/modules/fiscal/`, `src-tauri/src/commandes/fiscal.rs`, `parametres-fiscaux/` (sauf `social.toml`), `docs/fiscalite/` |
| **A9** | États financiers, liasse/DSF, tableaux de bord, reprise de données | `crates/ac-etats/`, `ui/src/modules/etats/`, `ui/src/modules/tableau-de-bord/`, `src-tauri/src/commandes/etats.rs` |
| **A10** | Plateforme en ligne, licence & anti-piratage, sauvegarde distante, mises à jour, installateur, releases | `plateforme/`, `crates/ac-licence/`, `src-tauri/` (sauf les `commandes/<module>.rs` des autres), `ui/src/modules/compte/`, `.github/workflows/release.yml`, `.github/workflows/plateforme.yml` |
| **Chef de projet** | Tout le reste : `AGENTS.md`, `PROMPT_*.md`, `docs/`, `Cargo.toml` racine, `.github/` (hors release), `scripts/` | — |

**Fichiers partagés** (modifiables par tous, uniquement dans sa propre section) : `src-tauri/src/commandes/mod.rs` (déclaration du module et liste des commandes), `CHANGELOG.md`, fichiers de verrouillage `Cargo.lock` / `package-lock.json`.

Chaque agent peut **ajouter** sa documentation dans `docs/modules/<son-module>.md` et ses ADR **proposés** dans `docs/adr/` (statut « Proposé » ; le chef de projet passe le statut à « Accepté »).

## 5. Qualité minimale exigée

- **Rust** : édition 2021, `cargo fmt`, `clippy -D warnings`, `#![forbid(unsafe_code)]` en tête de chaque crate, docs `///` sur tout élément public.
- **TypeScript** : `strict: true`, pas de `any`, ESLint sans avertissement, composants fonctionnels, pas de bibliothèque UI lourde (pas de MUI/Ant/Bootstrap).
- **Performance** : démarrage < 3 s sur PC 4 Go ; écran principal < 150 Mo RAM ; listes paginées (jamais charger 100 000 lignes en mémoire).
- **Accessibilité** : navigation complète au clavier, contrastes AA, libellés sur tous les champs.
- **Erreurs** : message clair pour un non-comptable + que faire. Jamais de trace technique brute à l'écran.

## 6. Fiscalité : comment travailler

- Les règles sont décrites dans `docs/fiscalite/regles-fiscales-benin.md` ; les valeurs vivent dans `parametres-fiscaux/2026/`.
- Chaque valeur porte `source = "..."` et `verifie = true|false`. **Aucune valeur n'est encore validée par un expert-comptable** : tout reste `verifie = false` tant que le chef de projet ne l'a pas confirmé.
- Une année fiscale = un dossier. On ne modifie **jamais** les valeurs d'une année passée (elles servent aux recalculs historiques) ; on crée l'année suivante.

## 7. Communication

- Questions et blocages : issue GitHub avec le libellé `agent:aN` et `question`. Ne pas deviner en silence.
- Dépendance vers un autre agent : issue avec `agent:aN` du demandeur **et** du propriétaire, en décrivant le contrat attendu (signature Rust / format).
- Chaque PR liste dans « Hors périmètre / suites » ce qui reste à faire.
