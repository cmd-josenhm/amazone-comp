//! Comptes de trésorerie : modèles, création, lecture, masquage et soldes.

use std::collections::BTreeSet;

use ac_compta::{
    BrouillonEcriture, LigneBrouillon, ModuleOrigine, NumeroCompte, passer_ecriture,
};
use ac_donnees::audit::{journaliser, Evenement};
use ac_donnees::rusqlite::OptionalExtension;
use ac_donnees::{Permission, Session, Transaction};
use ac_noyau::entreprise::Telephone;
use ac_noyau::Montant;
use serde::{Deserialize, Serialize};

use crate::journal::{journal_creer, journal_prochain_code};
use crate::{ErreurTresorerie, Resultat};

/// Type de compte de trésorerie (C5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TypeCompte {
    /// Compte bancaire (sous-compte de 521).
    Banque,
    /// Caisse d'espèces (sous-compte de 571) ; ne peut pas avoir un solde négatif.
    Caisse,
    /// Portefeuille Mobile Money (sous-compte de 55).
    MobileMoney,
}

impl TypeCompte {
    /// Valeur stockée en base et référée par l'interface (C8).
    pub fn libelle(self) -> &'static str {
        match self {
            TypeCompte::Banque => "BANQUE",
            TypeCompte::Caisse => "CAISSE",
            TypeCompte::MobileMoney => "MOBILE_MONEY",
        }
    }

    /// Libellé français du type (affiché dans l'interface).
    pub fn libelle_francais(self) -> &'static str {
        match self {
            TypeCompte::Banque => "Banque",
            TypeCompte::Caisse => "Caisse",
            TypeCompte::MobileMoney => "Mobile Money",
        }
    }

    /// Compte racine du type, auquel le compte de trésorerie est rattaché.
    pub fn compte_base(self) -> &'static str {
        match self {
            TypeCompte::Banque => "521",
            TypeCompte::Caisse => "571",
            TypeCompte::MobileMoney => "55",
        }
    }

    /// Préfixe des codes de journaux (`BQ` pour les banques, `MM` pour le Mobile Money).
    pub fn journal_prefixe(self) -> &'static str {
        match self {
            TypeCompte::Banque => "BQ",
            TypeCompte::Caisse => "CA",
            TypeCompte::MobileMoney => "MM",
        }
    }

    /// Vrai si un solde négatif est interdit pour ce type.
    pub fn ne_peut_pas_etre_negatif(self) -> bool {
        matches!(self, TypeCompte::Caisse)
    }

    /// Construit le type à partir de sa valeur de base (ex. `"MOBILE_MONEY"`).
    pub fn depuis_str(texte: &str) -> Resultat<Self> {
        match texte.trim() {
            "BANQUE" => Ok(TypeCompte::Banque),
            "CAISSE" => Ok(TypeCompte::Caisse),
            "MOBILE_MONEY" => Ok(TypeCompte::MobileMoney),
            autre => Err(ErreurTresorerie::TypeCompteInvalide(autre.to_owned())),
        }
    }
}

/// Paramètres de création d'un compte de trésorerie (`tresorerie_compte_creer`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreerCompte {
    /// Type de compte : `BANQUE`, `CAISSE` ou `MOBILE_MONEY`.
    pub type_compte: TypeCompte,
    /// Libellé du compte (ex. « Banque OBA »).
    pub libelle: String,
    /// Nom de la banque ou de l'opérateur (obligatoire pour BANQUE et MOBILE_MONEY).
    pub banque_operateur: Option<String>,
    /// Numéro de compte bancaire ou téléphone marchand ; inutile pour une caisse.
    pub numero: Option<String>,
    /// Solde d'ouverture en FCFA ; négatif = solde créditeur (interdit pour la caisse).
    pub solde_ouverture: Montant,
    /// Référence de la pièce justificative d'un solde d'ouverture non nul.
    pub piece_ouverture: Option<String>,
    /// Compte du plan comptable qui équilibre un solde d'ouverture non nul.
    pub compte_contrepartie_ouverture: Option<String>,
}

/// Paramètres d'ajout d'un opérateur Mobile Money.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreerOperateur {
    /// Nom affiché (ex. « Celtiis Cash »).
    pub nom: String,
    /// Libellé complémentaire facultatif.
    pub libelle: Option<String>,
}

/// Compte de trésorerie tel que retourné à l'interface (numéro masqué).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TresorerieCompte {
    /// Identifiant en base.
    pub id: i64,
    /// Type de compte.
    pub type_compte: TypeCompte,
    /// Libellé du compte.
    pub libelle: String,
    /// Nom de la banque ou de l'opérateur.
    pub banque_operateur: Option<String>,
    /// Numéro de compte bancaire ou numéro de téléphone (masqué à l'affichage).
    pub numero: Option<String>,
    /// Compte comptable du compte (sous-compte de 521 / 55 / 571).
    pub compte_comptable: String,
    /// Code du journal dédié (`BQn`, `MMn`, `CA`).
    pub journal: String,
    /// Solde d'ouverture en FCFA ; négatif = solde créditeur.
    pub solde_ouverture: Montant,
    /// Actif.
    pub actif: bool,
    /// Date d'ouverture (ISO 8601).
    pub date_ouverture: String,
}

