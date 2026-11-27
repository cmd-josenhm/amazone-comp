//! Catalogue public partagé avec les ventes (A4).

use ac_donnees::rusqlite::OptionalExtension;
use ac_donnees::{rusqlite, Connexion};
use ac_noyau::Montant;
use serde::{Deserialize, Serialize};

use crate::{ErreurAchats, Resultat};

/// Type commercial d'un article du catalogue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TypeArticle {
    /// Bien suivi en quantité et en valeur dans le stock permanent.
    BienStocke,
    /// Bien non stocké (achat ou vente sans suivi de stock).
    BienNonStocke,
    /// Prestation de service.
    Service,
}

impl TypeArticle {
    fn texte(self) -> &'static str {
        match self {
            Self::BienStocke => "BIEN_STOCKE",
            Self::BienNonStocke => "BIEN_NON_STOCKE",
            Self::Service => "SERVICE",
        }
    }

    fn depuis_texte(texte: &str) -> Option<Self> {
        match texte {
            "BIEN_STOCKE" => Some(Self::BienStocke),
            "BIEN_NON_STOCKE" => Some(Self::BienNonStocke),
            "SERVICE" => Some(Self::Service),
            _ => None,
        }
    }
}

/// Article catalogué.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Article {
    /// Identifiant stable en base.
    pub id: i64,
    /// Code interne unique, comparé sans distinction de casse.
    pub code: String,
    /// Désignation lisible.
    pub designation: String,
    /// Type de l'article.
    pub type_article: TypeArticle,
    /// Unité de mesure (pièce, kg, litre, heure…).
    pub unite: String,
    /// Prix de vente hors taxes en FCFA.
    pub prix_vente_ht: Montant,
    /// Groupe de taxation par défaut (code e-MECeF).
    pub groupe_taxation: String,
    /// Compte SYSCOHADA de vente.
    pub compte_vente: String,
    /// Compte SYSCOHADA d'achat.
    pub compte_achat: String,
    /// Seuil d'alerte de stock en millièmes.
    pub stock_min_milliemes: i64,
    /// Indique si l'article peut être utilisé pour de nouvelles opérations.
    pub actif: bool,
}

impl Article {
    /// Identifiant numérique stable.
    pub const fn identifiant(&self) -> i64 {
        self.id
    }

    /// Type d'article sous la forme persistée dans le schéma SQL.
    pub(crate) const fn type_texte(&self) -> &'static str {
        self.type_article.texte()
    }
}

/// Résultat paginé de recherche (pages numérotées à partir de 1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PageArticles {
    /// Numéro de page demandé.
    pub page: u32,
    /// Nombre total d'articles actifs correspondant à la recherche.
    pub total: u64,
    /// Articles actifs de cette page.
    pub articles: Vec<Article>,
}

const TAILLE_PAGE: i64 = 50;
const COLONNES_ARTICLE: &str = "id, code, designation, type_article, unite, prix_vente_ht,
    groupe_taxation, compte_vente, compte_achat, stock_min_milliemes, actif";

/// Recherche un article par code, sans distinction de casse.
pub fn article(conn: &Connexion, code: &str) -> Result<Option<Article>> {
    let requete = format!("SELECT {COLONNES_ARTICLE} FROM achats_article WHERE code = ?1");
    conn.query_row(&requete, [code.trim()], ligne_article)
        .optional()
        .map_err(ErreurAchats::from)
}

