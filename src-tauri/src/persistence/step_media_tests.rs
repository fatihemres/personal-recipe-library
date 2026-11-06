use super::*;
use crate::domain::{
    recipe_organization::*, recipe_relations::*, recipe_versions::*, recipe_workspace::*,
    recipes::*, reliability::*,
};
use rusqlite::{params, Connection};
use serde_json::json;
use std::path::Path;
fn id() -> String {
    uuid::Uuid::new_v4().to_string()
}
fn asset(db: &mut Database, root: &Path, color: u8) -> String {
    let p = root.join(format!("Şişe — {color}.png"));
    image::RgbaImage::from_pixel(48, 32, image::Rgba([color, 30, 80, 190]))
        .save(&p)
        .unwrap();
    db.import_cover(&p).unwrap().id
}
fn input(db: &mut Database) -> RecipeInput {
    let ingredient = db.create_ingredient("M5C2 kişisel malzeme").unwrap();
    let alt = db.create_ingredient("M5C2 açık alternatif").unwrap();
    let line = id();
    serde_json::from_value(json!({"id":id(),"expectedRevision":null,"title":"M5C2 — İ/ı adımlar","description":"Açıklama","kind":"food","servings":"1.250","prepMinutes":0,"cookMinutes":null,"notes":"İçerik notu","ingredients":[{"id":line,"ingredientId":ingredient.id,"quantity":"1.250","unitCode":"g","note":"İnce","alternatives":[{"id":id(),"ingredientId":alt.id,"quantity":"0.123456","unitCode":"mL"}]}],"steps":[{"id":id(),"instructions":"Önce karıştırın","ingredientLineIds":[line],"details":{"formatVersion":1,"title":"İlk aşama","durationMinutes":"1","temperatureValue":"350","temperatureUnit":"F","heatLevel":null,"equipment":"Kap","technique":null,"tips":"İ/ı"}},{"id":id(),"instructions":"Sonra bekletin"}]})).unwrap()
}
fn setup() -> (tempfile::TempDir, Database, RecipeInput, String, String) {
    let d = tempfile::tempdir().unwrap();
    let mut db = Database::open(d.path()).unwrap();
    let a = asset(&mut db, d.path(), 100);
    let b = asset(&mut db, d.path(), 180);
    let mut i = input(&mut db);
    i.cover_asset_id = Some(a.clone());
    i.steps[0].photo_asset_id = Some(a.clone());
    (d, db, i, a, b)
}
fn healthy(db: &Database) {
    assert_eq!(
        db.conn
            .query_row("PRAGMA integrity_check", [], |r| r.get::<_, String>(0))
            .unwrap(),
        "ok"
    );
    assert!(!db
        .conn
        .prepare("PRAGMA foreign_key_check")
        .unwrap()
        .exists([])
        .unwrap());
}
fn raw(db: &Database, id: &str) -> Vec<(i64, String, String, String)> {
    db.conn.prepare("SELECT version_number,snapshot,snapshot_checksum,content_fingerprint FROM recipe_versions WHERE recipe_id=?1 ORDER BY version_number").unwrap().query_map([id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?))).unwrap().collect::<Result<_,_>>().unwrap()
}
fn pins(db: &Database, table: &str, id: &str) -> i64 {
    assert!(!table.contains(' '));
    db.conn
        .query_row(
            &format!("SELECT count(*) FROM {table} WHERE recipe_id=?1"),
            [id],
            |r| r.get(0),
        )
        .unwrap()
}
fn versions(db: &Database, id: &str) -> usize {
    raw(db, id).len()
}
fn restore(r: &Recipe, n: i64) -> RestoreVersion {
    RestoreVersion {
        recipe_id: r.id.clone(),
        version_number: n,
        expected_revision: r.revision,
    }
}

