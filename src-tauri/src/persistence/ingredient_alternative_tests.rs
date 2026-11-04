use super::*;
use crate::domain::{
    recipe_organization::*, recipe_relations::*, recipe_versions::*, recipe_workspace::*,
    recipes::*, reliability::*,
};
use rusqlite::Connection;
fn id() -> String {
    uuid::Uuid::new_v4().to_string()
}
fn input(primary: &str, alt: &str) -> RecipeInput {
    let line = id();
    serde_json::from_value(serde_json::json!({"id":id(),"expectedRevision":null,"title":"M5B4 — Şişli I/ı İ/i","description":null,"kind":"food","servings":"1","prepMinutes":null,"cookMinutes":null,"notes":"İçerik","ingredients":[{"id":line,"ingredientId":primary,"quantity":"1.250","unitCode":"g","note":null,"alternatives":[{"id":id(),"ingredientId":alt,"quantity":"0.123456","unitCode":"mL"}]}],"steps":[{"id":id(),"instructions":"Hazırla","details":{"formatVersion":1,"title":"İlk","durationMinutes":"1","temperatureValue":null,"temperatureUnit":null,"heatLevel":null,"equipment":null,"technique":null,"tips":null},"ingredientLineIds":[line]}]})).unwrap()
}
fn setup() -> (tempfile::TempDir, Database, Recipe, String) {
    let dir = tempfile::tempdir().unwrap();
    let mut db = Database::open(dir.path()).unwrap();
    let p = db.create_ingredient("Ana malzeme").unwrap();
    let a = db.create_ingredient("Alternatif malzeme").unwrap();
    let r = db.save_recipe(input(&p.id, &a.id)).unwrap();
    (dir, db, r, a.id)
}
fn versions(db: &Database, r: &Recipe) -> usize {
    db.recipe_versions(&r.id, None, 100).unwrap().len()
}
fn health(db: &Database) {
    assert_eq!(
        db.conn
            .query_row("pragma integrity_check", [], |r| r.get::<_, String>(0))
            .unwrap(),
        "ok"
    );
    assert!(db
        .conn
        .prepare("pragma foreign_key_check")
        .unwrap()
        .query([])
        .unwrap()
        .next()
        .unwrap()
        .is_none());
}
#[test]
fn alternative_aggregate_survives_unrelated_saves_reorder_noop_and_restart() {
    let (dir, mut db, mut r, alt) = setup();
    let b = db.create_ingredient("İkinci alternatif").unwrap();
    let mut edit = r.to_input();
    let mut line = edit.ingredients[0].clone();
    line.id = id();
    for a in &mut line.alternatives {
        a.id = id();
        a.quantity = None;
        a.unit_code = None;
    }
    edit.ingredients.push(line);
    edit.ingredients[0]
        .alternatives
        .push(IngredientAlternative {
            id: id(),
            ingredient_id: b.id,
            quantity: None,
            unit_code: Some("g".into()),
        });
    r = db.save_recipe(edit).unwrap();
    assert_eq!(versions(&db, &r), 2);
    let before = serde_json::to_value(&r.ingredients).unwrap();
    r = db.save_recipe(r.to_input()).unwrap();
    assert_eq!(versions(&db, &r), 2);
    assert_eq!(serde_json::to_value(&r.ingredients).unwrap(), before);
    let mut edit = r.to_input();
    edit.notes = Some("Başka alan".into());
    edit.ingredients.reverse();
    edit.ingredients[1].alternatives.reverse();
    r = db.save_recipe(edit).unwrap();
    assert_eq!(versions(&db, &r), 3);
    let expected = serde_json::to_value(&r).unwrap();
    drop(db);
    let db = Database::open(dir.path()).unwrap();
    assert_eq!(
        serde_json::to_value(db.recipe(&r.id).unwrap()).unwrap(),
        expected
    );
    assert_eq!(r.ingredients[0].alternatives[0].ingredient_id, alt);
    assert_eq!(r.ingredients[1].quantity.as_deref(), Some("1.250"));
    assert_eq!(
        r.ingredients[1].alternatives[1].quantity.as_deref(),
        Some("0.123456")
    );
    assert!(db
        .recipe_version(&r.id, 3)
        .unwrap()
        .snapshot
        .ingredient_names
        .contains_key(&alt));
    health(&db);
}
#[test]
fn alternative_draft_recovery_commit_discard_and_unit_references_are_real() {
    let (dir, mut db, r, alt) = setup();
    let mut edit = r.to_input();
    edit.ingredients[0].alternatives[0].quantity = Some("0.".into());
    db.conn.execute("INSERT INTO units(code,dimension,canonical_code,factor) VALUES('alt-unit','mass','g',1)",[]).unwrap();
    edit.ingredients[0].alternatives[0].unit_code = Some("alt-unit".into());
    let draft = db
        .save_draft(DraftWrite {
            id: id(),
            expected_revision: None,
            input: edit,
        })
        .unwrap();
    assert_eq!(versions(&db, &r), 1);
    drop(db);
    let mut db = Database::open(dir.path()).unwrap();
    let draft = db.draft(&draft.id).unwrap();
    assert_eq!(
        draft.input.ingredients[0].alternatives[0]
            .quantity
            .as_deref(),
        Some("0.")
    );
    assert_eq!(draft.ingredient_names[&alt], "Alternatif malzeme");
    assert!(db
        .conn
        .execute("DELETE FROM units WHERE code='alt-unit'", [])
        .is_err());
    let mut valid = draft.input.clone();
    valid.ingredients[0].alternatives[0].quantity = Some("0.123456".into());
    let r = db
        .commit_recipe(valid, &draft.id, draft.revision, false)
        .unwrap();
    assert_eq!(versions(&db, &r), 2);
    assert!(db.draft(&draft.id).is_err());
    assert_eq!(
        db.conn
            .query_row("select count(*) from draft_alternative_units", [], |r| r
                .get::<_, i64>(0))
            .unwrap(),
        0
    );
    health(&db);
}
#[test]
fn alternative_history_mixed_formats_restore_and_reference_protection() {
    let (_dir, mut db, mut r, alt) = setup();
    let bytes: String = db
        .conn
        .query_row(
            "select snapshot from recipe_versions where recipe_id=?1 and version_number=1",
            [&r.id],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(
        db.recipe_version(&r.id, 1).unwrap().summary.format_version,
        2
    );
    let mut edit = r.to_input();
    edit.ingredients[0].alternatives.clear();
    r = db.save_recipe(edit).unwrap();
    assert_eq!(versions(&db, &r), 2);
    assert_eq!(
        db.delete_ingredient(&alt, 1).unwrap_err().code,
        "INGREDIENT_REFERENCED"
    );
    let diff = db.compare_recipe_versions(&r.id, 1, None).unwrap();
    assert!(diff
        .changes
        .iter()
        .any(|d| d.path.iter().any(|p| p == "alternatives")));
    r = db
        .restore_recipe_version(RestoreVersion {
            recipe_id: r.id.clone(),
            version_number: 1,
            expected_revision: r.revision,
        })
        .unwrap()
        .recipe;
    assert_eq!(versions(&db, &r), 3);
    assert_eq!(
        r.ingredients[0].alternatives[0].quantity.as_deref(),
        Some("0.123456")
    );
    assert_eq!(
        db.conn
            .query_row(
                "select snapshot from recipe_versions where recipe_id=?1 and version_number=1",
                [&r.id],
                |r| r.get::<_, String>(0)
            )
            .unwrap(),
        bytes
    );
    db.edit_ingredient(IngredientEdit {
        id: alt.clone(),
        revision: 1,
        name: "Yeni isim".into(),
        notes: None,
        preferred_unit: None,
        category_ids: None,
    })
    .unwrap();
    assert_eq!(
        db.recipe_version(&r.id, 1)
            .unwrap()
            .snapshot
            .ingredient_names[&alt],
        "Alternatif malzeme"
    );
    let no = db
        .restore_recipe_version(RestoreVersion {
            recipe_id: r.id.clone(),
            version_number: 1,
            expected_revision: r.revision,
        })
        .unwrap();
    assert!(!no.changed);
    assert_eq!(versions(&db, &r), 3);
    health(&db);
}
#[test]
fn alternatives_duplicate_variation_and_save_as_new_remap_independently() {
    let (_dir, mut db, r, _) = setup();
    db.save_private_note(PrivateNoteWrite {
        recipe_id: r.id.clone(),
        expected_revision: 0,
        markdown: "Özel".into(),
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
    let copy = db.duplicate_recipe(&r.id, r.revision).unwrap();
    let variation = db.create_variation(&r.id, r.revision).unwrap();
    let draft = db
        .save_draft(DraftWrite {
            id: id(),
            expected_revision: None,
            input: r.to_input(),
        })
        .unwrap();
    let as_new = db
        .commit_recipe(r.to_input(), &draft.id, draft.revision, true)
        .unwrap();
    for new in [&copy, &variation, &as_new] {
        assert_ne!(new.id, r.id);
        assert_ne!(new.ingredients[0].id, r.ingredients[0].id);
        assert_ne!(new.steps[0].id, r.steps[0].id);
        assert_ne!(
            new.ingredients[0].alternatives[0].id,
            r.ingredients[0].alternatives[0].id
        );
        assert_eq!(
            new.ingredients[0].alternatives[0].ingredient_id,
            r.ingredients[0].alternatives[0].ingredient_id
        );
        assert_eq!(
            new.steps[0].ingredient_line_ids,
            vec![new.ingredients[0].id.clone()]
        );
        assert_eq!(versions(&db, new), 1);
        assert!(!db.recipe_organization(&new.id).unwrap().favorite);
    }
    assert!(db.conn.execute("INSERT INTO recipe_ingredient_alternatives(id,recipe_id,line_id,ingredient_id,quantity,unit_code,position) VALUES(?1,?2,?3,?4,NULL,NULL,0)",rusqlite::params![id(),copy.id,r.ingredients[0].id,r.ingredients[0].alternatives[0].ingredient_id]).is_err());
    assert_eq!(db.private_note(&copy.id).unwrap().markdown, "Özel");
    assert_eq!(db.private_note(&variation.id).unwrap().markdown, "");
    assert!(db.recipe_links(&copy.id, 0, 20).unwrap().items.is_empty());
    health(&db);
}
#[test]
fn alternative_validation_cas_and_injected_failures_roll_back_every_owned_row() {
    let (dir, mut db, r, alt) = setup();
    let before = db
        .recipe_version(&r.id, 1)
        .unwrap()
        .summary
        .snapshot_checksum;
    let mut other = Database::open(dir.path()).unwrap();
    let mut edit = r.to_input();
    edit.notes = Some("Değişti".into());
    let current = db.save_recipe(edit).unwrap();
    assert_eq!(
        other.save_recipe(r.to_input()).unwrap_err().code,
        "CONFLICT"
    );
    let mut bad = current.to_input();
    bad.ingredients[0].alternatives[0].ingredient_id = id();
    assert_eq!(db.save_recipe(bad).unwrap_err().code, "INVALID_INPUT");
    assert_eq!(versions(&db, &r), 2);
    for mode in 0..5 {
        let mut bad = current.to_input();
        match mode {
            0 => bad.ingredients[0].alternatives[0].quantity = Some("0".into()),
            1 => {
                bad.ingredients[0].alternatives[0].ingredient_id =
                    bad.ingredients[0].ingredient_id.clone()
            }
            2 => {
                let duplicate = bad.ingredients[0].alternatives[0].clone();
                bad.ingredients[0].alternatives.push(duplicate);
            }
            3 => {
                for _ in 0..10 {
                    let mut a = bad.ingredients[0].alternatives[0].clone();
                    a.id = id();
                    bad.ingredients[0].alternatives.push(a);
                }
            }
            _ => bad.ingredients[0].alternatives[0].unit_code = Some("not-a-unit".into()),
        };
        assert!(db.save_recipe(bad).is_err());
    }
    db.conn.execute_batch("CREATE TRIGGER fail_alternative_insert BEFORE INSERT ON recipe_ingredient_alternatives BEGIN SELECT RAISE(ABORT,'injected alt failure');END;").unwrap();
    let mut edit = current.to_input();
    edit.title = "Kaybolmamalı".into();
    assert!(db.save_recipe(edit).is_err());
    assert_eq!(db.recipe(&r.id).unwrap().revision, current.revision);
    assert_eq!(
        db.recipe(&r.id).unwrap().ingredients[0].alternatives[0].ingredient_id,
        alt
    );
    assert_eq!(
        db.recipe_version(&r.id, 1)
            .unwrap()
            .summary
            .snapshot_checksum,
        before
    );
    assert_eq!(versions(&db, &r), 2);
    health(&db);
}
#[test]
fn alternative_line_deletion_lifecycle_and_reviewed_purge_keep_peers_and_drafts() {
    let (_dir, mut db, mut r, alt) = setup();
    let child = db.create_variation(&r.id, r.revision).unwrap();
    let original_alternative = serde_json::to_value(&child.ingredients[0].alternatives).unwrap();
    let child = db.archive_recipe(&child.id, child.revision, true).unwrap();
    assert_eq!(
        serde_json::to_value(&child.ingredients[0].alternatives).unwrap(),
        original_alternative
    );
    let child = db.archive_recipe(&child.id, child.revision, false).unwrap();
    let child = db.set_deleted(&child.id, child.revision, true).unwrap();
    assert_eq!(
        serde_json::to_value(&child.ingredients[0].alternatives).unwrap(),
        original_alternative
    );
    let child = db.set_deleted(&child.id, child.revision, false).unwrap();
    assert_eq!(versions(&db, &child), 1);
    let mut edit = r.to_input();
    edit.ingredients.clear();
    edit.steps[0].ingredient_line_ids.clear();
    r = db.save_recipe(edit).unwrap();
    assert_eq!(
        db.conn
            .query_row(
                "select count(*) from recipe_ingredient_alternatives where recipe_id=?1",
                [&r.id],
                |r| r.get::<_, i64>(0)
            )
            .unwrap(),
        0
    );
    assert!(
        !db.recipe_version(&r.id, 1).unwrap().snapshot.ingredients[0]
            .alternatives
            .is_empty()
    );
    r = db.archive_recipe(&r.id, r.revision, true).unwrap();
    assert!(db.save_recipe(r.to_input()).is_err());
    r = db.archive_recipe(&r.id, r.revision, false).unwrap();
    r = db.set_deleted(&r.id, r.revision, true).unwrap();
    assert!(db.save_recipe(r.to_input()).is_err());
    assert_eq!(
        db.purge_recipe(&r.id, r.revision).unwrap_err().code,
        "RECIPE_LINKS_BLOCK_PURGE"
    );
    let p = db.preview_recipe_purge(&r.id, r.revision, 0, 20).unwrap();
    db.purge_reviewed_recipe(ReviewedPurge {
        recipe_id: r.id.clone(),
        expected_revision: r.revision,
        token: p.token,
        acknowledge_detach: true,
    })
    .unwrap();
    assert!(db.recipe(&r.id).is_err());
    assert_eq!(versions(&db, &child), 1);
    assert_eq!(
        db.recipe(&child.id).unwrap().ingredients[0].alternatives[0].ingredient_id,
        alt
    );
    assert!(db.delete_ingredient(&alt, 1).is_err());
    health(&db);
}
#[test]
fn schema_eleven_upgrade_preserves_format_one_bytes_and_legacy_drafts_then_mixed_restore() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("library.sqlite3");
    std::fs::File::create(&path).unwrap();
    let mut conn = Connection::open(&path).unwrap();
    conn.pragma_update(None, "foreign_keys", true).unwrap();
    migrations::apply(&mut conn, &path, &migrations::MIGRATIONS[..11]).unwrap();
    let mut db = Database { conn };
    db.install_bundled_catalog().unwrap();
    let p = db.create_ingredient("Eski ana").unwrap();
    let alt = db.create_ingredient("Eski alternatif").unwrap();
    let mut legacy = input(&p.id, &alt.id);
    legacy.ingredients[0].alternatives.clear();
    let mut r = db.save_recipe(legacy).unwrap();
    let payload: String = db
        .conn
        .query_row(
            "select snapshot from recipe_versions where recipe_id=?1",
            [&r.id],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(
        db.recipe_version(&r.id, 1).unwrap().summary.format_version,
        1
    );
    let draft = db
        .save_draft(DraftWrite {
            id: id(),
            expected_revision: None,
            input: r.to_input(),
        })
        .unwrap();
    db.save_private_note(PrivateNoteWrite {
        recipe_id: r.id.clone(),
        expected_revision: 0,
        markdown: "Koru".into(),
    })
    .unwrap();
    db.set_recipe_organization(OrganizationWrite {
        recipe_id: r.id.clone(),
        expected_revision: 1,
        favorite: true,
        to_try: false,
        rating: None,
    })
    .unwrap();
    drop(db);
    let mut db = Database::open(dir.path()).unwrap();
    assert_eq!(db.bootstrap().unwrap().storage.schema_version, 12);
    assert_eq!(
        db.draft(&draft.id).unwrap().input.ingredients[0]
            .quantity
            .as_deref(),
        Some("1.250")
    );
    db.discard_draft(&draft.id, draft.revision).unwrap();
    r = db.save_recipe(r.to_input()).unwrap();
    assert_eq!(versions(&db, &r), 1);
    let mut edit = r.to_input();
    edit.ingredients[0]
        .alternatives
        .push(IngredientAlternative {
            id: id(),
            ingredient_id: alt.id,
            quantity: None,
            unit_code: None,
        });
    r = db.save_recipe(edit).unwrap();
    assert_eq!(
        db.recipe_version(&r.id, 2).unwrap().summary.format_version,
        2
    );
    assert_eq!(
        db.recipe_version(&r.id, 1).unwrap().summary.format_version,
        1
    );
    let restored = db
        .restore_recipe_version(RestoreVersion {
            recipe_id: r.id.clone(),
            version_number: 1,
            expected_revision: r.revision,
        })
        .unwrap();
    assert!(restored.changed);
    assert!(restored.recipe.ingredients[0].alternatives.is_empty());
    assert_eq!(restored.version_number, 3);
    assert_eq!(
        db.recipe_version(&r.id, 3).unwrap().summary.provenance,
        "restored"
    );
    assert_eq!(
        db.conn
            .query_row(
                "select snapshot from recipe_versions where recipe_id=?1 and version_number=1",
                [&r.id],
                |r| r.get::<_, String>(0)
            )
            .unwrap(),
        payload
    );
    assert_eq!(db.private_note(&r.id).unwrap().markdown, "Koru");
    assert!(db.recipe_organization(&r.id).unwrap().favorite);
    assert_eq!(
        db.conn
            .query_row(
                "select count(*) from ingredients where origin='catalog'",
                [],
                |r| r.get::<_, i64>(0)
            )
            .unwrap(),
        854
    );
    health(&db);
    drop(db);
    let db = Database::open(dir.path()).unwrap();
    assert_eq!(versions(&db, &r), 3);
    health(&db);
}
#[test]
fn alternative_history_refs_are_immutable_and_schema_twelve_failure_is_atomic() {
    let (_dir, db, r, _) = setup();
    for sql in ["UPDATE recipe_version_alternatives SET ingredient_id='bad'","DELETE FROM recipe_version_alternatives","INSERT OR REPLACE INTO recipe_version_alternatives SELECT * FROM recipe_version_alternatives"]{assert!(db.conn.execute(sql,[]).is_err(),"{sql}");}
    assert_eq!(versions(&db, &r), 1);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("library.sqlite3");
    std::fs::File::create(&path).unwrap();
    let mut conn = Connection::open(&path).unwrap();
    migrations::apply(&mut conn, &path, &migrations::MIGRATIONS[..11]).unwrap();
    let mut bad: Vec<_> = migrations::MIGRATIONS[..11]
        .iter()
        .map(|m| migrations::Migration {
            version: m.version,
            name: m.name,
            sql: m.sql,
        })
        .collect();
    bad.push(migrations::Migration {
        version: 12,
        name: "ingredient_alternatives",
        sql: "CREATE TABLE alt_partial(id);INVALID SQL;",
    });
    assert!(migrations::apply(&mut conn, &path, &bad).is_err());
    assert_eq!(
        conn.query_row("pragma user_version", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        11
    );
    assert!(!conn
        .query_row(
            "select exists(select 1 from sqlite_master where name='alt_partial')",
            [],
            |r| r.get::<_, bool>(0)
        )
        .unwrap());
    drop(conn);
    let db = Database::open(dir.path()).unwrap();
    health(&db);
}
#[test]
fn partial_alternative_measurements_and_failed_snapshot_or_draft_close_are_atomic() {
    let (_dir, mut db, mut r, _) = setup();
    let mut edit = r.to_input();
    edit.ingredients[0].alternatives[0].unit_code = None;
    r = db.save_recipe(edit).unwrap();
    assert_eq!(
        r.ingredients[0].alternatives[0].quantity.as_deref(),
        Some("0.123456")
    );
    assert!(r.ingredients[0].alternatives[0].unit_code.is_none());
    let draft = db
        .save_draft(DraftWrite {
            id: id(),
            expected_revision: None,
            input: r.to_input(),
        })
        .unwrap();
    for trigger in ["CREATE TRIGGER injected_snapshot BEFORE INSERT ON recipe_versions BEGIN SELECT RAISE(ABORT,'snapshot failed');END;","CREATE TRIGGER injected_close BEFORE UPDATE OF state ON editor_drafts BEGIN SELECT RAISE(ABORT,'close failed');END;"]{
 db.conn.execute_batch(trigger).unwrap();let mut edit=r.to_input();edit.ingredients[0].alternatives[0].quantity=Some("1.250".into());assert!(db.commit_recipe(edit,&draft.id,draft.revision,false).is_err());assert_eq!(db.recipe(&r.id).unwrap().revision,r.revision);assert_eq!(db.draft(&draft.id).unwrap().revision,draft.revision);assert_eq!(db.recipe(&r.id).unwrap().ingredients[0].alternatives[0].quantity.as_deref(),Some("0.123456"));assert_eq!(versions(&db,&r),2);db.conn.execute_batch("DROP TRIGGER IF EXISTS injected_snapshot;DROP TRIGGER IF EXISTS injected_close;").unwrap();}
    health(&db);
}
#[test]
fn ten_per_line_and_thousand_per_recipe_bounds_accept_boundary_and_reject_overflow() {
    let (_dir, mut db, mut r, _) = setup();
    let mut edit = r.to_input();
    for n in 1..10 {
        let ingredient = db
            .create_ingredient(&format!("Sınır alternatifi {n}"))
            .unwrap();
        edit.ingredients[0]
            .alternatives
            .push(IngredientAlternative {
                id: id(),
                ingredient_id: ingredient.id,
                quantity: None,
                unit_code: None,
            });
    }
    r = db.save_recipe(edit).unwrap();
    assert_eq!(r.ingredients[0].alternatives.len(), 10);
    let extra = db.create_ingredient("On birinci alternatif").unwrap();
    let mut invalid = r.to_input();
    invalid.ingredients[0]
        .alternatives
        .push(IngredientAlternative {
            id: id(),
            ingredient_id: extra.id,
            quantity: None,
            unit_code: None,
        });
    assert_eq!(db.save_recipe(invalid).unwrap_err().code, "INVALID_INPUT");
    assert_eq!(versions(&db, &r), 2);
    let template = r.ingredients[0].clone();
    let mut bulk = r.to_input();
    bulk.ingredients = (0..100)
        .map(|_| {
            let mut line = template.clone();
            line.id = id();
            for a in &mut line.alternatives {
                a.id = id();
            }
            line
        })
        .collect();
    bulk.steps[0].ingredient_line_ids = vec![bulk.ingredients[0].id.clone()];
    r = db.save_recipe(bulk).unwrap();
    assert_eq!(
        r.ingredients
            .iter()
            .map(|l| l.alternatives.len())
            .sum::<usize>(),
        1000
    );
    let mut invalid = r.to_input();
    let mut line = template;
    line.id = id();
    for a in &mut line.alternatives {
        a.id = id();
    }
    invalid.ingredients.push(line);
    assert_eq!(db.save_recipe(invalid).unwrap_err().code, "INVALID_INPUT");
    assert_eq!(versions(&db, &r), 3);
    health(&db);
}
#[test]
fn unknown_future_content_blocks_read_restore_save_purge_and_definition_deletion() {
    let (_dir, mut db, r, _) = setup();
    let future_ingredient = db
        .create_ingredient("Gelecekteki bilinmeyen başvuru")
        .unwrap();
    let v = db.recipe_version(&r.id, 1).unwrap();
    let mut payload = serde_json::to_value(v.snapshot).unwrap();
    payload["formatVersion"] = 3.into();
    payload["futureIngredient"] = future_ingredient.id.clone().into();
    let raw = serde_json::to_string(&payload).unwrap();
    let hash = migrations::checksum(&raw);
    db.conn.execute("INSERT INTO recipe_versions(recipe_id,version_number,format_version,source_revision,provenance,content_fingerprint,snapshot_checksum,snapshot) VALUES(?1,2,3,1,'edited',?2,?3,?4)",rusqlite::params![r.id,"0".repeat(64),hash,raw]).unwrap();
    assert_eq!(
        db.recipe_version(&r.id, 2).unwrap_err().code,
        "RECIPE_VERSION_UNSUPPORTED"
    );
    assert_eq!(
        db.save_recipe(r.to_input()).unwrap_err().code,
        "RECIPE_VERSION_UNSUPPORTED"
    );
    assert_eq!(
        db.restore_recipe_version(RestoreVersion {
            recipe_id: r.id.clone(),
            version_number: 1,
            expected_revision: r.revision
        })
        .unwrap_err()
        .code,
        "RECIPE_VERSION_UNSUPPORTED"
    );
    assert_eq!(
        db.delete_ingredient(&future_ingredient.id, 1)
            .unwrap_err()
            .code,
        "RECIPE_VERSION_UNSUPPORTED"
    );
    let trash = db.set_deleted(&r.id, r.revision, true).unwrap();
    assert_eq!(
        db.purge_recipe(&r.id, trash.revision).unwrap_err().code,
        "RECIPE_VERSION_UNSUPPORTED"
    );
    assert_eq!(
        db.recipe(&r.id).unwrap().ingredients[0].alternatives[0]
            .quantity
            .as_deref(),
        Some("0.123456")
    );
    health(&db);
}
