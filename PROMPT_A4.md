# PROMPT A4 — Ventes : clients, devis, factures, avoirs, facture normalisée (MECeF / e-MECeF)

> **Lis d'abord, dans cet ordre :** `AGENTS.md` (règles R1 à R14, **non négociables**), ce fichier, `docs/architecture.md`,
> `docs/contrats.md` (C3, C4, C5, C6, C7), `docs/fiscalite/regles-fiscales-benin.md` (§4.1, §4.2, §5), `docs/adr/0006-adaptateur-emecef.md`.
> Tu ne connais ni le projet ni le droit béninois : **ne devine rien**. Question : issue GitHub `agent:a4` + `question`.

## 1. Le projet en une minute

AMAZONE COMP est un logiciel de comptabilité **gratuit et open source** pour les **PME du Bénin**, Windows 10+ 64 bits, utilisable
**sans comptable**, **hors ligne d'abord**. Pile : **Tauri 2 + Rust** (métier) + **React/TypeScript** + **SQLite chiffrée**.
Monnaie : **FCFA entiers**. Référentiel : **SYSCOHADA révisé**. Dix agents en parallèle ; le **chef de projet** valide chaque PR.

## 2. Ta mission

Tu gères **tout le cycle de vente** : fichier clients, devis, factures, avoirs, ventes au comptoir, **facture normalisée**
(enregistrement auprès de la DGI, aujourd'hui par **simulateur**), impression PDF, et l'**écriture comptable** de chaque
facture validée. C'est le module le plus utilisé au quotidien : il doit être **rapide** et **impossible à mal utiliser**.

## 3. Ton périmètre

| Autorisé | Interdit |
|---|---|
| `crates/ac-ventes/`, `ui/src/modules/ventes/`, `src-tauri/src/commandes/ventes.rs`, ta section de `commandes/mod.rs`, `docs/modules/ventes.md`, ADR « Proposé » | Tout le reste. Le **catalogue d'articles** et les **stocks** appartiennent à A5 : tu les utilises via `ac_achats::catalogue` et `ac_achats::stock` (seule dépendance inter-modules autorisée). Les **encaissements** appartiennent à A6. Les **taux** viennent de A8. |

## 4. Connaissances métier indispensables (Bénin)

- **Facture normalisée** obligatoire (DGI) : chaque facture validée est transmise à la DGI (boîtier **MECeF** ou API en ligne **e-MECeF**),
  qui renvoie : **code MECeF/DGI**, **NIM**, **compteurs**, **date-heure**, **QR code** — à imprimer sur la facture.
  Le flux est en **deux temps** : soumettre → la DGI renvoie un identifiant et ses totaux → **confirmer** (ou annuler).
- **Pas d'accès à l'API pour l'instant** : on utilise `Simulateur` (déjà codé). Toute facture simulée affiche en grand
  « SIMULATION — NON VALABLE FISCALEMENT ».
- Une facture validée **ne se modifie jamais** : on émet une **facture d'avoir** (type `FA`, ou `EA` à l'export) qui référence
  le code MECeF de la facture d'origine (total ou partiel).
- **Groupes de taxation** e-MECeF par article : A exonéré, B taxable 18 %, C exportation, D régime d'exception, E TPS, F réservé
  (valeurs dans `parametres-fiscaux/<année>/emecef.toml`, **à confirmer**). Une entreprise au régime **TPS** ne facture **pas** de TVA (groupe E).
- **TVA** : 18 % sur le HT (taux lu dans les paramètres). **AIB** : prélevé **par toi (le vendeur) sur le client** et ajouté à la
  facture : 1 % si le client a un IFU (biens), 3 % (services, à confirmer), 5 % sans IFU ; base hors TVA ; codes e-MECeF A / B.
  Certains clients (État, sociétés désignées) **retiennent la TVA à la source** : le client te paie moins, la facture le mentionne.
  **Toutes ces règles sont calculées par `ac-fiscal` (A8)** : tu appelles ses fonctions, tu ne codes aucun taux.
