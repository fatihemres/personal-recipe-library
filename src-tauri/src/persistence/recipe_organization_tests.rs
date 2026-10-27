use super::*;
use crate::domain::{
    recipe_organization::{CollectionWrite, OrganizationWrite},
    recipe_search::RecipeSearchQuery,
    recipes::{RecipeIngredient, RecipeInput, RecipeStep},
};
use rusqlite::{params, Connection};

fn id() -> String {
    uuid::Uuid::new_v4().to_string()
}
fn input(ingredient_id: &str) -> RecipeInput {
    RecipeInput {
        id: id(),
        expected_revision: None,
        title: "İçli köfte".into(),
        description: Some("Geleneksel".into()),
        kind: "food".into(),
        servings: "1.250".into(),
        prep_minutes: Some(20),
        cook_minutes: None,
        notes: Some("Kişisel not".into()),
        ingredients: vec![RecipeIngredient {
            id: id(),
            ingredient_id: ingredient_id.into(),
            quantity: Some("1.250".into()),
            unit_code: "g".into(),
            note: None,
        }],
        steps: vec![RecipeStep {
            id: id(),
            instructions: "İyice yoğur".into(),
            details: None,
            ingredient_line_ids: vec![],
        }],
    }
}
fn query(text: &str) -> RecipeSearchQuery {
    RecipeSearchQuery {
        text: text.into(),
        scope: "active".into(),
        kind: None,
        ingredient_id: None,
        favorite_only: false,
        to_try_only: false,
        collection_id: None,
        created_from: None,
        created_to: None,
        modified_from: None,
        modified_to: None,
        sort: "relevance".into(),
        offset: 0,
        limit: 24,
    }
}
fn write(
    id: &str,
    revision: i64,
    favorite: bool,
    to_try: bool,
    rating: Option<i64>,
) -> OrganizationWrite {
    OrganizationWrite {
        recipe_id: id.into(),
        expected_revision: revision,
        favorite,
        to_try,
        rating,
    }
}

