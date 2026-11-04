use super::*;
use crate::{
    domain::{
        recipe_organization::*, recipe_relations::*, recipe_versions::*, recipe_workspace::*,
        recipes::*, reliability::*,
    },
    media::{ImportGate, Manifest, MediaStore, INPUT_BYTES},
};
use image::{DynamicImage, ImageDecoder, ImageEncoder};
use rusqlite::{params, Connection};
use serde_json::json;
use std::path::{Path, PathBuf};
fn id() -> String {
    uuid::Uuid::new_v4().to_string()
}
fn source(root: &Path, name: &str) -> PathBuf {
    let p = root.join(name);
    let image = image::RgbaImage::from_fn(30, 20, |x, y| {
        image::Rgba([x as u8 * 5, y as u8 * 7, 80, if x == 0 { 0 } else { 200 }])
    });
    DynamicImage::ImageRgba8(image)
        .save_with_format(&p, image::ImageFormat::Png)
        .unwrap();
    p
}
fn input(asset: Option<&str>) -> RecipeInput {
    serde_json::from_value(json!({"id":id(),"expectedRevision":null,"coverAssetId":asset,"title":"M5C1 — İ/ı kapak","description":"Özgün açıklama","kind":"food","servings":"1.250","prepMinutes":null,"cookMinutes":0,"notes":"İçerik notu","ingredients":[],"steps":[]})).unwrap()
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
fn count(db: &Database, id: &str) -> i64 {
    db.conn
        .query_row(
            "SELECT count(*) FROM recipe_versions WHERE recipe_id=?1",
            [id],
            |r| r.get(0),
        )
        .unwrap()
}
fn manifest(db: &Database, id: &str) -> Manifest {
    serde_json::from_str(
        &db.conn
            .query_row::<String, _, _>("SELECT manifest FROM media_assets WHERE id=?1", [id], |r| {
                r.get(0)
            })
            .unwrap(),
    )
    .unwrap()
}
fn asset_path(root: &Path, id: &str, variant: &str) -> PathBuf {
    root.join("media-v1/assets")
        .join(id)
        .join(format!("{variant}.png"))
}
fn raw(db: &Database, id: &str) -> Vec<(i64, String, String, String)> {
    db.conn.prepare("SELECT version_number,snapshot,snapshot_checksum,content_fingerprint FROM recipe_versions WHERE recipe_id=?1 ORDER BY version_number").unwrap().query_map([id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?))).unwrap().collect::<Result<Vec<_>,_>>().unwrap()
}
#[test]
fn media_real_formats_unicode_extension_and_transparency_are_sanitized_without_source_changes() {
    let dir = tempfile::tempdir().unwrap();
    let mut db = Database::open(dir.path()).unwrap();
    let p = source(dir.path(), "İçecek 👩‍🍳.HEIC");
    let original = std::fs::read(&p).unwrap();
    let a = db.import_cover(&p).unwrap();
    assert_eq!(original, std::fs::read(&p).unwrap());
    assert_eq!(a.width, 30);
    let full = image::open(asset_path(dir.path(), &a.id, "original"))
        .unwrap()
        .into_rgba8();
    assert_eq!(full.get_pixel(0, 0)[3], 0);
    assert_eq!(full.get_pixel(1, 0)[3], 200);
    for fmt in [image::ImageFormat::Jpeg, image::ImageFormat::WebP] {
        let p = dir.path().join(format!("{}—yanlış.png", id()));
        DynamicImage::ImageRgb8(image::RgbImage::from_pixel(
            40,
            25,
            image::Rgb([90, 20, 50]),
        ))
        .save_with_format(&p, fmt)
        .unwrap();
        let before = std::fs::read(&p).unwrap();
        let b = db.import_cover(&p).unwrap();
        assert_eq!((b.width, b.height), (40, 25));
        assert_eq!(before, std::fs::read(&p).unwrap());
        assert!(db
            .media_bytes(&b.id, "preview")
            .unwrap()
            .starts_with(b"\x89PNG"));
    }
    assert!(db.media_bytes(&a.id, "original").is_err());
    assert!(!serde_json::to_string(&a).unwrap().contains("HEIC"));
    healthy(&db);
}
#[test]
fn media_exif_orientation_gps_and_icc_are_removed_with_documented_eight_bit_normalization() {
    let dir = tempfile::tempdir().unwrap();
    let mut db = Database::open(dir.path()).unwrap();
    // Little-endian TIFF orientation=6 plus an unmistakable private metadata tail.
    let mut exif = vec![
        b'I', b'I', 42, 0, 8, 0, 0, 0, 1, 0, 0x12, 1, 3, 0, 1, 0, 0, 0, 6, 0, 0, 0, 0, 0, 0, 0,
    ];
    exif.extend_from_slice(b"GPS_PRIVATE_LOCATION");
    let image = image::RgbImage::from_pixel(40, 25, image::Rgb([200, 30, 80]));
    let mut bytes = Vec::new();
    let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut bytes, 100);
    encoder.set_exif_metadata(exif).unwrap();
    encoder
        .set_icc_profile(b"ICC_PRIVATE_TEST_PROFILE".to_vec())
        .unwrap();
    encoder
        .encode(image.as_raw(), 40, 25, image::ExtendedColorType::Rgb8)
        .unwrap();
    let p = dir.path().join("çekim.jpeg");
    std::fs::write(&p, &bytes).unwrap();
    let a = db.import_cover(&p).unwrap();
    assert_eq!((a.width, a.height), (25, 40));
    assert_eq!(std::fs::read(&p).unwrap(), bytes);
    let normalized = std::fs::read(asset_path(dir.path(), &a.id, "original")).unwrap();
    assert!(!normalized.windows(4).any(|v| v == b"eXIf" || v == b"iCCP"));
    assert!(!normalized.windows(3).any(|v| v == b"GPS"));
    let mut decoder =
        image::codecs::png::PngDecoder::new(std::io::Cursor::new(normalized)).unwrap();
    assert!(decoder.exif_metadata().unwrap().is_none());
    assert!(decoder.icc_profile().unwrap().is_none());
    assert_eq!(decoder.color_type(), image::ColorType::Rgba8);
    healthy(&db);
}
fn crc(bytes: &[u8]) -> u32 {
    let mut c = 0xffffffffu32;
    for b in bytes {
        c ^= u32::from(*b);
        for _ in 0..8 {
            c = (c >> 1) ^ (0xedb88320u32 & 0u32.wrapping_sub(c & 1));
        }
    }
    !c
}
#[test]
fn media_rejects_corrupt_truncated_animated_heic_byte_and_pixel_bombs_before_import_authorization()
{
    let dir = tempfile::tempdir().unwrap();
    let mut db = Database::open(dir.path()).unwrap();
    let png = source(dir.path(), "valid.png");
    let bytes = std::fs::read(&png).unwrap();
    for bad in [
        b"not a photo".to_vec(),
        b"\0\0\0\x18ftypheic\0\0\0\0heic".to_vec(),
        bytes[..bytes.len() - 2].to_vec(),
    ] {
        let p = dir.path().join(id());
        std::fs::write(&p, bad).unwrap();
        assert!(db.import_cover(&p).is_err());
    }
    let oversized = dir.path().join("big.jpg");
    std::fs::File::create(&oversized)
        .unwrap()
        .set_len(INPUT_BYTES + 1)
        .unwrap();
    assert_eq!(db.import_cover(&oversized).unwrap_err().code, "MEDIA_LIMIT");
    let boundary = dir.path().join("exact.jpg");
    std::fs::File::create(&boundary)
        .unwrap()
        .set_len(INPUT_BYTES)
        .unwrap();
    assert_ne!(db.import_cover(&boundary).unwrap_err().code, "MEDIA_LIMIT");
    let mut bomb = bytes.clone();
    bomb[16..20].copy_from_slice(&10000u32.to_be_bytes());
    bomb[20..24].copy_from_slice(&10000u32.to_be_bytes());
    let hash = crc(&bomb[12..29]);
    bomb[29..33].copy_from_slice(&hash.to_be_bytes());
    let p = dir.path().join("bomb.png");
    std::fs::write(&p, bomb).unwrap();
    assert_eq!(db.import_cover(&p).unwrap_err().code, "MEDIA_LIMIT");
    let mut animated = bytes.clone();
    let mut chunk = Vec::from(&b"\0\0\0\x08acTL\0\0\0\x02\0\0\0\0"[..]);
    let c = crc(&chunk[4..]);
    chunk.extend(c.to_be_bytes());
    animated.splice(33..33, chunk);
    let p = dir.path().join("animated.png");
    std::fs::write(&p, animated).unwrap();
    assert_eq!(db.import_cover(&p).unwrap_err().code, "MEDIA_UNSUPPORTED");
    assert_eq!(
        db.conn
            .query_row(
                "SELECT count(*) FROM media_assets WHERE state='ready'",
                [],
                |r| r.get::<_, i64>(0)
            )
            .unwrap(),
        0
    );
    healthy(&db);
}
#[test]
fn media_protocol_keys_and_app_owned_paths_reject_traversal_reserved_names_and_unknown_variants() {
    for key in [
        "../../library.sqlite3",
        "%2e%2e",
        "CON",
        "nul",
        "C:\\Users\\private",
        "00000000-0000-4000-8000-000000000000/../../x",
    ] {
        let uri = format!("recipe-media://localhost/{key}/thumb").parse::<tauri::http::Uri>();
        assert!(uri.is_err() || crate::media::request_key(&uri.unwrap()).is_err());
    }
    let uuid = id();
    for suffix in ["original", "thumb/extra", "thumb?path=/x", "thumb%2f.."] {
        let uri = format!("recipe-media://localhost/{uuid}/{suffix}")
            .parse()
            .unwrap();
        assert!(crate::media::request_key(&uri).is_err());
    }
    for uri in [
        format!("recipe-media://localhost/{uuid}/thumb"),
        format!("http://recipe-media.localhost/{uuid}/preview"),
    ] {
        assert!(crate::media::request_key(&uri.parse().unwrap()).is_ok());
    }
    let gate = ImportGate::default();
    let token = gate.acquire().unwrap();
    assert!(gate.acquire().is_err());
    drop(token);
    assert!(gate.acquire().is_ok());
    let dir = tempfile::tempdir().unwrap();
    let store = MediaStore::new(dir.path()).unwrap();
    assert!(store.clean_staging("../").is_err());
}
#[cfg(unix)]
#[test]
fn media_symlink_source_ancestor_staging_and_final_file_are_never_followed() {
    use std::os::unix::fs::symlink;
    let dir = tempfile::tempdir().unwrap();
    let mut db = Database::open(dir.path()).unwrap();
    let p = source(dir.path(), "real.png");
    let link = dir.path().join("link.png");
    symlink(&p, &link).unwrap();
    assert_eq!(db.import_cover(&link).unwrap_err().code, "MEDIA_PATH");
    let parent = dir.path().join("linkdir");
    symlink(dir.path(), &parent).unwrap();
    assert!(db.import_cover(&parent.join("real.png")).is_err());
    let a = db.import_cover(&p).unwrap();
    let preview = asset_path(dir.path(), &a.id, "preview");
    std::fs::remove_file(&preview).unwrap();
    symlink(&p, &preview).unwrap();
    assert!(db.media_bytes(&a.id, "preview").is_err());
    assert!(db.save_recipe(input(Some(&a.id))).is_err());
    let unknown = id();
    let root = dir.path().join("media-v1/staging").join(&unknown);
    symlink(dir.path(), &root).unwrap();
    assert!(MediaStore::new(dir.path())
        .unwrap()
        .prepare(&unknown, &p)
        .is_err());
    assert!(p.exists());
    healthy(&db);
}
#[test]
fn media_cover_save_noop_history_compare_restore_and_reference_rows_are_atomic_and_immutable() {
    let dir = tempfile::tempdir().unwrap();
    let mut db = Database::open(dir.path()).unwrap();
    let p = source(dir.path(), "a.png");
    let a = db.import_cover(&p).unwrap();
    let b = db.import_cover(&p).unwrap();
    let mut r = db.save_recipe(input(Some(&a.id))).unwrap();
    let first = raw(&db, &r.id);
    assert_eq!(
        db.recipe_version(&r.id, 1).unwrap().summary.format_version,
        3
    );
    r = db.save_recipe(r.to_input()).unwrap();
    assert_eq!(count(&db, &r.id), 1);
    let mut edit = r.to_input();
    edit.cover_asset_id = Some(b.id.clone());
    r = db.save_recipe(edit).unwrap();
    assert_eq!(count(&db, &r.id), 2);
    assert!(db
        .compare_recipe_versions(&r.id, 1, None)
        .unwrap()
        .changes
        .iter()
        .any(|c| c.path == ["coverAssetId"]));
    let restored = db
        .restore_recipe_version(RestoreVersion {
            recipe_id: r.id.clone(),
            version_number: 1,
            expected_revision: r.revision,
        })
        .unwrap();
    r = restored.recipe;
    assert!(restored.changed);
    assert_eq!(r.cover_asset_id.as_deref(), Some(a.id.as_str()));
    assert_eq!(
        db.recipe_version(&r.id, 3).unwrap().summary.provenance,
        "restored"
    );
    assert_eq!(&raw(&db, &r.id)[..1], first.as_slice());
    let mut remove = r.to_input();
    remove.cover_asset_id = None;
    r = db.save_recipe(remove).unwrap();
    assert!(r.cover_asset_id.is_none());
    assert_eq!(count(&db, &r.id), 4);
    MediaStore::new(dir.path())
        .unwrap()
        .verify(&manifest(&db, &a.id), false)
        .unwrap();
    for sql in [
        "DELETE FROM media_assets",
        "UPDATE media_assets SET state='failed' WHERE state='ready'",
        "INSERT OR REPLACE INTO media_assets SELECT * FROM media_assets",
        "DELETE FROM recipe_version_media",
        "UPDATE recipe_version_media SET asset_id=asset_id",
        "INSERT OR REPLACE INTO recipe_version_media SELECT * FROM recipe_version_media",
    ] {
        assert!(db.conn.execute(sql, []).is_err(), "{sql}");
    }
    healthy(&db);
}
#[test]
fn media_durable_drafts_survive_reopen_discard_and_explicit_commit_without_unintended_versions() {
    let dir = tempfile::tempdir().unwrap();
    let mut db = Database::open(dir.path()).unwrap();
    let p = source(dir.path(), "draft.png");
    let a = db.import_cover(&p).unwrap();
    let mut i = input(Some(&a.id));
    i.title = "".into();
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
    let mut db = Database::open(dir.path()).unwrap();
    let recovered = db.draft(&draft.id).unwrap();
    assert_eq!(
        recovered.input.cover_asset_id.as_deref(),
        Some(a.id.as_str())
    );
    let mut i = recovered.input;
    i.title = "Kurtarılan kapak".into();
    let d = db
        .save_draft(DraftWrite {
            id: draft.id,
            expected_revision: Some(draft.revision),
            input: i.clone(),
        })
        .unwrap();
    let mut r = db.commit_recipe(i, &d.id, d.revision, false).unwrap();
    assert_eq!(count(&db, &r.id), 1);
    let mut remove = r.to_input();
    remove.cover_asset_id = None;
    let d = db
        .save_draft(DraftWrite {
            id: id(),
            expected_revision: None,
            input: remove,
        })
        .unwrap();
    db.discard_draft(&d.id, d.revision).unwrap();
    r = db.recipe(&r.id).unwrap();
    assert_eq!(r.cover_asset_id, Some(a.id));
    assert_eq!(count(&db, &r.id), 1);
    assert_eq!(
        db.conn
            .query_row("SELECT count(*) FROM draft_media", [], |r| r
                .get::<_, i64>(0))
            .unwrap(),
        0
    );
    healthy(&db);
}
#[test]
fn media_duplicate_variation_and_reviewed_purge_keep_shared_history_drafts_and_independent_metadata(
) {
    let dir = tempfile::tempdir().unwrap();
    let mut db = Database::open(dir.path()).unwrap();
    let p = source(dir.path(), "shared.png");
    let a = db.import_cover(&p).unwrap();
    let mut parent = db.save_recipe(input(Some(&a.id))).unwrap();
    db.save_private_note(PrivateNoteWrite {
        recipe_id: parent.id.clone(),
        expected_revision: 0,
        markdown: "M5C1 NOTE".into(),
    })
    .unwrap();
    db.set_recipe_organization(OrganizationWrite {
        recipe_id: parent.id.clone(),
        expected_revision: 1,
        favorite: true,
        to_try: false,
        rating: Some(5),
    })
    .unwrap();
    let child = db.create_variation(&parent.id, parent.revision).unwrap();
    let duplicate = db.duplicate_recipe(&parent.id, parent.revision).unwrap();
    assert_eq!(child.cover_asset_id, parent.cover_asset_id);
    assert_eq!(duplicate.cover_asset_id, parent.cover_asset_id);
    assert_eq!(count(&db, &child.id), 1);
    assert_eq!(count(&db, &duplicate.id), 1);
    assert_eq!(db.private_note(&child.id).unwrap().markdown, "");
    assert_eq!(
        db.private_note(&duplicate.id).unwrap().markdown,
        "M5C1 NOTE"
    );
    assert!(!db.recipe_organization(&child.id).unwrap().favorite);
    let draft = db
        .save_draft(DraftWrite {
            id: id(),
            expected_revision: None,
            input: child.to_input(),
        })
        .unwrap();
    parent = db
        .archive_recipe(&parent.id, parent.revision, true)
        .unwrap();
    assert_eq!(parent.cover_asset_id, Some(a.id.clone()));
    parent = db
        .archive_recipe(&parent.id, parent.revision, false)
        .unwrap();
    parent = db.set_deleted(&parent.id, parent.revision, true).unwrap();
    parent = db.set_deleted(&parent.id, parent.revision, false).unwrap();
    assert_eq!(count(&db, &parent.id), 1);
    parent = db.set_deleted(&parent.id, parent.revision, true).unwrap();
    assert!(db.purge_recipe(&parent.id, parent.revision).is_err());
    let preview = db
        .preview_recipe_purge(&parent.id, parent.revision, 0, 20)
        .unwrap();
    db.purge_reviewed_recipe(ReviewedPurge {
        recipe_id: parent.id.clone(),
        expected_revision: parent.revision,
        token: preview.token,
        acknowledge_detach: true,
    })
    .unwrap();
    assert!(db.recipe(&parent.id).is_err());
    assert_eq!(
        db.recipe(&child.id).unwrap().cover_asset_id,
        Some(a.id.clone())
    );
    assert_eq!(
        db.draft(&draft.id).unwrap().input.cover_asset_id,
        Some(a.id.clone())
    );
    assert!(db.media_bytes(&a.id, "preview").is_ok());
    assert_eq!(count(&db, &duplicate.id), 1);
    healthy(&db);
}
#[test]
fn media_recipe_snapshot_and_draft_close_failures_and_two_connection_cas_roll_back_associations() {
    let dir = tempfile::tempdir().unwrap();
    let mut db = Database::open(dir.path()).unwrap();
    let p = source(dir.path(), "atomic.png");
    let a = db.import_cover(&p).unwrap();
    let b = db.import_cover(&p).unwrap();
    let r = db.save_recipe(input(Some(&a.id))).unwrap();
    let mut edit = r.to_input();
    edit.cover_asset_id = Some(b.id.clone());
    let draft = db
        .save_draft(DraftWrite {
            id: id(),
            expected_revision: None,
            input: edit.clone(),
        })
        .unwrap();
    let before = raw(&db, &r.id);
    for sql in ["CREATE TRIGGER injected BEFORE INSERT ON recipe_versions BEGIN SELECT RAISE(ABORT,'injected'); END;","CREATE TRIGGER injected BEFORE UPDATE OF state ON editor_drafts WHEN NEW.state='closed' BEGIN SELECT RAISE(ABORT,'injected'); END;"]{db.conn.execute_batch(sql).unwrap();assert!(db.commit_recipe(edit.clone(),&draft.id,draft.revision,false).is_err());assert_eq!(db.recipe(&r.id).unwrap().cover_asset_id,Some(a.id.clone()));assert_eq!(raw(&db,&r.id),before);assert_eq!(db.draft(&draft.id).unwrap().revision,draft.revision);db.conn.execute_batch("DROP TRIGGER injected").unwrap();}
    db.discard_draft(&draft.id, draft.revision).unwrap();
    let mut other = Database::open(dir.path()).unwrap();
    let saved = db.save_recipe(edit.clone()).unwrap();
    assert!(other.save_recipe(edit).is_err());
    assert_eq!(other.recipe(&r.id).unwrap().cover_asset_id, Some(b.id));
    assert_eq!(saved.revision, r.revision + 1);
    healthy(&db);
}
#[test]
fn media_recovery_finalizes_complete_journals_and_only_cleans_unreferenced_staging_after_interruption(
) {
    for finalized in [false, true] {
        let dir = tempfile::tempdir().unwrap();
        let db = Database::open(dir.path()).unwrap();
        let p = source(dir.path(), "crash.png");
        let uid = id();
        db.conn
            .execute(
                "INSERT INTO media_assets(id,state) VALUES(?1,'writing')",
                [&uid],
            )
            .unwrap();
        let store = MediaStore::new(dir.path()).unwrap();
        let m = store.prepare(&uid, &p).unwrap();
        if finalized {
            store.finalize(&m).unwrap();
        }
        drop(db);
        let db = Database::open(dir.path()).unwrap();
        assert!(db.media_bytes(&uid, "thumb").is_ok());
        assert_eq!(manifest(&db, &uid), m);
        healthy(&db);
    }
    let dir = tempfile::tempdir().unwrap();
    let db = Database::open(dir.path()).unwrap();
    let uid = id();
    db.conn
        .execute(
            "INSERT INTO media_assets(id,state) VALUES(?1,'writing')",
            [&uid],
        )
        .unwrap();
    let stage = dir.path().join("media-v1/staging").join(&uid);
    std::fs::create_dir(&stage).unwrap();
    std::fs::write(stage.join("original.png"), b"interrupted").unwrap();
    drop(db);
    let db = Database::open(dir.path()).unwrap();
    assert_eq!(
        db.conn
            .query_row("SELECT state FROM media_assets WHERE id=?1", [&uid], |r| {
                r.get::<_, String>(0)
            })
            .unwrap(),
        "failed"
    );
    assert!(!stage.exists());
    healthy(&db);
}
#[test]
fn media_finalize_sql_failure_recovers_on_restart_and_missing_referenced_bytes_never_silently_restore(
) {
    let dir = tempfile::tempdir().unwrap();
    let mut db = Database::open(dir.path()).unwrap();
    let p = source(dir.path(), "recover.png");
    db.conn.execute_batch("CREATE TRIGGER fail_ready BEFORE UPDATE ON media_assets WHEN NEW.state='ready' BEGIN SELECT RAISE(ABORT,'injected'); END;").unwrap();
    assert!(db.import_cover(&p).is_err());
    let uid: String = db
        .conn
        .query_row(
            "SELECT id FROM media_assets WHERE state='writing'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    db.conn.execute_batch("DROP TRIGGER fail_ready").unwrap();
    drop(db);
    let mut db = Database::open(dir.path()).unwrap();
    assert!(db.media_bytes(&uid, "preview").is_ok());
    let mut r = db.save_recipe(input(Some(&uid))).unwrap();
    let mut remove = r.to_input();
    remove.cover_asset_id = None;
    r = db.save_recipe(remove).unwrap();
    let before = raw(&db, &r.id);
    std::fs::remove_file(asset_path(dir.path(), &uid, "original")).unwrap();
    assert_eq!(
        db.restore_recipe_version(RestoreVersion {
            recipe_id: r.id.clone(),
            version_number: 1,
            expected_revision: r.revision
        })
        .unwrap_err()
        .code,
        "MEDIA_UNAVAILABLE"
    );
    assert_eq!(raw(&db, &r.id), before);
    assert!(db.recipe(&r.id).unwrap().cover_asset_id.is_none());
    healthy(&db);
}
#[test]
fn media_schema_twelve_upgrade_preserves_v1_v2_bytes_drafts_exact_quantities_and_catalog() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("library.sqlite3");
    std::fs::File::create(&path).unwrap();
    let mut conn = Connection::open(&path).unwrap();
    conn.pragma_update(None, "foreign_keys", true).unwrap();
    migrations::apply(&mut conn, &path, &migrations::MIGRATIONS[..12]).unwrap();
    let mut db = Database { conn };
    db.install_bundled_catalog().unwrap();
    let mut r = db.save_recipe(input(None)).unwrap();
    assert_eq!(
        db.recipe_version(&r.id, 1).unwrap().summary.format_version,
        2
    );
    let legacy_id = id();
    db.conn
        .execute(
            "INSERT INTO recipes(id,title,kind,servings) VALUES(?1,'Eski V1','beverage','1.250')",
            [&legacy_id],
        )
        .unwrap();
    let mut s = RecipeSnapshot::from_recipe(db.recipe(&legacy_id).unwrap());
    s.format_version = 1;
    let text = serde_json::to_string(&s).unwrap();
    db.conn.execute("INSERT INTO recipe_versions(recipe_id,version_number,format_version,source_revision,provenance,content_fingerprint,snapshot_checksum,snapshot) VALUES(?1,1,1,1,'created',?2,?3,?4)",params![legacy_id,super::recipe_versions::fingerprint(&s).unwrap(),migrations::checksum(&text),text]).unwrap();
    let draft = db
        .save_draft(DraftWrite {
            id: id(),
            expected_revision: None,
            input: r.to_input(),
        })
        .unwrap();
    let before = raw(&db, &r.id);
    let legacy = raw(&db, &legacy_id);
    drop(db);
    let mut db = Database::open(dir.path()).unwrap();
    assert_eq!(db.bootstrap().unwrap().storage.schema_version, 13);
    assert_eq!(raw(&db, &r.id), before);
    assert_eq!(raw(&db, &legacy_id), legacy);
    assert_eq!(db.draft(&draft.id).unwrap().revision, draft.revision);
    assert_eq!(db.recipe(&r.id).unwrap().servings, "1.250");
    db.discard_draft(&draft.id, draft.revision).unwrap();
    r = db
        .save_recipe(db.recipe(&r.id).unwrap().to_input())
        .unwrap();
    assert_eq!(count(&db, &r.id), 1);
    assert_eq!(
        db.conn
            .query_row("SELECT count(*) FROM catalog_ingredients", [], |r| r
                .get::<_, i64>(0))
            .unwrap(),
        854
    );
    let p = source(dir.path(), "new.png");
    let a = db.import_cover(&p).unwrap();
    let mut edit = r.to_input();
    edit.cover_asset_id = Some(a.id);
    r = db.save_recipe(edit).unwrap();
    let restored = db
        .restore_recipe_version(RestoreVersion {
            recipe_id: r.id.clone(),
            version_number: 1,
            expected_revision: r.revision,
        })
        .unwrap();
    assert!(restored.recipe.cover_asset_id.is_none());
    assert_eq!(restored.version_number, 3);
    assert_eq!(&raw(&db, &r.id)[..1], before.as_slice());
    healthy(&db);
}
#[test]
fn media_failed_migration_thirteen_is_atomic_and_retry_preserves_schema_twelve_data() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("library.sqlite3");
    std::fs::File::create(&path).unwrap();
    let mut conn = Connection::open(&path).unwrap();
    conn.pragma_update(None, "foreign_keys", true).unwrap();
    migrations::apply(&mut conn, &path, &migrations::MIGRATIONS[..12]).unwrap();
    let mut bad = migrations::MIGRATIONS[..12]
        .iter()
        .map(|m| migrations::Migration {
            version: m.version,
            name: m.name,
            sql: m.sql,
        })
        .collect::<Vec<_>>();
    bad.push(migrations::Migration {
        version: 13,
        name: "injected",
        sql: "CREATE TABLE should_rollback(id); SELECT missing_function();",
    });
    assert!(migrations::apply(&mut conn, &path, &bad).is_err());
    assert_eq!(
        migrations::validate(&conn, &migrations::MIGRATIONS[..12]).unwrap(),
        12
    );
    assert_eq!(
        conn.query_row(
            "SELECT count(*) FROM sqlite_master WHERE name='should_rollback'",
            [],
            |r| r.get::<_, i64>(0)
        )
        .unwrap(),
        0
    );
    drop(conn);
    let db = Database::open(dir.path()).unwrap();
    assert_eq!(db.bootstrap().unwrap().storage.schema_version, 13);
    healthy(&db);
}

