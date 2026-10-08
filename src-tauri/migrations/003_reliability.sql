ALTER TABLE recipes ADD COLUMN archived_at TEXT;
CREATE INDEX recipes_scope ON recipes(deleted_at, archived_at, kind, updated_at);
ALTER TABLE ingredients ADD COLUMN notes TEXT CHECK(length(notes)<=2000);
ALTER TABLE ingredients ADD COLUMN preferred_unit TEXT REFERENCES units(code) ON DELETE RESTRICT;
ALTER TABLE ingredients ADD COLUMN revision INTEGER NOT NULL DEFAULT 1 CHECK(revision>0);
ALTER TABLE ingredients ADD COLUMN updated_at TEXT;
UPDATE ingredients SET updated_at=created_at;
CREATE TABLE editor_drafts (
 id TEXT PRIMARY KEY CHECK(length(id)=36),
 recipe_id TEXT REFERENCES recipes(id) ON DELETE SET NULL,
 revision INTEGER NOT NULL CHECK(revision>0),
 format_version INTEGER NOT NULL DEFAULT 1 CHECK(format_version=1),
 state TEXT NOT NULL CHECK(state IN ('active','closed')),
 payload TEXT CHECK(payload IS NULL OR json_valid(payload)),
 updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
 CHECK((state='active' AND payload IS NOT NULL) OR (state='closed' AND payload IS NULL))
) STRICT;
CREATE INDEX drafts_recovery ON editor_drafts(state,updated_at);
CREATE INDEX drafts_recipe ON editor_drafts(recipe_id);
CREATE TABLE draft_ingredients (
 draft_id TEXT NOT NULL REFERENCES editor_drafts(id) ON DELETE CASCADE,
 ingredient_id TEXT NOT NULL REFERENCES ingredients(id) ON DELETE RESTRICT,
 PRIMARY KEY(draft_id,ingredient_id)
) STRICT;
CREATE INDEX draft_ingredient_reference ON draft_ingredients(ingredient_id);
