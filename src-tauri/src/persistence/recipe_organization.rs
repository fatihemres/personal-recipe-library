use super::Database;
use crate::domain::{
    recipe_organization::{
        CollectionWrite, OrganizationWrite, RecipeCollection, RecipeOrganization,
    },
    recipes::{conflict, invalid, not_found, search_name, valid_id},
    AppError,
};
use rusqlite::{params, OptionalExtension, TransactionBehavior};

impl Database {
    pub fn recipe_organization(&self, id: &str) -> Result<RecipeOrganization, AppError> {
        valid_id(id)?;
        let tx = self.conn.unchecked_transaction()?;
        let (revision, favorite, to_try, rating) = tx.query_row(
            "SELECT revision,favorite,to_try,rating FROM recipe_organization WHERE recipe_id=?1",
            [id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        ).optional()?.ok_or_else(not_found)?;
        let mut stmt = tx.prepare("SELECT collection_id FROM recipe_collection_members WHERE recipe_id=?1 ORDER BY collection_id")?;
        let collection_ids = stmt
            .query_map([id], |r| r.get::<_, String>(0))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(RecipeOrganization {
            recipe_id: id.into(),
            revision,
            favorite,
            to_try,
            rating,
            collection_ids,
        })
    }

    pub fn set_recipe_organization(
        &mut self,
        write: OrganizationWrite,
    ) -> Result<RecipeOrganization, AppError> {
        write.validate()?;
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let old: Option<(i64, bool, bool, Option<i64>)> = tx.query_row(
            "SELECT o.revision,o.favorite,o.to_try,o.rating FROM recipe_organization o JOIN recipes r ON r.id=o.recipe_id WHERE o.recipe_id=?1 AND r.deleted_at IS NULL",
            [&write.recipe_id], |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?))
        ).optional()?;
        let Some((revision, favorite, to_try, rating)) = old else {
            return Err(not_found());
        };
        if revision != write.expected_revision {
            return Err(conflict());
        }
        if favorite != write.favorite || to_try != write.to_try || rating != write.rating {
            let count = tx.execute(
                "UPDATE recipe_organization SET favorite=?2,to_try=?3,rating=?4,revision=revision+1 WHERE recipe_id=?1 AND revision=?5",
                params![write.recipe_id,write.favorite,write.to_try,write.rating,write.expected_revision],
            )?;
            if count != 1 {
                return Err(conflict());
            }
        }
        tx.commit()?;
        self.recipe_organization(&write.recipe_id)
    }

    pub fn collections(&self) -> Result<Vec<RecipeCollection>, AppError> {
        let mut stmt = self.conn.prepare(
            "SELECT c.id,c.title,c.revision,count(r.id) FROM recipe_collections c
             LEFT JOIN recipe_collection_members m ON m.collection_id=c.id
             LEFT JOIN recipes r ON r.id=m.recipe_id AND r.deleted_at IS NULL AND r.archived_at IS NULL
             GROUP BY c.id ORDER BY c.title_key,c.id",
        )?;
        let items = stmt
            .query_map([], |r| {
                Ok(RecipeCollection {
                    id: r.get(0)?,
                    title: r.get(1)?,
                    revision: r.get(2)?,
                    active_count: r.get(3)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(items)
    }

    pub fn save_collection(
        &mut self,
        write: CollectionWrite,
    ) -> Result<RecipeCollection, AppError> {
        write.validate()?;
        let title = write.title.trim().to_string();
        let key = search_name(&title);
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let duplicate: bool = tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM recipe_collections WHERE title_key=?1 AND id<>?2)",
            params![key, write.id],
            |r| r.get(0),
        )?;
        if duplicate {
            return Err(AppError::new(
                "COLLECTION_DUPLICATE",
                "errors.collectionDuplicate",
                true,
            ));
        }
        if let Some(revision) = write.expected_revision {
            let count = tx.execute("UPDATE recipe_collections SET title=?2,title_key=?3,revision=revision+1 WHERE id=?1 AND revision=?4",params![write.id,title,key,revision])?;
            if count != 1 {
                return Err(conflict());
            }
        } else {
            let exists: bool = tx.query_row(
                "SELECT EXISTS(SELECT 1 FROM recipe_collections WHERE id=?1)",
                [&write.id],
                |r| r.get(0),
            )?;
            if exists {
                return Err(conflict());
            }
            tx.execute(
                "INSERT INTO recipe_collections(id,title,title_key) VALUES(?1,?2,?3)",
                params![write.id, title, key],
            )?;
        }
        tx.commit()?;
        self.collections()?
            .into_iter()
            .find(|c| c.id == write.id)
            .ok_or_else(not_found)
    }

    pub fn delete_collection(&mut self, id: &str, revision: i64) -> Result<(), AppError> {
        valid_id(id)?;
        if revision < 1 {
            return Err(invalid());
        }
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let current: Option<i64> = tx
            .query_row(
                "SELECT revision FROM recipe_collections WHERE id=?1",
                [id],
                |r| r.get(0),
            )
            .optional()?;
        if current != Some(revision) {
            return Err(conflict());
        }
        tx.execute("UPDATE recipe_organization SET revision=revision+1 WHERE recipe_id IN (SELECT recipe_id FROM recipe_collection_members WHERE collection_id=?1)",[id])?;
        if tx.execute(
            "DELETE FROM recipe_collections WHERE id=?1 AND revision=?2",
            params![id, revision],
        )? != 1
        {
            return Err(conflict());
        }
        tx.commit()?;
        Ok(())
    }

    pub fn set_collection_membership(
        &mut self,
        recipe_id: &str,
        collection_id: &str,
        expected_revision: i64,
        member: bool,
    ) -> Result<RecipeOrganization, AppError> {
        valid_id(recipe_id)?;
        valid_id(collection_id)?;
        if expected_revision < 1 {
            return Err(invalid());
        }
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let revision: Option<i64> = tx.query_row(
            "SELECT o.revision FROM recipe_organization o JOIN recipes r ON r.id=o.recipe_id WHERE o.recipe_id=?1 AND r.deleted_at IS NULL",
            [recipe_id], |r| r.get(0)
        ).optional()?;
        let Some(revision) = revision else {
            return Err(not_found());
        };
        if revision != expected_revision {
            return Err(conflict());
        }
        let exists: bool = tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM recipe_collections WHERE id=?1)",
            [collection_id],
            |r| r.get(0),
        )?;
        if !exists {
            return Err(not_found());
        }
        let changed = if member {
            tx.execute("INSERT OR IGNORE INTO recipe_collection_members(collection_id,recipe_id) VALUES(?1,?2)",params![collection_id,recipe_id])?
        } else {
            tx.execute(
                "DELETE FROM recipe_collection_members WHERE collection_id=?1 AND recipe_id=?2",
                params![collection_id, recipe_id],
            )?
        };
        if changed > 0 {
            tx.execute(
                "UPDATE recipe_organization SET revision=revision+1 WHERE recipe_id=?1",
                [recipe_id],
            )?;
        }
        tx.commit()?;
        self.recipe_organization(recipe_id)
    }
}
