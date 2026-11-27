# Architecture d'AMAZONE COMP

Version 1.0 du document, novembre 2026. Propriétaire : chef de projet. Toute modification structurante passe par un ADR.

## 1. Vue d'ensemble

```
┌──────────────────────────────────────────────────────────────┐
│  Poste Windows (hors ligne d'abord)                          │
│                                                              │
│  ui/ (React + TS, WebView2)                                  │
│     │  invoke("commande", args)  ← IPC Tauri, JSON           │
│  src-tauri/ (commandes, état applicatif, session, updater)   │
│     │                                                        │
│  crates métier Rust                                          │
│   ac-ventes  ac-achats  ac-tresorerie  ac-paie  ac-etats     │
│        \         |          |            |        /          │
│         ac-fiscal (paramètres + calculs + déclarations)      │
│                 \           |            /                   │
│                  ac-compta (moteur SYSCOHADA)                │
│                          |                                   │
│     ac-documents (PDF/Excel)   ac-licence (compte, licence)  │
│                          |                                   │
│                  ac-donnees (SQLite chiffrée, audit)         │
│                          |                                   │
│                  ac-noyau (Montant, Taux, erreurs, dates)    │
│                                                              │
│  Fichier dossier : %APPDATA%\AmazoneComp\dossier.acdb        │
└──────────────────────────────┬───────────────────────────────┘
                               │ HTTPS (seulement si connecté)
┌──────────────────────────────┴───────────────────────────────┐
│  plateforme/ (Rust axum + PostgreSQL, un VPS)                │
│  inscription des entreprises (IFU, RCCM…) · comptes · CGU    │
│  licences signées Ed25519 · sauvegardes chiffrées (opaques)  │
│  avis · ASSISTANCE (messagerie) · annonces · flux de MAJ     │
│                                                              │
│  plateforme/web-admin/ (React) — console du propriétaire     │
│  https://admin.<domaine> · 2FA · entreprises · licences      │
│  ASSISTANCE en temps réel (SSE) · annonces · publications    │
└──────────────────────────────────────────────────────────────┘
```

## 2. Règle de dépendance entre crates (stricte)

Une crate ne dépend **que** des crates situées en dessous d'elle. Toute dépendance circulaire est interdite.

