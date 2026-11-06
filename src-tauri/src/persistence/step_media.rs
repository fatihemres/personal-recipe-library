//! Step ownership stays separate for current recipes, durable drafts and history.
use crate::domain::{
    recipes::{invalid, RecipeInput, RecipeStep},
    AppError,
};
use rusqlite::{params, Connection};
use std::collections::{BTreeSet, HashMap};

pub(super) fn available(conn: &Connection) -> Result<bool, AppError> {
    Ok(conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE name='recipe_step_media')",
        [],
        |r| r.get(0),
    )?)
}
/// Check every distinct asset at most once; never decode full images in recipe reads.
pub(super) fn verify(conn: &Connection, input: &RecipeInput, files: bool) -> Result<(), AppError> {
    if !available(conn)? && input.steps.iter().any(|s| s.photo_asset_id.is_some()) {
        return Err(invalid());
    }
    let ids: BTreeSet<_> = input
        .cover_asset_id
        .iter()
        .chain(input.steps.iter().filter_map(|s| s.photo_asset_id.as_ref()))
        .collect();
    for id in ids {
        super::media::verify_cover(conn, Some(id), files)?;
    }
    Ok(())
}
pub(super) fn read(
    conn: &Connection,
    recipe_id: &str,
    steps: &mut [RecipeStep],
) -> Result<(), AppError> {
    if !available(conn)? {
        return Ok(());
    }
    let mut stmt =
        conn.prepare("SELECT step_id,asset_id FROM recipe_step_media WHERE recipe_id=?1")?;
    let mut pins = stmt
        .query_map([recipe_id], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
        })?
        .collect::<Result<HashMap<_, _>, _>>()?;
    for step in steps {
        step.photo_asset_id = pins.remove(&step.id);
    }
    if !pins.is_empty() {
        return Err(AppError::integrity());
    }
    Ok(())
}
/// The caller has already recreated all stable child IDs inside its recipe CAS tx.
pub(super) fn pin_recipe(conn: &Connection, input: &RecipeInput) -> Result<(), AppError> {
    verify(conn, input, true)?;
    if !available(conn)? {
        return Ok(());
    }
    let mut stmt =
        conn.prepare("INSERT INTO recipe_step_media(recipe_id,step_id,asset_id) VALUES(?1,?2,?3)")?;
    for step in &input.steps {
        if let Some(asset) = &step.photo_asset_id {
            stmt.execute(params![input.id, step.id, asset])?;
        }
    }
    Ok(())
}
/// Payload and these pins are changed within the existing draft-revision transaction.
pub(super) fn pin_draft(
    conn: &Connection,
    draft_id: &str,
    input: &RecipeInput,
) -> Result<(), AppError> {
    verify(conn, input, false)?;
    if !available(conn)? {
        return Ok(());
    }
    conn.execute("DELETE FROM draft_step_media WHERE draft_id=?1", [draft_id])?;
    let mut stmt =
        conn.prepare("INSERT INTO draft_step_media(draft_id,step_id,asset_id) VALUES(?1,?2,?3)")?;
    for step in &input.steps {
        if let Some(asset) = &step.photo_asset_id {
            stmt.execute(params![draft_id, step.id, asset])?;
        }
    }
    Ok(())
}
pub(super) fn close_draft(conn: &Connection, id: &str) -> Result<(), AppError> {
    if available(conn)? {
        conn.execute("DELETE FROM draft_step_media WHERE draft_id=?1", [id])?;
    }
    Ok(())
}
