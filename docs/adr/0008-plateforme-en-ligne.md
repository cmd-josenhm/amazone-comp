# ADR 0008 — Plateforme en ligne minimale

- Statut : **Accepté** (chef de projet, 2026-11-27) — implémentation : A10 — **complété par les ADR 0009 (inscription) et 0010 (console d'administration et assistance)**

## Contexte
Seul coût accepté : l'hébergement. Pas d'API payante, pas de service tiers superflu.

## Décision
- Un seul service **Rust (axum)** + **PostgreSQL**, déployé en conteneur Docker sur un VPS (2 vCPU / 2–4 Go). Reverse proxy **Caddy** (HTTPS Let's Encrypt gratuit).
- Fonctions V1 : comptes (inscription, connexion, vérification e-mail, mot de passe oublié), CGU versionnées, entreprises, émission / renouvellement / révocation de licences, transfert de poste, sauvegardes chiffrées (blobs opaques, quota par compte), avis des utilisateurs, manifeste de mises à jour, **inscription des entreprises** (ADR 0009), **console d'administration web et messagerie ASSISTANCE** (ADR 0010).
- E-mails : SMTP de l'hébergeur ou d'un fournisseur avec offre gratuite suffisante (choix documenté par A10).
- Stockage des sauvegardes sur le disque du VPS (V1), avec sauvegarde de la base PostgreSQL chiffrée hors site (à prévoir).
- API versionnée `/v1/…`, JSON, limitation de débit, journalisation sans données personnelles inutiles.

## Conséquences
L'application doit fonctionner **entièrement** sans la plateforme, sauf : première activation, renouvellement de licence, mises à jour, sauvegarde distante, envoi des avis.
