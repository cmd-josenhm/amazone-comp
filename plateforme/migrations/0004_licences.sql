-- Licences signées (ADR 0004) : une ligne par émission / renouvellement / transfert.
-- Le jeton signé lui-même n'est pas stocké : il est recréé à la demande et sa charge
-- utile est reconstituée depuis cette table.

CREATE TABLE licences (
    id BIGSERIAL PRIMARY KEY,
    entreprise_id BIGINT NOT NULL REFERENCES entreprises(id) ON DELETE CASCADE,
    empreinte_poste TEXT NOT NULL,
    emise_le DATE NOT NULL,
    expire_le DATE NOT NULL,
    offre TEXT NOT NULL DEFAULT 'GRATUITE',
    statut TEXT NOT NULL DEFAULT 'ACTIVE'
        CHECK (statut IN ('ACTIVE', 'REMPLACEE', 'REVOQUEE')),
    motif TEXT,
    cree_le TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_licences_entreprise ON licences (entreprise_id, empreinte_poste, statut);

-- Historique des transferts de poste, pour la limite annuelle (2 par an par défaut).
CREATE TABLE transferts_poste (
    id BIGSERIAL PRIMARY KEY,
    entreprise_id BIGINT NOT NULL REFERENCES entreprises(id) ON DELETE CASCADE,
    ancienne_empreinte TEXT NOT NULL,
    nouvelle_empreinte TEXT NOT NULL,
    annee INTEGER NOT NULL,
    cree_le TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_transferts_entreprise_annee ON transferts_poste (entreprise_id, annee);

-- Paramètres de la plateforme, ajustables par l'administrateur (console, lot 7).
CREATE TABLE parametres (
    cle TEXT PRIMARY KEY,
    valeur TEXT NOT NULL
);

INSERT INTO parametres (cle, valeur) VALUES
    ('transferts_poste_par_an', '2'),
    ('verification_prealable', 'false');
