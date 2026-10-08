use super::Database;
use crate::domain::{recipes::*, AppError};
use rusqlite::{params, OptionalExtension, TransactionBehavior};
impl Database {
    pub fn units(&self) -> Result<Vec<Unit>, AppError> {
        let mut s = self
            .conn
            .prepare("SELECT code,dimension,canonical_code,factor FROM units ORDER BY rowid")?;
        let rows = s.query_map([], |r| {
            Ok(Unit {
                code: r.get(0)?,
                dimension: r.get(1)?,
                canonical_code: r.get(2)?,
                factor: r.get(3)?,
            })
        })?;
        Ok(rows.collect::<Result<_, _>>()?)
    }
    pub fn ingredients(&self, query: &str) -> Result<Vec<Ingredient>, AppError> {
        if query.chars().count() > 200 {
            return Err(invalid());
        }
        let mut s=self.conn.prepare("SELECT id,name FROM ingredients WHERE instr(search_name,?1)>0 ORDER BY search_name,id LIMIT 50")?;
        let rows = s.query_map([search_name(query)], |r| {
            Ok(Ingredient {
                id: r.get(0)?,
                name: r.get(1)?,
            })
        })?;
        Ok(rows.collect::<Result<_, _>>()?)
    }
    pub fn create_ingredient(&mut self, name: &str) -> Result<Ingredient, AppError> {
        let name = name.trim().to_string();
        let key = search_name(&name);
        if key.is_empty() || name.chars().count() > 200 || name.contains('\0') {
            return Err(invalid());
        }
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let existing = tx
            .query_row(
                "SELECT id,name FROM ingredients WHERE search_name=?1 ORDER BY origin DESC LIMIT 1",
                [&key],
                |r| {
                    Ok(Ingredient {
                        id: r.get(0)?,
                        name: r.get(1)?,
                    })
                },
            )
            .optional()?;
        let result = if let Some(i) = existing {
            i
        } else {
            let i = Ingredient {
                id: uuid::Uuid::new_v4().to_string(),
                name,
            };
            tx.execute(
                "INSERT INTO ingredients(id,name,search_name) VALUES(?1,?2,?3)",
                params![i.id, i.name, key],
            )?;
            i
        };
        tx.commit()?;
        Ok(result)
    }
    pub fn list_recipes(&self, trash: bool, kind: Option<String>) -> Result<Vec<Recipe>, AppError> {
        if kind
            .as_ref()
            .is_some_and(|k| !["food", "beverage"].contains(&k.as_str()))
        {
            return Err(invalid());
        }
        let mut s=self.conn.prepare("SELECT id FROM recipes WHERE (deleted_at IS NOT NULL)=?1 AND (?1 OR archived_at IS NULL) AND (?2 IS NULL OR kind=?2) ORDER BY updated_at DESC,id")?;
        let ids = s
            .query_map(params![trash, kind], |r| r.get::<_, String>(0))?
            .collect::<Result<Vec<_>, _>>()?;
        ids.iter().map(|id| self.recipe(id)).collect()
    }
    pub fn recipe(&self, id: &str) -> Result<Recipe, AppError> {
        valid_id(id)?;
        let tx = self.conn.unchecked_transaction()?;
        read_recipe(&tx, id)
    }
    pub fn save_recipe(&mut self, input: RecipeInput) -> Result<Recipe, AppError> {
        input.validate()?;
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        write_recipe(&tx, &input)?;
        tx.commit()?;
        self.recipe(&input.id)
    }
    pub fn set_deleted(
        &mut self,
        id: &str,
        revision: i64,
        deleted: bool,
    ) -> Result<Recipe, AppError> {
        valid_id(id)?;
        let count=self.conn.execute("UPDATE recipes SET deleted_at=CASE WHEN ?3 THEN strftime('%Y-%m-%dT%H:%M:%fZ','now') ELSE NULL END,revision=revision+1,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1 AND revision=?2 AND (deleted_at IS NULL)=?3",params![id,revision,deleted])?;
        if count != 1 {
            return Err(conflict());
        }
        self.recipe(id)
    }
}

