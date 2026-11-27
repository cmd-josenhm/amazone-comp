//! Fichier clients : création, modification, recherche, désactivation.
//!
//! Un client **facturé ne se supprime jamais** : on le désactive ([`desactiver`]), ses factures
//! restent consultables. Le client « Comptant » ([`CODE_COMPTANT`], créé par la migration) est
//! obligatoire pour la vente au comptoir : il n'est ni modifié ni désactivé.
//!
//! Codes clients : `CLI-000001`, `CLI-000002`… attribués automatiquement ; ce code sert de
//! **tiers comptable** (contrat C3, comptes 41x).

use ac_donnees::audit::{journaliser, Evenement};
use ac_donnees::rusqlite::params;
use ac_donnees::{Connexion, Permission, Session, Transaction};
use ac_noyau::entreprise::{courriel_valide, Rccm, Telephone};
use ac_noyau::Ifu;
use serde::{Deserialize, Serialize};

use crate::{ErreurVentes, Resultat};

/// Longueur maximale du nom (ou de la raison sociale) d'un client.
pub const NOM_LONGUEUR_MAX: usize = 200;

/// Code du client « Comptant » (vente au comptoir), créé avec chaque dossier.
pub const CODE_COMPTANT: &str = "CLI-000001";

/// Nombre maximal de lignes renvoyées par [`rechercher`].
const RECHERCHE_LIMITE_MAX: u32 = 500;

/// Nombre de résultats par défaut pour [`rechercher`].
pub const RECHERCHE_LIMITE_DEFAUT: u32 = 50;

/// Type de client.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TypeClient {
    /// Particulier sans raison sociale.
    Particulier,
    /// Entreprise (société, commerce, artisan…).
    Entreprise,
    /// Administration ou organisme public.
    Administration,
}

impl TypeClient {
    /// Code stocké en base (`PARTICULIER`, `ENTREPRISE`, `ADMINISTRATION`).
    pub fn code(self) -> &'static str {
        match self {
            TypeClient::Particulier => "PARTICULIER",
            TypeClient::Entreprise => "ENTREPRISE",
            TypeClient::Administration => "ADMINISTRATION",
        }
    }

    /// Construit depuis le code stocké en base (`None` si la valeur est inconnue).
    pub fn depuis_code(code: &str) -> Option<Self> {
        match code {
            "PARTICULIER" => Some(TypeClient::Particulier),
            "ENTREPRISE" => Some(TypeClient::Entreprise),
            "ADMINISTRATION" => Some(TypeClient::Administration),
            _ => None,
        }
    }
}

/// Informations saisies sur un client (création et modification).
///
/// Tout champ facultatif peut être laissé vide (`None` ou chaîne vide) ; les identifiants sont
/// validés et normalisés par [`InformationsClient::valider`] (IFU 13 chiffres, RCCM, téléphone
/// béninois `+229…`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InformationsClient {
    /// Nom ou raison sociale (obligatoire).
    pub nom: String,
    /// Type de client.
    pub type_client: TypeClient,
    /// IFU (identifiant fiscal unique, 13 chiffres), facultatif.
    pub ifu: Option<String>,
    /// Numéro RCCM, facultatif.
    pub rccm: Option<String>,
    /// Adresse postale, facultative.
    pub adresse: Option<String>,
    /// Téléphone, facultatif.
    pub telephone: Option<String>,
    /// Adresse électronique, facultative.
    pub courriel: Option<String>,
    /// Vrai si le client retient la TVA à la source (État, sociétés désignées).
    pub retient_tva: bool,
    /// Vrai si le client est exonéré d'AIB (le motif est alors obligatoire).
    pub exonere_aib: bool,
    /// Motif de l'exonération d'AIB.
    pub motif_exoneration_aib: Option<String>,
}

