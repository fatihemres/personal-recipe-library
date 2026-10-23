use super::*;
use crate::domain::{recipe_search::RecipeSearchQuery, recipes::*};
use std::time::Instant;

fn id() -> String {
    uuid::Uuid::new_v4().to_string()
}
fn query(text: &str) -> RecipeSearchQuery {
    RecipeSearchQuery {
        text: text.into(),
        scope: "active".into(),
        kind: None,
        ingredient_id: None,
        created_from: None,
        created_to: None,
        modified_from: None,
        modified_to: None,
        sort: "relevance".into(),
        offset: 0,
        limit: 24,
    }
}
fn input(title: &str, ingredient: &Ingredient) -> RecipeInput {
    RecipeInput {
        id: id(),
        expected_revision: None,
        title: title.into(),
        description: Some("Evde hazırlanan".into()),
        kind: "food".into(),
        servings: "2".into(),
        prep_minutes: None,
        cook_minutes: None,
        notes: Some("Özel not".into()),
        ingredients: vec![RecipeIngredient {
            id: id(),
            ingredient_id: ingredient.id.clone(),
            quantity: Some("0.123456".into()),
            unit_code: "g".into(),
            note: None,
        }],
        steps: vec![RecipeStep {
            id: id(),
            instructions: "Yavaşça karıştır".into(),
        }],
    }
}
#[test]
fn real_fts_tracks_recipe_writes_turkish_ingredient_rename_and_lifecycle() {
    let dir = tempfile::tempdir().unwrap();
    let mut db = Database::open(dir.path()).unwrap();
    let ingredient = db.create_ingredient("Şeker").unwrap();
    let mut draft = input("İncir Tatlısı", &ingredient);
    let saved = db.save_recipe(draft.clone()).unwrap();
    for text in ["İnc", "incir", "Şek", "şek", "ŞEK", "karıştır", "Özel"] {
        assert_eq!(db.search_recipes(query(text)).unwrap().total, 1, "{text}");
    }
    assert_eq!(db.search_recipes(query("' OR 1=1 --")).unwrap().total, 0);
    draft.expected_revision = Some(saved.revision);
    draft.title = "Işık Tatlısı".into();
    draft.steps[0].instructions = "Soğut".into();
    let saved = db.save_recipe(draft.clone()).unwrap();
    assert_eq!(db.search_recipes(query("incir")).unwrap().total, 0);
    assert_eq!(db.search_recipes(query("ışık")).unwrap().total, 1);
    assert_eq!(db.search_recipes(query("IŞIK")).unwrap().total, 1);
    assert_eq!(db.search_recipes(query("karıştır")).unwrap().total, 0);
    let original_fts: i64 = db
        .conn
        .query_row("SELECT count(*) FROM recipe_fts", [], |r| r.get(0))
        .unwrap();
    db.rebuild_recipe_search().unwrap();
    assert_eq!(
        db.conn
            .query_row("SELECT count(*) FROM recipe_fts", [], |r| r
                .get::<_, i64>(0))
            .unwrap(),
        original_fts
    );
    let archived = db.archive_recipe(&saved.id, saved.revision, true).unwrap();
    assert_eq!(db.search_recipes(query("ışık")).unwrap().total, 0);
    let mut archived_query = query("ışık");
    archived_query.scope = "archived".into();
    assert_eq!(db.search_recipes(archived_query).unwrap().total, 1);
    let deleted = db
        .set_deleted(&archived.id, archived.revision, true)
        .unwrap();
    let mut trash = query("ışık");
    trash.scope = "trash".into();
    assert_eq!(db.search_recipes(trash.clone()).unwrap().total, 1);
    let restored = db
        .set_deleted(&deleted.id, deleted.revision, false)
        .unwrap();
    assert_eq!(db.search_recipes(trash).unwrap().total, 0);
    assert_eq!(
        db.recipe(&restored.id).unwrap().ingredients[0]
            .quantity
            .as_deref(),
        Some("0.123456")
    );
    let deleted = db
        .set_deleted(&restored.id, restored.revision, true)
        .unwrap();
    db.purge_recipe(&deleted.id, deleted.revision).unwrap();
    assert_eq!(
        db.conn
            .query_row("SELECT count(*) FROM recipe_fts", [], |r| r
                .get::<_, i64>(0))
            .unwrap(),
        0
    );
    assert_eq!(
        db.conn
            .query_row("PRAGMA integrity_check", [], |r| r.get::<_, String>(0))
            .unwrap(),
        "ok"
    );
}

