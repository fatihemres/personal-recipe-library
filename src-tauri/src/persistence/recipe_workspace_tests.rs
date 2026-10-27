use super::*;
use crate::domain::{
    recipe_organization::{CollectionWrite, OrganizationWrite},
    recipe_search::RecipeSearchQuery,
    recipe_workspace::{PrivateNoteWrite, SavedFilterWrite},
    recipes::{RecipeIngredient, RecipeInput},
};
use rusqlite::Connection;

fn id() -> String {
    uuid::Uuid::new_v4().to_string()
}
fn recipe(ingredient_id: &str) -> RecipeInput {
    RecipeInput {
        id: id(),
        expected_revision: None,
        title: "İçli köfte".into(),
        description: None,
        kind: "food".into(),
        servings: "1.250".into(),
        prep_minutes: None,
        cook_minutes: None,
        notes: Some("Tarihî not".into()),
        ingredients: vec![RecipeIngredient {
            id: id(),
            ingredient_id: ingredient_id.into(),
            quantity: Some("1.250".into()),
            unit_code: "g".into(),
            note: None,
        }],
        steps: vec![],
    }
}
fn query() -> RecipeSearchQuery {
    RecipeSearchQuery {
        text: "İçli".into(),
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

#[test]
fn private_notes_are_independent_versioned_and_follow_lifecycle() {
    let dir = tempfile::tempdir().unwrap();
    let mut db = Database::open(dir.path()).unwrap();
    let ingredient = db.create_ingredient("Şeker").unwrap();
    let saved = db.save_recipe(recipe(&ingredient.id)).unwrap();
    assert_eq!(db.private_note(&saved.id).unwrap().revision, 0);
    let note = db
        .save_private_note(PrivateNoteWrite {
            recipe_id: saved.id.clone(),
            expected_revision: 0,
            markdown: "# Gizli\n<script>alert(1)</script>".into(),
        })
        .unwrap();
    assert_eq!(note.revision, 1);
    assert_eq!(
        db.save_private_note(PrivateNoteWrite {
            recipe_id: saved.id.clone(),
            expected_revision: 0,
            markdown: "Kaybolmasın".into()
        })
        .unwrap_err()
        .code,
        "CONFLICT"
    );
    assert_eq!(
        db.recipe(&saved.id).unwrap().notes.as_deref(),
        Some("Tarihî not")
    );
    let copy = db.duplicate_recipe(&saved.id, 1).unwrap();
    assert_eq!(db.private_note(&copy.id).unwrap().markdown, note.markdown);
    db.save_private_note(PrivateNoteWrite {
        recipe_id: copy.id.clone(),
        expected_revision: 1,
        markdown: "Bağımsız".into(),
    })
    .unwrap();
    assert_eq!(db.private_note(&saved.id).unwrap().markdown, note.markdown);
    let archived = db.archive_recipe(&saved.id, 1, true).unwrap();
    assert_eq!(db.private_note(&saved.id).unwrap().revision, 1);
    let trashed = db.set_deleted(&saved.id, archived.revision, true).unwrap();
    assert_eq!(db.private_note(&saved.id).unwrap().revision, 1);
    assert_eq!(
        db.save_private_note(PrivateNoteWrite {
            recipe_id: saved.id.clone(),
            expected_revision: 1,
            markdown: "Hayır".into()
        })
        .unwrap_err()
        .code,
        "NOT_FOUND"
    );
    db.purge_recipe(&saved.id, trashed.revision).unwrap();
    let count: i64 = db
        .conn
        .query_row(
            "SELECT count(*) FROM recipe_private_notes WHERE recipe_id=?1",
            [&saved.id],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(count, 0);
    drop(db);
    let db = Database::open(dir.path()).unwrap();
    assert_eq!(db.private_note(&copy.id).unwrap().markdown, "Bağımsız");
    assert_eq!(
        db.recipe(&copy.id).unwrap().ingredients[0]
            .quantity
            .as_deref(),
        Some("1.250")
    );
}

#[test]
fn filters_are_versioned_live_and_never_broaden_missing_references() {
    let dir = tempfile::tempdir().unwrap();
    let mut db = Database::open(dir.path()).unwrap();
    let ingredient = db.create_ingredient("İrmik").unwrap();
    let saved = db.save_recipe(recipe(&ingredient.id)).unwrap();
    let group = db
        .save_collection(CollectionWrite {
            id: id(),
            expected_revision: None,
            title: "İçli".into(),
        })
        .unwrap();
    db.set_collection_membership(&saved.id, &group.id, 1, true)
        .unwrap();
    let mut q = query();
    q.ingredient_id = Some(ingredient.id.clone());
    q.collection_id = Some(group.id.clone());
    let filter = db
        .save_filter(SavedFilterWrite {
            id: id(),
            expected_revision: None,
            title: "İşaretli".into(),
            query: q.clone(),
        })
        .unwrap();
    assert!(filter.available);
    assert_eq!(
        db.save_filter(SavedFilterWrite {
            id: id(),
            expected_revision: None,
            title: "işaretli".into(),
            query: q.clone()
        })
        .unwrap_err()
        .code,
        "FILTER_DUPLICATE"
    );
    assert_eq!(db.search_recipes(q.clone()).unwrap().total, 1);
    db.delete_collection(&group.id, group.revision).unwrap();
    assert!(!db.saved_filters().unwrap()[0].available);
    assert_eq!(
        db.resolve_filter(&filter.id).unwrap_err().code,
        "FILTER_REFERENCE_MISSING"
    );
    assert_eq!(db.search_recipes(q.clone()).unwrap().total, 0);
    db.conn
        .execute(
            "UPDATE saved_recipe_filters SET format_version=99 WHERE id=?1",
            [&filter.id],
        )
        .unwrap();
    let unsupported = &db.saved_filters().unwrap()[0];
    assert_eq!(unsupported.format_version, 99);
    assert!(unsupported.query.is_none() && !unsupported.available);
    assert_eq!(
        db.resolve_filter(&filter.id).unwrap_err().code,
        "FILTER_VERSION_UNSUPPORTED"
    );
    assert_eq!(
        db.save_filter(SavedFilterWrite {
            id: filter.id.clone(),
            expected_revision: Some(filter.revision),
            title: "Yeni".into(),
            query: q
        })
        .unwrap_err()
        .code,
        "CONFLICT"
    );
    db.delete_filter(&filter.id, filter.revision).unwrap();
    assert!(db.saved_filters().unwrap().is_empty());
}

#[test]
fn dashboard_matches_active_search_and_excludes_archive_and_trash() {
    let dir = tempfile::tempdir().unwrap();
    let mut db = Database::open(dir.path()).unwrap();
    assert_eq!(db.dashboard().unwrap().active_total, 0);
    let ing = db.create_ingredient("İrmik").unwrap();
    let first = db.save_recipe(recipe(&ing.id)).unwrap();
    db.set_recipe_organization(OrganizationWrite {
        recipe_id: first.id.clone(),
        expected_revision: 1,
        favorite: true,
        to_try: false,
        rating: None,
    })
    .unwrap();
    let second = db.save_recipe(recipe(&ing.id)).unwrap();
    db.archive_recipe(&second.id, 1, true).unwrap();
    let third = db.save_recipe(recipe(&ing.id)).unwrap();
    db.set_deleted(&third.id, 1, true).unwrap();
    let summary = db.dashboard().unwrap();
    assert_eq!(
        (
            summary.active_total,
            summary.food_count,
            summary.beverage_count,
            summary.favorite_count
        ),
        (1, 1, 0, 1)
    );
    assert_eq!(summary.recent_created[0].id, first.id);
    assert_eq!(db.search_recipes(query()).unwrap().total, 1);
}

#[test]
fn schema_seven_upgrade_preserves_recipe_and_organization() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("library.sqlite3");
    std::fs::File::create(&path).unwrap();
    let mut conn = Connection::open(&path).unwrap();
    conn.pragma_update(None, "foreign_keys", true).unwrap();
    migrations::apply(&mut conn, &path, &migrations::MIGRATIONS[..7]).unwrap();
    drop(conn);
    let conn = Connection::open(&path).unwrap();
    conn.pragma_update(None, "foreign_keys", true).unwrap();
    let recipe_id = id();
    let ingredient_id = id();
    let collection_id = id();
    conn.execute(
        "INSERT INTO ingredients(id,name,search_name) VALUES(?1,'İrmik','irmik')",
        [&ingredient_id],
    )
    .unwrap();
    conn.execute("INSERT INTO recipes(id,title,kind,servings,notes) VALUES(?1,'İçli köfte','food','1.250','Tarihî not')",[&recipe_id]).unwrap();
    conn.execute("INSERT INTO recipe_ingredients(id,recipe_id,ingredient_id,quantity,unit_code,position) VALUES(?1,?2,?3,'1.250','g',0)",rusqlite::params![id(),recipe_id,ingredient_id]).unwrap();
    conn.execute(
        "UPDATE recipe_organization SET favorite=1,to_try=1,rating=5 WHERE recipe_id=?1",
        [&recipe_id],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO recipe_collections(id,title,title_key) VALUES(?1,'Özel','özel')",
        [&collection_id],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO recipe_collection_members(collection_id,recipe_id) VALUES(?1,?2)",
        rusqlite::params![collection_id, recipe_id],
    )
    .unwrap();
    conn.execute("UPDATE recipe_search_state SET ready=0 WHERE id=1", [])
        .unwrap();
    drop(conn);
    let db = Database::open(dir.path()).unwrap();
    assert_eq!(db.bootstrap().unwrap().storage.schema_version, 9);
    assert_eq!(
        db.recipe(&recipe_id).unwrap().ingredients[0]
            .quantity
            .as_deref(),
        Some("1.250")
    );
    assert_eq!(db.recipe_organization(&recipe_id).unwrap().rating, Some(5));
    assert_eq!(db.collections().unwrap()[0].active_count, 1);
    assert_eq!(db.search_recipes(query()).unwrap().total, 1);
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
    drop(db);
    let db = Database::open(dir.path()).unwrap();
    assert_eq!(db.bootstrap().unwrap().storage.schema_version, 9);
    assert_eq!(
        db.recipe(&recipe_id).unwrap().notes.as_deref(),
        Some("Tarihî not")
    );
}
