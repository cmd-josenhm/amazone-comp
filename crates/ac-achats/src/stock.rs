//! Stock permanent valorisé au coût moyen unitaire pondéré (CMUP).

use ac_donnees::audit::{journaliser, Evenement};
use ac_donnees::rusqlite::OptionalExtension;
use ac_donnees::{rusqlite, Connexion, Permission, Session, Transaction};
use ac_noyau::Montant;
use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::catalogue::{Article, TypeArticle};
use crate::{ErreurAchats, Resultat};

/// Quantité et coût total d'une sortie valorisée au CMUP.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoutSortie {
    /// Quantité sortie en millièmes.
    pub quantite_milliemes: i64,
    /// Coût total de sortie en FCFA.
    pub cout_total: Montant,
}

/// Résultat d'un ajustement d'inventaire physique.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct AjustementInventaire {
    /// Quantité disponible avant l'ajustement, en millièmes.
    pub quantite_theorique_milliemes: i64,
    /// Quantité physique comptée, en millièmes.
    pub quantite_comptee_milliemes: i64,
    /// Écart signé (compté moins théorique), en millièmes.
    pub ecart_milliemes: i64,
    /// Valeur signée de l'écart, en FCFA.
    pub valeur_ecart: Montant,
}

#[derive(Debug, Clone, Copy)]
struct SoldeStock {
    quantite_milliemes: i64,
    valeur_fcfa: i64,
    dernier_cmup_fcfa: Option<i64>,
}

struct NouveauMouvement<'a> {
    article_id: i64,
    nature: &'a str,
    quantite_delta_milliemes: i64,
    valeur_delta_fcfa: i64,
    cmup_fcfa: i64,
    reference: &'a str,
}

/// Enregistre une entrée de stock au coût unitaire communiqué.
///
/// La transaction est celle de l'appelant : l'entrée et son événement d'audit sont atomiques avec
/// le document métier qui l'a déclenchée.
pub fn entrer(
    tx: &Transaction<'_>,
    session: &Session,
    article: &Article,
    quantite_milliemes: i64,
    cout_unitaire: Montant,
    reference: &str,
) -> Result<()> {
    session.exiger(Permission::Saisir)?;
    verifier_mouvement(article, quantite_milliemes, reference)?;
    if cout_unitaire.est_negatif() {
        return Err(ErreurAchats::CoutNegatif);
    }
    verifier_article_actif(tx, article)?;

    let solde = lire_solde(tx, article.id)?;
    let nouvelle_quantite = solde
        .quantite_milliemes
        .checked_add(quantite_milliemes)
        .ok_or(ErreurAchats::Depassement)?;
    let valeur_entree = valoriser_quantite(quantite_milliemes, cout_unitaire.francs())?;
    let cmup = if solde.quantite_milliemes == 0 {
        cout_unitaire.francs()
    } else {
        let cmup_precedent = solde
            .dernier_cmup_fcfa
            .ok_or(ErreurAchats::Depassement)?;
        cmup_pondere(
            cmup_precedent,
            solde.quantite_milliemes,
            cout_unitaire.francs(),
            quantite_milliemes,
            nouvelle_quantite,
        )?
    };
    let valeur_totale = solde
        .valeur_fcfa
        .checked_add(valeur_entree.francs())
        .ok_or(ErreurAchats::Depassement)?;
    verifier_montant(valeur_totale)?;
    inserer_mouvement(
        tx,
        session,
        NouveauMouvement {
            article_id: article.id,
            nature: "ENTREE",
            quantite_delta_milliemes: quantite_milliemes,
            valeur_delta_fcfa: valeur_entree.francs(),
            cmup_fcfa: cmup,
            reference,
        },
    )?;
    journaliser_mouvement(
        tx,
        session,
        article.id,
        "STOCK_ENTREE",
        quantite_milliemes,
        valeur_entree.francs(),
        reference,
    )
}

