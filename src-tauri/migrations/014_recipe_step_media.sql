-- One optional immutable media asset for each stable step UUID.
-- Associations, not files, cascade during owned aggregate replacement/purge.
CREATE TABLE recipe_step_media (
 recipe_id TEXT NOT NULL,
 step_id TEXT NOT NULL,
 asset_id TEXT NOT NULL REFERENCES media_assets(id) ON DELETE RESTRICT,
 PRIMARY KEY(recipe_id,step_id),
 FOREIGN KEY(recipe_id,step_id) REFERENCES recipe_steps(recipe_id,id) ON DELETE CASCADE
) STRICT;
CREATE INDEX step_media_asset ON recipe_step_media(asset_id);
CREATE TABLE draft_step_media (
 draft_id TEXT NOT NULL REFERENCES editor_drafts(id) ON DELETE CASCADE,
 step_id TEXT NOT NULL,
 asset_id TEXT NOT NULL REFERENCES media_assets(id) ON DELETE RESTRICT,
 PRIMARY KEY(draft_id,step_id)
) STRICT;
CREATE INDEX draft_step_media_asset ON draft_step_media(asset_id);
CREATE TABLE recipe_version_step_media (
 recipe_id TEXT NOT NULL,
 version_number INTEGER NOT NULL,
 step_id TEXT NOT NULL,
 asset_id TEXT NOT NULL REFERENCES media_assets(id) ON DELETE RESTRICT,
 PRIMARY KEY(recipe_id,version_number,step_id),
 FOREIGN KEY(recipe_id,version_number) REFERENCES recipe_versions(recipe_id,version_number) ON DELETE CASCADE
) STRICT;
CREATE INDEX historical_step_media_asset ON recipe_version_step_media(asset_id);
CREATE TRIGGER step_media_ready_insert BEFORE INSERT ON recipe_step_media BEGIN
 SELECT CASE WHEN NOT EXISTS(SELECT 1 FROM media_assets WHERE id=NEW.asset_id AND state='ready') THEN RAISE(ABORT,'step media not ready') END;
END;
CREATE TRIGGER draft_step_media_valid_insert BEFORE INSERT ON draft_step_media BEGIN
 SELECT CASE WHEN NOT EXISTS(SELECT 1 FROM media_assets WHERE id=NEW.asset_id AND state='ready') THEN RAISE(ABORT,'draft step media not ready') END;
 SELECT CASE WHEN (SELECT count(*) FROM editor_drafts d,json_each(d.payload,'$.steps') s WHERE d.id=NEW.draft_id AND d.state='active' AND json_extract(s.value,'$.id')=NEW.step_id)<>1
 OR NOT EXISTS(SELECT 1 FROM editor_drafts d,json_each(d.payload,'$.steps') s WHERE d.id=NEW.draft_id AND d.state='active' AND json_extract(s.value,'$.id')=NEW.step_id AND json_extract(s.value,'$.photoAssetId')=NEW.asset_id)
 THEN RAISE(ABORT,'draft step media ownership') END;
END;
CREATE TRIGGER step_media_ready_update BEFORE UPDATE ON recipe_step_media BEGIN
 SELECT CASE WHEN NOT EXISTS(SELECT 1 FROM media_assets WHERE id=NEW.asset_id AND state='ready') THEN RAISE(ABORT,'step media not ready') END;
END;
CREATE TRIGGER draft_step_media_valid_update BEFORE UPDATE ON draft_step_media BEGIN
 SELECT CASE WHEN NOT EXISTS(SELECT 1 FROM media_assets WHERE id=NEW.asset_id AND state='ready') THEN RAISE(ABORT,'draft step media not ready') END;
 SELECT CASE WHEN (SELECT count(*) FROM editor_drafts d,json_each(d.payload,'$.steps') s WHERE d.id=NEW.draft_id AND d.state='active' AND json_extract(s.value,'$.id')=NEW.step_id)<>1
 OR NOT EXISTS(SELECT 1 FROM editor_drafts d,json_each(d.payload,'$.steps') s WHERE d.id=NEW.draft_id AND d.state='active' AND json_extract(s.value,'$.id')=NEW.step_id AND json_extract(s.value,'$.photoAssetId')=NEW.asset_id)
 THEN RAISE(ABORT,'draft step media ownership') END;
END;
CREATE TRIGGER step_media_history_capture AFTER INSERT ON recipe_versions WHEN NEW.format_version=4 BEGIN
 INSERT INTO recipe_version_step_media(recipe_id,version_number,step_id,asset_id)
 SELECT NEW.recipe_id,NEW.version_number,json_extract(s.value,'$.id'),json_extract(s.value,'$.photoAssetId')
 FROM json_each(NEW.snapshot,'$.steps') s WHERE json_extract(s.value,'$.photoAssetId') IS NOT NULL;