impl InformationsClient {
    /// Valide et normalise les informations saisies.
    ///
    /// Le nom est dépouillé des espaces de bord ; l'IFU est réduit à ses 13 chiffres, le RCCM est
    /// normalisé ([`Rccm::new`]), le téléphone devient `+229…` ([`Telephone::new`]) ; les champs
    /// facultatifs vides deviennent `None`.
    pub fn valider(&self) -> Resultat<InformationsClient> {
        let nom = self.nom.trim();
        if nom.is_empty() {
            return Err(ErreurVentes::NomObligatoire);
        }
        if nom.chars().count() > NOM_LONGUEUR_MAX {
            return Err(ErreurVentes::NomTropLong(NOM_LONGUEUR_MAX));
        }
        let ifu = champ_present(self.ifu.as_deref())
            .map(|t| Ifu::new(t).map(|i| i.as_str().to_owned()))
            .transpose()?;
        let rccm = champ_present(self.rccm.as_deref())
            .map(|t| Rccm::new(t).map(|r| r.as_str().to_owned()))
            .transpose()?;
        let telephone = champ_present(self.telephone.as_deref())
            .map(|t| Telephone::new(t).map(|t| t.as_str().to_owned()))
            .transpose()?;
        let courriel = match champ_present(self.courriel.as_deref()) {
            Some(t) if !courriel_valide(t) => {
                return Err(ErreurVentes::CourrielInvalide(t.to_owned()));
            }
            autre => autre.map(str::to_owned),
        };
        let adresse = champ_present(self.adresse.as_deref()).map(str::to_owned);
        let motif_exoneration_aib =
            champ_present(self.motif_exoneration_aib.as_deref()).map(str::to_owned);
        if self.exonere_aib && motif_exoneration_aib.is_none() {
            return Err(ErreurVentes::MotifExonerationObligatoire);
        }
        Ok(InformationsClient {
            nom: nom.to_owned(),
            type_client: self.type_client,
            ifu,
            rccm,
            adresse,
            telephone,
            courriel,
            retient_tva: self.retient_tva,
            exonere_aib: self.exonere_aib,
            motif_exoneration_aib,
        })
    }
}

/// Champ facultatif : `None` si absent ou réduit à des espaces.
fn champ_present(texte: Option<&str>) -> Option<&str> {
    texte.map(str::trim).filter(|t| !t.is_empty())
}

/// Client enregistré dans le fichier clients.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Client {
    /// Identifiant interne (ne pas exposer dans les documents).
    pub id: i64,
    /// Code stable `CLI-…` (tiers comptable, contrat C3).
    pub code: String,
    /// Nom ou raison sociale.
    pub nom: String,
    /// Type de client.
    pub type_client: TypeClient,
    /// IFU (13 chiffres) s'il existe.
    pub ifu: Option<String>,
    /// Numéro RCCM.
    pub rccm: Option<String>,
    /// Adresse postale.
    pub adresse: Option<String>,
    /// Téléphone normalisé (`+229…`).
    pub telephone: Option<String>,
    /// Adresse électronique.
    pub courriel: Option<String>,
    /// Vrai si le client retient la TVA à la source.
    pub retient_tva: bool,
    /// Vrai si le client est exonéré d'AIB.
    pub exonere_aib: bool,
    /// Motif de l'exonération d'AIB.
    pub motif_exoneration_aib: Option<String>,
    /// Actif. Un client facturé n'est jamais supprimé : il est désactivé.
    pub actif: bool,
    /// Date de création (ISO-8601).
    pub cree_le: String,
    /// Date de dernière modification (ISO-8601).
    pub modifie_le: Option<String>,
}

