use super::*;
use crate::domain::{recipes::*, reliability::DraftWrite};
use rusqlite::params;
use serde_json::json;
fn id() -> String {
    uuid::Uuid::new_v4().to_string()
}
fn input(ingredient: &str) -> RecipeInput {
    serde_json::from_value(json!({"id":id(),"expectedRevision":null,"title":"İçli köfte — Şişli","description":null,"kind":"food","servings":"1.250","prepMinutes":null,"cookMinutes":null,"notes":"Tarihî not",
    "ingredients":[{"id":id(),"ingredientId":ingredient,"quantity":"1.250","unitCode":"g","note":"İnce"}],
    "steps":[{"id":id(),"instructions":"Yağı ısıtın"},{"id":id(),"instructions":"Karıştırın"}]})).unwrap()
}
fn details() -> StepDetails {
    serde_json::from_value(json!({"formatVersion":1,"title":"İlk hazırlık","durationMinutes":"0","temperatureValue":"180","temperatureUnit":"C","heatLevel":"Kısık ateş","equipment":"Döküm tava","technique":"Soteleme","tips":"İ/ı ve Ş/ş korunur"})).unwrap()
}
fn integrity(db: &Database) {
    assert_eq!(
        db.conn
            .query_row("PRAGMA integrity_check", [], |r| r.get::<_, String>(0))
            .unwrap(),
        "ok"
    );
    assert!(db
        .conn
        .prepare("PRAGMA foreign_key_check")
        .unwrap()
        .query([])
        .unwrap()
        .next()
        .unwrap()
        .is_none());
}
#[test]
fn structured_steps_roundtrip_reorder_duplicate_and_lifecycle() {
    let dir = tempfile::tempdir().unwrap();
    let mut db = Database::open(dir.path()).unwrap();
    let ingredient = db.create_ingredient("Şeker").unwrap();
    let mut source = input(&ingredient.id);
    source.steps[0].details = Some(details());
    source.steps[0].ingredient_line_ids = vec![source.ingredients[0].id.clone()];
    source.steps[1].ingredient_line_ids = source.steps[0].ingredient_line_ids.clone();
    let saved = db.save_recipe(source.clone()).unwrap();
    assert_eq!(
        saved.steps[0]
            .details
            .as_ref()
            .unwrap()
            .duration_minutes
            .as_deref(),
        Some("0")
    );
    assert!(saved.steps[1].details.is_none());
    let mut edit = saved.to_input();
    edit.steps.reverse();
    let reordered = db.save_recipe(edit).unwrap();
    assert_eq!(reordered.steps[1].id, source.steps[0].id);
    assert_eq!(
        reordered.steps[1].ingredient_line_ids,
        source.steps[0].ingredient_line_ids
    );
    let copy = db.duplicate_recipe(&saved.id, reordered.revision).unwrap();
    assert_ne!(copy.id, saved.id);
    assert_ne!(copy.ingredients[0].id, source.ingredients[0].id);
    assert_ne!(copy.steps[1].id, source.steps[0].id);
    assert_eq!(
        copy.steps[1].ingredient_line_ids,
        vec![copy.ingredients[0].id.clone()]
    );
    assert_eq!(
        copy.steps[1].details.as_ref().unwrap().tips.as_deref(),
        Some("İ/ı ve Ş/ş korunur")
    );
    assert_eq!(copy.ingredients[0].quantity.as_deref(), Some("1.250"));
    let archived = db
        .archive_recipe(&saved.id, reordered.revision, true)
        .unwrap();
    assert_eq!(
        archived.steps[1].ingredient_line_ids,
        source.steps[0].ingredient_line_ids
    );
    let active = db
        .archive_recipe(&saved.id, archived.revision, false)
        .unwrap();
    let trash = db.set_deleted(&saved.id, active.revision, true).unwrap();
    let restored = db.set_deleted(&saved.id, trash.revision, false).unwrap();
    assert_eq!(restored.steps[1].id, source.steps[0].id);
    let trash = db.set_deleted(&saved.id, restored.revision, true).unwrap();
    db.purge_recipe(&trash.id, trash.revision).unwrap();
    assert_eq!(
        db.conn
            .query_row(
                "SELECT count(*) FROM recipe_step_ingredients WHERE recipe_id=?1",
                [&saved.id],
                |r| r.get::<_, i64>(0)
            )
            .unwrap(),
        0
    );
    integrity(&db);
    drop(db);
    let db = Database::open(dir.path()).unwrap();
    assert_eq!(
        db.recipe(&copy.id).unwrap().steps[1].ingredient_line_ids,
        copy.steps[1].ingredient_line_ids
    );
    integrity(&db);
}
#[test]
fn same_recipe_links_are_validated_in_domain_and_sqlite_and_removed_safely() {
    let dir = tempfile::tempdir().unwrap();
    let mut db = Database::open(dir.path()).unwrap();
    let ingredient = db.create_ingredient("Mantar").unwrap();
    let a = db.save_recipe(input(&ingredient.id)).unwrap();
    let b = db.save_recipe(input(&ingredient.id)).unwrap();
    let mut invalid = a.to_input();
    invalid.steps[0].ingredient_line_ids = vec![b.ingredients[0].id.clone()];
    assert_eq!(db.save_recipe(invalid).unwrap_err().code, "INVALID_INPUT");
    assert!(db
        .conn
        .execute(
            "INSERT INTO recipe_step_ingredients(recipe_id,step_id,line_id) VALUES(?1,?2,?3)",
            params![a.id, a.steps[0].id, b.ingredients[0].id]
        )
        .is_err());
    let mut edit = a.to_input();
    edit.steps[0].ingredient_line_ids = vec![a.ingredients[0].id.clone()];
    let a = db.save_recipe(edit).unwrap();
    // A caller removing a line must explicitly remove its links; invalid payload is rejected.
    let mut invalid = a.to_input();
    invalid.ingredients.clear();
    assert_eq!(db.save_recipe(invalid).unwrap_err().code, "INVALID_INPUT");
    let mut edit = a.to_input();
    edit.ingredients.clear();
    edit.steps[0].ingredient_line_ids.clear();
    let a = db.save_recipe(edit).unwrap();
    assert!(a.steps[0].ingredient_line_ids.is_empty());
    let mut edit = a.to_input();
    edit.steps.clear();
    db.save_recipe(edit).unwrap();
    integrity(&db);
}
#[test]
fn advanced_validation_failed_commit_and_stale_revision_preserve_both_draft_and_recipe() {
    let dir = tempfile::tempdir().unwrap();
    let mut db = Database::open(dir.path()).unwrap();
    let ingredient = db.create_ingredient("Yağ").unwrap();
    let saved = db.save_recipe(input(&ingredient.id)).unwrap();
    let mut edited = saved.to_input();
    edited.steps[0].details = Some(details());
    let draft_id = id();
    let draft = db
        .save_draft(DraftWrite {
            id: draft_id.clone(),
            expected_revision: None,
            input: edited.clone(),
        })
        .unwrap();
    edited.ingredients[0].ingredient_id = id();
    assert!(db
        .commit_recipe(edited, &draft_id, draft.revision, false)
        .is_err());
    assert_eq!(db.recipe(&saved.id).unwrap().revision, 1);
    assert_eq!(db.draft(&draft_id).unwrap().revision, 1);
    let mut edited = saved.to_input();
    edited.steps[0].details = Some(details());
    db.save_recipe(edited.clone()).unwrap();
    assert_eq!(db.save_recipe(edited).unwrap_err().code, "CONFLICT");
    for (duration, temp, unit) in [
        ("-1", "180", Some("C")),
        ("10081", "180", Some("C")),
        ("1.5", "180", Some("C")),
        ("0", "180", None),
        ("0", "1.5", Some("F")),
        ("0", "10000", Some("C")),
    ] {
        let mut bad = db.recipe(&saved.id).unwrap().to_input();
        let mut d = details();
        d.duration_minutes = Some(duration.into());
        d.temperature_value = Some(temp.into());
        d.temperature_unit = unit.map(String::from);
        bad.steps[0].details = Some(d);
        assert_eq!(db.save_recipe(bad).unwrap_err().code, "INVALID_INPUT");
    }
    let mut unfinished = saved.to_input();
    let mut d = details();
    d.duration_minutes = Some("1.".into());
    d.temperature_value = Some("-".into());
    unfinished.steps[0].details = Some(d);
    let draft = db
        .save_draft(DraftWrite {
            id: id(),
            expected_revision: None,
            input: unfinished,
        })
        .unwrap();
    drop(db);
    let db = Database::open(dir.path()).unwrap();
    assert_eq!(
        db.draft(&draft.id).unwrap().input.steps[0]
            .details
            .as_ref()
            .unwrap()
            .temperature_value
            .as_deref(),
        Some("-")
    );
    integrity(&db);
}
#[test]
fn schema_eight_upgrade_preserves_legacy_recipes_drafts_catalog_and_organization() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("library.sqlite3");
    std::fs::File::create(&path).unwrap();
    let mut conn = rusqlite::Connection::open(&path).unwrap();
    conn.pragma_update(None, "foreign_keys", true).unwrap();
    migrations::apply(&mut conn, &path, &migrations::MIGRATIONS[..8]).unwrap();
    let mut db = Database { conn };
    db.install_bundled_catalog().unwrap();
    let ingredient = db.create_ingredient("Özel malzeme").unwrap();
    let mut old = input(&ingredient.id);
    old.expected_revision = Some(1);
    let catalog_id: String = db
        .conn
        .query_row(
            "SELECT id FROM catalog_ingredients ORDER BY id LIMIT 1",
            [],
            |r| r.get(0),
        )
        .unwrap();
    db.conn.execute("UPDATE ingredients SET name='Benim katalog adım',search_name='benim katalog adım' WHERE catalog_id=?1",[&catalog_id]).unwrap();
    db.conn
        .execute(
            "INSERT INTO ingredient_customizations(ingredient_id,field) VALUES(?1,'name')",
            [&catalog_id],
        )
        .unwrap();
    db.conn
        .execute(
            "INSERT INTO catalog_ingredient_favorites(catalog_id) VALUES(?1)",
            [&catalog_id],
        )
        .unwrap();
    db.conn.execute("INSERT INTO recipes(id,title,kind,servings,notes) VALUES(?1,?2,'food','1.250','Eski not')",params![old.id,old.title]).unwrap();
    db.conn.execute("INSERT INTO recipe_ingredients(id,recipe_id,ingredient_id,quantity,unit_code,position) VALUES(?1,?2,?3,'1.250','g',0)",params![old.ingredients[0].id,old.id,ingredient.id]).unwrap();
    db.conn
        .execute(
            "INSERT INTO recipe_steps(id,recipe_id,position,instructions) VALUES(?1,?2,0,?3)",
            params![old.steps[0].id, old.id, old.steps[0].instructions],
        )
        .unwrap();
    db.conn
        .execute(
            "UPDATE recipe_organization SET favorite=1,to_try=1,rating=4 WHERE recipe_id=?1",
            [&old.id],
        )
        .unwrap();
    db.conn
        .execute(
            "INSERT INTO recipe_private_notes(recipe_id,markdown) VALUES(?1,'Özel not')",
            [&old.id],
        )
        .unwrap();
    let draft_id = id();
    let payload = serde_json::to_string(&old).unwrap();
    assert!(!payload.contains("details"));
    db.conn
        .execute(
            "INSERT INTO editor_drafts(id,recipe_id,revision,state,payload) VALUES(?1,?2,1,'active',?3)",
            params![draft_id,old.id,payload],
        )
        .unwrap();
    drop(db);
    let mut db = Database::open(dir.path()).unwrap();
    assert_eq!(db.bootstrap().unwrap().storage.schema_version, 9);
    let saved = db.recipe(&old.id).unwrap();
    assert_eq!(saved.steps[0].id, old.steps[0].id);
    assert!(saved.steps[0].details.is_none());
    assert_eq!(saved.ingredients[0].quantity.as_deref(), Some("1.250"));
    assert_eq!(saved.revision, 1);
    assert_eq!(db.private_note(&old.id).unwrap().markdown, "Özel not");
    assert_eq!(
        db.conn
            .query_row(
                "SELECT name FROM ingredients WHERE id=?1",
                [&catalog_id],
                |r| r.get::<_, String>(0)
            )
            .unwrap(),
        "Benim katalog adım"
    );
    assert_eq!(
        db.conn
            .query_row(
                "SELECT count(*) FROM catalog_ingredient_favorites WHERE catalog_id=?1",
                [&catalog_id],
                |r| r.get::<_, i64>(0)
            )
            .unwrap(),
        1
    );
    assert_eq!(db.recipe_organization(&old.id).unwrap().rating, Some(4));
    assert!(db.draft(&draft_id).unwrap().input.steps[0]
        .details
        .is_none());
    assert_eq!(
        db.conn
            .query_row("SELECT count(*) FROM catalog_ingredients", [], |r| r
                .get::<_, i64>(0))
            .unwrap(),
        854
    );
    db.install_bundled_catalog().unwrap();
    assert_eq!(
        db.conn
            .query_row("SELECT max(version) FROM catalog_releases", [], |r| r
                .get::<_, i64>(0))
            .unwrap(),
        11
    );
    integrity(&db);
    drop(db);
    let db = Database::open(dir.path()).unwrap();
    assert_eq!(db.draft(&draft_id).unwrap().revision, 1);
    integrity(&db);
}
#[test]
fn migration_nine_failure_rolls_back_and_checksums_remain_valid() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("library.sqlite3");
    std::fs::File::create(&path).unwrap();
    let mut conn = rusqlite::Connection::open(&path).unwrap();
    conn.pragma_update(None, "foreign_keys", true).unwrap();
    migrations::apply(&mut conn, &path, &migrations::MIGRATIONS[..8]).unwrap();
    let mut failing: Vec<_> = migrations::MIGRATIONS[..8]
        .iter()
        .map(|m| migrations::Migration {
            version: m.version,
            name: m.name,
            sql: m.sql,
        })
        .collect();
    failing.push(migrations::Migration {
        version: 9,
        name: "structured_recipe_steps",
        sql: concat!(
            include_str!("../../migrations/009_structured_recipe_steps.sql"),
            "INVALID SQL;"
        ),
    });
    assert!(migrations::apply(&mut conn, &path, &failing).is_err());
    assert_eq!(
        migrations::validate(&conn, &migrations::MIGRATIONS[..8]).unwrap(),
        8
    );
    assert_eq!(
        conn.query_row(
            "SELECT count(*) FROM sqlite_master WHERE name='recipe_step_ingredients'",
            [],
            |r| r.get::<_, i64>(0)
        )
        .unwrap(),
        0
    );
    migrations::apply(&mut conn, &path, migrations::MIGRATIONS).unwrap();
    assert_eq!(
        migrations::validate(&conn, migrations::MIGRATIONS).unwrap(),
        9
    );
}