#[test]
fn step_photos_format_four_pins_cover_alternatives_photos_and_preserves_exact_content() {
    let (_d, mut db, mut i, a, b) = setup();
    i.steps[1].photo_asset_id = Some(b.clone());
    let mut r = db.save_recipe(i).unwrap();
    let v = db.recipe_version(&r.id, 1).unwrap();
    assert_eq!(v.summary.format_version, 4);
    assert_eq!(pins(&db, "recipe_version_media", &r.id), 1);
    assert_eq!(pins(&db, "recipe_version_alternatives", &r.id), 1);
    assert_eq!(pins(&db, "recipe_version_step_media", &r.id), 2);
    assert_eq!(v.snapshot.ingredients[0].quantity.as_deref(), Some("1.250"));
    assert_eq!(
        v.snapshot.ingredients[0].alternatives[0]
            .quantity
            .as_deref(),
        Some("0.123456")
    );
    assert_eq!(
        v.snapshot.steps[0]
            .details
            .as_ref()
            .unwrap()
            .temperature_unit
            .as_deref(),
        Some("F")
    );
    let before = raw(&db, &r.id);
    r = db.save_recipe(r.to_input()).unwrap();
    assert_eq!(raw(&db, &r.id), before);
    let mut edit = r.to_input();
    edit.steps.reverse();
    r = db.save_recipe(edit).unwrap();
    assert_eq!(r.steps[0].photo_asset_id.as_deref(), Some(b.as_str()));
    assert_eq!(r.steps[1].photo_asset_id.as_deref(), Some(a.as_str()));
    let mut edit = r.to_input();
    edit.steps[0].photo_asset_id = None;
    r = db.save_recipe(edit).unwrap();
    assert_eq!(pins(&db, "recipe_step_media", &r.id), 1);
    let changes = db.compare_recipe_versions(&r.id, 1, None).unwrap();
    assert!(changes
        .changes
        .iter()
        .any(|c| c.path.last().is_some_and(|s| s == "photoAssetId")));
    let result = db.restore_recipe_version(restore(&r, 1)).unwrap();
    r = result.recipe;
    assert!(result.changed);
    assert_eq!(result.version_number, 4);
    let last = db.recipe_version(&r.id, 4).unwrap();
    assert_eq!(last.summary.provenance, "restored");
    assert_eq!(last.summary.restored_from_version, Some(1));
    assert_eq!(&raw(&db, &r.id)[..1], before.as_slice());
    assert_eq!(r.steps[0].photo_asset_id, Some(a));
    assert_eq!(r.steps[1].photo_asset_id, Some(b));
    let noop = db.restore_recipe_version(restore(&r, 1)).unwrap();
    assert!(!noop.changed);
    assert_eq!(versions(&db, &r.id), 4);
    healthy(&db);
}

#[test]
fn step_photos_validate_uuid_unique_step_identity_ready_assets_and_owned_sql_links() {
    let (_d, mut db, i, a, _) = setup();
    let saved = db.save_recipe(i).unwrap();
    let before = raw(&db, &saved.id);
    for value in ["../secret.png".into(), id()] {
        let mut e = saved.to_input();
        e.steps[0].photo_asset_id = Some(value);
        assert!(db.save_recipe(e).is_err());
    }
    for state in ["writing", "failed"] {
        let asset = id();
        db.conn
            .execute(
                "INSERT INTO media_assets(id,state) VALUES(?1,?2)",
                params![asset, state],
            )
            .unwrap();
        let mut e = saved.to_input();
        e.steps[0].photo_asset_id = Some(asset);
        assert!(db.save_recipe(e).is_err());
    }
    let mut e = saved.to_input();
    e.steps[1].id = e.steps[0].id.clone();
    assert!(db.save_recipe(e).is_err());
    let mut other = crate::domain::reliability::independent(saved.to_input());
    other.title = "Başka tarif".into();
    let other = db.save_recipe(other).unwrap();
    assert!(db
        .conn
        .execute(
            "INSERT INTO recipe_step_media(recipe_id,step_id,asset_id) VALUES(?1,?2,?3)",
            params![saved.id, other.steps[1].id, a]
        )
        .is_err());
    assert!(db
        .conn
        .execute(
            "UPDATE recipe_step_media SET step_id=?2 WHERE recipe_id=?1",
            params![saved.id, id()]
        )
        .is_err());
    assert!(db.conn.execute("INSERT INTO recipe_version_step_media(recipe_id,version_number,step_id,asset_id) VALUES(?1,1,?2,?3)",params![saved.id,saved.steps[1].id,a]).is_err());
    for sql in [
        "UPDATE recipe_version_step_media SET asset_id=asset_id",
        "DELETE FROM recipe_version_step_media",
        "INSERT OR REPLACE INTO recipe_version_step_media SELECT * FROM recipe_version_step_media",
        "DELETE FROM media_assets WHERE state='ready'",
    ] {
        assert!(db.conn.execute(sql, []).is_err(), "{sql}");
    }
    assert_eq!(raw(&db, &saved.id), before);
    healthy(&db);
}