/// Crée un client dans la transaction de l'appelant, avec journal d'audit (règle R5).
///
/// Le code `CLI-…` est attribué automatiquement. L'IFU, s'il est renseigné, doit être libre.
pub fn creer(
    tx: &Transaction<'_>,
    session: &Session,
    informations: &InformationsClient,
) -> Resultat<Client> {
    session.exiger(Permission::Saisir)?;
    let informations = informations.valider()?;
    controler_ifu_libre(tx, &informations.ifu, None)?;
    let code = attribuer_code(tx)?;
    tx.execute(
        "INSERT INTO ventes_client
            (code, nom, type_client, ifu, rccm, adresse, telephone, courriel,
             retient_tva, exonere_aib, motif_exonere_aib, actif)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, 1)",
        params![
            code,
            informations.nom,
            informations.type_client.code(),
            informations.ifu,
            informations.rccm,
            informations.adresse,
            informations.telephone,
            informations.courriel,
            informations.retient_tva,
            informations.exonere_aib,
            informations.motif_exoneration_aib,
        ],
    )?;
    let id = tx.last_insert_rowid();
    journaliser(
        tx,
        &Evenement {
            utilisateur_id: Some(session.utilisateur_id),
            action: "CREER",
            objet_type: "ventes_client",
            objet_id: &code,
            details: &details_audit(&code, &informations.nom),
        },
    )?;
    obtenir_tx(tx, id)?.ok_or(ErreurVentes::DonneeIncoherente(format!("client {id}")))
}

/// Modifie un client existant (sauf le client « Comptant »), avec journal d'audit.
pub fn modifier(
    tx: &Transaction<'_>,
    session: &Session,
    id: i64,
    informations: &InformationsClient,
) -> Resultat<Client> {
    session.exiger(Permission::Saisir)?;
    let informations = informations.valider()?;
    let existant = client_ou_erreur(tx, id)?;
    controler_comptant(&existant)?;
    controler_ifu_libre(tx, &informations.ifu, Some(id))?;
    tx.execute(
        "UPDATE ventes_client
         SET nom = ?2, type_client = ?3, ifu = ?4, rccm = ?5, adresse = ?6, telephone = ?7,
             courriel = ?8, retient_tva = ?9, exonere_aib = ?10, motif_exonere_aib = ?11,
             modifie_le = strftime('%Y-%m-%dT%H:%M:%SZ','now')
         WHERE id = ?1",
        params![
            id,
            informations.nom,
            informations.type_client.code(),
            informations.ifu,
            informations.rccm,
            informations.adresse,
            informations.telephone,
            informations.courriel,
            informations.retient_tva,
            informations.exonere_aib,
            informations.motif_exoneration_aib,
        ],
    )?;
    journaliser(
        tx,
        &Evenement {
            utilisateur_id: Some(session.utilisateur_id),
            action: "MODIFIER",
            objet_type: "ventes_client",
            objet_id: &existant.code,
            details: &details_audit(&existant.code, &informations.nom),
        },
    )?;
    obtenir_tx(tx, id)?.ok_or(ErreurVentes::DonneeIncoherente(format!("client {id}")))
}

/// Désactive un client (jamais de suppression : un client facturé reste dans l'historique).
///
/// Le client « Comptant » ne peut pas être désactivé. Un client déjà désactivé est refusé.
pub fn desactiver(tx: &Transaction<'_>, session: &Session, id: i64) -> Resultat<()> {
    session.exiger(Permission::Saisir)?;
    let client = client_ou_erreur(tx, id)?;
    controler_comptant(&client)?;
    if !client.actif {
        return Err(ErreurVentes::ClientDejaInactif(client.code));
    }
    tx.execute(
        "UPDATE ventes_client
         SET actif = 0, modifie_le = strftime('%Y-%m-%dT%H:%M:%SZ','now')
         WHERE id = ?1",
        [id],
    )?;
    journaliser(
        tx,
        &Evenement {
            utilisateur_id: Some(session.utilisateur_id),
            action: "DESACTIVER",
            objet_type: "ventes_client",
            objet_id: &client.code,
            details: &details_audit(&client.code, &client.nom),
        },
    )?;
    Ok(())
}

