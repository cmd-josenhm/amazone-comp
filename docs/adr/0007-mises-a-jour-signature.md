# ADR 0007 — Mises à jour signées et signature de code

- Statut : **Accepté** (chef de projet, 2026-11-27) — implémentation : A10

## Décision
- **Plugin updater de Tauri 2** : paquets signés avec une clé minisign (privée en secret GitHub, publique dans `tauri.conf.json`). Manifeste servi par la plateforme (`/v1/mises-a-jour/windows/{version}`), qui vérifie la licence.
- Mode **automatique** (téléchargement en arrière-plan puis proposition d'installer à la fermeture) ou **manuel** (bouton « Rechercher une mise à jour »), réglable dans Paramètres. Jamais d'installation pendant une saisie.
- **Signature Authenticode** (éviter l'alerte SmartScreen) : pas encore de certificat. Options étudiées :
  - **SignPath Foundation** : signature **gratuite** pour les projets open source éligibles (dépôt public, licence OSI, build CI reproductible) — à demander en priorité ;
  - certificat OV/EV commercial (payant, ~100 à 400 € / an selon le fournisseur) ;
  - Azure Trusted Signing (abonnement mensuel, conditions d'éligibilité géographique à vérifier).
- En attendant : installateur non signé + somme SHA-256 publiée + avertissement SmartScreen documenté pour l'utilisateur.
