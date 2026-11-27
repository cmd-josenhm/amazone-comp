# PROMPT A8 — Fiscalité : paramètres, TVA, AIB, IS/IBA, TPS, retenues, calendrier, déclarations PDF/Excel

> **Lis d'abord, dans cet ordre :** `AGENTS.md` (règles R1 à R14, **non négociables**, en particulier **R2** et **R14**), ce fichier,
> `docs/fiscalite/regles-fiscales-benin.md` (en entier), `docs/fiscalite/sources.md`, `parametres-fiscaux/README.md`, `docs/contrats.md` (C4, C10).
> Tu ne connais pas le droit béninois : **ta première tâche est de le lire dans le texte**. Question : issue GitHub `agent:a8` + `question`.

## 1. Le projet en une minute

AMAZONE COMP est un logiciel de comptabilité **gratuit et open source** pour les **PME du Bénin**, Windows 10+ 64 bits, utilisable
**sans comptable**, **hors ligne d'abord**. Pile : **Tauri 2 + Rust** + **React/TypeScript** + **SQLite chiffrée**. **FCFA entiers**.
Régimes : **réel normal, réel simplifié, TPS**. Le logiciel **prépare** les déclarations (PDF + Excel) ; l'utilisateur les dépose sur **e-services** de la DGI.
Dix agents en parallèle ; le **chef de projet** valide chaque PR.

## 2. Ta mission

Tu es le **gardien de la loi fiscale** dans le logiciel. Tu tiens les **paramètres** de chaque année (taux, seuils, dates, avec leur source),
tu fournis les **fonctions de calcul** utilisées par les ventes, les achats et la paie, et tu produis les **déclarations** et le **calendrier fiscal**.
Une erreur chez toi = une pénalité pour l'utilisateur : **précision, sources, tests chiffrés**.

## 3. Ton périmètre

| Autorisé | Interdit |
|---|---|
| `crates/ac-fiscal/`, `ui/src/modules/fiscal/`, `src-tauri/src/commandes/fiscal.rs`, ta section de `commandes/mod.rs`, **`parametres-fiscaux/`** (sauf `social.toml`, propriété de A7), **`docs/fiscalite/`**, `docs/modules/fiscal.md`, ADR « Proposé » | Tout le reste. Tu ne lis **jamais** les tables des autres modules : tes déclarations se construisent avec `ac_compta::agregats_fiscaux` (lignes étiquetées, contrat C4). |

## 4. Connaissances métier — ce que tu dois établir toi-même

`docs/fiscalite/regles-fiscales-benin.md` résume l'essentiel ; les **valeurs provisoires** sont dans `parametres-fiscaux/2026/` avec `verifie = false`.
Elles viennent de sources secondaires. **Ta première mission est de les confronter au texte officiel** :
- **CGI du Bénin, édition 2026** (copie PDF : voir `docs/fiscalite/sources.md`) ;
- **Loi de finances 2026** (adoptée le 4 décembre 2025) et, **dès sa publication (décembre 2026)**, la **loi de finances 2027** → dossier `parametres-fiscaux/2027/` ;
- **Doctrine fiscale DGI** (2023), communiqués DGI (ex. AIB à 5 % pour IFU désactivé depuis le 7 septembre 2026).
Pour chaque valeur : article précis dans `source`, `note` en cas de doute. **Tu ne passes jamais `verifie = true`** : c'est le chef de projet, après avis de l'expert-comptable.

## 5. Ce qui existe déjà

- `parametres-fiscaux/2026/` : `tva`, `aib`, `impot-resultat`, `tps`, `retenues`, `penalites`, `calendrier`, `emecef` (+ `social` de A7). Test qui vérifie que chaque valeur a `source` et `verifie`.
- `crates/ac-fiscal` : `Regime`, `ParametresAnnee` (TVA et TPS typés seulement), `fichiers_embarques`, `tva::depuis_ht`, test de chargement.

## 6. Travail à livrer (une PR par lot)

### Lot 1 — Vérification des sources (≤ 5 jours) — **PRIORITÉ**
- Lire le CGI 2026 et la LF 2026 ; corriger les `source` avec les **articles exacts** ; noter les écarts dans `docs/fiscalite/sources.md`.
- Ouvrir une issue `fiscal-a-verifier` pour **chacune** des 10 contradictions listées dans `docs/fiscalite/sources.md` + celles que tu découvres.
- Documenter dans `docs/fiscalite/regles-fiscales-benin.md` (ta propriété désormais) : définition du réel simplifié, exigibilité de la TVA, règle d'arrondi officielle, mode de calcul des acomptes IS et TPS.

### Lot 2 — Paramètres entièrement typés (≤ 3 jours) — **PRIORITÉ pour A4/A5**
- Tous les fichiers (sauf `social.toml`) en structs typées dans `ParametresAnnee` ; sélection de l'année **par la date de l'opération** ; erreur claire si l'année manque (« Mettez à jour le logiciel pour l'année 2028 »).
- Fichier `amortissements.toml` (durées usuelles, modes autorisés) avec A5.