/// Réactive un client précédemment désactivé.
pub fn reactiver(tx: &Transaction<'_>, session: &Session, id: i64) -> Resultat<()> {
    session.exiger(Permission::Saisir)?;
    let client = client_ou_erreur(tx, id)?;
    if client.actif {
        return Err(ErreurVentes::ClientDejaActif(client.code));
    }
    tx.execute(
        "UPDATE ventes_client
         SET actif = 1, modifie_le = strftime('%Y-%m-%dT%H:%M:%SZ','now')
         WHERE id = ?1",
        [id],
    )?;
    journaliser(
        tx,
        &Evenement {
            utilisateur_id: Some(session.utilisateur_id),
            action: "REACTIVER",
            objet_type: "ventes_client",
            objet_id: &client.code,
            details: &details_audit(&client.code, &client.nom),
        },
    )?;
    Ok(())
}

/// Lit un client par son identifiant interne (`None` s'il n'existe pas).
pub fn obtenir(conn: &Connexion, id: i64) -> Resultat<Option<Client>> {
    let mut stmt = conn.prepare(&requete_client("WHERE id = ?1"))?;
    let mut lignes = stmt.query([id])?;
    match lignes.next()? {
        Some(ligne) => Ok(Some(lire_client(ligne)?)),
        None => Ok(None),
    }
}

/// Recherche des clients par nom, code, IFU ou téléphone (vide = tout lister).
///
/// La recherche est insensible à la casse (ASCII) et tolère les espaces saisis dans un numéro de
/// téléphone (`01 97 12 34 56` trouve `+2290197123456`). Les clients désactivés n'apparaissent
/// que si `inclure_inactifs` est vrai. Au plus `limite` lignes (borné à 500).
pub fn rechercher(
    conn: &Connexion,
    texte: &str,
    inclure_inactifs: bool,
    limite: u32,
) -> Resultat<Vec<Client>> {
    let limite = i64::from(limite.clamp(1, RECHERCHE_LIMITE_MAX));
    let texte = texte.trim();
    let motif = motif_like(texte);
    let chiffres: String = texte.chars().filter(|c| c.is_ascii_digit()).collect();
    let motif_telephone = if chiffres.is_empty() || chiffres == texte {
        motif.clone()
    } else {
        motif_like(&chiffres)
    };
    let filtre = !texte.is_empty();
    let mut stmt = conn.prepare(&requete_client(
        "WHERE (?1 OR actif = 1)
           AND (?2 = 0
                OR nom LIKE ?3 ESCAPE '\\'
                OR code LIKE ?3 ESCAPE '\\'
                OR ifu LIKE ?3 ESCAPE '\\'
                OR telephone LIKE ?3 ESCAPE '\\'
                OR telephone LIKE ?4 ESCAPE '\\')
         ORDER BY nom, id
         LIMIT ?5",
    ))?;
    let lignes = stmt.query_map(
        params![inclure_inactifs, filtre, motif, motif_telephone, limite],
        lire_client,
    )?;
    let mut clients = Vec::new();
    for ligne in lignes {
        clients.push(ligne?);
    }
    Ok(clients)
}

/// Liste complète des colonnes, avec la clause de sélection (`WHERE …`, `ORDER BY …`).
fn requete_client(selection: &str) -> String {
    format!(
        "SELECT id, code, nom, type_client, ifu, rccm, adresse, telephone, courriel,
                retient_tva, exonere_aib, motif_exonere_aib, actif, cree_le, modifie_le
         FROM ventes_client {selection}"
    )
}

/// Motif `LIKE` enveloppant le texte, avec `%`, `_` et `\` échappés (`ESCAPE '\'`).
fn motif_like(texte: &str) -> String {
    let echappe = texte
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_");
    format!("%{echappe}%")
}

