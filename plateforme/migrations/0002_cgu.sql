-- Conditions générales d'utilisation versionnées (ADR 0004) : l'utilisateur accepte
-- une version précise, qui est enregistrée (loi béninoise n° 2017-20).

CREATE TABLE cgu_versions (
    version INTEGER PRIMARY KEY,
    titre TEXT NOT NULL,
    texte TEXT NOT NULL,
    publiee_le TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE cgu_acceptations (
    id BIGSERIAL PRIMARY KEY,
    compte_id BIGINT NOT NULL REFERENCES comptes(id) ON DELETE CASCADE,
    cgu_version INTEGER NOT NULL REFERENCES cgu_versions(version),
    acceptee_le TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (compte_id, cgu_version)
);

-- Première version : brouillon « A COMPLETER » (les textes définitifs seront rédigés
-- par le propriétaire ; la console d'administration publiera les versions suivantes).
INSERT INTO cgu_versions (version, titre, texte) VALUES (
    1,
    'Conditions générales d''utilisation — brouillon',
    'A COMPLETER par le propriétaire (rédaction juridique). Ce brouillon est publié pour permettre le fonctionnement de la plateforme. En acceptant ces conditions, l''utilisateur s''engage à fournir des informations exactes lors de l''inscription de son entreprise et à ne pas partager sa licence. Le logiciel est distribué sous licence AGPL-3.0. Les données personnelles sont traitées conformément à la loi béninoise n° 2017-20 (Code du numérique) : collecte minimale, aucune donnée comptable transmise en clair.'
);