#[test]
fn step_photos_drafts_reopen_reorder_cancel_discard_and_save_as_new_keep_stable_associations() {
    let (d, mut db, i, a, b) = setup();
    let original = db.save_recipe(i).unwrap();
    let mut e = original.to_input();
    e.steps[1].photo_asset_id = Some(b.clone());
    let draft = db
        .save_draft(DraftWrite {
            id: id(),
            expected_revision: None,
            input: e,
        })
        .unwrap();
    assert_eq!(versions(&db, &original.id), 1);
    assert_eq!(
        db.conn
            .query_row(
                "SELECT count(*) FROM draft_step_media WHERE draft_id=?1",
                [&draft.id],
                |r| r.get::<_, i64>(0)
            )
            .unwrap(),
        2
    );
    drop(db);
    let mut db = Database::open(d.path()).unwrap();
    let recovered = db.draft(&draft.id).unwrap();
    assert_eq!(recovered.input.steps[0].photo_asset_id, Some(a));
    assert_eq!(recovered.input.steps[1].photo_asset_id, Some(b.clone()));
    let mut moved = recovered.input.clone();
    moved.steps.reverse();
    let moved = db
        .save_draft(DraftWrite {
            id: recovered.id.clone(),
            expected_revision: Some(recovered.revision),
            input: moved,
        })
        .unwrap();
    let copy = db
        .commit_recipe(moved.input.clone(), &moved.id, moved.revision, true)
        .unwrap();
    assert_ne!(copy.id, original.id);
    assert_ne!(copy.steps[0].id, original.steps[1].id);
    assert_eq!(copy.steps[0].photo_asset_id, Some(b));
    assert_eq!(versions(&db, &copy.id), 1);
    assert_eq!(versions(&db, &original.id), 1);
    assert!(db.draft(&draft.id).is_err());
    assert_eq!(
        db.conn
            .query_row(
                "SELECT count(*) FROM draft_step_media WHERE draft_id=?1",
                [&draft.id],
                |r| r.get::<_, i64>(0)
            )
            .unwrap(),
        0
    );
    let mut removed = original.to_input();
    removed.steps.clear();
    let draft = db
        .save_draft(DraftWrite {
            id: id(),
            expected_revision: None,
            input: removed,
        })
        .unwrap();
    db.discard_draft(&draft.id, draft.revision).unwrap();
    assert_eq!(db.recipe(&original.id).unwrap().steps.len(), 2);
    healthy(&db);
}

#[test]
fn step_photos_draft_pin_ownership_and_failed_autosave_roll_back_payload_and_revision() {
    let (_d, mut db, i, a, b) = setup();
    let r = db.save_recipe(i).unwrap();
    let draft = db
        .save_draft(DraftWrite {
            id: id(),
            expected_revision: None,
            input: r.to_input(),
        })
        .unwrap();
    for (step, asset) in [(id(), a.clone()), (r.steps[1].id.clone(), b.clone())] {
        assert!(db
            .conn
            .execute(
                "INSERT INTO draft_step_media(draft_id,step_id,asset_id) VALUES(?1,?2,?3)",
                params![draft.id, step, asset]
            )
            .is_err());
    }
    assert!(db
        .conn
        .execute(
            "UPDATE draft_step_media SET asset_id=?2 WHERE draft_id=?1",
            params![draft.id, b]
        )
        .is_err());
    db.conn.execute_batch("CREATE TRIGGER reject_draft_photo BEFORE INSERT ON draft_step_media BEGIN SELECT RAISE(ABORT,'injected draft pin'); END;").unwrap();
    let mut e = draft.input.clone();
    e.steps[1].photo_asset_id = Some(a);
    e.steps.reverse();
    assert!(db
        .save_draft(DraftWrite {
            id: draft.id.clone(),
            expected_revision: Some(draft.revision),
            input: e
        })
        .is_err());
    let reread = db.draft(&draft.id).unwrap();
    assert_eq!(reread.revision, draft.revision);
    assert_eq!(
        serde_json::to_value(reread.input).unwrap(),
        serde_json::to_value(draft.input).unwrap()
    );
    healthy(&db);
}

#[test]
fn step_photos_each_pin_and_draft_close_failure_roll_back_the_entire_recipe_transaction() {
    let (_d, mut db, i, a, b) = setup();
    let r = db.save_recipe(i).unwrap();
    let before = raw(&db, &r.id);
    let committed = serde_json::to_value(&r).unwrap();
    let mut e = r.to_input();
    e.title = "Başarısız değişiklik".into();
    e.steps[0].photo_asset_id = Some(b.clone());
    e.steps[1].photo_asset_id = Some(a);
    e.cover_asset_id = Some(b);
    let draft = db
        .save_draft(DraftWrite {
            id: id(),
            expected_revision: None,
            input: e.clone(),
        })
        .unwrap();
    for table in [
        "recipe_step_media",
        "recipe_version_media",
        "recipe_version_alternatives",
        "recipe_version_step_media",
    ] {
        db.conn.execute_batch(&format!("CREATE TRIGGER injected_failure BEFORE INSERT ON {table} BEGIN SELECT RAISE(ABORT,'injected pin'); END;")).unwrap();
        assert!(
            db.commit_recipe(e.clone(), &draft.id, draft.revision, false)
                .is_err(),
            "{table}"
        );
        assert_eq!(raw(&db, &r.id), before);
        assert_eq!(
            serde_json::to_value(db.recipe(&r.id).unwrap()).unwrap(),
            committed
        );
        assert_eq!(db.draft(&draft.id).unwrap().revision, draft.revision);
        healthy(&db);
        db.conn
            .execute_batch("DROP TRIGGER injected_failure")
            .unwrap();
    }
    db.conn.execute_batch("CREATE TRIGGER injected_close BEFORE UPDATE ON editor_drafts WHEN NEW.state='closed' BEGIN SELECT RAISE(ABORT,'injected close'); END;").unwrap();
    assert!(db
        .commit_recipe(e, &draft.id, draft.revision, false)
        .is_err());
    assert_eq!(raw(&db, &r.id), before);
    assert_eq!(db.recipe(&r.id).unwrap().revision, r.revision);
    healthy(&db);
}

