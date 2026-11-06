use super::*;
use crate::domain::{
    recipe_organization::*, recipe_relations::*, recipe_versions::*, recipe_workspace::*,
    recipes::*, reliability::*,
};
use rusqlite::{params, Connection};
fn id() -> String {
    uuid::Uuid::new_v4().to_string()
}
fn input(ingredient: &str) -> RecipeInput {
    let line = id();
    serde_json::from_value(serde_json::json!({"id":id(),"expectedRevision":null,"title":"M5B3 — İ/ı Gül suyu","description":null,"kind":"beverage","servings":"1.250","prepMinutes":null,"cookMinutes":0,"notes":"İçerik notu","ingredients":[{"id":line,"ingredientId":ingredient,"quantity":"1.250","unitCode":"mL","note":null}],"steps":[{"id":id(),"instructions":"Karıştırın","details":{"formatVersion":1,"title":"Hazırlık","durationMinutes":"1","temperatureValue":null,"temperatureUnit":null,"heatLevel":null,"equipment":"Bardak","technique":null,"tips":null},"ingredientLineIds":[line]}]})).unwrap()
}
fn setup() -> (tempfile::TempDir, Database, Recipe) {
    let dir = tempfile::tempdir().unwrap();
    let mut db = Database::open(dir.path()).unwrap();
    let ing = db.create_ingredient("M5B3 kişisel su").unwrap();
    let r = db.save_recipe(input(&ing.id)).unwrap();
    (dir, db, r)
}
fn related(db: &mut Database, a: &Recipe, b: &Recipe, linked: bool) {
    let rev = db.recipe_links(&a.id, 0, 20).unwrap().graph_revision;
    db.set_related_recipe(RelatedWrite {
        recipe_id: a.id.clone(),
        target_id: b.id.clone(),
        expected_graph_revision: rev,
        linked,
    })
    .unwrap();
}
fn parent(db: &mut Database, child: &Recipe, p: &Recipe) -> Result<(), crate::domain::AppError> {
    let rev = db.recipe_links(&child.id, 0, 20)?.graph_revision;
    db.set_variation_parent(ParentWrite {
        child_id: child.id.clone(),
        parent_id: Some(p.id.clone()),
        expected_graph_revision: rev,
    })
}
fn count(db: &Database, id: &str) -> usize {
    db.recipe_versions(id, None, 100).unwrap().len()
}
fn purge_write(db: &Database, r: &Recipe, ack: bool) -> ReviewedPurge {
    let p = db.preview_recipe_purge(&r.id, r.revision, 0, 20).unwrap();
    ReviewedPurge {
        recipe_id: r.id.clone(),
        expected_revision: r.revision,
        token: p.token,
        acknowledge_detach: ack,
    }
}
fn health(db: &Database) {
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
fn variations_copy_only_content_with_remapped_ids_and_independent_versions() {
    let (_dir, mut db, a) = setup();
    db.save_private_note(PrivateNoteWrite {
        recipe_id: a.id.clone(),
        expected_revision: 0,
        markdown: "Özel kaynak notu".into(),
    })
    .unwrap();
    db.set_recipe_organization(OrganizationWrite {
        recipe_id: a.id.clone(),
        expected_revision: 1,
        favorite: true,
        to_try: true,
        rating: Some(5),
    })
    .unwrap();
    let group = db
        .save_collection(CollectionWrite {
            id: id(),
            expected_revision: None,
            title: "Kişisel".into(),
        })
        .unwrap();
    db.set_collection_membership(&a.id, &group.id, 2, true)
        .unwrap();
    let b = db.create_variation(&a.id, a.revision).unwrap();
    assert_ne!(a.id, b.id);
    assert_ne!(a.ingredients[0].id, b.ingredients[0].id);
    assert_ne!(a.steps[0].id, b.steps[0].id);
    assert_eq!(
        b.steps[0].ingredient_line_ids,
        vec![b.ingredients[0].id.clone()]
    );
    assert_eq!(
        a.ingredients[0].ingredient_id,
        b.ingredients[0].ingredient_id
    );
    assert_eq!(b.ingredients[0].quantity.as_deref(), Some("1.250"));
    assert_eq!(
        b.steps[0]
            .details
            .as_ref()
            .unwrap()
            .duration_minutes
            .as_deref(),
        Some("1")
    );
    assert_eq!(b.notes, a.notes);
    assert_eq!(count(&db, &b.id), 1);
    assert_eq!(
        db.recipe_version(&b.id, 1).unwrap().summary.provenance,
        "created"
    );
    assert!(db.private_note(&b.id).unwrap().markdown.is_empty());
    let org = db.recipe_organization(&b.id).unwrap();
    assert!(!org.favorite);
    assert!(!org.to_try);
    assert!(org.rating.is_none());
    assert!(org.collection_ids.is_empty());
    let mut edit = b.to_input();
    edit.title = "Varyasyon düzenlendi".into();
    let b2 = db.save_recipe(edit).unwrap();
    assert_eq!(db.recipe(&a.id).unwrap().title, a.title);
    assert_eq!(count(&db, &a.id), 1);
    let restored = db
        .restore_recipe_version(RestoreVersion {
            recipe_id: b.id.clone(),
            version_number: 1,
            expected_revision: b2.revision,
        })
        .unwrap();
    assert_eq!(restored.version_number, 3);
    assert_eq!(db.recipe_links(&b.id, 0, 20).unwrap().items[0].id, a.id);
    let saved = db.save_recipe(restored.recipe.to_input()).unwrap();
    assert_eq!(count(&db, &saved.id), 3);
    let ordinary = db.duplicate_recipe(&a.id, a.revision).unwrap();
    assert_eq!(
        db.private_note(&ordinary.id).unwrap().markdown,
        "Özel kaynak notu"
    );
    assert_eq!(db.recipe_links(&ordinary.id, 0, 20).unwrap().total, 0);
    health(&db);
}
#[test]
fn single_parent_chains_cycle_guards_and_symmetric_relations_are_external_metadata() {
    let (_dir, mut db, a) = setup();
    let b = db.create_variation(&a.id, 1).unwrap();
    let c = db.create_variation(&b.id, 1).unwrap();
    assert_eq!(parent(&mut db, &a, &c).unwrap_err().code, "VARIATION_CYCLE");
    assert!(parent(&mut db, &a, &a).is_err());
    assert!(db
        .conn
        .execute(
            "INSERT INTO recipe_variations(child_id,parent_id) VALUES(?1,?2)",
            params![b.id, c.id]
        )
        .is_err());
    assert!(db
        .conn
        .execute(
            "UPDATE recipe_variations SET parent_id=?2 WHERE child_id=?1",
            params![b.id, c.id]
        )
        .is_err());
    assert!(db
        .conn
        .execute(
            "INSERT INTO recipe_variations(child_id,parent_id) VALUES(?1,?2)",
            params![a.id, id()]
        )
        .is_err());
    related(&mut db, &a, &c, true);
    let links = db.recipe_links(&c.id, 0, 20).unwrap();
    assert!(links
        .items
        .iter()
        .any(|r| r.id == a.id && r.relationship == "related"));
    let rev = links.graph_revision;
    assert_eq!(
        db.set_related_recipe(RelatedWrite {
            recipe_id: c.id.clone(),
            target_id: a.id.clone(),
            expected_graph_revision: rev,
            linked: true
        })
        .unwrap_err()
        .code,
        "RELATION_EXISTS"
    );
    related(&mut db, &c, &a, false);
    assert!(!db
        .recipe_links(&a.id, 0, 20)
        .unwrap()
        .items
        .iter()
        .any(|r| r.relationship == "related"));
    assert_eq!(count(&db, &a.id), 1);
    assert_eq!(count(&db, &c.id), 1);
    assert_eq!(db.recipe(&a.id).unwrap().revision, 1);
    health(&db);
}
#[test]
fn inactive_links_remain_visible_and_creation_is_rejected() {
    let (_dir, mut db, a) = setup();
    let b = db.create_variation(&a.id, 1).unwrap();
    related(&mut db, &a, &b, true);
    let a = db.archive_recipe(&a.id, 1, true).unwrap();
    assert_eq!(
        db.recipe_links(&b.id, 0, 20).unwrap().items[0].status,
        "archived"
    );
    assert!(db.create_variation(&a.id, a.revision).is_err());
    assert!(parent(&mut db, &b, &a).is_err());
    let c = db
        .save_recipe(input(&a.ingredients[0].ingredient_id))
        .unwrap();
    let rev = db.recipe_links(&c.id, 0, 20).unwrap().graph_revision;
    assert!(db
        .set_related_recipe(RelatedWrite {
            recipe_id: c.id,
            target_id: a.id.clone(),
            expected_graph_revision: rev,
            linked: true
        })
        .is_err());
    let a = db.archive_recipe(&a.id, a.revision, false).unwrap();
    let a = db.set_deleted(&a.id, a.revision, true).unwrap();
    assert!(db
        .recipe_links(&b.id, 0, 20)
        .unwrap()
        .items
        .iter()
        .all(|r| r.status == "trash"));
    assert_eq!(
        db.purge_recipe(&a.id, a.revision).unwrap_err().code,
        "RECIPE_LINKS_BLOCK_PURGE"
    );
    assert_eq!(count(&db, &a.id), 1);
    health(&db);
}
#[test]
fn reviewed_purge_detaches_only_target_and_keeps_peer_drafts_history_notes_and_ingredients() {
    let (_dir, mut db, a) = setup();
    let b = db.create_variation(&a.id, 1).unwrap();
    let peer = db
        .save_recipe(input(&a.ingredients[0].ingredient_id))
        .unwrap();
    related(&mut db, &a, &peer, true);
    db.save_private_note(PrivateNoteWrite {
        recipe_id: b.id.clone(),
        expected_revision: 0,
        markdown: "Alt tarif notu".into(),
    })
    .unwrap();
    let draft = db
        .save_draft(DraftWrite {
            id: id(),
            expected_revision: None,
            input: a.to_input(),
        })
        .unwrap();
    let a = db.set_deleted(&a.id, 1, true).unwrap();
    assert!(db
        .conn
        .execute("DELETE FROM recipes WHERE id=?1", [&a.id])
        .is_err());
    let preview = db.preview_recipe_purge(&a.id, a.revision, 0, 1).unwrap();
    assert_eq!(preview.links.total, 2);
    assert!(preview.links.has_more);
    let next = db.preview_recipe_purge(&a.id, a.revision, 1, 1).unwrap();
    assert_eq!(next.token, preview.token);
    assert!(!next.links.has_more);
    let w = purge_write(&db, &a, false);
    assert_eq!(
        db.purge_reviewed_recipe(w).unwrap_err().code,
        "RECIPE_LINKS_BLOCK_PURGE"
    );
    let w = purge_write(&db, &a, true);
    db.purge_reviewed_recipe(w).unwrap();
    assert!(db.recipe(&a.id).is_err());
    assert_eq!(db.recipe(&b.id).unwrap().revision, 1);
    assert_eq!(count(&db, &b.id), 1);
    assert_eq!(count(&db, &peer.id), 1);
    assert_eq!(db.private_note(&b.id).unwrap().markdown, "Alt tarif notu");
    assert_eq!(db.recipe_links(&b.id, 0, 20).unwrap().total, 0);
    assert_eq!(db.recipe_links(&peer.id, 0, 20).unwrap().total, 0);
    assert!(db.draft(&draft.id).unwrap().recipe_id.is_none());
    assert!(!db.ingredients("M5B3").unwrap().is_empty());
    health(&db);
}
#[test]
fn purge_tokens_detect_aba_peer_edits_and_target_revision_changes() {
    let (_dir, mut db, a) = setup();
    let b = db
        .save_recipe(input(&a.ingredients[0].ingredient_id))
        .unwrap();
    related(&mut db, &a, &b, true);
    db.set_deleted(&a.id, 1, true).unwrap();
    let preview = db.preview_recipe_purge(&a.id, 2, 0, 20).unwrap();
    let a = db.recipe(&a.id).unwrap();
    // Removing and re-adding a link while active is an ABA reference change.
    let active = db.set_deleted(&a.id, a.revision, false).unwrap();
    related(&mut db, &active, &b, false);
    related(&mut db, &active, &b, true);
    let trash = db.set_deleted(&a.id, active.revision, true).unwrap();
    let stale = ReviewedPurge {
        recipe_id: a.id.clone(),
        expected_revision: trash.revision,
        token: preview.token,
        acknowledge_detach: true,
    };
    assert_eq!(
        db.purge_reviewed_recipe(stale).unwrap_err().code,
        "PURGE_REVIEW_STALE"
    );
    let w = purge_write(&db, &trash, true);
    let mut edit = b.to_input();
    edit.title = "İlgili tarif değişti".into();
    db.save_recipe(edit).unwrap();
    assert_eq!(
        db.purge_reviewed_recipe(w).unwrap_err().code,
        "PURGE_REVIEW_STALE"
    );
    let mut w = purge_write(&db, &trash, true);
    w.expected_revision -= 1;
    assert_eq!(db.purge_reviewed_recipe(w).unwrap_err().code, "CONFLICT");
    assert!(db.recipe(&a.id).is_ok());
    health(&db);
}
#[test]
fn injected_variation_and_purge_failures_roll_back_every_owned_row_and_graph_revision() {
    let (_dir, mut db, a) = setup();
    db.conn.execute_batch("CREATE TRIGGER fail_variation BEFORE INSERT ON recipe_variations BEGIN SELECT RAISE(ABORT,'injected'); END;").unwrap();
    assert!(db.create_variation(&a.id, 1).is_err());
    assert_eq!(
        db.conn
            .query_row("SELECT count(*) FROM recipes", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        1
    );
    assert_eq!(count(&db, &a.id), 1);
    db.conn
        .execute_batch("DROP TRIGGER fail_variation")
        .unwrap();
    let b = db.create_variation(&a.id, 1).unwrap();
    let a = db.set_deleted(&a.id, 1, true).unwrap();
    let w = purge_write(&db, &a, true);
    let token = w.token.clone();
    db.conn.execute_batch("CREATE TRIGGER fail_purge BEFORE DELETE ON recipes BEGIN SELECT RAISE(ABORT,'injected'); END;").unwrap();
    assert!(db.purge_reviewed_recipe(w).is_err());
    assert_eq!(purge_write(&db, &a, true).token, token);
    assert_eq!(db.recipe_links(&b.id, 0, 20).unwrap().total, 1);
    assert_eq!(count(&db, &a.id), 1);
    health(&db);
}
#[test]
fn schema_ten_upgrade_preserves_original_data_and_rolls_back_failed_eleven() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("library.sqlite3");
    std::fs::File::create(&path).unwrap();
    let mut conn = Connection::open(&path).unwrap();
    conn.pragma_update(None, "foreign_keys", true).unwrap();
    migrations::apply(&mut conn, &path, &migrations::MIGRATIONS[..10]).unwrap();
    let mut db = Database { conn };
    db.install_bundled_catalog().unwrap();
    let ing = db.create_ingredient("M5B3 migration ingredient").unwrap();
    let a = db.save_recipe(input(&ing.id)).unwrap();
    let payload: String = db
        .conn
        .query_row(
            "SELECT snapshot FROM recipe_versions WHERE recipe_id=?1",
            [&a.id],
            |r| r.get(0),
        )
        .unwrap();
    db.save_private_note(PrivateNoteWrite {
        recipe_id: a.id.clone(),
        expected_revision: 0,
        markdown: "Eski özel not".into(),
    })
    .unwrap();
    db.set_recipe_organization(OrganizationWrite {
        recipe_id: a.id.clone(),
        expected_revision: 1,
        favorite: true,
        to_try: true,
        rating: Some(3),
    })
    .unwrap();
    let draft = db
        .save_draft(DraftWrite {
            id: id(),
            expected_revision: None,
            input: a.to_input(),
        })
        .unwrap();
    drop(db);
    let mut conn = Connection::open(&path).unwrap();
    let mut bad: Vec<_> = migrations::MIGRATIONS[..10]
        .iter()
        .map(|m| migrations::Migration {
            version: m.version,
            name: m.name,
            sql: m.sql,
        })
        .collect();
    bad.push(migrations::Migration {
        version: 11,
        name: "recipe_relationships",
        sql: "CREATE TABLE partial_probe(id); INVALID SQL;",
    });
    assert!(migrations::apply(&mut conn, &path, &bad).is_err());
    assert_eq!(
        conn.query_row("pragma user_version", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        10
    );
    assert!(!conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE name='partial_probe')",
            [],
            |r| r.get::<_, bool>(0)
        )
        .unwrap());
    drop(conn);
    for _ in 0..2 {
        let db = Database::open(dir.path()).unwrap();
        assert_eq!(db.bootstrap().unwrap().storage.schema_version, 14);
        assert_eq!(
            db.recipe(&a.id).unwrap().ingredients[0].quantity.as_deref(),
            Some("1.250")
        );
        assert_eq!(db.recipe(&a.id).unwrap().revision, 1);
        assert_eq!(db.private_note(&a.id).unwrap().markdown, "Eski özel not");
        assert!(db.recipe_organization(&a.id).unwrap().favorite);
        assert_eq!(db.draft(&draft.id).unwrap().revision, draft.revision);
        assert_eq!(
            db.conn
                .query_row(
                    "SELECT snapshot FROM recipe_versions WHERE recipe_id=?1",
                    [&a.id],
                    |r| r.get::<_, String>(0)
                )
                .unwrap(),
            payload
        );
        assert_eq!(db.recipe_links(&a.id, 0, 20).unwrap().graph_revision, 1);
        assert_eq!(
            db.conn
                .query_row("SELECT count(*) FROM catalog_ingredients", [], |r| r
                    .get::<_, i64>(0))
                .unwrap(),
            854
        );
        health(&db);
    }
    let backups: Vec<_> = std::fs::read_dir(dir.path())
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| {
            p.file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("pre-migration-10-")
        })
        .collect();
    assert!(!backups.is_empty());
    for backup in backups {
        let c = Connection::open_with_flags(backup, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
            .unwrap();
        assert_eq!(
            migrations::validate(&c, &migrations::MIGRATIONS[..10]).unwrap(),
            10
        );
        assert_eq!(
            c.query_row(
                "SELECT snapshot FROM recipe_versions WHERE recipe_id=?1",
                [&a.id],
                |r| r.get::<_, String>(0)
            )
            .unwrap(),
            payload
        );
    }
    let c = Connection::open(&path).unwrap();
    c.execute(
        "UPDATE schema_migrations SET checksum=?1 WHERE version=11",
        ["0".repeat(64)],
    )
    .unwrap();
    drop(c);
    assert_eq!(
        Database::open(dir.path()).err().unwrap().code,
        "SCHEMA_INTEGRITY"
    );
}

