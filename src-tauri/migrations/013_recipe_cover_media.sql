-- Media bytes are owned by a separate, restart-reconciled Rust filesystem journal.
-- Ready assets are immutable and conservatively retained: no automatic physical GC.
CREATE TABLE media_assets (
 id TEXT PRIMARY KEY,
 state TEXT NOT NULL CHECK(state IN ('writing','ready','failed')),
 manifest TEXT CHECK(manifest IS NULL OR (length(manifest)<=8192 AND json_valid(manifest))),
 created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
 CHECK(state<>'ready' OR (manifest IS NOT NULL AND json_extract(manifest,'$.formatVersion')=1 AND json_extract(manifest,'$.assetId')=id))
) STRICT;
CREATE INDEX media_pending ON media_assets(state,id);
CREATE TRIGGER media_immutable_update BEFORE UPDATE ON media_assets WHEN OLD.state='ready' BEGIN SELECT RAISE(ABORT,'immutable ready media'); END;
CREATE TRIGGER media_retained_delete BEFORE DELETE ON media_assets BEGIN SELECT RAISE(ABORT,'media retention requires explicit future cleanup'); END;
CREATE TRIGGER media_no_replace BEFORE INSERT ON media_assets WHEN EXISTS(SELECT 1 FROM media_assets WHERE id=NEW.id) BEGIN SELECT RAISE(ABORT,'immutable media identity'); END;
CREATE TABLE recipe_covers (
 recipe_id TEXT PRIMARY KEY REFERENCES recipes(id) ON DELETE CASCADE,
 asset_id TEXT NOT NULL REFERENCES media_assets(id) ON DELETE RESTRICT
) STRICT;
CREATE INDEX covers_asset ON recipe_covers(asset_id);
CREATE TABLE draft_media (
 draft_id TEXT PRIMARY KEY REFERENCES editor_drafts(id) ON DELETE CASCADE,
 asset_id TEXT NOT NULL REFERENCES media_assets(id) ON DELETE RESTRICT
) STRICT;
CREATE INDEX draft_media_asset ON draft_media(asset_id);
CREATE TABLE recipe_version_media (
 recipe_id TEXT NOT NULL,
 version_number INTEGER NOT NULL,
 asset_id TEXT NOT NULL REFERENCES media_assets(id) ON DELETE RESTRICT,
 PRIMARY KEY(recipe_id,version_number),
 FOREIGN KEY(recipe_id,version_number) REFERENCES recipe_versions(recipe_id,version_number) ON DELETE CASCADE
) STRICT;
CREATE INDEX historical_media_asset ON recipe_version_media(asset_id);
CREATE TRIGGER media_history_capture AFTER INSERT ON recipe_versions WHEN NEW.format_version=3 AND json_extract(NEW.snapshot,'$.coverAssetId') IS NOT NULL BEGIN
 INSERT INTO recipe_version_media(recipe_id,version_number,asset_id) VALUES(NEW.recipe_id,NEW.version_number,json_extract(NEW.snapshot,'$.coverAssetId'));
END;
CREATE TRIGGER media_history_update BEFORE UPDATE ON recipe_version_media BEGIN SELECT RAISE(ABORT,'immutable media history'); END;
CREATE TRIGGER media_history_delete BEFORE DELETE ON recipe_version_media WHEN EXISTS(SELECT 1 FROM recipe_versions WHERE recipe_id=OLD.recipe_id AND version_number=OLD.version_number) BEGIN SELECT RAISE(ABORT,'media history requires owned purge'); END;
CREATE TRIGGER media_history_insert BEFORE INSERT ON recipe_version_media BEGIN
 SELECT CASE WHEN EXISTS(SELECT 1 FROM recipe_version_media WHERE recipe_id=NEW.recipe_id AND version_number=NEW.version_number) OR NOT EXISTS(SELECT 1 FROM recipe_versions WHERE recipe_id=NEW.recipe_id AND version_number=NEW.version_number AND format_version=3 AND json_extract(snapshot,'$.coverAssetId')=NEW.asset_id) THEN RAISE(ABORT,'invalid media history reference') END;
 SELECT CASE WHEN NOT EXISTS(SELECT 1 FROM media_assets WHERE id=NEW.asset_id AND state='ready') THEN RAISE(ABORT,'media not ready') END;
END;
CREATE TRIGGER cover_ready_insert BEFORE INSERT ON recipe_covers BEGIN SELECT CASE WHEN NOT EXISTS(SELECT 1 FROM media_assets WHERE id=NEW.asset_id AND state='ready') THEN RAISE(ABORT,'media not ready') END; END;
CREATE TRIGGER cover_ready_update BEFORE UPDATE ON recipe_covers BEGIN SELECT CASE WHEN NOT EXISTS(SELECT 1 FROM media_assets WHERE id=NEW.asset_id AND state='ready') THEN RAISE(ABORT,'media not ready') END; END;
CREATE TRIGGER draft_media_ready_insert BEFORE INSERT ON draft_media BEGIN SELECT CASE WHEN NOT EXISTS(SELECT 1 FROM media_assets WHERE id=NEW.asset_id AND state='ready') THEN RAISE(ABORT,'media not ready') END; END;
CREATE TRIGGER draft_media_ready_update BEFORE UPDATE ON draft_media BEGIN SELECT CASE WHEN NOT EXISTS(SELECT 1 FROM media_assets WHERE id=NEW.asset_id AND state='ready') THEN RAISE(ABORT,'media not ready') END; END;
DROP TRIGGER version_alternatives_validate_insert;
CREATE TRIGGER version_alternatives_validate_insert BEFORE INSERT ON recipe_version_alternatives BEGIN
 SELECT CASE WHEN EXISTS(SELECT 1 FROM recipe_version_alternatives WHERE recipe_id=NEW.recipe_id AND version_number=NEW.version_number AND alternative_id=NEW.alternative_id)
 THEN RAISE(ABORT,'immutable alternative history reference') END;
 SELECT CASE WHEN NOT EXISTS(
 SELECT 1 FROM recipe_versions v,json_each(v.snapshot,'$.ingredients') i,json_each(i.value,'$.alternatives') a
 WHERE v.recipe_id=NEW.recipe_id AND v.version_number=NEW.version_number AND v.format_version IN (2,3)
 AND json_extract(i.value,'$.id')=NEW.line_id AND json_extract(a.value,'$.id')=NEW.alternative_id
 AND json_extract(a.value,'$.ingredientId')=NEW.ingredient_id AND json_extract(a.value,'$.unitCode') IS NEW.unit_code)
 THEN RAISE(ABORT,'invalid alternative history reference') END;
END;
CREATE TRIGGER version3_alternatives_capture AFTER INSERT ON recipe_versions WHEN NEW.format_version=3 BEGIN
 INSERT INTO recipe_version_alternatives(recipe_id,version_number,line_id,alternative_id,ingredient_id,unit_code)
 SELECT NEW.recipe_id,NEW.version_number,json_extract(i.value,'$.id'),json_extract(a.value,'$.id'),json_extract(a.value,'$.ingredientId'),json_extract(a.value,'$.unitCode') FROM json_each(NEW.snapshot,'$.ingredients') i,json_each(i.value,'$.alternatives') a;
END;
