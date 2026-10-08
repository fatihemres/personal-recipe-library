use super::*;
use crate::domain::recipes::*;
use rusqlite::params;
fn id() -> String {
    uuid::Uuid::new_v4().to_string()
}
fn input(ingredient: &Ingredient) -> RecipeInput {
    RecipeInput {
        id: id(),
        expected_revision: None,
        title: "İçli köfte".into(),
        description: Some("Türk mutfağı".into()),
        kind: "food".into(),
        servings: "2.50".into(),
        prep_minutes: Some(20),
        cook_minutes: None,
        notes: Some("Özel not".into()),
        ingredients: vec![RecipeIngredient {
            id: id(),
            ingredient_id: ingredient.id.clone(),
            quantity: Some("0.123456".into()),
            unit_code: "cc".into(),
            note: Some("Soğuk".into()),
        }],
        steps: vec![
            RecipeStep {
                id: id(),
                instructions: "Önce hazırlayın".into(),
            },
            RecipeStep {
                id: id(),
                instructions: "Sonra pişirin".into(),
            },
        ],
    }
}
#[test]
fn recipes_persist_edit_order_delete_restore_and_personal_ingredients_survive_restart() {
    let dir = tempfile::tempdir().unwrap();
    let mut db = Database::open(dir.path()).unwrap();
    let ingredient = db.create_ingredient("İçme suyu").unwrap();
    let draft = input(&ingredient);
    let recipe = db.save_recipe(draft.clone()).unwrap();
    assert_eq!(recipe.ingredients[0].quantity.as_deref(), Some("0.123456"));
    assert_eq!(recipe.ingredients[0].unit_code, "cc");
    assert_eq!(recipe.servings, "2.50");
    drop(db);
    let mut db = Database::open(dir.path()).unwrap();
    let recipe = db.recipe(&draft.id).unwrap();
    assert_eq!(recipe.title, "İçli köfte");
    assert_eq!(recipe.steps[1].instructions, "Sonra pişirin");
    let mut updated = draft.clone();
    updated.expected_revision = Some(recipe.revision);
    updated.title = "Güncel içli köfte".into();
    updated.steps.reverse();
    updated.ingredients[0].quantity = Some("999999999999.000001".into());
    let edited = db.save_recipe(updated.clone()).unwrap();
    assert_eq!(edited.steps[0].id, draft.steps[1].id);
    assert_eq!(edited.ingredients[0].id, draft.ingredients[0].id);
    assert_eq!(db.save_recipe(updated).unwrap_err().code, "CONFLICT");
    let deleted = db.set_deleted(&edited.id, edited.revision, true).unwrap();
    assert!(deleted.deleted_at.is_some());
    assert!(db.list_recipes(false, None).unwrap().is_empty());
    assert_eq!(db.list_recipes(true, None).unwrap().len(), 1);
    assert_eq!(db.ingredients("içme").unwrap()[0].id, ingredient.id);
    drop(db);
    let mut db = Database::open(dir.path()).unwrap();
    let restored = db
        .set_deleted(&deleted.id, deleted.revision, false)
        .unwrap();
    assert_eq!(restored.steps[0].id, draft.steps[1].id);
    assert_eq!(
        restored.ingredients[0].quantity.as_deref(),
        Some("999999999999.000001")
    );
    assert_eq!(
        db.list_recipes(false, Some("beverage".into()))
            .unwrap()
            .len(),
        0
    );
}
#[test]
fn unicode_names_normalize_deduplicate_and_parameterized_search_is_safe() {
    let dir = tempfile::tempdir().unwrap();
    let mut db = Database::open(dir.path()).unwrap();
    let a = db.create_ingredient("  İÇME   Suyu ").unwrap();
    let b = db.create_ingredient("içme suyu").unwrap();
    assert_eq!(a.id, b.id);
    let c = db.create_ingredient("IŞIK").unwrap();
    assert_eq!(db.ingredients("ışık").unwrap()[0].id, c.id);
    let d = db.create_ingredient("Yoğurt").unwrap();
    assert_eq!(d.id, db.create_ingredient("Yog\u{306}urt").unwrap().id);
    let sql = db.create_ingredient("x'); DROP TABLE recipes; --").unwrap();
    assert_eq!(db.ingredients("DROP TABLE").unwrap()[0].id, sql.id);
    assert!(db.create_ingredient("  ").is_err());
    assert_eq!(db.units().unwrap().len(), 6);
}
#[test]
fn invalid_quantities_references_and_child_conflicts_roll_back_all_related_writes() {
    let dir = tempfile::tempdir().unwrap();
    let mut db = Database::open(dir.path()).unwrap();
    let ingredient = db.create_ingredient("Un").unwrap();
    for q in [
        "0",
        "-1",
        "1e3",
        "NaN",
        "0.0000001",
        "1.2.3",
        "01",
        "1,5",
        "1000000000000",
    ] {
        let mut draft = input(&ingredient);
        draft.ingredients[0].quantity = Some(q.into());
        assert!(db.save_recipe(draft).is_err(), "{q}");
    }
    assert!(db.list_recipes(false, None).unwrap().is_empty());
    let original = input(&ingredient);
    let saved = db.save_recipe(original.clone()).unwrap();
    let mut broken = original.clone();
    broken.expected_revision = Some(saved.revision);
    broken.title = "Must rollback".into();
    broken.ingredients[0].ingredient_id = id();
    assert!(db.save_recipe(broken).is_err());
    assert_eq!(db.recipe(&saved.id).unwrap().title, saved.title);
    let mut other = input(&ingredient);
    other.steps[1].id = saved.steps[0].id.clone();
    assert!(db.save_recipe(other).is_err());
    assert_eq!(db.list_recipes(false, None).unwrap().len(), 1);
    assert!(db
        .conn
        .execute("DELETE FROM ingredients WHERE id=?1", [&ingredient.id])
        .is_err());
    let mut blank = input(&ingredient);
    blank.ingredients[0].quantity = None;
    blank.steps.clear();
    assert!(db.save_recipe(blank).is_ok());
}
#[test]
fn real_m1_upgrade_preserves_preferences_and_validates_both_checksums() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("library.sqlite3");
    let mut conn = Connection::open(&path).unwrap();
    migrations::apply(&mut conn, &path, &migrations::MIGRATIONS[..1]).unwrap();
    conn.execute("UPDATE preferences SET theme='dark'", [])
        .unwrap();
    drop(conn);
    let db = Database::open(dir.path()).unwrap();
    assert_eq!(db.bootstrap().unwrap().storage.schema_version, 2);
    assert_eq!(db.preferences().unwrap().theme, crate::domain::Theme::Dark);
    assert_eq!(
        migrations::validate(&db.conn, migrations::MIGRATIONS).unwrap(),
        2
    );
    drop(db);
    for _ in 0..3 {
        let db = Database::open(dir.path()).unwrap();
        assert_eq!(db.units().unwrap().len(), 6);
    }
    assert_eq!(
        std::fs::read_dir(dir.path())
            .unwrap()
            .filter_map(Result::ok)
            .filter(|e| e
                .file_name()
                .to_string_lossy()
                .starts_with("pre-migration-1-"))
            .count(),
        1
    );
}

