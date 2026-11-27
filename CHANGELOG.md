# Journal des modifications

Format : [Keep a Changelog](https://keepachangelog.com/fr/1.1.0/) — versions : [SemVer](https://semver.org/lang/fr/).

## [Non publié]

### Ajouté
- Base du projet : architecture, règles, ADR, contrats entre modules, paramètres fiscaux 2026 (non vérifiés),
  squelette Tauri 2 / Rust / React, CI, prompts des 10 agents.
- Inscription obligatoire des entreprises (ADR 0009) : fiche légale `ac_noyau::entreprise::FicheEntreprise`
  (IFU, RCCM, forme juridique, adresse, responsable…) avec validation partagée application / plateforme.
- Console d'administration web et messagerie ASSISTANCE (ADR 0010) : spécification, module provisoire « Assistance ».
- Lot 1 « Vérification des sources » (A8) : état des sources et des contradictions
  (`docs/fiscalite/sources.md`), guide des règles fiscales du Bénin adossé aux articles du CGI 2026
  (`docs/fiscalite/regles-fiscales-benin.md`), documentation du module (`docs/modules/fiscal.md`) et tests
  `ac-fiscal` garantissant qu'une `source` cite un texte légal et qu'une valeur vérifiée ne conserve pas
  une source provisoire. 25 points de loi restent ouverts, chacun suivi en issue `fiscal-a-verifier`.

### Modifié
- Paramètres fiscaux 2026 : chaque `source` cite désormais l'article exact du CGI 2026 (ou de la loi de
  finances pour la gestion 2026) ; plancher de l'IBA ramené de 500 000 à 250 000 FCFA (art. 64, 3) a)).
  Toutes les valeurs restent `verifie = false` : seul le chef de projet, après avis d'un expert-comptable,
  peut les valider.
