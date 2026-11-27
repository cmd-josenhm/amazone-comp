# PROMPT A6 — Trésorerie : banques, caisse, Mobile Money, règlements, rapprochement, import de relevés

> **Lis d'abord, dans cet ordre :** `AGENTS.md` (règles R1 à R14, **non négociables**), ce fichier, `docs/architecture.md`,
> `docs/contrats.md` (C3, C5), `docs/fiscalite/regles-fiscales-benin.md` (§6).
> Tu ne connais ni le projet ni le Bénin : **ne devine rien**. Question : issue GitHub `agent:a6` + `question`.

## 1. Le projet en une minute

AMAZONE COMP est un logiciel de comptabilité **gratuit et open source** pour les **PME du Bénin**, Windows 10+ 64 bits, utilisable
**sans comptable**, **hors ligne d'abord**. Pile : **Tauri 2 + Rust** + **React/TypeScript** + **SQLite chiffrée**. **FCFA entiers**.
**SYSCOHADA révisé**. Dix agents en parallèle ; le **chef de projet** valide chaque PR.

## 2. Ta mission

Tu suis **l'argent réel** de l'entreprise : comptes bancaires, caisse(s), portefeuilles **Mobile Money**, encaissements des clients,
paiements des fournisseurs et des salariés, dépenses courantes, virements internes, **rapprochement bancaire**, et
**import des relevés** bancaires et Mobile Money. Le Mobile Money est **facultatif** (activable dans les paramètres) et doit pouvoir
évoluer (nouveaux opérateurs, nouveaux formats) **sans changer le code** du reste.

## 3. Ton périmètre

| Autorisé | Interdit |
|---|---|
| `crates/ac-tresorerie/`, `ui/src/modules/tresorerie/`, `src-tauri/src/commandes/tresorerie.rs`, ta section de `commandes/mod.rs`, `docs/modules/tresorerie.md`, ADR « Proposé » | Tout le reste. Tu ne lis **pas** les tables de A4/A5/A7 : les montants dus se lisent dans `ac-compta` (lignes non lettrées 411/401/422 par tiers). |

## 4. Connaissances métier indispensables (Bénin)

- **Mobile Money** très répandu : **MTN MoMo**, **Moov Money**, **Celtiis Cash**. Chaque portefeuille (numéro de téléphone marchand) est un compte de trésorerie
  distinct : SYSCOHADA révisé classe 55 « instruments de monnaie électronique » (sous-compte exact à valider : 552/554…), avec son journal `MMx`.
  Les **frais** de transaction et de retrait sont des charges (631 frais bancaires ou compte à valider).
- **Banques** (521x, journaux `BQx`) : virements, chèques, prélèvements, agios. Formats de relevés **différents selon la banque** (BOA, Ecobank, Orabank, UBA, NSIA, SGB, BIIC, Coris, BSIC…) : **jamais de format codé en dur**, des **profils** configurables.
- **Caisse** (571x, journal `CA`) : espèces ; un solde de caisse **ne peut pas être négatif** (blocage avec message clair) ; arrêté de caisse (comptage des billets) quotidien conseillé.
- **Virements internes** (banque → caisse, MoMo → banque) : compte **585 virements de fonds** pour éviter les doublons.
- **Règlement d'une facture** : on débite la trésorerie et on crédite **411 du client** (ou on débite **401 du fournisseur**), puis on **lettre** la facture et le règlement (fonction `ac_compta::lettrer`). Paiement partiel : pas de lettrage tant que la somme n'est pas nulle ; on affiche le reste dû.

## 5. Ce qui existe déjà

Crate vide (plage de migrations 4000–4999). A1 fournit `passer_ecriture`, `lignes_non_lettrees`, `lettrer`, `journal::creer` ; A1 fournit `ac_documents::lire_tableau` (CSV/XLSX).

## 6. Travail à livrer (une PR par lot)

