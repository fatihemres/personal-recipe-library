CREATE TABLE recipe_graph_state (
 recipe_id TEXT PRIMARY KEY REFERENCES recipes(id) ON DELETE CASCADE,
 revision INTEGER NOT NULL DEFAULT 1 CHECK(revision>0)
) STRICT;
INSERT INTO recipe_graph_state(recipe_id) SELECT id FROM recipes;
CREATE TRIGGER recipe_graph_initialize AFTER INSERT ON recipes BEGIN
 INSERT INTO recipe_graph_state(recipe_id) VALUES(NEW.id);
END;
CREATE TABLE recipe_variations (
 child_id TEXT PRIMARY KEY REFERENCES recipes(id) ON DELETE RESTRICT,
 parent_id TEXT NOT NULL REFERENCES recipes(id) ON DELETE RESTRICT,
 CHECK(child_id<>parent_id)
) STRICT;
CREATE INDEX recipe_variation_parent ON recipe_variations(parent_id,child_id);
CREATE TABLE recipe_related (
 left_id TEXT NOT NULL REFERENCES recipes(id) ON DELETE RESTRICT,
 right_id TEXT NOT NULL REFERENCES recipes(id) ON DELETE RESTRICT,
 PRIMARY KEY(left_id,right_id), CHECK(left_id<right_id)
) STRICT;
CREATE INDEX recipe_related_right ON recipe_related(right_id,left_id);
CREATE TRIGGER variation_cycle_insert BEFORE INSERT ON recipe_variations BEGIN
 SELECT CASE WHEN EXISTS(WITH RECURSIVE ancestors(id) AS (
  SELECT NEW.parent_id UNION SELECT v.parent_id FROM recipe_variations v JOIN ancestors a ON v.child_id=a.id
 ) SELECT 1 FROM ancestors WHERE id=NEW.child_id) THEN RAISE(ABORT,'variation cycle') END;
END;
CREATE TRIGGER variation_cycle_update BEFORE UPDATE ON recipe_variations BEGIN
 SELECT CASE WHEN EXISTS(WITH RECURSIVE ancestors(id) AS (
  SELECT NEW.parent_id UNION SELECT v.parent_id FROM recipe_variations v JOIN ancestors a ON v.child_id=a.id WHERE v.child_id<>OLD.child_id
 ) SELECT 1 FROM ancestors WHERE id=NEW.child_id) THEN RAISE(ABORT,'variation cycle') END;
END;
CREATE TRIGGER variation_insert_revision AFTER INSERT ON recipe_variations BEGIN
 UPDATE recipe_graph_state SET revision=revision+1 WHERE recipe_id IN (NEW.child_id,NEW.parent_id);
END;
CREATE TRIGGER variation_delete_revision AFTER DELETE ON recipe_variations BEGIN
 UPDATE recipe_graph_state SET revision=revision+1 WHERE recipe_id IN (OLD.child_id,OLD.parent_id);
END;
CREATE TRIGGER variation_update_revision AFTER UPDATE ON recipe_variations BEGIN
 UPDATE recipe_graph_state SET revision=revision+1 WHERE recipe_id IN (OLD.child_id,OLD.parent_id,NEW.child_id,NEW.parent_id);
END;
CREATE TRIGGER related_insert_revision AFTER INSERT ON recipe_related BEGIN
 UPDATE recipe_graph_state SET revision=revision+1 WHERE recipe_id IN (NEW.left_id,NEW.right_id);
END;
CREATE TRIGGER related_delete_revision AFTER DELETE ON recipe_related BEGIN
 UPDATE recipe_graph_state SET revision=revision+1 WHERE recipe_id IN (OLD.left_id,OLD.right_id);
END;
CREATE TRIGGER related_update_revision AFTER UPDATE ON recipe_related BEGIN
 UPDATE recipe_graph_state SET revision=revision+1 WHERE recipe_id IN (OLD.left_id,OLD.right_id,NEW.left_id,NEW.right_id);
END;
