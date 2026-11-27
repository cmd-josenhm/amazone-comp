# PROMPT A2 — Données locales : base chiffrée, utilisateurs et rôles, audit, sauvegardes

> **Lis d'abord, dans cet ordre :** `AGENTS.md` (règles R1 à R14 et workflow Git, **non négociables**), ce fichier,
> `docs/architecture.md` (§4 et §5), `docs/contrats.md` (C2), `docs/securite.md`, `docs/adr/0005-base-chiffree-sqlcipher.md`.
> Tu ne connais pas le projet : **ne devine rien**. Question ou blocage : issue GitHub `agent:a2` + `question`.

## 1. Le projet en une minute

AMAZONE COMP est un logiciel de comptabilité **gratuit et open source (AGPL-3.0)** pour les **PME du Bénin**, sur
**Windows 10+ 64 bits** (PC de **4 Go de RAM**), utilisable **sans comptable**, **hors ligne d'abord**. Pile : **Tauri 2 + Rust**
+ **React/TypeScript** + **SQLite chiffrée**. **Un poste, un dossier d'entreprise**, plusieurs utilisateurs avec rôles.
Dix agents travaillent en parallèle ; le **chef de projet** relit, teste et fusionne chaque PR.

## 2. Ta mission

Tu garantis que **les données de l'entreprise ne se perdent pas, ne fuient pas et ne sont pas falsifiables** :
ouverture et chiffrement du dossier, gestion des clés, utilisateurs, connexion, rôles, journal d'audit, sauvegardes
locales, instantanés pour la sauvegarde distante, contrôle d'intégrité, moteur de migrations. Toutes les autres crates
reposent sur toi : ton API doit être **simple, stable et documentée**.

## 3. Ton périmètre

| Autorisé | Interdit |
|---|---|
| `crates/ac-donnees/` ; `docs/modules/donnees.md` ; nouveaux ADR (« Proposé ») | Tout le reste, y compris `src-tauri/` (A10) et l'interface (A3). Tu fournis des fonctions Rust ; A3/A10 les branchent. |

## 4. Connaissances indispensables

- Les données comptables sont **légalement** à conserver 10 ans et doivent être **fiables** (intangibilité des écritures).
- Les PC des PME béninoises sont souvent partagés, parfois volés, sujets aux coupures de courant (**SQLite en WAL + `synchronous=NORMAL`**, transactions courtes) et aux virus (sauvegardes déjà chiffrées).
- Le mot de passe sera parfois oublié : la **clé de secours** imprimée à la création est la seule issue. Le dire clairement.
- Rôles V1 (déjà codés dans `session.rs`) : `Administrateur`, `Comptable`, `Saisie`, `Paie`, `Lecture`.

## 5. Ce qui existe déjà