### Lot 1 — Comptes de trésorerie (≤ 3 jours)
Table `tresorerie_compte` (type BANQUE|CAISSE|MOBILE_MONEY, libellé, banque/opérateur, n° de compte ou de téléphone (masqué à l'affichage sauf 4 derniers chiffres), compte comptable créé comme sous-compte, journal créé, solde d'ouverture → écriture `AN`, actif).
Opérateurs Mobile Money en **données** (`tresorerie_operateur` : MTN MoMo, Moov Money, Celtiis Cash pré-remplis, ajout possible). Module Mobile Money désactivable.

### Lot 2 — Opérations (≤ 6 jours)
Encaissement client (choix du client → liste de ses factures ouvertes depuis `ac-compta` → règlement total/partiel → lettrage automatique si soldé),
paiement fournisseur (idem 401), paiement de salaires (422, lot depuis la paie : A7 te transmet l'état par une commande), dépense directe sans facture fournisseur saisie (avec pièce, compte de charge, TVA déductible seulement si facture normalisée — catégorie demandée),
recette diverse, virement interne (585), frais et agios, retrait/dépôt d'espèces. Chaque opération = une écriture dans le journal du compte, en **une transaction**, avec pièce justificative.

### Lot 3 — Import de relevés (≤ 6 jours)
- **Profils d'import** (`tresorerie_profil_import`) : séparateur, encodage, ligne d'en-tête, colonnes (date, libellé, débit, crédit **ou** montant signé, référence, solde), format de date. Profil enregistré par compte.
- Assistant : fichier → aperçu → correspondance des colonnes (mémorisée) → contrôle (dates, montants entiers, solde final cohérent si fourni) → **dédoublonnage** (empreinte date+montant+référence+libellé) → lignes de relevé stockées (`tresorerie_releve_ligne`), **sans écriture automatique**.
- Proposition d'écritures : règles simples configurables (« libellé contient FRAIS → 631 »), suggestions de rapprochement avec les factures ouvertes (même montant, référence) ; l'utilisateur valide.
- Formats Mobile Money réels inconnus : **demande au chef de projet des exemples anonymisés** (issue) et livre des profils de démonstration ; OFX facultatif.

### Lot 4 — Rapprochement bancaire (≤ 4 jours)
Pointage des lignes de relevé avec les lignes comptables du compte (auto + manuel), **état de rapprochement** à une date (solde banque, opérations non pointées des deux côtés, solde comptable) en PDF/Excel via `ac_documents`. Pointage verrouillé une fois l'état validé.

### Lot 5 — Caisse (≤ 2 jours)
Brouillard de caisse du jour, arrêté de caisse (billetage FCFA : 10 000, 5 000, 2 000, 1 000, 500 ; pièces 500, 250, 200, 100, 50, 25, 10, 5), écart de caisse → écriture sur compte d'écart à valider.

### Lot 6 — Interface « Trésorerie » (≤ 5 jours)
Tableau des soldes par compte, saisie rapide des opérations, encaissement depuis la fiche client (appelé aussi par l'écran de vente au comptoir de A4 : expose une route `/tresorerie/encaisser?client=…&montant=…`), import, rapprochement, caisse. Commandes `tresorerie_*`.

## 7. Contrats

- **Tu fournis** : commandes `tresorerie_*`, route d'encaissement, `tresorerie_soldes` (pour le tableau de bord de A9), `tresorerie_payer_salaires` (pour A7).
- **Tu consommes** : C3 + `lettrer`, `lignes_non_lettrees`, `journal::creer` (A1), `ac_documents::lire_tableau` et rendu (A1), C2 (A2).

## 8. Tests exigés (exemples chiffrés)

- Encaissement total d'une facture de 17 850 par MTN MoMo → 55x D 17 850 / 411 C 17 850 → facture lettrée, reste dû 0.
- Encaissement partiel 10 000 → non lettré, reste dû 7 850 ; second encaissement 7 850 → lettrage.
- Retrait de caisse de 60 000 avec 50 000 en caisse → refusé.
- Virement banque → caisse 100 000 → deux écritures via 585, 585 soldé.
- Import d'un CSV Windows-1252 avec accents et séparateur `;`, montants « 1 250 000 » → lignes correctes ; réimport → 0 doublon.
- Rapprochement : relevé 500 000, comptabilité 520 000, chèque émis non débité 20 000 → état équilibré.

## 9. Pièges à éviter

- Montants des relevés : jamais `parseFloat` ; les montants « 1 250 000,00 » sont convertis en entiers **en Rust** (et refusés s'il y a des centimes non nuls).
- Ne pas créer d'écriture automatiquement à l'import sans validation de l'utilisateur.
- Numéros de compte et de téléphone : données sensibles, masquées dans les listes et les journaux.

## 10. Définition de « terminé »

CI verte, couverture ≥ 80 %, `docs/modules/tresorerie.md` (comptes, écritures, format des profils d'import), CHANGELOG, PR fusionnée.

## 11. Pour démarrer

```powershell
git checkout main; git pull
git checkout -b a6/feat-comptes-tresorerie
cargo test -p ac-tresorerie
```
