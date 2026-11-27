# Journal des modifications

Format : [Keep a Changelog](https://keepachangelog.com/fr/1.1.0/) — versions : [SemVer](https://semver.org/lang/fr/).

## [Non publié]

### Ajouté
- Base du projet : architecture, règles, ADR, contrats entre modules, paramètres fiscaux 2026 (non vérifiés),
  squelette Tauri 2 / Rust / React, CI, prompts des 10 agents.
- Inscription obligatoire des entreprises (ADR 0009) : fiche légale `ac_noyau::entreprise::FicheEntreprise`
  (IFU, RCCM, forme juridique, adresse, responsable…) avec validation partagée application / plateforme.
- Console d'administration web et messagerie ASSISTANCE (ADR 0010) : spécification, module provisoire « Assistance ».
- Plateforme en ligne, lot 1 (ADR 0004, 0008, 0009) : service `plateforme/` (axum + PostgreSQL, hors
  workspace) — comptes (Argon2id), vérification du courriel par code à 6 chiffres, sessions,
  mot de passe oublié, CGU versionnées, inscription des entreprises validée par
  `FicheEntreprise::valider()` (IFU unique, statut `EN_ATTENTE_VERIFICATION`, justificatifs),
  licences signées Ed25519 (émission, renouvellement, transfert de poste limité), limitation de
  débit, Dockerfile + docker-compose (Caddy HTTPS), workflow `plateforme.yml` (tests PostgreSQL).