/// Recherche les articles actifs par code ou désignation, avec 50 résultats par page.
///
/// La première page est `1`. Une recherche vide retourne l'ensemble du catalogue actif, paginé.
pub fn rechercher(conn: &Connexion, texte: &str, page: u32) -> Result<PageArticles> {
    if page == 0 {
        return Err(ErreurAchats::PageInvalide);
    }
    let page_index = i64::from(page - 1);
    let offset = page_index
        .checked_mul(TAILLE_PAGE)
        .ok_or(ErreurAchats::Depassement)?;
    let motif = format!("%{}%", texte.trim());
    let total: i64 = conn.query_row(
        "SELECT COUNT(*) FROM achats_article
         WHERE actif = 1 AND (?1 = '' OR code LIKE ?2 OR designation LIKE ?2)",
        rusqlite::params![texte.trim(), motif],
        |row| row.get(0),
    )?;
    let requete = format!(
        "SELECT {COLONNES_ARTICLE} FROM achats_article
         WHERE actif = 1 AND (?1 = '' OR code LIKE ?2 OR designation LIKE ?2)
         ORDER BY code COLLATE NOCASE, id LIMIT ?3 OFFSET ?4"
    );
    let mut statement = conn.prepare(&requete)?;
    let articles = statement
        .query_map(
            rusqlite::params![texte.trim(), motif, TAILLE_PAGE, offset],
            ligne_article,
        )?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    let total = u64::try_from(total).map_err(|_| ErreurAchats::Depassement)?;
    Ok(PageArticles {
        page,
        total,
        articles,
    })
}

fn ligne_article(row: &rusqlite::Row<'_>) -> rusqlite::Result<Article> {
    let type_texte: String = row.get(3)?;
    let type_article = TypeArticle::depuis_texte(&type_texte)
        .ok_or(rusqlite::Error::InvalidQuery)?;
    let prix_vente = row.get::<_, i64>(5)?;
    let actif: i64 = row.get(10)?;
    Ok(Article {
        id: row.get(0)?,
        code: row.get(1)?,
        designation: row.get(2)?,
        type_article,
        unite: row.get(4)?,
        prix_vente_ht: Montant::new(prix_vente).map_err(|_| rusqlite::Error::InvalidQuery)?,
        groupe_taxation: row.get(6)?,
        compte_vente: row.get(7)?,
        compte_achat: row.get(8)?,
        stock_min_milliemes: row.get(9)?,
        actif: actif == 1,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ac_donnees::Base;
    use crate::migrations;

    fn base_avec_articles() -> Base {
        let mut base = Base::en_memoire().expect("base en mémoire");
        base.migrer(&[ac_donnees::migrations(), migrations()])
            .expect("migrations");
        base.connexion()
            .execute(
                "INSERT INTO achats_article
                 (code, designation, type_article, unite, prix_vente_ht, groupe_taxation,
                  compte_vente, compte_achat, stock_min_milliemes, actif)
                 VALUES ('ART-01', 'Cahier quadrillé', 'BIEN_STOCKE', 'PIECE', 750,
                         'B', '701', '601', 2000, 1),
                        ('SRV-01', 'Transport de marchandises', 'SERVICE', 'PRESTATION', 5000,
                         'B', '706', '604', 0, 1),
                        ('INACTIF', 'Ancien article', 'BIEN_NON_STOCKE', 'PIECE', 100,
                         'E', '701', '601', 0, 0)",
                [],
            )
            .expect("articles de test");
        base
    }

    #[test]
    fn lit_un_article_par_code_sans_casse() {
        let base = base_avec_articles();
        let resultat = article(base.connexion(), "art-01")
            .expect("recherche")
            .expect("article présent");
        assert_eq!(resultat.designation, "Cahier quadrillé");
        assert_eq!(resultat.prix_vente_ht.francs(), 750);
        assert_eq!(resultat.type_article, TypeArticle::BienStocke);
    }

    #[test]
    fn recherche_pagine_et_masque_les_articles_inactifs() {
        let base = base_avec_articles();
        let resultat = rechercher(base.connexion(), "transport", 1).expect("recherche");
        assert_eq!(resultat.total, 1);
        assert_eq!(resultat.articles[0].code, "SRV-01");
        assert!(rechercher(base.connexion(), "ancien", 1)
            .expect("recherche")
            .articles
            .is_empty());
        assert!(matches!(
            rechercher(base.connexion(), "", 0),
            Err(ErreurAchats::PageInvalide)
        ));
    }
}
