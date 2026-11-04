use super::Database;
use crate::{
    domain::{recipes::valid_id, AppError},
    media::{error, ImportedCover, Manifest, MediaStore},
};
use rusqlite::{params, Connection, OptionalExtension};
use std::path::Path;
pub(super) fn available(conn: &Connection) -> Result<bool, AppError> {
    Ok(conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE name='media_assets')",
        [],
        |r| r.get(0),
    )?)
}
fn store(conn: &Connection) -> Result<MediaStore, AppError> {
    let p = conn
        .path()
        .map(Path::new)
        .and_then(Path::parent)
        .ok_or_else(|| error("MEDIA_UNAVAILABLE"))?;
    MediaStore::new(p)
}
fn ready(conn: &Connection, id: &str) -> Result<Manifest, AppError> {
    valid_id(id)?;
    let raw: Option<String> = conn
        .query_row(
            "SELECT manifest FROM media_assets WHERE id=?1 AND state='ready'",
            [id],
            |r| r.get(0),
        )
        .optional()?;
    let raw = raw.ok_or_else(|| error("MEDIA_UNAVAILABLE"))?;
    if raw.len() > 8192 {
        return Err(error("MEDIA_UNAVAILABLE"));
    }
    let m: Manifest = serde_json::from_str(&raw).map_err(|_| error("MEDIA_UNAVAILABLE"))?;
    if m.asset_id != id {
        return Err(error("MEDIA_UNAVAILABLE"));
    }
    m.validate()?;
    Ok(m)
}
pub(super) fn verify_cover(
    conn: &Connection,
    id: Option<&str>,
    files: bool,
) -> Result<(), AppError> {
    if let Some(id) = id {
        let m = ready(conn, id)?;
        if files {
            store(conn)?
                .verify(&m, false)
                .map_err(|_| error("MEDIA_UNAVAILABLE"))?
        }
    }
    Ok(())
}
impl Database {
    pub fn import_cover(&mut self, source: &Path) -> Result<ImportedCover, AppError> {
        let id = uuid::Uuid::new_v4().to_string();
        let store = store(&self.conn)?;
        self.conn.execute(
            "INSERT INTO media_assets(id,state) VALUES(?1,'writing')",
            [&id],
        )?;
        let result = (|| {
            let manifest = store.prepare(&id, source)?;
            let raw = serde_json::to_string(&manifest).map_err(|_| error("MEDIA_UNAVAILABLE"))?;
            self.conn.execute(
                "UPDATE media_assets SET manifest=?2 WHERE id=?1 AND state='writing'",
                params![id, raw],
            )?;
            store.finalize(&manifest)?;
            self.finish_media(&manifest)?;
            Ok(ImportedCover {
                id: id.clone(),
                width: manifest.width,
                height: manifest.height,
            })
        })();
        if result.is_err() {
            // A finalized complete folder may be recoverable even if the last SQLite
            // commit failed. Keep the writing journal in that case for next startup.
            if store.recover(&id, None).is_err()
                && self
                    .conn
                    .execute(
                        "UPDATE media_assets SET state='failed' WHERE id=?1 AND state='writing'",
                        [&id],
                    )
                    .is_ok()
            {
                let _ = store.clean_staging(&id);
            }
        }
        result
    }
    fn finish_media(&mut self, m: &Manifest) -> Result<(), AppError> {
        m.validate()?;
        let raw = serde_json::to_string(m).map_err(|_| error("MEDIA_UNAVAILABLE"))?;
        let tx = self
            .conn
            .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
        if tx.execute(
            "UPDATE media_assets SET state='ready',manifest=?2 WHERE id=?1 AND state='writing'",
            params![m.asset_id, raw],
        )? != 1
        {
            return Err(error("MEDIA_UNAVAILABLE"));
        }
        tx.commit()?;
        Ok(())
    }
    pub(super) fn reconcile_media(&mut self) -> Result<(), AppError> {
        if !available(&self.conn)? {
            return Ok(());
        }
        let store = store(&self.conn)?;
        let mut after = String::new();
        loop {
            let pending = {
                let mut s=self.conn.prepare("SELECT id,manifest FROM media_assets WHERE state='writing' AND id>?1 ORDER BY id LIMIT 64")?;
                let rows = s
                    .query_map([&after], |r| {
                        Ok((r.get::<_, String>(0)?, r.get::<_, Option<String>>(1)?))
                    })?
                    .collect::<Result<Vec<_>, _>>()?;
                rows
            };
            if pending.is_empty() {
                break;
            }
            for (id, raw) in pending {
                after = id.clone();
                let expected = raw
                    .as_ref()
                    .map(|s| serde_json::from_str::<Manifest>(s))
                    .transpose();
                let recovered = expected
                    .map_err(|_| error("MEDIA_UNAVAILABLE"))
                    .and_then(|m| store.recover(&id, m.as_ref()));
                match recovered {
                    Ok(m) => self.finish_media(&m)?,
                    Err(_) => {
                        self.conn.execute("UPDATE media_assets SET state='failed' WHERE id=?1 AND state='writing'",[&id])?;
                        let _ = store.clean_staging(&id);
                    }
                }
            }
        }
        Ok(())
    }
    pub fn media_bytes(&self, id: &str, variant: &str) -> Result<Vec<u8>, AppError> {
        let m = ready(&self.conn, id)?;
        store(&self.conn)?.serve(&m, variant)
    }
}
