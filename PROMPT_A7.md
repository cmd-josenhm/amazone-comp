# PROMPT A7 — Paie : salariés, bulletins, ITS, VPS, CNSS, déclarations sociales

> **Lis d'abord, dans cet ordre :** `AGENTS.md` (règles R1 à R14, **non négociables**), ce fichier, `docs/architecture.md`,
> `docs/contrats.md` (C3, C4, C10), `docs/fiscalite/regles-fiscales-benin.md` (§4.6, §4.7), `parametres-fiscaux/2026/social.toml`.
> Tu ne connais ni le projet ni le droit béninois : **ne devine rien**. Question : issue GitHub `agent:a7` + `question`.

## 1. Le projet en une minute

AMAZONE COMP est un logiciel de comptabilité **gratuit et open source** pour les **PME du Bénin**, Windows 10+ 64 bits, utilisable
**sans comptable**, **hors ligne d'abord**. Pile : **Tauri 2 + Rust** + **React/TypeScript** + **SQLite chiffrée**. **FCFA entiers**.
Dix agents en parallèle ; le **chef de projet** valide chaque PR.

## 2. Ta mission

Tu permets à une PME de **payer ses salariés correctement et légalement** : fiches salariés, éléments variables du mois,
**bulletins de paie**, calcul de l'**ITS**, de la **CNSS** et du **VPS**, écriture comptable de paie, états à déclarer
(DGI et CNSS) et livre de paie. Les salaires sont **confidentiels** : seuls les rôles `Administrateur` et `Paie` y accèdent.

## 3. Ton périmètre

| Autorisé | Interdit |
|---|---|
| `crates/ac-paie/`, `ui/src/modules/paie/`, `src-tauri/src/commandes/paie.rs`, ta section de `commandes/mod.rs`, **`parametres-fiscaux/<année>/social.toml`**, `docs/modules/paie.md`, ADR « Proposé » | Tout le reste. Le paiement effectif des salaires (banque, Mobile Money, caisse) est fait par A6 à partir de ton état. |

## 4. Connaissances métier indispensables (Bénin) — chiffres dans `social.toml`, **tous à vérifier**

