use super::Database;
use crate::domain::{
    recipe_search::{RecipeSearchItem, RecipeSearchQuery},
    recipe_workspace::{Dashboard, PrivateNote, PrivateNoteWrite, SavedFilter, SavedFilterWrite},
    recipes::{conflict, invalid, not_found, search_name, valid_id},
    AppError,
};
use rusqlite::{params, OptionalExtension, TransactionBehavior};

impl Database {
    pub fn private_note(&self, recipe_id: &str) -> Result<PrivateNote, AppError> {
        valid_id(recipe_id)?;
        let exists: bool = self.conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM recipes WHERE id=?1)",
            [recipe_id],
            |r| r.get(0),
        )?;
        if !exists {
            return Err(not_found());
        }
        self.conn.query_row("SELECT format_version,markdown,revision,updated_at FROM recipe_private_notes WHERE recipe_id=?1", [recipe_id], |r| Ok(PrivateNote {
            recipe_id: recipe_id.to_owned(), format_version:r.get(0)?,markdown:r.get(1)?,revision:r.get(2)?,updated_at:r.get(3)?
        })).optional()?.map_or_else(|| Ok(PrivateNote {recipe_id:recipe_id.to_owned(),format_version:1,markdown:String::new(),revision:0,updated_at:None}),Ok)
    }

    pub fn save_private_note(&mut self, write: PrivateNoteWrite) -> Result<PrivateNote, AppError> {
        write.validate()?;
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let exists: bool = tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM recipes WHERE id=?1 AND deleted_at IS NULL)",
            [&write.recipe_id],
            |r| r.get(0),
        )?;
        if !exists {
            return Err(not_found());
        }
        let changed = if write.expected_revision == 0 {
            tx.execute("INSERT INTO recipe_private_notes(recipe_id,markdown) VALUES(?1,?2) ON CONFLICT(recipe_id) DO NOTHING", params![write.recipe_id,write.markdown])?
        } else {
            tx.execute("UPDATE recipe_private_notes SET markdown=?2,revision=revision+1,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE recipe_id=?1 AND revision=?3",params![write.recipe_id,write.markdown,write.expected_revision])?
        };
        if changed != 1 {
            return Err(conflict());
        }
        tx.commit()?;
        self.private_note(&write.recipe_id)
    }

    pub fn saved_filters(&self) -> Result<Vec<SavedFilter>, AppError> {
        let mut stmt = self.conn.prepare("SELECT id,title,revision,format_version,query_json FROM saved_recipe_filters ORDER BY title_key,id")?;
        let rows = stmt
            .query_map([], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, i64>(2)?,
                    r.get::<_, i64>(3)?,
                    r.get::<_, String>(4)?,
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        let mut result = Vec::with_capacity(rows.len());
        for (id, title, revision, format_version, json) in rows {
            let query: Option<RecipeSearchQuery> = if format_version == 1 {
                serde_json::from_str(&json).ok()
            } else {
                None
            };
            let available = if let Some(q) = &query {
                q.validate().is_ok() && self.filter_refs_exist(q)?
            } else {
                false
            };
            result.push(SavedFilter {
                id,
                title,
                revision,
                format_version,
                query,
                available,
            });
        }
        Ok(result)
    }

    pub fn resolve_filter(&self, id: &str) -> Result<RecipeSearchQuery, AppError> {
        valid_id(id)?;
        let row: Option<(i64, String)> = self
            .conn
            .query_row(
                "SELECT format_version,query_json FROM saved_recipe_filters WHERE id=?1",
                [id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()?;
        let Some((version, json)) = row else {
            return Err(not_found());
        };
        if version != 1 {
            return Err(AppError::new(
                "FILTER_VERSION_UNSUPPORTED",
                "errors.filterVersion",
                true,
            ));
        }
        let query: RecipeSearchQuery = serde_json::from_str(&json)
            .map_err(|_| AppError::new("FILTER_INVALID", "errors.filterInvalid", true))?;
        query
            .validate()
            .map_err(|_| AppError::new("FILTER_INVALID", "errors.filterInvalid", true))?;
        if !self.filter_refs_exist(&query)? {
            return Err(AppError::new(
                "FILTER_REFERENCE_MISSING",
                "errors.filterReference",
                true,
            ));
        }
        Ok(query)
    }

    fn filter_refs_exist(&self, query: &RecipeSearchQuery) -> Result<bool, AppError> {
        if let Some(id) = &query.ingredient_id {
            let present: bool = self.conn.query_row(
                "SELECT EXISTS(SELECT 1 FROM ingredients WHERE id=?1)",
                [id],
                |r| r.get(0),
            )?;
            if !present {
                return Ok(false);
            }
        }
        if let Some(id) = &query.collection_id {
            let present: bool = self.conn.query_row(
                "SELECT EXISTS(SELECT 1 FROM recipe_collections WHERE id=?1)",
                [id],
                |r| r.get(0),
            )?;
            if !present {
                return Ok(false);
            }
        }
        Ok(true)
    }

    pub fn save_filter(&mut self, write: SavedFilterWrite) -> Result<SavedFilter, AppError> {
        write.validate()?;
        let title = write.title.trim().to_owned();
        let key = search_name(&title);
        let json = serde_json::to_string(&write.query).map_err(|_| invalid())?;
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let duplicate: bool = tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM saved_recipe_filters WHERE title_key=?1 AND id<>?2)",
            params![key, write.id],
            |r| r.get(0),
        )?;
        if duplicate {
            return Err(AppError::new(
                "FILTER_DUPLICATE",
                "errors.filterDuplicate",
                true,
            ));
        }
        let changed = if let Some(revision) = write.expected_revision {
            tx.execute("UPDATE saved_recipe_filters SET title=?2,title_key=?3,query_json=?4,revision=revision+1,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1 AND revision=?5 AND format_version=1",params![write.id,title,key,json,revision])?
        } else {
            tx.execute("INSERT INTO saved_recipe_filters(id,title,title_key,query_json) VALUES(?1,?2,?3,?4) ON CONFLICT(id) DO NOTHING",params![write.id,title,key,json])?
        };
        if changed != 1 {
            return Err(conflict());
        }
        tx.commit()?;
        self.saved_filters()?
            .into_iter()
            .find(|f| f.id == write.id)
            .ok_or_else(not_found)
    }

    pub fn delete_filter(&mut self, id: &str, revision: i64) -> Result<(), AppError> {
        valid_id(id)?;
        if revision < 1 {
            return Err(invalid());
        }
        let changed = self.conn.execute(
            "DELETE FROM saved_recipe_filters WHERE id=?1 AND revision=?2",
            params![id, revision],
        )?;
        if changed != 1 {
            return Err(conflict());
        }
        Ok(())
    }

    pub fn dashboard(&self) -> Result<Dashboard, AppError> {
        let (active_total,food_count,beverage_count,favorite_count)=self.conn.query_row(
            "SELECT count(*),coalesce(sum(CASE WHEN r.kind='food' THEN 1 ELSE 0 END),0),coalesce(sum(CASE WHEN r.kind='beverage' THEN 1 ELSE 0 END),0),coalesce(sum(CASE WHEN o.favorite=1 THEN 1 ELSE 0 END),0) FROM recipes r JOIN recipe_organization o ON o.recipe_id=r.id WHERE r.deleted_at IS NULL AND r.archived_at IS NULL",[],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?)))?;
        Ok(Dashboard {
            active_total,
            food_count,
            beverage_count,
            favorite_count,
            recent_created: self.dashboard_recent("created_at")?,
            recent_modified: self.dashboard_recent("updated_at")?,
            collections: self.collections()?,
        })
    }
    fn dashboard_recent(&self, column: &str) -> Result<Vec<RecipeSearchItem>, AppError> {
        // Column is selected only from two internal literals; it never contains user input.
        let sql=format!("SELECT r.id,r.title,r.description,r.kind,r.servings,r.created_at,r.updated_at,o.revision,o.favorite,o.to_try,o.rating FROM recipes r JOIN recipe_organization o ON o.recipe_id=r.id WHERE r.deleted_at IS NULL AND r.archived_at IS NULL ORDER BY r.{column} DESC,r.id LIMIT 5");
        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt
            .query_map([], |r| {
                Ok(RecipeSearchItem {
                    id: r.get(0)?,
                    title: r.get(1)?,
                    description: r.get(2)?,
                    kind: r.get(3)?,
                    servings: r.get(4)?,
                    created_at: r.get(5)?,
                    updated_at: r.get(6)?,
                    organization_revision: r.get(7)?,
                    favorite: r.get(8)?,
                    to_try: r.get(9)?,
                    rating: r.get(10)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }
}
