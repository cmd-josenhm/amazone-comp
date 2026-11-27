//! Fiche légale de l'entreprise (ADR 0009, contrat C11).
//!
//! La **même** fiche et la **même** validation servent :
//! - dans l'application (assistant de premier lancement, Paramètres) ;
//! - sur la plateforme en ligne (inscription de l'entreprise).
//!
//! Les champs arrivent tels que saisis (textes bruts) : [`FicheEntreprise::valider`] renvoie la liste de
//! **tous** les problèmes, champ par champ, pour que l'interface les affiche d'un coup.
//! Les formats RCCM et téléphone ne sont pas encore confirmés officiellement : ils ne produisent que des
//! **avertissements** (issue `fiscal-a-verifier`), jamais un blocage à tort.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::{Date, ErreurNoyau, Ifu, Resultat};

// ---------------------------------------------------------------------------------------------
// Listes de choix
// ---------------------------------------------------------------------------------------------

/// Régime d'imposition du dossier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Regime {
    /// Réel normal : TVA, IS/IBA, AIB, DSF complète (système normal SYSCOHADA).
    ReelNormal,
    /// Réel simplifié : obligations allégées (à préciser par A8 selon le CGI de l'année).
    ReelSimplifie,
    /// Taxe professionnelle synthétique : petites entreprises, pas de TVA facturée.
    Tps,
}

impl Regime {
    /// Tous les régimes, dans l'ordre d'affichage.
    pub const TOUS: [Regime; 3] = [Regime::ReelNormal, Regime::ReelSimplifie, Regime::Tps];

    /// Vrai si l'entreprise facture et déclare la TVA.
    pub fn collecte_tva(self) -> bool {
        !matches!(self, Regime::Tps)
    }

    /// Libellé affiché.
    pub fn libelle(self) -> &'static str {
        match self {
            Regime::ReelNormal => "Réel normal",
            Regime::ReelSimplifie => "Réel simplifié",
            Regime::Tps => "Taxe professionnelle synthétique (TPS)",
        }
    }
}

/// Forme juridique (Acte uniforme OHADA sur les sociétés commerciales et le GIE).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FormeJuridique {
    /// Entreprise individuelle (y compris entreprenant).
    Ei,
    /// Société à responsabilité limitée.
    Sarl,
    /// SARL unipersonnelle.
    Sarlu,
    /// Société anonyme.
    Sa,
    /// Société par actions simplifiée.
    Sas,
    /// SAS unipersonnelle.
    Sasu,
    /// Société en nom collectif.
    Snc,
    /// Société en commandite simple.
    Scs,
    /// Groupement d'intérêt économique.
    Gie,
    /// Société coopérative.
    Cooperative,
    /// Association ou ONG.
    Association,
    /// Autre forme.
    Autre,
}

impl FormeJuridique {
    /// Toutes les formes, dans l'ordre d'affichage.
    pub const TOUTES: [FormeJuridique; 12] = [
        FormeJuridique::Ei,
        FormeJuridique::Sarl,
        FormeJuridique::Sarlu,
        FormeJuridique::Sa,
        FormeJuridique::Sas,
        FormeJuridique::Sasu,
        FormeJuridique::Snc,
        FormeJuridique::Scs,
        FormeJuridique::Gie,
        FormeJuridique::Cooperative,
        FormeJuridique::Association,
        FormeJuridique::Autre,
    ];

    /// Libellé affiché.
    pub fn libelle(self) -> &'static str {
        match self {
            FormeJuridique::Ei => "Entreprise individuelle",
            FormeJuridique::Sarl => "SARL",
            FormeJuridique::Sarlu => "SARL unipersonnelle",
            FormeJuridique::Sa => "Société anonyme (SA)",
            FormeJuridique::Sas => "SAS",
            FormeJuridique::Sasu => "SAS unipersonnelle",
            FormeJuridique::Snc => "Société en nom collectif (SNC)",
            FormeJuridique::Scs => "Société en commandite simple (SCS)",
            FormeJuridique::Gie => "Groupement d'intérêt économique (GIE)",
            FormeJuridique::Cooperative => "Société coopérative",
            FormeJuridique::Association => "Association / ONG",
            FormeJuridique::Autre => "Autre",
        }
    }

    /// Vrai pour les sociétés de capitaux (SARL, SA, SAS et leurs variantes unipersonnelles).
    pub fn est_societe_de_capitaux(self) -> bool {
        matches!(
            self,
            FormeJuridique::Sarl
                | FormeJuridique::Sarlu
                | FormeJuridique::Sa
                | FormeJuridique::Sas
                | FormeJuridique::Sasu
        )
    }
}

