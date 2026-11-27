# ADR 0006 — Adaptateur de facture normalisée (MECeF / e-MECeF)

- Statut : **Accepté** (chef de projet, 2026-11-27) — implémentation : A4

## Contexte
La facture normalisée est obligatoire au Bénin pour les entreprises concernées. Nous n'avons pas encore d'accès à l'API e-MECeF de la DGI.

## Décision
- Trait `ac_ventes::normalisation::Normaliseur` en **deux temps** (`soumettre` puis `finaliser(Confirmer|Annuler)`), reflet du fonctionnement e-MECeF.
- `Simulateur` disponible dès maintenant : les documents portent « SIMULATION — NON VALABLE FISCALEMENT » et `simulation = true` est stocké.
- `EMecefApi` (client HTTPS) écrit quand la spécification et le jeton seront obtenus ; URL de base et jeton dans les paramètres du dossier (jeton chiffré dans la base).
- **File d'attente** : sans Internet, la facture validée reste « en attente de normalisation » ; elle est soumise automatiquement au retour de la connexion. Elle ne peut pas être remise au client comme facture définitive avant normalisation.
- Option « saisie manuelle des références MECeF » pour les entreprises qui utilisent un boîtier MECeF physique (saisie du code et du NIM imprimés par le boîtier).
- Les totaux renvoyés par la DGI sont comparés aux totaux locaux : tout écart bloque la confirmation.

## Conséquences
Le reste du logiciel (comptabilité, TVA, AIB) ne dépend pas de l'implémentation choisie.
