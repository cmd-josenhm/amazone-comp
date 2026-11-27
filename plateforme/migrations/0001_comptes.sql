-- Comptes utilisateurs de la plateforme (ADR 0004). Les mots de passe sont hachés en Argon2id.

CREATE TABLE comptes (
    id BIGSERIAL PRIMARY KEY,
    courriel TEXT NOT NULL UNIQUE,
    nom TEXT NOT NULL,
    mot_de_passe_hash TEXT NOT NULL,
    courriel_verifie_le TIMESTAMPTZ,
    actif BOOLEAN NOT NULL DEFAULT TRUE,
    cree_le TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Codes à usage unique (vérification du courriel, réinitialisation du mot de passe).
-- Seule l'empreinte SHA-256 du code est stockée.
CREATE TABLE codes_verification (
    id BIGSERIAL PRIMARY KEY,
    compte_id BIGINT NOT NULL REFERENCES comptes(id) ON DELETE CASCADE,
    categorie TEXT NOT NULL CHECK (categorie IN ('VERIFICATION_COURRIEL', 'REINITIALISATION_MOT_DE_PASSE')),
    code_hash TEXT NOT NULL,
    expire_le TIMESTAMPTZ NOT NULL,
    tentatives INTEGER NOT NULL DEFAULT 0,
    utilise_le TIMESTAMPTZ,
    cree_le TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_codes_verification_compte ON codes_verification (compte_id, categorie, utilise_le);

-- Sessions : jetons d'accès courts. Seule l'empreinte SHA-256 du jeton est stockée.
CREATE TABLE sessions (
    id BIGSERIAL PRIMARY KEY,
    compte_id BIGINT NOT NULL REFERENCES comptes(id) ON DELETE CASCADE,
    jeton_hash TEXT NOT NULL UNIQUE,
    expire_le TIMESTAMPTZ NOT NULL,
    cree_le TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_sessions_compte ON sessions (compte_id);