/// Les 12 départements du Bénin.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[allow(missing_docs)] // les noms des départements se suffisent à eux-mêmes
pub enum Departement {
    Alibori,
    Atacora,
    Atlantique,
    Borgou,
    Collines,
    Couffo,
    Donga,
    Littoral,
    Mono,
    Oueme,
    Plateau,
    Zou,
}

impl Departement {
    /// Tous les départements, par ordre alphabétique.
    pub const TOUS: [Departement; 12] = [
        Departement::Alibori,
        Departement::Atacora,
        Departement::Atlantique,
        Departement::Borgou,
        Departement::Collines,
        Departement::Couffo,
        Departement::Donga,
        Departement::Littoral,
        Departement::Mono,
        Departement::Oueme,
        Departement::Plateau,
        Departement::Zou,
    ];

    /// Libellé affiché (avec accents).
    pub fn libelle(self) -> &'static str {
        match self {
            Departement::Alibori => "Alibori",
            Departement::Atacora => "Atacora",
            Departement::Atlantique => "Atlantique",
            Departement::Borgou => "Borgou",
            Departement::Collines => "Collines",
            Departement::Couffo => "Couffo",
            Departement::Donga => "Donga",
            Departement::Littoral => "Littoral",
            Departement::Mono => "Mono",
            Departement::Oueme => "Ouémé",
            Departement::Plateau => "Plateau",
            Departement::Zou => "Zou",
        }
    }
}

/// Secteur d'activité principal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SecteurActivite {
    /// Commerce (achat-revente).
    Commerce,
    /// Prestations de services.
    Services,
    /// Bâtiment et travaux publics.
    Btp,
    /// Industrie, transformation, artisanat de production.
    Industrie,
    /// Agriculture, élevage, pêche.
    Agriculture,
    /// Transport et logistique.
    Transport,
    /// Enseignement et formation.
    Enseignement,
    /// Santé.
    Sante,
    /// Hôtellerie et restauration.
    HotellerieRestauration,
    /// Autre secteur.
    Autre,
}

impl SecteurActivite {
    /// Tous les secteurs, dans l'ordre d'affichage.
    pub const TOUS: [SecteurActivite; 10] = [
        SecteurActivite::Commerce,
        SecteurActivite::Services,
        SecteurActivite::Btp,
        SecteurActivite::Industrie,
        SecteurActivite::Agriculture,
        SecteurActivite::Transport,
        SecteurActivite::Enseignement,
        SecteurActivite::Sante,
        SecteurActivite::HotellerieRestauration,
        SecteurActivite::Autre,
    ];