| Crate | Peut dépendre de |
|---|---|
| `ac-noyau` | (aucune crate interne) |
| `ac-donnees` | `ac-noyau` |
| `ac-compta` | `ac-noyau`, `ac-donnees` |
| `ac-documents` | `ac-noyau` |
| `ac-fiscal` | `ac-noyau`, `ac-donnees`, `ac-compta`, `ac-documents` |
| `ac-achats` (achats, catalogue d'articles, stocks, immobilisations) | `ac-noyau`, `ac-donnees`, `ac-compta`, `ac-fiscal`, `ac-documents` |
| `ac-ventes` | les mêmes **+ `ac-achats`** (uniquement les modules publics `catalogue` et `stock`) |
| `ac-tresorerie`, `ac-paie` | `ac-noyau`, `ac-donnees`, `ac-compta`, `ac-documents` (+ `ac-fiscal` pour les retenues) |
| `ac-etats` | `ac-noyau`, `ac-donnees`, `ac-compta`, `ac-fiscal`, `ac-documents` |
| `ac-licence` | `ac-noyau`, `ac-donnees` |
| `src-tauri` | toutes |

Les modules métier **ne se parlent pas directement** (ex. `ac-ventes` n'appelle pas `ac-tresorerie`), à une seule exception : `ac-ventes` utilise le catalogue d'articles et les sorties de stock de `ac-achats`. Un encaissement de facture est créé par `ac-tresorerie` qui **lettre** le compte client via `ac-compta`. La coordination entre modules se fait par la comptabilité (comptes, lettrage, étiquettes fiscales).

## 3. Principe central : tout finit en écriture comptable

1. Chaque module produit des **documents métier** (facture, bon de réception, bulletin, relevé…).
2. À la **validation** d'un document, le module construit un `BrouillonEcriture` et appelle `ac_compta::passer_ecriture(&tx, brouillon)` **dans la même transaction SQL** que la validation du document.
3. `ac-compta` vérifie : équilibre, comptes existants et mouvementables, période ouverte, journal, dates ; numérote ; enregistre ; écrit le journal d'audit.
4. Chaque ligne d'écriture peut porter une **étiquette fiscale** (`CodeFiscal`, ex. `TvaCollectee`, `AibCollecte`, `RetenueLoyer`) et une **base** (montant HT concerné). `ac-fiscal` calcule les déclarations **à partir des écritures étiquetées**, jamais en lisant les tables des autres modules.

Conséquence : les états financiers, la balance et les déclarations sont toujours cohérents entre eux.

## 4. Données locales

- Un seul fichier SQLite chiffré par installation (SQLCipher, AES-256). Mode WAL, `foreign_keys = ON`.
- **Migrations par crate**, numérotées par plages réservées, exécutées par le moteur de `ac-donnees` dans l'ordre global :

| Plage | Crate |
|---|---|
| 0001–0999 | `ac-donnees` (utilisateurs, rôles, audit, paramètres dossier) |
| 1000–1999 | `ac-compta` |
| 2000–2999 | `ac-ventes` |
| 3000–3999 | `ac-achats` |
| 4000–4999 | `ac-tresorerie` |
| 5000–5999 | `ac-paie` |
| 6000–6999 | `ac-fiscal` |
| 7000–7999 | `ac-etats` |
| 8000–8999 | `ac-licence` |

- Chaque crate expose `pub fn migrations() -> &'static [ac_donnees::Migration]`. `src-tauri` les enregistre toutes au démarrage.
- Une migration publiée n'est **jamais modifiée** ; on en ajoute une nouvelle.
- Tables nommées `<module>_<objet>` (ex. `ventes_facture`, `compta_ecriture`). Montants : `INTEGER` (FCFA). Taux : `TEXT` (décimal exact). Dates : `TEXT` ISO-8601.

## 5. Sécurité et utilisateurs

- Authentification locale : chaque utilisateur a un mot de passe (Argon2id). La clé de la base est aléatoire (256 bits) et **enveloppée** par la clé dérivée de chaque mot de passe + une **clé de secours** imprimée à la création du dossier.
- Rôles (V1) : `Administrateur`, `Comptable`, `Saisie`, `Paie`, `Lecture`. Les droits sont vérifiés **dans les commandes Rust** via `Session::exiger(Permission)`.
- Journal d'audit : chaque modification enregistre (horodatage, utilisateur, action, objet, empreinte avant/après, hash précédent → hash courant). Vérification d'intégrité disponible dans l'interface.

## 6. Compte, licence, mises à jour (résumé de l'ADR 0004)

1. Premier lancement : acceptation des CGU (version enregistrée), création de compte ou connexion sur la plateforme (**Internet obligatoire une seule fois**), **inscription de l'entreprise** avec sa fiche légale complète (IFU, RCCM, forme juridique, adresse, responsable… — ADR 0009), saisie une seule fois et réutilisée par la comptabilité.
2. La plateforme renvoie une **licence signée Ed25519** : `{compte, entreprise_ifu, empreinte_poste, emise_le, expire_le = +6 mois, offre = "gratuite"}`.
3. À chaque démarrage : vérification **hors ligne** (signature + date). Renouvellement automatique dès qu'Internet est disponible dans les 30 derniers jours.
4. Licence expirée : **15 jours de grâce**, puis **mode consultation** (lecture, exports, sauvegarde locale autorisés ; saisie bloquée). On ne prend jamais les données de l'utilisateur en otage.
5. Mises à jour : uniquement avec licence valide ; paquets signés (Tauri updater). Mode **automatique** ou **manuel**, réglable dans Paramètres.
6. **Administration** : le propriétaire gère entreprises, licences, annonces et versions depuis la console web, et **discute avec les utilisateurs** via ASSISTANCE (module « Assistance » dans le logiciel ↔ console web) — ADR 0010.

## 7. Facturation normalisée (résumé de l'ADR 0006)

Trait `ac_ventes::normalisation::Normaliseur` avec trois implémentations prévues :

- `Simulateur` (disponible dès maintenant, utilisé en développement et en démonstration ; les factures portent la mention « SIMULATION — NON VALABLE FISCALEMENT ») ;
- `EMecefApi` (API en ligne de la DGI : à implémenter quand l'accès sera obtenu ; URL et jeton dans les paramètres) ;
- `MecefMateriel` (boîtier MECeF : réservé, non prévu en V1).

## 8. Interface

- React + TypeScript strict, Vite, React Router. Pas de bibliothèque de composants lourde : design system maison dans `ui/src/design/`.
- Les écrans métier vivent dans `ui/src/modules/<module>/` et sont enregistrés dans le routeur par une déclaration `module.ts` (propriété de l'agent du module).
- Appels au cœur : `ui/src/lib/ipc.ts` → `appeler<T>("module_action", args)`. **Aucun calcul fiscal ou comptable dans l'interface.**
- Polices système (Segoe UI) : aucun téléchargement externe, fonctionnement 100 % hors ligne.

## 9. Performances (budget)

| Mesure | Budget |
|---|---|
| Installateur | < 15 Mo |
| Démarrage à froid | < 3 s (PC 4 Go, disque dur classique : < 6 s) |
| RAM (WebView2 compris) | < 150 Mo en usage courant, < 250 Mo en génération de déclaration |
| Saisie d'une écriture | < 100 ms |
| Balance sur 50 000 écritures | < 1 s |

## 10. Import de données (reprise)

Lecture des fichiers : `ac-documents` (A1) fournit `lire_tableau` (CSV UTF-8 / Windows-1252, XLSX). Chaque module importe ses propres données (clients A4, fournisseurs et articles A5, relevés A6, salariés A7) ; A9 fournit l'assistant « Reprise » et importe plan de comptes et balance d'ouverture. Formats acceptés : **CSV** (UTF-8 et Windows-1252) et **Excel .xlsx** (modèles fournis par le logiciel), **relevés bancaires CSV** (profils par banque) et **exports Mobile Money CSV**. Toujours en 3 étapes : *aperçu → correspondance des colonnes → validation avec rapport d'erreurs ligne par ligne*. Rien n'est écrit tant que l'utilisateur n'a pas validé. Détails : PROMPT_A9 (reprise) et PROMPT_A6 (relevés).
