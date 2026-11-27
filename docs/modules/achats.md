# Achats, catalogue et stocks

> Lot 1 — contrat public préparé avant l'implémentation, pour coordination avec A4.

## Catalogue public (`ac_achats::catalogue`)

Les fonctions de lecture ne modifient pas la base. Les résultats sont typés et les erreurs sont remontées, jamais masquées.

```rust
pub fn article(
    conn: &ac_donnees::Connexion,
    code: &str,
) -> Result<Option<Article>, ErreurAchats>;

pub fn rechercher(
    conn: &ac_donnees::Connexion,
    texte: &str,
    page: u32,
) -> Result<PageArticles, ErreurAchats>;
```

`Article` est le modèle public d'un article (identifiant stable, code, désignation, type, unité,
prix de vente HT en `ac_noyau::Montant`, groupe de taxation, comptes de vente et d'achat, seuil de stock
minimum en millièmes et statut actif). `PageArticles` contient la page demandée, le nombre total de
résultats et les articles actifs de cette page. La pagination commence à 1, avec 50 articles par page;
une page invalide renvoie `PageInvalide`. La recherche porte sur le code et la désignation, et son ordre
est déterministe.

La migration `achats_catalogue_stock` crée `achats_article` (code, désignation, type, unité, prix de vente
HT, groupe de taxation, comptes de vente/achat, seuil de stock minimum, statut) et
`achats_stock_mouvement` (écart quantitatif et valeur FCFA, CMUP, référence, utilisateur et date). Le
journal des mouvements est append-only. Les quantités et valeurs sont stockées en entiers SQLite.

## Stock public (`ac_achats::stock`)

```rust
pub fn entrer(
    tx: &ac_donnees::Transaction<'_>,
    session: &ac_donnees::Session,
    article: &Article,
    quantite_milliemes: i64,
    cout_unitaire: ac_noyau::Montant,
    reference: &str,
) -> Result<(), ErreurAchats>;

pub fn sortir(
    tx: &ac_donnees::Transaction<'_>,
    session: &ac_donnees::Session,
    article: &Article,
    quantite_milliemes: i64,
    reference: &str,
) -> Result<CoutSortie, ErreurAchats>;

pub fn disponible(
    conn: &ac_donnees::Connexion,
    article: &Article,
) -> Result<i64, ErreurAchats>;

pub fn ajuster_inventaire(
    tx: &ac_donnees::Transaction<'_>,
    session: &ac_donnees::Session,
    article: &Article,
    quantite_comptee_milliemes: i64,
    cout_unitaire: Option<ac_noyau::Montant>,
    reference: &str,
) -> Result<AjustementInventaire, ErreurAchats>;
```

`CoutSortie` expose la quantité sortie en millièmes et son coût total en FCFA (`Montant`).
`AjustementInventaire` expose les quantités théorique et comptée, l'écart en millièmes et la valeur
correspondante en FCFA. Les quantités sont toujours des `i64` en millièmes; aucune quantité ni valeur
monétaire n'est calculée en flottant. Le CMUP est recalculé à chaque entrée par moyenne pondérée des
quantités et coûts, arrondie au franc le plus proche (demi-franc arrondi à l'unité supérieure). Les
sorties et écarts négatifs sont valorisés au CMUP courant, leur valeur totale étant arrondie au franc
le plus proche; le dernier reliquat de stock est valorisé à la valeur comptable résiduelle pour éviter
un solde négatif dû aux arrondis. Un CMUP connu est conservé même lorsque la quantité en stock retombe à zéro.
Un écart positif reprend ce dernier CMUP et ignore `cout_unitaire`; si aucun CMUP n'a jamais existé,
`cout_unitaire` est obligatoire, sinon `CoutInventaireRequis` indique : « Indiquez le coût unitaire de
cet article pour valoriser l'inventaire ».

Toutes les écritures sont faites dans la transaction fournie par l'appelant et auditées dans cette même
transaction. Les fonctions vérifient la permission appropriée. Les stocks négatifs sont refusés par défaut;
aucun mouvement en-deçà de zéro n'est accepté par cette API. Une politique d'autorisation configurable
sera ajoutée seulement après définition de son paramétrage.

### Exemples de référence

- Entrée : 10 unités (10 000 millièmes) à 1 000 FCFA donne 10 unités et une valeur de stock de 10 000 FCFA.
- CMUP : 10 unités à 1 000 FCFA puis 10 à 1 200 FCFA donnent 20 unités valorisées à 22 000 FCFA,
  soit 1 100 FCFA l'unité; la sortie de 5 unités (5 000 millièmes) coûte 5 500 FCFA.
- Zéro : une sortie de 0 millième est invalide et ne crée aucun mouvement; une disponibilité initiale
  sans mouvement vaut 0 millième.
- Arrondi : une sortie de 1 millième au CMUP de 500 FCFA l'unité vaut 1 FCFA après arrondi au plus proche;
  la même sortie au CMUP de 400 FCFA vaut 0 FCFA. Les sommes de stock sont conservées en FCFA entiers.
- Une sortie de plus que le disponible est refusée sans modifier le stock.