#[test]
fn organization_persists_independently_of_recipe_revision_and_lifecycle() {
    let dir = tempfile::tempdir().unwrap();
    let mut db = Database::open(dir.path()).unwrap();
    let ingredient = db.create_ingredient("İçme suyu").unwrap();
    let recipe = db.save_recipe(input(&ingredient.id)).unwrap();
    let initial = db.recipe_organization(&recipe.id).unwrap();
    assert_eq!(initial.revision, 1);
    assert!(!initial.favorite && !initial.to_try && initial.rating.is_none());
    let state = db
        .set_recipe_organization(write(&recipe.id, 1, true, true, Some(5)))
        .unwrap();
    assert_eq!(state.revision, 2);
    assert_eq!(
        db.set_recipe_organization(write(&recipe.id, 2, true, true, Some(5)))
            .unwrap()
            .revision,
        2
    );
    assert_eq!(
        db.set_recipe_organization(write(&recipe.id, 1, false, false, None))
            .unwrap_err()
            .code,
        "CONFLICT"
    );
    assert_eq!(
        db.set_recipe_organization(write(&recipe.id, 2, true, true, Some(0)))
            .unwrap_err()
            .code,
        "INVALID_INPUT"
    );
    assert_eq!(
        db.set_recipe_organization(write(&recipe.id, 2, true, true, Some(6)))
            .unwrap_err()
            .code,
        "INVALID_INPUT"
    );
    let group = db
        .save_collection(CollectionWrite {
            id: id(),
            expected_revision: None,
            title: "İstanbul".into(),
        })
        .unwrap();
    let second = db
        .save_collection(CollectionWrite {
            id: id(),
            expected_revision: None,
            title: "Denenecekler".into(),
        })
        .unwrap();
    let state = db
        .set_collection_membership(&recipe.id, &group.id, 2, true)
        .unwrap();
    assert_eq!(state.revision, 3);
    assert_eq!(
        db.set_collection_membership(&recipe.id, &group.id, 3, true)
            .unwrap()
            .revision,
        3
    );
    let state = db
        .set_collection_membership(&recipe.id, &second.id, 3, true)
        .unwrap();
    assert_eq!(state.collection_ids.len(), 2);
    assert_eq!(
        db.set_collection_membership(&recipe.id, &second.id, 3, false)
            .unwrap_err()
            .code,
        "CONFLICT"
    );
    assert_eq!(db.recipe(&recipe.id).unwrap().revision, recipe.revision);
    let archived = db
        .archive_recipe(&recipe.id, recipe.revision, true)
        .unwrap();
    assert_eq!(
        db.collections()
            .unwrap()
            .iter()
            .find(|c| c.id == group.id)
            .unwrap()
            .active_count,
        0
    );
    assert!(db.recipe_organization(&recipe.id).unwrap().favorite);
    let trashed = db
        .set_deleted(&archived.id, archived.revision, true)
        .unwrap();
    assert_eq!(
        db.set_recipe_organization(write(&recipe.id, state.revision, false, false, None))
            .unwrap_err()
            .code,
        "NOT_FOUND"
    );
    db.set_deleted(&trashed.id, trashed.revision, false)
        .unwrap();
    drop(db);
    let mut db = Database::open(dir.path()).unwrap();
    let kept = db.recipe_organization(&recipe.id).unwrap();
    assert!(kept.favorite && kept.to_try && kept.rating == Some(5));
    assert_eq!(kept.collection_ids.len(), 2);
    assert_eq!(
        db.recipe(&recipe.id).unwrap().ingredients[0]
            .quantity
            .as_deref(),
        Some("1.250")
    );
    assert_eq!(db.recipe(&recipe.id).unwrap().ingredients[0].unit_code, "g");
    db.delete_collection(&group.id, group.revision).unwrap();
    assert_eq!(
        db.recipe_organization(&recipe.id)
            .unwrap()
            .collection_ids
            .len(),
        1
    );
    assert!(db.recipe(&recipe.id).is_ok());
    let restored = db.recipe(&recipe.id).unwrap();
    let deleted = db
        .set_deleted(&restored.id, restored.revision, true)
        .unwrap();
    db.purge_recipe(&recipe.id, deleted.revision).unwrap();
    assert!(db.recipe_organization(&recipe.id).is_err());
    assert_eq!(db.collections().unwrap().len(), 1);
    assert_eq!(db.ingredients("İçme").unwrap()[0].id, ingredient.id);
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
fn collections_normalize_turkish_names_and_failed_mutations_roll_back() {
    let dir = tempfile::tempdir().unwrap();
    let mut db = Database::open(dir.path()).unwrap();
    let group = db
        .save_collection(CollectionWrite {
            id: id(),
            expected_revision: None,
            title: "İçli Tarifler".into(),
        })
        .unwrap();
    assert_eq!(
        db.save_collection(CollectionWrite {
            id: id(),
            expected_revision: None,
            title: "içli   tarifler".into()
        })
        .unwrap_err()
        .code,
        "COLLECTION_DUPLICATE"
    );
    assert_eq!(
        db.save_collection(CollectionWrite {
            id: group.id.clone(),
            expected_revision: Some(99),
            title: "Başka".into()
        })
        .unwrap_err()
        .code,
        "CONFLICT"
    );
    assert_eq!(db.collections().unwrap()[0].title, "İçli Tarifler");
    let renamed = db
        .save_collection(CollectionWrite {
            id: group.id.clone(),
            expected_revision: Some(1),
            title: "Işık".into(),
        })
        .unwrap();
    assert_eq!(renamed.revision, 2);
    assert_eq!(
        db.save_collection(CollectionWrite {
            id: id(),
            expected_revision: None,
            title: "ışık".into()
        })
        .unwrap_err()
        .code,
        "COLLECTION_DUPLICATE"
    );
    assert_eq!(
        db.delete_collection(&group.id, 1).unwrap_err().code,
        "CONFLICT"
    );
    assert_eq!(db.collections().unwrap().len(), 1);
}

#[test]
fn failed_membership_insert_does_not_advance_organization_revision() {
    let dir = tempfile::tempdir().unwrap();
    let mut db = Database::open(dir.path()).unwrap();
    let ingredient = db.create_ingredient("Nane").unwrap();
    let recipe = db.save_recipe(input(&ingredient.id)).unwrap();
    let group = db
        .save_collection(CollectionWrite {
            id: id(),
            expected_revision: None,
            title: "Yemekler".into(),
        })
        .unwrap();
    db.conn.execute_batch("CREATE TRIGGER fail_membership BEFORE INSERT ON recipe_collection_members BEGIN SELECT RAISE(ABORT,'rollback proof'); END;").unwrap();
    assert!(db
        .set_collection_membership(&recipe.id, &group.id, 1, true)
        .is_err());
    assert_eq!(db.recipe_organization(&recipe.id).unwrap().revision, 1);
    assert!(db
        .recipe_organization(&recipe.id)
        .unwrap()
        .collection_ids
        .is_empty());
    db.conn
        .execute_batch("DROP TRIGGER fail_membership")
        .unwrap();
    assert_eq!(
        db.set_collection_membership(&recipe.id, &group.id, 1, true)
            .unwrap()
            .revision,
        2
    );
}

#[test]
fn search_filters_combine_with_fts_scope_sort_pagination_and_bound_ids() {
    let dir = tempfile::tempdir().unwrap();
    let mut db = Database::open(dir.path()).unwrap();
    let ingredient = db.create_ingredient("Şeker").unwrap();
    let a = db.save_recipe(input(&ingredient.id)).unwrap();
    let b = db.save_recipe(input(&ingredient.id)).unwrap();
    db.set_recipe_organization(write(&a.id, 1, true, true, Some(4)))
        .unwrap();
    let group = db
        .save_collection(CollectionWrite {
            id: id(),
            expected_revision: None,
            title: "Favori".into(),
        })
        .unwrap();
    db.set_collection_membership(&a.id, &group.id, 2, true)
        .unwrap();
    let mut q = query("şek");
    q.favorite_only = true;
    q.to_try_only = true;
    q.collection_id = Some(group.id.clone());
    q.kind = Some("food".into());
    q.ingredient_id = Some(ingredient.id.clone());
    q.created_from = Some("2020-01-01".into());
    q.modified_to = Some("2099-12-31".into());
    q.sort = "title_asc".into();
    q.limit = 1;
    assert_eq!(db.search_recipes(q.clone()).unwrap().items[0].id, a.id);
    assert_eq!(db.search_recipes(q.clone()).unwrap().total, 1);
    db.rebuild_recipe_search().unwrap();
    assert_eq!(db.search_recipes(q.clone()).unwrap().total, 1);
    q.offset = 1;
    assert_eq!(db.search_recipes(q.clone()).unwrap().items.len(), 0);
    q.offset = 0;
    q.text = "' OR 1=1 --".into();
    assert_eq!(db.search_recipes(q.clone()).unwrap().total, 0);
    q.text = "şek".into();
    q.collection_id = Some("bad' OR 1=1".into());
    assert_eq!(
        db.search_recipes(q.clone()).unwrap_err().code,
        "INVALID_INPUT"
    );
    q.collection_id = Some(group.id);
    q.scope = "trash".into();
    assert_eq!(db.search_recipes(q).unwrap_err().code, "INVALID_INPUT");
    let deleted = db.set_deleted(&a.id, a.revision, true).unwrap();
    let mut q = query("şek");
    q.favorite_only = true;
    assert_eq!(db.search_recipes(q.clone()).unwrap().total, 0);
    db.set_deleted(&a.id, deleted.revision, false).unwrap();
    assert_eq!(db.search_recipes(q).unwrap().total, 1);
    assert_eq!(
        db.recipe(&b.id).unwrap().ingredients[0].quantity.as_deref(),
        Some("1.250")
    );
}

#[test]
fn migration_six_to_seven_preserves_rows_and_checksum_on_restart() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("library.sqlite3");
    let mut conn = Connection::open(&path).unwrap();
    conn.pragma_update(None, "foreign_keys", true).unwrap();
    migrations::apply(&mut conn, &path, &migrations::MIGRATIONS[..6]).unwrap();
    let mut prior = Database { conn };
    prior.install_bundled_catalog().unwrap();
    assert_eq!(
        prior
            .conn
            .query_row("SELECT count(*) FROM catalog_ingredients", [], |r| r
                .get::<_, i64>(0))
            .unwrap(),
        854
    );
    let conn = prior.conn;
    let recipe_id = id();
    let ingredient_id = id();
    let line_id = id();
    conn.execute(
        "INSERT INTO ingredients(id,name,search_name) VALUES(?1,'Şeker','şeker')",
        [&ingredient_id],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO recipes(id,title,kind,servings) VALUES(?1,'İçli köfte','food','1.250')",
        [&recipe_id],
    )
    .unwrap();
    conn.execute("INSERT INTO recipe_ingredients(id,recipe_id,ingredient_id,quantity,unit_code,position) VALUES(?1,?2,?3,'1.250','g',0)",params![line_id,recipe_id,ingredient_id]).unwrap();
    let mut draft = input(&ingredient_id);
    draft.id = recipe_id.clone();
    draft.expected_revision = Some(1);
    let payload = serde_json::to_string(&draft).unwrap();
    conn.execute("INSERT INTO editor_drafts(id,recipe_id,revision,state,payload) VALUES(?1,?2,1,'active',?3)",params![id(),recipe_id,payload]).unwrap();
    drop(conn);
    let db = Database::open(dir.path()).unwrap();
    assert_eq!(db.bootstrap().unwrap().storage.schema_version, 9);
    assert_eq!(db.recipe_organization(&recipe_id).unwrap().revision, 1);
    assert_eq!(
        db.recipe(&recipe_id).unwrap().ingredients[0]
            .quantity
            .as_deref(),
        Some("1.250")
    );
    assert_eq!(
        db.conn
            .query_row("SELECT count(*) FROM editor_drafts", [], |r| r
                .get::<_, i64>(0))
            .unwrap(),
        1
    );
    assert_eq!(db.drafts().unwrap().len(), 1);
    assert_eq!(
        db.conn
            .query_row("SELECT count(*) FROM catalog_ingredients", [], |r| r
                .get::<_, i64>(0))
            .unwrap(),
        854
    );
    assert_eq!(
        db.conn
            .query_row("PRAGMA integrity_check", [], |r| r.get::<_, String>(0))
            .unwrap(),
        "ok"
    );
    drop(db);
    let db = Database::open(dir.path()).unwrap();
    assert_eq!(db.recipe_organization(&recipe_id).unwrap().revision, 1);
    assert_eq!(
        migrations::validate(&db.conn, migrations::MIGRATIONS).unwrap(),
        9
    );
}