#[test]
fn step_photos_two_connections_stale_recipe_and_draft_actions_never_lose_the_winning_photo() {
    let (d, mut db, i, _a, b) = setup();
    let original = db.save_recipe(i).unwrap();
    let mut second = Database::open(d.path()).unwrap();
    let mut e = original.to_input();
    e.steps[1].photo_asset_id = Some(b.clone());
    let winner = db.save_recipe(e).unwrap();
    assert_eq!(
        second.save_recipe(original.to_input()).unwrap_err().code,
        "CONFLICT"
    );
    assert_eq!(
        second.recipe(&winner.id).unwrap().steps[1].photo_asset_id,
        Some(b)
    );
    let draft = db
        .save_draft(DraftWrite {
            id: id(),
            expected_revision: None,
            input: winner.to_input(),
        })
        .unwrap();
    let mut e = draft.input.clone();
    e.steps.reverse();
    let updated = db
        .save_draft(DraftWrite {
            id: draft.id.clone(),
            expected_revision: Some(draft.revision),
            input: e,
        })
        .unwrap();
    assert!(second.discard_draft(&draft.id, draft.revision).is_err());
    assert!(second
        .commit_recipe(draft.input, &draft.id, draft.revision, false)
        .is_err());
    assert_eq!(second.draft(&draft.id).unwrap().revision, updated.revision);
    assert_eq!(versions(&db, &winner.id), 2);
    healthy(&db);
}

#[test]
fn step_photos_historical_missing_files_and_active_drafts_never_silently_restore_or_drop_photos() {
    let (d, mut db, mut i, a, b) = setup();
    i.cover_asset_id = None; // Verify missing step-only media, independently of cover checks.
    let r = db.save_recipe(i).unwrap();
    let mut e = r.to_input();
    e.steps[0].photo_asset_id = Some(b);
    e.cover_asset_id = None;
    let r = db.save_recipe(e).unwrap();
    let before = raw(&db, &r.id);
    let draft = db
        .save_draft(DraftWrite {
            id: id(),
            expected_revision: None,
            input: r.to_input(),
        })
        .unwrap();
    assert_eq!(
        db.restore_recipe_version(restore(&r, 1)).unwrap_err().code,
        "RECIPE_RESTORE_DRAFT"
    );
    db.discard_draft(&draft.id, draft.revision).unwrap();
    let original = d
        .path()
        .join("media-v1/assets")
        .join(a)
        .join("original.png");
    std::fs::rename(&original, original.with_extension("retained-test-copy")).unwrap();
    assert_eq!(
        db.restore_recipe_version(restore(&r, 1)).unwrap_err().code,
        "MEDIA_UNAVAILABLE"
    );
    assert_eq!(raw(&db, &r.id), before);
    assert_eq!(db.recipe(&r.id).unwrap().revision, r.revision);
    healthy(&db);
}

