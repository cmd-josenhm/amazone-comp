# Module Trésorerie — agent A6

Statut : **Lot 1** (`PROMPT_A6.md`) — comptes de trésorerie. Propriétaire : A6.

## Objectif

Suivre les comptes bancaires, caisses et portefeuilles Mobile Money. Chaque compte possède un sous-compte
comptable, un journal de trésorerie et un solde d'ouverture. Un solde d'ouverture non nul devient une écriture
équilibrée du journal `AN` par `ac_compta::passer_ecriture`.

## Tables

| Table | Rôle |
|---|---|
| `tresorerie_compte` | Type, libellé, banque/opérateur, numéro, sous-compte comptable, journal, solde d'ouverture, actif et date d'ouverture. |
| `tresorerie_operateur` | Référentiel de données des opérateurs Mobile Money, avec MTN MoMo, Moov Money et Celtiis Cash préremplis. |
| `tresorerie_configuration` | Configuration par dossier, dont l'activation du module Mobile Money (activé par défaut). |
| `compta_journal` | Journaux comptables `AN`, `BQn`, `MMn` et `CA` créés avec les comptes. A1 prévoit cette table et `journal::creer` dans C3 ; la migration A6 utilise `CREATE TABLE IF NOT EXISTS` et la structure C3, car cette API n'est pas encore présente sur la branche principale. |

La migration 4000 est enregistrée dans `EtatApplication::toutes_les_migrations`. Le module d'interface est déjà
inscrit dans `ui/src/modules/registre.ts`.

## Règles d'ouverture

- `BANQUE` : sous-compte de `521`, journal `BQ1`, `BQ2`, etc. Le nom de la banque et le numéro de compte sont requis.
- `CAISSE` : sous-compte de `571`, journal `CA` partagé ; le solde d'ouverture ne peut pas être négatif.
- `MOBILE_MONEY` : sous-compte de `55`, journal `MM1`, `MM2`, etc. L'opérateur doit être actif et le téléphone est
  validé/normalisé avec `ac_noyau::entreprise::Telephone` avant stockage.
- Le numéro de sous-compte est le premier suffixe disponible sous la racine (par exemple `5211`, puis `5212`).
  La numérotation Mobile Money reste rattachée à `55` (`551`, `552`, …) ; le sous-compte SYSCOHADA exact
  (`552`, `554` ou autre) doit être confirmé par un expert-comptable avant usage comptable en production.
- Les montants restent des `Montant` entiers FCFA bornés à `Montant::MAX`. Le solde de caisse négatif est refusé
  dans le code et par une contrainte SQLite.
- Un solde d'ouverture non nul exige une référence de pièce justificative et un compte de contrepartie **existant
  et mouvementable**, choisi explicitement dans le plan comptable. Aucun compte de compensation (par exemple `519`)
  n'est codé en dur : C3/C5 ne définissent pas le compte de contrepartie des ouvertures. L'écriture AN débite le
  sous-compte de trésorerie et crédite la contrepartie si le solde est positif ; les sens sont inversés si le solde
  est négatif.
- Les numéros bancaires et téléphones sont stockés dans la base locale mais retournés masqués (quatre derniers
  caractères visibles). Ils ne sont pas copiés dans le journal d'audit.
- Toute création de compte/opérateur et toute écriture AN est journalisée par `ac_donnees::audit::journaliser`
  dans la même transaction SQL. Une erreur annule l'ensemble de la création.

## Opérateurs Mobile Money

Les opérateurs sont des données : MTN MoMo, Moov Money et Celtiis Cash sont insérés à la migration. Une personne
ayant le rôle Administrateur peut ajouter un opérateur via `tresorerie_operateur_creer`, et activer/désactiver le
module pour le dossier via `tresorerie_mobile_money_configurer`. La configuration est journalisée et une création de
compte Mobile Money est refusée lorsque le module est désactivé ; les comptes existants restent conservés. La valeur
par défaut est activée.

## Commandes IPC

| Commande | Rôle | Permission |
|---|---|---|
| `tresorerie_comptes` | Liste des comptes actifs, numéros masqués. | Consulter |
| `tresorerie_compte_creer` | Crée le sous-compte, le journal et l'écriture AN éventuelle. | Valider |
| `tresorerie_compte_afficher` | Lit un compte par identifiant, numéro masqué. | Consulter |
| `tresorerie_soldes` | Soldes comptables, y compris les comptes encore à zéro. | Consulter |
| `tresorerie_configuration` | Lit la configuration Mobile Money du dossier. | Consulter |
| `tresorerie_mobile_money_configurer` | Active ou désactive Mobile Money pour le dossier. | Administrer |
| `tresorerie_operateurs` | Liste les opérateurs actifs et désactivés. | Consulter |
| `tresorerie_operateur_creer` | Ajoute un opérateur en données. | Administrer |

Les commandes IPC conservent les noms de champs Rust en `snake_case` (C8). Messages d'erreur visibles en français.

## Exemples chiffrés testés

- Banque OBA, solde initial de `50 000 FCFA` justifié par un relevé et compte de contrepartie `101` choisi dans le
  plan : sous-compte `5211`, journal `BQ1`, écriture AN `D 5211 50 000 / C 101 50 000`.
- Une deuxième banque reçoit le sous-compte `5212` et le journal `BQ2`, sans collision.
- Portefeuille Moov Money : téléphone normalisé, masqué à la sortie, sous-compte sous `55` et journal `MM1`.
- Sans compte de contrepartie valable, un solde d'ouverture non nul est refusé plutôt que d'utiliser un numéro de compte
  non confirmé.

## Limites hors Lot 1

- L'ajout et la désactivation des comptes, les soldes courants par opérations, les règlements, les virements et la
  caisse opérationnelle seront traités dans les lots suivants.
- Import de relevés et rapprochement bancaire : Lots 3–4 (`PROMPT_A6.md`).
- Les commandes de `ac-compta::journal` et de création de sous-comptes de A1 ne sont pas encore disponibles dans la
  branche principale ; la migration A6 assure la compatibilité de base et utilise les tables C3 en attendant.
