//! Comptes de trésorerie : modèles, création, lecture, masquage et soldes.

use ac_compta::{BrouillonEcriture, LigneBrouillon, ModuleOrigine, NumeroCompte, passer_ecriture};
use ac_donnees::{Session, Transaction};
use ac_noyau::Montant;
use serde::{Deserialize, Serialize};

use crate::journal::{journal_creer, journal_prochant_code};
use crate::{ErreurTresorerie, Resultat};

/// Type de compte de trésorerie (C5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TypeCompte {
    /// Compte bancaire (compte 521).
    Banque,
    /// Caisse d'espèces (compte 571) ; ne peut pas avoir un solde négatif.
    Caisse,
    /// Portefeuille Mobile Money (compte 55).
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

    /// Compte comptable de base du type (C5).
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
    /// Nom de la banque ou de l'opérateur (champ obligatoire pour `MOBILE_MONEY`).
    pub banque_operateur: Option<String>,
    /// Numéro de compte bancaire ou numéro de téléphone (champ obligatoire sauf pour la caisse).
    pub numero: Option<String>,
    /// Solde d'ouverture en FCFA ; négatif = solde créditeur (interdit pour la caisse).
    pub solde_ouverture: Montant,
}

/// Compte de trésorerie tel que stocké (numéros non masqués ; le masquage est en [`masquer`]).
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

/// Opérateur Mobile Money pré-remplis ou ajouté par l'utilisateur (`tresorerie_operateur`).
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

/// Solde comptable d'un compte de trésorerie lu dans `compta_ligne` (A1 fournira
/// `ac_compta::solde_compte` dans `ac-compta` ; TODO pour le Lot 2).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TresorerieSolde {
    /// Compte mouvementé (voir C5).
    pub compte: NumeroCompte,
    /// Type de trésorerie du compte.
    pub type_compte: TypeCompte,
    /// Solde comptable en FCFA (positif = solde débiteur).
    pub solde: Montant,
}

/// Compte comptable de référence pour les soldes d'ouverture (C5 — à valider par l'expert-comptable).
const COMPTE_AN: &str = "519";

/// Masque un numéro de compte ou de téléphone pour l'affichage : le début est remplacé par des
/// points et seuls les 4 derniers chiffres restent visibles (identifiants sensibles).
pub fn masquer(texte: &str) -> String {
    let chiffres: Vec<char> = texte.trim().chars().collect();
    if chiffres.len() <= 4 {
        return "••••".to_owned();
    }
    let debut = "•".repeat(chiffres.len() - 4);
    let fin: String = chiffres[chiffres.len() - 4..].iter().collect();
    format!("{debut}{fin}")
}

fn compte_existe(tx: &Transaction, numero: &str) -> Resultat<bool> {
    let existe: bool = tx.query_row(
        "SELECT EXISTS(SELECT 1 FROM compta_compte WHERE numero = ?1)",
        [numero],
        |r| r.get(0),
    )?;
    Ok(existe)
}

fn operateur_existe(tx: &Transaction, nom: &str) -> Resultat<bool> {
    let existe: bool = tx.query_row(
        "SELECT EXISTS(SELECT 1 FROM tresorerie_operateur WHERE nom = ?1)",
        [nom],
        |r| r.get(0),
    )?;
    Ok(existe)
}

/// Insère le compte comptable du type (C5) ; erreur si déjà présent.
fn creer_compte_comptable(tx: &Transaction, type_compte: TypeCompte) -> Resultat<String> {
    let base = NumeroCompte::new(type_compte.compte_base())?;
    if compte_existe(tx, base.as_str())? {
        return Err(ErreurTresorerie::CompteComptableExistant(base.to_owned()));
    }
    tx.execute(
        "INSERT INTO compta_compte (numero, libelle, actif) VALUES (?1, ?2, 1)",
        [base.as_str(), type_compte.libelle()],
    )?;
    Ok(base.to_owned())
}

/// Alloue le code journal du compte : `CA` pour la caisse ; `BQn` / `MMn` sinon (contrat C3).
fn creer_journal_du_compte(tx: &Transaction, type_compte: TypeCompte) -> Resultat<String> {
    match type_compte {
        TypeCompte::Caisse => Ok("CA".to_owned()),
        _ => journal_prochant_code(tx, type_compte.journal_prefixe()),
    }
}

