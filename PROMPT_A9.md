# PROMPT A9 — États financiers SYSCOHADA, DSF, tableau de bord, reprise de données

> **Lis d'abord, dans cet ordre :** `AGENTS.md` (règles R1 à R14, **non négociables**), ce fichier, `docs/architecture.md` (§3, §10),
> `docs/contrats.md` (C3, C5, C7), `docs/fiscalite/regles-fiscales-benin.md` (§2, §3).
> Tu ne connais ni le projet ni le SYSCOHADA : **ne devine rien**. Question : issue GitHub `agent:a9` + `question`.

## 1. Le projet en une minute

AMAZONE COMP est un logiciel de comptabilité **gratuit et open source** pour les **PME du Bénin**, Windows 10+ 64 bits, utilisable
**sans comptable**, **hors ligne d'abord**. Pile : **Tauri 2 + Rust** + **React/TypeScript** + **SQLite chiffrée**. **FCFA entiers**.
Référentiel : **SYSCOHADA révisé**. Dix agents en parallèle ; le **chef de projet** valide chaque PR.

## 2. Ta mission

Tu transformes les écritures en **informations** : balance, grand livre, journaux, **états financiers SYSCOHADA** (bilan, compte de résultat,
tableau des flux de trésorerie, notes), **DSF** (liasse fiscale annuelle), **tableau de bord** clair pour un dirigeant non comptable,
et l'**assistant de reprise de données** (démarrer avec l'historique d'une autre comptabilité ou d'un cahier Excel).

## 3. Ton périmètre

| Autorisé | Interdit |
|---|---|
| `crates/ac-etats/`, `ui/src/modules/etats/`, `ui/src/modules/tableau-de-bord/`, `src-tauri/src/commandes/etats.rs`, ta section de `commandes/mod.rs`, `docs/modules/etats.md`, ADR « Proposé » | Tout le reste. Tu **lis** la comptabilité uniquement via les fonctions de A1 (`balance`, `grand_livre`, `solde_compte`, `agregats_fiscaux`), jamais par SQL direct sur les tables `compta_*`. |

## 4. Connaissances métier indispensables

