use super::Database;
use crate::domain::{
    catalog::stable_id,
    ingredient_library::LibraryQuery,
    recipes::{RecipeIngredient, RecipeInput},
    reliability::{DraftWrite, IngredientEdit},
};
fn query(search: &str, origin: &str, category: Option<&str>, offset: i64) -> LibraryQuery {
    LibraryQuery {
        search: search.into(),
        origin: origin.into(),
        category_id: category.map(|k| stable_id("category", k)),
        offset,
        limit: 30,
        view: None,
        uncategorized: false,
    }
}
#[test]
fn personal_category_assignment_is_atomic_and_facets_count_distinct_descendants() {
    let dir = tempfile::tempdir().unwrap();
    let mut db = Database::open(dir.path()).unwrap();
    db.install_bundled_catalog().unwrap();
    let own = db.create_ingredient("Özel şifalı ot").unwrap();
    let herb = stable_id("category", "herbs");
    let food = stable_id("category", "food");
    let edited = db
        .edit_ingredient(IngredientEdit {
            id: own.id.clone(),
            revision: 1,
            name: own.name.clone(),
            notes: None,
            preferred_unit: None,
            category_ids: Some(vec![herb.clone(), food.clone()]),
        })
        .unwrap();
    assert_eq!(edited.revision, 2);
    let detail = db.ingredient_detail(&own.id, "personal").unwrap();
    assert_eq!(detail.item.categories.len(), 2);
    let personal = db
        .ingredient_library(query("Özel şifalı", "personal", Some("food"), 0))
        .unwrap();
    assert_eq!(personal.total, 1);
    assert_eq!(
        personal
            .categories
            .iter()
            .find(|c| c.category.key == "food")
            .unwrap()
            .count,
        1
    );
    let mut uncategorized = query("Özel şifalı", "personal", None, 0);
    uncategorized.uncategorized = true;
    assert_eq!(
        db.ingredient_library(uncategorized.clone()).unwrap().total,
        0
    );
    let bad = db.edit_ingredient(IngredientEdit {
        id: own.id.clone(),
        revision: 2,
        name: "Changed without commit".into(),
        notes: None,
        preferred_unit: None,
        category_ids: Some(vec![uuid::Uuid::new_v4().to_string()]),
    });
    assert!(bad.is_err());
    assert_eq!(
        db.ingredient_detail(&own.id, "personal").unwrap().item.name,
        own.name
    );
    assert_eq!(
        db.ingredient_detail(&own.id, "personal")
            .unwrap()
            .item
            .categories
            .len(),
        2
    );
    db.edit_ingredient(IngredientEdit {
        id: own.id.clone(),
        revision: 2,
        name: own.name.clone(),
        notes: None,
        preferred_unit: None,
        category_ids: Some(vec![]),
    })
    .unwrap();
    assert_eq!(db.ingredient_library(uncategorized).unwrap().total, 1);
    drop(db);
    let mut db = Database::open(dir.path()).unwrap();
    assert!(db
        .ingredient_detail(&own.id, "personal")
        .unwrap()
        .item
        .categories
        .is_empty());
    assert_eq!(db.catalog_status().unwrap().production_definitions, 795);
    db.edit_ingredient(IngredientEdit {
        id: own.id.clone(),
        revision: 3,
        name: own.name.clone(),
        notes: None,
        preferred_unit: None,
        category_ids: Some(vec![herb]),
    })
    .unwrap();
    db.delete_ingredient(&own.id, 4).unwrap();
    assert!(db.ingredient_detail(&own.id, "personal").is_err());
}

