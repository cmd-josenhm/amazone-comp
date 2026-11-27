# Module Paie (`ac-paie`) — agent A7

> Statut : **Lot 1 (paramètres sociaux typés) en cours de revue**. Les calculs de cotisations,
> les bulletins, les salariés et les écritures de paie ne sont pas encore livrés.

## Rôle

Paie béninoise : salariés, bulletins, ITS, VPS, CNSS, redevance audiovisuelle, déclarations
sociales. Le module lit ses paramètres dans `parametres-fiscaux/<année>/social.toml` (propriété A7).

## Paramètres sociaux typés (Lot 1)

Type : `ac_paie::ParametresSociaux`.

- **Chargement par année d'application (contrat C10)** : `ParametresSociaux::pour_date(date)`
  charge les paramètres de l'année civile de la **date de l'opération** (date de paie), pas la
  date du jour. `ParametresSociaux::charger(annee)` charge une année précise.
  Une année sans fichier renvoie `ErreurPaie::AnneeInconnue` (« mettre à jour le logiciel »).
- **Couverture complète** : `deny_unknown_fields` refuse toute clé du fichier absente de la
  struct. Le test `chaque_cle_du_fichier_est_couverte` vérifie la correspondance dans l'autre sens.
- **Source obligatoire (R14)** : chaque valeur est une `Valeur<T>` (`valeur`, `source`, `verifie`,
  `note`). Le test `chaque_valeur_a_une_source` échoue si une source est vide.
- **Validation au chargement** : barème non vide, bornes strictement croissantes, seule la
  dernière tranche sans borne ; mois de redevance entre 1 et 12 ; taux minimum des risques
  professionnels ≤ maximum ; SMIG strictement positif.
- **Aucun flottant** : taux en `Taux`, montants en `Montant` (FCFA entiers).

### Accesseurs

| Méthode | Retour | Paramètre |
|---|---|---|
| `bareme_its()` | `&[Tranche]` | `its_bareme_mensuel` |
| `base_its()` | `BaseIts` | `its_base_imposable` (`NonTranchee` aujourd'hui) |
| `redevance_audiovisuelle(mois)` | `Option<Montant>` | `its_redevance_audiovisuelle` |
| `plafond_cnss_mensuel()` | `Option<Montant>` (`None` si `0`) | `cnss_plafond_mensuel` |
| `periodicite_cnss()` | `Periodicite` | `cnss_periodicite` (`NonTranchee` aujourd'hui) |
| `majorations()` | `&[Majoration]` | `heures_supplementaires_majorations` (vide aujourd'hui) |
| `sources()` | `Vec<SourceValeur>` | source et `verifie` de chaque paramètre |

Les autres paramètres (VPS, taux CNSS, SMIG) sont des champs publics de la struct, avec la même
structure `Valeur<T>`.

## ITS

`its::calculer_its(brut_imposable, bareme)` applique un barème progressif par tranches, avec un
arrondi au franc inférieur par tranche. Il ne contient aucun taux : le barème vient de
`ParametresSociaux`. **La base imposable n'est pas tranchée** (issue #44) : tant que ce point
n'est pas confirmé, le calcul ne doit pas servir à un bulletin définitif.

## Valeurs non vérifiées et questions ouvertes

Aucune valeur de `social.toml` 2026 n'est vérifiée (`verifie = false`). Les points incertains
ont une issue `fiscal-a-verifier` :

| Issue | Paramètre | Question |
|---|---|---|
| #43 | `its_bareme_mensuel` | Premier seuil ITS : 60 000 ou 50 000 FCFA ? |
| #44 | `its_base_imposable` | Base ITS : brut ou brut moins CNSS salarié ? |
| #45 | `cnss_plafond_mensuel` | Plafond de l'assiette CNSS : aucun ou 600 000 FCFA ? |
| #46 | `its_redevance_audiovisuelle` | Mois, montants et exemptions de la redevance |
| #47 | `cnss_periodicite` | Périodicité et date limite CNSS selon l'effectif |
| #48 | `smig_mensuel` | SMIG 2026 (52 000 FCFA à confirmer) |
| #49 | `heures_supplementaires_majorations` | Taux de majoration des heures supplémentaires |

Sources et contradictions détaillées : `docs/fiscalite/sources.md` et
`docs/fiscalite/regles-fiscales-benin.md` (§4.6 et §4.7).

## Commandes IPC, tables, écritures

- Commandes `paie_*` : **aucune** au Lot 1 (`src-tauri/src/commandes/paie.rs` n'existe pas encore).
- Tables : **aucune** au Lot 1 (migrations 5000–5999 à venir).
- Écritures de paie (comptes, étiquettes `Its`, `Vps`, `CnssSalarie`, `CnssEmployeur`,
  `RedevanceAudiovisuelle`) : **aucune** au Lot 1.

## Limites connues

- Pas de calcul de cotisations (Lot 2), pas de salariés (Lot 3), pas de bulletins (Lot 4).
- Le plafond CNSS `0` signifie « pas de plafond » : convention provisoire, à confirmer (#45).
- Les paramètres 2027 n'existent pas encore : toute paie datée de 2027 renvoie `AnneeInconnue`.

## Tests

- Tests chiffrés écrits à la main : seuil exact de l'ITS (60 000), cas zéro, cotisations sur
  300 000 FCFA (CNSS 3,6 % = 10 800 ; prestations 9 % = 27 000 ; pension employeur 6,4 % = 19 200 ;
  risques 1 % = 3 000 ; VPS 4 % = 12 000 ; VPS privé 2 % = 6 000).
- Test « chaque valeur a une source » et test de couverture du fichier.
