# ADR 0003 — Licence du code : GNU AGPL-3.0

- Statut : **Accepté** (propriétaire, 2026-11-27)

## Contexte
Code public sur GitHub, logiciel gratuit, possibilité d'une offre payante plus tard, protection contre la récupération commerciale fermée.

## Décision
- Licence **AGPL-3.0-only** (gratuite). Toute redistribution ou service en ligne basé sur une version modifiée doit publier son code.
- **Accord de contribution** (`docs/ACCORD_CONTRIBUTION.md`) : le propriétaire peut relicencier les contributions → offre commerciale possible plus tard (double licence).
- La **marque** « AMAZONE COMP », le logo et la plateforme ne sont pas sous AGPL.

## Conséquences
Dépendances : licences compatibles uniquement (MIT, Apache-2.0, BSD, ISC, MPL-2.0…), contrôlées par `cargo deny` (`deny.toml`).
