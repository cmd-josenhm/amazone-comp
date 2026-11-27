# Plateforme en ligne AMAZONE COMP — Propriétaire : A10

Service unique **Rust (axum) + PostgreSQL**, déployé sur un VPS (ADR 0008). Rien n'est encore écrit :
A10 crée ce dossier en suivant `PROMPT_A10.md`.

Arborescence attendue :

```
plateforme/
├── Cargo.toml            crate binaire « amazone-plateforme » (hors workspace de l'application)
├── migrations/           SQL PostgreSQL (sqlx)
├── src/
│   ├── main.rs
│   ├── comptes.rs        inscription, connexion, vérification e-mail, mot de passe oublié
│   ├── cgu.rs            versions des conditions, acceptations
│   ├── licences.rs       émission / renouvellement / révocation / transfert (Ed25519)
│   ├── sauvegardes.rs    dépôt et récupération de blobs chiffrés
│   ├── avis.rs           avis et critiques des utilisateurs
│   ├── mises_a_jour.rs   manifeste du plugin updater de Tauri
│   └── admin.rs          tableau de bord minimal du propriétaire
├── Dockerfile
├── docker-compose.yml    plateforme + postgres + caddy
└── .env.exemple          variables (aucun secret réel)
```

**La clé privée de signature des licences n'est jamais commitée** (`plateforme/secrets/` est ignoré par Git).