#[test]
fn real_search_composes_filters_sorts_pages_and_bounds_typo_work() {
    let dir = tempfile::tempdir().unwrap();
    let mut db = Database::open(dir.path()).unwrap();
    let ingredient = db.create_ingredient("Domates").unwrap();
    let other = db.create_ingredient("Süt").unwrap();
    for (index, title) in ["Çorba", "Cacık", "Ispanak", "İncir", "Armut"]
        .iter()
        .enumerate()
    {
        let item = if index < 3 { &ingredient } else { &other };
        db.save_recipe(input(title, item)).unwrap();
    }
    let mut q = query("");
    q.sort = "title_asc".into();
    q.limit = 2;
    let first = db.search_recipes(q.clone()).unwrap();
    assert_eq!(first.total, 5);
    assert_eq!(
        first
            .items
            .iter()
            .map(|v| v.title.as_str())
            .collect::<Vec<_>>(),
        vec!["Armut", "Cacık"]
    );
    assert!(first.has_more);
    q.offset = 2;
    let second = db.search_recipes(q.clone()).unwrap();
    assert_eq!(
        second
            .items
            .iter()
            .map(|v| v.title.as_str())
            .collect::<Vec<_>>(),
        vec!["Çorba", "Ispanak"]
    );
    q.ingredient_id = Some(ingredient.id.clone());
    q.offset = 0;
    assert_eq!(db.search_recipes(q.clone()).unwrap().total, 3);
    q.kind = Some("beverage".into());
    assert_eq!(db.search_recipes(q.clone()).unwrap().total, 0);
    q.kind = None;
    q.created_from = Some("9999-01-01".into());
    assert_eq!(db.search_recipes(q.clone()).unwrap().total, 0);
    q.created_from = None;
    q.ingredient_id = None;
    q.text = "Çorbq".into();
    let fuzzy = db.search_recipes(q).unwrap();
    assert_eq!(fuzzy.total, 0);
    assert_eq!(fuzzy.suggestions, vec!["Çorba"]);
    assert!(db.search_recipes(query("a\0b")).is_err());
    assert!(db.search_recipes(query(&"a".repeat(201))).is_err());
    let foreign_keys = db
        .conn
        .prepare("PRAGMA foreign_key_check")
        .unwrap()
        .query([])
        .unwrap()
        .next()
        .unwrap()
        .is_none();
    assert!(foreign_keys);
    let count: i64 = db
        .conn
        .query_row("SELECT count(*) FROM recipe_search_docs", [], |r| r.get(0))
        .unwrap();
    assert_eq!(count, 5);
    db.conn
        .execute("UPDATE recipe_search_state SET ready=0 WHERE id=1", [])
        .unwrap();
    drop(db);
    let db = Database::open(dir.path()).unwrap();
    assert_eq!(db.search_recipes(query("çorba")).unwrap().total, 1);
}

