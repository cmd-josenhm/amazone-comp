-- Inscription des entreprises (ADR 0009, contrat C11). La fiche complète est stockée
-- en JSONB (tous les champs de FicheEntreprise) ; l'IFU normalisé est unique sur toute
-- la plateforme.

CREATE TABLE entreprises (
    id BIGSERIAL PRIMARY KEY,
    compte_id BIGINT NOT NULL REFERENCES comptes(id) ON DELETE CASCADE,
    ifu TEXT NOT NULL UNIQUE,
    fiche JSONB NOT NULL,
    statut TEXT NOT NULL DEFAULT 'EN_ATTENTE_VERIFICATION'
        CHECK (statut IN ('EN_ATTENTE_VERIFICATION', 'VERIFIEE', 'REJETEE', 'SUSPENDUE')),
    motif TEXT,
    statut_change_le TIMESTAMPTZ,
    cree_le TIMESTAMPTZ NOT NULL DEFAULT now(),
    maj_le TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_entreprises_compte ON entreprises (compte_id);

-- Un changement d'IFU est une demande validée par l'administrateur (ADR 0009 §5).
CREATE TABLE demandes_changement_ifu (
    id BIGSERIAL PRIMARY KEY,
    entreprise_id BIGINT NOT NULL REFERENCES entreprises(id) ON DELETE CASCADE,
    ancien_ifu TEXT NOT NULL,
    nouveau_ifu TEXT NOT NULL,
    statut TEXT NOT NULL DEFAULT 'EN_ATTENTE'
        CHECK (statut IN ('EN_ATTENTE', 'APPROUVEE', 'REFUSEE')),
    cree_le TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_demandes_ifu ON demandes_changement_ifu (entreprise_id, statut);

-- Justificatifs facultatifs (attestation IFU, extrait RCCM) : PDF / JPG / PNG ≤ 5 Mo.
CREATE TABLE justificatifs (
    id BIGSERIAL PRIMARY KEY,
    entreprise_id BIGINT NOT NULL REFERENCES entreprises(id) ON DELETE CASCADE,
    categorie TEXT NOT NULL CHECK (categorie IN ('ATTESTATION_IFU', 'EXTRAIT_RCCM')),
    nom_fichier TEXT NOT NULL,
    mime TEXT NOT NULL CHECK (mime IN ('application/pdf', 'image/png', 'image/jpeg')),
    contenu BYTEA NOT NULL,
    taille INTEGER NOT NULL CHECK (taille > 0),
    depose_le TIMESTAMPTZ NOT NULL DEFAULT now()
);
