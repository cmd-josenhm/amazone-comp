-- Comptes de trésorerie (agent A6, plage 4000–4999).
-- La table compta_journal reprend les colonnes prévues par le contrat C3 d'A1. CREATE IF NOT EXISTS
-- permet de cohabiter avec sa migration lorsque celle-ci sera disponible sur la branche principale.

CREATE TABLE tresorerie_operateur (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    nom         TEXT NOT NULL COLLATE NOCASE UNIQUE
                    CHECK (length(trim(nom)) BETWEEN 1 AND 80),
    libelle     TEXT CHECK (libelle IS NULL OR length(trim(libelle)) BETWEEN 1 AND 120),
    actif       INTEGER NOT NULL DEFAULT 1 CHECK (actif IN (0, 1)),
    cree_le     TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
);

-- Les opérateurs sont des données : l'utilisateur peut en ajouter sans modifier le code.
INSERT INTO tresorerie_operateur (nom, libelle) VALUES
    ('MTN MoMo', 'MTN Mobile Money'),
    ('Moov Money', 'Moov Money'),
    ('Celtiis Cash', 'Celtiis Cash');

CREATE TABLE tresorerie_configuration (
    id                  INTEGER PRIMARY KEY CHECK (id = 1),
    mobile_money_actif  INTEGER NOT NULL DEFAULT 1 CHECK (mobile_money_actif IN (0, 1))
);

INSERT INTO tresorerie_configuration (id, mobile_money_actif) VALUES (1, 1);

-- Compatibilité avec le contrat C3 : A1 prévoit la table et la fonction journal::creer.
-- La branche principale ne les fournit pas encore ; cette définition minimale permet au Lot 1
-- d'enregistrer les journaux BQx, MMx et CA sans dupliquer la table si A1 l'a déjà créée.
CREATE TABLE IF NOT EXISTS compta_journal (
    code                TEXT PRIMARY KEY,
    libelle             TEXT NOT NULL CHECK (length(trim(libelle)) > 0),
    type                TEXT NOT NULL
                            CHECK (type IN ('VENTES','ACHATS','TRESORERIE','PAIE','OD','AN','STOCKS')),
    compte_tresorerie   TEXT REFERENCES compta_compte(numero)
);

-- Journal requis pour l'écriture d'ouverture ; A1 pourra aussi le créer dans sa migration.
INSERT OR IGNORE INTO compta_journal (code, libelle, type)
VALUES ('AN', 'À-nouveaux', 'AN');

CREATE TABLE tresorerie_compte (
    id               INTEGER PRIMARY KEY AUTOINCREMENT,
    type_compte      TEXT NOT NULL CHECK (type_compte IN ('BANQUE','CAISSE','MOBILE_MONEY')),
    libelle          TEXT NOT NULL CHECK (length(trim(libelle)) BETWEEN 1 AND 200),
    banque_operateur TEXT,
    numero           TEXT,
    compte_comptable TEXT NOT NULL UNIQUE REFERENCES compta_compte(numero),
    journal          TEXT NOT NULL REFERENCES compta_journal(code),
    solde_ouverture  INTEGER NOT NULL DEFAULT 0
                         CHECK (solde_ouverture BETWEEN -1000000000000000 AND 1000000000000000),
    actif            INTEGER NOT NULL DEFAULT 1 CHECK (actif IN (0, 1)),
    date_ouverture   TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    CHECK (
        (type_compte = 'CAISSE' AND (numero IS NULL OR length(trim(numero)) > 0))
        OR
        (type_compte IN ('BANQUE','MOBILE_MONEY') AND numero IS NOT NULL AND length(trim(numero)) > 0)
    ),
    CHECK (
        type_compte NOT IN ('BANQUE','MOBILE_MONEY')
        OR (banque_operateur IS NOT NULL AND length(trim(banque_operateur)) > 0)
    ),
    CHECK (type_compte != 'CAISSE' OR solde_ouverture >= 0)
);

CREATE UNIQUE INDEX tresorerie_compte_identite_unique
    ON tresorerie_compte (type_compte, banque_operateur COLLATE NOCASE, numero)
    WHERE numero IS NOT NULL;

CREATE INDEX tresorerie_compte_actif ON tresorerie_compte (actif, libelle);