pub(super) fn read_recipe(conn: &rusqlite::Connection, id: &str) -> Result<Recipe, AppError> {
    let mut recipe=conn.query_row("SELECT id,revision,title,description,kind,servings,prep_minutes,cook_minutes,notes,created_at,updated_at,deleted_at,archived_at FROM recipes WHERE id=?1",[id],|r|Ok(Recipe{archived_at:r.get(12)?,ingredient_names:Default::default(),id:r.get(0)?,revision:r.get(1)?,title:r.get(2)?,description:r.get(3)?,kind:r.get(4)?,servings:r.get(5)?,prep_minutes:r.get(6)?,cook_minutes:r.get(7)?,notes:r.get(8)?,created_at:r.get(9)?,updated_at:r.get(10)?,deleted_at:r.get(11)?,ingredients:vec![],steps:vec![]})).optional()?.ok_or_else(not_found)?;
    let mut s=conn.prepare("SELECT id,ingredient_id,quantity,unit_code,note FROM recipe_ingredients WHERE recipe_id=?1 ORDER BY position")?;
    recipe.ingredients = s
        .query_map([id], |r| {
            Ok(RecipeIngredient {
                id: r.get(0)?,
                ingredient_id: r.get(1)?,
                quantity: r.get(2)?,
                unit_code: r.get(3)?,
                note: r.get(4)?,
            })
        })?
        .collect::<Result<_, _>>()?;
    let mut s = conn
        .prepare("SELECT id,instructions FROM recipe_steps WHERE recipe_id=?1 ORDER BY position")?;
    recipe.steps = s
        .query_map([id], |r| {
            Ok(RecipeStep {
                id: r.get(0)?,
                instructions: r.get(1)?,
            })
        })?
        .collect::<Result<_, _>>()?;
    for line in &recipe.ingredients {
        let name: String = conn.query_row(
            "SELECT name FROM ingredients WHERE id=?1",
            [&line.ingredient_id],
            |r| r.get(0),
        )?;
        recipe
            .ingredient_names
            .insert(line.ingredient_id.clone(), name);
    }
    Ok(recipe)
}

pub(super) fn write_recipe(
    tx: &rusqlite::Transaction<'_>,
    input: &RecipeInput,
) -> Result<(), AppError> {
    if let Some(revision) = input.expected_revision {
        let count=tx.execute("UPDATE recipes SET title=?2,description=?3,kind=?4,servings=?5,prep_minutes=?6,cook_minutes=?7,notes=?8,revision=revision+1,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1 AND revision=?9 AND deleted_at IS NULL AND archived_at IS NULL",params![input.id,input.title.trim(),input.description,input.kind,input.servings,input.prep_minutes,input.cook_minutes,input.notes,revision])?;
        if count != 1 {
            return Err(conflict());
        }
    } else {
        if tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM recipes WHERE id=?1)",
            [&input.id],
            |r| r.get::<_, bool>(0),
        )? {
            return Err(conflict());
        }
        tx.execute("INSERT INTO recipes(id,title,description,kind,servings,prep_minutes,cook_minutes,notes) VALUES(?1,?2,?3,?4,?5,?6,?7,?8)",params![input.id,input.title.trim(),input.description,input.kind,input.servings,input.prep_minutes,input.cook_minutes,input.notes])?;
    }
    tx.execute(
        "DELETE FROM recipe_ingredients WHERE recipe_id=?1",
        [&input.id],
    )?;
    tx.execute("DELETE FROM recipe_steps WHERE recipe_id=?1", [&input.id])?;
    for (position, i) in input.ingredients.iter().enumerate() {
        let valid=tx.query_row("SELECT EXISTS(SELECT 1 FROM ingredients WHERE id=?1) AND EXISTS(SELECT 1 FROM units WHERE code=?2)",params![i.ingredient_id,i.unit_code],|r|r.get::<_,bool>(0))?;
        if !valid {
            return Err(invalid());
        }
        tx.execute("INSERT INTO recipe_ingredients(id,recipe_id,ingredient_id,quantity,unit_code,position,note) VALUES(?1,?2,?3,?4,?5,?6,?7)",params![i.id,input.id,i.ingredient_id,i.quantity,i.unit_code,position as i64,i.note])?;
    }
    for (position, s) in input.steps.iter().enumerate() {
        tx.execute(
            "INSERT INTO recipe_steps(id,recipe_id,position,instructions) VALUES(?1,?2,?3,?4)",
            params![s.id, input.id, position as i64, s.instructions.trim()],
        )?;
    }
    Ok(())
}