- `Base` (ouverture SQLite **non chiffrée** pour l'instant, mémoire pour tests), `Migration`, `migrer` (ordre global, détection des doublons),
  `audit::journaliser` + `verifier_chaine` (chaînage SHA-256, déclencheurs d'ajout seul), `Session` / `Role` / `Permission` + matrice testée,
  migration `0001_socle.sql` (entreprise, utilisateurs, audit, paramètres).

## 6. Travail à livrer (une PR par lot, dans cet ordre)

### Lot 1 — SQLCipher (≤ 4 jours) — **PRIORITÉ**
- Passer `rusqlite` à la fonctionnalité `bundled-sqlcipher-vendored-openssl`. Elle se déclare dans le `Cargo.toml` **racine**
  (`[workspace.dependencies]`), qui appartient au chef de projet : modifie cette seule ligne dans ta PR et signale-le en tête
  de la description. Le contrôle de périmètre échouera sur ce fichier : c'est attendu, le chef de projet le validera à la main.
- `Base::ouvrir(chemin, &CleBase)` : `PRAGMA key` en hexadécimal brut (`x'…'`), vérification immédiate (lecture de `sqlite_master`),
  erreur claire `MotDePasseIncorrect` si la clé est fausse. Paramètres SQLCipher 4 par défaut.
- Mesurer et noter dans la PR : temps de compilation CI, taille ajoutée au binaire release, temps d'ouverture.

### Lot 2 — Clés et enveloppes (≤ 5 jours)
- `CleBase` : 32 octets aléatoires (`getrandom`), `zeroize` à la destruction, jamais `Debug`/`Display` en clair.
- `enveloppe::sceller(cle, mot_de_passe) -> Enveloppe` / `ouvrir(enveloppe, mot_de_passe) -> CleBase` : Argon2id (paramètres calibrés ~0,5 s sur PC modeste, stockés avec l'enveloppe),
  XChaCha20-Poly1305 (ou AES-256-GCM, justifie par ADR). Enveloppes stockées dans un **fichier d'en-tête séparé** `dossier.acdb.cles` (la base étant chiffrée, on ne peut pas y lire l'enveloppe avant de l'ouvrir).
- **Clé de secours** : 24 mots (liste française de 2048 mots, libre de droits, embarquée) ou code de 32 caractères groupés par 4 ; enveloppe dédiée.
- Changement de mot de passe = nouvelle enveloppe (pas de re-chiffrement de la base). Réinitialisation par l'administrateur pour un autre utilisateur.

### Lot 3 — Dossier, utilisateurs, connexion (≤ 5 jours)
- `dossier::creer(chemin, entreprise: NouvelleEntreprise, admin: NouvelUtilisateur) -> (Base, CleDeSecours)` : crée le fichier, applique **toutes** les migrations reçues en paramètre, l'entreprise et l'administrateur.
- `utilisateurs::{creer, modifier_role, desactiver, lister, changer_mot_de_passe}` (permission `Administrer`, sauf changement de son propre mot de passe).
- `connexion::ouvrir_session(base, identifiant, mot_de_passe) -> Session` : Argon2id (format PHC), **5 échecs → blocage 15 minutes** (horodatage stocké), audit `CONNEXION` / `ECHEC_CONNEXION`.
- Politique : mot de passe ≥ 10 caractères, refus des 1 000 mots de passe les plus courants (liste embarquée).
- Il doit toujours rester **au moins un administrateur actif**.

### Lot 4 — Audit consultable et intégrité (≤ 3 jours)
- `audit::lister(conn, filtre { utilisateur, objet_type, periode }, page)` ; `integrite::verifier(conn) -> RapportIntegrite`
  (`PRAGMA integrity_check`, `foreign_key_check`, chaîne d'audit). Utilisé par A3 dans « Paramètres → Vérifier l'intégrité ».
- Détails d'audit en **JSON canonique** (clés triées) : fournis `audit::details(&impl Serialize) -> String`.

### Lot 5 — Migrations robustes (≤ 2 jours)
- Stocker l'empreinte SHA-256 de chaque migration appliquée ; au démarrage, une migration publiée **modifiée** → erreur bloquante explicite.
- Vérifier que chaque version est dans la plage de sa crate (`plage` passée avec le lot).
- Sauvegarde automatique **avant** d'appliquer des migrations à une base existante.

### Lot 6 — Sauvegardes locales et instantanés (≤ 5 jours)
- `sauvegarde::creer(base, dossier) -> CheminSauvegarde` via `VACUUM INTO` (fichier toujours chiffré avec la même clé) + fichier de métadonnées (date, version du logiciel, empreinte SHA-256).
- Rotation : 7 quotidiennes, 4 hebdomadaires, 12 mensuelles. `sauvegarde::restaurer(chemin, cle)` : vérification d'empreinte + ouverture test **avant** remplacement, copie de l'actuel en `avant-restauration-<date>`.
- `instantane::exporter(base) -> Vec<u8>` / `importer(octets, cle)` : flux binaire chiffré auto-décrit (en-tête versionné) que A10 envoie à la plateforme. **La plateforme ne doit jamais pouvoir le déchiffrer.**
- Exporter une copie vers une clé USB (`sauvegarde::copier_vers(chemin)`).

## 7. Contrats

- **Tu fournis** C2 : `Base`, `Transaction`, `Connexion`, `Migration`, `audit`, `Session`/`Role`/`Permission`, `dossier`, `utilisateurs`, `connexion`, `sauvegarde`, `instantane`, `integrite`.
  Toute modification de signature publique existante = ADR (A1, A4–A10 en dépendent).
- **Tu consommes** : `ac-noyau` uniquement.

## 8. Tests exigés

- Ouverture avec mauvais mot de passe → `MotDePasseIncorrect` (et le fichier n'est pas modifié).
- Fichier ouvert sans clé par un SQLite standard → illisible (test : les 16 premiers octets ne sont pas « SQLite format 3 »).
- Clé de secours → accès retrouvé après « oubli » du mot de passe ; changement de mot de passe → l'ancien ne fonctionne plus.
- 5 échecs de connexion → blocage ; 15 minutes plus tard (horloge injectée) → autorisé.
- Altération directe d'une ligne d'audit (via une connexion qui supprime le déclencheur) → `verifier_chaine` renvoie `AuditAltere(n)`.
- Sauvegarde puis restauration → mêmes données, même chaîne d'audit ; restauration d'un fichier corrompu → refus, base actuelle intacte.
- Rotation : 40 jours simulés → exactement les sauvegardes attendues.
- Migration modifiée après application → erreur explicite.
- Impossible de désactiver le dernier administrateur.

## 9. Pièges à éviter

- Jamais de clé, de mot de passe ou de hash dans les journaux, les messages d'erreur ou `Debug`.
- Pas d'horloge système en dur dans la logique testable : injecter un `Horloge` (trait) pour les tests.
- Ne pas bloquer l'interface : opérations longues (sauvegarde, vérification) conçues pour être appelées hors du fil principal.
- Windows : chemins avec accents/espaces (`C:\Users\Sèna Dossou\…`) → tests avec de tels noms.

## 10. Définition de « terminé »

CI verte, couverture ≥ 80 %, `docs/modules/donnees.md` (API, format de l'en-tête de clés, format d'instantané, procédure de récupération), CHANGELOG, PR fusionnée.

## 11. Pour démarrer

```powershell
git checkout main; git pull
git checkout -b a2/feat-sqlcipher
cargo test -p ac-donnees
```
