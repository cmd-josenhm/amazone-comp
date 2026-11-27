# Module Comptabilité (et noyau) — agent A1

> Propriétaire : agent A1. Ce document décrit ce qui est livré dans `ac-noyau`, `ac-compta` et
> `ac-documents`, avec des exemples chiffrés testés. Il est mis à jour à chaque lot
> (voir `PROMPT_A1.md` pour le plan complet).

## État d'avancement

| Lot | Contenu | État |
|---|---|---|
| 1 | Noyau finalisé : `Quantite`, `Montant::multiplier_quantite`, `Montant::repartir` | **Livré** (issue #9) |
| 2 | Plan de comptes SYSCOHADA | À venir |
| 3 | Exercices, périodes, journaux | À venir |
| 4 | `passer_ecriture` complète, brouillons, contre-passation | À venir |
| 5 | Lectures (balance, grand-livre, agrégats fiscaux) | À venir |
| 6 | Lettrage | À venir |
| 7 | Clôture et à-nouveaux | À venir |
| 8 | Rendu PDF / Excel (`ac-documents`) | À venir |
| 9 | Interface « Comptabilité » et commandes Tauri | À venir |

## Types de base (`ac-noyau`)

### `Montant` — francs CFA entiers (ADR 0002)

Un montant est un entier de francs (`i64`), borné à ±10^15 FCFA, sérialisé en entier JSON.
Jamais de flottant (règle R1). Affichage français : `1 234 567 FCFA` (espaces fines insécables).

### `Quantite` — quantité exacte en millièmes (Lot 1, issue #9)

Une quantité non entière (1,5 kg, 2,5 heures…) est stockée en **millièmes** (`i64`), jamais en
flottant : 1 000 millièmes = 1 unité. Bornée à ±10^15 millièmes (soit 10^12 unités) pour que la
multiplication par un `Montant` ne puisse pas déborder.

| Construction | Résultat |
|---|---|
| `Quantite::depuis_milliemes(3_500)` | 3,5 |
| `Quantite::depuis_unites(2)` | 2,000 |
| `Quantite::depuis_texte("1,25")` | 1,25 (virgule ou point acceptés, au plus 3 décimales) |

Affichage : `1,5`, `1,25`, `2`, `0` (format français, zéros inutiles supprimés). Sérialisation :
entier JSON en millièmes — l'interface ne fait aucun calcul (contrat C8). Tout texte qui dépasse
3 décimales est refusé (jamais de troncature en silence) : `QuantiteInvalide`.

### `Montant::multiplier_quantite(q, arrondi)` — prix unitaire × quantité (Lot 1)

Produit exact « francs × millièmes » calculé en entier 128 bits, divisé par 1 000 selon la règle
d'arrondi demandée. Le 0,5 exact s'éloigne de zéro (même convention que `appliquer_taux`).

| Exemple | `PlusProche` | `Inferieur` | `Superieur` |
|---|---|---|---|
| 1 250 FCFA × 3,5 = 4 375 | 4 375 | 4 375 | 4 375 |
| 1 001 FCFA × 0,333 = 333,333 | 333 | 333 | 334 |
| 1 FCFA × 0,5 = 0,5 exact | 1 | 0 | 1 |
| −1 FCFA × 0,5 (avoir) | −1 | 0 | −1 |
| 1 250 FCFA × 0 (cas zéro) | 0 | 0 | 0 |

### `Montant::repartir(parts)` — répartition sans perte d'un franc (Lot 1)

Chaque part reçoit sa proportion du total (arrondie au franc le plus proche, 0,5 s'éloignant de
zéro) ; la **différence d'arrondi est absorbée par la plus grande part** (à parts égales, la
première fournie). La somme des parts renvoyées est **toujours exactement égale** au montant
réparti.

| Exemple | Résultat |
|---|---|
| 100 FCFA en parts `[1, 1, 1]` | 34 / 33 / 33 |
| 100 FCFA en parts `[1, 1, 2]` | 25 / 25 / 50 (proportions exactes) |
| 102 FCFA en parts `[1, 1, 2]` | 26 / 26 / 50 (l'écart −1 va à la plus grande part) |
| −100 FCFA en parts `[1, 1, 1]` | −34 / −33 / −33 (même règle sur la valeur absolue) |
| 5 FCFA en 8 parts égales | 0 / 0 / 0 / 1 / 1 / 1 / 1 / 1 (aucune part négative) |
| 0 FCFA en parts `[0, 0]` | 0 / 0 (cas zéro) |

Si absorber l'écart rendrait la plus grande part négative (total très petit devant le nombre de
parts), l'excédent est reporté sur les parts suivantes, toujours de la plus grande à la plus
petite : aucune part ne devient négative pour un total positif.

Erreurs (`ErreurNoyau::RepartitionImpossible`) : parts vides, part négative, ou somme des parts
nulle pour un montant non nul.

### Tests de propriété

`proptest` (dépendance de développement, justifiée dans la PR) vérifie sur des milliers de cas :

- `a + b − b == a` (opérateurs et méthodes contrôlées) ;
- la répartition conserve le total, les parts gardent le signe du total et restent dans ses bornes ;
- `× 1` (une unité exacte) est l'identité, `× 0` annule tout, quel que soit l'arrondi ;
- la multiplication par une quantité positive conserve l'ordre des montants.

## Moteur comptable (`ac-compta`) — état actuel

- Types : `NumeroCompte`, `CodeFiscal`, `BrouillonEcriture`, `LigneBrouillon` (contrat C3).
- `verifier_brouillon` (équilibre Σ débits = Σ crédits, comptes à 2 chiffres minimum) et
  `passer_ecriture` minimal. Contrôles complets, numérotation sans trou : Lot 4.
- Migration `1000_compta_socle.sql` : tables `compta_compte`, `compta_ecriture`, `compta_ligne`
  avec déclencheurs d'immuabilité des écritures validées (règle R3).

## Documents (`ac-documents`) — état actuel

- Contrat `Document` / `Bloc` / `Cellule` (contrat C7). Le rendu PDF / Excel arrive au Lot 8.

## Limites connues

- Les formats officiels RCCM / téléphone / n° CNSS restent à confirmer par le chef de projet
  (issues `fiscal-a-verifier`) : la validation actuelle n'avertit que, sans bloquer à tort
  une entreprise réelle (ADR 0009).
- `proptest` est déclarée comme dépendance de développement directement dans
  `crates/ac-noyau/Cargo.toml`, car le `Cargo.toml` racine est hors périmètre A1 ;
  à remonter dans `[workspace.dependencies]` par le chef de projet si souhaité.