#[test]
fn step_photos_duplicate_variation_lifecycle_and_reviewed_purge_keep_shared_assets_and_orphan_drafts(
) {
    let (d, mut db, i, a, _) = setup();
    let mut r = db.save_recipe(i).unwrap();
    db.save_private_note(PrivateNoteWrite {
        recipe_id: r.id.clone(),
        expected_revision: 0,
        markdown: "M5C2 bağımsız özel not".into(),
    })
    .unwrap();
    db.set_recipe_organization(OrganizationWrite {
        recipe_id: r.id.clone(),
        expected_revision: 1,
        favorite: true,
        to_try: true,
        rating: Some(4),
    })
    .unwrap();
    let child = db.create_variation(&r.id, r.revision).unwrap();
    let copy = db.duplicate_recipe(&r.id, r.revision).unwrap();
    for other in [&child, &copy] {
        assert_ne!(other.steps[0].id, r.steps[0].id);
        assert_eq!(other.steps[0].photo_asset_id, r.steps[0].photo_asset_id);
        assert_eq!(versions(&db, &other.id), 1);
        assert!(!db.recipe_organization(&other.id).unwrap().favorite);
    }
    assert_eq!(db.private_note(&child.id).unwrap().markdown, "");
    assert_eq!(
        db.private_note(&copy.id).unwrap().markdown,
        "M5C2 bağımsız özel not"
    );
    let draft = db
        .save_draft(DraftWrite {
            id: id(),
            expected_revision: None,
            input: r.to_input(),
        })
        .unwrap();
    r = db.archive_recipe(&r.id, r.revision, true).unwrap();
    assert!(db.save_recipe(r.to_input()).is_err());
    assert_eq!(
        db.restore_recipe_version(restore(&r, 1)).unwrap_err().code,
        "RECIPE_RESTORE_INACTIVE"
    );
    r = db.archive_recipe(&r.id, r.revision, false).unwrap();
    r = db.set_deleted(&r.id, r.revision, true).unwrap();
    r = db.set_deleted(&r.id, r.revision, false).unwrap();
    assert_eq!(versions(&db, &r.id), 1);
    assert_eq!(r.steps[0].photo_asset_id, Some(a.clone()));
    r = db.set_deleted(&r.id, r.revision, true).unwrap();
    assert!(db.purge_recipe(&r.id, r.revision).is_err());
    let review = db.preview_recipe_purge(&r.id, r.revision, 0, 20).unwrap();
    let request = || ReviewedPurge {
        recipe_id: r.id.clone(),
        expected_revision: r.revision,
        token: review.token.clone(),
        acknowledge_detach: true,
    };
    db.conn.execute_batch("CREATE TRIGGER reject_step_purge BEFORE DELETE ON recipe_step_media BEGIN SELECT RAISE(ABORT,'injected purge'); END;").unwrap();
    assert!(db.purge_reviewed_recipe(request()).is_err());
    assert!(db.recipe(&r.id).is_ok());
    assert_eq!(versions(&db, &child.id), 1);
    db.conn
        .execute_batch("DROP TRIGGER reject_step_purge")
        .unwrap();
    db.purge_reviewed_recipe(request()).unwrap();
    assert!(db.recipe(&r.id).is_err());
    assert!(db.draft(&draft.id).unwrap().recipe_id.is_none());
    assert_eq!(
        db.draft(&draft.id).unwrap().input.steps[0].photo_asset_id,
        Some(a.clone())
    );
    for other in [&child, &copy] {
        assert_eq!(
            db.recipe(&other.id).unwrap().steps[0].photo_asset_id,
            Some(a.clone())
        );
        assert_eq!(pins(&db, "recipe_version_step_media", &other.id), 1);
    }
    assert!(d
        .path()
        .join("media-v1/assets")
        .join(a)
        .join("original.png")
        .is_file());
    healthy(&db);
}