    /// Libellé affiché.
    pub fn libelle(self) -> &'static str {
        match self {
            SecteurActivite::Commerce => "Commerce",
            SecteurActivite::Services => "Services",
            SecteurActivite::Btp => "Bâtiment et travaux publics",
            SecteurActivite::Industrie => "Industrie et transformation",
            SecteurActivite::Agriculture => "Agriculture, élevage, pêche",
            SecteurActivite::Transport => "Transport et logistique",
            SecteurActivite::Enseignement => "Enseignement et formation",
            SecteurActivite::Sante => "Santé",
            SecteurActivite::HotellerieRestauration => "Hôtellerie et restauration",
            SecteurActivite::Autre => "Autre",
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Identifiants : RCCM et téléphone
// ---------------------------------------------------------------------------------------------

/// Numéro du Registre du commerce et du crédit mobilier (RCCM), normalisé en majuscules.
///
/// Format attendu (à confirmer) : `RB/<GREFFE>/<AA> <A|B> <numéro>`, par exemple `RB/COT/23 B 12345`.
/// Un numéro au format non reconnu est **accepté** ; [`Rccm::format_reconnu`] permet d'avertir.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Rccm(String);

impl Rccm {
    /// Longueur maximale acceptée.
    pub const LONGUEUR_MAX: usize = 40;

    /// Normalise (majuscules, espaces simples, sans espace autour des `/`) et contrôle le minimum :
    /// non vide, au plus [`Rccm::LONGUEUR_MAX`] caractères, uniquement lettres, chiffres, espaces, `/` et `-`.
    pub fn new(texte: &str) -> Resultat<Self> {
        let mots: Vec<&str> = texte.split_whitespace().collect();
        let normalise = mots
            .join(" ")
            .to_uppercase()
            .replace(" /", "/")
            .replace("/ ", "/");
        let caracteres_valides = normalise
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, ' ' | '/' | '-'));
        if normalise.is_empty()
            || normalise.chars().count() > Self::LONGUEUR_MAX
            || !caracteres_valides
        {
            return Err(ErreurNoyau::RccmInvalide(texte.to_owned()));
        }
        Ok(Rccm(normalise))
    }

    /// Vrai si le numéro suit le format `RB/<GREFFE>/<AA> <A|B> <numéro>`
    /// (greffe : 2 à 5 lettres ; année : 2 ou 4 chiffres ; numéro : 1 à 10 chiffres).
    pub fn format_reconnu(&self) -> bool {
        let mut parties = self.0.split('/');
        let (Some(pays), Some(greffe), Some(reste), None) = (
            parties.next(),
            parties.next(),
            parties.next(),
            parties.next(),
        ) else {
            return false;
        };
        let greffe_ok =
            (2..=5).contains(&greffe.len()) && greffe.chars().all(|c| c.is_ascii_uppercase());
        let mut mots = reste.split(' ');
        let (Some(annee), Some(lettre), Some(numero), None) =
            (mots.next(), mots.next(), mots.next(), mots.next())
        else {
            return false;
        };
        let annee_ok = matches!(annee.len(), 2 | 4) && annee.chars().all(|c| c.is_ascii_digit());
        let numero_ok =
            (1..=10).contains(&numero.len()) && numero.chars().all(|c| c.is_ascii_digit());
        pays == "RB" && greffe_ok && annee_ok && matches!(lettre, "A" | "B") && numero_ok
    }

    /// Valeur normalisée.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for Rccm {
    type Error = ErreurNoyau;
    fn try_from(s: String) -> Resultat<Self> {
        Rccm::new(&s)
    }
}

impl From<Rccm> for String {
    fn from(r: Rccm) -> String {
        r.0
    }
}

impl fmt::Display for Rccm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Numéro de téléphone béninois, stocké sous la forme **`+229` suivi des chiffres nationaux**
/// (par exemple `+2290197123456`).
///
/// - Format actuel : **10 chiffres commençant par `01`** (`01 97 12 34 56`).
/// - Ancien format à 8 chiffres : accepté, mais [`Telephone::est_ancien_format`] permet d'avertir.
/// - Préfixes acceptés à la saisie : `+229`, `00229`. Espaces, points, tirets et parenthèses ignorés.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Telephone(String);

impl Telephone {
    /// Indicatif international du Bénin.
    pub const INDICATIF: &'static str = "+229";

    /// Valide et normalise un numéro saisi.
    pub fn new(texte: &str) -> Resultat<Self> {
        let erreur = || ErreurNoyau::TelephoneInvalide(texte.to_owned());
        let compact: String = texte
            .chars()
            .filter(|c| !matches!(c, ' ' | '.' | '-' | '(' | ')'))
            .collect();
        let national = if let Some(r) = compact.strip_prefix("+229") {
            r
        } else if let Some(r) = compact.strip_prefix("00229") {
            r
        } else {
            compact.as_str()
        };
        if !national.chars().all(|c| c.is_ascii_digit()) {
            return Err(erreur());
        }
        let valide = (national.len() == 10 && national.starts_with("01")) || national.len() == 8;
        if !valide {
            return Err(erreur());
        }
        Ok(Telephone(format!("{}{national}", Self::INDICATIF)))
    }

