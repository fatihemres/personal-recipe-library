ALTER TABLE recipe_steps ADD COLUMN details_version INTEGER CHECK(details_version IS NULL OR details_version=1);
ALTER TABLE recipe_steps ADD COLUMN title TEXT CHECK(title IS NULL OR length(title)<=200);
ALTER TABLE recipe_steps ADD COLUMN duration_minutes INTEGER CHECK(duration_minutes IS NULL OR duration_minutes BETWEEN 0 AND 10080);
ALTER TABLE recipe_steps ADD COLUMN temperature_value TEXT CHECK(temperature_value IS NULL OR length(temperature_value) BETWEEN 1 AND 5);
ALTER TABLE recipe_steps ADD COLUMN temperature_unit TEXT CHECK(temperature_unit IS NULL OR temperature_unit IN ('C','F'));
ALTER TABLE recipe_steps ADD COLUMN heat_level TEXT CHECK(heat_level IS NULL OR length(heat_level)<=200);
ALTER TABLE recipe_steps ADD COLUMN equipment TEXT CHECK(equipment IS NULL OR length(equipment)<=1000);
ALTER TABLE recipe_steps ADD COLUMN technique TEXT CHECK(technique IS NULL OR length(technique)<=1000);
ALTER TABLE recipe_steps ADD COLUMN tips TEXT CHECK(tips IS NULL OR length(tips)<=5000)
 CHECK((temperature_value IS NULL)=(temperature_unit IS NULL))
 CHECK(temperature_value IS NULL OR (
   temperature_value NOT GLOB '*[^0-9-]*' AND temperature_value NOT IN ('','-')
   AND instr(substr(temperature_value,2),'-')=0
   AND CAST(temperature_value AS INTEGER) BETWEEN -999 AND 9999))
 CHECK(details_version IS NOT NULL OR (title IS NULL AND duration_minutes IS NULL
   AND temperature_value IS NULL AND temperature_unit IS NULL AND heat_level IS NULL
   AND equipment IS NULL AND technique IS NULL AND tips IS NULL));
CREATE UNIQUE INDEX recipe_step_owner ON recipe_steps(recipe_id,id);
CREATE UNIQUE INDEX recipe_line_owner ON recipe_ingredients(recipe_id,id);
CREATE TABLE recipe_step_ingredients (
 recipe_id TEXT NOT NULL,
 step_id TEXT NOT NULL,
 line_id TEXT NOT NULL,
 PRIMARY KEY(step_id,line_id),
 FOREIGN KEY(recipe_id,step_id) REFERENCES recipe_steps(recipe_id,id) ON DELETE CASCADE,
 FOREIGN KEY(recipe_id,line_id) REFERENCES recipe_ingredients(recipe_id,id) ON DELETE CASCADE
) STRICT;
CREATE INDEX recipe_step_line_reference ON recipe_step_ingredients(recipe_id,line_id);
