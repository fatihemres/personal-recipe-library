use super::{
    recipes::{read_recipe, write_recipe},
    Database,
};
use crate::domain::{recipe_relations::*, recipes::*, reliability::independent, AppError};
use rusqlite::{params, Connection, TransactionBehavior};
use sha2::{Digest, Sha256};
const REFERENCES:&str="SELECT r.id,r.title,r.revision,CASE WHEN r.deleted_at IS NOT NULL THEN 'trash' WHEN r.archived_at IS NOT NULL THEN 'archived' ELSE 'active' END,edges.kind FROM (
 SELECT parent_id peer,'parent' kind FROM recipe_variations WHERE child_id=?1
 UNION ALL SELECT child_id,'child' FROM recipe_variations WHERE parent_id=?1
 UNION ALL SELECT right_id,'related' FROM recipe_related WHERE left_id=?1
 UNION ALL SELECT left_id,'related' FROM recipe_related WHERE right_id=?1
) edges JOIN recipes r ON r.id=edges.peer ORDER BY edges.kind,r.id";
fn reference(r: &rusqlite::Row<'_>) -> rusqlite::Result<RecipeReference> {
    Ok(RecipeReference {
        id: r.get(0)?,
        title: r.get(1)?,
        revision: r.get(2)?,
        status: r.get(3)?,
        relationship: r.get(4)?,
    })
}
fn active(conn: &Connection, id: &str) -> Result<(), AppError> {
    valid_id(id)?;
    let ok:bool=conn.query_row("SELECT EXISTS(SELECT 1 FROM recipes WHERE id=?1 AND deleted_at IS NULL AND archived_at IS NULL)",[id],|r|r.get(0))?;
    if ok {
        Ok(())
    } else {
        Err(AppError::new(
            "RELATION_INACTIVE",
            "errors.relationInactive",
            true,
        ))
    }
}
fn graph_revision(conn: &Connection, id: &str) -> Result<i64, AppError> {
    valid_id(id)?;
    conn.query_row(
        "SELECT revision FROM recipe_graph_state WHERE recipe_id=?1",
        [id],
        |r| r.get(0),
    )
    .map_err(|e| {
        if matches!(e, rusqlite::Error::QueryReturnedNoRows) {
            not_found()
        } else {
            e.into()
        }
    })
}
fn check_graph(conn: &Connection, id: &str, expected: i64) -> Result<(), AppError> {
    if expected < 1 {
        return Err(invalid());
    }
    if graph_revision(conn, id)? != expected {
        return Err(conflict());
    }
    Ok(())
}
pub(super) fn ensure_no_links(conn: &Connection, id: &str) -> Result<(), AppError> {
    let exists:bool=conn.query_row("SELECT EXISTS(SELECT 1 FROM recipe_variations WHERE child_id=?1 OR parent_id=?1) OR EXISTS(SELECT 1 FROM recipe_related WHERE left_id=?1 OR right_id=?1)",[id],|r|r.get(0))?;
    if exists {
        Err(AppError::new(
            "RECIPE_LINKS_BLOCK_PURGE",
            "errors.recipeLinksBlockPurge",
            true,
        ))
    } else {
        Ok(())
    }
}
fn links(conn: &Connection, id: &str, offset: u32, limit: u32) -> Result<RecipeLinks, AppError> {
    if limit == 0 || limit > 100 || offset > 1_000_000 {
        return Err(invalid());
    }
    let revision = graph_revision(conn, id)?;
    let total:i64=conn.query_row("SELECT (SELECT count(*) FROM recipe_variations WHERE child_id=?1 OR parent_id=?1)+(SELECT count(*) FROM recipe_related WHERE left_id=?1 OR right_id=?1)",[id],|r|r.get(0))?;
    let items = conn
        .prepare(&format!("{REFERENCES} LIMIT ?2 OFFSET ?3"))?
        .query_map(params![id, limit, offset], reference)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(RecipeLinks {
        graph_revision: revision,
        has_more: i64::from(offset) + (items.len() as i64) < total,
        total,
        items,
    })
}
// Stream all dependencies into a deterministic review token, without materializing
// all referenced aggregates. Graph revision detects even detach/re-add (ABA).
fn purge_token(conn: &Connection, id: &str, revision: i64) -> Result<String, AppError> {
    let mut hash = Sha256::new();
    hash.update(
        serde_json::to_vec(&(id, revision, graph_revision(conn, id)?))
            .map_err(|_| AppError::integrity())?,
    );
    let mut stmt = conn.prepare(REFERENCES)?;
    let rows = stmt.query_map([id], reference)?;
    for row in rows {
        let bytes = serde_json::to_vec(&row?).map_err(|_| AppError::integrity())?;
        hash.update((bytes.len() as u64).to_le_bytes());
        hash.update(bytes);
    }
    Ok(format!("{:x}", hash.finalize()))
}
fn eligible(conn: &Connection, id: &str, revision: i64) -> Result<(), AppError> {
    valid_id(id)?;
    if revision < 1 {
        return Err(invalid());
    }
    let ok:bool=conn.query_row("SELECT EXISTS(SELECT 1 FROM recipes WHERE id=?1 AND revision=?2 AND deleted_at IS NOT NULL)",params![id,revision],|r|r.get(0))?;
    if !ok {
        return Err(conflict());
    }
    Ok(())
}
impl Database {
    pub fn recipe_links(&self, id: &str, offset: u32, limit: u32) -> Result<RecipeLinks, AppError> {
        let tx = self.conn.unchecked_transaction()?;
        links(&tx, id, offset, limit)
    }
    pub fn create_variation(&mut self, id: &str, revision: i64) -> Result<Recipe, AppError> {
        valid_id(id)?;
        if revision < 1 {
            return Err(invalid());
        }
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        active(&tx, id)?;
        let source = read_recipe(&tx, id)?;
        if source.revision != revision {
            return Err(conflict());
        }
        let input = independent(source.to_input());
        write_recipe(&tx, &input)?;
        tx.execute(
            "INSERT INTO recipe_variations(child_id,parent_id) VALUES(?1,?2)",
            params![input.id, id],
        )?;
        // Deliberately do not copy independent organization or private Markdown.
        let result = read_recipe(&tx, &input.id)?;
        tx.commit()?;
        Ok(result)
    }
    pub fn set_variation_parent(&mut self, write: ParentWrite) -> Result<(), AppError> {
        valid_id(&write.child_id)?;
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        check_graph(&tx, &write.child_id, write.expected_graph_revision)?;
        if let Some(parent) = &write.parent_id {
            active(&tx, &write.child_id)?;
            active(&tx, parent)?;
            if parent == &write.child_id {
                return Err(invalid());
            }
            let cycle:bool=tx.query_row("WITH RECURSIVE ancestors(id) AS (SELECT ?1 UNION SELECT parent_id FROM recipe_variations JOIN ancestors ON child_id=ancestors.id) SELECT EXISTS(SELECT 1 FROM ancestors WHERE id=?2)",params![parent,write.child_id],|r|r.get(0))?;
            if cycle {
                return Err(AppError::new(
                    "VARIATION_CYCLE",
                    "errors.variationCycle",
                    true,
                ));
            }
            tx.execute("INSERT INTO recipe_variations(child_id,parent_id) VALUES(?1,?2) ON CONFLICT(child_id) DO UPDATE SET parent_id=excluded.parent_id",params![write.child_id,parent])?;
        } else {
            tx.execute(
                "DELETE FROM recipe_variations WHERE child_id=?1",
                [&write.child_id],
            )?;
        }
        tx.commit()?;
        Ok(())
    }
    pub fn set_related_recipe(&mut self, write: RelatedWrite) -> Result<(), AppError> {
        valid_id(&write.recipe_id)?;
        valid_id(&write.target_id)?;
        if write.recipe_id == write.target_id {
            return Err(invalid());
        }
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        check_graph(&tx, &write.recipe_id, write.expected_graph_revision)?;
        let (a, b) = if write.recipe_id < write.target_id {
            (&write.recipe_id, &write.target_id)
        } else {
            (&write.target_id, &write.recipe_id)
        };
        if write.linked {
            active(&tx, a)?;
            active(&tx, b)?;
            let duplicate: bool = tx.query_row(
                "SELECT EXISTS(SELECT 1 FROM recipe_related WHERE left_id=?1 AND right_id=?2)",
                params![a, b],
                |r| r.get(0),
            )?;
            if duplicate {
                return Err(AppError::new(
                    "RELATION_EXISTS",
                    "errors.relationExists",
                    true,
                ));
            }
            tx.execute(
                "INSERT INTO recipe_related(left_id,right_id) VALUES(?1,?2)",
                params![a, b],
            )?;
        } else if tx.execute(
            "DELETE FROM recipe_related WHERE left_id=?1 AND right_id=?2",
            params![a, b],
        )? != 1
        {
            return Err(not_found());
        }
        tx.commit()?;
        Ok(())
    }
    pub fn preview_recipe_purge(
        &self,
        id: &str,
        revision: i64,
        offset: u32,
        limit: u32,
    ) -> Result<PurgePreview, AppError> {
        let tx = self.conn.unchecked_transaction()?;
        eligible(&tx, id, revision)?;
        Ok(PurgePreview {
            recipe_id: id.into(),
            recipe_revision: revision,
            token: purge_token(&tx, id, revision)?,
            links: links(&tx, id, offset, limit)?,
        })
    }
    pub fn purge_reviewed_recipe(&mut self, write: ReviewedPurge) -> Result<(), AppError> {
        if write.token.len() != 64 || !write.token.bytes().all(|c| c.is_ascii_hexdigit()) {
            return Err(invalid());
        }
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        eligible(&tx, &write.recipe_id, write.expected_revision)?;
        if purge_token(&tx, &write.recipe_id, write.expected_revision)? != write.token {
            return Err(AppError::new(
                "PURGE_REVIEW_STALE",
                "errors.purgeReviewStale",
                true,
            ));
        }
        if !write.acknowledge_detach {
            ensure_no_links(&tx, &write.recipe_id)?;
        }
        // Unknown future snapshot references must never be detached by this binary.
        super::recipe_versions::check_purge_format(&tx, &write.recipe_id)?;
        tx.execute(
            "DELETE FROM recipe_variations WHERE child_id=?1 OR parent_id=?1",
            [&write.recipe_id],
        )?;
        tx.execute(
            "DELETE FROM recipe_related WHERE left_id=?1 OR right_id=?1",
            [&write.recipe_id],
        )?;
        super::reliability::purge_owned_recipe(&tx, &write.recipe_id, write.expected_revision)?;
        tx.commit()?;
        Ok(())
    }
}
