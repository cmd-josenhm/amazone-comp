-- Socle du dossier d'entreprise. Propriétaire : A2. Ne jamais modifier une migration publiée.

CREATE TABLE donnees_entreprise (
    id                  INTEGER PRIMARY KEY CHECK (id = 1),      -- un seul dossier par base
    -- Fiche légale (ac_noyau::entreprise::FicheEntreprise, ADR 0009) ; validée par FicheEntreprise::valider
    raison_sociale      TEXT NOT NULL CHECK (length(trim(raison_sociale)) > 0),
    nom_commercial      TEXT,
    sigle               TEXT,
    forme_juridique     TEXT NOT NULL CHECK (forme_juridique IN ('EI','SARL','SARLU','SA','SAS','SASU','SNC','SCS','GIE','COOPERATIVE','ASSOCIATION','AUTRE')),
    date_creation       TEXT NOT NULL,                           -- AAAA-MM-JJ
    ifu                 TEXT NOT NULL CHECK (length(ifu) = 13 AND ifu NOT GLOB '*[^0-9]*'),
    rccm                TEXT NOT NULL,                           -- normalisé (Rccm::new)
    numero_cnss         TEXT,
    regime_fiscal       TEXT NOT NULL CHECK (regime_fiscal IN ('REEL_NORMAL','REEL_SIMPLIFIE','TPS')),
    centre_impots       TEXT NOT NULL,
    secteur             TEXT NOT NULL CHECK (secteur IN ('COMMERCE','SERVICES','BTP','INDUSTRIE','AGRICULTURE','TRANSPORT','ENSEIGNEMENT','SANTE','HOTELLERIE_RESTAURATION','AUTRE')),
    activite_principale TEXT NOT NULL,
    effectif            INTEGER CHECK (effectif IS NULL OR effectif >= 0),
    departement         TEXT NOT NULL CHECK (departement IN ('ALIBORI','ATACORA','ATLANTIQUE','BORGOU','COLLINES','COUFFO','DONGA','LITTORAL','MONO','OUEME','PLATEAU','ZOU')),
    commune             TEXT NOT NULL,
    adresse             TEXT NOT NULL,                           -- quartier, rue, lot
    boite_postale       TEXT,
    telephone           TEXT NOT NULL,                           -- normalisé +229… (Telephone::new)
    courriel            TEXT NOT NULL,
    site_web            TEXT,
    responsable_nom       TEXT NOT NULL,
    responsable_prenoms   TEXT NOT NULL,
    responsable_fonction  TEXT NOT NULL,
    responsable_telephone TEXT NOT NULL,
    responsable_courriel  TEXT NOT NULL,
    -- Paramètres comptables du dossier
    systeme_comptable   TEXT NOT NULL CHECK (systeme_comptable IN ('NORMAL','SMT')),
    debut_exercice      TEXT NOT NULL,                           -- AAAA-MM-JJ
    -- Lien avec la plateforme (ADR 0009 §4) : identifiant distant et dernier statut connu
    plateforme_id       TEXT,
    statut_plateforme   TEXT CHECK (statut_plateforme IS NULL OR statut_plateforme IN ('EN_ATTENTE_VERIFICATION','VERIFIEE','REJETEE','SUSPENDUE')),
    cree_le             TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    modifie_le          TEXT
);

CREATE TABLE donnees_utilisateur (
    id                  INTEGER PRIMARY KEY,
    identifiant         TEXT NOT NULL UNIQUE,
    nom_complet         TEXT NOT NULL,
    role                TEXT NOT NULL CHECK (role IN ('ADMINISTRATEUR','COMPTABLE','SAISIE','PAIE','LECTURE')),
    empreinte_mdp       TEXT NOT NULL,                           -- Argon2id (format PHC)
    cle_enveloppee      BLOB,                                    -- clé de la base chiffrée par la clé dérivée du mot de passe
    actif               INTEGER NOT NULL DEFAULT 1,
    cree_le             TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
);

CREATE TABLE donnees_audit (
    id                  INTEGER PRIMARY KEY,
    horodatage          TEXT NOT NULL,
    utilisateur_id      INTEGER,
    action              TEXT NOT NULL,                           -- CREER, MODIFIER, VALIDER, CONTREPASSER, SUPPRIMER_BROUILLON, CONNEXION…
    objet_type          TEXT NOT NULL,                           -- ex. compta_ecriture
    objet_id            TEXT NOT NULL,
    details             TEXT NOT NULL,                           -- JSON canonique
    hash_precedent      TEXT NOT NULL,
    hash                TEXT NOT NULL UNIQUE
);

-- Le journal d'audit est en ajout seul.
CREATE TRIGGER donnees_audit_pas_de_modification
BEFORE UPDATE ON donnees_audit
BEGIN
    SELECT RAISE(ABORT, 'journal d''audit non modifiable');
END;

CREATE TRIGGER donnees_audit_pas_de_suppression
BEFORE DELETE ON donnees_audit
BEGIN
    SELECT RAISE(ABORT, 'journal d''audit non supprimable');
END;

CREATE TABLE donnees_parametre (
    cle                 TEXT PRIMARY KEY,
    valeur              TEXT NOT NULL
);
