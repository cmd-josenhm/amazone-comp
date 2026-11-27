# Contrats entre modules

Ce document fige les interfaces dont dépendent plusieurs agents. **Modifier un contrat = ADR + accord du
chef de projet** (règle R11). Le code de référence est indiqué pour chaque contrat : en cas de doute,
c'est le code qui fait foi, et ce document doit être corrigé.

## C1. Types de base — `ac-noyau` (A1)

| Type | Rôle | Règles |
|---|---|---|
| `Montant` | FCFA entiers (`i64`), borné à ±10^15 | Jamais de flottant. Sérialisé en entier JSON. `appliquer_taux(taux, Arrondi)` pour tout calcul en pourcentage. |
| `Taux` | Fraction exacte (`Decimal`), 0 ≤ t ≤ 1 | Sérialisé en texte (`"0.18"`). Construit par `Taux::pourcentage("18")` ou `Taux::fraction("0.18")`. |
| `Arrondi` | `PlusProche` / `Inferieur` / `Superieur` | Chaque calcul choisit explicitement sa règle et la documente. |
| `Periode` | Intervalle de dates fermé | `Periode::mois(a, m)`, `Periode::annee(a)`. |
| `Ifu` | 13 chiffres | Validé à la saisie. |
| `entreprise::{FicheEntreprise, Regime, FormeJuridique, Departement, SecteurActivite, Rccm, Telephone}` | Fiche légale de l'entreprise | Voir C11 et ADR 0009. `ac_fiscal::Regime` est un ré-export. |
| `parametres::Valeur<T>` | `{ valeur, source, verifie, note }` | Toute valeur réglementaire. |

Code : `crates/ac-noyau/src/`.

## C2. Données — `ac-donnees` (A2)

- `Base::ouvrir(chemin)`, `Base::en_memoire()` (tests), `Base::migrer(&[lots])`, `Base::transaction()`.
- `Migration { version, nom, sql }`, plages réservées (architecture §4). Chaque crate expose `pub fn migrations()`.
- `audit::journaliser(&tx, &Evenement)` : **dans la même transaction** que la modification.
- `Session { utilisateur_id, role }`, `Session::exiger(Permission)`. Permissions : `Consulter`, `Saisir`, `Valider`, `GererPaie`, `Declarer`, `Cloturer`, `Administrer`.
- Les autres crates utilisent `ac_donnees::rusqlite` (ré-export) : jamais une autre version de rusqlite.

## C3. Écritures — `ac-compta` (A1)

```rust
pub fn passer_ecriture(tx: &Transaction, session: &Session, b: &BrouillonEcriture) -> Result<EcritureId, ErreurCompta>;
pub fn verifier_brouillon(b: &BrouillonEcriture) -> Result<(), ErreurCompta>;
```

- `BrouillonEcriture { journal, date, libelle, piece, origine: ModuleOrigine, reference_origine, lignes }`.
- `LigneBrouillon { compte: NumeroCompte, libelle, debit, credit, tiers, code_fiscal, base_fiscale, taux }`.
- **Codes journaux standard** : `VE` ventes, `AC` achats, `BQ` banque (un par compte : `BQ1`, `BQ2`…), `CA` caisse, `MM` Mobile Money (`MM1`…), `PA` paie, `OD` opérations diverses, `AN` à-nouveaux, `ST` stocks.
- **Tiers** : code texte stable préfixé : `CLI-000123` (clients, A4), `FRS-000045` (fournisseurs, A5), `SAL-000007` (salariés, A7). Obligatoire sur les comptes 40x, 41x, 42x.
- **Une écriture validée est immuable** (déclencheurs SQL). Correction = contre-passation (A1 fournit `contre_passer`).
- Fonctions de lecture à fournir par A1 : `solde_compte`, `balance`, `grand_livre`, `lignes_par_code_fiscal(periode)`, `lettrer`.

## C4. Étiquettes fiscales — `ac_compta::CodeFiscal` (A1 + A8)

Chaque ligne d'écriture ayant un effet fiscal porte un `CodeFiscal` et, si pertinent, sa `base_fiscale` (HT) et son `taux`.
`ac-fiscal` prépare les déclarations **uniquement** à partir de ces lignes.

| Module émetteur | Lignes à étiqueter |
|---|---|
| Ventes (A4) | `TvaCollectee` (base HT, taux), `CaExonere`, `CaExport`, `AibCollecte`, `TvaRetenueParClient` |
| Achats (A5) | `TvaDeductibleBiens` / `Services` / `Immobilisations`, `AibSupporte`, `RetenueLoyer`, `RetenueNonResident`, `TvaRetenueSurFournisseur` |
| Paie (A7) | `Its`, `Vps`, `CnssSalarie`, `CnssEmployeur`, `RedevanceAudiovisuelle` |
| Fiscal (A8) | `ImpotResultat`, `AcompteImpotResultat`, `Tps` (écritures de liquidation) |

## C5. Comptes SYSCOHADA de référence (à valider par l'expert-comptable)

