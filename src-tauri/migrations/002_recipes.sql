CREATE TABLE units (
 code TEXT PRIMARY KEY, dimension TEXT NOT NULL CHECK(dimension IN ('mass','volume','count')),
 canonical_code TEXT NOT NULL REFERENCES units(code), factor INTEGER NOT NULL CHECK(factor > 0)
) STRICT;
INSERT INTO units VALUES ('g','mass','g',1),('kg','mass','g',1000),('mL','volume','mL',1),('cc','volume','mL',1),('L','volume','mL',1000),('adet','count','adet',1);
CREATE TABLE ingredients (
 id TEXT PRIMARY KEY CHECK(length(id)=36), name TEXT NOT NULL CHECK(length(trim(name)) BETWEEN 1 AND 200),
 search_name TEXT NOT NULL, origin TEXT NOT NULL DEFAULT 'personal' CHECK(origin IN ('personal','catalog')),
 created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
) STRICT;
CREATE UNIQUE INDEX personal_ingredient_name ON ingredients(search_name) WHERE origin='personal';
CREATE TABLE recipes (
 id TEXT PRIMARY KEY CHECK(length(id)=36), title TEXT NOT NULL CHECK(length(trim(title)) BETWEEN 1 AND 200),
 description TEXT CHECK(length(description)<=10000), kind TEXT NOT NULL CHECK(kind IN ('food','beverage')),
 servings TEXT NOT NULL CHECK(length(servings) BETWEEN 1 AND 19 AND servings NOT GLOB '*[^0-9.]*' AND servings GLOB '*[1-9]*'
 AND servings NOT LIKE '.%' AND servings NOT LIKE '%.' AND servings NOT LIKE '%.%.%'
 AND (servings NOT LIKE '0%' OR servings LIKE '0.%')
 AND length(CASE WHEN instr(servings,'.')=0 THEN servings ELSE substr(servings,1,instr(servings,'.')-1) END)<=12
 AND (instr(servings,'.')=0 OR length(servings)-instr(servings,'.')<=6)),
 prep_minutes INTEGER CHECK(prep_minutes BETWEEN 0 AND 10080), cook_minutes INTEGER CHECK(cook_minutes BETWEEN 0 AND 10080),
 notes TEXT CHECK(length(notes)<=20000), revision INTEGER NOT NULL DEFAULT 1 CHECK(revision>0),
 created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
 updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')), deleted_at TEXT
) STRICT;
CREATE INDEX recipes_listing ON recipes(deleted_at,kind,updated_at);
CREATE TABLE recipe_ingredients (
 id TEXT PRIMARY KEY CHECK(length(id)=36), recipe_id TEXT NOT NULL REFERENCES recipes(id) ON DELETE CASCADE,
 ingredient_id TEXT NOT NULL REFERENCES ingredients(id) ON DELETE RESTRICT,
 quantity TEXT CHECK(quantity IS NULL OR (length(quantity) BETWEEN 1 AND 19 AND quantity NOT GLOB '*[^0-9.]*' AND quantity GLOB '*[1-9]*'
 AND quantity NOT LIKE '.%' AND quantity NOT LIKE '%.' AND quantity NOT LIKE '%.%.%'
 AND (quantity NOT LIKE '0%' OR quantity LIKE '0.%')
 AND length(CASE WHEN instr(quantity,'.')=0 THEN quantity ELSE substr(quantity,1,instr(quantity,'.')-1) END)<=12
 AND (instr(quantity,'.')=0 OR length(quantity)-instr(quantity,'.')<=6))),
 unit_code TEXT NOT NULL REFERENCES units(code) ON DELETE RESTRICT,
 position INTEGER NOT NULL CHECK(position>=0), note TEXT CHECK(length(note)<=2000),
 UNIQUE(recipe_id,position)
) STRICT;
CREATE INDEX recipe_ingredient_reference ON recipe_ingredients(ingredient_id);
CREATE TABLE recipe_steps (
 id TEXT PRIMARY KEY CHECK(length(id)=36), recipe_id TEXT NOT NULL REFERENCES recipes(id) ON DELETE CASCADE,
 position INTEGER NOT NULL CHECK(position>=0), instructions TEXT NOT NULL CHECK(length(trim(instructions)) BETWEEN 1 AND 10000),
 UNIQUE(recipe_id,position)
) STRICT;