/// Opérateur Mobile Money pré-rempli ou ajouté par l'utilisateur.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TresorerieOperateur {
    /// Identifiant en base.
    pub id: i64,
    /// Nom de l'opérateur (ex. « MTN MoMo »).
    pub nom: String,
    /// Libellé optionnel.
    pub libelle: Option<String>,
    /// Actif.
    pub actif: bool,
}

/// Configuration de trésorerie propre au dossier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TresorerieConfiguration {
    /// Vrai si les comptes Mobile Money peuvent être créés et utilisés.
    pub mobile_money_actif: bool,
}

/// Solde comptable d'un compte de trésorerie lu dans `compta_ligne`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TresorerieSolde {
    /// Compte mouvementé (voir C5).
    pub compte: NumeroCompte,
    /// Type de trésorerie du compte.
    pub type_compte: TypeCompte,
    /// Solde comptable en FCFA (positif = solde débiteur).
    pub solde: Montant,
}

/// Masque un numéro de compte ou de téléphone : seuls les quatre derniers caractères sont visibles.
pub fn masquer(texte: &str) -> String {
    let chiffres: String = texte
        .chars()
        .filter(|caractere| caractere.is_ascii_digit())
        .collect();
    if chiffres.is_empty() {
        return "••••".to_owned();
    }
    let masques = chiffres.len().saturating_sub(4);
    format!("{}{}", "•".repeat(masques), &chiffres[masques..])
}

fn contrepartie_mouvementable(tx: &Transaction<'_>, numero: &str) -> Resultat<bool> {
    let utilisable: bool = tx.query_row(
        "SELECT EXISTS(
            SELECT 1 FROM compta_compte
            WHERE numero = ?1 AND actif = 1 AND mouvementable = 1
         )",
        [numero],
        |ligne| ligne.get(0),
    )?;
    Ok(utilisable)
}

fn verifier_operateur_actif(tx: &Transaction<'_>, nom: &str) -> Resultat<String> {
    let operateur: Option<(String, i64)> = tx
        .query_row(
            "SELECT nom, actif FROM tresorerie_operateur WHERE nom = ?1 COLLATE NOCASE",
            [nom],
            |ligne| Ok((ligne.get(0)?, ligne.get(1)?)),
        )
        .optional()?;
    match operateur {
        None => Err(ErreurTresorerie::OperateurInconnu(nom.to_owned())),
        Some((nom, 0)) => Err(ErreurTresorerie::OperateurInactif(nom)),
        Some((nom, _)) => Ok(nom),
    }
}

/// Lit la configuration du module Trésorerie.
pub fn configuration(tx: &Transaction<'_>) -> Resultat<TresorerieConfiguration> {
    let actif: i64 = tx.query_row(
        "SELECT mobile_money_actif FROM tresorerie_configuration WHERE id = 1",
        [],
        |ligne| ligne.get(0),
    )?;
    Ok(TresorerieConfiguration {
        mobile_money_actif: actif != 0,
    })
}

/// Active ou désactive les comptes Mobile Money pour ce dossier.
pub fn configurer_mobile_money(
    tx: &Transaction<'_>,
    session: &Session,
    actif: bool,
) -> Resultat<TresorerieConfiguration> {
    session.exiger(Permission::Administrer)?;
    let configuration = configuration(tx)?;
    if configuration.mobile_money_actif == actif {
        return Ok(configuration);
    }
    let actif_sql = if actif { 1_i64 } else { 0_i64 };
    tx.execute(
        "UPDATE tresorerie_configuration SET mobile_money_actif = ?1 WHERE id = 1",
        [actif_sql],
    )?;
    let details = serde_json::json!({ "mobile_money_actif": actif }).to_string();
    journaliser(
        tx,
        &Evenement {
            utilisateur_id: Some(session.utilisateur_id),
            action: "MODIFIER",
            objet_type: "tresorerie_configuration",
            objet_id: "1",
            details: &details,
        },
    )?;
    Ok(TresorerieConfiguration {
        mobile_money_actif: actif,
    })
}

fn verifier_numero_non_utilise(
    tx: &Transaction<'_>,
    type_compte: TypeCompte,
    banque_operateur: Option<&str>,
    numero: Option<&str>,
) -> Resultat<()> {
    let Some(numero) = numero else {
        return Ok(());
    };
    let utilise: bool = tx.query_row(
        "SELECT EXISTS(
            SELECT 1 FROM tresorerie_compte
            WHERE type_compte = ?1
              AND banque_operateur = ?2 COLLATE NOCASE
              AND numero = ?3
         )",
        ac_donnees::rusqlite::params![type_compte.libelle(), banque_operateur, numero],
        |ligne| ligne.get(0),
    )?;
    if utilise {
        return Err(ErreurTresorerie::NumeroDejaUtilise);
    }
    Ok(())
}

