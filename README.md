# AMAZONE COMP

**Logiciel de comptabilité gratuit et libre pour les PME du Bénin.**

AMAZONE COMP permet à une PME béninoise de tenir toute sa comptabilité, de A à Z, sans être comptable :
saisie guidée, facturation normalisée (MECeF / e-MECeF), achats, stocks, trésorerie (banque, caisse,
MTN MoMo, Moov Money, Celtiis), paie (ITS, VPS, CNSS), fiscalité (TVA, AIB, IS, IBA, TPS) et états
financiers SYSCOHADA révisé. Le logiciel **prépare** les déclarations (PDF + Excel au format DGI) ;
l'utilisateur les dépose lui-même sur e-services de la DGI.

> Statut : **en construction — V1 prévue en 6 mois.** Ne pas utiliser en production avant la
> validation des règles fiscales par un expert-comptable (voir [docs/fiscalite](docs/fiscalite/)).

---

## Caractéristiques

| Domaine | Contenu V1 |
|---|---|
| Plateforme | Windows 10 et suivants, **64 bits uniquement**, PC à partir de **4 Go de RAM** |
| Fonctionnement | **Hors ligne d'abord**. En ligne seulement pour : compte, licence, mises à jour, sauvegarde/synchronisation |
| Périmètre | Un poste, **un dossier d'entreprise** par installation. Plusieurs utilisateurs avec rôles |
| Pays / devise | **Bénin uniquement**, **FCFA (XOF) uniquement**, interface **en français uniquement** |
| Référentiel | SYSCOHADA révisé (Acte uniforme OHADA relatif au droit comptable, 2017) |
| Régimes fiscaux | Réel normal, réel simplifié, TPS (Taxe professionnelle synthétique) |
| Licence d'utilisation | Gratuite, **obligatoire**, renouvelée **tous les 6 mois** (compte sur la plateforme AMAZONE COMP) |
| Licence du code | [AGPL-3.0](LICENSE) |

## Pile technique (choisie pour la légèreté)

- **Tauri 2** : application de bureau qui utilise WebView2 (déjà présent sur Windows 10/11), pas de Chromium embarqué. Installateur de quelques Mo, faible consommation mémoire (objectif : **< 150 Mo de RAM** en usage normal).
- **Rust** (cœur métier) : calculs exacts, sûreté mémoire, rapidité. Montants en **entiers FCFA** (jamais de nombres flottants).
- **SQLite chiffrée (SQLCipher)** : base locale, un seul fichier, sans serveur.
- **React + TypeScript + Vite** : interface, design sobre noir et blanc.
- **Plateforme en ligne** (`plateforme/`) : Rust (axum) + PostgreSQL, déployable sur un petit VPS (seul coût : l'hébergement).

Détails : [docs/architecture.md](docs/architecture.md) et [docs/adr/](docs/adr/).

## Arborescence

```
amazone-comp/
├── AGENTS.md              Règles communes OBLIGATOIRES pour tous les contributeurs (humains et IA)
├── PROMPT_A1.md … A10.md  Mission détaillée de chacun des 10 agents
├── crates/                Cœur métier Rust (une crate par module)
│   ├── ac-noyau           Types communs : Montant, Taux, dates, erreurs, identifiants
│   ├── ac-compta          Moteur comptable SYSCOHADA (plan, journaux, écritures, balance…)
│   ├── ac-documents       Rendu commun PDF et Excel (factures, déclarations, états)
│   ├── ac-donnees         SQLite chiffrée, migrations, journal d'audit, sauvegardes, rôles
│   ├── ac-ventes          Clients, devis, factures, avoirs, adaptateur MECeF / e-MECeF
│   ├── ac-achats          Fournisseurs, achats, stocks
│   ├── ac-tresorerie      Banque, caisse, Mobile Money, rapprochement, import de relevés
│   ├── ac-paie            Salariés, bulletins, ITS, VPS, CNSS
│   ├── ac-fiscal          Paramètres fiscaux, TVA, AIB, IS/IBA, TPS, calendrier, déclarations
│   ├── ac-etats           États financiers, liasse/DSF, tableaux de bord, reprise de données
│   └── ac-licence         Compte, licence signée, synchronisation (côté application)
├── src-tauri/             Application de bureau (commandes IPC, démarrage, mise à jour)
├── ui/                    Interface React + TypeScript
├── parametres-fiscaux/    Taux et barèmes versionnés par année, avec source légale
├── plateforme/            Serveur en ligne : comptes, licences, sauvegardes, avis, mises à jour
├── docs/                  Architecture, ADR, contrats, fiscalité, feuille de route
└── scripts/               Outils de vérification (périmètre des agents, etc.)
```

## Démarrer (développeurs)

Prérequis : Rust stable (voir `rust-toolchain.toml`), Node.js ≥ 20, Visual Studio Build Tools (MSVC, « Développement Desktop en C++ »), WebView2.

```powershell
npm install
cargo test --workspace
npm run test
npm run dev
```

`npm run dev` lance l'application de bureau avec rechargement à chaud (Tauri + Vite).

## Contribuer

Le projet est développé par un chef de projet et 10 agents. **Lire [AGENTS.md](AGENTS.md) et [CONTRIBUTING.md](CONTRIBUTING.md) avant toute contribution.** Aucune modification directe de `main` : tout passe par une Pull Request relue, testée et validée par le chef de projet.

## Sécurité

Signaler une faille en privé : voir [SECURITY.md](SECURITY.md).

## Avertissement

AMAZONE COMP aide à tenir la comptabilité et à préparer les déclarations. L'utilisateur reste responsable de ses déclarations auprès de la DGI et de la CNSS. Les taux sont ceux des textes cités dans `parametres-fiscaux/` ; vérifiez toujours la loi de finances de l'année.

## Licence

Copyright © 2026 A COMPLETER (propriétaire du projet AMAZONE COMP).
Code distribué sous licence [GNU AGPL-3.0](LICENSE). La marque « AMAZONE COMP », le logo et la plateforme en ligne ne sont pas couverts par cette licence.