- **Mentions obligatoires** sur la facture (liste à faire valider par l'expert-comptable, issue `fiscal-a-verifier`) : raison sociale,
  adresse, **IFU** et RCCM du vendeur ; numéro unique et date ; client (nom, IFU s'il en a) ; désignation, quantité, prix unitaire,
  groupe de taxation ; totaux HT, TVA, TTC, AIB ; mode de paiement ; éléments de sécurité MECeF.
- Paiements courants : espèces, Mobile Money (MTN MoMo, Moov Money, Celtiis), virement, chèque, crédit (paiement différé).

## 5. Ce qui existe déjà

- `crates/ac-ventes/src/normalisation.rs` : trait `Normaliseur` (`soumettre` / `finaliser`), types `DemandeNormalisation`,
  `FactureSoumise`, `FactureNormalisee`, `ErreurNormalisation`, et `Simulateur` (testé).
- `ac_compta::passer_ecriture`, `CodeFiscal` (A1) ; `ac_fiscal::tva::depuis_ht` (A8) ; `ac_documents::Document` (A1, rendu en cours).

## 6. Travail à livrer (une PR par lot)

### Lot 1 — Clients (≤ 3 jours)
Table `ventes_client` (code `CLI-000001` attribué automatiquement, nom/raison sociale, type PARTICULIER|ENTREPRISE|ADMINISTRATION,
IFU facultatif validé, RCCM, adresse, téléphone, e-mail, `retient_tva` bool, `exonere_aib` bool + motif, actif). Création, modification, recherche
(nom, IFU, téléphone), désactivation (jamais de suppression d'un client facturé). Client « Comptant » par défaut. Import CSV/XLSX via `ac_documents::lire_tableau` (aperçu, correspondance de colonnes, rapport d'erreurs ligne par ligne).

### Lot 2 — Factures : modèle et calcul (≤ 6 jours)
Tables `ventes_document` (type DEVIS|FACTURE|AVOIR, statut BROUILLON|VALIDEE|EN_ATTENTE_NORMALISATION|NORMALISEE|ANNULEE, numéro, date, client, mode de paiement, références) et `ventes_ligne`
(article ou ligne libre, désignation, quantité en millièmes, prix unitaire **HT** ou **TTC** selon le choix de saisie, remise, groupe de taxation).
Calcul **côté Rust** des totaux par groupe, TVA, AIB et TTC en appelant `ac-fiscal` (si une fonction manque, issue à A8 avec la signature voulue).
Règle d'arrondi : demander à A8 (règle unique, cohérente avec e-MECeF). Devis convertible en facture.

### Lot 3 — Validation, numérotation, écriture comptable (≤ 5 jours)
- Numérotation continue **sans trou** par type et par année : `F2027-000001`, `AV2027-000001`, `D2027-000001`.
- À la validation, **dans une seule transaction** : numéro, statut, sortie de stock (`ac_achats::stock::sortir`) pour les articles stockés, écriture :
  `411 (tiers CLI-…) débit TTC + AIB` / `70x crédit HT par compte de vente de l'article` / `443x crédit TVA` (étiquette `TvaCollectee`, base, taux)
  / compte d'AIB à reverser crédit (étiquette `AibCollecte`) / `CaExonere` ou `CaExport` sur les lignes concernées. Comptes en **constantes nommées**
  (`docs/contrats.md` C5), à valider par l'expert.
- Avoir : écriture inverse, référence à la facture d'origine, retour en stock si demandé.

### Lot 4 — Normalisation (≤ 5 jours)
- Après validation : `soumettre` → comparer les totaux DGI aux totaux locaux (écart → blocage + message) → `finaliser(Confirmer)` → stocker les éléments de sécurité.
- **Hors ligne** : statut `EN_ATTENTE_NORMALISATION`, file d'attente rejouée automatiquement (commande appelée par la coque au retour réseau) ;
  la facture ne peut pas être imprimée comme définitive avant normalisation (filigrane « EN ATTENTE DE NORMALISATION »).
- Paramètre du dossier : `Simulateur` | `EMecefApi` (désactivé tant qu'il n'existe pas) | `SaisieManuelle` (l'utilisateur recopie le code et le NIM d'un boîtier MECeF).
- Préparer `EMecefApi` derrière une interface HTTP abstraite (trait `ClientHttp`) pour qu'il soit testable sans réseau ; ne l'active pas.

### Lot 5 — Documents PDF (≤ 3 jours)
Facture, avoir, devis via `ac_documents` : noir et blanc, logo facultatif, mentions obligatoires, QR code (si A1 ne le fournit pas, demande-le),
filigranes SIMULATION / EN ATTENTE. Envoi : « Enregistrer le PDF » (pas d'envoi d'e-mail en V1).

### Lot 6 — Interface « Ventes » (≤ 6 jours)
Écrans : liste des factures (filtres, statut, payé/non payé **lu dans `ac-compta` via le lettrage**), saisie rapide d'une facture au clavier
(ajout d'article par code ou recherche, totaux mis à jour par appel Rust), **vente au comptoir** (client Comptant, encaissement immédiat proposé → écran A6),
fiche client (historique, solde dû), avoirs, devis. Commandes `ventes_*`.

## 7. Contrats

- **Tu fournis** : C6 (`Normaliseur`), commandes `ventes_*`, `ventes_importer_clients` (appelée par l'assistant de reprise de A9).
- **Tu consommes** : C3/C4 (A1), `ac_fiscal` (A8), `ac_achats::{catalogue, stock}` (A5), `ac_documents` (A1), C2 (A2).

## 8. Tests exigés (exemples chiffrés)

- 3 sacs de ciment à 5 000 HT, client avec IFU, régime réel : HT 15 000, TVA 2 700, TTC 17 700, AIB 1 % = 150, net à payer 17 850 (valeurs d'après les paramètres non vérifiés : le test lit les paramètres, il ne code pas 18 %).
- Même vente, client sans IFU : AIB 5 % = 750.
- Entreprise au régime TPS : aucune TVA, groupe E.
- Écriture générée équilibrée et étiquetée ; numérotation sans trou même après une validation échouée.
- Avoir partiel de 1 sac → écriture inverse proportionnelle, référence d'origine.
- Écart de totaux simulé par un faux `Normaliseur` → confirmation bloquée.
- Hors ligne → `EN_ATTENTE_NORMALISATION`, puis normalisée au rejeu.

## 9. Pièges à éviter

- Ne jamais supprimer une facture validée ; ne jamais réutiliser un numéro.
- Ne pas calculer la TVA ou l'AIB dans l'interface ni en dur : `ac-fiscal`.
- Prix TTC saisi → recalcul du HT **en Rust** avec la règle d'arrondi validée.
- Ne pas imprimer une facture simulée sans la mention SIMULATION.

## 10. Définition de « terminé »

CI verte, couverture ≥ 80 %, `docs/modules/ventes.md` (tables, écritures générées avec comptes et étiquettes, flux de normalisation), CHANGELOG, PR fusionnée.

## 11. Pour démarrer

```powershell
git checkout main; git pull
git checkout -b a4/feat-clients
cargo test -p ac-ventes
```