#[test]
fn media_full_thirty_megapixel_input_and_bounded_preview_are_processed_offline() {
    let dir = tempfile::tempdir().unwrap();
    let mut db = Database::open(dir.path()).unwrap();
    let p = dir.path().join("30MP.png");
    DynamicImage::ImageRgba8(image::RgbaImage::from_pixel(
        6000,
        5000,
        image::Rgba([80, 130, 90, 180]),
    ))
    .save_with_format(&p, image::ImageFormat::Png)
    .unwrap();
    assert!(p.metadata().unwrap().len() < INPUT_BYTES);
    let start = std::time::Instant::now();
    let a = db.import_cover(&p).unwrap();
    println!(
        "M5C1_RESOURCE import_30mp_ms={} input_bytes={}",
        start.elapsed().as_millis(),
        p.metadata().unwrap().len()
    );
    assert_eq!((a.width, a.height), (6000, 5000));
    let preview = image::load_from_memory(&db.media_bytes(&a.id, "preview").unwrap()).unwrap();
    assert!(preview.width() <= 1280 && preview.height() <= 1280);
    let thumb = image::load_from_memory(&db.media_bytes(&a.id, "thumb").unwrap()).unwrap();
    assert!(thumb.width() <= 384 && thumb.height() <= 384);
    healthy(&db);
}
#[test]
fn media_filesystem_failure_never_overwrites_existing_files_or_authorizes_a_partial_asset() {
    let dir = tempfile::tempdir().unwrap();
    let db = Database::open(dir.path()).unwrap();
    let p = source(dir.path(), "source.png");
    let uid = id();
    let stage = dir.path().join("media-v1/staging").join(&uid);
    std::fs::create_dir(&stage).unwrap();
    std::fs::write(stage.join("original.png"), b"existing obstacle").unwrap();
    assert!(MediaStore::new(dir.path())
        .unwrap()
        .prepare(&uid, &p)
        .is_err());
    assert_eq!(
        std::fs::read(stage.join("original.png")).unwrap(),
        b"existing obstacle"
    );
    db.conn
        .execute(
            "INSERT INTO media_assets(id,state) VALUES(?1,'writing')",
            [&uid],
        )
        .unwrap();
    let r = id();
    db.conn
        .execute(
            "INSERT INTO recipes(id,title,kind,servings) VALUES(?1,'partially staged','food','1')",
            [&r],
        )
        .unwrap();
    assert!(db
        .conn
        .execute(
            "INSERT INTO recipe_covers(recipe_id,asset_id) VALUES(?1,?2)",
            params![r, uid]
        )
        .is_err());
    healthy(&db);
}

