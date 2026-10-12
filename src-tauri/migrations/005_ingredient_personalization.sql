-- Favorited canonical definitions remain visible even before recipe materialization.
CREATE TABLE catalog_ingredient_favorites (
 catalog_id TEXT PRIMARY KEY REFERENCES catalog_ingredients(id) ON DELETE CASCADE,
 created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
) STRICT;
CREATE TABLE personal_ingredient_favorites (
 ingredient_id TEXT PRIMARY KEY REFERENCES ingredients(id) ON DELETE CASCADE,
 created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
) STRICT;
CREATE TABLE ingredient_personal_aliases (
 catalog_id TEXT PRIMARY KEY REFERENCES catalog_ingredients(id) ON DELETE CASCADE,
 alias TEXT NOT NULL CHECK(length(trim(alias)) BETWEEN 1 AND 200),
 search_key TEXT NOT NULL,
 updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
) STRICT;
CREATE INDEX ingredient_personal_alias_search ON ingredient_personal_aliases(search_key);
CREATE INDEX recipe_ingredient_recent ON recipe_ingredients(ingredient_id,recipe_id);
