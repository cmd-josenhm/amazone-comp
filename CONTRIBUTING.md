# Contribuer à AMAZONE COMP

Merci de votre intérêt. Les règles complètes et **obligatoires** sont dans [AGENTS.md](AGENTS.md) : elles s'appliquent aussi aux contributeurs humains.

## Processus

1. Ouvrir (ou choisir) une issue. Pour une évolution importante : issue `rfc:` d'abord.
2. Créer une branche depuis `main` : `aN/<type>-<sujet>` (agents) ou `ext/<type>-<sujet>` (contributeurs externes).
3. Développer **avec les tests**. Respecter les règles R1 à R14 d'`AGENTS.md`.
4. Lancer localement les vérifications listées dans `AGENTS.md` §3.
5. Ouvrir une Pull Request en remplissant **tout** le modèle.
6. Le chef de projet relit, teste et décide : fusion, demande de corrections, ou refus motivé.

## Critères de validation d'une PR (checklist du relecteur)

- [ ] CI verte (format, lint, tests, couverture ≥ 80 % sur les crates métier, périmètre).
- [ ] Le périmètre de l'agent est respecté.
- [ ] Aucun montant en flottant, aucun taux en dur, aucun `unwrap` en production.
- [ ] Les calculs fiscaux citent leur source et ont des tests chiffrés.
- [ ] Les écritures générées sont équilibrées et passent par `ac-compta`.
- [ ] Les modifications de données passent par le journal d'audit.
- [ ] Textes UI en français clair, compréhensibles par un non-comptable.
- [ ] Pas de nouvelle dépendance injustifiée ; licence compatible AGPL-3.0.
- [ ] Documentation du module mise à jour (`docs/modules/…`).
- [ ] Le CHANGELOG est complété (section « Non publié »).

## Licence des contributions

En contribuant, vous acceptez que votre contribution soit distribuée sous licence AGPL-3.0 et vous
**cédez au propriétaire du projet le droit de la redistribuer sous d'autres conditions** (y compris
commerciales), conformément à l'accord de contribution [docs/ACCORD_CONTRIBUTION.md](docs/ACCORD_CONTRIBUTION.md).
Sans cet accord, la PR ne peut pas être fusionnée.