#[test]
fn paginated_relationships_and_real_connection_stale_cas_preserve_all_targets() {
    let (dir, mut db, a) = setup();
    let b = db
        .save_recipe(input(&a.ingredients[0].ingredient_id))
        .unwrap();
    let c = db
        .save_recipe(input(&a.ingredients[0].ingredient_id))
        .unwrap();
    let mut other = Database::open(dir.path()).unwrap();
    let old = other.recipe_links(&a.id, 0, 20).unwrap().graph_revision;
    related(&mut db, &a, &b, true);
    assert_eq!(
        other
            .set_related_recipe(RelatedWrite {
                recipe_id: a.id.clone(),
                target_id: c.id.clone(),
                expected_graph_revision: old,
                linked: true
            })
            .unwrap_err()
            .code,
        "CONFLICT"
    );
    assert_eq!(
        other
            .set_variation_parent(ParentWrite {
                child_id: a.id.clone(),
                parent_id: Some(c.id),
                expected_graph_revision: old
            })
            .unwrap_err()
            .code,
        "CONFLICT"
    );
    for _ in 0..24 {
        let peer = db
            .save_recipe(input(&a.ingredients[0].ingredient_id))
            .unwrap();
        related(&mut db, &a, &peer, true);
    }
    let page = db.recipe_links(&a.id, 0, 20).unwrap();
    assert_eq!(page.total, 25);
    assert_eq!(page.items.len(), 20);
    assert!(page.has_more);
    let next = db.recipe_links(&a.id, 20, 20).unwrap();
    assert_eq!(next.items.len(), 5);
    assert!(!next.has_more);
    assert!(page
        .items
        .iter()
        .all(|r| !next.items.iter().any(|n| n.id == r.id)));
    assert!(db.recipe_links(&a.id, 0, 101).is_err());
    assert!(db.recipe_links("invalid", 0, 20).is_err());
    assert_eq!(count(&db, &a.id), 1);
    let trash = db.set_deleted(&a.id, 1, true).unwrap();
    let w = purge_write(&db, &trash, true);
    related(&mut other, &trash, &b, false);
    assert_eq!(
        db.purge_reviewed_recipe(w).unwrap_err().code,
        "PURGE_REVIEW_STALE"
    );
    assert!(db.recipe(&b.id).is_ok());
    health(&db);
}