| Usage | Compte |
|---|---|
| Clients | 411 |
| Fournisseurs | 401 (immobilisations : 481) |
| Personnel, rémunérations dues | 422 |
| CNSS | 431 |
| État, impôt sur les bénéfices | 441 |
| État, TVA facturée | 443 (4431 ventes, 4432 prestations…) |
| État, TVA récupérable | 445 (4451 immobilisations, 4452 achats, 4454 services…) |
| État, TVA due / crédit de TVA | 4441 / 4449 |
| État, impôts retenus à la source (ITS, retenues) | 447 |
| Banques | 521 |
| Instruments de monnaie électronique (Mobile Money) | 55 (à confirmer : 552 / 554) |
| Caisse | 571 |
| Ventes de marchandises / services | 701 / 706 |
| Achats de marchandises | 601 |
| Rémunérations du personnel | 661 |
| Charges sociales | 664 |
| Impôts et taxes (VPS…) | 64 |
| Impôt sur le résultat | 891 (IMF : 895) |

A1 livre le plan complet ; A4–A8 utilisent **uniquement** des comptes du plan via des constantes nommées
déclarées dans leur crate (`const COMPTE_CLIENTS: &str = "411";`) — jamais de numéro dispersé dans le code.

## C6. Facture normalisée — `ac_ventes::normalisation` (A4)

Trait `Normaliseur` : `soumettre(&DemandeNormalisation) → FactureSoumise` puis `finaliser(uid, Decision) → Option<FactureNormalisee>`.
Implémentations : `Simulateur` (disponible), `EMecefApi` (à venir). Une facture n'est **définitive** qu'après `finaliser(Confirmer)`.
Les éléments de sécurité (`code_mecef`, `nim`, `compteurs`, `date_heure`, `qr_code`) sont stockés et imprimés.

## C7. Documents — `ac-documents` (A1)

`Document { titre, entete_entreprise, blocs: Vec<Bloc>, pied_de_page }` → `rendre_pdf(&Document)` / `rendre_xlsx(&Document)`.
Les modules construisent un `Document` ; seul `ac-documents` connaît les bibliothèques PDF / Excel.

## C8. Commandes IPC (tous)

- Nom `module_action` (snake_case), déclarées dans `src-tauri/src/commandes/<module>.rs`, enregistrées dans `commandes/mod.rs`.
- Signature : `fn x(etat: State<EtatApplication>, …) -> Result<T, ErreurCommande>` avec `T: Serialize`. Champs en **snake_case français, sans `rename_all = "camelCase"`** : les noms sont identiques côté TypeScript (`montant_ht`, pas `montantHt`).
- Côté interface : `appeler<T>("module_action", { … })` depuis `ui/src/lib/ipc.ts`. Types TypeScript du module dans `ui/src/modules/<module>/types.ts`.
- Montants transmis en **entiers** (FCFA), taux en **texte**.

## C9. Modules d'interface (A3 + chaque agent)

`ui/src/modules/<module>/module.tsx` exporte `declaration: DeclarationModule` (`id`, `libelle`, `chemin`, `ordre`, `Page`).
Les composants viennent de `ui/src/design/composants.tsx`. Aucun style de couleur en dur : jetons CSS uniquement.

## C10. Paramètres réglementaires (A8, A7)

`parametres-fiscaux/<année>/<fichier>.toml`, chaque valeur `{ valeur, source, verifie, note }`. Chargement via
`ac_fiscal::ParametresAnnee::charger(annee)` (A8) et `ac_paie` pour `social.toml` (A7). L'année applicable est
celle de **la date de l'opération** (pas la date du jour).

## C11. Fiche entreprise — `ac_noyau::entreprise` (A1, utilisé par A2, A3, A10)

- `FicheEntreprise` : identité légale, IFU, RCCM, fiscalité, activité, adresse, contacts, responsable du compte (liste exacte : ADR 0009).
- `FicheEntreprise::valider(aujourdhui: Date) -> RapportValidation { erreurs, avertissements }` : **la même validation** dans l'application (assistant de démarrage, Paramètres)
  et sur la plateforme (inscription). Chaque `ProblemeChamp { champ, message }` nomme le champ (nom Rust) pour que l'interface le surligne.
- Identifiants validés : `Ifu` (13 chiffres), `Rccm` (normalisé, format `RB/<GREFFE>/<AA> <A|B> <numéro>` en avertissement), `Telephone` (normalisé `+229 01 XX XX XX XX`).
- Stockage local : `donnees_entreprise` (A2). Stockage plateforme : table `entreprises` (A10). Les deux reprennent **tous** les champs.

## C12. Assistance et annonces (A10)

- Commandes logiciel : `assistance_lister`, `assistance_ouvrir(categorie, sujet, message, pieces, diagnostic)`, `assistance_repondre(id, message, pieces)`, `assistance_marquer_lue(id)`, `annonces_lister`.
- API plateforme : `/v1/assistance/conversations` (GET/POST), `/v1/assistance/conversations/{id}/messages` (GET/POST, paramètre `depuis` pour la récupération incrémentale), `/v1/annonces` (GET).
- API console : `/admin/v1/…` (authentification administrateur + TOTP), flux temps réel `/admin/v1/assistance/flux` (SSE).
