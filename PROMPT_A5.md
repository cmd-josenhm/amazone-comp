# PROMPT A5 — Achats, catalogue d'articles, stocks, immobilisations et amortissements

> **Lis d'abord, dans cet ordre :** `AGENTS.md` (règles R1 à R14, **non négociables**), ce fichier, `docs/architecture.md`,
> `docs/contrats.md` (C3, C4, C5, C7), `docs/fiscalite/regles-fiscales-benin.md` (§2, §4.1, §4.2, §4.5).
> Tu ne connais ni le projet ni le droit béninois : **ne devine rien**. Question : issue GitHub `agent:a5` + `question`.

## 1. Le projet en une minute

AMAZONE COMP est un logiciel de comptabilité **gratuit et open source** pour les **PME du Bénin**, Windows 10+ 64 bits, utilisable
**sans comptable**, **hors ligne d'abord**. Pile : **Tauri 2 + Rust** + **React/TypeScript** + **SQLite chiffrée**. **FCFA entiers**.
**SYSCOHADA révisé**. Dix agents en parallèle ; le **chef de projet** valide chaque PR.

## 2. Ta mission

Tu gères ce que l'entreprise **achète et possède** : fournisseurs, factures fournisseurs (avec la TVA récupérable, l'AIB subi
et les retenues à la source), le **catalogue d'articles** (partagé avec les ventes), les **stocks** (quantités et valeur),
et le **registre des immobilisations** avec leurs **amortissements**.

## 3. Ton périmètre

| Autorisé | Interdit |
|---|---|
| `crates/ac-achats/`, `ui/src/modules/achats/`, `src-tauri/src/commandes/achats.rs`, ta section de `commandes/mod.rs`, `docs/modules/achats.md`, ADR « Proposé » | Tout le reste. Les règlements fournisseurs appartiennent à A6, les taux à A8. |

**Responsabilité particulière :** `ac-ventes` (A4) dépend de tes modules publics **`catalogue`** et **`stock`**. Ils doivent être livrés
**tôt** (lot 1) et rester **stables** (toute rupture = ADR).

## 4. Connaissances métier indispensables