/// Lit une ligne de [`requete_client`].
fn lire_client(ligne: &ac_donnees::rusqlite::Row<'_>) -> ac_donnees::rusqlite::Result<Client> {
    let code_type: String = ligne.get(3)?;
    let type_client = TypeClient::depuis_code(&code_type).ok_or_else(|| {
        ac_donnees::rusqlite::Error::FromSqlConversionFailure(
            3,
            ac_donnees::rusqlite::types::Type::Text,
            Box::new(ErreurVentes::DonneeIncoherente(format!(
                "type_client {code_type}"
            ))),
        )
    })?;
    Ok(Client {
        id: ligne.get(0)?,
        code: ligne.get(1)?,
        nom: ligne.get(2)?,
        type_client,
        ifu: ligne.get(4)?,
        rccm: ligne.get(5)?,
        adresse: ligne.get(6)?,
        telephone: ligne.get(7)?,
        courriel: ligne.get(8)?,
        retient_tva: ligne.get(9)?,
        exonere_aib: ligne.get(10)?,
        motif_exoneration_aib: ligne.get(11)?,
        actif: ligne.get(12)?,
        cree_le: ligne.get(13)?,
        modifie_le: ligne.get(14)?,
    })
}

/// Code suivant : `CLI-` + numéro à 6 chiffres, d'après le plus grand code existant.
fn attribuer_code(tx: &Transaction<'_>) -> Resultat<String> {
    let suivant: i64 = tx.query_row(
        "SELECT COALESCE(MAX(CAST(substr(code, 5) AS INTEGER)), 0) + 1 FROM ventes_client",
        [],
        |ligne| ligne.get(0),
    )?;
    Ok(format!("CLI-{suivant:06}"))
}

/// Détails JSON d'un événement d'audit (format canonique : clés triées par `serde_json`).
fn details_audit(code: &str, nom: &str) -> String {
    let mut objet = serde_json::Map::new();
    objet.insert("code".to_owned(), code.into());
    objet.insert("nom".to_owned(), nom.into());
    serde_json::Value::Object(objet).to_string()
}

/// Lit un client ou renvoie [`ErreurVentes::ClientInconnu`].
fn client_ou_erreur(tx: &Transaction<'_>, id: i64) -> Resultat<Client> {
    obtenir_tx(tx, id)?.ok_or(ErreurVentes::ClientInconnu(id))
}

/// [`obtenir`] dans une transaction.
fn obtenir_tx(tx: &Transaction<'_>, id: i64) -> Resultat<Option<Client>> {
    let mut stmt = tx.prepare(&requete_client("WHERE id = ?1"))?;
    let mut lignes = stmt.query([id])?;
    match lignes.next()? {
        Some(ligne) => Ok(Some(lire_client(ligne)?)),
        None => Ok(None),
    }
}

/// Le client « Comptant » ne se modifie pas et ne se désactive pas.
fn controler_comptant(client: &Client) -> Resultat<()> {
    if client.code == CODE_COMPTANT {
        Err(ErreurVentes::ClientComptantProtege)
    } else {
        Ok(())
    }
}