- **ITS** (impôt sur les traitements et salaires) : barème **mensuel progressif par tranches** (0 % jusqu'à un premier seuil, puis 10 %, 15 %, 19 %, 30 %),
  retenu sur le salaire par l'employeur et reversé à la DGI le **10 du mois suivant**. Contradiction ouverte sur le **premier seuil (60 000 ou 50 000 FCFA)**
  et sur la **base** (brut ou brut moins CNSS salarié ?). **Redevance audiovisuelle** (ORTB/SRTV) sur certains mois : mécanisme à confirmer.
- **CNSS** : prestations familiales (employeur), pension (employeur + salarié), risques professionnels (employeur, **1 % à 4 % selon l'activité** : taux fixé pour chaque entreprise dans ses paramètres).
  **Plafond** de l'assiette : contradiction ouverte (aucun / 600 000 FCFA). Périodicité et date limite : à confirmer (mensuelle ou trimestrielle selon l'effectif).
- **VPS** (versement patronal sur salaires) : charge de l'employeur, % des salaires bruts ; taux réduit pour l'enseignement privé ; **non dû** par les entreprises à la TPS ; exonérations temporaires possibles (entreprises nouvelles, premier emploi) à paramétrer.
- **SMIG** : salaire minimum mensuel (52 000 FCFA selon nos sources, à confirmer). Alerte si le salaire de base d'un temps plein est inférieur.
- **Code du travail** du Bénin : mentions obligatoires du bulletin, heures supplémentaires (taux de majoration), congés payés — **à documenter avec sources** avant de coder
  (ajouter les paramètres dans `social.toml` avec `source` et `verifie = false`).
- Écriture de paie type (comptes à valider) : `661` débit brut / `422` crédit net à payer / `431` crédit CNSS salarié / `447` crédit ITS (et redevance) ;
  `664` débit charges patronales CNSS / `431` crédit ; `641x` débit VPS / `447` crédit. Étiquettes `Its`, `Vps`, `CnssSalarie`, `CnssEmployeur`, `RedevanceAudiovisuelle` avec leurs bases.

## 5. Ce qui existe déjà

- `crates/ac-paie/src/its.rs` : `Tranche`, `bareme_2026()` (lu dans `social.toml`), `calculer_its(brut_imposable, bareme)` avec exemples testés
  (100 000 → 4 000 ; 300 000 → 33 500 ; 1 000 000 → 221 500 avec le barème **non vérifié**).
- `social.toml` 2026 complet mais **non vérifié** (`verifie = false`).

## 6. Travail à livrer (une PR par lot)

### Lot 1 — Paramètres sociaux typés (≤ 3 jours)
Struct `ParametresSociaux` couvrant tout `social.toml`, chargée par année (date de la paie), test « chaque valeur a une source ». Ouvrir les issues `fiscal-a-verifier`
pour : premier seuil ITS, base ITS, plafond CNSS, redevance audiovisuelle, périodicité CNSS, SMIG, majorations d'heures supplémentaires.

### Lot 2 — Calculs purs (≤ 5 jours)
Fonctions sans base de données, une par cotisation : `cnss_salarie`, `cnss_employeur` (détail par branche), `vps`, `its` (existe), `redevance`, `net_a_payer`.
Un `CalculBulletin` complet à partir d'un `EntreeBulletin` (salaire de base, heures, primes imposables / non imposables, avantages en nature, absences, avances) → `Bulletin` (toutes les lignes, bases, taux, montants, arrondis documentés).

### Lot 3 — Salariés et rubriques (≤ 4 jours)
`paie_salarie` (code `SAL-000001`, identité, n° CNSS, IFU éventuel, date d'embauche, contrat CDI/CDD/stage, catégorie, salaire de base, mode de paiement, actif),
`paie_rubrique` (catalogue paramétrable : gain / retenue, imposable ITS oui/non, soumis CNSS oui/non), éléments variables du mois. Import CSV/XLSX des salariés.

### Lot 4 — Période de paie et écriture (≤ 4 jours)
Ouverture d'un mois, calcul de tous les bulletins, contrôle, **validation** (verrouillage + écriture `PA` étiquetée + audit), correction après validation par **bulletin rectificatif** (jamais de modification).
Commande `paie_etat_paiement` (net par salarié + mode) utilisée par A6 pour payer.

### Lot 5 — Documents et déclarations (≤ 5 jours)
Bulletin PDF (mentions légales), livre de paie (PDF/Excel), état mensuel ITS/VPS pour la DGI, déclaration CNSS (format Excel attendu par la CNSS : **demande un modèle officiel** au chef de projet par issue), état annuel des salaires.

### Lot 6 — Interface « Paie » (≤ 5 jours)
Salariés, saisie des variables du mois en grille, aperçu d'un bulletin (calcul Rust en direct), validation du mois, documents. Accès refusé (message clair) aux rôles sans `GererPaie`. Commandes `paie_*`.

## 7. Contrats

- **Tu fournis** : `paie_etat_paiement` (A6), lignes étiquetées pour A8 (ITS/VPS), `paie_importer_salaries` (A9), états de paie pour la DSF (A9).
- **Tu consommes** : C3/C4 (A1), `ac_documents` (A1), C2 (A2) avec la permission `GererPaie`.

## 8. Tests exigés (exemples chiffrés — valeurs lues dans les paramètres, jamais codées)

- Salaire brut 300 000 : CNSS salarié 3,6 % = 10 800 ; ITS selon la base retenue (documenter les deux hypothèses tant que la question est ouverte, un test par hypothèse, l'une marquée `#[ignore]`) ;
  charges patronales PF 9 % = 27 000, pension 6,4 % = 19 200, RP au taux de l'entreprise (ex. 1 % = 3 000) ; VPS 4 % = 12 000.
- Entreprise à la TPS → VPS = 0. Enseignement privé → VPS 2 %.
- Salaire < SMIG à temps plein → alerte (pas de blocage).
- Écriture de paie équilibrée, étiquetée, et verrouillage du mois après validation.
- Rôle `Comptable` → `NonAutorise(GererPaie)`.

## 9. Pièges à éviter

- Ne pas appliquer d'abattement ou de quotient familial sur l'ITS sans source écrite (nos sources indiquent : aucun).
- Les arrondis : par ligne de cotisation, au franc ; règle documentée et identique partout.
- Aucune donnée de salaire dans les journaux techniques ou les messages d'erreur.

## 10. Définition de « terminé »

CI verte, couverture ≥ 80 %, `docs/modules/paie.md` (formules, hypothèses ouvertes, écritures), CHANGELOG, PR fusionnée.

## 11. Pour démarrer

```powershell
git checkout main; git pull
git checkout -b a7/feat-parametres-sociaux
cargo test -p ac-paie
```