fn open_schema(root: &Path, version: usize) -> Database {
    let path = root.join("library.sqlite3");
    std::fs::File::create(&path).unwrap();
    let mut conn = Connection::open(&path).unwrap();
    conn.pragma_update(None, "foreign_keys", true).unwrap();
    migrations::apply(&mut conn, &path, &migrations::MIGRATIONS[..version]).unwrap();
    Database { conn }
}
#[test]
fn step_photos_schema_thirteen_upgrade_keeps_all_three_legacy_formats_bytes_checksums_drafts_and_pins(
) {
    let d = tempfile::tempdir().unwrap();
    let path = d.path().join("library.sqlite3");
    let mut db = open_schema(d.path(), 11);
    let mut first = input(&mut db);
    first.ingredients[0].alternatives.clear();
    let v1 = db.save_recipe(first).unwrap();
    assert_eq!(
        db.recipe_version(&v1.id, 1).unwrap().summary.format_version,
        1
    );
    migrations::apply(&mut db.conn, &path, &migrations::MIGRATIONS[..12]).unwrap();
    let second = input(&mut db);
    let v2 = db.save_recipe(second).unwrap();
    assert_eq!(
        db.recipe_version(&v2.id, 1).unwrap().summary.format_version,
        2
    );
    migrations::apply(&mut db.conn, &path, &migrations::MIGRATIONS[..13]).unwrap();
    db.install_bundled_catalog().unwrap();
    let a = asset(&mut db, d.path(), 120);
    let mut third = input(&mut db);
    third.cover_asset_id = Some(a.clone());
    let v3 = db.save_recipe(third).unwrap();
    assert_eq!(
        db.recipe_version(&v3.id, 1).unwrap().summary.format_version,
        3
    );
    let draft = db
        .save_draft(DraftWrite {
            id: id(),
            expected_revision: None,
            input: v3.to_input(),
        })
        .unwrap();
    let originals = [&v1, &v2, &v3].map(|r| raw(&db, &r.id));
    let ledger: Vec<(i64, String)> = db
        .conn
        .prepare("SELECT version,checksum FROM schema_migrations ORDER BY version")
        .unwrap()
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();
    drop(db);
    let mut db = Database::open(d.path()).unwrap();
    assert_eq!(db.bootstrap().unwrap().storage.schema_version, 14);
    assert_eq!(
        db.conn
            .query_row("SELECT count(*) FROM recipe_version_step_media", [], |r| {
                r.get::<_, i64>(0)
            })
            .unwrap(),
        0
    );
    for (r, before) in [&v1, &v2, &v3].into_iter().zip(originals) {
        assert_eq!(raw(&db, &r.id), before);
        assert!(db
            .recipe_version(&r.id, 1)
            .unwrap()
            .snapshot
            .steps
            .iter()
            .all(|s| s.photo_asset_id.is_none()));
        let saved = db
            .save_recipe(db.recipe(&r.id).unwrap().to_input())
            .unwrap();
        assert_eq!(versions(&db, &saved.id), 1);
    }
    for (version, checksum) in ledger {
        assert_eq!(
            db.conn
                .query_row(
                    "SELECT checksum FROM schema_migrations WHERE version=?1",
                    [version],
                    |r| r.get::<_, String>(0)
                )
                .unwrap(),
            checksum
        );
    }
    assert_eq!(db.draft(&draft.id).unwrap().revision, draft.revision);
    db.discard_draft(&draft.id, draft.revision).unwrap();
    let mut e = db.recipe(&v3.id).unwrap().to_input();
    e.steps[0].photo_asset_id = Some(a);
    let r = db.save_recipe(e).unwrap();
    assert_eq!(
        db.recipe_version(&r.id, 2).unwrap().summary.format_version,
        4
    );
    assert_eq!(pins(&db, "recipe_version_media", &r.id), 2);
    assert_eq!(pins(&db, "recipe_version_alternatives", &r.id), 2);
    let restored = db.restore_recipe_version(restore(&r, 1)).unwrap();
    assert!(restored
        .recipe
        .steps
        .iter()
        .all(|s| s.photo_asset_id.is_none()));
    assert!(restored.recipe.cover_asset_id.is_some());
    assert_eq!(restored.version_number, 3);
    let payload: serde_json::Value = serde_json::from_str(&raw(&db, &r.id)[2].1).unwrap();
    assert!(payload["steps"][0].get("photoAssetId").unwrap().is_null());
    assert_eq!(
        db.conn
            .query_row("SELECT count(*) FROM catalog_ingredients", [], |r| r
                .get::<_, i64>(0))
            .unwrap(),
        854
    );
    drop(db);
    healthy(&Database::open(d.path()).unwrap());
}

#[test]
fn step_photos_failed_migration_fourteen_rolls_back_tables_and_preserves_schema_thirteen() {
    let d = tempfile::tempdir().unwrap();
    let path = d.path().join("library.sqlite3");
    let mut db = open_schema(d.path(), 13);
    let i = input(&mut db);
    let r = db.save_recipe(i).unwrap();
    let before = raw(&db, &r.id);
    let bad_sql = Box::leak(
        format!(
            "{}\nSELECT missing_function();",
            migrations::MIGRATIONS[13].sql
        )
        .into_boxed_str(),
    );
    let mut bad: Vec<_> = migrations::MIGRATIONS[..13]
        .iter()
        .map(|m| migrations::Migration {
            version: m.version,
            name: m.name,
            sql: m.sql,
        })
        .collect();
    bad.push(migrations::Migration {
        version: 14,
        name: "recipe_step_media",
        sql: bad_sql,
    });
    assert!(migrations::apply(&mut db.conn, &path, &bad).is_err());
    assert_eq!(
        migrations::validate(&db.conn, &migrations::MIGRATIONS[..13]).unwrap(),
        13
    );
    assert_eq!(raw(&db, &r.id), before);
    assert_eq!(db.conn.query_row("SELECT count(*) FROM sqlite_master WHERE name IN ('recipe_step_media','draft_step_media','recipe_version_step_media')",[],|r|r.get::<_,i64>(0)).unwrap(),0);
    drop(db);
    let db = Database::open(d.path()).unwrap();
    assert_eq!(db.bootstrap().unwrap().storage.schema_version, 14);
    assert_eq!(raw(&db, &r.id), before);
    healthy(&db);
}

