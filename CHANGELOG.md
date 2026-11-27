# Journal des modifications

Format : [Keep a Changelog](https://keepachangelog.com/fr/1.1.0/) — versions : [SemVer](https://semver.org/lang/fr/).

## [Non publié]

### Ajouté
- Base du projet : architecture, règles, ADR, contrats entre modules, paramètres fiscaux 2026 (non vérifiés),
  squelette Tauri 2 / Rust / React, CI, prompts des 10 agents.
- Inscription obligatoire des entreprises (ADR 0009) : fiche légale `ac_noyau::entreprise::FicheEntreprise`
  (IFU, RCCM, forme juridique, adresse, responsable…) avec validation partagée application / plateforme.
- Console d'administration web et messagerie ASSISTANCE (ADR 0010) : spécification, module provisoire « Assistance ».
- Paie (A7, Lot 1) : `ac_paie::ParametresSociaux`, paramètres sociaux typés chargés par année de l'opération, avec source par valeur ; ITS, base et périodicité CNSS, majorations non tranchées (issues #43 à #49).
