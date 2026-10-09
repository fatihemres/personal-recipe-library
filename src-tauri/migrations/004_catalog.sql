-- Canonical system data is separate from recipe-facing/user-owned ingredient rows.
CREATE TABLE catalog_sources (
 id TEXT NOT NULL, version TEXT NOT NULL, name TEXT NOT NULL,
 url TEXT NOT NULL, license TEXT NOT NULL, attribution TEXT NOT NULL,
 retrieved_at TEXT NOT NULL, artifact_sha256 TEXT NOT NULL CHECK(length(artifact_sha256)=64),
 PRIMARY KEY(id,version)
) STRICT;
CREATE TABLE catalog_releases (
 dataset TEXT NOT NULL, version INTEGER NOT NULL CHECK(version>0),
 purpose TEXT NOT NULL CHECK(purpose IN ('validation','production')),
 manifest_sha256 TEXT NOT NULL CHECK(length(manifest_sha256)=64),
 artifact_sha256 TEXT NOT NULL CHECK(length(artifact_sha256)=64),
 installed_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
 PRIMARY KEY(dataset,version)
) STRICT;
CREATE TABLE catalog_import_runs (
 id TEXT PRIMARY KEY, dataset TEXT NOT NULL, version INTEGER NOT NULL,
 status TEXT NOT NULL CHECK(status IN ('running','succeeded','failed')),
 report TEXT CHECK(report IS NULL OR json_valid(report)), error_code TEXT,
 started_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')), finished_at TEXT
) STRICT;
CREATE TABLE catalog_ingredients (
 id TEXT PRIMARY KEY CHECK(length(id)=36), canonical_key TEXT NOT NULL UNIQUE,
 canonical_name TEXT NOT NULL, ingredient_type TEXT NOT NULL,
 preferred_unit TEXT REFERENCES units(code) ON DELETE RESTRICT,
 dataset TEXT NOT NULL, catalog_version INTEGER NOT NULL,
 record_sha256 TEXT NOT NULL CHECK(length(record_sha256)=64),
 FOREIGN KEY(dataset,catalog_version) REFERENCES catalog_releases(dataset,version)
) STRICT;
CREATE TABLE catalog_names (
 ingredient_id TEXT NOT NULL REFERENCES catalog_ingredients(id) ON DELETE CASCADE,
 locale TEXT NOT NULL CHECK(locale IN ('tr','en')), name TEXT NOT NULL,
 search_key TEXT NOT NULL, PRIMARY KEY(ingredient_id,locale)
) STRICT;
CREATE INDEX catalog_name_search ON catalog_names(search_key);
CREATE TABLE catalog_aliases (
 ingredient_id TEXT NOT NULL REFERENCES catalog_ingredients(id) ON DELETE CASCADE,
 locale TEXT NOT NULL CHECK(locale IN ('tr','en')), name TEXT NOT NULL,
 search_key TEXT NOT NULL, PRIMARY KEY(ingredient_id,locale,search_key)
) STRICT;
CREATE INDEX catalog_alias_search ON catalog_aliases(search_key);
CREATE TABLE ingredient_categories (
 id TEXT PRIMARY KEY CHECK(length(id)=36), stable_key TEXT NOT NULL UNIQUE,
 parent_id TEXT REFERENCES ingredient_categories(id) ON DELETE RESTRICT,
 name_tr TEXT NOT NULL, name_en TEXT NOT NULL,
 origin TEXT NOT NULL CHECK(origin IN ('catalog','personal')),
 dataset TEXT, CHECK(parent_id IS NULL OR parent_id<>id)
) STRICT;
CREATE INDEX ingredient_category_parent ON ingredient_categories(parent_id);
CREATE TRIGGER ingredient_category_no_cycle BEFORE UPDATE OF parent_id ON ingredient_categories
WHEN NEW.parent_id IS NOT NULL
BEGIN
 SELECT RAISE(ABORT,'category cycle') WHERE EXISTS (
  WITH RECURSIVE ancestors(id,parent_id) AS (
   SELECT id,parent_id FROM ingredient_categories WHERE id=NEW.parent_id
   UNION SELECT c.id,c.parent_id FROM ingredient_categories c JOIN ancestors a ON c.id=a.parent_id
  ) SELECT 1 FROM ancestors WHERE id=NEW.id
 );
