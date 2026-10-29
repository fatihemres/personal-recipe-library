use super::*;
use crate::domain::{
    recipe_organization::*, recipe_versions::RecipeSnapshot, recipe_workspace::*, recipes::*,
    reliability::*,
};
use rusqlite::{params, Connection, TransactionBehavior};
use serde_json::json;

fn id() -> String {
    uuid::Uuid::new_v4().to_string()
}
fn input(ingredient: &str, kind: &str) -> RecipeInput {
    let line = id();
    serde_json::from_value(json!({"id":id(),"expectedRevision":null,"title":"İçli köfte — Şişli","description":null,"kind":kind,"servings":"1.250","prepMinutes":null,"cookMinutes":0,"notes":"Tarifin kendi notu",
    "ingredients":[{"id":line,"ingredientId":ingredient,"quantity":"1.250","unitCode":"g","note":"İnce"},{"id":id(),"ingredientId":ingredient,"quantity":null,"unitCode":"adet","note":null}],
    "steps":[{"id":id(),"instructions":"Yağı ısıtın","details":{"formatVersion":1,"title":"İlk aşama","durationMinutes":"0","temperatureValue":"180","temperatureUnit":"C","heatLevel":"Kısık ateş","equipment":"Döküm tava","technique":"Soteleme","tips":"İ/ı/Ş/ş"},"ingredientLineIds":[line]},{"id":id(),"instructions":"Servis yapın"}]})).unwrap()
}
fn count(db: &Database, recipe: &str) -> i64 {
    db.conn
        .query_row(
            "SELECT count(*) FROM recipe_versions WHERE recipe_id=?1",
            [recipe],
            |r| r.get(0),
        )
        .unwrap()
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
fn historical_db(dir: &std::path::Path) -> Database {
    let path = dir.join("library.sqlite3");
    std::fs::File::create(&path).unwrap();
    let mut conn = Connection::open(&path).unwrap();
    conn.pragma_update(None, "foreign_keys", true).unwrap();
    migrations::apply(&mut conn, &path, &migrations::MIGRATIONS[..9]).unwrap();
    Database { conn }
}
fn insert_old(db: &mut Database, value: &RecipeInput) {
    // Use historical schema columns, not the current writer (which requires history).
    super::tests::insert_legacy_recipe(&db.conn, value);
    db.conn
        .execute(
            "UPDATE recipes SET notes=?2 WHERE id=?1",
            params![value.id, value.notes],
        )
        .unwrap();
}

#[test]
fn initial_versions_meaningful_edits_noops_and_bounded_queries() {
    let dir = tempfile::tempdir().unwrap();
    let mut db = Database::open(dir.path()).unwrap();
    assert_eq!(db.bootstrap().unwrap().storage.schema_version, 10);
    let ingredient = db.create_ingredient("Şeker").unwrap();
    for kind in ["food", "beverage"] {
        let mut saved = db.save_recipe(input(&ingredient.id, kind)).unwrap();
        let v1 = super::recipe_versions::read_version(&db.conn, &saved.id, 1).unwrap();
        assert_eq!(v1.summary.provenance, "created");
        assert_eq!(v1.summary.source_revision, 1);
        assert_eq!(
            v1.snapshot.ingredients[0].quantity.as_deref(),
            Some("1.250")
        );
        assert_eq!(v1.snapshot.ingredients[1].quantity, None);
        assert_eq!(
            v1.snapshot.steps[0].ingredient_line_ids,
            vec![saved.ingredients[0].id.clone()]
        );
        assert_eq!(
            v1.snapshot.steps[0]
                .details
                .as_ref()
                .unwrap()
                .tips
                .as_deref(),
            Some("İ/ı/Ş/ş")
        );
        // Explicit no-op saves retain preexisting optimistic revision behavior.
        saved = db.save_recipe(saved.to_input()).unwrap();
        assert_eq!(saved.revision, 2);
        assert_eq!(count(&db, &saved.id), 1);
        let mut edit = saved.to_input();
        edit.title.push_str(" yeni");
        saved = db.save_recipe(edit).unwrap();
        let mut edit = saved.to_input();
        edit.ingredients[0].quantity = Some("1.25".into());
        saved = db.save_recipe(edit).unwrap();
        assert_eq!(count(&db, &saved.id), 3);
        assert_eq!(
            super::recipe_versions::read_version(&db.conn, &saved.id, 2)
                .unwrap()
                .summary
                .provenance,
            "edited"
        );
        assert_eq!(
            super::recipe_versions::read_version(&db.conn, &saved.id, 3)
                .unwrap()
                .snapshot
                .ingredients[0]
                .quantity
                .as_deref(),
            Some("1.25")
        );
        let first = super::recipe_versions::list_versions(&db.conn, &saved.id, None, 2).unwrap();
        assert_eq!(
            first.iter().map(|v| v.version_number).collect::<Vec<_>>(),
            vec![3, 2]
        );
        assert_eq!(
            super::recipe_versions::list_versions(&db.conn, &saved.id, Some(2), 2).unwrap()[0]
                .version_number,
            1
        );
        for limit in [0, 101, u32::MAX] {
            assert!(
                super::recipe_versions::list_versions(&db.conn, &saved.id, None, limit).is_err()
            );
        }
        assert!(super::recipe_versions::list_versions(&db.conn, &saved.id, Some(0), 1).is_err());
        assert!(super::recipe_versions::list_versions(&db.conn, "' OR 1=1 --", None, 1).is_err());
        assert_eq!(
            super::recipe_versions::read_version(&db.conn, &saved.id, 99)
                .unwrap_err()
                .code,
            "NOT_FOUND"
        );
    }
    integrity(&db);
}

#[test]
fn canonical_comparison_uses_persisted_values_and_relationship_sets() {
    let dir = tempfile::tempdir().unwrap();
    let mut db = Database::open(dir.path()).unwrap();
    let ingredient = db.create_ingredient("Süt").unwrap();
    let mut value = input(&ingredient.id, "food");
    value.steps[0].ingredient_line_ids = value.ingredients.iter().map(|l| l.id.clone()).collect();
    let saved = db.save_recipe(value).unwrap();
    let mut noop = saved.to_input();
    noop.title = format!(" {} ", noop.title);
    noop.steps[0].instructions = format!(" {} ", noop.steps[0].instructions);
    noop.steps[0].details.as_mut().unwrap().duration_minutes = Some("000".into());
    noop.steps[0].ingredient_line_ids.reverse();
    let saved = db.save_recipe(noop).unwrap();
    assert_eq!(count(&db, &saved.id), 1);
    let mut edit = saved.to_input();
    edit.steps.reverse();
    edit.ingredients.reverse();
    let saved = db.save_recipe(edit).unwrap();
    assert_eq!(count(&db, &saved.id), 2);
    let v = super::recipe_versions::read_version(&db.conn, &saved.id, 2).unwrap();
    assert_eq!(v.snapshot.steps[1].id, saved.steps[1].id);
    assert_eq!(v.snapshot.ingredients[0].id, saved.ingredients[0].id);
    // NULL and empty content notes are distinct; so are explicit zero and missing duration.
    let mut edit = saved.to_input();
    edit.notes = None;
    edit.steps[1].details.as_mut().unwrap().duration_minutes = None;
    let saved = db.save_recipe(edit).unwrap();
    assert_eq!(count(&db, &saved.id), 3);
    let mut edit = saved.to_input();
    edit.notes = Some(String::new());
    let saved = db.save_recipe(edit).unwrap();
    assert_eq!(count(&db, &saved.id), 4);
    integrity(&db);
}

#[test]
fn historical_names_ids_and_definition_protection_survive_line_removal_and_purge() {
    let dir = tempfile::tempdir().unwrap();
    let mut db = Database::open(dir.path()).unwrap();
    let ingredient = db.create_ingredient("Eski kişisel ad").unwrap();
    db.conn.execute("INSERT INTO units(code,dimension,canonical_code,factor) VALUES('history-unit','count','adet',1)",[]).unwrap();
    let mut source = input(&ingredient.id, "food");
    source.ingredients[0].unit_code = "history-unit".into();
    let saved = db.save_recipe(source).unwrap();
    let before = serde_json::to_string(
        &super::recipe_versions::read_version(&db.conn, &saved.id, 1).unwrap(),
    )
    .unwrap();
    db.edit_ingredient(IngredientEdit {
        id: ingredient.id.clone(),
        revision: 1,
        name: "Yeni kişisel ad".into(),
        notes: None,
        preferred_unit: None,
        category_ids: None,
    })
    .unwrap();
    let saved = db.save_recipe(saved.to_input()).unwrap();
    assert_eq!(count(&db, &saved.id), 1);
    assert_eq!(
        serde_json::to_string(
            &super::recipe_versions::read_version(&db.conn, &saved.id, 1).unwrap()
        )
        .unwrap(),
        before
    );
    let mut edit = saved.to_input();
    edit.title.push('!');
    let saved = db.save_recipe(edit).unwrap();
    assert_eq!(
        super::recipe_versions::read_version(&db.conn, &saved.id, 2)
            .unwrap()
            .snapshot
            .ingredient_names[&ingredient.id],
        "Yeni kişisel ad"
    );
    let mut edit = saved.to_input();
    edit.ingredients.clear();
    for step in &mut edit.steps {
        step.ingredient_line_ids.clear();
    }
    let saved = db.save_recipe(edit).unwrap();
    assert_eq!(
        db.delete_ingredient(&ingredient.id, 2).unwrap_err().code,
        "INGREDIENT_REFERENCED"
    );
    assert!(db
        .conn
        .execute("DELETE FROM ingredients WHERE id=?1", [&ingredient.id])
        .is_err());
    assert!(db
        .conn
        .execute("DELETE FROM units WHERE code='history-unit'", [])
        .is_err());
    assert_eq!(
        serde_json::to_string(
            &super::recipe_versions::read_version(&db.conn, &saved.id, 1).unwrap()
        )
        .unwrap(),
        before
    );
    let other = db.save_recipe(input(&ingredient.id, "beverage")).unwrap();
    let trash = db.set_deleted(&saved.id, saved.revision, true).unwrap();
    db.purge_recipe(&trash.id, trash.revision).unwrap();
    assert_eq!(count(&db, &saved.id), 0);
    assert_eq!(count(&db, &other.id), 1);
    db.conn
        .execute("DELETE FROM units WHERE code='history-unit'", [])
        .unwrap();
    assert!(db.delete_ingredient(&ingredient.id, 2).is_err());
    let trash = db.set_deleted(&other.id, other.revision, true).unwrap();
    db.purge_recipe(&trash.id, trash.revision).unwrap();
    db.delete_ingredient(&ingredient.id, 2).unwrap();
    integrity(&db);
}

#[test]
fn snapshots_and_reference_rows_are_immutable_but_owned_history_cascades_on_deliberate_purge() {
    let dir = tempfile::tempdir().unwrap();
    let mut db = Database::open(dir.path()).unwrap();
    let ingredient = db.create_ingredient("Un").unwrap();
    let saved = db.save_recipe(input(&ingredient.id, "food")).unwrap();
    for sql in ["UPDATE recipe_versions SET provenance='upgrade_baseline'", "DELETE FROM recipe_versions", "UPDATE recipe_version_ingredients SET unit_code='kg'", "DELETE FROM recipe_version_ingredients", "INSERT OR REPLACE INTO recipe_versions SELECT * FROM recipe_versions", "INSERT OR REPLACE INTO ingredients SELECT * FROM ingredients WHERE id IN (SELECT ingredient_id FROM recipe_version_ingredients)"] {
        assert!(db.conn.execute(sql,[]).is_err(),"{sql}");
    }
    let mut v = super::recipe_versions::read_version(&db.conn, &saved.id, 1)
        .unwrap()
        .snapshot;
    v.format_version = 2;
    assert_eq!(v.validate().unwrap_err().code, "RECIPE_VERSION_UNSUPPORTED");
    v.format_version = 1;
    v.steps[0].ingredient_line_ids = vec![id()];
    assert!(v.validate().is_err());
    assert!(serde_json::from_value::<RecipeSnapshot>(
        json!({"formatVersion":1,"sql":"DROP TABLE recipes"})
    )
    .is_err());
    let trash = db.set_deleted(&saved.id, saved.revision, true).unwrap();
    assert!(db.purge_recipe(&trash.id, trash.revision - 1).is_err());
    assert_eq!(count(&db, &saved.id), 1);
    db.purge_recipe(&trash.id, trash.revision).unwrap();
    assert_eq!(count(&db, &saved.id), 0);
    integrity(&db);
}

#[test]
fn drafts_duplicates_save_as_new_and_organization_keep_independent_version_boundaries() {
    let dir = tempfile::tempdir().unwrap();
    let mut db = Database::open(dir.path()).unwrap();
    let ingredient = db.create_ingredient("Su").unwrap();
    let value = input(&ingredient.id, "beverage");
    let draft = db
        .save_draft(DraftWrite {
            id: id(),
            expected_revision: None,
            input: value.clone(),
        })
        .unwrap();
    assert_eq!(count(&db, &value.id), 0);
    let saved = db
        .commit_recipe(value, &draft.id, draft.revision, false)
        .unwrap();
    assert_eq!(count(&db, &saved.id), 1);
    assert!(db.draft(&draft.id).is_err());
    let mut pending = saved.to_input();
    pending.title.push_str(" taslak");
    let draft = db
        .save_draft(DraftWrite {
            id: id(),
            expected_revision: None,
            input: pending.clone(),
        })
        .unwrap();
    let draft = db
        .save_draft(DraftWrite {
            id: draft.id.clone(),
            expected_revision: Some(draft.revision),
            input: pending.clone(),
        })
        .unwrap();
    assert_eq!(count(&db, &saved.id), 1);
    db.discard_draft(&draft.id, draft.revision).unwrap();
    assert_eq!(count(&db, &saved.id), 1);
    let org = db.recipe_organization(&saved.id).unwrap();
    db.set_recipe_organization(OrganizationWrite {
        recipe_id: saved.id.clone(),
        expected_revision: org.revision,
        favorite: true,
        to_try: true,
        rating: Some(5),
    })
    .unwrap();
    let collection = db
        .save_collection(CollectionWrite {
            id: id(),
            expected_revision: None,
            title: "İçecekler".into(),
        })
        .unwrap();
    let org = db.recipe_organization(&saved.id).unwrap();
    db.set_collection_membership(&saved.id, &collection.id, org.revision, true)
        .unwrap();
    db.save_private_note(PrivateNoteWrite {
        recipe_id: saved.id.clone(),
        expected_revision: 0,
        markdown: "# Ayrı özel not".into(),
    })
    .unwrap();
    assert_eq!(count(&db, &saved.id), 1);
    let payload: String = db
        .conn
        .query_row(
            "SELECT snapshot FROM recipe_versions WHERE recipe_id=?1 AND version_number=1",
            [&saved.id],
            |r| r.get(0),
        )
        .unwrap();
    let value: serde_json::Value = serde_json::from_str(&payload).unwrap();
    assert_eq!(value["notes"], "Tarifin kendi notu");
    assert!(!payload.contains("# Ayrı özel not"));
    for key in [
        "favorite",
        "rating",
        "collectionIds",
        "privateNote",
        "revision",
        "expectedRevision",
        "deletedAt",
        "archivedAt",
    ] {
        assert!(value.get(key).is_none(), "{key}");
    }
    let archived = db.archive_recipe(&saved.id, saved.revision, true).unwrap();
    let active = db
        .archive_recipe(&saved.id, archived.revision, false)
        .unwrap();
    let trash = db.set_deleted(&saved.id, active.revision, true).unwrap();
    let active = db.set_deleted(&saved.id, trash.revision, false).unwrap();
    assert_eq!(count(&db, &saved.id), 1);
    let copy = db.duplicate_recipe(&active.id, active.revision).unwrap();
    assert_eq!(count(&db, &copy.id), 1);
    assert_eq!(
        super::recipe_versions::read_version(&db.conn, &copy.id, 1)
            .unwrap()
            .summary
            .provenance,
        "created"
    );
    assert_ne!(copy.steps[0].id, saved.steps[0].id);
    assert_eq!(
        copy.steps[0].ingredient_line_ids,
        vec![copy.ingredients[0].id.clone()]
    );
    let pending = db
        .save_draft(DraftWrite {
            id: id(),
            expected_revision: None,
            input: active.to_input(),
        })
        .unwrap();
    let as_new = db
        .commit_recipe(pending.input, &pending.id, pending.revision, true)
        .unwrap();
    assert_ne!(as_new.id, active.id);
    assert_eq!(count(&db, &as_new.id), 1);
    assert_eq!(count(&db, &active.id), 1);
    assert_eq!(
        as_new.steps[0].ingredient_line_ids,
        vec![as_new.ingredients[0].id.clone()]
    );
    integrity(&db);
}

#[test]
fn failed_snapshot_draft_close_and_stale_saves_roll_back_every_part_of_the_transaction() {
    let dir = tempfile::tempdir().unwrap();
    let mut db = Database::open(dir.path()).unwrap();
    let ingredient = db.create_ingredient("Sarımsak").unwrap();
    let saved = db.save_recipe(input(&ingredient.id, "food")).unwrap();
    let mut edit = saved.to_input();
    edit.title.push('!');
    let draft = db
        .save_draft(DraftWrite {
            id: id(),
            expected_revision: None,
            input: edit.clone(),
        })
        .unwrap();
    for (trigger,sql) in [("fail_snapshot","CREATE TRIGGER fail_snapshot BEFORE INSERT ON recipe_versions BEGIN SELECT RAISE(ABORT,'injected'); END;"),("fail_close","CREATE TRIGGER fail_close BEFORE UPDATE ON editor_drafts WHEN NEW.state='closed' BEGIN SELECT RAISE(ABORT,'injected'); END;")] {
        db.conn.execute_batch(sql).unwrap();
        assert!(db.commit_recipe(edit.clone(),&draft.id,draft.revision,false).is_err());
        assert_eq!(db.recipe(&saved.id).unwrap().revision,1);
        assert_eq!(db.recipe(&saved.id).unwrap().title,saved.title);
        assert_eq!(db.draft(&draft.id).unwrap().revision,draft.revision);
        assert_eq!(count(&db,&saved.id),1);
        db.conn.execute_batch(&format!("DROP TRIGGER {trigger}")).unwrap();
    }
    let changed = db
        .commit_recipe(edit.clone(), &draft.id, draft.revision, false)
        .unwrap();
    assert_eq!(count(&db, &saved.id), 2);
    assert_eq!(db.save_recipe(edit).unwrap_err().code, "CONFLICT");
    assert_eq!(count(&db, &saved.id), 2);
    let mut bad = changed.to_input();
    bad.ingredients[0].ingredient_id = id();
    assert!(db.save_recipe(bad).is_err());
    assert_eq!(db.recipe(&saved.id).unwrap().revision, changed.revision);
    assert!(db
        .commit_recipe(changed.to_input(), &draft.id, draft.revision, false)
        .is_err());
    let mut malformed = changed.to_input();
    malformed.steps[0].details.as_mut().unwrap().format_version = 2;
    assert!(db.save_recipe(malformed).is_err());
    assert_eq!(count(&db, &saved.id), 2);
    // Initial creation and independent duplicate must roll back as well.
    db.conn.execute_batch("CREATE TRIGGER fail_snapshot BEFORE INSERT ON recipe_versions BEGIN SELECT RAISE(ABORT,'injected'); END;").unwrap();
    let value = input(&ingredient.id, "food");
    let new_id = value.id.clone();
    assert!(db.save_recipe(value).is_err());
    assert!(db.recipe(&new_id).is_err());
    assert!(db.duplicate_recipe(&changed.id, changed.revision).is_err());
    assert_eq!(
        db.conn
            .query_row("SELECT count(*) FROM recipes", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        1
    );
    integrity(&db);
}

#[test]
fn schema_nine_upgrade_creates_honest_complete_baselines_and_preserves_user_data_and_backup() {
    let dir = tempfile::tempdir().unwrap();
    let mut old = historical_db(dir.path());
    old.install_bundled_catalog().unwrap();
    let ingredient = old.create_ingredient("Özel malzeme").unwrap();
    let mut ids = vec![];
    for status in ["active", "archived", "trash"] {
        let value = input(&ingredient.id, "food");
        insert_old(&mut old, &value);
        ids.push(value.id.clone());
        old.conn.execute("UPDATE recipes SET revision=12, archived_at=CASE WHEN ?2='archived' THEN '2026-01-01' END,deleted_at=CASE WHEN ?2='trash' THEN '2026-01-01' END WHERE id=?1",params![value.id,status]).unwrap();
        old.conn
            .execute(
                "UPDATE recipe_organization SET favorite=1,to_try=1,rating=4 WHERE recipe_id=?1",
                [&value.id],
            )
            .unwrap();
        old.conn
            .execute(
                "INSERT INTO recipe_private_notes(recipe_id,markdown) VALUES(?1,'# Koru')",
                [&value.id],
            )
            .unwrap();
    }
    // M5A fields and their line relationship already exist in schema 9.
    let (step,line):(String,String)=old.conn.query_row("SELECT s.id,i.id FROM recipe_steps s JOIN recipe_ingredients i ON i.recipe_id=s.recipe_id WHERE s.recipe_id=?1 ORDER BY s.position,i.position LIMIT 1",[&ids[0]],|r|Ok((r.get(0)?,r.get(1)?))).unwrap();
    old.conn.execute("UPDATE recipe_steps SET details_version=1,duration_minutes=0,temperature_value='180',temperature_unit='C',tips='İ/ı koru' WHERE id=?1",[&step]).unwrap();
    old.conn
        .execute(
            "INSERT INTO recipe_step_ingredients(recipe_id,step_id,line_id) VALUES(?1,?2,?3)",
            params![ids[0], step, line],
        )
        .unwrap();
    let saved = old.recipe(&ids[0]).unwrap();
    let mut pending = saved.to_input();
    pending.title = "Kaydedilmemiş".into();
    let draft = old
        .save_draft(DraftWrite {
            id: id(),
            expected_revision: None,
            input: pending,
        })
        .unwrap();
    let collection = old
        .save_collection(CollectionWrite {
            id: id(),
            expected_revision: None,
            title: "Eski koleksiyon".into(),
        })
        .unwrap();
    let org = old.recipe_organization(&ids[0]).unwrap();
    old.set_collection_membership(&ids[0], &collection.id, org.revision, true)
        .unwrap();
    let filter=old.save_filter(serde_json::from_value(json!({"id":id(),"expectedRevision":null,"title":"İçecek filtresi","query":{"text":"İçli","scope":"active","kind":"food","ingredientId":ingredient.id,"favoriteOnly":true,"toTryOnly":true,"collectionId":collection.id,"createdFrom":null,"createdTo":null,"modifiedFrom":null,"modifiedTo":null,"sort":"modified","offset":0,"limit":24}})).unwrap()).unwrap();
    old.conn
        .execute("UPDATE preferences SET theme='dark'", [])
        .unwrap();
    let canonical: String = old
        .conn
        .query_row(
            "SELECT id FROM catalog_ingredients ORDER BY id LIMIT 1",
            [],
            |r| r.get(0),
        )
        .unwrap();
    old.conn
        .execute(
            "INSERT INTO ingredient_customizations(ingredient_id,field) VALUES(?1,'name')",
            [&canonical],
        )
        .unwrap();
    old.conn.execute("UPDATE ingredients SET name='Benim katalog etiketim',search_name='benim katalog etiketim' WHERE catalog_id=?1",[&canonical]).unwrap();
    old.conn
        .execute(
            "INSERT INTO catalog_ingredient_favorites(catalog_id) VALUES(?1)",
            [&canonical],
        )
        .unwrap();
    drop(old);
    for _ in 0..3 {
        let mut db = Database::open(dir.path()).unwrap();
        assert_eq!(db.bootstrap().unwrap().storage.schema_version, 10);
        for id in &ids {
            let v = super::recipe_versions::read_version(&db.conn, id, 1).unwrap();
            assert_eq!(v.summary.provenance, "upgrade_baseline");
            assert_eq!(v.summary.source_revision, 12);
            assert_eq!(count(&db, id), 1);
            assert_eq!(db.recipe(id).unwrap().revision, 12);
            assert_eq!(v.snapshot.ingredients[0].quantity.as_deref(), Some("1.250"));
            assert!(db.recipe_organization(id).unwrap().favorite);
            assert_eq!(db.private_note(id).unwrap().markdown, "# Koru");
        }
        assert_eq!(
            super::recipe_versions::read_version(&db.conn, &ids[0], 1)
                .unwrap()
                .snapshot
                .steps[0]
                .ingredient_line_ids,
            vec![line.clone()]
        );
        assert_eq!(db.draft(&draft.id).unwrap().input.title, "Kaydedilmemiş");
        assert_eq!(db.draft(&draft.id).unwrap().revision, draft.revision);
        assert_eq!(
            db.recipe_organization(&ids[0]).unwrap().collection_ids,
            vec![collection.id.clone()]
        );
        assert_eq!(db.saved_filters().unwrap()[0].id, filter.id);
        assert_eq!(
            db.saved_filters().unwrap()[0]
                .query
                .as_ref()
                .unwrap()
                .collection_id
                .as_deref(),
            Some(collection.id.as_str())
        );
        assert_eq!(db.preferences().unwrap().theme, crate::domain::Theme::Dark);
        assert_eq!(
            db.conn
                .query_row(
                    "SELECT name FROM ingredients WHERE catalog_id=?1",
                    [&canonical],
                    |r| r.get::<_, String>(0)
                )
                .unwrap(),
            "Benim katalog etiketim"
        );
        assert_eq!(
            db.conn
                .query_row("SELECT count(*) FROM catalog_ingredients", [], |r| r
                    .get::<_, i64>(0))
                .unwrap(),
            854
        );
        assert_eq!(
            db.conn
                .query_row("SELECT count(*) FROM catalog_aliases", [], |r| r
                    .get::<_, i64>(0))
                .unwrap(),
            269
        );
        assert_eq!(
            db.conn
                .query_row("SELECT count(*) FROM ingredient_categories", [], |r| r
                    .get::<_, i64>(0))
                .unwrap(),
            50
        );
        assert_eq!(
            db.conn
                .query_row(
                    "SELECT count(*) FROM ingredient_customizations WHERE ingredient_id=?1",
                    [&canonical],
                    |r| r.get::<_, i64>(0)
                )
                .unwrap(),
            1
        );
        assert_eq!(
            db.conn
                .query_row(
                    "SELECT count(*) FROM catalog_ingredient_favorites",
                    [],
                    |r| r.get::<_, i64>(0)
                )
                .unwrap(),
            1
        );
        db.install_bundled_catalog().unwrap();
        integrity(&db);
    }
    let backup = std::fs::read_dir(dir.path())
        .unwrap()
        .map(|p| p.unwrap().path())
        .find(|p| {
            p.file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("pre-migration-9-")
        })
        .unwrap();
    let reader =
        Connection::open_with_flags(backup, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY).unwrap();
    assert_eq!(
        migrations::validate(&reader, &migrations::MIGRATIONS[..9]).unwrap(),
        9
    );
    assert_eq!(
        reader
            .query_row("SELECT count(*) FROM recipes", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        3
    );
}

#[test]
fn failed_backfill_leaves_schema_nine_intact_and_can_retry_without_partial_history() {
    let dir = tempfile::tempdir().unwrap();
    let mut old = historical_db(dir.path());
    let ingredient = old.create_ingredient("Un").unwrap();
    let mut value = input(&ingredient.id, "food");
    value.id = "00000000-0000-4000-8000-000000000001".into();
    insert_old(&mut old, &value);
    let mut bad = input(&ingredient.id, "food");
    bad.id = "ffffffff-ffff-4fff-8fff-ffffffffffff".into();
    insert_old(&mut old, &bad);
    // A malformed legacy ID cannot silently become an authoritative snapshot.
    old.conn
        .execute(
            "UPDATE recipe_steps SET id='xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx' WHERE recipe_id=?1 AND position=0",
            [&bad.id],
        )
        .unwrap();
    drop(old);
    assert!(Database::open(dir.path()).is_err());
    let conn = Connection::open(dir.path().join("library.sqlite3")).unwrap();
    assert_eq!(
        migrations::validate(&conn, &migrations::MIGRATIONS[..9]).unwrap(),
        9
    );
    assert_eq!(
        conn.query_row(
            "SELECT count(*) FROM sqlite_master WHERE name='recipe_versions'",
            [],
            |r| r.get::<_, i64>(0)
        )
        .unwrap(),
        0
    );
    // Test fixture repair is explicit, not an application migration policy.
    conn.execute(
        "UPDATE recipe_steps SET id=?2 WHERE recipe_id=?1 AND id='xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx'",
        params![bad.id, id()],
    )
    .unwrap();
    drop(conn);
    let db = Database::open(dir.path()).unwrap();
    assert_eq!(count(&db, &value.id), 1);
    assert_eq!(count(&db, &bad.id), 1);
    integrity(&db);
}

#[test]
fn fingerprint_v1_is_pinned_and_ignores_only_capture_metadata_and_empty_detail_envelopes() {
    let mut snapshot: RecipeSnapshot =
        serde_json::from_str(include_str!("fixtures/recipe-version-v1.json")).unwrap();
    snapshot.validate().unwrap();
    let fingerprint = super::recipe_versions::fingerprint(&snapshot).unwrap();
    assert_eq!(
        fingerprint,
        "e33cd7e9351ac26175928234b490a4ba37119594a1944ace30070e6cff7d073b"
    );
    snapshot.recipe_created_at = "2026-01-01".into();
    for name in snapshot.ingredient_names.values_mut() {
        *name = "Yeni etiket".into();
    }
    snapshot.steps[0].ingredient_line_ids.reverse();
    assert_eq!(
        super::recipe_versions::fingerprint(&snapshot).unwrap(),
        fingerprint
    );
    snapshot.ingredients[0].quantity = Some("1.25".into());
    assert_ne!(
        super::recipe_versions::fingerprint(&snapshot).unwrap(),
        fingerprint
    );
    snapshot.steps[0].details = None;
    let plain = super::recipe_versions::fingerprint(&snapshot).unwrap();
    snapshot.steps[0].details=Some(serde_json::from_value(json!({"formatVersion":1,"title":null,"durationMinutes":null,"temperatureValue":null,"temperatureUnit":null,"heatLevel":null,"equipment":null,"technique":null,"tips":null})).unwrap());
    assert_eq!(
        super::recipe_versions::fingerprint(&snapshot).unwrap(),
        plain
    );
}

#[test]
fn future_snapshot_formats_are_read_only_and_block_unsafe_old_application_save_or_purge() {
    let dir = tempfile::tempdir().unwrap();
    let mut db = Database::open(dir.path()).unwrap();
    let ingredient = db.create_ingredient("Kakao").unwrap();
    let saved = db.save_recipe(input(&ingredient.id, "food")).unwrap();
    let snapshot = super::recipe_versions::read_version(&db.conn, &saved.id, 1)
        .unwrap()
        .snapshot;
    let mut future = serde_json::to_value(snapshot).unwrap();
    future["formatVersion"] = json!(2);
    future["assetIds"] = json!([id()]);
    let payload = serde_json::to_string(&future).unwrap();
    // Simulate a later writer. This build has no format-2/media authoring path.
    db.conn.execute("INSERT INTO recipe_versions(recipe_id,version_number,format_version,source_revision,provenance,content_fingerprint,snapshot_checksum,snapshot) VALUES(?1,2,2,2,'edited',?2,?3,?4)",params![saved.id,"0".repeat(64),migrations::checksum(&payload),payload]).unwrap();
    db.conn
        .execute("UPDATE recipes SET revision=2 WHERE id=?1", [&saved.id])
        .unwrap();
    let error = super::recipe_versions::read_version(&db.conn, &saved.id, 2).unwrap_err();
    assert_eq!(error.code, "RECIPE_VERSION_UNSUPPORTED");
    assert!(error.recoverable);
    let mut edit = db.recipe(&saved.id).unwrap().to_input();
    edit.title.push('!');
    assert_eq!(
        db.save_recipe(edit).unwrap_err().code,
        "RECIPE_VERSION_UNSUPPORTED"
    );
    assert_eq!(db.recipe(&saved.id).unwrap().revision, 2);
    assert_eq!(count(&db, &saved.id), 2);
    let trash = db.set_deleted(&saved.id, 2, true).unwrap();
    assert_eq!(
        db.purge_recipe(&trash.id, trash.revision).unwrap_err().code,
        "RECIPE_VERSION_UNSUPPORTED"
    );
    assert_eq!(count(&db, &saved.id), 2);
    assert!(db.recipe(&saved.id).is_ok());
    integrity(&db);
}

#[test]
fn reserved_restored_from_fk_is_same_recipe_and_owned_purge_does_not_break_self_references() {
    let dir = tempfile::tempdir().unwrap();
    let mut db = Database::open(dir.path()).unwrap();
    let ingredient = db.create_ingredient("Su").unwrap();
    let saved = db.save_recipe(input(&ingredient.id, "beverage")).unwrap();
    // DDL compatibility only: no restore command or user-facing workflow exists.
    db.conn.execute("INSERT INTO recipe_versions(recipe_id,version_number,format_version,source_revision,provenance,restored_from_version,content_fingerprint,snapshot_checksum,snapshot) SELECT recipe_id,2,1,2,'restored',1,content_fingerprint,snapshot_checksum,snapshot FROM recipe_versions WHERE recipe_id=?1 AND version_number=1",[&saved.id]).unwrap();
    assert_eq!(count(&db, &saved.id), 2);
    assert!(db.conn.execute("INSERT INTO recipe_versions(recipe_id,version_number,format_version,source_revision,provenance,restored_from_version,content_fingerprint,snapshot_checksum,snapshot) SELECT recipe_id,3,1,3,'restored',99,content_fingerprint,snapshot_checksum,snapshot FROM recipe_versions WHERE recipe_id=?1 AND version_number=1",[&saved.id]).is_err());
    assert!(db
        .conn
        .execute(
            "DELETE FROM recipe_versions WHERE recipe_id=?1 AND version_number=1",
            [&saved.id]
        )
        .is_err());
    let trash = db.set_deleted(&saved.id, saved.revision, true).unwrap();
    db.purge_recipe(&trash.id, trash.revision).unwrap();
    assert_eq!(count(&db, &saved.id), 0);
    integrity(&db);
}

#[test]
fn concurrent_expected_revisions_append_exactly_one_version_and_never_lose_the_winner() {
    let dir = tempfile::tempdir().unwrap();
    let mut db = Database::open(dir.path()).unwrap();
    let ingredient = db.create_ingredient("Ihlamur").unwrap();
    let saved = db.save_recipe(input(&ingredient.id, "beverage")).unwrap();
    let barrier = std::sync::Arc::new(std::sync::Barrier::new(2));
    let jobs: Vec<_> = (0..2)
        .map(|n| {
            let root = dir.path().to_path_buf();
            let barrier = barrier.clone();
            let mut edit = saved.to_input();
            edit.title = format!("Eşzamanlı {n}");
            std::thread::spawn(move || {
                let mut db = Database::open(&root).unwrap();
                barrier.wait();
                db.save_recipe(edit)
            })
        })
        .collect();
    let results: Vec<_> = jobs.into_iter().map(|h| h.join().unwrap()).collect();
    assert_eq!(results.iter().filter(|r| r.is_ok()).count(), 1);
    assert_eq!(
        results.iter().find_map(|r| r.as_ref().err()).unwrap().code,
        "CONFLICT"
    );
    assert_eq!(count(&db, &saved.id), 2);
    let current = db.recipe(&saved.id).unwrap();
    assert_eq!(
        super::recipe_versions::read_version(&db.conn, &saved.id, 2)
            .unwrap()
            .snapshot
            .title,
        current.title
    );
    integrity(&db);
}

#[test]
fn history_checksum_corruption_and_migration_ledger_changes_fail_without_rewriting_history() {
    let dir = tempfile::tempdir().unwrap();
    let mut db = Database::open(dir.path()).unwrap();
    let ingredient = db.create_ingredient("Yulaf").unwrap();
    let saved = db.save_recipe(input(&ingredient.id, "food")).unwrap();
    // Explicitly simulate external corruption; ordinary UPDATE is trigger-blocked.
    db.conn.execute_batch("DROP TRIGGER recipe_versions_immutable_update; UPDATE recipe_versions SET snapshot=json_set(snapshot,'$.title','Bozulmuş');").unwrap();
    assert_eq!(
        super::recipe_versions::read_version(&db.conn, &saved.id, 1)
            .unwrap_err()
            .code,
        "SCHEMA_INTEGRITY"
    );
    let mut edit = saved.to_input();
    edit.title = "Yeni".into();
    assert!(db.save_recipe(edit).is_err());
    assert_eq!(db.recipe(&saved.id).unwrap().revision, saved.revision);
    assert_eq!(count(&db, &saved.id), 1);
    db.conn
        .execute(
            "UPDATE schema_migrations SET checksum=printf('%064d',0) WHERE version=10",
            [],
        )
        .unwrap();
    drop(db);
    let path = dir.path().join("library.sqlite3");
    let before = std::fs::read(&path).unwrap();
    assert_eq!(
        Database::open(dir.path()).err().unwrap().code,
        "SCHEMA_INTEGRITY"
    );
    assert_eq!(before, std::fs::read(path).unwrap());
}

#[test]
fn baseline_pages_more_than_one_batch_and_empty_startups_are_idempotent() {
    let dir = tempfile::tempdir().unwrap();
    let mut old = historical_db(dir.path());
    let ingredient = old.create_ingredient("Su").unwrap();
    for _ in 0..70 {
        insert_old(&mut old, &input(&ingredient.id, "beverage"));
    }
    drop(old);
    for _ in 0..2 {
        let db = Database::open(dir.path()).unwrap();
        assert_eq!(
            db.conn
                .query_row(
                    "SELECT count(*) FROM recipe_versions WHERE provenance='upgrade_baseline'",
                    [],
                    |r| r.get::<_, i64>(0)
                )
                .unwrap(),
            70
        );
        integrity(&db);
    }
    let empty = tempfile::tempdir().unwrap();
    for _ in 0..2 {
        let db = Database::open(empty.path()).unwrap();
        assert_eq!(
            db.conn
                .query_row("SELECT count(*) FROM recipe_versions", [], |r| r
                    .get::<_, i64>(0))
                .unwrap(),
            0
        );
        integrity(&db);
    }
}

#[test]
fn interrupted_baseline_writer_child() {
    let Ok(path) = std::env::var("RECIPE_VERSION_CRASH_DIR") else {
        return;
    };
    let path = std::path::Path::new(&path);
    let mut conn = Connection::open(path.join("library.sqlite3")).unwrap();
    conn.pragma_update(None, "foreign_keys", true).unwrap();
    let tx = conn
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .unwrap();
    tx.execute_batch(migrations::MIGRATIONS[9].sql).unwrap();
    super::recipe_versions::backfill(&tx).unwrap();
    std::fs::write(path.join("ready"), b"backfill is uncommitted").unwrap();
    loop {
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
}

#[test]
fn process_killed_before_baseline_commit_reopens_the_original_profile_then_retries() {
    let dir = tempfile::tempdir().unwrap();
    let mut old = historical_db(dir.path());
    let ingredient = old.create_ingredient("Süt").unwrap();
    let value = input(&ingredient.id, "food");
    insert_old(&mut old, &value);
    drop(old);
    let mut child = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "persistence::recipe_version_tests::interrupted_baseline_writer_child",
            "--nocapture",
        ])
        .env("RECIPE_VERSION_CRASH_DIR", dir.path())
        .stdout(std::process::Stdio::null())
        .spawn()
        .unwrap();
    let start = std::time::Instant::now();
    while !dir.path().join("ready").exists() {
        if start.elapsed() > std::time::Duration::from_secs(20) {
            let _ = child.kill();
            let _ = child.wait();
            panic!("child did not backfill");
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    child.kill().unwrap();
    child.wait().unwrap();
    let reader = Connection::open(dir.path().join("library.sqlite3")).unwrap();
    assert_eq!(
        migrations::validate(&reader, &migrations::MIGRATIONS[..9]).unwrap(),
        9
    );
    drop(reader);
    let db = Database::open(dir.path()).unwrap();
    assert_eq!(count(&db, &value.id), 1);
    assert_eq!(
        super::recipe_versions::read_version(&db.conn, &value.id, 1)
            .unwrap()
            .summary
            .provenance,
        "upgrade_baseline"
    );
    integrity(&db);
}