#[test]
fn collision_candidates_separate_exact_alias_and_ambiguous_forms_without_merging() {
    let dir = tempfile::tempdir().unwrap();
    let mut db = Database::open(dir.path()).unwrap();
    db.install_bundled_catalog().unwrap();
    let own = db.create_ingredient("İçli karışım").unwrap();
    for spelling in ["İÇLİ KARIŞIM", "içli karışım", "I\u{0307}ÇLİ KARIŞIM"] {
        let matches = db.ingredient_candidates(spelling).unwrap();
        assert!(
            matches
                .iter()
                .any(|c| c.id == own.id && c.reason == "exact_name"),
            "{spelling}"
        );
    }
    assert!(!db
        .ingredient_candidates("IÇLI KARIŞIM")
        .unwrap()
        .iter()
        .any(|c| c.id == own.id));
    let dotless = db.create_ingredient("Iğdır karışımı").unwrap();
    assert!(db
        .ingredient_candidates("ığdır karışımı")
        .unwrap()
        .iter()
        .any(|c| c.id == dotless.id && c.reason == "exact_name"));
    let sugar = db.ingredient_candidates("Granulated sugar").unwrap();
    assert!(sugar
        .iter()
        .any(|c| c.id == stable_id("ingredient", "sugar-granulated") && c.reason == "exact_name"));
    let paste = db.ingredient_candidates("badem ezmesi").unwrap();
    assert!(!paste
        .iter()
        .any(|c| c.id == stable_id("ingredient", "usda-sr-168588")));
    assert!(paste
        .iter()
        .any(|c| c.id == stable_id("ingredient", "usda-sr-170160")));
    assert!(db
        .ingredient_candidates("badem püresi")
        .unwrap()
        .iter()
        .any(|c| c.id == stable_id("ingredient", "usda-sr-168588")));
    assert_ne!(
        stable_id("ingredient", "usda-sr-168588"),
        stable_id("ingredient", "usda-sr-170160")
    );
    assert_eq!(db.catalog_status().unwrap().production_definitions, 795);
}

