CREATE TABLE achats_article (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    code TEXT NOT NULL COLLATE NOCASE UNIQUE,
    designation TEXT NOT NULL,
    type_article TEXT NOT NULL CHECK (type_article IN ('BIEN_STOCKE', 'BIEN_NON_STOCKE', 'SERVICE')),
    unite TEXT NOT NULL,
    prix_vente_ht INTEGER NOT NULL CHECK (prix_vente_ht >= 0),
    groupe_taxation TEXT NOT NULL,
    compte_vente TEXT NOT NULL,
    compte_achat TEXT NOT NULL,
    stock_min_milliemes INTEGER NOT NULL DEFAULT 0 CHECK (stock_min_milliemes >= 0),
    actif INTEGER NOT NULL DEFAULT 1 CHECK (actif IN (0, 1))
);

CREATE INDEX achats_article_designation_idx ON achats_article(designation COLLATE NOCASE);

CREATE TABLE achats_stock_mouvement (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    article_id INTEGER NOT NULL REFERENCES achats_article(id),
    nature TEXT NOT NULL CHECK (nature IN ('ENTREE', 'SORTIE', 'AJUSTEMENT')),
    quantite_delta_milliemes INTEGER NOT NULL CHECK (quantite_delta_milliemes != 0),
    valeur_delta_fcfa INTEGER NOT NULL,
    cmup_fcfa INTEGER NOT NULL CHECK (cmup_fcfa >= 0),
    reference TEXT NOT NULL,
    utilisateur_id INTEGER NOT NULL,
    cree_le TEXT NOT NULL
);

CREATE INDEX achats_stock_article_mouvement_idx
    ON achats_stock_mouvement(article_id, id);
CREATE INDEX achats_stock_reference_idx ON achats_stock_mouvement(reference);

CREATE TRIGGER achats_stock_mouvement_non_negatif
BEFORE INSERT ON achats_stock_mouvement
WHEN COALESCE((
    SELECT SUM(quantite_delta_milliemes)
    FROM achats_stock_mouvement
    WHERE article_id = NEW.article_id
), 0) + NEW.quantite_delta_milliemes < 0
BEGIN
    SELECT RAISE(ABORT, 'stock négatif interdit');
END;

CREATE TRIGGER achats_stock_mouvement_immutable_update
BEFORE UPDATE ON achats_stock_mouvement
BEGIN
    SELECT RAISE(ABORT, 'les mouvements de stock sont immuables');
END;

CREATE TRIGGER achats_stock_mouvement_immutable_delete
BEFORE DELETE ON achats_stock_mouvement
BEGIN
    SELECT RAISE(ABORT, 'les mouvements de stock sont immuables');
END;