#[test]
fn edit_replaces_removed_rows_and_database_rejects_malformed_decimal_text() {
    let dir = tempfile::tempdir().unwrap();
    let mut db = Database::open(dir.path()).unwrap();
    let ingredient = db.create_ingredient("Un").unwrap();
    let original = input(&ingredient);
    let saved = db.save_recipe(original.clone()).unwrap();
    for value in ["1..2", ".2", "2.", "01", "0", "1.1234567"] {
        assert!(
            db.conn
                .execute(
                    "UPDATE recipe_ingredients SET quantity=?1 WHERE recipe_id=?2",
                    params![value, saved.id]
                )
                .is_err(),
            "{value}"
        );
    }
    let mut edited = original;
    edited.expected_revision = Some(saved.revision);
    edited.ingredients.clear();
    edited.steps.remove(0);
    edited.steps.push(RecipeStep {
        id: id(),
        instructions: "Servis edin".into(),
    });
    let result = db.save_recipe(edited).unwrap();
    assert!(result.ingredients.is_empty());
    assert_eq!(result.steps[0].id, saved.steps[1].id);
    assert_eq!(result.steps[1].instructions, "Servis edin");
    assert_eq!(result.created_at, saved.created_at);
    let units = db.units().unwrap();
    let cc = units.iter().find(|u| u.code == "cc").unwrap();
    assert_eq!(cc.canonical_code, "mL");
    assert_eq!(cc.factor, 1);
}