    /// Vrai pour un ancien numéro à 8 chiffres (antérieur au passage à 10 chiffres).
    pub fn est_ancien_format(&self) -> bool {
        self.chiffres_nationaux().len() == 8
    }

    /// Chiffres nationaux, sans l'indicatif.
    pub fn chiffres_nationaux(&self) -> &str {
        self.0.strip_prefix(Self::INDICATIF).unwrap_or(&self.0)
    }

    /// Forme d'affichage : `+229 01 97 12 34 56`.
    pub fn affichage(&self) -> String {
        let chiffres: Vec<char> = self.chiffres_nationaux().chars().collect();
        let groupes: Vec<String> = chiffres
            .chunks(2)
            .map(|g| g.iter().collect::<String>())
            .collect();
        format!("{} {}", Self::INDICATIF, groupes.join(" "))
    }

    /// Valeur stockée (`+229…`).
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for Telephone {
    type Error = ErreurNoyau;
    fn try_from(s: String) -> Resultat<Self> {
        Telephone::new(&s)
    }
}

impl From<Telephone> for String {
    fn from(t: Telephone) -> String {
        t.0
    }
}

impl fmt::Display for Telephone {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.affichage())
    }
}

/// Contrôle simple d'une adresse électronique : une seule `@`, partie locale non vide,
/// domaine contenant un point (ni au début ni à la fin), aucun espace.
pub fn courriel_valide(texte: &str) -> bool {
    let texte = texte.trim();
    if texte.chars().any(char::is_whitespace) || texte.len() > 254 {
        return false;
    }
    let mut parties = texte.split('@');
    let (Some(local), Some(domaine), None) = (parties.next(), parties.next(), parties.next())
    else {
        return false;
    };
    !local.is_empty()
        && domaine.contains('.')
        && !domaine.starts_with('.')
        && !domaine.ends_with('.')
        && !domaine.contains("..")
}

// ---------------------------------------------------------------------------------------------
// La fiche
// ---------------------------------------------------------------------------------------------

/// Personne responsable du compte sur la plateforme (son courriel sert d'identifiant de connexion).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Responsable {
    /// Nom de famille ★.
    pub nom: String,
    /// Prénoms ★.
    pub prenoms: String,
    /// Fonction dans l'entreprise ★ (gérant, directeur financier…).
    pub fonction: String,
    /// Téléphone ★.
    pub telephone: String,
    /// Courriel ★ (identifiant de connexion).
    pub courriel: String,
}

/// Fiche légale de l'entreprise, telle que saisie (★ = obligatoire). Voir ADR 0009.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FicheEntreprise {
    /// Raison sociale ★.
    pub raison_sociale: String,
    /// Nom commercial.
    pub nom_commercial: Option<String>,
    /// Sigle.
    pub sigle: Option<String>,
    /// Forme juridique ★.
    pub forme_juridique: Option<FormeJuridique>,
    /// Date de création ★ (ne peut pas être dans le futur).
    pub date_creation: Option<Date>,
    /// IFU ★ (13 chiffres).
    pub ifu: String,
    /// Numéro RCCM ★.
    pub rccm: String,
    /// Numéro employeur CNSS (★ si l'effectif salarié est supérieur à zéro).
    pub numero_cnss: Option<String>,
    /// Régime fiscal ★.
    pub regime: Option<Regime>,
    /// Centre des impôts de rattachement ★.
    pub centre_impots: String,
    /// Secteur d'activité ★.
    pub secteur: Option<SecteurActivite>,
    /// Activité principale ★ (texte court).
    pub activite_principale: String,
    /// Effectif salarié.
    pub effectif: Option<u32>,
    /// Département ★.
    pub departement: Option<Departement>,
    /// Commune ★.
    pub commune: String,
    /// Quartier, rue, lot ★.
    pub adresse: String,
    /// Boîte postale.
    pub boite_postale: Option<String>,
    /// Téléphone de l'entreprise ★.
    pub telephone: String,
    /// Courriel de l'entreprise ★.
    pub courriel: String,
    /// Site web.
    pub site_web: Option<String>,
    /// Responsable du compte ★.
    pub responsable: Responsable,
}

