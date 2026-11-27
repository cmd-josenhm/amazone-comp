# ADR 0001 — Pile technique : Tauri 2 + Rust + React/TypeScript + SQLite

- Statut : **Accepté** (chef de projet, 2026-11-27)

## Contexte
Application Windows 10+ 64 bits, PC de 4 Go de RAM, hors ligne d'abord, gratuite (pas de licence logicielle payante), calculs financiers exacts.

## Décision
- **Tauri 2** (WebView2 fourni par Windows) plutôt qu'Electron (Chromium embarqué : ~150 Mo d'installateur, 300+ Mo de RAM).
- **Rust** pour tout le métier : sûreté mémoire, rapidité, typage fort des montants, un seul binaire.
- **React + TypeScript strict + Vite** pour l'interface : écosystème connu des agents, rendu léger. Aucune bibliothèque de composants : design system maison.
- **SQLite** (fichier unique, sans serveur) chiffré par SQLCipher (ADR 0005).
- **Plateforme en ligne** : Rust (axum) + PostgreSQL sur un seul VPS (ADR 0008).

## Conséquences
- Installateur visé < 15 Mo, RAM < 150 Mo.
- WebView2 requis : déjà présent sur Windows 10 (mises à jour) et 11 ; l'installateur l'installe si absent (`downloadBootstrapper`).
- Les agents doivent connaître Rust : toute la logique est testable sans interface.