/// Un IFU identifie un seul client (sauf le client lui-même en modification).
fn controler_ifu_libre(
    tx: &Transaction<'_>,
    ifu: &Option<String>,
    sauf_id: Option<i64>,
) -> Resultat<()> {
    let Some(ifu) = ifu else {
        return Ok(());
    };
    let deja_pris: bool = tx.query_row(
        "SELECT EXISTS(SELECT 1 FROM ventes_client WHERE ifu = ?1 AND id != ?2)",
        params![ifu, sauf_id.unwrap_or(-1)],
        |ligne| ligne.get(0),
    )?;
    if deja_pris {
        Err(ErreurVentes::IfuDejaUtilise(ifu.clone()))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ac_donnees::{Base, Role};

    fn base_avec_clients() -> Base {
        let mut base = Base::en_memoire().unwrap();
        base.migrer(&[ac_donnees::base::migrations(), crate::migrations()])
            .unwrap();
        base
    }

    fn session(role: Role) -> Session {
        Session {
            utilisateur_id: 7,
            role,
        }
    }

    fn informations(nom: &str) -> InformationsClient {
        InformationsClient {
            nom: nom.to_owned(),
            type_client: TypeClient::Particulier,
            ifu: None,
            rccm: None,
            adresse: None,
            telephone: None,
            courriel: None,
            retient_tva: false,
            exonere_aib: false,
            motif_exoneration_aib: None,
        }
    }

    #[test]
    fn client_comptant_par_defaut_et_protege() {
        let mut base = base_avec_clients();
        let saisie = session(Role::Saisie);
        let comptant = obtenir(base.connexion(), 1).unwrap().unwrap();
        assert_eq!(comptant.code, CODE_COMPTANT);
        assert_eq!(comptant.nom, "Client comptant");
        assert!(comptant.actif);

        // Ni modifié, ni désactivé.
        let tx = base.transaction().unwrap();
        assert!(matches!(
            modifier(&tx, &saisie, 1, &informations("Autre nom")),
            Err(ErreurVentes::ClientComptantProtege)
        ));
        assert!(matches!(
            desactiver(&tx, &saisie, 1),
            Err(ErreurVentes::ClientComptantProtege)
        ));
    }

    #[test]
    fn codes_attribues_automatiquement() {
        let mut base = base_avec_clients();
        let saisie = session(Role::Saisie);
        let tx = base.transaction().unwrap();
        // Le client Comptant porte CLI-000001 : les suivants sont CLI-000002, CLI-000003.
        let premier = creer(&tx, &saisie, &informations("Boutique Étoile")).unwrap();
        assert_eq!(premier.code, "CLI-000002");
        let deuxieme = creer(&tx, &saisie, &informations("Kiosque Amen")).unwrap();
        assert_eq!(deuxieme.code, "CLI-000003");
        assert!(premier.actif);
    }

    #[test]
    fn validation_et_normalisation_des_identifiants() {
        let mut base = base_avec_clients();
        let saisie = session(Role::Saisie);
        let tx = base.transaction().unwrap();

        // Nom obligatoire, longueur maximale 200 (201 refusé).
        assert!(matches!(
            creer(&tx, &saisie, &informations("   ")),
            Err(ErreurVentes::NomObligatoire)
        ));
        let trop_long = "x".repeat(NOM_LONGUEUR_MAX + 1);
        assert!(matches!(
            creer(&tx, &saisie, &informations(&trop_long)),
            Err(ErreurVentes::NomTropLong(200))
        ));

        // IFU facultatif mais validé : 13 chiffres, espaces ignorés.
        let mut avec_ifu = informations("Boutique Étoile");
        avec_ifu.ifu = Some("3201 9123 45678".to_owned());
        avec_ifu.telephone = Some("01 97 12 34 56".to_owned());
        let client = creer(&tx, &saisie, &avec_ifu).unwrap();
        assert_eq!(client.ifu.as_deref(), Some("3201912345678"));
        assert_eq!(client.telephone.as_deref(), Some("+2290197123456"));

        let mut ifu_court = informations("Mauvais IFU");
        ifu_court.ifu = Some("320191234567".to_owned()); // 12 chiffres
        assert!(creer(&tx, &saisie, &ifu_court).is_err());

        // Courriel contrôlé.
        let mut mauvais_courriel = informations("Sans courriel");
        mauvais_courriel.courriel = Some("pas-un-courriel".to_owned());
        assert!(matches!(
            creer(&tx, &saisie, &mauvais_courriel),
            Err(ErreurVentes::CourrielInvalide(_))
        ));

        // Exonération d'AIB : motif obligatoire.
        let mut exonere = informations("État du Bénin");
        exonere.exonere_aib = true;
        assert!(matches!(
            creer(&tx, &saisie, &exonere),
            Err(ErreurVentes::MotifExonerationObligatoire)
        ));
        exonere.motif_exoneration_aib = Some("  Convention n° 42  ".to_owned());
        let client = creer(&tx, &saisie, &exonere).unwrap();
        assert_eq!(
            client.motif_exoneration_aib.as_deref(),
            Some("Convention n° 42")
        );
    }

    #[test]
    fn ifu_unique_entre_clients() {
        let mut base = base_avec_clients();
        let saisie = session(Role::Saisie);
        let mut a = informations("Client A");
        a.ifu = Some("3201912345678".to_owned());
        let mut b = informations("Client B");
        b.ifu = Some("3201912345678".to_owned());

        let tx = base.transaction().unwrap();
        let client_a = creer(&tx, &saisie, &a).unwrap();
        assert!(matches!(
            creer(&tx, &saisie, &b),
            Err(ErreurVentes::IfuDejaUtilise(_))
        ));

        // Se réattribuer son propre IFU est permis ; le prêter à un autre ne l'est pas.
        assert!(modifier(&tx, &saisie, client_a.id, &a).is_ok());
        b.ifu = Some("3201912345679".to_owned());
        let client_b = creer(&tx, &saisie, &b).unwrap();
        let mut b_vers_ifu_a = b.clone();
        b_vers_ifu_a.ifu = Some("3201912345678".to_owned());
        assert!(matches!(
            modifier(&tx, &saisie, client_b.id, &b_vers_ifu_a),
            Err(ErreurVentes::IfuDejaUtilise(_))
        ));
    }

    #[test]
    fn recherche_nom_ifu_et_telephone() {
        let mut base = base_avec_clients();
        let saisie = session(Role::Saisie);
        let mut etoile = informations("Boutique Étoile");
        etoile.ifu = Some("3201912345678".to_owned());
        etoile.telephone = Some("01 97 12  34 56".to_owned());
        let mut amen = informations("Kiosque Amen");
        amen.telephone = Some("01 96 00 11 22".to_owned());
        let mut cent = informations("100% Sud");
        cent.type_client = TypeClient::Entreprise;
        {
            let tx = base.transaction().unwrap();
            creer(&tx, &saisie, &etoile).unwrap();
            creer(&tx, &saisie, &amen).unwrap();
            creer(&tx, &saisie, &cent).unwrap();
        }
        let conn = base.connexion();

        // Par nom (sans tenir compte de la casse, sous-chaîne acceptée).
        let trouves = rechercher(conn, "AMEN", false, 50).unwrap();
        assert_eq!(trouves.len(), 1);
        assert_eq!(trouves[0].nom, "Kiosque Amen");

        // Par IFU, partiel.
        let trouves = rechercher(conn, "345678", false, 50).unwrap();
        assert_eq!(trouves.len(), 1);
        assert_eq!(trouves[0].nom, "Boutique Étoile");

        // Par téléphone, avec ou sans espaces (stocké +2290197123456).
        for texte in ["01 97 12 34 56", "97123456", "0197123456"] {
            let trouves = rechercher(conn, texte, false, 50).unwrap();
            assert_eq!(trouves.len(), 1, "recherche « {texte} »");
            assert_eq!(trouves[0].nom, "Boutique Étoile");
        }

        // Le joker « % » est pris au littéral : « 100% Sud » seulement, pas tout le fichier.
        let trouves = rechercher(conn, "%", false, 50).unwrap();
        assert_eq!(trouves.len(), 1);
        assert_eq!(trouves[0].nom, "100% Sud");

        // Vide = tout lister (les actifs : 3 clients + le Comptant).
        let trouves = rechercher(conn, "", false, 50).unwrap();
        assert_eq!(trouves.len(), 4);
    }

    #[test]
    fn desactivation_puis_reactivation() {
        let mut base = base_avec_clients();
        let saisie = session(Role::Saisie);
        let tx = base.transaction().unwrap();
        let client = creer(&tx, &saisie, &informations("Boutique Étoile")).unwrap();
        desactiver(&tx, &saisie, client.id).unwrap();
        assert!(matches!(
            desactiver(&tx, &saisie, client.id),
            Err(ErreurVentes::ClientDejaInactif(_))
        ));
        reactiver(&tx, &saisie, client.id).unwrap();
        assert!(matches!(
            reactiver(&tx, &saisie, client.id),
            Err(ErreurVentes::ClientDejaActif(_))
        ));
        desactiver(&tx, &saisie, client.id).unwrap();
        tx.commit().unwrap();

        // Un client désactivé sort de la recherche courante mais reste consultable.
        let actifs = rechercher(base.connexion(), "", false, 50).unwrap();
        assert_eq!(actifs.len(), 1); // le client Comptant seul
        assert_eq!(actifs[0].code, CODE_COMPTANT);
        let tous = rechercher(base.connexion(), "", true, 50).unwrap();
        assert_eq!(tous.len(), 2); // Comptant + Étoile désactivé
        let lu = obtenir(base.connexion(), client.id).unwrap().unwrap();
        assert!(!lu.actif);

        // Un client inconnu est refusé.
        let tx = base.transaction().unwrap();
        assert!(matches!(
            desactiver(&tx, &saisie, 4242),
            Err(ErreurVentes::ClientInconnu(4242))
        ));
    }

    #[test]
    fn permissions_selon_le_role() {
        let mut base = base_avec_clients();
        let lecture = session(Role::Lecture);
        let tx = base.transaction().unwrap();
        // Le rôle Saisie peut créer ; le rôle Lecture ne peut pas.
        assert!(creer(&tx, &lecture, &informations("Interdit")).is_err());
        assert!(creer(&tx, &session(Role::Saisie), &informations("Permis")).is_ok());
    }

    #[test]
    fn journal_d_audit_chaine() {
        let mut base = base_avec_clients();
        let saisie = session(Role::Saisie);
        {
            let tx = base.transaction().unwrap();
            let client = creer(&tx, &saisie, &informations("Boutique Étoile")).unwrap();
            modifier(
                &tx,
                &saisie,
                client.id,
                &informations("Boutique Étoile SARL"),
            )
            .unwrap();
            desactiver(&tx, &saisie, client.id).unwrap();
            tx.commit().unwrap();
        }
        assert_eq!(
            ac_donnees::audit::verifier_chaine(base.connexion()).unwrap(),
            3
        );
    }

    #[test]
    fn modification_complete() {
        let mut base = base_avec_clients();
        let saisie = session(Role::Saisie);
        let tx = base.transaction().unwrap();
        let client = creer(&tx, &saisie, &informations("Boutique Étoile")).unwrap();
        let mut complet = informations("Boutique Étoile SARL");
        complet.type_client = TypeClient::Entreprise;
        complet.ifu = Some("3201912345678".to_owned());
        complet.rccm = Some("rb/cot/23 b 12345".to_owned());
        complet.adresse = Some("  Ganhi, Cotonou  ".to_owned());
        complet.telephone = Some("01 97 12 34 56".to_owned());
        complet.courriel = Some("contact@etoile.bj".to_owned());
        complet.retient_tva = true;
        complet.exonere_aib = true;
        complet.motif_exoneration_aib = Some("Marché public".to_owned());
        let modifie = modifier(&tx, &saisie, client.id, &complet).unwrap();
        assert_eq!(modifie.nom, "Boutique Étoile SARL");
        assert_eq!(modifie.type_client, TypeClient::Entreprise);
        assert_eq!(modifie.ifu.as_deref(), Some("3201912345678"));
        assert_eq!(modifie.rccm.as_deref(), Some("RB/COT/23 B 12345"));
        assert_eq!(modifie.adresse.as_deref(), Some("Ganhi, Cotonou"));
        assert_eq!(modifie.telephone.as_deref(), Some("+2290197123456"));
        assert_eq!(modifie.courriel.as_deref(), Some("contact@etoile.bj"));
        assert!(modifie.retient_tva);
        assert!(modifie.exonere_aib);
        assert!(modifie.modifie_le.is_some());
    }
}