/// Crée un sous-compte à partir du compte racine de la classe.
fn creer_compte_comptable(
    tx: &Transaction<'_>,
    type_compte: TypeCompte,
    libelle: &str,
) -> Resultat<String> {
    let racine = type_compte.compte_base();
    let mouvements: bool = tx.query_row(
        "SELECT EXISTS(SELECT 1 FROM compta_ligne WHERE compte = ?1)",
        [racine],
        |ligne| ligne.get(0),
    )?;
    if mouvements {
        return Err(ErreurTresorerie::CompteRacineAvecMouvements(
            racine.to_owned(),
        ));
    }

    let libelle_racine = match type_compte {
        TypeCompte::Banque => "Banques",
        TypeCompte::Caisse => "Caisses",
        TypeCompte::MobileMoney => "Instruments de monnaie électronique",
    };
    tx.execute(
        "INSERT OR IGNORE INTO compta_compte (numero, libelle, mouvementable, actif)
         VALUES (?1, ?2, 0, 1)",
        ac_donnees::rusqlite::params![racine, libelle_racine],
    )?;
    tx.execute(
        "UPDATE compta_compte SET mouvementable = 0 WHERE numero = ?1",
        [racine],
    )?;

    let mut suffixes = BTreeSet::new();
    let mut stmt = tx.prepare(
        "SELECT numero FROM compta_compte
         WHERE substr(numero, 1, length(?1)) = ?1 AND length(numero) > length(?1)",
    )?;
    let mut lignes = stmt.query([racine])?;
    while let Some(ligne) = lignes.next()? {
        let numero: String = ligne.get(0)?;
        if let Some(suffixe) = numero
            .strip_prefix(racine)
            .and_then(|texte| texte.parse::<u64>().ok())
            .filter(|suffixe| *suffixe > 0)
        {
            suffixes.insert(suffixe);
        }
    }
    drop(lignes);
    drop(stmt);

    let mut suffixe = 1_u64;
    for utilise in suffixes {
        if utilise == suffixe {
            suffixe = suffixe
                .checked_add(1)
                .ok_or_else(|| ErreurTresorerie::SousCompteIndisponible(racine.to_owned()))?;
        } else if utilise > suffixe {
            break;
        }
    }
    let numero = format!("{racine}{suffixe}");
    if NumeroCompte::new(&numero).is_err() {
        return Err(ErreurTresorerie::SousCompteIndisponible(
            racine.to_owned(),
        ));
    }
    tx.execute(
        "INSERT INTO compta_compte (numero, libelle, mouvementable, actif)
         VALUES (?1, ?2, 1, 1)",
        ac_donnees::rusqlite::params![numero, libelle],
    )?;
    Ok(numero)
}

/// Alloue le code journal du compte : `CA` pour la caisse ; `BQn` / `MMn` sinon (contrat C3).
fn creer_journal_du_compte(tx: &Transaction<'_>, type_compte: TypeCompte) -> Resultat<String> {
    match type_compte {
        TypeCompte::Caisse => Ok("CA".to_owned()),
        TypeCompte::Banque | TypeCompte::MobileMoney => {
            journal_prochain_code(tx, type_compte.journal_prefixe())
        }
    }
}

/// Vérifie et renvoie la référence de pièce d'une ouverture non nulle.
fn valider_piece_ouverture(params: &CreerCompte) -> Resultat<Option<String>> {
    if params.solde_ouverture.est_zero() {
        return Ok(None);
    }
    let piece = params
        .piece_ouverture
        .as_deref()
        .map(str::trim)
        .filter(|piece| !piece.is_empty())
        .ok_or(ErreurTresorerie::PieceOuvertureObligatoire)?;
    Ok(Some(piece.to_owned()))
}

/// Vérifie et renvoie le compte comptable choisi pour équilibrer une ouverture non nulle.
fn valider_contrepartie_ouverture(
    tx: &Transaction<'_>,
    params: &CreerCompte,
) -> Resultat<Option<NumeroCompte>> {
    if params.solde_ouverture.est_zero() {
        return Ok(None);
    }
    let texte = params
        .compte_contrepartie_ouverture
        .as_deref()
        .map(str::trim)
        .filter(|texte| !texte.is_empty())
        .ok_or(ErreurTresorerie::ContrepartieOuvertureObligatoire)?;
    let compte = NumeroCompte::new(texte)?;
    if !contrepartie_mouvementable(tx, compte.as_str())? {
        return Err(ErreurTresorerie::ContrepartieInconnue(
            compte.to_string(),
        ));
    }
    Ok(Some(compte))
}

/// Écrit l'écriture AN du solde d'ouverture via `ac_compta::passer_ecriture` (C3, journal `AN`).
fn ecrire_an_ouverture(
    tx: &Transaction<'_>,
    session: &Session,
    compte: &TresorerieCompte,
    piece: &str,
    contrepartie: &NumeroCompte,
) -> Resultat<()> {
    let solde = compte.solde_ouverture.abs();
    let compte_comptable = NumeroCompte::new(&compte.compte_comptable)?;
    let lignes = if compte.solde_ouverture.est_negatif() {
        vec![
            LigneBrouillon::credit(compte_comptable, solde),
            LigneBrouillon::debit(contrepartie.clone(), solde),
        ]
    } else {
        vec![
            LigneBrouillon::debit(compte_comptable, solde),
            LigneBrouillon::credit(contrepartie.clone(), solde),
        ]
    };
    let brouillon = BrouillonEcriture {
        journal: "AN".to_owned(),
        date: chrono::Local::now().date_naive(),
        libelle: format!("Solde d'ouverture — {}", compte.libelle),
        piece: Some(piece.to_owned()),
        origine: ModuleOrigine::Tresorerie,
        reference_origine: Some(compte.compte_comptable.clone()),
        lignes,
    };
    passer_ecriture(tx, session, &brouillon)?;
    Ok(())
}