#[test]
fn step_photos_five_hundred_steps_share_immutable_bytes_with_bounded_snapshot_and_batched_reads() {
    let (d, mut db, mut i, a, _) = setup();
    i.steps = (0..500)
        .map(|n| RecipeStep {
            id: id(),
            instructions: format!("{n}. adım — İ/ı"),
            photo_asset_id: Some(a.clone()),
            details: None,
            ingredient_line_ids: vec![],
        })
        .collect();
    let start = std::time::Instant::now();
    let mut r = db.save_recipe(i).unwrap();
    let save = start.elapsed();
    assert_eq!(pins(&db, "recipe_step_media", &r.id), 500);
    assert_eq!(pins(&db, "recipe_version_step_media", &r.id), 500);
    assert!(raw(&db, &r.id)[0].1.len() < 200_000);
    let start = std::time::Instant::now();
    for _ in 0..10 {
        assert_eq!(db.recipe(&r.id).unwrap().steps.len(), 500);
    }
    let reads = start.elapsed();
    r = db.save_recipe(r.to_input()).unwrap();
    assert_eq!(versions(&db, &r.id), 1);
    let start = std::time::Instant::now();
    let draft = db
        .save_draft(DraftWrite {
            id: id(),
            expected_revision: None,
            input: r.to_input(),
        })
        .unwrap();
    assert_eq!(
        db.conn
            .query_row(
                "SELECT count(*) FROM draft_step_media WHERE draft_id=?1",
                [draft.id],
                |r| r.get::<_, i64>(0)
            )
            .unwrap(),
        500
    );
    let draft_time = start.elapsed();
    eprintln!(
        "M5C2 500 steps: save={}ms; 10 batched recipe reads={}ms; draft={}ms; snapshot={} bytes",
        save.as_millis(),
        reads.as_millis(),
        draft_time.as_millis(),
        raw(&db, &r.id)[0].1.len()
    );
    assert!(d
        .path()
        .join("media-v1/assets")
        .join(a)
        .join("original.png")
        .is_file());
    healthy(&db);
}

#[test]
fn step_photos_format_two_and_three_capture_guards_still_pin_after_migration_fourteen() {
    let (_d, mut db, i, a, _) = setup();
    let r = db.save_recipe(i).unwrap();
    for (version, format) in [(2, 2), (3, 3)] {
        let mut s = RecipeSnapshot::from_recipe(r.clone());
        s.format_version = format;
        for step in &mut s.steps {
            step.photo_asset_id = None;
        }
        if format == 2 {
            s.cover_asset_id = None;
        }
        s.validate().unwrap();
        let text = serde_json::to_string(&s).unwrap();
        db.conn.execute("INSERT INTO recipe_versions(recipe_id,version_number,format_version,source_revision,provenance,content_fingerprint,snapshot_checksum,snapshot) VALUES(?1,?2,?3,1,'edited',?4,?5,?6)",params![r.id,version,format,super::recipe_versions::fingerprint(&s).unwrap(),migrations::checksum(&text),text]).unwrap();
        assert_eq!(
            db.recipe_version(&r.id, version)
                .unwrap()
                .summary
                .format_version,
            format
        );
    }
    assert_eq!(pins(&db, "recipe_version_alternatives", &r.id), 3);
    assert_eq!(pins(&db, "recipe_version_media", &r.id), 2);
    assert_eq!(pins(&db, "recipe_version_step_media", &r.id), 1);
    assert!(db
        .conn
        .execute(
            "INSERT INTO recipe_version_media(recipe_id,version_number,asset_id) VALUES(?1,2,?2)",
            params![r.id, a]
        )
        .is_err());
    let mut bad = RecipeSnapshot::from_recipe(r);
    bad.format_version = 3;
    assert!(bad.validate().is_err());
    healthy(&db);
}

#[test]
fn step_photos_invalid_historical_pin_aborts_format_four_cover_and_alternative_capture_together() {
    let (_d, mut db, i, _a, _b) = setup();
    let r = db.save_recipe(i).unwrap();
    let before = raw(&db, &r.id);
    let failed = id();
    db.conn
        .execute(
            "INSERT INTO media_assets(id,state) VALUES(?1,'failed')",
            [&failed],
        )
        .unwrap();
    for missing in [id(), failed] {
        let mut snapshot = RecipeSnapshot::from_recipe(r.clone());
        snapshot.steps[0].photo_asset_id = Some(missing);
        let text = serde_json::to_string(&snapshot).unwrap();
        assert!(db.conn.execute("INSERT INTO recipe_versions(recipe_id,version_number,format_version,source_revision,provenance,content_fingerprint,snapshot_checksum,snapshot) VALUES(?1,2,4,2,'edited',?2,?3,?4)",params![r.id,super::recipe_versions::fingerprint(&snapshot).unwrap(),migrations::checksum(&text),text]).is_err());
        assert_eq!(raw(&db, &r.id), before);
        assert_eq!(pins(&db, "recipe_version_media", &r.id), 1);
        assert_eq!(pins(&db, "recipe_version_alternatives", &r.id), 1);
        assert_eq!(pins(&db, "recipe_version_step_media", &r.id), 1);
    }
    healthy(&db);
}

