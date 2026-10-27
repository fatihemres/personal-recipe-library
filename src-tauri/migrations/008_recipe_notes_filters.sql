CREATE TABLE recipe_private_notes (
  recipe_id TEXT PRIMARY KEY REFERENCES recipes(id) ON DELETE CASCADE,
  format_version INTEGER NOT NULL DEFAULT 1 CHECK(format_version = 1),
  markdown TEXT NOT NULL DEFAULT '',
  revision INTEGER NOT NULL DEFAULT 1 CHECK(revision > 0),
  updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  CHECK(length(markdown) <= 50000)
) STRICT;

CREATE TABLE saved_recipe_filters (
  id TEXT PRIMARY KEY,
  title TEXT NOT NULL CHECK(length(trim(title)) BETWEEN 1 AND 100),
  title_key TEXT NOT NULL UNIQUE,
  revision INTEGER NOT NULL DEFAULT 1 CHECK(revision > 0),
  format_version INTEGER NOT NULL DEFAULT 1 CHECK(format_version > 0),
  query_json TEXT NOT NULL CHECK(json_valid(query_json)),
  updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
) STRICT;