/// Crée un compte de trésorerie dans la transaction de l'appelant : sous-compte comptable,
/// journal dédié et, si le solde est non nul, écriture AN équilibrée.
pub fn creer_compte(
    tx: &Transaction<'_>,
    session: &Session,
    params: &CreerCompte,
) -> Resultat<TresorerieCompte> {
    let libelle = params.libelle.trim();
    if libelle.is_empty() {
        return Err(ErreurTresorerie::LibelleVide);
    }
    if libelle.chars().count() > 200 {
        return Err(ErreurTresorerie::LibelleTropLong);
    }
    if params.type_compte.ne_peut_pas_etre_negatif() && params.solde_ouverture.est_negatif() {
        return Err(ErreurTresorerie::CaisseSoldeNegatif(
            params.solde_ouverture.francs(),
        ));
    }

    let banque_operateur = match params.type_compte {
        TypeCompte::Banque => {
            let nom = params
                .banque_operateur
                .as_deref()
                .map(str::trim)
                .filter(|nom| !nom.is_empty())
                .ok_or(ErreurTresorerie::BanqueOperateurObligatoire)?;
            Some(nom.to_owned())
        }
        TypeCompte::MobileMoney => {
            if !configuration(tx)?.mobile_money_actif {
                return Err(ErreurTresorerie::MobileMoneyDesactive);
            }
            let nom = params
                .banque_operateur
                .as_deref()
                .map(str::trim)
                .filter(|nom| !nom.is_empty())
                .ok_or(ErreurTresorerie::OperateurObligatoire)?;
            Some(verifier_operateur_actif(tx, nom)?)
        }
        TypeCompte::Caisse => None,
    };

    let numero = match params.type_compte {
        TypeCompte::Caisse => None,
        TypeCompte::Banque => {
            let texte = params
                .numero
                .as_deref()
                .map(str::trim)
                .filter(|texte| !texte.is_empty())
                .ok_or(ErreurTresorerie::NumeroObligatoire)?;
            Some(texte.to_owned())
        }
        TypeCompte::MobileMoney => {
            let texte = params
                .numero
                .as_deref()
                .map(str::trim)
                .filter(|texte| !texte.is_empty())
                .ok_or(ErreurTresorerie::NumeroObligatoire)?;
            let telephone = Telephone::new(texte)
                .map_err(|_| ErreurTresorerie::TelephoneMobileMoneyInvalide)?;
            Some(telephone.as_str().to_owned())
        }
    };
    verifier_numero_non_utilise(
        tx,
        params.type_compte,
        banque_operateur.as_deref(),
        numero.as_deref(),
    )?;
    let piece_ouverture = valider_piece_ouverture(params)?;
    let contrepartie = valider_contrepartie_ouverture(tx, params)?;

    let compte_comptable = creer_compte_comptable(tx, params.type_compte, libelle)?;
    let journal = creer_journal_du_compte(tx, params.type_compte)?;
    let compte_journal = if params.type_compte == TypeCompte::Caisse {
        None
    } else {
        Some(compte_comptable.as_str())
    };
    journal_creer(
        tx,
        &journal,
        &format!("Journal de trésorerie — {journal}"),
        compte_journal,
    )?;
    tx.execute(
        "INSERT INTO tresorerie_compte
             (type_compte, libelle, banque_operateur, numero, compte_comptable, journal, solde_ouverture, actif)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 1)",
        ac_donnees::rusqlite::params![
            params.type_compte.libelle(),
            libelle,
            banque_operateur,
            numero,
            compte_comptable,
            journal,
            params.solde_ouverture.francs(),
        ],
    )?;
    let id = tx.last_insert_rowid();
    let compte = lire_compte(tx, id)?;
    if let (Some(piece), Some(contrepartie)) =
        (piece_ouverture.as_deref(), contrepartie.as_ref())
    {
        ecrire_an_ouverture(tx, session, &compte, piece, contrepartie)?;
    }

    let details = serde_json::json!({
        "compte_comptable": &compte.compte_comptable,
        "journal": &compte.journal,
        "solde_ouverture": compte.solde_ouverture.francs(),
        "type_compte": compte.type_compte.libelle(),
    })
    .to_string();
    let id_texte = compte.id.to_string();
    journaliser(
        tx,
        &Evenement {
            utilisateur_id: Some(session.utilisateur_id),
            action: "CREER",
            objet_type: "tresorerie_compte",
            objet_id: &id_texte,
            details: &details,
        },
    )?;
    Ok(compte)
}

/// Ajoute un opérateur Mobile Money en données, sans modifier le code du module.
pub fn creer_operateur(
    tx: &Transaction<'_>,
    session: &Session,
    params: &CreerOperateur,
) -> Resultat<TresorerieOperateur> {
    session.exiger(Permission::Administrer)?;
    let nom = params.nom.trim();
    if nom.is_empty() {
        return Err(ErreurTresorerie::NomOperateurVide);
    }
    if nom.chars().count() > 80 {
        return Err(ErreurTresorerie::NomOperateurTropLong);
    }
    let libelle = params
        .libelle
        .as_deref()
        .map(str::trim)
        .filter(|texte| !texte.is_empty());
    if libelle.is_some_and(|texte| texte.chars().count() > 120) {
        return Err(ErreurTresorerie::LibelleOperateurTropLong);
    }
    let existe: bool = tx.query_row(
        "SELECT EXISTS(SELECT 1 FROM tresorerie_operateur WHERE nom = ?1 COLLATE NOCASE)",
        [nom],
        |ligne| ligne.get(0),
    )?;
    if existe {
        return Err(ErreurTresorerie::OperateurDejaExistant(nom.to_owned()));
    }
    tx.execute(
        "INSERT INTO tresorerie_operateur (nom, libelle) VALUES (?1, ?2)",
        ac_donnees::rusqlite::params![nom, libelle],
    )?;
    let id = tx.last_insert_rowid();
    let details = serde_json::json!({ "nom": nom }).to_string();
    let id_texte = id.to_string();
    journaliser(
        tx,
        &Evenement {
            utilisateur_id: Some(session.utilisateur_id),
            action: "CREER",
            objet_type: "tresorerie_operateur",
            objet_id: &id_texte,
            details: &details,
        },
    )?;
    Ok(TresorerieOperateur {
        id,
        nom: nom.to_owned(),
        libelle: libelle.map(str::to_owned),
        actif: true,
    })
}