/// Écrit l'écriture AN du solde d'ouverture via `ac_compta::passer_ecriture` (C3, journal `AN`).
/// Solde nul : aucune écriture (l'entrée doublement nulle n'est pas une écriture).
fn ecrire_an_ouverture(
    tx: &Transaction,
    session: &Session,
    compte: &TresorerieCompte,
) -> Resultat<()> {
    let solde = compte.solde_ouverture;
    if solde.est_zero() {
        return Ok(());
    }
    let compte_comptable = NumeroCompte::new(&compte.compte_comptable)?;
    let an = NumeroCompte::new(COMPTE_AN)?;
    let lignes = if solde.est_negatif() {
        vec![
            LigneBrouillon::credit(compte_comptable.clone(), solde.abs()),
            LigneBrouillon::debit(an.clone(), solde.abs()),
        ]
    } else {
        vec![
            LigneBrouillon::debit(compte_comptable.clone(), solde),
            LigneBrouillon::credit(an.clone(), solde),
        ]
    };
    let brouillon = BrouillonEcriture {
        journal: "AN".to_owned(),
        date: chrono::Utc::now().date_naive(),
        libelle: format!("Solde d'ouverture — {}", compte.libelle),
        piece: None,
        origine: ModuleOrigine::Tresorerie,
        reference_origine: Some(compte.compte_comptable.clone()),
        lignes,
    };
    passer_ecriture(tx, session, &brouillon)?;
    Ok(())
}

/// Crée un compte de trésorerie dans la transaction de l'appelant : sous-compte comptable,
/// journal déclaré, et — si le solde est non nul — l'écriture AN.
pub fn creer_compte(tx: &Transaction, session: &Session, params: &CreerCompte) -> Resultat<TresorerieCompte> {
    let libelle = params.libelle.trim();
    if libelle.is_empty() {
        return Err(ErreurTresorerie::LibelleVide);
    }
    let numero = params
        .numero
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty());
    if numero.is_none() && params.type_compte != TypeCompte::Caisse {
        return Err(ErreurTresorerie::NumeroObligatoire);
    }
    let banque_operateur = match (params.banque_operateur.as_deref().map(str::trim), params.type_compte) {
        (Some(""), _) => None,
        (Some(nom), TypeCompte::MobileMoney) => {
            if !operateur_existe(tx, nom)? {
                return Err(ErreurTresorerie::OperateurInconnu(nom.to_owned()));
            }
            Some(nom.to_owned())
        }
        (None, TypeCompte::MobileMoney) => return Err(ErreurTresorerie::OperateurObligatoire),
        (Some(nom), _) => Some(nom.to_owned()),
        (None, _) => None,
    };
    if params.type_compte.ne_peut_pas_etre_negatif() && params.solde_ouverture.est_negatif() {
        return Err(ErreurTresorerie::CaisseSoldeNegatif(
            params.solde_ouverture.francs(),
        ));
    }
    let compte_comptable = creer_compte_comptable(tx, params.type_compte)?;
    let journal = creer_journal_du_compte(tx, params.type_compte)?;
    journal_creer(tx, &journal, &format!("Journal de la trésorerie — {journal}"))?;
    tx.execute(
        "INSERT INTO tresorerie_compte
             (type_compte, libelle, banque_operateur, numero, compte_comptable, journal, solde_ouverture, actif)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 1)",
        [
            params.type_compte.libelle().to_owned(),
            libelle.to_owned(),
            banque_operateur.clone(),
            numero.map(str::to_owned),
            compte_comptable.clone(),
            journal.clone(),
            params.solde_ouverture.francs(),
        ],
    )?;
    let id = tx.last_insert_rowid();
    let compte = lire_compte(tx, id)?;
    ecrire_an_ouverture(tx, session, &compte)?;
    Ok(compte)
}

fn lire_compte_lignes(l: &ac_donnees::rusqlite::Row<'_>) -> Resultat<TresorerieCompte> {
    let id: i64 = l.get(0)?;
    let type_compte: String = l.get(1)?;
    let libelle: String = l.get(2)?;
    let banque_operateur: Option<String> = l.get(3)?;
    let numero: Option<String> = l.get(4)?;
    let compte_comptable: String = l.get(5)?;
    let journal: String = l.get(6)?;
    let solde_ouverture: i64 = l.get(7)?;
    let actif: i64 = l.get(8)?;
    let date_ouverture: String = l.get(9)?;
    Ok(TresorerieCompte {
        id,
        type_compte: TypeCompte::depuis_str(&type_compte)?,
        libelle,
        banque_operateur,
        numero: numero.map(|n| masquer(&n)),
        compte_comptable,
        journal,
        solde_ouverture: Montant::new(solde_ouverture)?,
        actif: actif != 0,
        date_ouverture,
    })
}

