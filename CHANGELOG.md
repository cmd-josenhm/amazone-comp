# Journal des modifications

Format : [Keep a Changelog](https://keepachangelog.com/fr/1.1.0/) — versions : [SemVer](https://semver.org/lang/fr/).

## [Non publié]

### Ajouté
- Trésorerie (Lot 1, A6) : comptes de trésorerie (BANQUE, CAISSE, MOBILE_MONEY) avec masquage des
  numéros de compte et de téléphone, écriture AN de solde d'ouverture via `ac_compta::passer_ecriture`
  ; opérateurs Mobile Money en données ; voir `docs/modules/tresorerie.md`.
- Base du projet : architecture, règles, ADR, contrats entre modules, paramètres fiscaux 2026 (non vérifiés),
  squelette Tauri 2 / Rust / React, CI, prompts des 10 agents.
- Inscription obligatoire des entreprises (ADR 0009) : fiche légale `ac_noyau::entreprise::FicheEntreprise`
  (IFU, RCCM, forme juridique, adresse, responsable…) avec validation partagée application / plateforme.
- Console d'administration web et messagerie ASSISTANCE (ADR 0010) : spécification, module provisoire « Assistance ».