fn lire_compte_lignes(ligne: &ac_donnees::rusqlite::Row<'_>) -> Resultat<TresorerieCompte> {
    let id: i64 = ligne.get(0)?;
    let type_compte: String = ligne.get(1)?;
    let libelle: String = ligne.get(2)?;
    let banque_operateur: Option<String> = ligne.get(3)?;
    let numero: Option<String> = ligne.get(4)?;
    let compte_comptable: String = ligne.get(5)?;
    let journal: String = ligne.get(6)?;
    let solde_ouverture: i64 = ligne.get(7)?;
    let actif: i64 = ligne.get(8)?;
    let date_ouverture: String = ligne.get(9)?;
    Ok(TresorerieCompte {
        id,
        type_compte: TypeCompte::depuis_str(&type_compte)?,
        libelle,
        banque_operateur,
        numero: numero.map(|numero| masquer(&numero)),
        compte_comptable,
        journal,
        solde_ouverture: Montant::new(solde_ouverture)?,
        actif: actif != 0,
        date_ouverture,
    })
}

fn lire_compte(tx: &Transaction<'_>, id: i64) -> Resultat<TresorerieCompte> {
    let mut stmt = tx.prepare(
        "SELECT id, type_compte, libelle, banque_operateur, numero, compte_comptable, journal, solde_ouverture, actif, date_ouverture
         FROM tresorerie_compte WHERE id = ?1",
    )?;
    let mut lignes = stmt.query([id])?;
    let ligne = lignes
        .next()?
        .ok_or(ErreurTresorerie::CompteInconnu(id))?;
    lire_compte_lignes(&ligne)
}

/// Lit un compte par identifiant (erreur `COMPTE_INCONNU` si absent).
pub fn compte_par_id(tx: &Transaction<'_>, id: i64) -> Resultat<TresorerieCompte> {
    lire_compte(tx, id)
}

/// Liste les comptes de trésorerie actifs, par libellé.
pub fn comptes(tx: &Transaction<'_>) -> Resultat<Vec<TresorerieCompte>> {
    let mut stmt = tx.prepare(
        "SELECT id, type_compte, libelle, banque_operateur, numero, compte_comptable, journal, solde_ouverture, actif, date_ouverture
         FROM tresorerie_compte WHERE actif = 1 ORDER BY libelle, id",
    )?;
    let mut lignes = stmt.query([])?;
    let mut out = Vec::new();
    while let Some(ligne) = lignes.next()? {
        out.push(lire_compte_lignes(&ligne)?);
    }
    Ok(out)
}

/// Liste tous les opérateurs Mobile Money (actifs et désactivés), par nom.
pub fn operateurs(tx: &Transaction<'_>) -> Resultat<Vec<TresorerieOperateur>> {
    let mut stmt = tx.prepare(
        "SELECT id, nom, libelle, actif FROM tresorerie_operateur ORDER BY nom COLLATE NOCASE",
    )?;
    let mut lignes = stmt.query([])?;
    let mut out = Vec::new();
    while let Some(ligne) = lignes.next()? {
        let id: i64 = ligne.get(0)?;
        let nom: String = ligne.get(1)?;
        let libelle: Option<String> = ligne.get(2)?;
        let actif: i64 = ligne.get(3)?;
        out.push(TresorerieOperateur {
            id,
            nom,
            libelle,
            actif: actif != 0,
        });
    }
    Ok(out)
}

