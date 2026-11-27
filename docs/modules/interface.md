# Module interface — coque et design system

Propriétaire : A3. La coque rend le menu commun et les routes de l'application. Les écrans métier restent dans le dossier de leur agent.

## Design system partagé

Les composants publics vivent dans `ui/src/design/composants.tsx` et leurs styles dans `ui/src/design/composants.css`.
Les couleurs sont exclusivement les jetons de `ui/src/design/jetons.css`. Les modules doivent réutiliser ces composants plutôt
que créer un bouton, un champ ou un tableau local.

Le Lot 1 fournit : `Bouton`, `Champ`, `Carte`, `Bandeau`, `Montant`, `Tableau`, `ChampMontant`, `ChampDate`, `Selecteur`,
`CaseACocher`, `Dialogue`, `Assistant`, `Onglets`, `Notification`, `EtatVide`, `Chargement` et `MessageErreur`.
La page `/#/demo-design` est une démonstration interne, exposée uniquement pendant le développement.

## Contrats d'utilisation

- Les props et callbacks sont en français et les données IPC restent en `snake_case`, conformément à C8.
- `Tableau` reçoit `page`, `total` et `surPage`. Le tri et le changement de page sont des événements contrôlés : la sélection
  est envoyée à Rust par le module appelant, aucune pagination complète n'est chargée dans l'interface.
- `ChampMontant` ne reçoit et ne renvoie que des entiers FCFA sûrs. Aucun calcul comptable ou fiscal n'est effectué en TypeScript.
- `ChampDate` ne décide pas si une date métier est valide : la validation de la fiche et des règles métier appartient à
  `FicheEntreprise::valider` ou à la commande Rust concernée.
- `Dialogue`, `Assistant` et `Onglets` gèrent leur navigation au clavier. Un écran qui ajoute une action doit lui donner un
  libellé accessible en français.
- `MessageErreur` reçoit le message déjà préparé par `ErreurAppel` et ajoute une consigne compréhensible ; il ne montre pas de
  trace technique brute.

## Commandes consommées

La commande système existante `systeme_infos` fournit les informations d'application qui seront affichées par la coque finale.
Les commandes des modules métier sont documentées par leurs agents et sont appelées uniquement via `ui/src/lib/ipc.ts`.
Ce document ne crée aucune commande IPC non validée par un contrat.

## Routes internes

- `/#/demo-design` : démonstration du Lot 1, seulement en mode développement.
- Les routes métier sont enregistrées par `ui/src/modules/registre.ts` via `DeclarationModule` (C9).
- `/parametres`, `/demarrage` et `/a-propos` sont les routes de la coque A3 ; leurs données personnelles restent hors de ce lot.
