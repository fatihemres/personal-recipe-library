use super::*;
use crate::domain::{recipes::*, reliability::*};
use rusqlite::params;
fn id() -> String {
    uuid::Uuid::new_v4().to_string()
}
fn input(ingredient: &Ingredient) -> RecipeInput {
    RecipeInput {
        id: id(),
        expected_revision: None,
        title: "Şekerli içecek".into(),
        kind: "beverage".into(),
        description: Some("İçecek açıklaması".into()),
        servings: "2.50".into(),
        prep_minutes: Some(5),
        cook_minutes: None,
        notes: Some("Not".into()),
        ingredients: vec![RecipeIngredient {
            id: id(),
            ingredient_id: ingredient.id.clone(),
            quantity: Some("35.000001".into()),
            unit_code: "cc".into(),
            note: Some("Özel".into()),
        }],
        steps: vec![
            RecipeStep {
                id: id(),
                instructions: "Karıştırın".into(),
            },
            RecipeStep {
                id: id(),
                instructions: "Servis edin".into(),
            },
        ],
    }
}
#[test]
fn actual_saved_ingredient_search_handles_all_requested_turkish_variants_and_empty_catalog() {
    let dir = tempfile::tempdir().unwrap();
    let mut db = Database::open(dir.path()).unwrap();
    let empty = db.search_personal("Şek").unwrap();
    assert_eq!(empty.total, 0);
    assert!(empty.items.is_empty());
    let sugar = db.create_ingredient("Şeker").unwrap();
    drop(db);
    let mut db = Database::open(dir.path()).unwrap();
    for query in ["Şek", "şek", "ŞEK", "şeker"] {
        let found = db.search_personal(query).unwrap();
        assert_eq!(found.total, 1);
        assert_eq!(found.items[0].id, sugar.id, "{query}");
        assert_eq!(db.ingredients(query).unwrap()[0].id, sugar.id);
    }
    assert_eq!(db.create_ingredient("  ŞEKER ").unwrap().id, sugar.id);
    assert!(db.search_personal("olmayan").unwrap().items.is_empty());
    assert_eq!(db.search_personal("olmayan").unwrap().total, 1);
    let dotted = db.create_ingredient("İçme suyu").unwrap();
    let dotless = db.create_ingredient("Ihlamur").unwrap();
    for query in ["İÇ", "iç", "İç"] {
        assert_eq!(db.search_personal(query).unwrap().items[0].id, dotted.id);
    }
    for query in ["IHL", "ıhl", "Ihl"] {
        assert_eq!(db.search_personal(query).unwrap().items[0].id, dotless.id);
    }
    assert!(db.search_personal("ihl").unwrap().items.is_empty()); // Turkish dotted i is distinct from dotless ı.
}
#[test]
fn incomplete_new_and_edit_drafts_recover_without_overwriting_saved_recipe() {
    let dir = tempfile::tempdir().unwrap();
    let mut db = Database::open(dir.path()).unwrap();
    let ingredient = db.create_ingredient("Şeker").unwrap();
    let saved = db.save_recipe(input(&ingredient)).unwrap();
    let mut edit = saved.to_input();
    edit.title = "Yeni başlık".into();
    edit.ingredients[0].quantity = Some("1.".into());
    edit.steps[1].instructions.clear();
    let draft = db
        .save_draft(DraftWrite {
            id: id(),
            expected_revision: None,
            input: edit.clone(),
        })
        .unwrap();
    let mut new = input(&ingredient);
    new.title.clear();
    new.servings = "0.".into();
    let new_draft = db
        .save_draft(DraftWrite {
            id: id(),
            expected_revision: None,
            input: new,
        })
        .unwrap();
    assert_eq!(db.recipe(&saved.id).unwrap().title, saved.title);
    assert_eq!(db.scope_recipes("active", None).unwrap().len(), 1);
    drop(db);
    let mut db = Database::open(dir.path()).unwrap();
    let recovered = db.draft(&draft.id).unwrap();
    assert_eq!(recovered.recipe_id.as_deref(), Some(saved.id.as_str()));
    assert_eq!(
        recovered.input.ingredients[0].quantity.as_deref(),
        Some("1.")
    );
    assert!(recovered.input.steps[1].instructions.is_empty());
    assert_eq!(db.drafts().unwrap().len(), 2);
    edit.ingredients[0].quantity = Some("1.000001".into());
    edit.steps[1].instructions = "Bitti".into();
    let updated = db
        .save_draft(DraftWrite {
            id: draft.id.clone(),
            expected_revision: Some(draft.revision),
            input: edit.clone(),
        })
        .unwrap();
    let committed = db
        .commit_recipe(edit.clone(), &draft.id, updated.revision, false)
        .unwrap();
    assert_eq!(committed.title, "Yeni başlık");
    assert_eq!(committed.revision, 2);
    assert_eq!(db.drafts().unwrap().len(), 1);
    assert!(db
        .save_draft(DraftWrite {
            id: draft.id,
            expected_revision: Some(updated.revision),
            input: edit
        })
        .is_err());
    db.discard_draft(&new_draft.id, new_draft.revision).unwrap();
    assert!(db.drafts().unwrap().is_empty());
    assert!(db
        .save_draft(DraftWrite {
            id: new_draft.id,
            expected_revision: None,
            input: new_draft.input
        })
        .is_err());
}
#[test]
fn duplicate_is_independent_preserves_order_and_reuses_shared_ingredients() {
    let dir = tempfile::tempdir().unwrap();
    let mut db = Database::open(dir.path()).unwrap();
    let ingredient = db.create_ingredient("Şeker").unwrap();
    let source = db.save_recipe(input(&ingredient)).unwrap();
    let copy = db.duplicate_recipe(&source.id, source.revision).unwrap();
    assert_ne!(copy.id, source.id);
    assert_eq!(copy.title, source.title);
    assert_eq!(copy.servings, "2.50");
    assert_eq!(copy.steps[0].instructions, source.steps[0].instructions);
    assert_ne!(copy.steps[0].id, source.steps[0].id);
    assert_ne!(copy.ingredients[0].id, source.ingredients[0].id);
    assert_eq!(copy.ingredients[0].ingredient_id, ingredient.id);
    assert_eq!(copy.ingredients[0].quantity, source.ingredients[0].quantity);
    assert_eq!(copy.ingredients[0].unit_code, "cc");
    let mut edit = copy.to_input();
    edit.title = "Kopya düzenlendi".into();
    db.save_recipe(edit).unwrap();
    assert_eq!(db.recipe(&source.id).unwrap().title, source.title);
    assert_eq!(db.search_personal("").unwrap().total, 1);
}
#[test]
fn archive_trash_restore_purge_retains_shared_ingredients_and_orphan_drafts() {
    let dir = tempfile::tempdir().unwrap();
    let mut db = Database::open(dir.path()).unwrap();
    let ingredient = db.create_ingredient("Şeker").unwrap();
    let saved = db.save_recipe(input(&ingredient)).unwrap();
    let draft = db
        .save_draft(DraftWrite {
            id: id(),
            expected_revision: None,
            input: saved.to_input(),
        })
        .unwrap();
    let archived = db.archive_recipe(&saved.id, 1, true).unwrap();
    assert!(db.scope_recipes("active", None).unwrap().is_empty());
    assert_eq!(db.scope_recipes("archived", None).unwrap().len(), 1);
    assert!(db.purge_recipe(&saved.id, 2).is_err());
    assert!(db
        .commit_recipe(draft.input.clone(), &draft.id, draft.revision, false)
        .is_err());
    assert_eq!(db.drafts().unwrap().len(), 1);
    let deleted = db.set_deleted(&saved.id, archived.revision, true).unwrap();
    let restored = db.set_deleted(&saved.id, deleted.revision, false).unwrap();
    assert!(restored.archived_at.is_some());
    let active = db
        .archive_recipe(&saved.id, restored.revision, false)
        .unwrap();
    let deleted = db.set_deleted(&saved.id, active.revision, true).unwrap();
    db.purge_recipe(&saved.id, deleted.revision).unwrap();
    let orphan = db.draft(&draft.id).unwrap();
    assert!(orphan.recipe_id.is_none());
    assert_eq!(db.ingredients("Şek").unwrap()[0].id, ingredient.id);
    assert!(db.delete_ingredient(&ingredient.id, 1).is_err());
    let rescued = db
        .commit_recipe(orphan.input.clone(), &orphan.id, orphan.revision, true)
        .unwrap();
    assert_ne!(rescued.id, saved.id);
    assert_eq!(rescued.ingredients[0].ingredient_id, ingredient.id);
    assert!(db.drafts().unwrap().is_empty());
    drop(db);
    let db = Database::open(dir.path()).unwrap();
    assert_eq!(db.recipe(&rescued.id).unwrap().title, saved.title);
}
#[test]
fn ingredient_rename_duplicate_conflicts_stale_writes_and_draft_references_are_safe() {
    let dir = tempfile::tempdir().unwrap();
    let mut db = Database::open(dir.path()).unwrap();
    let ingredient = db.create_ingredient("Şeker").unwrap();
    db.create_ingredient("Tuz").unwrap();
    let recipe = db.save_recipe(input(&ingredient)).unwrap();
    let copy = db.duplicate_recipe(&recipe.id, 1).unwrap();
    let edit = |name: &str, revision| IngredientEdit {
        id: ingredient.id.clone(),
        name: name.into(),
        revision,
        notes: Some("Kişisel bilgi".into()),
        preferred_unit: Some("g".into()),
    };
    assert_eq!(
        db.edit_ingredient(edit("TUZ", 1)).unwrap_err().code,
        "INGREDIENT_DUPLICATE"
    );
    let updated = db.edit_ingredient(edit("Esmer şeker", 1)).unwrap();
    assert_eq!(updated.revision, 2);
    assert_eq!(
        db.edit_ingredient(edit("Başka", 1)).unwrap_err().code,
        "CONFLICT"
    );
    for id in [&recipe.id, &copy.id] {
        assert_eq!(
            db.recipe(id).unwrap().ingredient_names[&ingredient.id],
            "Esmer şeker"
        );
    }
    assert_eq!(
        db.delete_ingredient(&ingredient.id, 2).unwrap_err().code,
        "INGREDIENT_REFERENCED"
    );
    let unused = db.create_ingredient("Yoğurt").unwrap();
    let mut draft_input = input(&unused);
    draft_input.title.clear();
    let draft = db
        .save_draft(DraftWrite {
            id: id(),
            expected_revision: None,
            input: draft_input,
        })
        .unwrap();
    assert!(db.delete_ingredient(&unused.id, 1).is_err());
    db.discard_draft(&draft.id, 1).unwrap();
    db.delete_ingredient(&unused.id, 1).unwrap();
}
#[test]
fn stale_sessions_and_failed_commit_roll_back_recipe_and_keep_drafts() {
    let dir = tempfile::tempdir().unwrap();
    let mut db = Database::open(dir.path()).unwrap();
    let ingredient = db.create_ingredient("Şeker").unwrap();
    let saved = db.save_recipe(input(&ingredient)).unwrap();
    let mut first = saved.to_input();
    first.title = "İlk oturum".into();
    let a = db
        .save_draft(DraftWrite {
            id: id(),
            expected_revision: None,
            input: first.clone(),
        })
        .unwrap();
    let mut second = saved.to_input();
    second.title = "İkinci oturum".into();
    let b = db
        .save_draft(DraftWrite {
            id: id(),
            expected_revision: None,
            input: second.clone(),
        })
        .unwrap();
    let mut other = Database::open(dir.path()).unwrap();
    other
        .commit_recipe(first, &a.id, a.revision, false)
        .unwrap();
    assert_eq!(
        db.commit_recipe(second.clone(), &b.id, b.revision, false)
            .unwrap_err()
            .code,
        "CONFLICT"
    );
    assert_eq!(db.draft(&b.id).unwrap().input.title, second.title);
    let next = db
        .save_draft(DraftWrite {
            id: b.id.clone(),
            expected_revision: Some(b.revision),
            input: second.clone(),
        })
        .unwrap();
    assert_eq!(
        other.discard_draft(&b.id, b.revision).unwrap_err().code,
        "DRAFT_CONFLICT"
    );
    let copied = db
        .commit_recipe(second, &b.id, next.revision, true)
        .unwrap();
    assert_ne!(copied.id, saved.id);
    let mut broken = input(&ingredient);
    broken.ingredients[0].ingredient_id = id();
    assert!(db
        .save_draft(DraftWrite {
            id: id(),
            expected_revision: None,
            input: broken
        })
        .is_err());
    assert!(db.drafts().unwrap().is_empty());
    let draft = db
        .save_draft(DraftWrite {
            id: id(),
            expected_revision: None,
            input: input(&ingredient),
        })
        .unwrap();
    let mut collision = draft.input.clone();
    collision.steps[0].id = copied.steps[0].id.clone();
    assert!(db.commit_recipe(collision, &draft.id, 1, false).is_err());
    assert!(db.recipe(&draft.input.id).is_err());
    assert_eq!(db.draft(&draft.id).unwrap().revision, 1);
}
#[test]
fn m2a_migration_preserves_records_decimal_values_settings_and_checksums() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("library.sqlite3");
    let mut conn = Connection::open(&path).unwrap();
    conn.pragma_update(None, "foreign_keys", true).unwrap();
    migrations::apply(&mut conn, &path, &migrations::MIGRATIONS[..2]).unwrap();
    let ingredient = id();
    let recipe = id();
    conn.execute(
        "INSERT INTO ingredients(id,name,search_name) VALUES(?1,'Şeker','şeker')",
        [&ingredient],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO recipes(id,title,kind,servings) VALUES(?1,'İçecek','beverage','2.50')",
        [&recipe],
    )
    .unwrap();
    conn.execute("INSERT INTO recipe_ingredients(id,recipe_id,ingredient_id,quantity,unit_code,position) VALUES(?1,?2,?3,'35.000001','cc',0)",params![id(),recipe,ingredient]).unwrap();
    conn.execute("UPDATE preferences SET theme='dark'", [])
        .unwrap();
    drop(conn);
    for _ in 0..3 {
        let db = Database::open(dir.path()).unwrap();
        assert_eq!(
            db.recipe(&recipe).unwrap().ingredients[0]
                .quantity
                .as_deref(),
            Some("35.000001")
        );
        assert_eq!(db.bootstrap().unwrap().storage.schema_version, 4);
        assert_eq!(db.preferences().unwrap().theme, crate::domain::Theme::Dark);
        assert_eq!(
            migrations::validate(&db.conn, migrations::MIGRATIONS).unwrap(),
            4
        );
    }
}
// Run in a child process to test abrupt OS termination with a live SQLite/WAL connection.
#[test]
fn crash_writer_child() {
    let Ok(path) = std::env::var("RECIPE_DRAFT_CRASH_DIR") else {
        return;
    };
    let root = std::path::Path::new(&path);
    let mut db = Database::open(root).unwrap();
    let ingredient = db.create_ingredient("Şeker").unwrap();
    let mut draft = input(&ingredient);
    draft.ingredients[0].quantity = Some("35.".into());
    draft.steps[0].instructions.clear();
    db.save_draft(DraftWrite {
        id: id(),
        expected_revision: None,
        input: draft,
    })
    .unwrap();
    std::fs::write(root.join("ready"), "ok").unwrap();
    loop {
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
}
#[test]
fn unexpected_termination_recovers_committed_draft_from_real_sqlite() {
    let dir = tempfile::tempdir().unwrap();
    let mut child = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "persistence::reliability_tests::crash_writer_child",
            "--nocapture",
        ])
        .env("RECIPE_DRAFT_CRASH_DIR", dir.path())
        .stdout(std::process::Stdio::null())
        .spawn()
        .unwrap();
    let start = std::time::Instant::now();
    while !dir.path().join("ready").exists() {
        if start.elapsed() > std::time::Duration::from_secs(10) {
            let _ = child.kill();
            let _ = child.wait();
            panic!("child did not finish durable write");
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    child.kill().unwrap();
    child.wait().unwrap();
    let db = Database::open(dir.path()).unwrap();
    let drafts = db.drafts().unwrap();
    assert_eq!(drafts.len(), 1);
    assert_eq!(
        drafts[0].input.ingredients[0].quantity.as_deref(),
        Some("35.")
    );
    assert!(drafts[0].input.steps[0].instructions.is_empty());
    assert!(db.scope_recipes("active", None).unwrap().is_empty());
}
