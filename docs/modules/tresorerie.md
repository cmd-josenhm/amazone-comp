# Module Trésorerie — agent A6

Statut : **Lot 1** (`PROMPT_A6.md`) — comptes de trésorerie. Owner : A6.

## Objectif

Suivre les actifs liquides de l'entreprise : comptes bancaires, caisses d'espèces et portefeuilles
**Mobile Money**, avec un solde d'ouverture qui devient une écriture comptable (`AN`) via
`ac_compta::passer_ecriture`.

## Tables

| Table | Rôle |
|---|---|
| `tresorerie_compte` | Compte de trésorerie : type, libellé, banque/opérateur, numéro, compte comptable, journal, solde d'ouverture, actif, date d'ouverture. |
| `tresorerie_operateur` | Opérateurs Mobile Money pré-remplis (données — Le module est désactivable). |
| `compta_journal` | Journaux déclarés par les comptes (`journal_creer`, `journal_prochant_code`). A1 fournit `journal::creer` dans `ac-compta`. |

## Modèles

- `TypeCompte` : `BANQUE` (compte 521), `CAISSE` (571, ne peut pas être négatif), `MOBILE_MONEY` (55).
- `TresorerieCompte` : compte stocké ; `numero` masqué en lecture (`masquer` : 4 derniers chiffres visibles).
- `TresorerieSolde` : solde comptable lu dans `compta_ligne` (A1 fournit `solde_compte` dans `ac-compta` — TODO Lot 2).
- `CreerCompte` : paramètres de la commande `tresorerie_compte_creer`.

## Processus de création (`creer_compte`)

1. Validation : libellé non vide ; numéro obligatoire sauf caisse ; opérateur Mobile Money obligatoire et présent dans `tresorerie_operateur` ; caisse ne peut pas avoir un solde négatif.
2. Sous-compte comptable : `521` / `571` / `55` (C5), inséré dans `compta_compte`.
3. Journal : `CA` pour la caisse ; `BQ1`, `BQ2`… pour la banque ; `MM1`, `MM2`… pour le Mobile Money (contrat C3).
4. Écriture AN (si solde non nul) : `passer_ecriture` avec journal `AN`, `ModuleOrigine::Tresorerie`, lignes `D 521/55/571` + `C 519` (ou inversé si solde créditeur).
   - Compte AN : `519` — à valider par l'expert-comptable (note évidente, compte « AN des trésoreries »).

## Commandes IPC

| Commande | Rôle | Permission |
|---|---|---|
| `tresorerie_comptes` | Liste des comptes actifs (numéros masqués). | Consulter |
| `tresorerie_compte_creer` | Crée un compte (BANQUE / CAISSE / MOBILE_MONEY). | Saisir |
| `tresorerie_compte_afficher` | Lit un compte par identifiant. | Consulter |
| `tresorerie_soldes` | Soldes comptables par compte (`compta_ligne`). | Consulter |
| `tresorerie_operateurs` | Liste des opérateurs Mobile Money. | Consulter |

Erreurs renvoyées : `ErreurTresorerie` → codes `TYPE_COMPTE_INVALIDE`, `LIBELLE_VIDE`,
`NUMERO_OBLIGATOIRE`, `OPERATEUR_OBLIGATOIRE`, `OPERATEUR_INCONNU`, `COMPTE_COMPTABLE_EXISTANT`,
`CAISSE_SOLDE_NEGATIF`, `COMPTE_INCONNU`, `JOURNAL_CODE_INVALIDE`, `ERREUR_COMPTA` (A1), `ERREUR_DONNEES` (A2).

## Exemples chiffrés (tests)

- Création d'une banque avec numéro `000000101` et solde d'ouverture `50 000` : compte comptable `521`, journal `BQ1`, écriture AN `D 521 50 000 / C 519 50 000`.
- Deuxième banque : journal `BQ2`.
- Modèle Mobile Money `Moov Money`, numéro `0612 345 678`, solde `1 000` : compte comptable `55`, journal `MM1`.
- `masquer("000000101")` → `•••••0101`.

## Limites connues (TODO A6/A1)

- Import de relevés et rapprochement bancaire : **Lot 3–4** (voir `PROMPT_A6.md`).
- Encaissements / décaissements : **Lot 2**.
- `solde_compte` : A1 (plage C3).
