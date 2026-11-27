# PROMPT A1 — Noyau, moteur comptable SYSCOHADA et rendu des documents

> **Lis d'abord, dans cet ordre :** `AGENTS.md` (règles R1 à R14 et workflow Git, **non négociables**), ce fichier,
> `docs/architecture.md`, `docs/contrats.md`, `docs/fiscalite/regles-fiscales-benin.md`, `docs/adr/0002-montants-entiers-fcfa.md`.
> Tu ne connais ni le projet ni le droit béninois : **ne devine rien**. Question ou blocage : issue GitHub avec les libellés `agent:a1` et `question`.

## 1. Le projet en une minute

AMAZONE COMP est un logiciel de comptabilité **gratuit et open source (AGPL-3.0)** pour les **PME du Bénin**, sur
**Windows 10+ 64 bits**, utilisable **sans comptable**. Il fonctionne **hors ligne** ; Internet ne sert qu'au compte,
à la licence gratuite de 6 mois, aux mises à jour et à la sauvegarde. Pile : **Tauri 2 + Rust** (métier) +
**React/TypeScript** (interface) + **SQLite chiffrée**. Monnaie : **FCFA entiers**. Référentiel : **SYSCOHADA révisé**.
Dix agents travaillent en parallèle, chacun dans son périmètre ; le **chef de projet** relit, teste et fusionne chaque PR.

## 2. Ta mission

Tu es le **cœur** du système. Tous les autres modules dépendent de toi : ventes, achats, trésorerie, paie et fiscalité
produisent des écritures **par tes fonctions**, et les états financiers lisent **tes** tables. Ta priorité absolue :
**justesse, immuabilité, lisibilité des erreurs, performance**. Tu livres vite les fonctions dont les autres ont besoin
(`passer_ecriture`, lectures), puis tu complètes.

## 3. Ton périmètre

| Autorisé | Interdit |
|---|---|
| `crates/ac-noyau/`, `crates/ac-compta/`, `crates/ac-documents/`, `ui/src/modules/comptabilite/`, `src-tauri/src/commandes/compta.rs` ; ta section de `src-tauri/src/commandes/mod.rs` ; `docs/modules/compta.md` ; nouveaux ADR (statut « Proposé ») | Tout le reste. Besoin ailleurs → issue à l'agent propriétaire. |

## 4. Connaissances métier indispensables

- **Partie double** : chaque écriture a au moins 2 lignes ; Σ débits = Σ crédits, au franc près.
- **SYSCOHADA révisé** (Acte uniforme OHADA du 26 janvier 2017) : 9 classes (1 ressources durables, 2 actif immobilisé,
  3 stocks, 4 tiers, 5 trésorerie, 6 charges, 7 produits, 8 autres charges/produits et impôt sur le résultat,
  9 engagements/analytique). Comptes à 2 chiffres minimum ; les sous-comptes allongent le numéro (411 → 4111 → 41110001).
- **Intangibilité** : une écriture **validée** ne se modifie ni ne se supprime. Correction = **contre-passation**
  (écriture inverse datée, qui référence l'originale) puis nouvelle écriture correcte. Les **brouillons** (non validés) sont libres.
- **Numérotation** : continue et **sans trou** par journal et par exercice, attribuée **à la validation**.
- **Exercice** : 12 mois, en général 1er janvier – 31 décembre. Périodes mensuelles verrouillables. Une fois l'exercice
  **clôturé**, plus aucune écriture n'y entre.
- **Lettrage** : rapprocher sur un compte de tiers (411 client, 401 fournisseur…) les lignes qui se compensent
  (facture ↔ règlement). Code de lettrage `AA`, `AB`, … ; lettrage partiel interdit (somme nulle exigée), on suit le
  « reste à payer » par les lignes non lettrées.
- **Clôture** : détermination du résultat (produits classes 7 et 8 − charges classes 6 et 8), virement au compte
  **131 Résultat net : bénéfice** ou **139 Résultat net : perte** ; reprise des soldes des classes 1 à 5 dans le nouvel
  exercice en **à-nouveaux** (journal `AN`). L'affectation du résultat (réserves, dividendes) est une OD saisie ensuite.
- **Étiquettes fiscales** (`CodeFiscal`) : tu les **stockes** et les **agrèges** ; tu ne calcules aucune taxe (c'est A8).

## 5. Ce qui existe déjà (à lire avant de coder)

- `ac-noyau` : `Montant` (i64 borné, arrondis), `Taux` (décimal), `Periode`, `Ifu`, `parametres::Valeur`. Testé.
- `ac-compta` : `NumeroCompte`, `CodeFiscal`, `BrouillonEcriture`, `LigneBrouillon`, `verifier_brouillon`,
  `passer_ecriture` minimal, migration `1000_compta_socle.sql` (tables `compta_compte`, `compta_ecriture`, `compta_ligne`
  + déclencheurs d'immuabilité). Tu **ajoutes** des migrations 1001, 1002… (ne modifie jamais 1000 une fois fusionnée
  dans `main` ; tant qu'aucune base réelle n'existe, le chef de projet peut accepter une modification explicitement signalée).
- `ac-documents` : contrat `Document` / `Bloc` / `Cellule`, fonctions `rendre_pdf` / `rendre_xlsx` non implémentées.

## 6. Travail à livrer (une PR par lot, dans cet ordre)

### Lot 1 — Noyau finalisé (≤ 3 jours)
- Ajouter `Quantite` (millièmes, i64) avec `Montant::multiplier_quantite(q, Arrondi)`.
- `Montant::repartir(parts: &[i64]) -> Vec<Montant>` (répartition sans perte d'un franc : la différence d'arrondi va à la plus grande part).
- Tests de propriété (`proptest` en dev-dépendance, justifier en PR) : `a + b - b == a`, répartition conserve le total.

### Lot 2 — Plan de comptes SYSCOHADA (≤ 5 jours)
- Fichier de données `crates/ac-compta/donnees/plan_syscohada.csv` : **tous** les comptes à 2 et 3 chiffres du plan
  SYSCOHADA révisé + les comptes à 4 chiffres usuels d'une PME (dont 4431, 4432, 4441, 4449, 4451, 4452, 4454, 447x, 521x,
  55x monnaie électronique, 571x, 585). Colonnes : `numero;libelle;classe;nature(BILAN|GESTION|HAO|ENGAGEMENT);sens_normal(D|C|M);lettrable(0|1);tiers_obligatoire(0|1)`.
  Indique ta source exacte dans l'en-tête du fichier (texte de l'Acte uniforme, page/annexe). Libellés **officiels**.
