# Journal des modifications

Format : [Keep a Changelog](https://keepachangelog.com/fr/1.1.0/) — versions : [SemVer](https://semver.org/lang/fr/).

## [Non publié]

### Ajouté
- Base du projet : architecture, règles, ADR, contrats entre modules, paramètres fiscaux 2026 (non vérifiés),
  squelette Tauri 2 / Rust / React, CI, prompts des 10 agents.
- Inscription obligatoire des entreprises (ADR 0009) : fiche légale `ac_noyau::entreprise::FicheEntreprise`
  (IFU, RCCM, forme juridique, adresse, responsable…) avec validation partagée application / plateforme.
- Console d'administration web et messagerie ASSISTANCE (ADR 0010) : spécification, module provisoire « Assistance ».
- Noyau finalisé (issue #9) : quantités exactes en millièmes `ac_noyau::Quantite` (jamais de flottant, règle R1),
  `Montant::multiplier_quantite` (prix unitaire × quantité avec règle d'arrondi explicite) et `Montant::repartir`
  (répartition proportionnelle sans perte d'un franc, la différence d'arrondi allant à la plus grande part),
  avec tests de propriété (`proptest`).