/// Un problème détecté sur un champ.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProblemeChamp {
    /// Nom du champ (nom Rust, identique côté TypeScript ; `responsable.courriel` pour un sous-champ).
    pub champ: String,
    /// Message en français, compréhensible par un non-comptable, qui dit quoi faire.
    pub message: String,
}

/// Résultat de [`FicheEntreprise::valider`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RapportValidation {
    /// Problèmes bloquants : la fiche ne peut pas être enregistrée.
    pub erreurs: Vec<ProblemeChamp>,
    /// Points à vérifier : la fiche peut être enregistrée.
    pub avertissements: Vec<ProblemeChamp>,
}

impl RapportValidation {
    /// Vrai si aucune erreur bloquante.
    pub fn est_valide(&self) -> bool {
        self.erreurs.is_empty()
    }

    fn erreur(&mut self, champ: &str, message: &str) {
        self.erreurs.push(ProblemeChamp {
            champ: champ.to_owned(),
            message: message.to_owned(),
        });
    }

    fn avertissement(&mut self, champ: &str, message: &str) {
        self.avertissements.push(ProblemeChamp {
            champ: champ.to_owned(),
            message: message.to_owned(),
        });
    }

    fn exiger_texte(&mut self, champ: &str, valeur: &str, libelle: &str) -> bool {
        if valeur.trim().is_empty() {
            self.erreur(champ, &format!("Indiquez {libelle}."));
            false
        } else {
            true
        }
    }

    fn exiger_choix<T>(&mut self, champ: &str, valeur: Option<T>, libelle: &str) {
        if valeur.is_none() {
            self.erreur(champ, &format!("Choisissez {libelle}."));
        }
    }

    fn controler_telephone(&mut self, champ: &str, valeur: &str) {
        if !self.exiger_texte(champ, valeur, "un numéro de téléphone") {
            return;
        }
        match Telephone::new(valeur) {
            Ok(t) if t.est_ancien_format() => self.avertissement(
                champ,
                "Ce numéro a 8 chiffres (ancien format). Les numéros béninois ont maintenant 10 chiffres et commencent par 01.",
            ),
            Ok(_) => {}
            Err(_) => self.erreur(
                champ,
                "Numéro de téléphone invalide. Exemple attendu : 01 97 12 34 56 (10 chiffres).",
            ),
        }
    }

    fn controler_courriel(&mut self, champ: &str, valeur: &str) {
        if !self.exiger_texte(champ, valeur, "une adresse électronique") {
            return;
        }
        if !courriel_valide(valeur) {
            self.erreur(
                champ,
                "Adresse électronique invalide. Exemple attendu : contact@entreprise.bj.",
            );
        }
    }
}

