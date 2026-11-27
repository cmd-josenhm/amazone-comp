# ADR 0002 — Montants en entiers FCFA, taux décimaux exacts

- Statut : **Accepté** (chef de projet, 2026-11-27)

## Contexte
Les flottants (`f64`) produisent des erreurs d'arrondi (0,1 + 0,2 ≠ 0,3) inacceptables en comptabilité. Le FCFA n'a pas de subdivision en usage.

## Décision
- `ac_noyau::Montant` = `i64` de **francs**, borné à ±10^15. Stocké `INTEGER` en base, transmis en entier JSON.
- `ac_noyau::Taux` = `rust_decimal::Decimal` (fraction), stocké et transmis en **texte**.
- Tout calcul de pourcentage passe par `Montant::appliquer_taux(taux, Arrondi)` avec une règle d'arrondi explicite.
- Les quantités non entières (1,5 kg) sont exprimées en **millièmes** (`i64`).
- Lint `clippy::float_arithmetic = deny` dans tout le workspace ; ESLint interdit `parseFloat`.

## Conséquences
Aucun calcul monétaire côté TypeScript : l'interface affiche ce que le Rust calcule.