END;
CREATE TRIGGER step_media_history_insert BEFORE INSERT ON recipe_version_step_media BEGIN
 SELECT CASE WHEN EXISTS(SELECT 1 FROM recipe_version_step_media WHERE recipe_id=NEW.recipe_id AND version_number=NEW.version_number AND step_id=NEW.step_id)
 THEN RAISE(ABORT,'immutable step media reference') END;
 SELECT CASE WHEN (SELECT count(*) FROM recipe_versions v,json_each(v.snapshot,'$.steps') s WHERE v.recipe_id=NEW.recipe_id AND v.version_number=NEW.version_number AND v.format_version=4 AND json_extract(s.value,'$.id')=NEW.step_id)<>1
 OR NOT EXISTS(SELECT 1 FROM recipe_versions v,json_each(v.snapshot,'$.steps') s WHERE v.recipe_id=NEW.recipe_id AND v.version_number=NEW.version_number AND v.format_version=4 AND json_extract(s.value,'$.id')=NEW.step_id AND json_extract(s.value,'$.photoAssetId')=NEW.asset_id)
 THEN RAISE(ABORT,'invalid step media history reference') END;
 SELECT CASE WHEN NOT EXISTS(SELECT 1 FROM media_assets WHERE id=NEW.asset_id AND state='ready') THEN RAISE(ABORT,'historical step media not ready') END;
END;
CREATE TRIGGER step_media_history_update BEFORE UPDATE ON recipe_version_step_media BEGIN SELECT RAISE(ABORT,'immutable step media history'); END;
CREATE TRIGGER step_media_history_delete BEFORE DELETE ON recipe_version_step_media
 WHEN EXISTS(SELECT 1 FROM recipe_versions WHERE recipe_id=OLD.recipe_id AND version_number=OLD.version_number) BEGIN SELECT RAISE(ABORT,'step media history requires owned purge'); END;

-- Extend only in this new migration: formats3/4 cover pins and formats2/3/4
-- alternative pins must remain complete. Existing rows and old projections stay frozen.
DROP TRIGGER media_history_capture;
CREATE TRIGGER media_history_capture AFTER INSERT ON recipe_versions WHEN NEW.format_version IN (3,4) AND json_extract(NEW.snapshot,'$.coverAssetId') IS NOT NULL BEGIN
 INSERT INTO recipe_version_media(recipe_id,version_number,asset_id) VALUES(NEW.recipe_id,NEW.version_number,json_extract(NEW.snapshot,'$.coverAssetId'));
END;
DROP TRIGGER media_history_insert;
CREATE TRIGGER media_history_insert BEFORE INSERT ON recipe_version_media BEGIN
 SELECT CASE WHEN EXISTS(SELECT 1 FROM recipe_version_media WHERE recipe_id=NEW.recipe_id AND version_number=NEW.version_number) OR NOT EXISTS(SELECT 1 FROM recipe_versions WHERE recipe_id=NEW.recipe_id AND version_number=NEW.version_number AND format_version IN (3,4) AND json_extract(snapshot,'$.coverAssetId')=NEW.asset_id) THEN RAISE(ABORT,'invalid media history reference') END;
 SELECT CASE WHEN NOT EXISTS(SELECT 1 FROM media_assets WHERE id=NEW.asset_id AND state='ready') THEN RAISE(ABORT,'media not ready') END;
END;
DROP TRIGGER version_alternatives_validate_insert;
CREATE TRIGGER version_alternatives_validate_insert BEFORE INSERT ON recipe_version_alternatives BEGIN
 SELECT CASE WHEN EXISTS(SELECT 1 FROM recipe_version_alternatives WHERE recipe_id=NEW.recipe_id AND version_number=NEW.version_number AND alternative_id=NEW.alternative_id)
 THEN RAISE(ABORT,'immutable alternative history reference') END;
 SELECT CASE WHEN NOT EXISTS(
 SELECT 1 FROM recipe_versions v,json_each(v.snapshot,'$.ingredients') i,json_each(i.value,'$.alternatives') a
 WHERE v.recipe_id=NEW.recipe_id AND v.version_number=NEW.version_number AND v.format_version IN (2,3,4)
 AND json_extract(i.value,'$.id')=NEW.line_id AND json_extract(a.value,'$.id')=NEW.alternative_id
 AND json_extract(a.value,'$.ingredientId')=NEW.ingredient_id AND json_extract(a.value,'$.unitCode') IS NEW.unit_code)
 THEN RAISE(ABORT,'invalid alternative history reference') END;
END;
CREATE TRIGGER version4_alternatives_capture AFTER INSERT ON recipe_versions WHEN NEW.format_version=4 BEGIN
 INSERT INTO recipe_version_alternatives(recipe_id,version_number,line_id,alternative_id,ingredient_id,unit_code)
 SELECT NEW.recipe_id,NEW.version_number,json_extract(i.value,'$.id'),json_extract(a.value,'$.id'),json_extract(a.value,'$.ingredientId'),json_extract(a.value,'$.unitCode')
 FROM json_each(NEW.snapshot,'$.ingredients') i,json_each(i.value,'$.alternatives') a;
END;
