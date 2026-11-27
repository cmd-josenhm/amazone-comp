-- Socle comptable. Propriétaire : A1. Ne jamais modifier une migration publiée.

CREATE TABLE compta_compte (
    numero          TEXT PRIMARY KEY CHECK (numero GLOB '[1-9]*' AND length(numero) BETWEEN 2 AND 12),
    libelle         TEXT NOT NULL,
    mouvementable   INTEGER NOT NULL DEFAULT 1,
    actif           INTEGER NOT NULL DEFAULT 1
);

CREATE TABLE compta_ecriture (
    id                  INTEGER PRIMARY KEY,
    journal             TEXT NOT NULL,
    numero              INTEGER,                       -- numérotation continue par journal/exercice (A1)
    date                TEXT NOT NULL,
    libelle             TEXT NOT NULL CHECK (length(trim(libelle)) > 0),
    piece               TEXT,
    origine             TEXT NOT NULL,
    reference_origine   TEXT,
    contrepasse_id      INTEGER REFERENCES compta_ecriture(id),
    cree_par            INTEGER NOT NULL,
    cree_le             TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
);

CREATE TABLE compta_ligne (
    id              INTEGER PRIMARY KEY,
    ecriture_id     INTEGER NOT NULL REFERENCES compta_ecriture(id),
    rang            INTEGER NOT NULL,
    compte          TEXT NOT NULL REFERENCES compta_compte(numero),
    libelle         TEXT,
    debit           INTEGER NOT NULL DEFAULT 0 CHECK (debit >= 0),
    credit          INTEGER NOT NULL DEFAULT 0 CHECK (credit >= 0),
    tiers           TEXT,
    code_fiscal     TEXT,
    base_fiscale    INTEGER,
    taux            TEXT,
    lettrage        TEXT,                              -- seule colonne modifiable (lettrage), voir déclencheur
    CHECK ((debit = 0) <> (credit = 0)),
    UNIQUE (ecriture_id, rang)
);

CREATE INDEX compta_ligne_compte ON compta_ligne(compte);
CREATE INDEX compta_ligne_code_fiscal ON compta_ligne(code_fiscal) WHERE code_fiscal IS NOT NULL;
CREATE INDEX compta_ecriture_date ON compta_ecriture(date);

-- Règle R3 : une écriture validée est immuable (seul le lettrage d'une ligne peut évoluer).
CREATE TRIGGER compta_ecriture_immuable BEFORE UPDATE ON compta_ecriture
BEGIN
    SELECT RAISE(ABORT, 'écriture validée non modifiable : utiliser la contre-passation');
END;

CREATE TRIGGER compta_ecriture_non_supprimable BEFORE DELETE ON compta_ecriture
BEGIN
    SELECT RAISE(ABORT, 'écriture validée non supprimable : utiliser la contre-passation');
END;

CREATE TRIGGER compta_ligne_immuable BEFORE UPDATE ON compta_ligne
WHEN NEW.ecriture_id IS NOT OLD.ecriture_id OR NEW.rang IS NOT OLD.rang OR NEW.compte IS NOT OLD.compte
  OR NEW.debit IS NOT OLD.debit OR NEW.credit IS NOT OLD.credit OR NEW.tiers IS NOT OLD.tiers
  OR NEW.code_fiscal IS NOT OLD.code_fiscal OR NEW.base_fiscale IS NOT OLD.base_fiscale
  OR NEW.taux IS NOT OLD.taux OR NEW.libelle IS NOT OLD.libelle
BEGIN
    SELECT RAISE(ABORT, 'ligne d''écriture validée non modifiable');
END;

CREATE TRIGGER compta_ligne_non_supprimable BEFORE DELETE ON compta_ligne
BEGIN
    SELECT RAISE(ABORT, 'ligne d''écriture validée non supprimable');
END;
