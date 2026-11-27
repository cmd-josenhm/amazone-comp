# Plateforme en ligne AMAZONE COMP — Propriétaire : A10

Service unique **Rust (axum) + PostgreSQL**, déployé en conteneur Docker sur un VPS (ADR 0008).
Crate binaire **séparé** du workspace de l'application (son propre `Cargo.toml` et `Cargo.lock`) :
il dépend de `ac-noyau` et `ac-licence` par chemin pour la validation unique de la fiche
entreprise (C11) et le format du jeton de licence (ADR 0004).

**État : lot 1 livré** (comptes, vérification du courriel, sessions, mot de passe oublié,
CGU versionnées, inscription des entreprises, licences signées Ed25519). Les lots suivants
(client `ac-licence`, glue `src-tauri`, console d'administration, ASSISTANCE, sauvegardes,
mises à jour) viendront ensuite — voir `PROMPT_A10.md`.

Arborescence :

```
plateforme/
├── Cargo.toml            crate binaire « amazone-plateforme » (hors workspace de l'application)
├── migrations/           SQL PostgreSQL (sqlx)
├── src/
│   ├── main.rs           démarrage, configuration, écoute HTTP
│   ├── lib.rs            état partagé, authentification, routeur
│   ├── config.rs         variables d'environnement, clé de signature
│   ├── erreur.rs         erreurs API (messages français)
│   ├── jetons.rs         jetons opaques, codes, jetons de licence Ed25519
│   ├── limitation.rs     limitation de débit (couche tower)
│   ├── courriel.rs       envoi des codes (journal, futur SMTP)
│   ├── comptes.rs        inscription, vérification, sessions, mot de passe oublié
│   ├── cgu.rs            versions des conditions, acceptations
│   ├── entreprises.rs    fiche entreprise, statuts, justificatifs (ADR 0009)
│   ├── licences.rs       émission / renouvellement / transfert (Ed25519)
│   └── (à venir) assistance.rs, annonces.rs, sauvegardes.rs, avis.rs,
│       mises_a_jour.rs, admin/ (lots suivants)
├── web-admin/            console d'administration (lot 7)
├── tests/parcours.rs     tests d'intégration (PostgreSQL requis)
├── Dockerfile
├── docker-compose.yml    plateforme + postgres + caddy
├── Caddyfile             HTTPS Let's Encrypt
└── .env.exemple          variables (aucun secret réel)
```

**La clé privée de signature des licences n'est jamais commitée** (`plateforme/secrets/` et
`.env` sont ignorés par Git) : elle est lue dans une variable d'environnement ou un fichier
hors dépôt. La clé publique est affichée au démarrage du service.

Documentation complète : `docs/modules/plateforme.md` (API, déploiement, rotation des clés).