#[test]
fn category_edit_keeps_saved_recipe_reference_and_exact_quantity_after_restart() {
    let dir = tempfile::tempdir().unwrap();
    let mut db = Database::open(dir.path()).unwrap();
    db.install_bundled_catalog().unwrap();
    let own = db.create_ingredient("Deneme kategori otu").unwrap();
    let saved = db
        .save_recipe(RecipeInput {
            id: uuid::Uuid::new_v4().to_string(),
            expected_revision: None,
            title: "Kategori bağlantısı".into(),
            description: None,
            kind: "food".into(),
            servings: "1".into(),
            prep_minutes: None,
            cook_minutes: None,
            notes: None,
            ingredients: vec![RecipeIngredient {
                id: uuid::Uuid::new_v4().to_string(),
                ingredient_id: own.id.clone(),
                quantity: Some("0.123456".into()),
                unit_code: "g".into(),
                note: None,
            }],
            steps: vec![],
        })
        .unwrap();
    db.edit_ingredient(IngredientEdit {
        id: own.id.clone(),
        revision: 1,
        name: own.name.clone(),
        notes: None,
        preferred_unit: None,
        category_ids: Some(vec![stable_id("category", "herbs")]),
    })
    .unwrap();
    assert_eq!(
        db.delete_ingredient(&own.id, 2).unwrap_err().code,
        "INGREDIENT_REFERENCED"
    );
    drop(db);
    let db = Database::open(dir.path()).unwrap();
    let recipe = db.recipe(&saved.id).unwrap();
    assert_eq!(recipe.ingredients[0].ingredient_id, own.id);
    assert_eq!(recipe.ingredients[0].quantity.as_deref(), Some("0.123456"));
    assert_eq!(
        db.ingredient_detail(&own.id, "personal")
            .unwrap()
            .item
            .categories[0]
            .key,
        "herbs"
    );
}
#[test]
fn library_pages_real_catalog_and_personal_rows_with_exact_distinct_counts() {
    let dir = tempfile::tempdir().unwrap();
    let mut db = Database::open(dir.path()).unwrap();
    db.install_bundled_catalog().unwrap();
    let personal = db.create_ingredient("Ailemin özel karışımı").unwrap();
    let first = db.ingredient_library(query("", "all", None, 0)).unwrap();
    assert_eq!(
        (first.catalog_count, first.personal_count, first.total),
        (795, 1, 796)
    );
    assert_eq!(first.categories.len(), 50);
    let changes: i64 = db
        .conn
        .query_row("SELECT total_changes()", [], |r| r.get(0))
        .unwrap();
    let start = std::time::Instant::now();
    let mut ids = std::collections::HashSet::new();
    for offset in (0..795).step_by(30) {
        let page = db
            .ingredient_library(query("", "catalog", None, offset))
            .unwrap();
        assert_eq!(page.total, 795);
        assert!(page.items.len() <= 30);
        for i in page.items {
            assert_eq!(i.origin, "catalog");
            assert!(ids.insert(i.id));
        }
    }
    assert_eq!(ids.len(), 795);
    assert_eq!(
        db.conn
            .query_row("SELECT total_changes()", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        changes
    );
    eprintln!(
        "25 real library pages / 795 distinct definitions: {:?}",
        start.elapsed()
    );
    let own = db
        .ingredient_library(query("", "personal", None, 0))
        .unwrap();
    assert_eq!(own.total, 1);
    assert_eq!(own.items[0].id, personal.id);
    assert!(db
        .ingredient_library(query("", "catalog", None, 796))
        .unwrap()
        .items
        .is_empty());
}
#[test]
fn library_search_and_descendant_facets_compose_without_duplicate_alias_matches() {
    let dir = tempfile::tempdir().unwrap();
    let mut db = Database::open(dir.path()).unwrap();
    db.install_bundled_catalog().unwrap();
    for (key, queries) in [
        (
            "sugar-granulated",
            vec![
                "Şek",
                "şek",
                "ŞEK",
                "şeker",
                "S\u{0327}EK",
                "Granulated sugar",
            ],
        ),
        ("project-raki", vec!["RAKI", "rakı"]),
        ("project-siyez-grain", vec!["SİYEZ BUĞ", "siyez buğ"]),
        ("project-india-pale-ale", vec!["IPA"]),
    ] {
        for search in queries {
            let p = db
                .ingredient_library(query(search, "catalog", None, 0))
                .unwrap();
            assert_eq!(
                p.items
                    .iter()
                    .filter(|i| i.id == stable_id("ingredient", key))
                    .count(),
                1,
                "{search}"
            );
        }
    }
    let p = db
        .ingredient_library(query("Orgeat", "all", Some("beverage"), 0))
        .unwrap();
    assert_eq!(p.total, 1);
    assert_eq!(
        p.categories
            .iter()
            .find(|c| c.category.key == "syrups")
            .unwrap()
            .count,
        1
    );
    assert_eq!(
        db.ingredient_library(query("Orgeat", "all", Some("food"), 0))
            .unwrap()
            .total,
        0
    );
    let cheeses = db
        .ingredient_library(query("", "catalog", Some("cheeses"), 0))
        .unwrap();
    assert_eq!(cheeses.total, 43);
    let dairy = db
        .ingredient_library(query("", "catalog", Some("dairy"), 0))
        .unwrap();
    assert!(dairy.total >= cheeses.total);
    assert_eq!(
        dairy.total,
        dairy
            .categories
            .iter()
            .find(|c| c.category.key == "dairy")
            .unwrap()
            .count
    );
    assert_eq!(
        db.ingredient_library(query("' OR 1=1 --", "all", None, 0))
            .unwrap()
            .total,
        0
    );
    let mut bad = query("", "all", None, 0);
    bad.limit = 101;
    assert!(db.ingredient_library(bad).is_err());
}
#[test]
fn library_details_and_personal_edit_preserve_recipe_draft_and_restart() {
    let dir = tempfile::tempdir().unwrap();
    let mut db = Database::open(dir.path()).unwrap();
    db.install_bundled_catalog().unwrap();
    let sugar = stable_id("ingredient", "sugar-granulated");
    let builtin = db.ingredient_detail(&sugar, "catalog").unwrap();
    assert_eq!(builtin.item.name, "Toz şeker");
    assert_eq!(builtin.item.recipe_id.as_deref(), Some(sugar.as_str()));
    assert_eq!(builtin.sources.len(), 2);
    assert!(builtin.sources.iter().all(|s| s.license == "CC0-1.0"));
    assert!(builtin.metadata.is_empty());
    assert_eq!(builtin.dimensions, vec!["mass"]);
    let own = db.create_ingredient("Iğdır karışımım").unwrap();
    let input = RecipeInput {
        id: uuid::Uuid::new_v4().to_string(),
        expected_revision: None,
        title: "Kütüphane tarifi".into(),
        description: None,
        kind: "food".into(),
        servings: "1".into(),
        prep_minutes: None,
        cook_minutes: None,
        notes: None,
        ingredients: vec![RecipeIngredient {
            id: uuid::Uuid::new_v4().to_string(),
            ingredient_id: own.id.clone(),
            quantity: Some("35.000001".into()),
            unit_code: "cc".into(),
            note: None,
        }],
        steps: vec![],
    };
    let saved = db.save_recipe(input.clone()).unwrap();
    let draft = db
        .save_draft(DraftWrite {
            id: uuid::Uuid::new_v4().to_string(),
            expected_revision: None,
            input,
        })
        .unwrap();
    db.edit_ingredient(IngredientEdit {
        id: own.id.clone(),
        revision: 1,
        name: "İçli özel karışımım".into(),
        notes: Some("Aile notu".into()),
        preferred_unit: Some("cc".into()),
        category_ids: None,
    })
    .unwrap();
    assert_eq!(
        db.delete_ingredient(&own.id, 2).unwrap_err().code,
        "INGREDIENT_REFERENCED"
    );
    assert!(db
        .edit_ingredient(IngredientEdit {
            id: sugar,
            revision: 1,
            name: "Dokunma".into(),
            notes: None,
            preferred_unit: None,
            category_ids: None,
        })
        .is_err());
    drop(db);
    let mut db = Database::open(dir.path()).unwrap();
    db.install_bundled_catalog().unwrap();
    let found = db
        .ingredient_library(query("İÇLİ", "personal", None, 0))
        .unwrap();
    assert_eq!(found.items[0].id, own.id);
    let detail = db.ingredient_detail(&own.id, "personal").unwrap();
    assert_eq!(detail.item.notes.as_deref(), Some("Aile notu"));
    assert!(detail.sources.is_empty());
    assert_eq!(
        db.recipe(&saved.id).unwrap().ingredients[0]
            .quantity
            .as_deref(),
        Some("35.000001")
    );
    assert_eq!(
        db.draft(&draft.id).unwrap().input.ingredients[0].ingredient_id,
        own.id
    );
}
#[test]
fn library_keeps_pending_catalog_definitions_visible_and_preserves_existing_overrides() {
    let dir = tempfile::tempdir().unwrap();
    let mut db = Database::open(dir.path()).unwrap();
    let own = db.create_ingredient("Toz şeker").unwrap();
    db.install_bundled_catalog().unwrap();
    let p = db.ingredient_library(query("", "all", None, 0)).unwrap();
    assert_eq!((p.total, p.catalog_count, p.personal_count), (796, 795, 1));
    let sugar = db
        .ingredient_detail(&stable_id("ingredient", "sugar-granulated"), "catalog")
        .unwrap();
    assert!(sugar.item.recipe_id.is_none());
    assert_eq!(
        db.ingredient_detail(&own.id, "personal")
            .unwrap()
            .item
            .recipe_id,
        Some(own.id)
    );
    let honey = stable_id("ingredient", "honey");
    db.customize_catalog_ingredient(IngredientEdit {
        id: honey.clone(),
        revision: 1,
        name: "Aile balı".into(),
        notes: None,
        preferred_unit: Some("kg".into()),
        category_ids: None,
    })
    .unwrap();
    let d = db.ingredient_detail(&honey, "catalog").unwrap();
    assert_eq!(d.item.name, "Aile balı");
    assert_eq!(d.canonical_tr.as_deref(), Some("Bal"));
    assert_eq!(
        db.ingredient_library(query("Honey", "catalog", None, 0))
            .unwrap()
            .items[0]
            .id,
        honey
    );
}

#[test]
fn favorites_recent_overrides_and_aliases_survive_restart_without_mutating_catalog() {
    let dir = tempfile::tempdir().unwrap();
    let mut db = Database::open(dir.path()).unwrap();
    db.install_bundled_catalog().unwrap();
    let sugar = stable_id("ingredient", "sugar-granulated");
    let own = db.create_ingredient("Aile şerbeti").unwrap();
    assert!(db.set_ingredient_favorite(&sugar, "catalog", true).unwrap());
    assert!(db
        .set_ingredient_favorite(&own.id, "personal", true)
        .unwrap());
    assert_eq!(
        db.ingredient_library(LibraryQuery {
            view: Some("favorites".into()),
            uncategorized: false,
            ..query("", "all", None, 0)
        })
        .unwrap()
        .total,
        2
    );
    let original = db.ingredient_detail(&sugar, "catalog").unwrap();
    let revision = original.item.revision.unwrap();
    let changed = db
        .set_catalog_presentation(
            crate::domain::ingredient_library::CatalogPresentationInput {
                id: sugar.clone(),
                revision,
                name: Some("Bizim şeker".into()),
                alias: Some("Ev tatlısı".into()),
                preferred_unit: Some("kg".into()),
                notes: Some("Rafın üstü".into()),
            },
        )
        .unwrap();
    assert_eq!(changed.canonical_tr.as_deref(), Some("Toz şeker"));
    assert_eq!(changed.item.name, "Bizim şeker");
    assert_eq!(
        db.ingredient_library(query("Ev tatlısı", "all", None, 0))
            .unwrap()
            .total,
        1
    );
    assert_eq!(
        db.available_ingredients("Ev tatlısı").unwrap().items[0].id,
        sugar
    );
    assert_eq!(
        db.ingredient_candidates("Şeker")
            .unwrap()
            .iter()
            .filter(|c| c.id == sugar)
            .count(),
        1
    );
    let rejected = db.set_catalog_presentation(
        crate::domain::ingredient_library::CatalogPresentationInput {
            id: sugar.clone(),
            revision: changed.item.revision.unwrap(),
            name: Some("Bizim şeker".into()),
            alias: Some("Honey".into()),
            preferred_unit: Some("kg".into()),
            notes: Some("Rafın üstü".into()),
        },
    );
    assert_eq!(rejected.unwrap_err().code, "INGREDIENT_DUPLICATE");
    assert_eq!(
        db.ingredient_detail(&sugar, "catalog")
            .unwrap()
            .personal_alias
            .as_deref(),
        Some("Ev tatlısı")
    );
    let mut recipe = RecipeInput {
        id: uuid::Uuid::new_v4().to_string(),
        expected_revision: None,
        title: "Tarif".into(),
        description: None,
        kind: "food".into(),
        servings: "1".into(),
        prep_minutes: None,
        cook_minutes: None,
        notes: None,
        ingredients: vec![RecipeIngredient {
            id: uuid::Uuid::new_v4().to_string(),
            ingredient_id: sugar.clone(),
            quantity: Some("35.000001".into()),
            unit_code: "g".into(),
            note: None,
        }],
        steps: vec![],
    };
    let saved = db.save_recipe(recipe.clone()).unwrap();
    recipe.expected_revision = Some(saved.revision);
    assert_eq!(
        db.ingredient_library(LibraryQuery {
            view: Some("recent".into()),
            uncategorized: false,
            ..query("", "all", None, 0)
        })
        .unwrap()
        .items[0]
            .id,
        sugar
    );
    drop(db);
    let mut db = Database::open(dir.path()).unwrap();
    assert_eq!(
        db.ingredient_library(LibraryQuery {
            view: Some("favorites".into()),
            uncategorized: false,
            ..query("", "all", None, 0)
        })
        .unwrap()
        .total,
        2
    );
    assert_eq!(
        db.recipe(&saved.id).unwrap().ingredients[0]
            .quantity
            .as_deref(),
        Some("35.000001")
    );
    let reverted = db
        .set_catalog_presentation(
            crate::domain::ingredient_library::CatalogPresentationInput {
                id: sugar.clone(),
                revision: changed.item.revision.unwrap(),
                name: None,
                alias: None,
                preferred_unit: None,
                notes: None,
            },
        )
        .unwrap();
    assert_eq!(reverted.item.name, reverted.canonical_tr.clone().unwrap());
    assert_eq!(reverted.personal_alias, None);
    assert_eq!(
        db.ingredient_library(query("Ev tatlısı", "all", None, 0))
            .unwrap()
            .total,
        0
    );
    assert_eq!(
        db.ingredient_library(LibraryQuery {
            view: Some("favorites".into()),
            uncategorized: false,
            ..query("", "all", None, 0)
        })
        .unwrap()
        .total,
        2
    );
    db.set_ingredient_favorite(&sugar, "catalog", false)
        .unwrap();
    assert_eq!(
        db.ingredient_library(LibraryQuery {
            view: Some("favorites".into()),
            uncategorized: false,
            ..query("", "all", None, 0)
        })
        .unwrap()
        .total,
        1
    );
}

#[test]
fn version_four_upgrade_and_relevance_keep_existing_recipe_and_distinct_suggestions() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("library.sqlite3");
    let mut conn = rusqlite::Connection::open(&path).unwrap();
    conn.pragma_update(None, "foreign_keys", true).unwrap();
    super::migrations::apply(&mut conn, &path, &super::migrations::MIGRATIONS[..4]).unwrap();
    let mut db = Database { conn };
    let ingredient = db.create_ingredient("Ev salçası").unwrap();
    let input = RecipeInput {
        id: uuid::Uuid::new_v4().to_string(),
        expected_revision: None,
        title: "Eski tarif".into(),
        description: None,
        kind: "food".into(),
        servings: "1".into(),
        prep_minutes: None,
        cook_minutes: None,
        notes: None,
        ingredients: vec![RecipeIngredient {
            id: uuid::Uuid::new_v4().to_string(),
            ingredient_id: ingredient.id.clone(),
            quantity: Some("0.123456".into()),
            unit_code: "g".into(),
            note: None,
        }],
        steps: vec![],
    };
    db.save_recipe(input.clone()).unwrap();
    drop(db);
    let mut db = Database::open(dir.path()).unwrap();
    assert_eq!(db.bootstrap().unwrap().storage.schema_version, 5);
    assert_eq!(
        db.recipe(&input.id).unwrap().ingredients[0]
            .quantity
            .as_deref(),
        Some("0.123456")
    );
    db.install_bundled_catalog().unwrap();
    let exact = db
        .create_ingredient("Granulated sugar syrup blend")
        .unwrap();
    db.set_ingredient_favorite(&exact.id, "personal", true)
        .unwrap();
    let result = db.available_ingredients("Granulated sugar").unwrap();
    assert_eq!(
        result.items[0].id,
        stable_id("ingredient", "sugar-granulated")
    );
    assert!(
        result
            .items
            .iter()
            .filter(|i| i.id == stable_id("ingredient", "sugar-granulated"))
            .count()
            == 1
    );
    let recent = db.available_ingredients_mode("", "recent").unwrap();
    assert_eq!(recent.items[0].id, ingredient.id);
    assert!(db
        .available_ingredients_mode("", "favorites")
        .unwrap()
        .items
        .iter()
        .any(|i| i.id == exact.id));
    assert_eq!(
        db.ingredient_candidates("granulated sugar")
            .unwrap()
            .iter()
            .filter(|c| c.id == stable_id("ingredient", "sugar-granulated"))
            .count(),
        1
    );
}