- **TVA déductible** : la TVA payée sur les achats (facture d'un fournisseur au réel, facture normalisée) se déduit de la TVA collectée.
  On distingue : biens et marchandises (compte 4452), services (4454), immobilisations (4451) — comptes à valider par l'expert.
  Limitation à **90 %** pour le BTP et l'industrie (LF 2026, paramètre `limite_deduction_btp_industrie`, à confirmer) ; carburant déductible si lié à l'activité.
  Un fournisseur au régime **TPS** ne facture pas de TVA : rien à déduire.
- **AIB subi** : le fournisseur ajoute l'AIB à sa facture (1 % / 3 % / 5 %) ; pour toi c'est un **crédit d'impôt** (étiquette `AibSupporte`) imputé plus tard sur l'IS/IBA/TPS par A8.
- **Retenues à la source** que **tu** dois opérer : sur un **loyer** payé (12 % bailleur personne physique, 10 % personne morale — paramètres) et sur une prestation d'un **non-résident** (20 %).
  La facture du bailleur est saisie avec la retenue ; tu dois le loyer net au bailleur et la retenue à la DGI (étiquettes `RetenueLoyer`, `RetenueNonResident`).
- **Stocks** : suivi quantitatif **permanent** (chaque entrée/sortie) et valorisation au **coût moyen unitaire pondéré (CMUP)** recalculé à chaque entrée.
  Comptablement, par défaut **inventaire intermittent** SYSCOHADA : achats en 601/602…, et à la clôture (ou chaque mois si l'utilisateur le choisit)
  écriture de **variation de stock** (31x / 6031) à partir de la valeur d'inventaire. **Choix à faire valider par l'expert-comptable** (issue `fiscal-a-verifier`).
- **Immobilisations** (classe 2) : biens durables (> 1 an, valeur significative — seuil paramétrable). **Amortissement linéaire** par défaut
  (dégressif si autorisé par le CGI : à vérifier), au prorata temporis (jours ou mois : à confirmer). Dotation : 681 / 28x. Cession : sortie de l'actif, plus/moins-value (HAO, comptes 81x/82x).
  Durées usuelles (bâtiments, matériel, véhicules, informatique…) à mettre dans un fichier de paramètres que tu proposes (`parametres-fiscaux/<année>/amortissements.toml` appartient à A8 : demande-lui de le créer avec toi).

## 5. Ce qui existe déjà

- Crate vide (`migrations()` vide, plage 3000–3999). `ac_compta::passer_ecriture`, `CodeFiscal` (A1), paramètres A8.

## 6. Travail à livrer (une PR par lot)

### Lot 1 — Catalogue et API de stock (≤ 5 jours) — **PRIORITÉ pour A4**
- Table `achats_article` : code, désignation, type BIEN_STOCKE | BIEN_NON_STOCKE | SERVICE, unité (pièce, kg, litre, sac, carton, m, m², heure…),
  prix de vente HT, groupe de taxation e-MECeF par défaut (B en réel, E en TPS), compte de vente (701/702/706…), compte d'achat (601/602/604/605…), stock minimum, actif.
- `pub mod catalogue` : `article(conn, code)`, `rechercher(conn, texte, page)`.
- `pub mod stock` : `entrer(tx, session, article, quantite_milliemes, cout_unitaire, reference)`, `sortir(tx, session, article, quantite_milliemes, reference) -> CoutSortie`,
  `disponible(conn, article)`, `ajuster_inventaire(...)`. Stock négatif : **refusé** par défaut, autorisable dans les paramètres (avec alerte).
- Documenter ces signatures dans `docs/modules/achats.md` **avant** de coder et notifier A4 par issue.

### Lot 2 — Fournisseurs (≤ 2 jours)
`achats_fournisseur` (code `FRS-000001`, raison sociale, IFU, régime connu RÉEL|TPS|INCONNU, résident oui/non, adresse, téléphone, e-mail, banque / Mobile Money, actif). Import CSV/XLSX (`ac_documents::lire_tableau`).

### Lot 3 — Factures fournisseurs (≤ 6 jours)
- Saisie : fournisseur, n° de facture du fournisseur (unicité par fournisseur), date, code MECeF de la facture reçue (facultatif mais conseillé),
  lignes (article ou nature de charge, HT, TVA déductible oui/non + catégorie), AIB subi, retenue à opérer (loyer, non-résident).
- Validation → écriture `AC` : charges/achats ou immobilisation débit HT, 445x débit TVA déductible (étiquette + base + taux, avec limitation 90 % si applicable),
  AIB subi débit (étiquette `AibSupporte`), 401 crédit net dû au fournisseur, retenue crédit (étiquette `RetenueLoyer` / `RetenueNonResident`).
  Réception de marchandises stockées → `stock::entrer` dans la même transaction.
- Avoirs fournisseurs. Commandes fournisseurs **facultatives** (V1 : bon de commande simple, transformable en facture).

### Lot 4 — Inventaire et variation de stock (≤ 4 jours)
Inventaire physique (saisie des quantités comptées, écarts, validation), valorisation CMUP, écriture de variation (31x/6031, 32x/6032…) à la date choisie, état des stocks PDF/Excel.

### Lot 5 — Immobilisations et amortissements (≤ 6 jours)
Registre (`achats_immobilisation` : désignation, compte 2x, date d'acquisition et de mise en service, valeur HT, durée, mode, valeur résiduelle), création depuis une facture fournisseur,
plan d'amortissement calculé (tableau année par année, prorata), écriture des dotations (mensuelle ou annuelle, au choix), cession / mise au rebut, tableau des immobilisations pour la DSF (fourni à A9).

### Lot 6 — Interface « Achats et stocks » (≤ 5 jours)
Écrans : fournisseurs, factures fournisseurs (liste, saisie au clavier, statut payé/non payé via le lettrage de `ac-compta`), articles, stocks (alertes de stock minimum), inventaire, immobilisations. Commandes `achats_*`.

## 7. Contrats

- **Tu fournis** : `catalogue`, `stock` (pour A4), tableau des immobilisations et amortissements (pour A9), `achats_importer_fournisseurs`, `achats_importer_articles` (pour A9).
- **Tu consommes** : C3/C4 (A1), `ac-fiscal` (A8), `ac_documents` (A1), C2 (A2).

## 8. Tests exigés (exemples chiffrés)

- Facture fournisseur : 200 000 HT, TVA 18 % = 36 000, AIB subi 1 % = 2 000 → 401 crédité de 238 000 ; 4452 débit 36 000 étiqueté ; écriture équilibrée.
- Entreprise BTP avec limitation 90 % : TVA déductible 32 400, 3 600 en charge.
- Loyer 300 000 à un bailleur personne physique, retenue 12 % = 36 000 → dû au bailleur 264 000.
- CMUP : 10 unités à 1 000 puis 10 à 1 200 → CMUP 1 100 ; sortie de 5 → coût 5 500.
- Stock négatif refusé par défaut.
- Immobilisation 1 200 000 sur 5 ans linéaire, mise en service le 1er juillet → dotation de la 1re année 120 000 (prorata 6 mois), puis 240 000 par an.

## 9. Pièges à éviter

- Ne jamais coder un taux : tout vient des paramètres via `ac-fiscal`.
- Les quantités sont en **millièmes** (`i64`) ; jamais de flottant pour une quantité ou un coût.
- Une facture fournisseur validée est immuable : correction par avoir.
- L'API `stock` doit fonctionner **dans la transaction de l'appelant** (A4), sans en ouvrir une autre.

## 10. Définition de « terminé »

CI verte, couverture ≥ 80 %, `docs/modules/achats.md` (API publique, tables, écritures avec comptes et étiquettes, méthode de stock choisie), CHANGELOG, PR fusionnée.

## 11. Pour démarrer

```powershell
git checkout main; git pull
git checkout -b a5/feat-catalogue-stock
cargo test -p ac-achats
```