#[test]
fn step_photos_five_hundred_distinct_ready_asset_fixtures_keep_verification_memory_bounded() {
    let (d, mut db, mut i, a, _) = setup();
    let raw_manifest: String = db
        .conn
        .query_row("SELECT manifest FROM media_assets WHERE id=?1", [&a], |r| {
            r.get(0)
        })
        .unwrap();
    let template: crate::media::Manifest = serde_json::from_str(&raw_manifest).unwrap();
    // Load fixtures use distinct ready IDs/manifests and real verified PNG bytes.
    // Hard links avoid manufacturing 2+GB of identical image copies in each CI run;
    // this is NOT an application dedup/import feature or 500 separate camera imports.
    i.steps.clear();
    for n in 0..500 {
        let asset = id();
        let folder = d.path().join("media-v1/assets").join(&asset);
        std::fs::create_dir(&folder).unwrap();
        let mut manifest = template.clone();
        manifest.asset_id = asset.clone();
        for v in &manifest.variants {
            std::fs::hard_link(
                d.path()
                    .join("media-v1/assets")
                    .join(&a)
                    .join(format!("{}.png", v.name)),
                folder.join(format!("{}.png", v.name)),
            )
            .unwrap();
        }
        let raw = serde_json::to_string(&manifest).unwrap();
        std::fs::write(folder.join("manifest.json"), &raw).unwrap();
        db.conn
            .execute(
                "INSERT INTO media_assets(id,state,manifest) VALUES(?1,'ready',?2)",
                params![asset, raw],
            )
            .unwrap();
        i.steps.push(RecipeStep {
            id: id(),
            instructions: format!("Ayrı fotoğraf {n} — Ş/İ/ı"),
            photo_asset_id: Some(asset),
            details: None,
            ingredient_line_ids: vec![],
        });
    }
    let start = std::time::Instant::now();
    let r = db.save_recipe(i).unwrap();
    let save = start.elapsed();
    assert_eq!(pins(&db, "recipe_step_media", &r.id), 500);
    assert_eq!(pins(&db, "recipe_version_step_media", &r.id), 500);
    let start = std::time::Instant::now();
    for _ in 0..10 {
        assert_eq!(db.recipe(&r.id).unwrap().steps.len(), 500);
    }
    let reads = start.elapsed();
    assert_eq!(
        db.recipe_version(&r.id, 1).unwrap().snapshot.steps.len(),
        500
    );
    eprintln!("M5C2 500 distinct ready asset fixtures: save={}ms; 10 recipe reads={}ms; snapshot={} bytes",save.as_millis(),reads.as_millis(),raw(&db,&r.id)[0].1.len());
    healthy(&db);
}

#[test]
fn step_photos_unfinished_new_draft_keeps_photo_until_a_valid_explicit_commit_after_reopen() {
    let (d, mut db, mut i, a, _) = setup();
    i.title.clear();
    i.steps[0].instructions.clear();
    let draft = db
        .save_draft(DraftWrite {
            id: id(),
            expected_revision: None,
            input: i,
        })
        .unwrap();
    assert_eq!(
        db.conn
            .query_row("SELECT count(*) FROM recipes", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        0
    );
    drop(db);
    let mut db = Database::open(d.path()).unwrap();
    let mut recovered = db.draft(&draft.id).unwrap();
    assert_eq!(recovered.input.steps[0].photo_asset_id, Some(a));
    assert!(db
        .commit_recipe(
            recovered.input.clone(),
            &recovered.id,
            recovered.revision,
            false
        )
        .is_err());
    assert_eq!(db.draft(&draft.id).unwrap().revision, draft.revision);
    recovered.input.title = "Yeni fotoğraflı tarif".into();
    recovered.input.steps[0].instructions = "Artık hazır".into();
    let draft = db
        .save_draft(DraftWrite {
            id: draft.id,
            expected_revision: Some(draft.revision),
            input: recovered.input,
        })
        .unwrap();
    let recipe = db
        .commit_recipe(draft.input, &draft.id, draft.revision, false)
        .unwrap();
    assert_eq!(versions(&db, &recipe.id), 1);
    assert!(recipe.steps[0].photo_asset_id.is_some());
    healthy(&db);
}

#[test]
fn step_photos_unsupported_legacy_schema_never_silently_drops_submitted_photo_references() {
    for version in [12, 13] {
        let d = tempfile::tempdir().unwrap();
        let mut db = open_schema(d.path(), version);
        let mut i = input(&mut db);
        i.steps[0].photo_asset_id = Some(id());
        assert!(db.save_recipe(i.clone()).is_err());
        assert!(db
            .save_draft(DraftWrite {
                id: id(),
                expected_revision: None,
                input: i
            })
            .is_err());
        assert_eq!(
            db.conn
                .query_row("SELECT count(*) FROM recipes", [], |r| r.get::<_, i64>(0))
                .unwrap(),
            0
        );
        assert_eq!(
            db.conn
                .query_row("SELECT count(*) FROM editor_drafts", [], |r| r
                    .get::<_, i64>(0))
                .unwrap(),
            0
        );
        healthy(&db);
    }
}