/// Enregistre une sortie de stock et retourne son coût au CMUP courant.
pub fn sortir(
    tx: &Transaction<'_>,
    session: &Session,
    article: &Article,
    quantite_milliemes: i64,
    reference: &str,
) -> Result<CoutSortie> {
    session.exiger(Permission::Saisir)?;
    verifier_mouvement(article, quantite_milliemes, reference)?;
    verifier_article_actif(tx, article)?;
    let solde = lire_solde(tx, article.id)?;
    if quantite_milliemes > solde.quantite_milliemes {
        return Err(ErreurAchats::StockInsuffisant);
    }
    let cmup = cmup_courant(solde)?;
    let cout = cout_au_cmup(
        cmup,
        quantite_milliemes,
        solde.quantite_milliemes,
        solde.valeur_fcfa,
    )?;
    let valeur_restante = solde
        .valeur_fcfa
        .checked_sub(cout.francs())
        .ok_or(ErreurAchats::Depassement)?;
    verifier_montant(valeur_restante)?;
    inserer_mouvement(
        tx,
        session,
        NouveauMouvement {
            article_id: article.id,
            nature: "SORTIE",
            quantite_delta_milliemes: -quantite_milliemes,
            valeur_delta_fcfa: -cout.francs(),
            cmup_fcfa: cmup,
            reference,
        },
    )?;
    journaliser_mouvement(
        tx,
        session,
        article.id,
        "STOCK_SORTIE",
        -quantite_milliemes,
        -cout.francs(),
        reference,
    )?;
    Ok(CoutSortie {
        quantite_milliemes,
        cout_total: cout,
    })
}

/// Retourne la quantité disponible en millièmes (zéro si aucun mouvement n'existe).
pub fn disponible(conn: &Connexion, article: &Article) -> Result<i64> {
    verifier_article_stockable(article)?;
    verifier_article_existant(conn, article)?;
    let solde = lire_solde(conn, article.id)?;
    Ok(solde.quantite_milliemes)
}

/// Aligne le stock théorique sur le comptage physique dans la transaction de l'appelant.
///
/// Pour un écart positif, le dernier CMUP connu est utilisé même si le stock est nul. Le coût fourni
/// n'est utilisé que si l'article n'a jamais eu de CMUP; dans ce cas il est obligatoire.
pub fn ajuster_inventaire(
    tx: &Transaction<'_>,
    session: &Session,
    article: &Article,
    quantite_comptee_milliemes: i64,
    cout_unitaire: Option<Montant>,
    reference: &str,
) -> Result<AjustementInventaire> {
    session.exiger(Permission::Saisir)?;
    if quantite_comptee_milliemes < 0 {
        return Err(ErreurAchats::InventaireNegatif);
    }
    if reference.trim().is_empty() {
        return Err(ErreurAchats::ReferenceVide);
    }
    verifier_article_actif(tx, article)?;
    let solde = lire_solde(tx, article.id)?;
    let ecart = quantite_comptee_milliemes
        .checked_sub(solde.quantite_milliemes)
        .ok_or(ErreurAchats::Depassement)?;
    if ecart == 0 {
        return Ok(AjustementInventaire {
            quantite_theorique_milliemes: solde.quantite_milliemes,
            quantite_comptee_milliemes,
            ecart_milliemes: 0,
            valeur_ecart: Montant::ZERO,
        });
    }

    let (nature, valeur_delta, cmup) = if ecart > 0 {
        let cmup_actuel = if solde.quantite_milliemes > 0 {
            Some(cmup_courant(solde)?)
        } else {
            solde.dernier_cmup_fcfa
        };
        let cout = match cmup_actuel {
            Some(valeur) => valeur,
            None => {
                let indique = cout_unitaire.ok_or(ErreurAchats::CoutInventaireRequis)?;
                if indique.est_negatif() {
                    return Err(ErreurAchats::CoutNegatif);
                }
                indique.francs()
            }
        };
        let valeur = valoriser_quantite(ecart, cout)?;
        let nouvelle_valeur = solde
            .valeur_fcfa
            .checked_add(valeur.francs())
            .ok_or(ErreurAchats::Depassement)?;
        verifier_montant(nouvelle_valeur)?;
        ("AJUSTEMENT", valeur.francs(), cout)
    } else {
        let quantite_ecart = ecart.checked_abs().ok_or(ErreurAchats::Depassement)?;
        let cmup = cmup_courant(solde)?;
        let valeur = cout_au_cmup(
            cmup,
            quantite_ecart,
            solde.quantite_milliemes,
            solde.valeur_fcfa,
        )?;
        let valeur_restante = solde
            .valeur_fcfa
            .checked_sub(valeur.francs())
            .ok_or(ErreurAchats::Depassement)?;
        verifier_montant(valeur_restante)?;
        ("AJUSTEMENT", -valeur.francs(), cmup)
    };

    inserer_mouvement(
        tx,
        session,
        NouveauMouvement {
            article_id: article.id,
            nature,
            quantite_delta_milliemes: ecart,
            valeur_delta_fcfa: valeur_delta,
            cmup_fcfa: cmup,
            reference,
        },
    )?;
    journaliser_mouvement(
        tx,
        session,
        article.id,
        "STOCK_AJUSTEMENT",
        ecart,
        valeur_delta,
        reference,
    )?;
    Ok(AjustementInventaire {
        quantite_theorique_milliemes: solde.quantite_milliemes,
        quantite_comptee_milliemes,
        ecart_milliemes: ecart,
        valeur_ecart: Montant::new(valeur_delta)?,
    })
}

