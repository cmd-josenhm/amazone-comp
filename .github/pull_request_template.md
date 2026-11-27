## Agent et tâche

- Agent : **A?**
- Issue / tâche : #
- Module : 

## Ce que fait cette PR

<!-- 3 à 10 lignes, en français, compréhensibles par le chef de projet. -->

## Règles fiscales ou comptables concernées

<!-- Article du CGI / LF / SYSCOHADA appliqué, fichier de paramètres utilisé. « Aucune » si sans objet. -->

## Exemples chiffrés testés

<!-- Ex. : facture 100 000 HT → TVA 18 000 → TTC 118 000 (test `tva::exemples_chiffres`). -->

## Checklist (obligatoire, cocher honnêtement)

- [ ] J'ai relu `AGENTS.md` et mon `PROMPT_AN.md`.
- [ ] Je n'ai modifié que mon périmètre (`node scripts/verifier-perimetre.mjs aN` passe).
- [ ] Aucun montant en flottant, aucun taux en dur, aucun `unwrap`/`expect`/`panic` en production.
- [ ] Toute écriture comptable passe par `ac_compta::passer_ecriture`.
- [ ] Toute modification de données est journalisée (audit).
- [ ] Tests ajoutés, avec au moins un cas limite ; couverture ≥ 80 %.
- [ ] `cargo fmt`, `cargo clippy -D warnings`, `cargo test --workspace`, `npm run lint`, `npm run test` passent.
- [ ] Textes visibles en français clair ; erreurs compréhensibles par un non-comptable.
- [ ] Pas de nouvelle dépendance, OU dépendance justifiée ci-dessous (taille, licence, maintenance).
- [ ] Documentation `docs/modules/<module>.md` et `CHANGELOG.md` mis à jour.
- [ ] J'accepte l'accord de contribution (`docs/ACCORD_CONTRIBUTION.md`).

## Nouvelles dépendances (justification)

## Hors périmètre / suites à prévoir
