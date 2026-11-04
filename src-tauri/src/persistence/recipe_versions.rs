use super::recipes::read_recipe;
use crate::domain::{recipe_versions::*, recipes::*, AppError};
use rusqlite::{params, Connection, OptionalExtension, Transaction};
use sha2::{Digest, Sha256};

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
pub(super) fn fingerprint(snapshot: &RecipeSnapshot) -> Result<String, AppError> {
    Ok(digest(
        &serde_json::to_vec(&sorted_keys(if snapshot.format_version == 1 {
            snapshot.comparison_v1()
        } else {
            snapshot.comparison_v2()
        }))
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
/// Full snapshots are read singly; the public service caps each metadata page.
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
    // Dispatch before strict supported-format decoding, so a future format with new fields is
    // a recoverable unsupported version, never silently narrowed to a supported format.
    let header: Header = serde_json::from_str(&payload).map_err(|_| AppError::integrity())?;
    if header.format_version != summary.format_version || header.recipe_id != id {
        return Err(AppError::integrity());
    }
    if ![1, 2].contains(&header.format_version) {
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
        "SELECT EXISTS(SELECT 1 FROM recipe_versions WHERE recipe_id=?1 AND format_version NOT IN (1,2))",
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
    capture_with_origin(tx, id, provenance, None)
}
pub(super) fn capture_with_origin(
    tx: &Transaction<'_>,
    id: &str,
    provenance: &str,
    restored_from: Option<i64>,
) -> Result<(), AppError> {
    let recipe = read_recipe(tx, id)?;
    let revision = recipe.revision;
    let mut snapshot = RecipeSnapshot::from_recipe(recipe);
    // Migration010 runs before012 during old-profile initialization. Its honest
    // baseline remains format1; no table or historical snapshot is back-edited.
    if !tx.query_row(
        "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE name='recipe_ingredient_alternatives')",
        [],
        |r| r.get::<_, bool>(0),
    )? {
        snapshot.format_version = 1;
    }
    snapshot.validate()?;
    let hash = fingerprint(&snapshot)?;
    let latest = list_versions(tx, id, None, 1)?
        .first()
        .map(|v| v.version_number);
    if let Some(version) = latest {
        // Validate retained bytes as well as the fingerprint before trusting a no-op.
        if read_version(tx, id, version)?.snapshot.comparison_v2() == snapshot.comparison_v2() {
            return Ok(());
        }
    }
    let next = latest
        .unwrap_or(0)
        .checked_add(1)
        .ok_or_else(AppError::integrity)?;
    let provenance = if restored_from.is_some() {
        "restored"
    } else if latest.is_some() {
        "edited"
    } else {
        provenance
    };
    let payload = serde_json::to_string(&snapshot).map_err(|_| AppError::integrity())?;
    if payload.len() > MAX_SNAPSHOT_BYTES {
        return Err(invalid());
    }
    tx.execute("INSERT INTO recipe_versions(recipe_id,version_number,format_version,source_revision,provenance,content_fingerprint,snapshot_checksum,snapshot,restored_from_version) VALUES(?1,?2,?9,?3,?4,?5,?6,?7,?8)",params![id,next,revision,provenance,hash,digest(payload.as_bytes()),payload,restored_from,snapshot.format_version])?;
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

impl super::Database {
    pub fn recipe_versions(
        &self,
        id: &str,
        before: Option<i64>,
        limit: u32,
    ) -> Result<Vec<RecipeVersionSummary>, AppError> {
        valid_id(id)?;
        let tx = self.conn.unchecked_transaction()?;
        let exists: bool = tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM recipes WHERE id=?1)",
            [id],
            |r| r.get(0),
        )?;
        if !exists {
            return Err(not_found());
        }
        list_versions(&tx, id, before, limit)
    }
    pub fn recipe_version(&self, id: &str, version: i64) -> Result<RecipeVersion, AppError> {
        read_version(&self.conn, id, version)
    }
    pub fn compare_recipe_versions(
        &self,
        id: &str,
        left: i64,
        right: Option<i64>,
    ) -> Result<VersionComparison, AppError> {
        let tx = self.conn.unchecked_transaction()?;
        let current = read_recipe(&tx, id)?;
        let revision = current.revision;
        let a = read_version(&tx, id, left)?.snapshot;
        let b = match right {
            Some(v) => read_version(&tx, id, v)?.snapshot,
            None => RecipeSnapshot::from_recipe(current),
        };
        let mut changes = Vec::new();
        compare_values(
            &mut changes,
            &mut Vec::new(),
            &comparison(&a),
            &comparison(&b),
        );
        Ok(VersionComparison {
            current_revision: revision,
            changes,
        })
    }
    pub fn restore_recipe_version(
        &mut self,
        write: RestoreVersion,
    ) -> Result<RestoreResult, AppError> {
        valid_id(&write.recipe_id)?;
        if write.version_number < 1 || write.expected_revision < 1 {
            return Err(invalid());
        }
        let tx = self
            .conn
            .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
        let current = read_recipe(&tx, &write.recipe_id)?;
        if current.revision != write.expected_revision {
            return Err(conflict());
        }
        if current.deleted_at.is_some() || current.archived_at.is_some() {
            return Err(AppError::new(
                "RECIPE_RESTORE_INACTIVE",
                "errors.recipeRestoreInactive",
                true,
            ));
        }
        let draft: bool = tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM editor_drafts WHERE recipe_id=?1 AND state='active')",
            [&write.recipe_id],
            |r| r.get(0),
        )?;
        if draft {
            return Err(AppError::new(
                "RECIPE_RESTORE_DRAFT",
                "errors.recipeRestoreDraft",
                true,
            ));
        }
        // Unknown retained formats may own references that this binary cannot reconstruct.
        check_purge_format(&tx, &write.recipe_id)?;
        let selected = read_version(&tx, &write.recipe_id, write.version_number)?;
        let latest = list_versions(&tx, &write.recipe_id, None, 1)?
            .pop()
            .ok_or_else(AppError::integrity)?;
        read_version(&tx, &write.recipe_id, latest.version_number)?;
        for line in &selected.snapshot.ingredients {
            let valid:bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM ingredients WHERE id=?1) AND EXISTS(SELECT 1 FROM units WHERE code=?2)",params![line.ingredient_id,line.unit_code],|r|r.get(0))?;
            if !valid {
                return Err(AppError::new(
                    "RECIPE_VERSION_REFERENCE",
                    "errors.recipeVersionReference",
                    true,
                ));
            }
        }
        for a in selected
            .snapshot
            .ingredients
            .iter()
            .flat_map(|l| &l.alternatives)
        {
            let valid:bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM ingredients WHERE id=?1) AND (?2 IS NULL OR EXISTS(SELECT 1 FROM units WHERE code=?2))",params![a.ingredient_id,a.unit_code],|r|r.get(0))?;
            if !valid {
                return Err(AppError::new(
                    "RECIPE_VERSION_REFERENCE",
                    "errors.recipeVersionReference",
                    true,
                ));
            }
        }
        if RecipeSnapshot::from_recipe(current.clone()).comparison_v2()
            == selected.snapshot.comparison_v2()
        {
            return Ok(RestoreResult {
                recipe: current,
                changed: false,
                version_number: latest.version_number,
            });
        }
        let mut input = selected.snapshot.content();
        input.expected_revision = Some(write.expected_revision);
        input.validate()?;
        super::recipes::write_recipe_with_history(&tx, &input, Some(write.version_number))?;
        let recipe = read_recipe(&tx, &write.recipe_id)?;
        let newest = list_versions(&tx, &write.recipe_id, None, 1)?
            .pop()
            .ok_or_else(AppError::integrity)?;
        if newest.version_number != latest.version_number + 1
            || newest.provenance != "restored"
            || newest.restored_from_version != Some(write.version_number)
        {
            return Err(AppError::integrity());
        }
        tx.commit()?;
        Ok(RestoreResult {
            recipe,
            changed: true,
            version_number: newest.version_number,
        })
    }
}
// Compare exact content plus captured labels. Arrays remain ordered; step links
// use the frozen set-order projection. Private/lifecycle metadata never enters.
fn comparison(snapshot: &RecipeSnapshot) -> serde_json::Value {
    let mut value = snapshot.comparison_v2();
    value["ingredientNames"] =
        serde_json::to_value(&snapshot.ingredient_names).expect("string map");
    value
}
fn compare_values(
    out: &mut Vec<VersionChange>,
    path: &mut Vec<String>,
    a: &serde_json::Value,
    b: &serde_json::Value,
) {
    use serde_json::Value;
    if a == b {
        return;
    }
    match (a, b) {
        (Value::Object(a), Value::Object(b)) => {
            let keys: std::collections::BTreeSet<_> = a.keys().chain(b.keys()).collect();
            for key in keys {
                path.push(key.clone());
                compare_values(
                    out,
                    path,
                    a.get(key).unwrap_or(&Value::Null),
                    b.get(key).unwrap_or(&Value::Null),
                );
                path.pop();
            }
        }
        (Value::Array(a), Value::Array(b)) => {
            for i in 0..a.len().max(b.len()) {
                path.push(i.to_string());
                compare_values(
                    out,
                    path,
                    a.get(i).unwrap_or(&Value::Null),
                    b.get(i).unwrap_or(&Value::Null),
                );
                path.pop();
            }
        }
        _ => out.push(VersionChange {
            path: path.clone(),
            before: a.clone(),
            after: b.clone(),
        }),
    }
}
