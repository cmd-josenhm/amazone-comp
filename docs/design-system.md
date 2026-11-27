# Design system — sobre, noir et blanc

Inspiration : la clarté des pages Google (beaucoup de blanc, peu d'éléments, texte lisible). Logo : en attente du graphiste ; icône provisoire noire « AC ».

## Règles

1. **Couleurs** : uniquement les jetons de `ui/src/design/jetons.css`. Fond blanc, texte `#202124`, gris secondaires, **noir** pour l'action principale et l'élément de menu actif. Rouge / vert / ambre **seulement** pour erreur / succès / alerte, jamais décoratifs.
2. **Typographie** : Segoe UI (police système Windows), 14 px de base, titres légers (400). Aucune police téléchargée.
3. **Une action principale par écran** (bouton noir). Les autres actions en boutons blancs bordés.
4. **Formulaires** : libellé au-dessus du champ, aide en gris sous le champ, erreur en rouge avec la solution (« L'IFU doit contenir 13 chiffres »).
5. **Montants** : chiffres tabulaires, alignés à droite, format `1 234 567 FCFA` (`formaterMontant`).
6. **Langage** : phrases courtes pour un non-comptable. Le terme technique est en second : « Argent que vos clients vous doivent (compte 411) ».
7. **Assistants pas à pas** pour toute opération complexe (clôture, déclaration, import), avec retour arrière possible avant validation.
8. **Clavier** : tout est faisable au clavier ; `Entrée` valide, `Échap` annule, raccourcis listés dans l'aide.
9. **Écran minimal** : 1024 × 640 (petits portables).
10. Pas d'animation lourde, pas d'image de fond : priorité à la vitesse sur PC modeste.

## Composants disponibles (`ui/src/design/composants.tsx`)

`Bouton` (principal / secondaire), `Champ` (texte, multiligne, aide, erreur), `Carte`, `Bandeau` (info / alerte), `Montant`, `formaterMontant`.
A3 ajoute au fur et à mesure : `Tableau` (paginé, tri, filtres), `Selecteur`, `ChampDate`, `ChampMontant` (saisie en entiers avec séparateurs), `Dialogue`, `Assistant` (étapes), `Onglets`, `Notification`.
