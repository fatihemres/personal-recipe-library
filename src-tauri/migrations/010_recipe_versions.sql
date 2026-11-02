CREATE TABLE recipe_versions (
 recipe_id TEXT NOT NULL REFERENCES recipes(id) ON DELETE CASCADE,
 version_number INTEGER NOT NULL CHECK(version_number>0),
 format_version INTEGER NOT NULL CHECK(format_version>0),
 source_revision INTEGER NOT NULL CHECK(source_revision>0),
 provenance TEXT NOT NULL CHECK(provenance IN ('created','edited','upgrade_baseline','restored')),
 restored_from_version INTEGER,
 captured_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
 content_fingerprint TEXT NOT NULL CHECK(length(content_fingerprint)=64 AND content_fingerprint NOT GLOB '*[^0-9a-f]*'),
 snapshot_checksum TEXT NOT NULL CHECK(length(snapshot_checksum)=64 AND snapshot_checksum NOT GLOB '*[^0-9a-f]*'),
 snapshot TEXT NOT NULL CHECK(length(CAST(snapshot AS BLOB))<=134217728 AND json_valid(snapshot)),
 PRIMARY KEY(recipe_id,version_number),
 FOREIGN KEY(recipe_id,restored_from_version) REFERENCES recipe_versions(recipe_id,version_number) DEFERRABLE INITIALLY DEFERRED,
 CHECK((provenance='restored')=(restored_from_version IS NOT NULL)),
 CHECK(restored_from_version IS NULL OR restored_from_version BETWEEN 1 AND version_number-1),
 CHECK((version_number=1 AND provenance IN ('created','upgrade_baseline')) OR (version_number>1 AND provenance IN ('edited','restored'))),
 CHECK(json_extract(snapshot,'$.formatVersion') IS format_version AND json_extract(snapshot,'$.recipeId') IS recipe_id),
 CHECK(json_type(snapshot,'$.ingredients') IS 'array' AND json_type(snapshot,'$.steps') IS 'array')
) STRICT;

-- Derived from the immutable payload in the same INSERT. Retained history protects
-- ingredients and units even after removal from the currently editable recipe.
CREATE TABLE recipe_version_ingredients (
 recipe_id TEXT NOT NULL,
 version_number INTEGER NOT NULL,
 line_id TEXT NOT NULL,
 ingredient_id TEXT NOT NULL REFERENCES ingredients(id) ON DELETE RESTRICT,
 unit_code TEXT NOT NULL REFERENCES units(code) ON DELETE RESTRICT,
 PRIMARY KEY(recipe_id,version_number,line_id),
 FOREIGN KEY(recipe_id,version_number) REFERENCES recipe_versions(recipe_id,version_number) ON DELETE CASCADE
) STRICT;
CREATE INDEX historical_ingredient_reference ON recipe_version_ingredients(ingredient_id);
CREATE INDEX historical_unit_reference ON recipe_version_ingredients(unit_code);

CREATE TRIGGER recipe_version_sequence BEFORE INSERT ON recipe_versions BEGIN
 SELECT CASE WHEN NEW.version_number<>(SELECT coalesce(max(version_number),0)+1 FROM recipe_versions WHERE recipe_id=NEW.recipe_id)
 THEN RAISE(ABORT,'recipe version sequence') END;
END;
CREATE TRIGGER recipe_version_references AFTER INSERT ON recipe_versions BEGIN
 INSERT INTO recipe_version_ingredients(recipe_id,version_number,line_id,ingredient_id,unit_code)
 SELECT NEW.recipe_id,NEW.version_number,json_extract(value,'$.id'),json_extract(value,'$.ingredientId'),json_extract(value,'$.unitCode')
 FROM json_each(NEW.snapshot,'$.ingredients');
END;
CREATE TRIGGER recipe_versions_immutable_update BEFORE UPDATE ON recipe_versions BEGIN
 SELECT RAISE(ABORT,'immutable recipe version');
END;
CREATE TRIGGER recipe_versions_owned_delete BEFORE DELETE ON recipe_versions
 WHEN EXISTS(SELECT 1 FROM recipes WHERE id=OLD.recipe_id) BEGIN
 SELECT RAISE(ABORT,'recipe history requires recipe purge');
END;
CREATE TRIGGER recipe_version_refs_immutable_update BEFORE UPDATE ON recipe_version_ingredients BEGIN
 SELECT RAISE(ABORT,'immutable recipe version reference');
END;
CREATE TRIGGER recipe_version_refs_owned_delete BEFORE DELETE ON recipe_version_ingredients
 WHEN EXISTS(SELECT 1 FROM recipe_versions WHERE recipe_id=OLD.recipe_id AND version_number=OLD.version_number) BEGIN
 SELECT RAISE(ABORT,'history reference requires version purge');
END;
CREATE TRIGGER recipe_version_refs_validate_insert BEFORE INSERT ON recipe_version_ingredients BEGIN
 SELECT CASE WHEN NOT EXISTS(
  SELECT 1 FROM recipe_versions v,json_each(v.snapshot,'$.ingredients') i
  WHERE v.recipe_id=NEW.recipe_id AND v.version_number=NEW.version_number
   AND json_extract(i.value,'$.id')=NEW.line_id
   AND json_extract(i.value,'$.ingredientId')=NEW.ingredient_id
   AND json_extract(i.value,'$.unitCode')=NEW.unit_code)
 THEN RAISE(ABORT,'invalid history reference') END;
END;
