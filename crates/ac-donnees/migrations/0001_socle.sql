-- Socle du dossier d'entreprise. Propriétaire : A2. Ne jamais modifier une migration publiée.

CREATE TABLE donnees_entreprise (
    id                  INTEGER PRIMARY KEY CHECK (id = 1),      -- un seul dossier par base
    raison_sociale      TEXT NOT NULL,
    ifu                 TEXT NOT NULL CHECK (length(ifu) = 13),
    rccm                TEXT,
    forme_juridique     TEXT NOT NULL,                           -- EI, SARL, SA, SAS, SNC…
    regime_fiscal       TEXT NOT NULL CHECK (regime_fiscal IN ('REEL_NORMAL','REEL_SIMPLIFIE','TPS')),
    systeme_comptable   TEXT NOT NULL CHECK (systeme_comptable IN ('NORMAL','SMT')),
    centre_impots       TEXT,
    adresse             TEXT,
    telephone           TEXT,
    courriel            TEXT,
    debut_exercice      TEXT NOT NULL,                           -- AAAA-MM-JJ
    cree_le             TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
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