- **Système normal** SYSCOHADA : **Bilan** (actif / passif, rubriques codées par références `AD`, `AE`… `BA`… `CA`… `DA`…), **Compte de résultat**
  (références `TA`… `RA`…, soldes intermédiaires : marge commerciale `XA`, valeur ajoutée `XB`, excédent brut d'exploitation `XC`, résultat d'exploitation `XD`, résultat financier `XE`,
  résultat des activités ordinaires `XF`, résultat HAO `XG`, résultat net `XI`), **TFT** (flux opérationnels, d'investissement, de financement), **Notes annexes** (au moins les notes obligatoires).
  Les **tables de correspondance comptes → rubriques** viennent de l'Acte uniforme OHADA (2017) : tu les mets dans un fichier de données versionné (`crates/ac-etats/donnees/correspondances_syscohada.csv`) avec la source exacte.
- **Système minimal de trésorerie (SMT)** : pour les très petites entités ; état des recettes et dépenses, situation de trésorerie, bilan simplifié. Seuils d'éligibilité : à vérifier (A8).
- **DSF** (déclaration statistique et fiscale) : liasse annuelle déposée avec la déclaration de résultat. Format **béninois** exact à obtenir (modèle e-services / DGI) : **demande-le au chef de projet par issue** ; en attendant, produire les états du système normal au format SYSCOHADA en Excel.
- Contrôles de cohérence obligatoires : total actif = total passif ; résultat du bilan = résultat du compte de résultat ; variation de trésorerie du TFT = variation des comptes de classe 5.

## 5. Ce qui existe déjà

Crate vide (plage de migrations 7000–7999). A1 livre les lectures (lot 5 de A1) et `ac_documents` ; tant que ce n'est pas fusionné, travaille sur tes
correspondances et tes calculs à partir d'une **balance fictive** (struct en mémoire) : tes fonctions prennent une balance en entrée, pas une base.

## 6. Travail à livrer (une PR par lot)

### Lot 1 — Correspondances et moteur d'états (≤ 6 jours)
`correspondances_syscohada.csv` (compte ou plage → rubrique, sens, signe, solde débiteur/créditeur selon la rubrique), `etats::bilan(balance) -> Bilan`,
`etats::compte_resultat(balance) -> CompteResultat`, contrôles de cohérence avec messages clairs (« Le compte 4711 n'est rattaché à aucune rubrique »).
Colonnes exercice N et N-1.

### Lot 2 — Balance, grand livre, journaux (≤ 3 jours)
États imprimables PDF/Excel à partir des lectures de A1 : balance générale (2, 3, 4 chiffres ou détaillée), balances auxiliaires clients / fournisseurs (par tiers), grand livre, journaux, centralisateur. Pagination, totaux par page.

### Lot 3 — TFT et notes (≤ 5 jours)
Tableau des flux de trésorerie selon le modèle SYSCOHADA révisé ; notes annexes obligatoires (au minimum : méthodes, immobilisations et amortissements — données de A5 —, stocks, créances et dettes par échéance, capitaux propres, impôts — données de A8).

### Lot 4 — SMT (≤ 3 jours)
États du système minimal de trésorerie pour les dossiers concernés (`systeme_comptable = 'SMT'`).

### Lot 5 — DSF (≤ 5 jours)
Assemblage de la liasse : états financiers + tableaux fiscaux de A8 + informations générales de l'entreprise ; export Excel (modèle officiel dès qu'il est fourni, sinon modèle SYSCOHADA) + PDF ; liste de contrôle avant dépôt.

### Lot 6 — Tableau de bord (≤ 4 jours)
Accueil (`/`) en langage simple : **trésorerie disponible** par compte (A6), **chiffre d'affaires** du mois et cumul (comparaison N-1), **à encaisser** (clients) et **à payer** (fournisseurs) avec les montants **en retard**, **TVA estimée du mois**,
**prochaines échéances fiscales** (A8 `fiscal_prochaines_echeances`), **alertes** (stock minimum A5, licence A10, sauvegarde ancienne A2). Petits graphiques en **SVG maison** noir et blanc (pas de bibliothèque de graphiques lourde). Chargement < 1 s.

### Lot 7 — Reprise de données (≤ 5 jours)
Assistant « Reprise » (étape 6 du démarrage de A3 et accessible ensuite) : modèles Excel téléchargeables (générés par `ac_documents`) pour chaque type ;
**toi** : plan de comptes personnalisé (sous-comptes) et **balance d'ouverture** (→ écriture `AN` équilibrée, refus si déséquilibre, comptes de tiers détaillés par tiers) ;
**délégués** aux commandes des modules : clients `ventes_importer_clients` (A4), fournisseurs et articles `achats_importer_*` (A5), salariés `paie_importer_salaries` (A7), relevés (A6).
Toujours : aperçu → correspondance des colonnes → rapport d'erreurs ligne par ligne → validation explicite → rien n'est écrit avant.

## 7. Contrats

- **Tu fournis** : commandes `etats_*`, `tableau_de_bord_*`, modèles de reprise.
- **Tu consommes** : lectures de A1, `ac_documents` (A1), soldes de trésorerie (A6), immobilisations (A5), données fiscales et échéances (A8), commandes d'import des modules.

## 8. Tests exigés (exemples chiffrés)

- Balance fictive : capital 1 000 000 (101 C), banque 1 300 000 (521 D), ventes 1 000 000 (701 C), achats 700 000 (601 D) → bilan actif 1 300 000 = passif 1 000 000 + résultat 300 000 ; compte de résultat XI = 300 000.
- Compte non rattaché → erreur explicite, pas d'état faux.
- Balance d'ouverture déséquilibrée de 1 FCFA → refus avec l'écart affiché.
- Import de 500 lignes dont 3 invalides → rapport listant les 3 lignes et leurs erreurs, aucune écriture.
- Tableau de bord sur 50 000 écritures < 1 s (test de performance `#[ignore]`, résultat dans la PR).

## 9. Pièges à éviter

- Ne jamais « équilibrer » un état par un poste fourre-tout : un écart est une **erreur à corriger** et doit être affiché.
- Signes : un compte à solde inversé (ex. banque créditrice = découvert) change de côté du bilan selon les règles SYSCOHADA.
- Les états d'un exercice clôturé sont figés : régénération identique au franc près.

## 10. Définition de « terminé »

CI verte, couverture ≥ 80 %, `docs/modules/etats.md` (correspondances, sources, contrôles), captures du tableau de bord dans la PR, CHANGELOG, PR fusionnée.

## 11. Pour démarrer

```powershell
git checkout main; git pull
git checkout -b a9/feat-correspondances-bilan
cargo test -p ac-etats
```
