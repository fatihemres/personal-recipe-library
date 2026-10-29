use super::{
    recipe_search::index_ingredient_recipes,
    recipes::{read_recipe, write_recipe},
    Database,
};
use crate::domain::{recipes::*, reliability::*, AppError};
use rusqlite::{params, OptionalExtension, Transaction, TransactionBehavior};
fn close_draft(tx: &Transaction<'_>, id: &str, revision: i64) -> Result<(), AppError> {
    let count=tx.execute("UPDATE editor_drafts SET state='closed',payload=NULL,revision=revision+1,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1 AND revision=?2 AND state='active'",params![id,revision])?;
    if count != 1 {
        return Err(draft_conflict());
    }
    tx.execute("DELETE FROM draft_ingredients WHERE draft_id=?1", [id])?;
    Ok(())
}
impl Database {
    pub fn draft(&self, id: &str) -> Result<Draft, AppError> {
        valid_id(id)?;
        let tx = self.conn.unchecked_transaction()?;
        let (revision,recipe_id,updated_at,payload):(i64,Option<String>,String,String)=tx.query_row("SELECT revision,recipe_id,updated_at,payload FROM editor_drafts WHERE id=?1 AND state='active'",[id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?))).optional()?.ok_or_else(not_found)?;
        let input: RecipeInput =
            serde_json::from_str(&payload).map_err(|_| AppError::integrity())?;
        let mut names = std::collections::BTreeMap::new();
        for line in &input.ingredients {
            let name = tx.query_row(
                "SELECT name FROM ingredients WHERE id=?1",
                [&line.ingredient_id],
                |r| r.get::<_, String>(0),
            )?;
            names.insert(line.ingredient_id.clone(), name);
        }
        Ok(Draft {
            id: id.into(),
            revision,
            recipe_id,
            updated_at,
            input,
            ingredient_names: names,
        })
    }
    pub fn drafts(&self) -> Result<Vec<Draft>, AppError> {
        let mut s = self.conn.prepare(
            "SELECT id FROM editor_drafts WHERE state='active' ORDER BY updated_at DESC,id",
        )?;
        let ids = s
            .query_map([], |r| r.get::<_, String>(0))?
            .collect::<Result<Vec<_>, _>>()?;
        ids.iter().map(|id| self.draft(id)).collect()
    }
    pub fn save_draft(&mut self, write: DraftWrite) -> Result<Draft, AppError> {
        write.validate()?;
        let payload = serde_json::to_string(&write.input).map_err(|_| invalid())?;
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let target: Option<String> = if write.input.expected_revision.is_some() {
            tx.query_row(
                "SELECT id FROM recipes WHERE id=?1",
                [&write.input.id],
                |r| r.get(0),
            )
            .optional()?
        } else {
            None
        };
        if let Some(revision) = write.expected_revision {
            // A session's recipe target and base revision are immutable, including after a purge or stale edit.
            let old:String=tx.query_row("SELECT payload FROM editor_drafts WHERE id=?1 AND revision=?2 AND state='active'",params![write.id,revision],|r|r.get(0)).optional()?.ok_or_else(draft_conflict)?;
            let previous: RecipeInput =
                serde_json::from_str(&old).map_err(|_| AppError::integrity())?;
            if previous.id != write.input.id
                || previous.expected_revision != write.input.expected_revision
            {
                return Err(draft_conflict());
            }
            tx.execute("UPDATE editor_drafts SET payload=?3,revision=revision+1,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1 AND revision=?2 AND state='active'",params![write.id,revision,payload])?;
        } else {
            if write.input.expected_revision.is_some() && target.is_none() {
                return Err(not_found());
            }
            if tx.query_row(
                "SELECT EXISTS(SELECT 1 FROM editor_drafts WHERE id=?1)",
                [&write.id],
                |r| r.get::<_, bool>(0),
            )? {
                return Err(draft_conflict());
            }
            tx.execute("INSERT INTO editor_drafts(id,recipe_id,revision,state,payload) VALUES(?1,?2,1,'active',?3)",params![write.id,target,payload])?;
        }
        tx.execute(
            "DELETE FROM draft_ingredients WHERE draft_id=?1",
            [&write.id],
        )?;
        for line in &write.input.ingredients {
            if !tx.query_row(
                "SELECT EXISTS(SELECT 1 FROM units WHERE code=?1)",
                [&line.unit_code],
                |r| r.get::<_, bool>(0),
            )? {
                return Err(invalid());
            }
            tx.execute(
                "INSERT OR IGNORE INTO draft_ingredients(draft_id,ingredient_id) VALUES(?1,?2)",
                params![write.id, line.ingredient_id],
            )?;
        }
        tx.commit()?;
        self.draft(&write.id)
    }
    pub fn discard_draft(&mut self, id: &str, revision: i64) -> Result<(), AppError> {
        valid_id(id)?;
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        close_draft(&tx, id, revision)?;
        tx.commit()?;
        Ok(())
    }
    pub fn commit_recipe(
        &mut self,
        input: RecipeInput,
        draft_id: &str,
        draft_revision: i64,
        as_copy: bool,
    ) -> Result<Recipe, AppError> {
        input.validate()?;
        valid_id(draft_id)?;
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let payload: String = tx
            .query_row(
                "SELECT payload FROM editor_drafts WHERE id=?1 AND revision=?2 AND state='active'",
                params![draft_id, draft_revision],
                |r| r.get(0),
            )
            .optional()?
            .ok_or_else(draft_conflict)?;
        let saved: RecipeInput =
            serde_json::from_str(&payload).map_err(|_| AppError::integrity())?;
        if saved.id != input.id || saved.expected_revision != input.expected_revision {
            return Err(draft_conflict());
        }
        let input = if as_copy { independent(input) } else { input };
        write_recipe(&tx, &input)?;
        close_draft(&tx, draft_id, draft_revision)?;
        tx.commit()?;
        self.recipe(&input.id)
    }
    pub fn duplicate_recipe(&mut self, id: &str, revision: i64) -> Result<Recipe, AppError> {
        valid_id(id)?;
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let source = read_recipe(&tx, id)?;
        if source.revision != revision || source.deleted_at.is_some() {
            return Err(conflict());
        }
        let input = independent(source.to_input());
        write_recipe(&tx, &input)?;
        // A duplicated recipe is a new personal copy; its private note starts with
        // the same text but has its own revision and future edits are independent.
        tx.execute("INSERT INTO recipe_private_notes(recipe_id,format_version,markdown) SELECT ?1,format_version,markdown FROM recipe_private_notes WHERE recipe_id=?2",params![input.id,id])?;
        tx.commit()?;
        self.recipe(&input.id)
    }
    pub fn scope_recipes(
        &self,
        scope: &str,
        kind: Option<String>,
    ) -> Result<Vec<Recipe>, AppError> {
        if !["active", "archived", "trash"].contains(&scope)
            || kind
                .as_ref()
                .is_some_and(|k| !["food", "beverage"].contains(&k.as_str()))
        {
            return Err(invalid());
        }
        let mut s=self.conn.prepare("SELECT id FROM recipes WHERE (?1='trash' AND deleted_at IS NOT NULL OR ?1='active' AND deleted_at IS NULL AND archived_at IS NULL OR ?1='archived' AND deleted_at IS NULL AND archived_at IS NOT NULL) AND (?2 IS NULL OR kind=?2) ORDER BY updated_at DESC,id")?;
        let ids = s
            .query_map(params![scope, kind], |r| r.get::<_, String>(0))?
            .collect::<Result<Vec<_>, _>>()?;
        ids.iter().map(|id| self.recipe(id)).collect()
    }
    pub fn archive_recipe(
        &mut self,
        id: &str,
        revision: i64,
        archived: bool,
    ) -> Result<Recipe, AppError> {
        valid_id(id)?;
        let count=self.conn.execute("UPDATE recipes SET archived_at=CASE WHEN ?3 THEN strftime('%Y-%m-%dT%H:%M:%fZ','now') ELSE NULL END,revision=revision+1,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1 AND revision=?2 AND deleted_at IS NULL AND (archived_at IS NULL)=?3",params![id,revision,archived])?;
        if count != 1 {
            return Err(conflict());
        }
        self.recipe(id)
    }
    pub fn purge_recipe(&mut self, id: &str, revision: i64) -> Result<(), AppError> {
        valid_id(id)?;
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let eligible:bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM recipes WHERE id=?1 AND revision=?2 AND deleted_at IS NOT NULL)",params![id,revision],|r|r.get(0))?;
        if !eligible {
            return Err(conflict());
        }
        super::recipe_versions::check_purge_format(&tx, id)?;
        if tx.execute(
            "DELETE FROM recipes WHERE id=?1 AND revision=?2 AND deleted_at IS NOT NULL",
            params![id, revision],
        )? != 1
        {
            return Err(conflict());
        }
        tx.commit()?;
        Ok(())
    }
    pub fn search_personal(&self, query: &str) -> Result<IngredientSearch, AppError> {
        if query.chars().count() > 200 || query.contains('\0') {
            return Err(invalid());
        }
        let tx = self.conn.unchecked_transaction()?;
        let total = tx.query_row(
            "SELECT count(*) FROM ingredients WHERE origin='personal'",
            [],
            |r| r.get(0),
        )?;
        let mut s=tx.prepare("SELECT id,name,notes,preferred_unit,revision,origin FROM ingredients WHERE origin='personal' AND instr(search_name,?1)>0 ORDER BY (search_name=?1) DESC,search_name,id LIMIT 51")?;
        let mut items = s
            .query_map([search_name(query)], |r| {
                Ok(PersonalIngredient {
                    id: r.get(0)?,
                    name: r.get(1)?,
                    notes: r.get(2)?,
                    preferred_unit: r.get(3)?,
                    revision: r.get(4)?,
                    origin: r.get(5)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        let has_more = items.len() > 50;
        items.truncate(50);
        Ok(IngredientSearch {
            items,
            total,
            has_more,
        })
    }
    pub fn edit_ingredient(
        &mut self,
        input: IngredientEdit,
    ) -> Result<PersonalIngredient, AppError> {
        valid_id(&input.id)?;
        let name = input.name.trim();
        let key = search_name(name);
        let categories = input.category_ids.as_deref();
        if key.is_empty()
            || name.chars().count() > 200
            || name.contains('\0')
            || input
                .notes
                .as_ref()
                .is_some_and(|n| n.chars().count() > 2000 || n.contains('\0'))
            || categories.is_some_and(|c| c.len() > 50)
        {
            return Err(invalid());
        }
        if let Some(ids) = categories {
            let mut unique = std::collections::HashSet::new();
            for id in ids {
                valid_id(id)?;
                if !unique.insert(id) {
                    return Err(invalid());
                }
            }
        }
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        if let Some(ids) = categories {
            for id in ids {
                if !tx.query_row(
                    "SELECT EXISTS(SELECT 1 FROM ingredient_categories WHERE id=?1)",
                    [id],
                    |r| r.get::<_, bool>(0),
                )? {
                    return Err(invalid());
                }
            }
        }
        if tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM ingredients WHERE search_name=?1 AND id<>?2)",
            params![key, input.id],
            |r| r.get::<_, bool>(0),
        )? {
            return Err(AppError::new(
                "INGREDIENT_DUPLICATE",
                "errors.ingredientDuplicate",
                true,
            ));
        }
        if tx.execute("UPDATE ingredients SET name=?3,search_name=?4,notes=?5,preferred_unit=?6,revision=revision+1,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1 AND revision=?2 AND origin='personal'",params![input.id,input.revision,name,key,input.notes,input.preferred_unit])?!=1 {return Err(conflict());}
        index_ingredient_recipes(&tx, &input.id)?;
        if let Some(ids) = categories {
            tx.execute(
                "DELETE FROM personal_category_memberships WHERE ingredient_id=?1",
                [&input.id],
            )?;
            for id in ids {
                tx.execute("INSERT INTO personal_category_memberships(ingredient_id,category_id) VALUES(?1,?2)",params![input.id,id])?;
            }
        }
        let result = tx.query_row(
            "SELECT id,name,notes,preferred_unit,revision,origin FROM ingredients WHERE id=?1",
            [&input.id],
            |r| {
                Ok(PersonalIngredient {
                    id: r.get(0)?,
                    name: r.get(1)?,
                    notes: r.get(2)?,
                    preferred_unit: r.get(3)?,
                    revision: r.get(4)?,
                    origin: r.get(5)?,
                })
            },
        )?;
        tx.commit()?;
        Ok(result)
    }
    pub fn delete_ingredient(&mut self, id: &str, revision: i64) -> Result<(), AppError> {
        valid_id(id)?;
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let referenced:bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM recipe_ingredients WHERE ingredient_id=?1) OR EXISTS(SELECT 1 FROM draft_ingredients WHERE ingredient_id=?1) OR EXISTS(SELECT 1 FROM recipe_version_ingredients WHERE ingredient_id=?1)",[id],|r|r.get(0))?;
        if referenced {
            return Err(AppError::new(
                "INGREDIENT_REFERENCED",
                "errors.ingredientReferenced",
                true,
            ));
        }
        // Personal category links are user-owned; they must be removed within
        // the same transaction before the restrictive ingredient FK is checked.
        let current: Option<i64> = tx
            .query_row(
                "SELECT revision FROM ingredients WHERE id=?1 AND origin='personal'",
                [id],
                |r| r.get(0),
            )
            .optional()?;
        if current != Some(revision) {
            return Err(conflict());
        }
        tx.execute(
            "DELETE FROM personal_category_memberships WHERE ingredient_id=?1",
            [id],
        )?;
        if tx.execute(
            "DELETE FROM ingredients WHERE id=?1 AND revision=?2 AND origin='personal'",
            params![id, revision],
        )? != 1
        {
            return Err(conflict());
        }
        tx.commit()?;
        Ok(())
    }
}
