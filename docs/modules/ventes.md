# Module Ventes (A4)

Clients, devis, factures, avoirs et facture normalisée MECeF / e-MECeF. Propriétaire : **agent A4**.
Source : `PROMPT_A4.md`, `docs/contrats.md` (C3, C4, C6, C8), ADR 0006.

## État des lots

| Lot | Contenu | État |
|---|---|---|
| 1 — Clients | Fichier clients, recherche, désactivation, client « Comptant » | **Livré** |
| 2 — Factures : modèle et calcul | Tables `ventes_document` / `ventes_ligne`, totaux via `ac-fiscal` | À faire |
| 3 — Validation, numérotation, écriture comptable | `F2027-000001`, écriture VE, sorties de stock | À faire |
| 4 — Normalisation | `soumettre` → `finaliser`, file d'attente hors ligne | À faire |
| 5 — Documents PDF | Facture, avoir, devis + filigranes | À faire |
| 6 — Interface « Ventes » | Saisie rapide, vente au comptoir, historique client | À faire |

## Lot 1 — Fichier clients

### Table `ventes_client` (migration 2000)

| Colonne | Type | Règle |
|---|---|---|
| `id` | INTEGER | Clé interne |
| `code` | TEXT | `CLI-000001`… attribué automatiquement (tiers comptable, contrat C3) |
| `nom` | TEXT | Nom ou raison sociale (obligatoire, 200 caractères au plus) |
| `type_client` | TEXT | `PARTICULIER` \| `ENTREPRISE` \| `ADMINISTRATION` |
| `ifu` | TEXT | Facultatif, **13 chiffres** (`ac_noyau::Ifu`) ; unique entre clients |
| `rccm` | TEXT | Facultatif, normalisé (`ac_noyau::entreprise::Rccm`) |
| `adresse` | TEXT | Facultative |
| `telephone` | TEXT | Facultatif, normalisé `+229…` (`ac_noyau::entreprise::Telephone`) |
| `courriel` | TEXT | Facultatif, contrôlé (`ac_noyau::entreprise::courriel_valide`) |
| `retient_tva` | INTEGER | Le client retient la TVA à la source (État, sociétés désignées) |
| `exonere_aib` | INTEGER | Exonéré d'AIB ; `motif_exonere_aib` obligatoire dans ce cas |
| `actif` | INTEGER | Un client **facturé ne se supprime jamais** : on le désactive |
| `cree_le` / `modifie_le` | TEXT | Horodatages ISO-8601 |

Règles complémentaires :

- Le **client « Comptant »** (`CLI-000001`, « Client comptant ») est créé par la migration pour la
  vente au comptoir : il ne peut être ni modifié ni désactivé (contrôle Rust + déclencheur SQL).
- La recherche porte sur le **nom, le code, l'IFU et le téléphone** ; elle est insensible à la
  casse (ASCII) et tolère les espaces saisis dans un téléphone (`01 97 12 34 56` trouve
  `+2290197123456`). Les caractères `%` et `_` sont pris au littéral. Résultats limités (500 au plus).
- Les identifiants saisis sont **validés et normalisés côté Rust** (`ac_ventes::clients::InformationsClient::valider`).
- Toute création / modification / désactivation / réactivation est **journalisée** dans l'audit
  chaîné (`ac_donnees::audit`, règle R5), action `CREER` / `MODIFIER` / `DESACTIVER` / `REACTIVER`
  sur l'objet `ventes_client` identifié par son code.

### Commandes IPC (contrat C8)

| Commande | Arguments | Renvoie | Permission |
|---|---|---|---|
| `ventes_client_creer` | `informations` | `Client` | `Saisir` |
| `ventes_client_modifier` | `id`, `informations` | `Client` | `Saisir` |
| `ventes_client_obtenir` | `id` | `Client` | `Consulter` |
| `ventes_clients_rechercher` | `recherche { texte, inclure_inactifs, limite }` | `Client[]` | `Consulter` |
| `ventes_client_desactiver` | `id` | — | `Saisir` |
| `ventes_client_reactiver` | `id` | — | `Saisir` |

Types TypeScript : `ui/src/modules/ventes/types.ts` (champs snake_case identiques au Rust).
Erreurs renvoyées avec un code stable (`VALIDATION`, `CLIENT_INCONNU`, `CLIENT_COMPTANT_PROTEGE`,
`ETAT_CLIENT`, `IFU_DEJA_UTILISE`, `NON_AUTORISE`, `DONNEES`) et un message français.

### Écran « Clients »

`ui/src/modules/ventes/clients.tsx` : liste et recherche (nom, IFU, téléphone), filtre des
désactivés, formulaire de création / modification, désactivation / réactivation. Aucun calcul
métier dans l'interface (architecture §8) : les contrôles viennent de `ac-ventes`.

## Suites / hors périmètre du Lot 1

- **Import CSV/XLSX des clients** (aperçu, correspondance de colonnes, rapport d'erreurs ligne par
  ligne) : dépend de `ac_documents::lire_tableau` (A1), **pas encore livré**. Commande prévue :
  `ventes_importer_clients` (appelée aussi par l'assistant de reprise de A9).
- Suppression physique d'un client jamais facturé : non proposée (désactivation uniquement) ; à
  décider avec le chef de projet à l'arrivée des factures (Lot 2/3).
- Écran fiche client (historique, solde dû) : Lot 6.

## Écritures comptables générées

Aucune au Lot 1. À partir du Lot 3, chaque facture validée passe au journal `VE` :
`411 (tiers CLI-…) débit TTC + AIB` / `70x crédit HT` / `443x crédit TVA` (étiquette `TvaCollectee`) /
compte d'AIB à reverser crédit (étiquette `AibCollecte`) — voir `docs/contrats.md` C4 et C5.