- Migration : colonnes supplémentaires de `compta_compte` (classe, nature, sens, lettrable, tiers_obligatoire, systeme : NORMAL/SMT/TOUS).
- `plan::initialiser(tx, systeme)` charge le plan à la création du dossier ; `plan::creer_sous_compte(tx, session, parent, numero, libelle)`
  (un sous-compte doit commencer par le numéro du parent ; interdit si le parent a déjà des mouvements directs — à documenter).

### Lot 3 — Exercices, périodes, journaux (≤ 4 jours)
- Tables `compta_exercice (id, debut, fin, statut OUVERT|CLOTURE)`, `compta_periode (exercice_id, mois, statut OUVERTE|VERROUILLEE)`,
  `compta_journal (code, libelle, type VENTES|ACHATS|TRESORERIE|PAIE|OD|AN|STOCKS, compte_tresorerie NULL)`.
- Journaux créés par défaut : `VE`, `AC`, `CA`, `OD`, `AN`, `PA`, `ST` ; les journaux `BQx` et `MMx` sont créés par A6 via ta fonction `journal::creer`.

### Lot 4 — `passer_ecriture` complète + brouillons + contre-passation (≤ 6 jours) — **PRIORITÉ pour A4, A5, A6, A7**
- Contrôles supplémentaires : date dans un exercice ouvert et une période ouverte ; journal existant ; comptes existants,
  actifs et mouvementables ; `tiers` obligatoire si le compte l'exige ; `code_fiscal` ⇒ `base_fiscale` renseignée
  (sauf codes sans base, à lister) ; libellé ≤ 200 caractères.
- Numérotation sans trou (`compta_sequence (journal, exercice, dernier)` mise à jour dans la même transaction).
- `contre_passer(tx, session, id, date, motif) -> EcritureId` : lignes inversées, `contrepasse_id`, audit `CONTREPASSER`.
  Impossible deux fois sur la même écriture.
- Brouillons d'OD : tables `compta_brouillon*` modifiables, `valider_brouillon` → `passer_ecriture`.
- **Mesure** : 10 000 écritures de 3 lignes passées en < 5 s sur base en mémoire (test `#[ignore]` de performance + résultat dans la PR).

### Lot 5 — Lectures (≤ 5 jours) — **PRIORITÉ pour A8, A9**
- `solde_compte(conn, prefixe, periode)`, `balance(conn, periode, niveau_chiffres) -> Vec<LigneBalance>` (ouverture, mouvements, soldes),
  `grand_livre(conn, compte, periode, page) -> Page<LigneGrandLivre>` (solde progressif), `journal(conn, code, periode, page)`,
  `lignes_non_lettrees(conn, compte, tiers) -> Vec<LigneOuverte>` (utilisé par A6 pour les règlements),
  `agregats_fiscaux(conn, periode) -> Vec<AgregatFiscal { code, taux, base, montant_debit, montant_credit }>` (utilisé par A8).
- Index nécessaires ; balance de 50 000 écritures < 1 s (test de performance).