#[test]
fn blank_details_and_invalid_versions_never_erase_committed_steps() {
    let dir = tempfile::tempdir().unwrap();
    let mut db = Database::open(dir.path()).unwrap();
    let ingredient = db.create_ingredient("Çörek otu").unwrap();
    let mut value = input(&ingredient.id);
    let mut d = details();
    d.title = Some(String::new());
    d.duration_minutes = None;
    d.temperature_value = None;
    d.temperature_unit = None;
    d.heat_level = None;
    d.equipment = None;
    d.technique = None;
    d.tips = None;
    value.steps[0].details = Some(d);
    let saved = db.save_recipe(value).unwrap();
    assert_eq!(
        saved.steps[0].details.as_ref().unwrap().duration_minutes,
        None
    );
    assert!(db
        .conn
        .execute(
            "UPDATE recipe_steps SET temperature_value='1.5',temperature_unit='C' WHERE id=?1",
            [&saved.steps[0].id]
        )
        .is_err());
    assert!(db
        .conn
        .execute(
            "UPDATE recipe_steps SET temperature_value='180',temperature_unit=NULL WHERE id=?1",
            [&saved.steps[0].id]
        )
        .is_err());
    let mut bad = saved.to_input();
    bad.steps[0].details.as_mut().unwrap().format_version = 2;
    assert_eq!(
        db.save_recipe(bad.clone()).unwrap_err().code,
        "INVALID_INPUT"
    );
    assert_eq!(
        db.save_draft(DraftWrite {
            id: id(),
            expected_revision: None,
            input: bad
        })
        .unwrap_err()
        .code,
        "INVALID_INPUT"
    );
    let mut bad = saved.to_input();
    bad.steps[0].ingredient_line_ids = vec![saved.ingredients[0].id.clone(); 2];
    assert_eq!(db.save_recipe(bad).unwrap_err().code, "INVALID_INPUT");
    assert_eq!(db.recipe(&saved.id).unwrap().revision, 1);
    integrity(&db);
}