END;
CREATE TABLE catalog_category_memberships (
 ingredient_id TEXT NOT NULL REFERENCES catalog_ingredients(id) ON DELETE CASCADE,
 category_id TEXT NOT NULL REFERENCES ingredient_categories(id) ON DELETE RESTRICT,
 PRIMARY KEY(ingredient_id,category_id)
) STRICT;
CREATE TABLE personal_category_memberships (
 ingredient_id TEXT NOT NULL REFERENCES ingredients(id) ON DELETE RESTRICT,
 category_id TEXT NOT NULL REFERENCES ingredient_categories(id) ON DELETE RESTRICT,
 PRIMARY KEY(ingredient_id,category_id)
) STRICT;
CREATE TABLE catalog_dimensions (
 ingredient_id TEXT NOT NULL REFERENCES catalog_ingredients(id) ON DELETE CASCADE,
 dimension TEXT NOT NULL CHECK(dimension IN ('mass','volume','count')),
 PRIMARY KEY(ingredient_id,dimension)
) STRICT;
CREATE TABLE catalog_relations (
 ingredient_id TEXT NOT NULL REFERENCES catalog_ingredients(id) ON DELETE CASCADE,
 related_id TEXT NOT NULL REFERENCES catalog_ingredients(id) ON DELETE RESTRICT,
 kind TEXT NOT NULL CHECK(kind IN ('form_of','related')),
 CHECK(ingredient_id<>related_id), PRIMARY KEY(ingredient_id,related_id,kind)
) STRICT;
CREATE TABLE catalog_provenance (
 ingredient_id TEXT NOT NULL REFERENCES catalog_ingredients(id) ON DELETE CASCADE,
 source_id TEXT NOT NULL, source_version TEXT NOT NULL, external_id TEXT NOT NULL,
 source_description TEXT NOT NULL,
 FOREIGN KEY(source_id,source_version) REFERENCES catalog_sources(id,version),
 PRIMARY KEY(ingredient_id,source_id,source_version,external_id),
 UNIQUE(source_id,source_version,external_id)
) STRICT;
-- Absence of an observation means unknown, including allergens and alcohol content.
CREATE TABLE catalog_observations (
 ingredient_id TEXT NOT NULL REFERENCES catalog_ingredients(id) ON DELETE CASCADE,
 kind TEXT NOT NULL CHECK(kind IN ('density','nutrient','allergen','abv','dietary')),
 code TEXT NOT NULL, value TEXT NOT NULL, unit TEXT, basis TEXT NOT NULL,
 source_id TEXT NOT NULL, source_version TEXT NOT NULL, external_id TEXT NOT NULL,
 FOREIGN KEY(ingredient_id,source_id,source_version,external_id)
 REFERENCES catalog_provenance(ingredient_id,source_id,source_version,external_id),
 PRIMARY KEY(ingredient_id,kind,code,basis)
) STRICT;
ALTER TABLE ingredients ADD COLUMN catalog_id TEXT REFERENCES catalog_ingredients(id) ON DELETE RESTRICT;
CREATE INDEX ingredient_catalog_link ON ingredients(catalog_id);
CREATE TABLE ingredient_customizations (
 ingredient_id TEXT NOT NULL REFERENCES ingredients(id) ON DELETE CASCADE,
 field TEXT NOT NULL CHECK(field IN ('name','notes','preferred_unit')),
 changed_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
 PRIMARY KEY(ingredient_id,field)
) STRICT;
CREATE TABLE catalog_collisions (
 catalog_id TEXT NOT NULL REFERENCES catalog_ingredients(id) ON DELETE CASCADE,
 personal_id TEXT NOT NULL REFERENCES ingredients(id) ON DELETE CASCADE,
 state TEXT NOT NULL CHECK(state IN ('pending','linked')),
 PRIMARY KEY(catalog_id,personal_id)
) STRICT;