/// Soldes comptables des comptes actifs, y compris ceux qui n'ont encore aucun mouvement.
pub fn soldes_comptes(tx: &Transaction<'_>) -> Resultat<Vec<TresorerieSolde>> {
    let mut stmt = tx.prepare(
        "SELECT c.compte_comptable, c.type_compte, COALESCE(SUM(l.debit - l.credit), 0)
         FROM tresorerie_compte c
         LEFT JOIN compta_ligne l ON l.compte = c.compte_comptable
         WHERE c.actif = 1
         GROUP BY c.id, c.compte_comptable, c.type_compte
         ORDER BY c.compte_comptable",
    )?;
    let mut lignes = stmt.query([])?;
    let mut out = Vec::new();
    while let Some(ligne) = lignes.next()? {
        let numero: String = ligne.get(0)?;
        let type_compte: String = ligne.get(1)?;
        let solde: i64 = ligne.get(2)?;
        out.push(TresorerieSolde {
            compte: NumeroCompte::new(&numero)?,
            type_compte: TypeCompte::depuis_str(&type_compte)?,
            solde: Montant::new(solde)?,
        });
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ac_donnees::{Base, Role};

    fn base_test() -> Base {
        let mut base = Base::en_memoire().unwrap();
        base.migrer(&[
            ac_donnees::base::migrations(),
            ac_compta::migrations(),
            crate::migrations(),
        ])
        .unwrap();
        base.connexion()
            .execute(
                "INSERT INTO compta_compte (numero, libelle) VALUES ('101', 'Capital social')",
                [],
            )
            .unwrap();
        base
    }

    fn session_test() -> Session {
        Session {
            utilisateur_id: 1,
            role: Role::Comptable,
        }
    }

    fn session_admin_test() -> Session {
        Session {
            utilisateur_id: 1,
            role: Role::Administrateur,
        }
    }

    fn params_banque() -> CreerCompte {
        CreerCompte {
            type_compte: TypeCompte::Banque,
            libelle: "Banque OBA".to_owned(),
            banque_operateur: Some("OBA".to_owned()),
            numero: Some("000000101".to_owned()),
            solde_ouverture: Montant::new(50_000).unwrap(),
            piece_ouverture: Some("Relevé initial".to_owned()),
            compte_contrepartie_ouverture: Some("101".to_owned()),
        }
    }

    #[test]
    fn masque_les_identifiants() {
        assert_eq!(masquer("000000101"), "•••••0101");
        assert_eq!(masquer("0123456789"), "••••••6789");
        assert_eq!(masquer("0123"), "0123");
        assert_eq!(masquer("  "), "••••");
    }

    #[test]
    fn creation_banque_avec_solde_ouverture_et_audit() {
        let mut base = base_test();
        let tx = base.transaction().unwrap();
        let session = session_test();
        let compte = creer_compte(&tx, &session, &params_banque()).unwrap();
        tx.commit().unwrap();

        assert_eq!(compte.compte_comptable, "5211");
        assert_eq!(compte.journal, "BQ1");
        assert_eq!(compte.numero, Some("•••••0101".to_owned()));
        assert_eq!(compte.solde_ouverture, Montant::new(50_000).unwrap());
        assert!(compte.actif);

        let somme_debit: i64 = base
            .connexion()
            .query_row(
                "SELECT SUM(debit) FROM compta_ligne WHERE compte = '5211'",
                [],
                |ligne| ligne.get(0),
            )
            .unwrap();
        let somme_credit: i64 = base
            .connexion()
            .query_row(
                "SELECT SUM(credit) FROM compta_ligne WHERE compte = '101'",
                [],
                |ligne| ligne.get(0),
            )
            .unwrap();
        let audits: i64 = base
            .connexion()
            .query_row("SELECT COUNT(*) FROM donnees_audit", [], |ligne| ligne.get(0))
            .unwrap();
        let details_audit: String = base
            .connexion()
            .query_row("SELECT group_concat(details, ' ') FROM donnees_audit", [], |ligne| {
                ligne.get(0)
            })
            .unwrap();
        assert_eq!(somme_debit, 50_000);
        assert_eq!(somme_credit, 50_000);
        assert_eq!(audits, 2);
        assert!(!details_audit.contains("000000101"));

        let tx = base.transaction().unwrap();
        let lu = compte_par_id(&tx, compte.id).unwrap();
        assert_eq!(lu.numero, Some("•••••0101".to_owned()));
        assert_eq!(comptes(&tx).unwrap().len(), 1);
        assert_eq!(
            compte_par_id(&tx, 999).unwrap_err().code(),
            "COMPTE_INCONNU"
        );
    }

    #[test]
    fn creation_banque_avec_solde_crediteur_inverse_l_ecriture() {
        let mut base = base_test();
        let tx = base.transaction().unwrap();
        let session = session_test();
        let mut params = params_banque();
        params.solde_ouverture = Montant::new(-50_000).unwrap();
        let compte = creer_compte(&tx, &session, &params).unwrap();
        tx.commit().unwrap();

        let debit_contrepartie: i64 = base
            .connexion()
            .query_row(
                "SELECT SUM(debit) FROM compta_ligne WHERE compte = '101'",
                [],
                |ligne| ligne.get(0),
            )
            .unwrap();
        let credit_tresorerie: i64 = base
            .connexion()
            .query_row(
                "SELECT SUM(credit) FROM compta_ligne WHERE compte = ?1",
                [&compte.compte_comptable],
                |ligne| ligne.get(0),
            )
            .unwrap();
        assert_eq!(debit_contrepartie, 50_000);
        assert_eq!(credit_tresorerie, 50_000);
    }

    #[test]
    fn creation_de_deux_banques_alloue_des_sous_comptes_et_journaux_distincts() {
        let mut base = base_test();
        let tx = base.transaction().unwrap();
        let session = session_test();
        creer_compte(&tx, &session, &params_banque()).unwrap();
        let mut autre = params_banque();
        autre.libelle = "Banque de l'Est".to_owned();
        autre.numero = Some("000000202".to_owned());
        let compte = creer_compte(&tx, &session, &autre).unwrap();
        tx.commit().unwrap();
        assert_eq!(compte.compte_comptable, "5212");
        assert_eq!(compte.journal, "BQ2");
    }

    #[test]
    fn numero_de_compte_deja_utilise_refuse() {
        let mut base = base_test();
        let tx = base.transaction().unwrap();
        let session = session_test();
        creer_compte(&tx, &session, &params_banque()).unwrap();
        let erreur = creer_compte(&tx, &session, &params_banque()).unwrap_err();
        assert_eq!(erreur.code(), "NUMERO_DEJA_UTILISE");
    }

    #[test]
    fn libelle_vide_et_nom_de_banque_manquant_refuses() {
        let mut base = base_test();
        let tx = base.transaction().unwrap();
        let session = session_test();
        let mut params = params_banque();
        params.libelle = "   ".to_owned();
        assert_eq!(
            creer_compte(&tx, &session, &params).unwrap_err().code(),
            "LIBELLE_VIDE"
        );
        let mut params = params_banque();
        params.banque_operateur = None;
        assert_eq!(
            creer_compte(&tx, &session, &params).unwrap_err().code(),
            "BANQUE_OPERATEUR_OBLIGATOIRE"
        );
    }

    #[test]
    fn operateur_mobile_money_inconnu_refuse() {
        let mut base = base_test();
        let tx = base.transaction().unwrap();
        let session = session_test();
        let erreur = creer_compte(
            &tx,
            &session,
            &CreerCompte {
                type_compte: TypeCompte::MobileMoney,
                libelle: "Portefeuille inconnu".to_owned(),
                banque_operateur: Some("Opérateur inconnu".to_owned()),
                numero: Some("+229 01 97 12 34 56".to_owned()),
                solde_ouverture: Montant::ZERO,
                piece_ouverture: None,
                compte_contrepartie_ouverture: None,
            },
        )
        .unwrap_err();
        assert_eq!(erreur.code(), "OPERATEUR_INCONNU");
    }

    #[test]
    fn solde_non_nul_exige_une_piece_justificative() {
        let mut base = base_test();
        let tx = base.transaction().unwrap();
        let session = session_test();
        let mut params = params_banque();
        params.piece_ouverture = None;
        let erreur = creer_compte(&tx, &session, &params).unwrap_err();
        assert_eq!(erreur.code(), "PIECE_OUVERTURE_OBLIGATOIRE");
    }

    #[test]
    fn solde_non_nul_exige_une_contrepartie_existante() {
        let mut base = base_test();
        let tx = base.transaction().unwrap();
        let session = session_test();
        let mut params = params_banque();
        params.compte_contrepartie_ouverture = None;
        let erreur = creer_compte(&tx, &session, &params).unwrap_err();
        assert_eq!(
            erreur.code(),
            "CONTREPARTIE_OUVERTURE_OBLIGATOIRE"
        );
    }

    #[test]
    fn creation_caisse_sans_solde_cree_un_sous_compte_et_journal_ca() {
        let mut base = base_test();
        let tx = base.transaction().unwrap();
        let session = session_test();
        let compte = creer_compte(
            &tx,
            &session,
            &CreerCompte {
                type_compte: TypeCompte::Caisse,
                libelle: "Caisse centrale".to_owned(),
                banque_operateur: None,
                numero: None,
                solde_ouverture: Montant::ZERO,
                piece_ouverture: None,
                compte_contrepartie_ouverture: None,
            },
        )
        .unwrap();
        tx.commit().unwrap();
        assert_eq!(compte.compte_comptable, "5711");
        assert_eq!(compte.journal, "CA");
    }

    #[test]
    fn creation_caisse_negative_refusee() {
        let mut base = base_test();
        let tx = base.transaction().unwrap();
        let session = session_test();
        let erreur = creer_compte(
            &tx,
            &session,
            &CreerCompte {
                type_compte: TypeCompte::Caisse,
                libelle: "Caisse centrale".to_owned(),
                banque_operateur: None,
                numero: None,
                solde_ouverture: Montant::new(-10).unwrap(),
                piece_ouverture: None,
                compte_contrepartie_ouverture: None,
            },
        )
        .unwrap_err();
        assert_eq!(erreur.code(), "CAISSE_SOLDE_NEGATIF");
    }

    #[test]
    fn creation_mobile_money_normalise_et_masque_le_telephone() {
        let mut base = base_test();
        let tx = base.transaction().unwrap();
        let session = session_test();
        let compte = creer_compte(
            &tx,
            &session,
            &CreerCompte {
                type_compte: TypeCompte::MobileMoney,
                libelle: "Portefeuille Moov".to_owned(),
                banque_operateur: Some("Moov Money".to_owned()),
                numero: Some("+229 01 97 12 34 56".to_owned()),
                solde_ouverture: Montant::new(1_000).unwrap(),
                piece_ouverture: Some("Relevé Mobile Money".to_owned()),
                compte_contrepartie_ouverture: Some("101".to_owned()),
            },
        )
        .unwrap();
        tx.commit().unwrap();
        assert_eq!(compte.compte_comptable, "551");
        assert_eq!(compte.journal, "MM1");
        assert_eq!(compte.numero, Some("•••••••••3456".to_owned()));
    }

    #[test]
    fn numero_mobile_money_invalide_refuse_sans_le_reveler_dans_le_message() {
        let mut base = base_test();
        let tx = base.transaction().unwrap();
        let session = session_test();
        let erreur = creer_compte(
            &tx,
            &session,
            &CreerCompte {
                type_compte: TypeCompte::MobileMoney,
                libelle: "Portefeuille Moov".to_owned(),
                banque_operateur: Some("Moov Money".to_owned()),
                numero: Some("numéro privé".to_owned()),
                solde_ouverture: Montant::ZERO,
                piece_ouverture: None,
                compte_contrepartie_ouverture: None,
            },
        )
        .unwrap_err();
        assert_eq!(erreur.code(), "TELEPHONE_MOBILE_MONEY_INVALIDE");
        assert!(!erreur.to_string().contains("numéro privé"));
    }

    #[test]
    fn operateur_inactif_refuse_pour_un_nouveau_compte() {
        let mut base = base_test();
        base.connexion()
            .execute(
                "UPDATE tresorerie_operateur SET actif = 0 WHERE nom = 'Moov Money'",
                [],
            )
            .unwrap();
        let tx = base.transaction().unwrap();
        let session = session_test();
        let erreur = creer_compte(
            &tx,
            &session,
            &CreerCompte {
                type_compte: TypeCompte::MobileMoney,
                libelle: "Portefeuille Moov".to_owned(),
                banque_operateur: Some("Moov Money".to_owned()),
                numero: Some("+229 01 97 12 34 56".to_owned()),
                solde_ouverture: Montant::ZERO,
                piece_ouverture: None,
                compte_contrepartie_ouverture: None,
            },
        )
        .unwrap_err();
        assert_eq!(erreur.code(), "OPERATEUR_INACTIF");
    }

    #[test]
    fn creation_banque_sans_numero_refusee() {
        let mut base = base_test();
        let tx = base.transaction().unwrap();
        let session = session_test();
        let mut params = params_banque();
        params.numero = None;
        assert_eq!(
            creer_compte(&tx, &session, &params).unwrap_err().code(),
            "NUMERO_OBLIGATOIRE"
        );
    }

    #[test]
    fn soldes_lus_dans_compta_incluent_les_comptes_sans_mouvement() {
        let mut base = base_test();
        let tx = base.transaction().unwrap();
        let session = session_test();
        let mut params = params_banque();
        params.solde_ouverture = Montant::ZERO;
        params.compte_contrepartie_ouverture = None;
        creer_compte(&tx, &session, &params).unwrap();
        tx.commit().unwrap();

        let tx = base.transaction().unwrap();
        let soldes = soldes_comptes(&tx).unwrap();
        assert!(soldes.iter().any(|solde| {
            solde.compte == NumeroCompte::new("5211").unwrap()
                && solde.solde == Montant::ZERO
        }));
    }

    #[test]
    fn operateurs_preremplis_et_ajout_possible() {
        let mut base = base_test();
        let tx = base.transaction().unwrap();
        let session = session_admin_test();
        let avant = operateurs(&tx).unwrap();
        assert!(avant.iter().any(|operateur| operateur.nom == "MTN MoMo"));
        assert!(avant.iter().any(|operateur| operateur.nom == "Moov Money"));
        assert!(avant.iter().any(|operateur| operateur.nom == "Celtiis Cash"));
        tx.rollback().unwrap();

        let tx = base.transaction().unwrap();
        let ajoute = creer_operateur(
            &tx,
            &session,
            &CreerOperateur {
                nom: "Opérateur de test".to_owned(),
                libelle: Some("Test local".to_owned()),
            },
        )
        .unwrap();
        assert!(ajoute.actif);
        tx.commit().unwrap();

        let tx = base.transaction().unwrap();
        let apres = operateurs(&tx).unwrap();
        assert!(apres.iter().any(|operateur| operateur.nom == "Opérateur de test"));
    }

    #[test]
    fn module_mobile_money_est_desactivable_et_audite() {
        let mut base = base_test();
        let admin = session_admin_test();
        let tx = base.transaction().unwrap();
        assert!(configuration(&tx).unwrap().mobile_money_actif);
        assert!(!configurer_mobile_money(&tx, &admin, false)
            .unwrap()
            .mobile_money_actif);
        assert!(!configurer_mobile_money(&tx, &admin, false)
            .unwrap()
            .mobile_money_actif);
        tx.commit().unwrap();

        let tx = base.transaction().unwrap();
        let erreur = creer_compte(
            &tx,
            &session_test(),
            &CreerCompte {
                type_compte: TypeCompte::MobileMoney,
                libelle: "Portefeuille MTN".to_owned(),
                banque_operateur: Some("MTN MoMo".to_owned()),
                numero: Some("+229 01 97 12 34 56".to_owned()),
                solde_ouverture: Montant::ZERO,
                piece_ouverture: None,
                compte_contrepartie_ouverture: None,
            },
        )
        .unwrap_err();
        assert_eq!(erreur.code(), "MOBILE_MONEY_DESACTIVE");
        assert!(configurer_mobile_money(&tx, &admin, true)
            .unwrap()
            .mobile_money_actif);
        tx.commit().unwrap();

        let audits: i64 = base
            .connexion()
            .query_row("SELECT COUNT(*) FROM donnees_audit", [], |ligne| ligne.get(0))
            .unwrap();
        assert_eq!(audits, 2);
    }

    #[test]
    fn compte_contrepartie_absent_refuse() {
        let mut base = base_test();
        let tx = base.transaction().unwrap();
        let session = session_test();
        let mut params = params_banque();
        params.compte_contrepartie_ouverture = Some("1999".to_owned());
        let erreur = creer_compte(&tx, &session, &params).unwrap_err();
        assert_eq!(erreur.code(), "CONTREPARTIE_INCONNUE");
    }
}
