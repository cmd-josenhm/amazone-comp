# Module Fiscalité (`ac-fiscal`, écran « Fiscalité »)

Propriétaire : **agent A8**. Prompt : `PROMPT_A8.md`. Règles fiscales : `docs/fiscalite/`.

Le module tient les **paramètres fiscaux versionnés** de chaque année, fournit les **fonctions de calcul
pures** utilisées par les ventes (A4), les achats (A5), la paie (A7) et les états (A9), et prépare les
**déclarations PDF / Excel** et le **calendrier fiscal**. Il **ne dépose rien** : l'utilisateur dépose sur
e-services.

État : **Lot 1 (vérification des sources) livré** — paramètres adossés aux articles du CGI 2026, aucune
valeur `verifie = true`, 25 issues `fiscal-a-verifier` ouvertes.

## 1. Paramètres (contrat C10)

- Arborescence : `parametres-fiscaux/<année>/<fichier>.toml`, embarqués à la compilation
  (`ac_fiscal::fichiers_embarques`).
- Chaque valeur réglementaire est un `ac_noyau::parametres::Valeur<T>` : `{ valeur, source, verifie, note }`.
- **`source` cite toujours un texte identifiable** : article du CGI, article de la loi de finances, ou —
  pour les seuils fixés par voie réglementaire — l'arrêté correspondant. Deux tests le garantissent dans
  `ac-fiscal` : `parametres::tests::toutes_les_sources_citent_un_texte_legal` (aucune `source` vide,
  générique ou sans référence d'article) et
  `parametres::tests::une_valeur_verifiee_exige_une_source_definitive` (une valeur `verifie = true` ne
  peut pas conserver une source provisoire : « à vérifier », « à obtenir », « à confirmer »…).
- **L'année applicable est celle de la date de l'opération**, jamais celle du jour.
- **Une année publiée ne se réécrit pas** : une clé mal nommée ou une valeur corrigée s'introduit dans la
  version suivante du fichier (ou de l'année suivante). Le Lot 1 a néanmoins corrigé, dans les fichiers
  2026 non encore publiés, les valeurs **fausses au regard du texte** (plancher de l'IBA) et les `source`
  sans article : voir `docs/fiscalite/sources.md` §2.

| Fichier | Contenu | Propriétaire |
|---|---|---|
| `tva.toml` | taux, seuil (par arrêté), déduction carburant et plafond BTP, retenues de TVA | A8 |
| `aib.toml` | taux par catégorie d'opération, base, imputation | A8 |
| `impot-resultat.toml` | IS, IBA, minimum de perception, planchers, acomptes, redevance ORTB | A8 |
| `tps.toml` | taux, minimum, seuil (par arrêté), redevance, acomptes | A8 |
| `retenues.toml` | loyers, non-résidents, IRCM | A8 |
| `penalites.toml` | majorations et intérêts (simulation d'alertes seulement) | A8 |
| `calendrier.toml` | échéances déclaratives et sociales | A8 (+ A7 pour le social) |
| `emecef.toml` | groupes de taxation, codes AIB, types de facture et de paiement | A8 (consommé par A4) |
| `social.toml` | ITS, VPS, CNSS, SMIG | **A7** |

## 2. Codes fiscaux consommés (contrat C4, propriété de A1)

`ac-fiscal` construit ses déclarations **uniquement** à partir des lignes d'écriture étiquetées par
`ac_compta::CodeFiscal` — il ne lit **jamais** les tables des autres modules. Les étiquettes qui
intéressent A8, telles que produites par les autres modules :

- **Ventes (A4)** : `TvaCollectee` (base HT + taux), `CaExonere`, `CaExport`, `AibCollecte`,
  `TvaRetenueParClient` ;
- **Achats (A5)** : `TvaDeductibleBiens` / `Services` / `Immobilisations`, `AibSupporte`, `RetenueLoyer`,
  `RetenueNonResident`, `TvaRetenueSurFournisseur` ;
- **Paie (A7)** : `Its`, `Vps`, `CnssSalarie`, `CnssEmployeur`, `RedevanceAudiovisuelle` ;
- **Fiscal (A8, écritures de liquidation)** : `ImpotResultat`, `AcompteImpotResultat`, `Tps`.

⚠️ Trois pièges confirmés au Lot 1, à respecter par A4 et A5 :

1. un **loyer** ne porte jamais `AibSupporte` (CGI 2026, art. 131, paragraphe 7 : les loyers relèvent de la
   retenue de l'art. 106) ;
2. une **vente exonérée** ne doit pas générer de `TvaCollectee` à 0 % : elle porte `CaExonere`
   (CGI 2026, art. 229) ;
3. une **exportation** ne porte pas `CaExonere` (droit à déduction différent des exonérations : art. 241 et
   `CaExport`, issue #51).

## 3. Fonctions de calcul (crate `ac-fiscal`)

Livré au Lot 3 ; aujourd'hui :

| Fonction | État |
|---|---|
| `tva::depuis_ht(ht, taux) -> Ventilation` | livrée, arrondi au franc le plus proche, testée |
| `regime::Regime` (ré-export de `ac_noyau::entreprise`) | livré |
| `parametres::ParametresAnnee::charger(annee)` | livré : TVA et TPS typées ; **Lot 2** : tous les fichiers |
| `tva::ventiler_ligne`, `aib::calculer`, `tva::retenue_a_la_source`, `retenues::loyer`, `retenues::non_resident`, `impot_resultat::calculer`, `tps::calculer`, `regime::suggerer` | Lot 3 |

Règle d'arrondi : **explicite, paramétrée, documentée** (aucune règle officielle dans le CGI — issue #31).
Montants en `ac_noyau::Montant` (entiers FCFA), taux en `ac_noyau::Taux` (décimal exact) : jamais de
flottant (R1), jamais de taux en dur (R2).

## 4. Commandes IPC (`src-tauri/src/commandes/fiscal.rs`)

Aucune commande livrée (Lot 8). Les noms prévus : `fiscal_prochaines_echeances` (pour A9),
`fiscal_parametres_annee`, `fiscal_preparer_declaration`, `fiscal_exporter_declaration`,
`fiscal_marquer_deposee`.

## 5. Écran « Fiscalité » (`ui/src/modules/fiscal/`)

Placeholder (Lot 8). Cible : calendrier, préparation guidée d'une déclaration (période → contrôles → aperçu
→ export PDF/Excel → « marquer comme déposée » avec date et référence), historique, page « Règles et
sources » listant chaque paramètre, sa source et son état de vérification.

## 6. Tables et migrations

Aucune table pour l'instant. Plage réservée : **6000–6999** (architecture §4). Prévues : échéances
(`fiscal_echeance`), déclarations préparées/déposées (`fiscal_declaration`), simulations de pénalités.

## 7. Limites connues

- **Aucune valeur fiscale n'est validée** : `verifie = false` partout. L'interface doit afficher l'état de
  vérification de chaque paramètre et l'avertissement « Vérifiez avant dépôt sur e-services ».
- **25 points de loi restent ouverts** (issues #22 à #40, #51 à #55 et #59) : voir `docs/fiscalite/sources.md`
  §3 et §4. Les plus structurants : le régime « réel simplifié » (#28), l'arrondi et e-MECeF (#31 et #54),
  l'exigibilité de la TVA (#30, tranchée sur le texte mais à valider), le plancher de l'IBA (#25).
- **Seuil de la TPS et seuil d'assujettissement à la TVA** : fixés par arrêté, non obtenus (#39).
- Le module ne calcule pas encore le prorata de déduction, la patente, les droits d'enregistrement.