fn lire_compte(tx: &Transaction, id: i64) -> Resultat<TresorerieCompte> {
    let mut stmt = tx.prepare(
        "SELECT id, type_compte, libelle, banque_operateur, numero, compte_comptable, journal, solde_ouverture, actif, date_ouverture
         FROM tresorerie_compte WHERE id = ?1",
    )?;
    let mut lignes = stmt.query([id])?;
    let l = lignes.next()?.ok_or(ErreurTresorerie::CompteInconnu(id))?;
    Ok(lire_compte_lignes(&l)?)
}

/// Lit un compte par identifiant (erreur `COMPTE_INCONNU` si absent).
pub fn compte_par_id(tx: &Transaction, id: i64) -> Resultat<TresorerieCompte> {
    lire_compte(tx, id)
}

/// Liste les comptes de trésorerie actifs, par libellé.
pub fn comptes(tx: &Transaction) -> Resultat<Vec<TresorerieCompte>> {
    let mut stmt = tx.prepare(
        "SELECT id, type_compte, libelle, banque_operateur, numero, compte_comptable, journal, solde_ouverture, actif, date_ouverture
         FROM tresorerie_compte WHERE actif = 1 ORDER BY libelle, id",
    )?;
    let mut lignes = stmt.query([])?;
    let mut out = Vec::new();
    while let Some(l) = lignes.next()? {
        out.push(lire_compte_lignes(&l)?);
    }
    Ok(out)
}

/// Liste les opérateurs de trésorerie (pré-remplis + ajoutés).
pub fn operateurs(tx: &Transaction) -> Resultat<Vec<TresorerieOperateur>> {
    let mut stmt = tx.prepare("SELECT id, nom, libelle, actif FROM tresorerie_operateur ORDER BY nom")?;
    let mut lignes = stmt.query([])?;
    let mut out = Vec::new();
    while let Some(l) = lignes.next()? {
        let id: i64 = l.get(0)?;
        let nom: String = l.get(1)?;
        let libelle: Option<String> = l.get(2)?;
        let actif: i64 = l.get(3)?;
        out.push(TresorerieOperateur { id, nom, libelle, actif: actif != 0 });
    }
    Ok(out)
}