impl FicheEntreprise {
    /// Contrôle toute la fiche et renvoie **tous** les problèmes.
    ///
    /// `aujourdhui` sert à refuser une date de création future (passée en paramètre pour rester une
    /// fonction pure et testable). L'unicité de l'IFU sur la plateforme est contrôlée par la plateforme.
    pub fn valider(&self, aujourdhui: Date) -> RapportValidation {
        let mut r = RapportValidation::default();

        // Identité légale
        r.exiger_texte("raison_sociale", &self.raison_sociale, "la raison sociale");
        r.exiger_choix(
            "forme_juridique",
            self.forme_juridique,
            "la forme juridique",
        );
        match self.date_creation {
            None => r.erreur(
                "date_creation",
                "Indiquez la date de création de l'entreprise.",
            ),
            Some(d) if d > aujourdhui => r.erreur(
                "date_creation",
                "La date de création ne peut pas être dans le futur.",
            ),
            Some(_) => {}
        }

        // Identifiants
        if r.exiger_texte("ifu", &self.ifu, "l'IFU (identifiant fiscal unique)")
            && Ifu::new(&self.ifu).is_err()
        {
            r.erreur(
                "ifu",
                "L'IFU doit compter exactement 13 chiffres. Il figure sur votre attestation d'immatriculation.",
            );
        }
        if r.exiger_texte("rccm", &self.rccm, "le numéro RCCM (registre du commerce)") {
            match Rccm::new(&self.rccm) {
                Err(_) => r.erreur(
                    "rccm",
                    "Numéro RCCM invalide : utilisez uniquement lettres, chiffres, espaces, « / » et « - » (40 caractères au plus).",
                ),
                Ok(rccm) if !rccm.format_reconnu() => r.avertissement(
                    "rccm",
                    "Format RCCM inhabituel. Vérifiez votre extrait : la forme courante est RB/COT/23 B 12345.",
                ),
                Ok(_) => {}
            }
        }
        let salaries = self.effectif.unwrap_or(0) > 0;
        let cnss_vide = self
            .numero_cnss
            .as_deref()
            .is_none_or(|n| n.trim().is_empty());
        if salaries && cnss_vide {
            r.erreur(
                "numero_cnss",
                "Vous déclarez des salariés : indiquez votre numéro d'employeur CNSS.",
            );
        }

        // Fiscalité
        r.exiger_choix("regime", self.regime, "le régime fiscal");
        r.exiger_texte(
            "centre_impots",
            &self.centre_impots,
            "votre centre des impôts de rattachement",
        );
        match (self.forme_juridique, self.regime) {
            (Some(FormeJuridique::Ei), Some(Regime::ReelNormal)) => r.avertissement(
                "regime",
                "Une entreprise individuelle au réel normal est rare : vérifiez le régime indiqué sur votre avis d'imposition.",
            ),
            (Some(f), Some(Regime::Tps)) if f.est_societe_de_capitaux() => r.avertissement(
                "regime",
                "Une société de capitaux à la TPS est inhabituelle : vérifiez le régime indiqué sur votre avis d'imposition.",
            ),
            _ => {}
        }

        // Activité
        r.exiger_choix("secteur", self.secteur, "le secteur d'activité");
        r.exiger_texte(
            "activite_principale",
            &self.activite_principale,
            "l'activité principale",
        );

        // Adresse
        r.exiger_choix("departement", self.departement, "le département");
        r.exiger_texte("commune", &self.commune, "la commune");
        r.exiger_texte("adresse", &self.adresse, "le quartier ou la rue");

        // Contact entreprise
        r.controler_telephone("telephone", &self.telephone);
        r.controler_courriel("courriel", &self.courriel);

        // Responsable du compte
        let resp = &self.responsable;
        r.exiger_texte("responsable.nom", &resp.nom, "le nom du responsable");
        r.exiger_texte(
            "responsable.prenoms",
            &resp.prenoms,
            "les prénoms du responsable",
        );
        r.exiger_texte(
            "responsable.fonction",
            &resp.fonction,
            "la fonction du responsable",
        );
        r.controler_telephone("responsable.telephone", &resp.telephone);
        r.controler_courriel("responsable.courriel", &resp.courriel);

        r
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date(a: i32, m: u32, j: u32) -> Date {
        Date::from_ymd_opt(a, m, j).unwrap()
    }

    fn fiche_complete() -> FicheEntreprise {
        FicheEntreprise {
            raison_sociale: "Boutique Exemple SARL".into(),
            nom_commercial: Some("Boutique Exemple".into()),
            sigle: None,
            forme_juridique: Some(FormeJuridique::Sarl),
            date_creation: Some(date(2021, 3, 15)),
            ifu: "3201912345678".into(),
            rccm: "RB/COT/21 B 12345".into(),
            numero_cnss: None,
            regime: Some(Regime::ReelSimplifie),
            centre_impots: "CIME Cotonou".into(),
            secteur: Some(SecteurActivite::Commerce),
            activite_principale: "Vente de fournitures de bureau".into(),
            effectif: Some(0),
            departement: Some(Departement::Littoral),
            commune: "Cotonou".into(),
            adresse: "Quartier Ganhi, rue 123".into(),
            boite_postale: None,
            telephone: "01 97 12 34 56".into(),
            courriel: "contact@exemple.bj".into(),
            site_web: None,
            responsable: Responsable {
                nom: "AHOUANSOU".into(),
                prenoms: "Marie".into(),
                fonction: "Gérante".into(),
                telephone: "+229 01 66 00 11 22".into(),
                courriel: "marie@exemple.bj".into(),
            },
        }
    }

    fn champs(p: &[ProblemeChamp]) -> Vec<&str> {
        p.iter().map(|x| x.champ.as_str()).collect()
    }

    // --- Téléphone ---

    #[test]
    fn telephone_dix_chiffres_normalise() {
        let t = Telephone::new("01 97 12 34 56").unwrap();
        assert_eq!(t.as_str(), "+2290197123456");
        assert_eq!(t.affichage(), "+229 01 97 12 34 56");
        assert!(!t.est_ancien_format());
        assert_eq!(Telephone::new("+229 01.97.12.34.56").unwrap(), t);
        assert_eq!(Telephone::new("00229-0197123456").unwrap(), t);
    }

    #[test]
    fn telephone_ancien_format_accepte_avec_drapeau() {
        let t = Telephone::new("97 12 34 56").unwrap();
        assert!(t.est_ancien_format());
        assert_eq!(t.affichage(), "+229 97 12 34 56");
    }

    #[test]
    fn telephone_refuse() {
        assert!(Telephone::new("").is_err());
        assert!(Telephone::new("0297123456").is_err()); // 10 chiffres sans 01
        assert!(Telephone::new("019712345").is_err()); // 9 chiffres
        assert!(Telephone::new("01971234567").is_err()); // 11 chiffres
        assert!(Telephone::new("01 97 AB 34 56").is_err());
        assert!(Telephone::new("+33 6 12 34 56 78").is_err());
    }

    // --- RCCM ---

    #[test]
    fn rccm_normalise_et_reconnu() {
        let r = Rccm::new("  rb / cot /23   b 12345 ").unwrap();
        assert_eq!(r.as_str(), "RB/COT/23 B 12345");
        assert!(r.format_reconnu());
        assert!(Rccm::new("RB/PNO/2019 A 7").unwrap().format_reconnu());
    }

    #[test]
    fn rccm_non_reconnu_mais_accepte() {
        for texte in [
            "RB-COT-23-B-12345",
            "RB/COT/23 C 12345",
            "CI/ABJ/23 B 12345",
            "RB/COT/123 B 1",
        ] {
            let r = Rccm::new(texte).unwrap();
            assert!(!r.format_reconnu(), "{texte}");
        }
    }

    #[test]
    fn rccm_refuse() {
        assert!(Rccm::new("   ").is_err());
        assert!(Rccm::new("RB/COT/23 B 12345 ; DROP").is_err());
        assert!(Rccm::new(&"A".repeat(41)).is_err());
        assert!(Rccm::new(&"A".repeat(40)).is_ok()); // limite exacte
    }

    // --- Courriel ---

    #[test]
    fn courriels() {
        assert!(courriel_valide("contact@exemple.bj"));
        assert!(courriel_valide(" a.b+c@sous.domaine.bj "));
        assert!(!courriel_valide("contact@exemple"));
        assert!(!courriel_valide("@exemple.bj"));
        assert!(!courriel_valide("a@@exemple.bj"));
        assert!(!courriel_valide("a b@exemple.bj"));
        assert!(!courriel_valide("a@.exemple.bj"));
        assert!(!courriel_valide("a@exemple..bj"));
    }

    // --- Fiche ---

    #[test]
    fn fiche_complete_sans_probleme() {
        let r = fiche_complete().valider(date(2026, 11, 27));
        assert!(r.est_valide(), "{:?}", r.erreurs);
        assert!(r.avertissements.is_empty(), "{:?}", r.avertissements);
    }

    #[test]
    fn fiche_vide_liste_tous_les_champs_obligatoires() {
        let r = FicheEntreprise::default().valider(date(2026, 11, 27));
        assert_eq!(
            champs(&r.erreurs),
            vec![
                "raison_sociale",
                "forme_juridique",
                "date_creation",
                "ifu",
                "rccm",
                "regime",
                "centre_impots",
                "secteur",
                "activite_principale",
                "departement",
                "commune",
                "adresse",
                "telephone",
                "courriel",
                "responsable.nom",
                "responsable.prenoms",
                "responsable.fonction",
                "responsable.telephone",
                "responsable.courriel",
            ]
        );
        assert!(r.avertissements.is_empty());
    }

    #[test]
    fn date_de_creation_limite() {
        let mut f = fiche_complete();
        f.date_creation = Some(date(2026, 11, 27));
        assert!(f.valider(date(2026, 11, 27)).est_valide()); // aujourd'hui : accepté
        f.date_creation = Some(date(2026, 11, 28));
        let r = f.valider(date(2026, 11, 27));
        assert_eq!(champs(&r.erreurs), vec!["date_creation"]);
    }

    #[test]
    fn ifu_mal_forme_bloque() {
        let mut f = fiche_complete();
        f.ifu = "320191234567".into(); // 12 chiffres
        assert_eq!(champs(&f.valider(date(2026, 11, 27)).erreurs), vec!["ifu"]);
    }

    #[test]
    fn cnss_obligatoire_des_un_salarie() {
        let mut f = fiche_complete();
        f.effectif = Some(1);
        assert_eq!(
            champs(&f.valider(date(2026, 11, 27)).erreurs),
            vec!["numero_cnss"]
        );
        f.numero_cnss = Some("  ".into());
        assert!(!f.valider(date(2026, 11, 27)).est_valide());
        f.numero_cnss = Some("1234567".into());
        assert!(f.valider(date(2026, 11, 27)).est_valide());
        f.effectif = Some(0);
        f.numero_cnss = None;
        assert!(f.valider(date(2026, 11, 27)).est_valide());
    }

    #[test]
    fn avertissements_non_bloquants() {
        let mut f = fiche_complete();
        f.rccm = "RB-COT-21-B-12345".into();
        f.telephone = "97 12 34 56".into();
        f.regime = Some(Regime::Tps); // SARL à la TPS
        let r = f.valider(date(2026, 11, 27));
        assert!(r.est_valide());
        assert_eq!(
            champs(&r.avertissements),
            vec!["rccm", "regime", "telephone"]
        );

        let mut ei = fiche_complete();
        ei.forme_juridique = Some(FormeJuridique::Ei);
        ei.regime = Some(Regime::ReelNormal);
        assert_eq!(
            champs(&ei.valider(date(2026, 11, 27)).avertissements),
            vec!["regime"]
        );
    }

    #[test]
    fn courriel_et_telephone_invalides_du_responsable() {
        let mut f = fiche_complete();
        f.responsable.courriel = "marie.exemple.bj".into();
        f.responsable.telephone = "12345".into();
        assert_eq!(
            champs(&f.valider(date(2026, 11, 27)).erreurs),
            vec!["responsable.telephone", "responsable.courriel"]
        );
    }

    #[test]
    fn serialisation_json_snake_case() {
        let json = serde_json::to_value(fiche_complete()).unwrap();
        assert_eq!(json["forme_juridique"], "SARL");
        assert_eq!(json["regime"], "REEL_SIMPLIFIE");
        assert_eq!(json["departement"], "LITTORAL");
        assert_eq!(json["secteur"], "COMMERCE");
        assert_eq!(json["date_creation"], "2021-03-15");
        assert_eq!(json["responsable"]["prenoms"], "Marie");
        let relu: FicheEntreprise = serde_json::from_value(json).unwrap();
        assert_eq!(relu, fiche_complete());
    }

    #[test]
    fn listes_completes() {
        assert_eq!(Departement::TOUS.len(), 12);
        assert_eq!(Departement::Oueme.libelle(), "Ouémé");
        assert!(FormeJuridique::Sasu.est_societe_de_capitaux());
        assert!(!FormeJuridique::Snc.est_societe_de_capitaux());
        assert!(Regime::ReelNormal.collecte_tva());
        assert!(!Regime::Tps.collecte_tva());
        assert!(SecteurActivite::TOUS
            .iter()
            .all(|s| !s.libelle().is_empty()));
    }
}
