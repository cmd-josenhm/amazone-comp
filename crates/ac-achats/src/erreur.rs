//! Erreurs du catalogue et du stock.

use thiserror::Error;

/// Erreurs métier et techniques des opérations d'achats.
#[derive(Debug, Error)]
pub enum ErreurAchats {
    /// Erreur SQLite.
    #[error("erreur de base de données : {0}")]
    Sql(#[from] ac_donnees::rusqlite::Error),
    /// Erreur provenant de la couche des données, notamment des permissions.
    #[error(transparent)]
    Donnees(#[from] ac_donnees::ErreurDonnees),
    /// Montant hors limites ou calcul monétaire impossible.
    #[error(transparent)]
    Noyau(#[from] ac_noyau::ErreurNoyau),
    /// Code article inconnu.
    #[error("article introuvable")]
    ArticleIntrouvable,
    /// Un article non stocké ou un service ne peut pas avoir de mouvement de stock.
    #[error("cet article n'est pas un bien stocké")]
    ArticleNonStocke,
    /// La quantité doit être strictement positive pour une entrée ou une sortie.
    #[error("la quantité doit être supérieure à zéro")]
    QuantiteNulleOuNegative,
    /// La quantité comptée ne peut pas être négative.
    #[error("la quantité inventoriée ne peut pas être négative")]
    InventaireNegatif,
    /// La sortie demandée dépasse la quantité disponible.
    #[error("stock insuffisant : la sortie demandée dépasse le disponible")]
    StockInsuffisant,
    /// Un inventaire positif sans CMUP historique exige un coût de référence.
    #[error("Indiquez le coût unitaire de cet article pour valoriser l'inventaire")]
    CoutInventaireRequis,
    /// Un coût unitaire négatif est interdit.
    #[error("le coût unitaire ne peut pas être négatif")]
    CoutNegatif,
    /// La référence du mouvement ne peut pas être vide.
    #[error("la référence du mouvement est obligatoire")]
    ReferenceVide,
    /// Une page de recherche est numérotée à partir de 1.
    #[error("le numéro de page doit être supérieur ou égal à 1")]
    PageInvalide,
    /// L'accumulation de quantités ou de valeurs dépasse les limites supportées.
    #[error("la quantité ou la valorisation du stock dépasse les limites supportées")]
    Depassement,
}
