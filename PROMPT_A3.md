# PROMPT A3 — Interface : coque, design system, assistant de démarrage, Paramètres, À propos et avis

> **Lis d'abord, dans cet ordre :** `AGENTS.md` (règles R1 à R14, **non négociables**), ce fichier, `docs/architecture.md` (§8),
> `docs/contrats.md` (C8, C9), `docs/design-system.md`.
> Tu ne connais pas le projet : **ne devine rien**. Question ou blocage : issue GitHub `agent:a3` + `question`.

## 1. Le projet en une minute

AMAZONE COMP est un logiciel de comptabilité **gratuit et open source** pour les **PME du Bénin**, sur **Windows 10+ 64 bits**
(PC de **4 Go de RAM**), utilisé par des **non-comptables** (commerçants, artisans, gérants). Interface **en français uniquement**,
**hors ligne d'abord**. Pile : **Tauri 2** (WebView2) + **Rust** (tout le métier) + **React 19 / TypeScript strict / Vite**.
Design **sobre noir et blanc**, inspiré de la clarté de Google. Dix agents travaillent en parallèle ; le **chef de projet** valide chaque PR.

## 2. Ta mission

Tu construis **tout ce qui n'est pas un écran métier** : la coque (menu, barre, recherche), le **design system** que les
9 autres agents utilisent, la connexion et le verrouillage, l'**assistant de premier lancement**, les **Paramètres**,
la page **À propos** avec le **formulaire d'avis/critiques**, l'aide et l'accessibilité. Tu es le garant de la
**cohérence visuelle** et de la **simplicité** pour un utilisateur qui n'est pas comptable.

## 3. Ton périmètre

| Autorisé | Interdit |
|---|---|
| `ui/` **sauf** les dossiers `ui/src/modules/<module>/` des autres agents ; `ui/src/modules/registre.ts` et `ui/src/modules/types.ts` sont à toi ; `src-tauri/src/commandes/systeme.rs` ; `docs/modules/interface.md` | `ui/src/modules/{comptabilite,ventes,achats,tresorerie,paie,fiscal,etats,tableau-de-bord,compte}/`, **`ui/src/pages/a-propos/proprietaire.ts`** (seul le chef de projet y met les informations personnelles du propriétaire), tout le Rust hors `systeme.rs`. |

## 4. Connaissances indispensables

- Utilisateurs : souvent **peu familiers de l'informatique**, écrans de 1366×768, souris parfois absente (pavé tactile), coupures de courant.
- **Aucun calcul monétaire ou fiscal en TypeScript** (règle R1) : l'interface affiche ce que Rust renvoie. Montants reçus en **entiers** FCFA, affichés avec `formaterMontant`.
- Vocabulaire : terme simple d'abord, terme comptable entre parenthèses. « Argent à recevoir des clients (411) ».
- **Polices système uniquement** (Segoe UI), **aucune ressource réseau** (CSP stricte dans `tauri.conf.json`).

## 5. Ce qui existe déjà

