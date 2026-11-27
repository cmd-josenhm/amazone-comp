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

Tous les composants sont des composants React fonctionnels sans dépendance d'interface externe. Ils exposent un nom accessible,
conservent un focus visible et utilisent les jetons de `ui/src/design/jetons.css`. Les callbacks `sur...` ne réalisent aucun
calcul comptable ou fiscal : ils transmettent les choix à l'écran appelant, puis l'écran appelle Rust.

### Composants de base

- `Bouton` : variantes `principal` et `secondaire`. Le type est `button` par défaut ; il peut être utilisé dans un formulaire.
- `Champ` : texte, courriel, téléphone, mot de passe ou multiligne, avec libellé, aide, erreur et état obligatoire.
- `Carte` : bloc regroupant une section, avec titre facultatif.
- `Bandeau` : message `info`, `alerte`, `succes` ou `erreur`, avec le rôle ARIA correspondant.
- `Montant` et `formaterMontant` : affichage d'un entier sûr en FCFA, avec chiffres tabulaires et séparateurs français.

### Composants du Lot 1

- `Tableau<Ligne>` : reçoit `lignes`, `colonnes`, `page`, `total` et `surPage`. La page et le tri sont contrôlés par l'appelant
  (`surChangementPage`, `surChangementTri`) afin que la pagination soit réalisée côté Rust. Une colonne peut déclarer `format: "montant"`;
  les montants sont alors alignés à droite. Une ligne cliquable reste activable par Entrée ou Espace.
- `ChampMontant` : saisie d'un entier FCFA avec séparateurs de milliers. Les lettres, décimales et montants négatifs (sauf
  `negatifAutorise`) sont refusés sans appeler le callback. Le callback reçoit un `number` entier sûr.
- `ChampDate` : saisie `JJ/MM/AAAA` et calendrier simple. Le calendrier est navigable par les boutons et se ferme avec Échap.
- `Selecteur` : champ `combobox` avec recherche, liste `listbox` et navigation avec les flèches, Entrée et Échap.
- `CaseACocher` : case native avec libellé, aide, erreur et état obligatoire.
- `Dialogue` : modal avec `role="dialog"`, `aria-modal`, retour du focus, piège de tabulation et fermeture par Échap.
- `Assistant` : étapes précédentes accessibles, boutons Précédent/Suivant et étape finale avec `resumeFinal` avant Valider.
- `Onglets` : onglets WAI-ARIA avec navigation par flèches, Début et Fin.
- `Notification` : toast avec rôle d'état ou d'alerte, fermeture au clavier et durée facultative.
- `EtatVide` : titre, explication et action facultative pour une liste sans résultat.
- `Chargement` : état d'attente annoncé comme statut, avec le texte français « Chargement… » par défaut.
- `MessageErreur` : affiche le `message` d'une `ErreurAppel` (ou d'une erreur compatible) et une action proposée ou un bouton Réessayer.

La démonstration de ces composants est disponible à `/#/demo-design` en mode développement (`import.meta.env.DEV` uniquement).
