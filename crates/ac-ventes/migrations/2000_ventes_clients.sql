-- Clients de la vente. Propriétaire : A4 (plage 2000–2999). Ne jamais modifier une migration publiée.

CREATE TABLE ventes_client (
    id                  INTEGER PRIMARY KEY,
    -- Code stable, tiers comptable « CLI-… » (contrat C3), attribué automatiquement.
    code                TEXT NOT NULL UNIQUE
                        CHECK (code GLOB 'CLI-[0-9][0-9][0-9][0-9][0-9][0-9]'),
    nom                 TEXT NOT NULL CHECK (length(trim(nom)) > 0),
    type_client         TEXT NOT NULL
                        CHECK (type_client IN ('PARTICULIER','ENTREPRISE','ADMINISTRATION')),
    -- IFU facultatif : 13 chiffres quand il existe (contrat C11).
    ifu                 TEXT CHECK (ifu IS NULL OR (length(ifu) = 13 AND ifu NOT GLOB '*[^0-9]*')),
    rccm                TEXT,
    adresse             TEXT,
    telephone           TEXT,                           -- normalisé +229… (Telephone::new)
    courriel            TEXT,
    -- Le client retient la TVA à la source (État, sociétés désignées).
    retient_tva         INTEGER NOT NULL DEFAULT 0 CHECK (retient_tva IN (0, 1)),
    -- Exonération d'AIB accordée au client : motif obligatoire.
    exonere_aib         INTEGER NOT NULL DEFAULT 0 CHECK (exonere_aib IN (0, 1)),
    motif_exonere_aib   TEXT,
    -- Un client facturé ne se supprime jamais : on le désactive (actif = 0).
    actif               INTEGER NOT NULL DEFAULT 1 CHECK (actif IN (0, 1)),
    cree_le             TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    modifie_le          TEXT,
    CHECK (exonere_aib = 0 OR (motif_exonere_aib IS NOT NULL AND length(trim(motif_exonere_aib)) > 0))
);

-- Un IFU identifie un seul client.
CREATE UNIQUE INDEX ventes_client_ifu_unique ON ventes_client (ifu) WHERE ifu IS NOT NULL;

-- Recherche par nom ou téléphone.
CREATE INDEX ventes_client_nom ON ventes_client (nom);
CREATE INDEX ventes_client_telephone ON ventes_client (telephone);

-- Client « Comptant » obligatoire (vente au comptoir), créé avec chaque dossier.
INSERT INTO ventes_client (id, code, nom, type_client, actif)
VALUES (1, 'CLI-000001', 'Client comptant', 'PARTICULIER', 1);

-- Le client « Comptant » ne peut pas être désactivé (la vente au comptoir doit toujours marcher).
CREATE TRIGGER ventes_client_comptant_toujours_actif
BEFORE UPDATE OF actif ON ventes_client
WHEN old.code = 'CLI-000001' AND new.actif = 0
BEGIN
    SELECT RAISE(ABORT, 'le client Comptant ne peut pas etre desactive');
END;