### Lot 3 — Fonctions de calcul pures (≤ 6 jours) — **PRIORITÉ pour A4/A5**
API stable, documentée, chaque fonction avec exemples chiffrés en test :
- `tva::ventiler_ligne` (HT→TVA→TTC et TTC→HT) selon le groupe de taxation e-MECeF et le régime ; **règle d'arrondi unique** documentée.
- `aib::calculer(base_ht, CategorieAib, client: ProfilClientAib) -> Montant` (avec IFU / sans IFU / IFU désactivé / marché public / exonéré) et code e-MECeF associé.
- `tva::retenue_a_la_source(...)` (client État/désigné ; fournisseur TPS 100 % / réel 40 %).
- `retenues::loyer`, `retenues::non_resident`.
- `impot_resultat::calculer(resultat_fiscal, produits_encaissables, profil) -> DetailImpot { impot_calcule, minimum, du, acomptes, aib_imputable, solde }`.
- `tps::calculer(ca, profil)`, acomptes et solde.
- `regime::suggerer(forme_juridique, ca_estime) -> Suggestion` (utilisé par l'assistant de démarrage de A3) et alerte de **dépassement du seuil** TPS.

### Lot 4 — Déclaration de TVA mensuelle (≤ 5 jours)
À partir de `agregats_fiscaux(periode)` : CA taxable, exonéré, export ; TVA collectée ; TVA déductible par catégorie (avec limitation 90 % si applicable) ; retenues de TVA subies ; crédit du mois précédent ;
TVA due ou crédit à reporter. Rendu **PDF** (lisible) et **Excel** reproduisant le formulaire DGI (demande au chef de projet le **modèle officiel** d'e-services par issue ; en attendant, mise en page complète et versionnée `modeles/tva/2026.json`).
Écriture de liquidation proposée (443 / 445 / 4441 ou 4449) validée par l'utilisateur.

### Lot 5 — AIB, retenues, ITS/VPS (≤ 4 jours)
Déclaration mensuelle AIB (état détaillé par client : IFU, nom, base, taux, AIB), retenues (loyers, non-résidents), récapitulatif ITS/VPS (lignes de paie étiquetées) — PDF + Excel.

### Lot 6 — Annuel : IS / IBA / TPS (≤ 6 jours)
Tableau de passage du résultat comptable au **résultat fiscal** (réintégrations / déductions saisies avec aide et liste type), minimum de perception, imputation de l'AIB subi et des acomptes, solde ; acomptes de l'année suivante ;
déclaration TPS annuelle ; écritures d'impôt (891 / 441, 895). Fournit à A9 les montants pour la DSF.

### Lot 7 — Calendrier et alertes (≤ 3 jours)
Échéances de l'année selon le régime (depuis `calendrier.toml`), état de chaque échéance (préparée / exportée / marquée déposée par l'utilisateur), alertes J-7 et J-1, simulation des pénalités de retard (`penalites.toml`), commande `fiscal_prochaines_echeances` pour le tableau de bord (A9).

### Lot 8 — Interface « Fiscalité » (≤ 4 jours)
Calendrier, préparation guidée de chaque déclaration (période → contrôles → aperçu → export PDF/Excel → « Marquer comme déposée sur e-services » avec date et référence de dépôt), historique, page « Règles et sources » listant chaque paramètre, sa source et son état de vérification.

## 7. Contrats

- **Tu fournis** : C10 (paramètres), les fonctions de calcul du lot 3 (A4, A5, A7), `regime::suggerer` (A3), `fiscal_prochaines_echeances` (A9), données fiscales pour la DSF (A9).
- **Tu consommes** : C4 `agregats_fiscaux` (A1), `ac_documents` (A1), C2 (A2) avec la permission `Declarer`.

## 8. Tests exigés (exemples chiffrés — les tests lisent les paramètres)

- TVA du mois : ventes taxables 10 000 000 HT (TVA 1 800 000), achats 4 000 000 HT (TVA 720 000), crédit antérieur 100 000 → TVA due 980 000.
- Même mois, entreprise BTP avec limitation 90 % : déductible 648 000 → TVA due 1 052 000.
- Mois avec TVA déductible > collectée → crédit à reporter, aucune TVA due.
- AIB : base 1 000 000, client avec IFU (biens) → 10 000 ; sans IFU → 50 000.
- IS : résultat fiscal 2 000 000 à 30 % = 600 000 ; CA 100 000 000, minimum 1 % = 1 000 000 → impôt dû 1 000 000. Résultat déficitaire, CA 10 000 000 → minimum 250 000 (plancher).
- TPS : CA 8 000 000 à 5 % = 400 000 ; CA 100 000 → minimum 10 000.
- Une valeur sans `source` dans un TOML → le test de cohérence échoue.

## 9. Pièges à éviter

- **R14** : jamais de règle inventée. Une incertitude = paramètre + `note` + issue.
- Ne jamais modifier les valeurs d'une année passée publiée : créer l'année suivante.
- Les déclarations sont des **aides** : toujours afficher « Vérifiez avant dépôt sur e-services ».
- Pas de flottant, pas de taux en dur, arrondis explicites et documentés.

## 10. Définition de « terminé »

CI verte, couverture ≥ 80 %, `docs/modules/fiscal.md` + `docs/fiscalite/` à jour, chaque formule documentée avec sa source, CHANGELOG, PR fusionnée.

## 11. Pour démarrer

```powershell
git checkout main; git pull
git checkout -b a8/docs-sources-cgi-2026
cargo test -p ac-fiscal
```
