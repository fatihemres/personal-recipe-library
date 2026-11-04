-- Owned aggregate rows: the writer rebuilds these explicitly from the validated
-- input after replacing primary lines, all inside the recipe/CAS/history tx.
CREATE TABLE recipe_ingredient_alternatives (
 id TEXT PRIMARY KEY,
 recipe_id TEXT NOT NULL,
 line_id TEXT NOT NULL,
 ingredient_id TEXT NOT NULL REFERENCES ingredients(id) ON DELETE RESTRICT,
 quantity TEXT CHECK(quantity IS NULL OR (length(quantity) BETWEEN 1 AND 19)),
 unit_code TEXT REFERENCES units(code) ON DELETE RESTRICT,
 position INTEGER NOT NULL CHECK(position BETWEEN 0 AND 9),
 UNIQUE(line_id,position), UNIQUE(line_id,ingredient_id),
 FOREIGN KEY(recipe_id,line_id) REFERENCES recipe_ingredients(recipe_id,id) ON DELETE CASCADE
) STRICT;
CREATE INDEX alternative_ingredient_reference ON recipe_ingredient_alternatives(ingredient_id);
CREATE INDEX alternative_recipe_order ON recipe_ingredient_alternatives(recipe_id,line_id,position);
CREATE TABLE recipe_version_alternatives (
 recipe_id TEXT NOT NULL,
 version_number INTEGER NOT NULL,
 line_id TEXT NOT NULL,
 alternative_id TEXT NOT NULL,
 ingredient_id TEXT NOT NULL REFERENCES ingredients(id) ON DELETE RESTRICT,
 unit_code TEXT REFERENCES units(code) ON DELETE RESTRICT,
 PRIMARY KEY(recipe_id,version_number,alternative_id),
 FOREIGN KEY(recipe_id,version_number) REFERENCES recipe_versions(recipe_id,version_number) ON DELETE CASCADE
) STRICT;
CREATE INDEX historical_alternative_ingredient ON recipe_version_alternatives(ingredient_id);
CREATE INDEX historical_alternative_unit ON recipe_version_alternatives(unit_code);
CREATE TRIGGER version_alternatives_capture AFTER INSERT ON recipe_versions WHEN NEW.format_version=2 BEGIN
 INSERT INTO recipe_version_alternatives(recipe_id,version_number,line_id,alternative_id,ingredient_id,unit_code)
 SELECT NEW.recipe_id,NEW.version_number,json_extract(i.value,'$.id'),json_extract(a.value,'$.id'),json_extract(a.value,'$.ingredientId'),json_extract(a.value,'$.unitCode')
 FROM json_each(NEW.snapshot,'$.ingredients') i,json_each(i.value,'$.alternatives') a;
END;
CREATE TRIGGER version_alternatives_immutable_update BEFORE UPDATE ON recipe_version_alternatives BEGIN
 SELECT RAISE(ABORT,'immutable alternative history reference');
END;
CREATE TRIGGER version_alternatives_owned_delete BEFORE DELETE ON recipe_version_alternatives
 WHEN EXISTS(SELECT 1 FROM recipe_versions WHERE recipe_id=OLD.recipe_id AND version_number=OLD.version_number) BEGIN
 SELECT RAISE(ABORT,'alternative history reference requires recipe purge');
END;
CREATE TRIGGER version_alternatives_validate_insert BEFORE INSERT ON recipe_version_alternatives BEGIN
 SELECT CASE WHEN EXISTS(SELECT 1 FROM recipe_version_alternatives WHERE recipe_id=NEW.recipe_id AND version_number=NEW.version_number AND alternative_id=NEW.alternative_id)
 THEN RAISE(ABORT,'immutable alternative history reference') END;
 SELECT CASE WHEN NOT EXISTS(
 SELECT 1 FROM recipe_versions v,json_each(v.snapshot,'$.ingredients') i,json_each(i.value,'$.alternatives') a
 WHERE v.recipe_id=NEW.recipe_id AND v.version_number=NEW.version_number AND v.format_version=2
 AND json_extract(i.value,'$.id')=NEW.line_id AND json_extract(a.value,'$.id')=NEW.alternative_id
 AND json_extract(a.value,'$.ingredientId')=NEW.ingredient_id AND json_extract(a.value,'$.unitCode') IS NEW.unit_code)
 THEN RAISE(ABORT,'invalid alternative history reference') END;
END;
CREATE TABLE draft_alternative_units (
 draft_id TEXT NOT NULL REFERENCES editor_drafts(id) ON DELETE CASCADE,
 unit_code TEXT NOT NULL REFERENCES units(code) ON DELETE RESTRICT,
 PRIMARY KEY(draft_id,unit_code)
) STRICT;
CREATE TRIGGER alternative_not_primary BEFORE INSERT ON recipe_ingredient_alternatives BEGIN
 SELECT CASE WHEN EXISTS(SELECT 1 FROM recipe_ingredients WHERE recipe_id=NEW.recipe_id AND id=NEW.line_id AND ingredient_id=NEW.ingredient_id)
 THEN RAISE(ABORT,'alternative equals primary ingredient') END;
END;