/// Soldes comptables lus dans `compta_ligne` (A1 fournit `solde_compte` dans `ac-compta`).
pub fn soldes_comptes(tx: &Transaction) -> Resultat<Vec<TresorerieSolde>> {
    let mut stmt = tx.prepare(
        "SELECT c.numero, c.type_compte, COALESCE(SUM(l.debit - l.credit), 0)
         FROM compta_ligne l
         JOIN tresorerie_compte c ON c.compte_comptable = l.compte
         GROUP BY l.compte ORDER BY l.compte",
    )?;
    let mut lignes = stmt.query([])?;
    let mut out = Vec::new();
    while let Some(l) = lignes.next()? {
        let numero: String = l.get(0)?;
        let type_compte: String = l.get(1)?;
        let solde: i64 = l.get(2)?;
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
        base.migrer(&[ac_donnees::base::migrations(), crate::migrations()])
            .unwrap();
        base
    }

    fn session_test() -> Session {
        Session {
            utilisuser_id: 1,
            role: Role::Comptable,
        }
    }

    fn params_banque() -> CreerCompte {
        CreerCompte {
            type_compte: TypeCompte::Banque,
            libelle: "Banque OBA".to_owned(),
            banque_operateur: Some("OBA".to_owned()),
            numero: Some("000000101".to_owned()),
            solde_ouverture: Montant::new(50_000).unwrap(),
        }
    }

    #[test]
    fn masque_les_idents() {
        assert_eq!(masquer("000000101"), "•••••0101");
        assert_eq!(masquer("0123456789"), "••••••6789");
        assert_eq!(masquer("0123"), "••••");
        assert_eq!(masquer("  "), "••••");
    }

    #[test]
    fn creation_banque_avec_solde_ouverture() {
        let base = base_test();
        let tx = base.transaction().unwrap();
        let session = session_test();
        let compte = creer_compte(&tx, &session, &params_banque()).unwrap();
        tx.commit().unwrap();
        assert_eq!(compte.compte_comptable, "521");
        assert_eq!(compte.journal, "BQ1");
        assert_eq!(compte.numero, Some("•••••0101".to_owned()));
        assert!(compte.actif);

        // L'écriture AN existe : D 521 de 50 000, C 519.
        let somme_debit: i64 = base
            .connexion()
            .query_row("SELECT SUM(debit) FROM compta_ligne WHERE compte = '521'", [], |r| r.get(0))
            .unwrap();
        let somme_credit: i64 = base
            .connexion()
            .query_row("SELECT SUM(credit) FROM compta_ligne WHERE compte = '519'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(somme_debit, 50_000);
        assert_eq!(somme_credit, 50_000);
    }

    #[test]
    fn creation_banque_seconde_a_le_journal_bq2() {
        let base = base_test();
        let tx = base.transaction().unwrap();
        let session = session_test();
        creer_compte(&tx, &session, &params_banque()).unwrap();
        let mut autre = params_banque();
        autre.libelle = "Banque de l'Est".to_owned();
        let compte = creer_compte(&tx, &session, &autre).unwrap();
        tx.commit().unwrap();
        assert_eq!(compte.journal, "BQ2");
    }

    #[test]
    fn creation_caisse() {
        let base = base_test();
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
            },
        )
        .unwrap();
        tx.commit().unwrap();
        assert_eq!(compte.compte_comptable, "571");
        assert_eq!(compte.journal, "CA");
    }

    #[test]
    fn creation_caisse_negative_refusee() {
        let base = base_test();
        let tx = base.transaction().unwrap();
        let session = session_test();
        let e = creer_compte(
            &tx,
            &session,
            &CreerCompte {
                type_compte: TypeCompte::Caisse,
                libelle: "Caisse centrale".to_owned(),
                banque_operateur: None,
                numero: None,
                solde_ouverture: Montant::new(-10).unwrap(),
            },
        )
        .unwrap_err();
        assert_eq!(e.code(), "CAISSE_SOLDE_NEGATIF");
    }

    #[test]
    fn creation_moov_money_sur_donnees() {
        let base = base_test();
        let tx = base.transaction().unwrap();
        let session = session_test();
        let compte = creer_compte(
            &tx,
            &session,
            &CreerCompte {
                type_compte: TypeCompte::MobileMoney,
                libelle: "Moov Money — close-out".to_owned(),
                banque_operateur: Some("Moov Money".to_owned()),
                numero: Some("0612 345 678".to_owned()),
                solde_ouverture: Montant::new(1_000).unwrap(),
            },
        )
        .unwrap();
        tx.commit().unwrap();
        assert_eq!(compte.compte_comptable, "55");
        assert_eq!(compte.journal, "MM1");
    }

    #[test]
    fn creation_moov_money_sans_operateur_refusee() {
        let base = base_test();
        let tx = base.transaction().unwrap();
        let session = session_test();
        let e = creer_compte(
            &tx,
            &session,
            &CreerCompte {
                type_compte: TypeCompte::MobileMoney,
                libelle: "Moov Money".to_owned(),
                banque_operateur: None,
                numero: Some("0612 345 678".to_owned()),
                solde_ouverture: Montant::ZERO,
            },
        )
        .unwrap_err();
        assert_eq!(e.code(), "OPERATEUR_OBLIGATOIRE");
    }

    #[test]
    fn creation_banque_sans_numero_refusee() {
        let base = base_test();
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
    fn soldes_lues_dans_compta() {
        let base = base_test();
        let tx = base.transaction().unwrap();
        let session = session_test();
        creer_compte(&tx, &session, &params_banque()).unwrap();
        tx.commit().unwrap();
        let soldes = soldes_comptes(&base.transaction().unwrap()).unwrap();
        assert!(soldes
            .iter()
            .any(|s| s.compte == NumeroCompte::new("521").unwrap()
                && s.solde == Montant::new(50_000).unwrap()));
    }

    #[test]
    fn operateurs_preremplis() {
        let base = base_test();
        let tx = base.transaction().unwrap();
        let out = operateurs(&tx).unwrap();
        assert!(out.iter().any(|o| o.nom == "MTN MoMo" && o.actif));
        assert!(out.iter().any(|o| o.nom == "Moov Money" && o.actif));
        assert!(out.iter().any(|o| o.nom == "Celtiis Cash" && o.actif));
    }
}