#[test]
fn media_missing_previews_regenerate_from_verified_original_without_rewriting_asset_or_history() {
    let dir = tempfile::tempdir().unwrap();
    let mut db = Database::open(dir.path()).unwrap();
    let p = source(dir.path(), "cache.png");
    let a = db.import_cover(&p).unwrap();
    let r = db.save_recipe(input(Some(&a.id))).unwrap();
    let original = std::fs::read(asset_path(dir.path(), &a.id, "original")).unwrap();
    let m = manifest(&db, &a.id);
    let snapshots = raw(&db, &r.id);
    for variant in ["thumb", "preview"] {
        let expected = db.media_bytes(&a.id, variant).unwrap();
        std::fs::remove_file(asset_path(dir.path(), &a.id, variant)).unwrap();
        assert_eq!(db.media_bytes(&a.id, variant).unwrap(), expected);
        assert_eq!(db.media_bytes(&a.id, variant).unwrap(), expected);
    }
    assert_eq!(manifest(&db, &a.id), m);
    assert_eq!(raw(&db, &r.id), snapshots);
    assert_eq!(
        std::fs::read(asset_path(dir.path(), &a.id, "original")).unwrap(),
        original
    );
    let saved = db.save_recipe(r.to_input()).unwrap();
    assert_eq!(count(&db, &saved.id), 1);
    healthy(&db);
}
#[test]
fn media_ready_directory_recovery_handles_an_interrupted_rename_without_mutating_ready_metadata() {
    let dir = tempfile::tempdir().unwrap();
    let mut db = Database::open(dir.path()).unwrap();
    let p = source(dir.path(), "rename.png");
    let a = db.import_cover(&p).unwrap();
    let m = manifest(&db, &a.id);
    std::fs::rename(
        dir.path().join("media-v1/assets").join(&a.id),
        dir.path().join("media-v1/staging").join(&a.id),
    )
    .unwrap();
    assert!(db.media_bytes(&a.id, "thumb").is_ok());
    assert_eq!(manifest(&db, &a.id), m);
    healthy(&db);
}
