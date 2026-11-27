# Module États financiers, DSF, Tableau de bord et Reprise (`ac-etats` — Agent A9)

## Rôle

La crate `ac-etats` transforme les écritures et balances comptables en états financiers SYSCOHADA révisé (Bilan, Compte de résultat, TFT, Notes annexes), prépare la liasse fiscale (DSF) et l'état SMT, alimente le tableau de bord dirigeant et orchestre la reprise des données d'ouverture.

## État d'avancement

- **Lot 1 — Correspondances et moteur d'états (livré dans `ac-etats`)** :
  - Table de correspondances versionnée `crates/ac-etats/donnees/correspondances_syscohada.csv` ;
  - Types `Balance`, `LigneBalance`, `SensSolde`, `Correspondances` ;
  - Calcul pur `bilan(balance) -> Resultat<Bilan>` (Actif `AD`..`BZ` en Brut / Amort-Dépréc / Net N et Net N-1 ; Passif `CA`..`DZ` en Net N et Net N-1) ;
  - Calcul pur `compte_resultat(balance) -> Resultat<CompteResultat>` (rubriques `TA`..`RS`, chiffre d'affaires et soldes intermédiaires `XA`..`XI`, N et N-1) ;
  - Contrôles de cohérence (`verifier_coherence`, `generer_etats_complets`).
- **Lots 2 à 7** : à venir (dépendent notamment du Lot 5 et de `ac-documents` d'A1).

## Sources des correspondances

- **Référentiel** : Acte uniforme OHADA relatif au droit comptable et à l'information financière (**AUDCIF**, adopté le 26 janvier 2017 à Brazzaville, JO OHADA spécial du 15 février 2017), Annexe — Modèle des états financiers du **Système normal**, table de correspondance Postes / Comptes.
- **Statut de validation** : en attente de relecture par un expert-comptable et d'alignement avec le `plan_syscohada.csv` d'A1 (issues de suivi ouvertes sur GitHub).

## Règles de correspondance et signes

1. **Spécificité maximale (préfixe le plus long)** :
   - Ex. `245` / `2495` (Matériel de transport → `AN`) prime sur `24` (Autres matériels → `AM`).
   - Ex. `6031` (`RB`), `6032` (`RD`), `6033` (`RF`), `659` (`RL`), `679` (`RN`), `759` (`TJ`), `779` (`TL`).
2. **Reclassement des comptes à solde inversé (non-compensation au Bilan)** :
   - `52`–`55`, `57`–`58` : solde débiteur → `BS` (Trésorerie-Actif) ; solde créditeur (découvert) → `DR` (Trésorerie-Passif).
   - `411`–`419` : solde débiteur → `BI` (Clients) ; solde créditeur → `DI` (Clients, avances reçues).
   - `401`–`409` : solde créditeur → `DJ` (Fournisseurs d'exploitation) ; solde débiteur → `BH` (Fournisseurs avances versées).
   - `42`, `43`, `44` : solde débiteur → `BJ` (Autres créances) ; solde créditeur → `DK` (Dettes fiscales et sociales).
   - `45`, `46`, `471`–`473`, `475` : solde débiteur → `BJ` ; solde créditeur → `DM` (Autres dettes).
3. **Résultat net au Bilan (`CJ`)** :
   - Avant écriture de clôture (`13` nul), `CJ` agrège le résultat algébrique des comptes de gestion et HAO (classes `6`, `7`, `8`).
   - Après écriture de clôture (classes `6`, `7`, `8` soldées), `CJ` reprend le solde des comptes `131`–`139`.
4. **Montants exacts** : tous les calculs utilisent `ac_noyau::Montant` (entiers FCFA, zéro flottant).

## Contrôles de cohérence obligatoires

- **Compte non rattaché** : tout compte absent de la table (ex. classe 9, compte `474` de répartition périodique non soldé ou compte inconnu) bloque la génération avec le message explicite `« Le compte <numéro> n'est rattaché à aucune rubrique »`.
- **Équilibre du Bilan** : `Total Actif Net (BZ) == Total Passif (DZ)` vérifié pour N et pour N-1 (si comparatif présent). Aucun poste fourre-tout n'est jamais créé : tout écart renvoie `ErreurEtats::BilanDesequilibre`.
- **Cohérence Bilan / Compte de résultat** : `verifier_coherence` vérifie `bilan.resultat_net == compte_resultat.resultat_net` (N et N-1).
- **Intégrité de la balance** : refus des doublons de comptes et des totaux débit/crédit négatifs.

## Dépendances et interfaces

- **Consomme** : `ac-noyau` (`Montant`, `Periode`), `ac-compta` (`NumeroCompte`, `ErreurCompta`), `ac-donnees`.
- **Contrat attendu d'A1** : `balance`, `grand_livre`, `solde_compte`, `agregats_fiscaux` (suivi par issue de coordination A9/A1). Tant que le Lot 5 d'A1 n'est pas livré, le moteur prend une `Balance` construite en mémoire.
- **Écrans / commandes IPC / tables SQL** : aucun pour le Lot 1 (migrations `7000–7999` vides à ce stade ; écrans et commandes `etats_*` / `tableau_de_bord_*` prévus aux Lots 2 à 7).

## Limites connues

- Pas encore de lecture SQLite via `ac_compta::balance` (Lot 2 après livraison d'A1).
- TFT, notes annexes, SMT, DSF, tableau de bord et reprise de données relèvent des Lots 3 à 7.
- Le format Excel officiel de la liasse DSF béninoise (e-services / DGI) est en attente de confirmation par le chef de projet.