### Lot 6 — Lettrage (≤ 3 jours)
- `lettrer(tx, session, lignes: &[i64]) -> CodeLettrage` (même compte, même tiers, somme nulle), `delettrer` (interdit sur exercice clôturé). Seule la colonne `lettrage` peut changer (le déclencheur existant le permet déjà).

### Lot 7 — Clôture et à-nouveaux (≤ 5 jours)
- Assistant de clôture en étapes, chaque étape testée : contrôles (balance équilibrée, comptes 47 d'attente soldés, brouillons restants, périodes verrouillées),
  écriture de résultat (131 / 139), création de l'exercice suivant, à-nouveaux classes 1 à 5 (journal `AN`), verrouillage.
  **Irréversible** : double confirmation côté interface et audit.

### Lot 8 — `ac-documents` (≤ 6 jours) — **PRIORITÉ pour A4 (facture PDF) et A8 (déclarations)**
- ADR « Proposé » choisissant les bibliothèques : `rust_xlsxwriter` (Excel, MIT/Apache) et une bibliothèque PDF pure Rust
  (`printpdf` ou `genpdf`) ; police libre embarquée (licence OFL, ex. Noto Sans ou Liberation Sans, sous-ensemble latin) : budget **< 1 Mo** ajouté au binaire.
- Rendu PDF A4 portrait : en-tête entreprise, titre, blocs, tableaux paginés avec répétition des en-têtes, « Page x / y », pied de page.
- Rendu Excel : une feuille par tableau, montants en **nombres** (format `# ##0`), dates en dates.
- `lire_tableau(octets, nom_fichier) -> Vec<Vec<String>>` : CSV (séparateur `;` ou `,` détecté, UTF-8 ou Windows-1252) et XLSX
  (`calamine`, MIT) ; refuse > 20 Mo ; aucune formule évaluée. Utilisé par A4, A5, A6, A7, A9 pour les imports.

### Lot 9 — Interface « Comptabilité » + commandes (≤ 6 jours)
- `ui/src/modules/comptabilite/` : plan de comptes (recherche, ajout de sous-compte), saisie d'OD (grille clavier, équilibre affiché
  **calculé par Rust**, boutons « Enregistrer le brouillon » / « Valider »), consultation des journaux, lettrage manuel, assistant de clôture.
  Composants du design system uniquement (`ui/src/design/composants.tsx`) — demande à A3 ceux qui manquent.
- Commandes `compta_*` dans `src-tauri/src/commandes/compta.rs`, enregistrées dans ta section de `commandes/mod.rs`.

## 7. Contrats

- **Tu fournis** : C1 (noyau), C3 (écritures), C4 (étiquettes), C5 (plan), C7 (documents). Toute modification de signature publique = ADR.
- **Tu consommes** : C2 (`ac-donnees` : `Transaction`, `Session`, `audit::journaliser`).

## 8. Tests exigés (exemples chiffrés obligatoires)

- Facture : 411 D 118 000 / 7011 C 100 000 / 4431 C 18 000 → acceptée, numéro VE-2027-000001.
- Déséquilibre d'un franc → `Desequilibre { debit: 118 000, credit: 117 999 }`.
- Contre-passation de l'écriture précédente → balance du compte 411 revenue à 0 ; seconde contre-passation refusée.
- Écriture datée dans une période verrouillée → `PeriodeFermee`.
- Rollback de transaction au milieu → la séquence de numérotation n'a pas de trou au prochain essai.
- Clôture : produits 1 000 000, charges 700 000 → 131 crédité de 300 000 ; à-nouveaux équilibrés ; aucune ligne de classe 6/7 en `AN`.
- Lettrage : facture 118 000 + règlement 118 000 → lettré ; 118 000 + 100 000 → refusé.
- Couverture ≥ 80 % sur tes trois crates.

## 9. Pièges à éviter

- Ne jamais additionner des `i64` bruts hors `Montant` ; ne jamais convertir en flottant pour « arrondir ».
- Ne pas mettre de logique fiscale dans `ac-compta` (taux, seuils) : c'est A8.
- Ne pas exposer `rusqlite` dans tes signatures publiques autrement que via `ac_donnees::Transaction` / `Connexion`.
- Messages d'erreur pour un non-comptable : « Le total des débits (118 000 FCFA) ne correspond pas au total des crédits (117 999 FCFA). »
- Pas de requête SQL construite par concaténation de texte : toujours des paramètres `?1`.

## 10. Définition de « terminé » (pour chaque lot)

CI verte, couverture ≥ 80 %, `docs/modules/compta.md` à jour (tables, fonctions, exemples), `CHANGELOG.md` complété,
PR relue et fusionnée par le chef de projet.

## 11. Pour démarrer

```powershell
git checkout main; git pull
git checkout -b a1/feat-noyau-quantite
cargo test -p ac-noyau -p ac-compta
```