fn verifier_mouvement(article: &Article, quantite: i64, reference: &str) -> Result<()> {
    verifier_article_stockable(article)?;
    if quantite <= 0 {
        return Err(ErreurAchats::QuantiteNulleOuNegative);
    }
    if reference.trim().is_empty() {
        return Err(ErreurAchats::ReferenceVide);
    }
    Ok(())
}

fn verifier_article_stockable(article: &Article) -> Result<()> {
    if article.type_article != TypeArticle::BienStocke {
        return Err(ErreurAchats::ArticleNonStocke);
    }
    Ok(())
}

fn verifier_article_existant(conn: &Connexion, article: &Article) -> Result<()> {
    let type_texte: Option<String> = conn
        .query_row(
            "SELECT type_article FROM achats_article WHERE id = ?1",
            [article.id],
            |row| row.get(0),
        )
        .optional()?;
    let type_texte = type_texte.ok_or(ErreurAchats::ArticleIntrouvable)?;
    if type_texte != article.type_texte() {
        return Err(ErreurAchats::ArticleNonStocke);
    }
    Ok(())
}

fn verifier_article_actif(tx: &Transaction<'_>, article: &Article) -> Result<()> {
    verifier_article_stockable(article)?;
    let enregistrement: Option<(String, i64)> = tx
        .query_row(
            "SELECT type_article, actif FROM achats_article WHERE id = ?1",
            [article.id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()?;
    let (type_article, actif) = enregistrement.ok_or(ErreurAchats::ArticleIntrouvable)?;
    if type_article != article.type_texte() {
        return Err(ErreurAchats::ArticleNonStocke);
    }
    if actif != 1 {
        return Err(ErreurAchats::ArticleIntrouvable);
    }
    Ok(())
}

fn lire_solde(conn: &Connexion, article_id: i64) -> Result<SoldeStock> {
    let (quantite, valeur): (i64, i64) = conn.query_row(
        "SELECT COALESCE(SUM(quantite_delta_milliemes), 0),
                COALESCE(SUM(valeur_delta_fcfa), 0)
         FROM achats_stock_mouvement WHERE article_id = ?1",
        [article_id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;
    if quantite < 0 || valeur < 0 {
        return Err(ErreurAchats::Depassement);
    }
    let dernier_cmup_fcfa = conn
        .query_row(
            "SELECT cmup_fcfa FROM achats_stock_mouvement
             WHERE article_id = ?1 ORDER BY id DESC LIMIT 1",
            [article_id],
            |row| row.get(0),
        )
        .optional()?;
    Ok(SoldeStock {
        quantite_milliemes: quantite,
        valeur_fcfa: valeur,
        dernier_cmup_fcfa,
    })
}

fn cmup_courant(solde: SoldeStock) -> Result<i64> {
    solde
        .dernier_cmup_fcfa
        .ok_or(ErreurAchats::CoutInventaireRequis)
}

fn cmup_pondere(
    cmup_precedent_fcfa: i64,
    quantite_precedente_milliemes: i64,
    cout_entree_fcfa: i64,
    quantite_entree_milliemes: i64,
    quantite_totale_milliemes: i64,
) -> Result<i64> {
    if cmup_precedent_fcfa < 0
        || quantite_precedente_milliemes <= 0
        || cout_entree_fcfa < 0
        || quantite_entree_milliemes <= 0
        || quantite_totale_milliemes <= 0
    {
        return Err(ErreurAchats::Depassement);
    }
    let valeur_precedente = i128::from(cmup_precedent_fcfa)
        .checked_mul(i128::from(quantite_precedente_milliemes))
        .ok_or(ErreurAchats::Depassement)?;
    let valeur_entree = i128::from(cout_entree_fcfa)
        .checked_mul(i128::from(quantite_entree_milliemes))
        .ok_or(ErreurAchats::Depassement)?;
    let numerateur = valeur_precedente
        .checked_add(valeur_entree)
        .ok_or(ErreurAchats::Depassement)?;
    diviser_arrondi_positif(numerateur, i128::from(quantite_totale_milliemes))
}

fn valoriser_quantite(quantite_milliemes: i64, cout_unitaire_fcfa: i64) -> Result<Montant> {
    if quantite_milliemes < 0 || cout_unitaire_fcfa < 0 {
        return Err(ErreurAchats::Depassement);
    }
    let numerateur = i128::from(quantite_milliemes)
        .checked_mul(i128::from(cout_unitaire_fcfa))
        .ok_or(ErreurAchats::Depassement)?;
    let valeur = diviser_arrondi_positif(numerateur, 1000)?;
    Montant::new(valeur).map_err(ErreurAchats::from)
}

fn cout_au_cmup(
    cmup_fcfa: i64,
    quantite_milliemes: i64,
    total_milliemes: i64,
    valeur_stock_fcfa: i64,
) -> Result<Montant> {
    if quantite_milliemes <= 0
        || total_milliemes <= 0
        || quantite_milliemes > total_milliemes
        || valeur_stock_fcfa < 0
    {
        return Err(ErreurAchats::Depassement);
    }
    if quantite_milliemes == total_milliemes {
        return Montant::new(valeur_stock_fcfa).map_err(ErreurAchats::from);
    }
    let valeur_cmup = valoriser_quantite(quantite_milliemes, cmup_fcfa)?;
    Montant::new(valeur_cmup.francs().min(valeur_stock_fcfa)).map_err(ErreurAchats::from)
}

fn diviser_arrondi_positif(numerateur: i128, denominateur: i128) -> Result<i64> {
    if numerateur < 0 || denominateur <= 0 {
        return Err(ErreurAchats::Depassement);
    }
    let demi = denominateur / 2;
    let arrondi = numerateur
        .checked_add(demi)
        .ok_or(ErreurAchats::Depassement)?
        / denominateur;
    i64::try_from(arrondi).map_err(|_| ErreurAchats::Depassement)
}

fn verifier_montant(francs: i64) -> Result<()> {
    Montant::new(francs).map(|_| ()).map_err(ErreurAchats::from)
}

fn inserer_mouvement(
    tx: &Transaction<'_>,
    session: &Session,
    mouvement: NouveauMouvement<'_>,
) -> Result<()> {
    tx.execute(
        "INSERT INTO achats_stock_mouvement
         (article_id, nature, quantite_delta_milliemes, valeur_delta_fcfa, cmup_fcfa,
          reference, utilisateur_id, cree_le)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        rusqlite::params![
            mouvement.article_id,
            mouvement.nature,
            mouvement.quantite_delta_milliemes,
            mouvement.valeur_delta_fcfa,
            mouvement.cmup_fcfa,
            mouvement.reference.trim(),
            session.utilisateur_id,
            Utc::now().to_rfc3339()
        ],
    )?;
    Ok(())
}

fn journaliser_mouvement(
    tx: &Transaction<'_>,
    session: &Session,
    article_id: i64,
    action: &str,
    quantite_delta_milliemes: i64,
    valeur_delta_fcfa: i64,
    reference: &str,
) -> Result<()> {
    let details = serde_json::json!({
        "quantite_delta_milliemes": quantite_delta_milliemes,
        "valeur_delta_fcfa": valeur_delta_fcfa,
        "reference": reference.trim(),
    })
    .to_string();
    journaliser(
        tx,
        &Evenement {
            utilisateur_id: Some(session.utilisateur_id),
            action,
            objet_type: "achats_article",
            objet_id: &article_id.to_string(),
            details: &details,
        },
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalogue::article;
    use crate::migrations;
    use ac_donnees::{Base, Role};

    fn base_et_article() -> (Base, Article, Session) {
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
                        ('SRV-01', 'Transport', 'SERVICE', 'PRESTATION', 5000,
                         'B', '706', '604', 0, 1)",
                [],
            )
            .expect("articles de test");
        let article = article(base.connexion(), "ART-01")
            .expect("lecture article")
            .expect("article présent");
        let session = Session {
            utilisateur_id: 1,
            role: Role::Administrateur,
        };
        (base, article, session)
    }

    fn entrer_test(
        base: &mut Base,
        session: &Session,
        article: &Article,
        quantite: i64,
        cout: i64,
        reference: &str,
    ) {
        let tx = base.transaction().expect("transaction");
        entrer(
            &tx,
            session,
            article,
            quantite,
            Montant::new(cout).expect("coût valide"),
            reference,
        )
        .expect("entrée");
        tx.commit().expect("validation transaction");
    }

    #[test]
    fn cmup_et_cout_de_sortie_exemple_manuel() {
        let (mut base, article, session) = base_et_article();
        entrer_test(&mut base, &session, &article, 10_000, 1_000, "REC-001");
        entrer_test(&mut base, &session, &article, 10_000, 1_200, "REC-002");
        assert_eq!(disponible(base.connexion(), &article).expect("disponible"), 20_000);

        let tx = base.transaction().expect("transaction");
        let cout = sortir(&tx, &session, &article, 5_000, "BL-001").expect("sortie");
        assert_eq!(cout.cout_total.francs(), 5_500);
        tx.commit().expect("validation transaction");
        assert_eq!(disponible(base.connexion(), &article).expect("disponible"), 15_000);
    }

    #[test]
    fn stock_initial_et_sortie_nulle_sont_a_zero_sans_mouvement() {
        let (mut base, article, session) = base_et_article();
        assert_eq!(disponible(base.connexion(), &article).expect("disponible"), 0);
        let tx = base.transaction().expect("transaction");
        assert!(matches!(
            sortir(&tx, &session, &article, 0, "BL-VIDE"),
            Err(ErreurAchats::QuantiteNulleOuNegative)
        ));
        drop(tx);
        assert_eq!(disponible(base.connexion(), &article).expect("disponible"), 0);
    }

    #[test]
    fn arrondi_au_franc_le_plus_proche_pour_milliemes() {
        let (mut base, article, session) = base_et_article();
        entrer_test(&mut base, &session, &article, 1, 500, "REC-500");
        let solde = lire_solde(base.connexion(), article.id).expect("solde");
        assert_eq!(solde.valeur_fcfa, 1);
        assert_eq!(solde.dernier_cmup_fcfa, Some(500));
        let tx = base.transaction().expect("transaction");
        let sortie = sortir(&tx, &session, &article, 1, "BL-500").expect("sortie arrondie");
        assert_eq!(sortie.cout_total.francs(), 1);
        tx.commit().expect("validation transaction");

        let (mut base_zero, article_zero, session_zero) = base_et_article();
        entrer_test(&mut base_zero, &session_zero, &article_zero, 1, 400, "REC-400");
        assert_eq!(
            lire_solde(base_zero.connexion(), article_zero.id)
                .expect("solde")
                .valeur_fcfa,
            0
        );
        let tx = base_zero.transaction().expect("transaction");
        let sortie = sortir(&tx, &session_zero, &article_zero, 1, "BL-400")
            .expect("sortie arrondie");
        assert_eq!(sortie.cout_total.francs(), 0);
        tx.commit().expect("validation transaction");
    }

    #[test]
    fn sortie_superieure_au_disponible_refusee_sans_modifier_le_stock() {
        let (mut base, article, session) = base_et_article();
        entrer_test(&mut base, &session, &article, 1_000, 700, "REC-001");
        let tx = base.transaction().expect("transaction");
        assert!(matches!(
            sortir(&tx, &session, &article, 1_001, "BL-001"),
            Err(ErreurAchats::StockInsuffisant)
        ));
        drop(tx);
        assert_eq!(disponible(base.connexion(), &article).expect("disponible"), 1_000);
    }

    #[test]
    fn inventaire_positif_reutilise_le_cmup_historique_apres_stock_nul() {
        let (mut base, article, session) = base_et_article();
        entrer_test(&mut base, &session, &article, 1_000, 700, "REC-001");
        {
            let tx = base.transaction().expect("transaction");
            sortir(&tx, &session, &article, 1_000, "BL-001").expect("sortie");
            tx.commit().expect("validation transaction");
        }
        let tx = base.transaction().expect("transaction");
        let ajustement = ajuster_inventaire(
            &tx,
            &session,
            &article,
            2_000,
            Some(Montant::new(999).expect("coût valide")),
            "INV-001",
        )
        .expect("ajustement");
        assert_eq!(ajustement.ecart_milliemes, 2_000);
        assert_eq!(ajustement.valeur_ecart.francs(), 1_400);
        tx.commit().expect("validation transaction");
    }

    #[test]
    fn inventaire_initial_exige_un_cout_et_accepte_zero_compte() {
        let (mut base, article, session) = base_et_article();
        let tx = base.transaction().expect("transaction");
        assert!(matches!(
            ajuster_inventaire(&tx, &session, &article, 1_000, None, "INV-001"),
            Err(ErreurAchats::CoutInventaireRequis)
        ));
        drop(tx);

        let tx = base.transaction().expect("transaction");
        let resultat = ajuster_inventaire(&tx, &session, &article, 0, None, "INV-002")
            .expect("inventaire à zéro");
        assert_eq!(resultat.ecart_milliemes, 0);
        assert_eq!(resultat.valeur_ecart.francs(), 0);
        tx.commit().expect("validation transaction");
    }

    #[test]
    fn articles_non_stockes_refuses_et_permissions_verifiees() {
        let (mut base, _, session) = base_et_article();
        let service = article(base.connexion(), "SRV-01")
            .expect("lecture article")
            .expect("service présent");
        let tx = base.transaction().expect("transaction");
        assert!(matches!(
            entrer(&tx, &session, &service, 1_000, Montant::new(100).expect("coût"), "REC"),
            Err(ErreurAchats::ArticleNonStocke)
        ));
        drop(tx);

        let lecture_seule = Session {
            utilisateur_id: 2,
            role: Role::Lecture,
        };
        let tx = base.transaction().expect("transaction");
        assert!(matches!(
            entrer(&tx, &lecture_seule, &service, 1_000, Montant::new(100).expect("coût"), "REC"),
            Err(ErreurAchats::Donnees(_))
        ));
    }
}