- `ui/src/design/` : jetons CSS, `Bouton`, `Champ`, `Carte`, `Bandeau`, `Montant`, `formaterMontant`.
- `ui/src/coque/Coque.tsx` (menu à partir du registre des modules), `App.tsx` (HashRouter), `lib/ipc.ts` (`appeler`, `ErreurAppel`, `dansTauri`).
- Pages : `Parametres.tsx` (choix mises à jour automatique/manuel, non enregistré), `Demarrage.tsx` (7 étapes, texte seulement),
  `a-propos/APropos.tsx` (créateur avec champs « A COMPLETER », formulaire d'avis qui appelle `compte_envoyer_avis`).
- Tests Vitest + Testing Library (`App.test.tsx`). ESLint strict, `tsc` strict.

## 6. Travail à livrer (une PR par lot)

### Lot 1 — Design system complet (≤ 6 jours) — **PRIORITÉ** (les autres agents en ont besoin)
Composants accessibles (rôles ARIA, clavier, focus visible), chacun avec tests et une page de démonstration interne `/#/demo-design` (visible en développement seulement) :
`Tableau` (pagination côté Rust : props `page`, `total`, `surPage` ; tri ; ligne cliquable ; colonnes montants alignées à droite),
`ChampMontant` (saisie d'entiers avec séparateurs de milliers, refuse décimales et lettres, renvoie un `number` entier),
`ChampDate` (JJ/MM/AAAA, calendrier simple), `Selecteur` (avec recherche), `CaseACocher`, `Dialogue` (confirmation, focus piégé, Échap),
`Assistant` (étapes, précédent/suivant, résumé avant validation), `Onglets`, `Notification` (toasts), `EtatVide`, `Chargement`, `MessageErreur` (affiche `ErreurAppel.message` + action proposée).
Documenter chaque composant dans `docs/design-system.md` (section « Composants », la tienne).

### Lot 2 — Coque finale et performance (≤ 3 jours)
- Chargement paresseux de chaque module (`React.lazy`) : seul le module affiché est en mémoire.
- Barre d'état : utilisateur connecté, rôle, état de la licence (commande A10 `compte_etat_licence`), année fiscale + bandeau permanent
  « N valeurs fiscales ne sont pas encore validées par un expert » (commande existante `systeme_infos`).
- Raccourcis clavier globaux (Ctrl+K recherche, Ctrl+N nouveau selon le module, F1 aide) documentés dans une fenêtre d'aide.
- Mesurer la RAM (Gestionnaire des tâches, processus WebView2) avec 3 modules ouverts : cible < 150 Mo, résultat dans la PR.

### Lot 3 — Connexion et verrouillage (≤ 3 jours)
- Écran de connexion (identifiant, mot de passe, « Mot de passe oublié → clé de secours »), messages des erreurs A2 (blocage après 5 échecs).
- Verrouillage automatique après 15 minutes d'inactivité (durée réglable dans Paramètres), déverrouillage par mot de passe.
- Les commandes Rust (`dossier_*`, `session_*`) sont fournies par A10 (glue `src-tauri`) à partir des fonctions de A2 : **ouvre une issue** listant
  précisément les commandes et types attendus. En attendant, un **adaptateur simulé** (`ui/src/lib/simulation.ts`) activé quand `!dansTauri()` permet de développer dans le navigateur.

### Lot 4 — Assistant de premier lancement (≤ 6 jours)
Les 7 étapes de `Demarrage.tsx`, chacune un écran de l'`Assistant` :
1. **Conditions d'utilisation** : texte fourni par la plateforme (A10) ; case « J'ai lu et j'accepte » obligatoire ; version enregistrée.
2. **Compte AMAZONE COMP** : créer un compte ou se connecter (Internet nécessaire **une seule fois**, message clair si hors ligne).
3. **Entreprise** : raison sociale, IFU (13 chiffres, validé), RCCM, forme juridique (EI, SARL, SA, SAS, SNC, GIE, autre), adresse, téléphone,
   **régime fiscal** (réel normal / réel simplifié / TPS) avec une **aide au choix** simple (questions : forme juridique, CA annuel estimé) — la règle de décision est fournie par A8 (commande `fiscal_suggerer_regime`), pas codée ici.
4. **Administrateur** : identifiant, mot de passe (jauge de robustesse), **affichage et impression de la clé de secours** avec case « Je l'ai notée et rangée ».
5. **Comptabilité** : date de début d'exercice, système (normal / minimal de trésorerie, suggestion par A8), comptes de trésorerie (banques, caisse, Mobile Money : MTN MoMo, Moov Money, Celtiis — **facultatifs**).
6. **Reprise de données** (facultative) : renvoie vers l'assistant « Reprise » de A9.
7. **Terminé** : visite guidée en 5 bulles du tableau de bord.

### Lot 5 — Paramètres (≤ 4 jours)
Sections : **Dossier** (infos entreprise, régime — changement réservé à l'administrateur avec avertissement), **Utilisateurs et rôles** (liste, ajout, rôle, désactivation, réinitialisation),
**Mises à jour** (automatique / manuel, « Rechercher maintenant », version installée — logique A10), **Sauvegardes** (locales : dossier, fréquence, « Sauvegarder maintenant », « Restaurer » ; en ligne : renvoi vers le module Compte),
**Sécurité** (délai de verrouillage, « Vérifier l'intégrité des données » → rapport A2, journal d'audit consultable), **Modules facultatifs** (activer/désactiver Mobile Money, Stocks, Paie).

### Lot 6 — À propos et avis (≤ 2 jours)
- Garder la structure : logiciel (version, licence AGPL-3.0, lien vers le code source, crédits des bibliothèques libres générés automatiquement), **le créateur** (lu depuis `proprietaire.ts` : nom, photo, contact, LinkedIn, GitHub, autres — **ne remplis rien toi-même**, les valeurs restent « A COMPLETER »),
  **formulaire d'avis** (type, note 1–5 facultative, message, coordonnées facultatives) → `compte_envoyer_avis` (A10 met l'avis en file d'attente hors ligne).
- Lister « Mes avis envoyés » (commande A10 `compte_lister_avis`) avec leur statut (en attente / envoyé).

## 7. Contrats

- **Tu fournis** C9 (`DeclarationModule`, registre) et le design system. Changer une prop publique d'un composant utilisé = prévenir par issue + ADR si cassant.
- **Tu consommes** les commandes des autres (liste dans `docs/modules/interface.md`, tenue par toi) ; ne les invente pas : demande-les par issue.

## 8. Tests exigés

- Chaque composant : rendu, clavier (Tab, Entrée, Échap, flèches), états d'erreur, accessibilité (`getByRole` avec nom).
- `ChampMontant` : « 1 250 000 » → 1250000 ; « 12,5 » refusé ; « -500 » seulement si `negatifAutorise`.
- Assistant : impossible d'avancer sans accepter les CGU ; IFU de 12 chiffres → erreur affichée ; clé de secours non confirmée → bloqué.
- Couverture ≥ 80 % de `ui/src/design`, `ui/src/coque`, `ui/src/pages` (`npm run --workspace ui test:couverture`).

## 9. Pièges à éviter

- Aucune bibliothèque UI externe (MUI, Ant, Bootstrap, Tailwind…), aucune police ou icône téléchargée ; icônes en SVG inline minimalistes si nécessaire.
- Aucune couleur en dur hors `jetons.css` ; pas de `dangerouslySetInnerHTML`.
- Ne jamais stocker de données sensibles dans `localStorage`.
- Textes : pas d'anglais visible (« Loading… » interdit → « Chargement… »).

## 10. Définition de « terminé »

`npm run lint` et `npm run test` verts, couverture ≥ 80 %, captures d'écran dans la PR, `docs/modules/interface.md` et `docs/design-system.md` à jour, CHANGELOG, PR fusionnée.

## 11. Pour démarrer

```powershell
git checkout main; git pull
git checkout -b a3/feat-design-tableau
npm install
npm run --workspace ui dev
```
