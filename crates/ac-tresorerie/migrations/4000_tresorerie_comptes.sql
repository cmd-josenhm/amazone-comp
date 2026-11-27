-- Comptes de trésorerie (agent A6, plage 4000–4999). Propriétaire : A6. Ne jamais modifier une migration publiée.
-- Tableaux métier : tresorerie_compte, tresorerie_operateur ; journaux déclarés : compta_journal.

CREATE TABLE tresorerie_operateur (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    nom         TEXT NOT NULL UNIQUE,                 -- ex. « MTN MoMo »
    libelle     TEXT,
    actif       INTEGER NOT NULL DEFAULT 1,
    cree_le     TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
);

-- Opérateurs Mobile Money pré-remplis (données : le module est désactivable, voir PROMPT_A6).
INSERT INTO tresorerie_operateur (nom, libelle) VALUES
    ('MTN MoMo', 'MTN Mobile Money'),
    ('Moov Money', 'Moov Money'),
    ('Celtiis Cash', 'Celtiis Cash');

CREATE TABLE tresorerie_compte (
    id               INTEGER PRIMARY KEY AUTOINCREMENT,
    type_compte      TEXT NOT NULL CHECK (type_compte IN ('BANQUE','CAISSE','MOBILE_MONEY')),
    libelle          TEXT NOT NULL CHECK (length(trim(libelle)) > 0),
    banque_operateur TEXT,                              -- nom de la banque ou de l'opérateur (table tresorerie_operateur)
    numero           TEXT,                              -- numéro de compte bancaire ou numéro de téléphone
    compte_comptable TEXT NOT NULL UNIQUE,              -- sous-compte de 521 / 55 / 571
    journal          TEXT NOT NULL,                     -- code journal BQn / MMn / CA
    solde_ouverture  INTEGER NOT NULL DEFAULT 0,        -- FCFA, signe négatif = solde créditeur
    actif            INTEGER NOT NULL DEFAULT 1,
    date_ouverture   TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
);

-- Registre des journaux comptables déclarés par les comptes de trésorerie.
-- A1 fournit `journal::creer` dans `ac-compta` (contrat C3) ; A6 fait usage ici pour le Lot 1.
CREATE TABLE compta_journal (
    code    TEXT PRIMARY KEY,
    libelle TEXT NOT NULL
);
