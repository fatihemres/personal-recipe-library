CREATE TABLE recipe_search_docs (
 recipe_id TEXT PRIMARY KEY REFERENCES recipes(id) ON DELETE CASCADE,
 title_tr TEXT NOT NULL,
 title_en TEXT NOT NULL,
 body_tr TEXT NOT NULL,
 body_en TEXT NOT NULL,
 ingredients_tr TEXT NOT NULL,
 ingredients_en TEXT NOT NULL,
 sort_key TEXT NOT NULL
) STRICT;
CREATE VIRTUAL TABLE recipe_fts USING fts5(
 title_tr, title_en, body_tr, body_en, ingredients_tr, ingredients_en,
 tokenize = 'unicode61 remove_diacritics 0'
);
CREATE TRIGGER recipe_search_docs_insert AFTER INSERT ON recipe_search_docs BEGIN
 INSERT INTO recipe_fts(rowid,title_tr,title_en,body_tr,body_en,ingredients_tr,ingredients_en)
 VALUES (new.rowid,new.title_tr,new.title_en,new.body_tr,new.body_en,new.ingredients_tr,new.ingredients_en);
END;
CREATE TRIGGER recipe_search_docs_update AFTER UPDATE ON recipe_search_docs BEGIN
 DELETE FROM recipe_fts WHERE rowid=old.rowid;
 INSERT INTO recipe_fts(rowid,title_tr,title_en,body_tr,body_en,ingredients_tr,ingredients_en)
 VALUES (new.rowid,new.title_tr,new.title_en,new.body_tr,new.body_en,new.ingredients_tr,new.ingredients_en);
END;
CREATE TRIGGER recipe_search_docs_delete AFTER DELETE ON recipe_search_docs BEGIN
 DELETE FROM recipe_fts WHERE rowid=old.rowid;
END;
CREATE INDEX recipes_search_scope ON recipes(deleted_at,archived_at,kind,updated_at,id);
CREATE INDEX recipes_search_created ON recipes(created_at,id);
CREATE TABLE recipe_search_state (id INTEGER PRIMARY KEY CHECK(id=1), ready INTEGER NOT NULL CHECK(ready IN (0,1))) STRICT;
INSERT INTO recipe_search_state(id,ready) VALUES(1,0);
