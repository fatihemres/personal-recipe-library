CREATE TABLE recipe_organization (
 recipe_id TEXT PRIMARY KEY REFERENCES recipes(id) ON DELETE CASCADE,
 revision INTEGER NOT NULL DEFAULT 1 CHECK(revision > 0),
 favorite INTEGER NOT NULL DEFAULT 0 CHECK(favorite IN (0,1)),
 to_try INTEGER NOT NULL DEFAULT 0 CHECK(to_try IN (0,1)),
 rating INTEGER CHECK(rating BETWEEN 1 AND 5)
) STRICT;
INSERT INTO recipe_organization(recipe_id) SELECT id FROM recipes;
CREATE TRIGGER recipe_organization_insert AFTER INSERT ON recipes BEGIN
 INSERT INTO recipe_organization(recipe_id) VALUES(new.id);
END;
CREATE INDEX recipe_organization_favorite ON recipe_organization(favorite,recipe_id);
CREATE INDEX recipe_organization_to_try ON recipe_organization(to_try,recipe_id);

CREATE TABLE recipe_collections (
 id TEXT PRIMARY KEY CHECK(length(id)=36),
 title TEXT NOT NULL CHECK(length(trim(title)) BETWEEN 1 AND 100),
 title_key TEXT NOT NULL UNIQUE,
 revision INTEGER NOT NULL DEFAULT 1 CHECK(revision > 0),
 created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
) STRICT;
CREATE TABLE recipe_collection_members (
 collection_id TEXT NOT NULL REFERENCES recipe_collections(id) ON DELETE CASCADE,
 recipe_id TEXT NOT NULL REFERENCES recipes(id) ON DELETE CASCADE,
 PRIMARY KEY(collection_id,recipe_id)
) STRICT;
CREATE INDEX recipe_collection_members_recipe ON recipe_collection_members(recipe_id,collection_id);
