use super::recipes::read_recipe;
use crate::domain::{recipe_versions::*, recipes::*, AppError};
use rusqlite::{params, Connection, OptionalExtension, Transaction};
use sha2::{Digest, Sha256};

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
pub(super) fn fingerprint(snapshot: &RecipeSnapshot) -> Result<String, AppError> {
    Ok(digest(
        &serde_json::to_vec(&sorted_keys(snapshot.comparison_v1()))
            .map_err(|_| AppError::integrity())?,
    ))
}
fn sorted_keys(value: serde_json::Value) -> serde_json::Value {
    use serde_json::Value;
    match value {
        Value::Object(fields) => {
            let sorted: std::collections::BTreeMap<_, _> = fields
                .into_iter()
                .map(|(k, v)| (k, sorted_keys(v)))
                .collect();
            Value::Object(sorted.into_iter().collect())
        }
        Value::Array(values) => Value::Array(values.into_iter().map(sorted_keys).collect()),
        scalar => scalar,
    }
}
fn summary(row: &rusqlite::Row<'_>) -> rusqlite::Result<RecipeVersionSummary> {
    Ok(RecipeVersionSummary {
        recipe_id: row.get(0)?,
        version_number: row.get(1)?,
        format_version: row.get(2)?,
        source_revision: row.get(3)?,
        provenance: row.get(4)?,
        restored_from_version: row.get(5)?,
        captured_at: row.get(6)?,
        content_fingerprint: row.get(7)?,
        snapshot_checksum: row.get(8)?,
    })
}

/// Bounded keyset metadata query shared by capture and future history services.
/// No history IPC/UI is exposed in M5B-1.
pub(super) fn list_versions(
    conn: &Connection,
    id: &str,
    before_version: Option<i64>,
    limit: u32,
) -> Result<Vec<RecipeVersionSummary>, AppError> {
    valid_id(id)?;
    if !(1..=100).contains(&limit) || before_version.is_some_and(|v| v < 1) {
        return Err(invalid());
    }
    let mut stmt=conn.prepare("SELECT recipe_id,version_number,format_version,source_revision,provenance,restored_from_version,captured_at,content_fingerprint,snapshot_checksum FROM recipe_versions WHERE recipe_id=?1 AND (?2 IS NULL OR version_number<?2) ORDER BY version_number DESC LIMIT ?3")?;
    let result = stmt
        .query_map(params![id, before_version, limit], summary)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(result)
}

pub(super) fn read_version(
    conn: &Connection,
    id: &str,
    version: i64,
) -> Result<RecipeVersion, AppError> {
    valid_id(id)?;
    if version < 1 {
        return Err(invalid());
    }
    let (summary,payload)=conn.query_row("SELECT recipe_id,version_number,format_version,source_revision,provenance,restored_from_version,captured_at,content_fingerprint,snapshot_checksum,snapshot FROM recipe_versions WHERE recipe_id=?1 AND version_number=?2",params![id,version],|row|Ok((summary(row)?,row.get::<_,String>(9)?))).optional()?.ok_or_else(not_found)?;
    if payload.len() > MAX_SNAPSHOT_BYTES || digest(payload.as_bytes()) != summary.snapshot_checksum
    {
        return Err(AppError::integrity());
    }
    #[derive(serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Header {
        format_version: i64,
        recipe_id: String,
    }
    // Dispatch before strict V1 decoding, so a future format with new fields is
    // a recoverable unsupported version, never silently narrowed to V1.
    let header: Header = serde_json::from_str(&payload).map_err(|_| AppError::integrity())?;
    if header.format_version != summary.format_version || header.recipe_id != id {
        return Err(AppError::integrity());
    }
    if header.format_version != 1 {
        return Err(AppError::new(
            "RECIPE_VERSION_UNSUPPORTED",
            "errors.recipeVersionUnsupported",
            true,
        ));
    }
    let snapshot: RecipeSnapshot =
        serde_json::from_str(&payload).map_err(|_| AppError::integrity())?;
    snapshot.validate()?;
    if snapshot.recipe_id != id
        || snapshot.format_version != summary.format_version
        || fingerprint(&snapshot)? != summary.content_fingerprint
    {
        return Err(AppError::integrity());
    }
    Ok(RecipeVersion { summary, snapshot })
}

pub(super) fn check_purge_format(conn: &Connection, id: &str) -> Result<(), AppError> {
    let unsupported: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM recipe_versions WHERE recipe_id=?1 AND format_version<>1)",
        [id],
        |r| r.get(0),
    )?;
    if unsupported {
        return Err(AppError::new(
            "RECIPE_VERSION_UNSUPPORTED",
            "errors.recipeVersionUnsupported",
            true,
        ));
    }
    Ok(())
}

/// Always called after the complete aggregate write, before its transaction commits.
/// Hash labels separately: renaming an ingredient is not a recipe-content edit.
pub(super) fn capture(tx: &Transaction<'_>, id: &str, provenance: &str) -> Result<(), AppError> {
    let recipe = read_recipe(tx, id)?;
    let revision = recipe.revision;
    let snapshot = RecipeSnapshot::from_recipe(recipe);
    snapshot.validate()?;
    let hash = fingerprint(&snapshot)?;
    let latest = list_versions(tx, id, None, 1)?
        .first()
        .map(|v| v.version_number);
    if let Some(version) = latest {
        // Validate retained bytes as well as the fingerprint before trusting a no-op.
        if read_version(tx, id, version)?.summary.content_fingerprint == hash {
            return Ok(());
        }
    }
    let next = latest
        .unwrap_or(0)
        .checked_add(1)
        .ok_or_else(AppError::integrity)?;
    let provenance = if latest.is_some() {
        "edited"
    } else {
        provenance
    };
    let payload = serde_json::to_string(&snapshot).map_err(|_| AppError::integrity())?;
    if payload.len() > MAX_SNAPSHOT_BYTES {
        return Err(invalid());
    }
    tx.execute("INSERT INTO recipe_versions(recipe_id,version_number,format_version,source_revision,provenance,content_fingerprint,snapshot_checksum,snapshot) VALUES(?1,?2,1,?3,?4,?5,?6,?7)",params![id,next,revision,provenance,hash,digest(payload.as_bytes()),payload])?;
    // SQL derives reference rows; reading immediately also verifies serialization.
    read_version(tx, id, next)?;
    Ok(())
}

/// Migration 010 hook, inside the migration runner's IMMEDIATE transaction and
/// pre-upgrade backup boundary. One aggregate in memory; 64 IDs per cursor page.
pub(super) fn backfill(tx: &Transaction<'_>) -> Result<(), AppError> {
    let mut after = String::new();
    loop {
        let ids = {
            let mut stmt = tx.prepare("SELECT id FROM recipes WHERE id>?1 ORDER BY id LIMIT 64")?;
            let ids = stmt
                .query_map([&after], |r| r.get::<_, String>(0))?
                .collect::<Result<Vec<_>, _>>()?;
            ids
        };
        if ids.is_empty() {
            break;
        }
        for id in &ids {
            capture(tx, id, "upgrade_baseline")?;
        }
        after = ids.last().expect("nonempty page").clone();
    }
    let complete:bool=tx.query_row("SELECT (SELECT count(*) FROM recipes)=(SELECT count(*) FROM recipe_versions WHERE version_number=1 AND provenance='upgrade_baseline') AND NOT EXISTS(SELECT 1 FROM recipe_versions WHERE version_number<>1)",[],|r|r.get(0))?;
    if !complete {
        return Err(AppError::integrity());
    }
    Ok(())
}