#[test]
fn english_catalog_aliases_and_personal_renames_refresh_linked_recipe_index() {
    let dir = tempfile::tempdir().unwrap();
    let mut db = Database::open(dir.path()).unwrap();
    db.install_bundled_catalog().unwrap();
    let sugar = db
        .ingredients("Granulated sugar")
        .unwrap()
        .into_iter()
        .find(|i| i.name == "Toz şeker")
        .unwrap();
    let saved = db.save_recipe(input("Sade tatlı", &sugar)).unwrap();
    for term in ["Toz", "Granulated", "White sugar"] {
        let page = db.search_recipes(query(term)).unwrap();
        assert_eq!(page.total, 1, "{term}");
        assert_eq!(page.items[0].id, saved.id);
    }
    let personal = db.create_ingredient("Deneme otum").unwrap();
    db.save_recipe(input("Otlu yemek", &personal)).unwrap();
    assert_eq!(db.search_recipes(query("Deneme")).unwrap().total, 1);
    db.edit_ingredient(crate::domain::reliability::IngredientEdit {
        id: personal.id.clone(),
        revision: 1,
        name: "Benim özel otum".into(),
        notes: None,
        preferred_unit: None,
        category_ids: None,
    })
    .unwrap();
    assert_eq!(db.search_recipes(query("Deneme")).unwrap().total, 0);
    assert_eq!(db.search_recipes(query("Benim")).unwrap().total, 1);
    drop(db);
    let db = Database::open(dir.path()).unwrap();
    assert_eq!(db.search_recipes(query("White sugar")).unwrap().total, 1);
    assert_eq!(db.search_recipes(query("Benim")).unwrap().total, 1);
}

#[test]
fn catalog_sized_isolated_recipe_collection_has_bounded_search_pages() {
    let dir = tempfile::tempdir().unwrap();
    let mut db = Database::open(dir.path()).unwrap();
    db.install_bundled_catalog().unwrap();
    let catalog_count: i64 = db
        .conn
        .query_row("SELECT count(*) FROM catalog_ingredients", [], |r| r.get(0))
        .unwrap();
    assert_eq!(catalog_count, 854);
    let ingredient = db.create_ingredient("Yük testi malzemesi").unwrap();
    let start = Instant::now();
    for n in 0..500 {
        db.save_recipe(input(&format!("Deneme tarifi {n:04}"), &ingredient))
            .unwrap();
    }
    let insert_elapsed = start.elapsed();
    let mut samples = Vec::new();
    for n in 0..100 {
        let mut q = query(if n % 2 == 0 { "Deneme" } else { "malzemesi" });
        q.offset = ((n % 10) * 24) as i64;
        let started = Instant::now();
        let page = db.search_recipes(q).unwrap();
        samples.push(started.elapsed().as_micros());
        assert_eq!(page.total, 500);
        assert_eq!(page.items.len(), 24);
    }
    samples.sort_unstable();
    eprintln!(
        "M4A isolated benchmark: catalog={catalog_count}, recipes=500, insert_ms={}, search_us_p50={}, search_us_p95={}, search_us_max={}",
        insert_elapsed.as_millis(),
        samples[49],
        samples[94],
        samples[99]
    );
    assert_eq!(
        db.conn
            .query_row("PRAGMA integrity_check", [], |r| r.get::<_, String>(0))
            .unwrap(),
        "ok"
    );
}

#[test]
fn failed_index_write_rolls_back_recipe_and_keeps_previous_search_result() {
    let dir = tempfile::tempdir().unwrap();
    let mut db = Database::open(dir.path()).unwrap();
    let ingredient = db.create_ingredient("Armut").unwrap();
    let mut edit = input("Eski tarif", &ingredient);
    let saved = db.save_recipe(edit.clone()).unwrap();
    db.conn.execute_batch(
        "CREATE TRIGGER m4a_test_abort BEFORE UPDATE ON recipe_search_docs BEGIN SELECT RAISE(ABORT, 'test rollback'); END;",
    ).unwrap();
    edit.expected_revision = Some(saved.revision);
    edit.title = "Yeni tarif".into();
    assert!(db.save_recipe(edit).is_err());
    assert_eq!(db.recipe(&saved.id).unwrap().title, "Eski tarif");
    assert_eq!(db.search_recipes(query("Eski")).unwrap().total, 1);
    assert_eq!(db.search_recipes(query("Yeni")).unwrap().total, 0);
}
