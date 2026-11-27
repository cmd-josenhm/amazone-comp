# Paramètres fiscaux et sociaux versionnés

**Règle R2 : aucun taux, seuil, barème ou date fiscale n'est écrit dans le code.** Tout est ici.

- Un dossier par année d'application (`2026/`, `2027/`…). On ne modifie **jamais** une année passée
  après sa publication : on crée l'année suivante (les recalculs historiques en dépendent).
- Chaque valeur réglementaire est une table `{ valeur, source, verifie, note }` :
  - `source` : texte légal précis (article du CGI, article de la loi de finances, décret, arrêté) ;
  - `verifie` : `true` **uniquement** après validation par un expert-comptable, décidée par le chef de projet ;
  - `note` : contradictions entre sources, points d'attention.
- Les fichiers sont embarqués dans l'exécutable à la compilation (`ac-fiscal`, `ac-paie`) et un test
  vérifie que chaque valeur a bien une `source` et un `verifie`.

| Fichier | Contenu | Propriétaire |
|---|---|---|
| `tva.toml` | TVA : taux, seuil, retenues, limites de déduction | A8 |
| `aib.toml` | Acompte sur impôt assis sur les bénéfices | A8 |
| `impot-resultat.toml` | IS, IBA, minimum de perception, acomptes | A8 |
| `tps.toml` | Taxe professionnelle synthétique | A8 |
| `retenues.toml` | Retenues à la source (loyers, non-résidents…) | A8 |
| `penalites.toml` | Pénalités et intérêts de retard | A8 |
| `calendrier.toml` | Échéances déclaratives | A8 (+ A7 pour le social) |
| `emecef.toml` | Groupes de taxation et codes de la facture normalisée | A8 (consommé par A4) |
| `social.toml` | ITS, VPS, CNSS, SMIG, redevance audiovisuelle sur salaires | A7 |

> ⚠️ **ÉTAT AU 2026-11-27 : AUCUNE valeur n'est vérifiée.** Les valeurs viennent de recherches documentaires
> (CGI 2026, presse sur la LF 2026, guides professionnels) listées dans `docs/fiscalite/sources.md`.
> Elles doivent être confrontées au texte officiel du CGI 2026 et de la loi de finances 2026, puis
> validées par un expert-comptable avant toute mise en production.
