use super::*;
use crate::domain::{
    recipe_organization::*, recipe_versions::*, recipe_workspace::*, recipes::*, reliability::*,
};
use rusqlite::params;
use sha2::{Digest, Sha256};
fn id() -> String {
    uuid::Uuid::new_v4().to_string()
}
fn input(ingredient: &str) -> RecipeInput {
    let line = id();
    serde_json::from_value(serde_json::json!({"id":id(),"expectedRevision":null,"title":"M5B2 TEST4 bağımsız kayıt — İ/ı","description":"Türkçe","kind":"beverage","servings":"1","prepMinutes":null,"cookMinutes":0,"notes":"Tarif notu",
 "ingredients":[{"id":line,"ingredientId":ingredient,"quantity":"1.250","unitCode":"mL","note":null}],
 "steps":[{"id":id(),"instructions":"Suyu bardağa ekleyin.","details":{"formatVersion":1,"title":"Suyu hazırlayın","durationMinutes":"1","temperatureValue":null,"temperatureUnit":null,"heatLevel":"Soğuk","equipment":"Bardak","technique":null,"tips":"İnce"},"ingredientLineIds":[line]}]})).unwrap()
}
fn setup() -> (tempfile::TempDir, Database, Recipe) {
    let dir = tempfile::tempdir().unwrap();
    let mut db = Database::open(dir.path()).unwrap();
    let i = db.create_ingredient("M5B2 TEST4 Gül suyu").unwrap();
    let r = db.save_recipe(input(&i.id)).unwrap();
    (dir, db, r)
}
fn edit(db: &mut Database, r: Recipe) -> Recipe {
    let mut value = RecipeSnapshot::from_recipe(r.clone()).content();
    value.expected_revision = Some(r.revision);
    value.title = "İkinci başlık".into();
    value.ingredients[0].quantity = Some("1.25".into());
    db.save_recipe(value).unwrap()
}
fn request(r: &Recipe, v: i64) -> RestoreVersion {
    RestoreVersion {
        recipe_id: r.id.clone(),
        version_number: v,
        expected_revision: r.revision,
    }
}
fn bytes(db: &Database, id: &str) -> Vec<String> {
    db.conn
        .prepare("SELECT snapshot FROM recipe_versions WHERE recipe_id=?1 ORDER BY version_number")
        .unwrap()
        .query_map([id], |r| r.get(0))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap()
}
fn healthy(db: &Database) {
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
fn restore_is_one_atomic_new_version_with_original_identity_and_independent_metadata() {
    let (dir, mut db, v1) = setup();
    let v2 = edit(&mut db, v1.clone());
    let original = bytes(&db, &v1.id);
    db.set_recipe_organization(OrganizationWrite {
        recipe_id: v1.id.clone(),
        expected_revision: 1,
        favorite: true,
        to_try: true,
        rating: Some(4),
    })
    .unwrap();
    db.save_private_note(PrivateNoteWrite {
        recipe_id: v1.id.clone(),
        expected_revision: 0,
        markdown: "M5B2 TEST4 KONTROL: özel not korunuyor.".into(),
    })
    .unwrap();
    let result = db.restore_recipe_version(request(&v2, 1)).unwrap();
    assert!(result.changed);
    assert_eq!(result.version_number, 3);
    assert_eq!(result.recipe.id, v1.id);
    assert_eq!(result.recipe.revision, 3);
    let v3 = db.recipe_version(&v1.id, 3).unwrap();
    assert_eq!(v3.summary.provenance, "restored");
    assert_eq!(v3.summary.restored_from_version, Some(1));
    assert_eq!(v3.summary.source_revision, 3);
    assert_eq!(v3.snapshot.title, v1.title);
    assert_eq!(
        v3.snapshot.ingredients[0].quantity.as_deref(),
        Some("1.250")
    );
    assert_eq!(v3.snapshot.ingredients[0].id, v1.ingredients[0].id);
    assert_eq!(v3.snapshot.steps[0].id, v1.steps[0].id);
    assert_eq!(
        v3.snapshot.steps[0].ingredient_line_ids,
        v1.steps[0].ingredient_line_ids
    );
    assert_eq!(
        v3.snapshot.steps[0]
            .details
            .as_ref()
            .unwrap()
            .duration_minutes
            .as_deref(),
        Some("1")
    );
    assert_eq!(&bytes(&db, &v1.id)[..2], &original);
    assert!(db.recipe_organization(&v1.id).unwrap().favorite);
    assert!(db
        .private_note(&v1.id)
        .unwrap()
        .markdown
        .contains("M5B2 TEST4 KONTROL"));
    let nochange = db
        .restore_recipe_version(request(&result.recipe, 1))
        .unwrap();
    assert!(!nochange.changed);
    assert_eq!(nochange.recipe.revision, 3);
    assert_eq!(bytes(&db, &v1.id).len(), 3);
    drop(db);
    let mut db = Database::open(dir.path()).unwrap();
    assert!(db.recipe_organization(&v1.id).unwrap().favorite);
    assert_eq!(
        db.private_note(&v1.id).unwrap().markdown,
        "M5B2 TEST4 KONTROL: özel not korunuyor."
    );
    assert_eq!(bytes(&db, &v1.id).len(), 3);
    let archived = db.archive_recipe(&v1.id, 3, true).unwrap();
    let active = db.archive_recipe(&v1.id, archived.revision, false).unwrap();
    assert!(db.recipe_organization(&v1.id).unwrap().favorite);
    assert_eq!(bytes(&db, &v1.id).len(), 3);
    let copy = db.duplicate_recipe(&active.id, active.revision).unwrap();
    assert_ne!(copy.id, v1.id);
    assert_ne!(copy.steps[0].id, v1.steps[0].id);
    assert_eq!(db.recipe_versions(&copy.id, None, 20).unwrap().len(), 1);
    healthy(&db);
}
#[test]
fn history_pagination_comparison_exact_strings_labels_order_and_all_step_fields() {
    let (_dir, mut db, v1) = setup();
    let mut saved = edit(&mut db, v1.clone());
    let diff = db.compare_recipe_versions(&v1.id, 1, Some(2)).unwrap();
    assert!(diff
        .changes
        .iter()
        .any(|c| c.path == ["ingredients", "0", "quantity"]
            && c.before == "1.250"
            && c.after == "1.25"));
    let mut value = RecipeSnapshot::from_recipe(saved.clone()).content();
    value.expected_revision = Some(saved.revision);
    value.steps[0].details.as_mut().unwrap().equipment = Some("Sürahi".into());
    value.steps[0].ingredient_line_ids.clear();
    saved = db.save_recipe(value).unwrap();
    let diff = db.compare_recipe_versions(&v1.id, 1, None).unwrap();
    assert!(diff
        .changes
        .iter()
        .any(|c| c.path.contains(&"equipment".into())));
    assert!(diff
        .changes
        .iter()
        .any(|c| c.path.contains(&"ingredientLineIds".into())));
    db.conn
        .execute(
            "UPDATE ingredients SET name='Yeni ad',search_name='yeni ad' WHERE id=?1",
            [&v1.ingredients[0].ingredient_id],
        )
        .unwrap();
    assert_ne!(
        db.recipe_version(&v1.id, 1)
            .unwrap()
            .snapshot
            .ingredient_names[&v1.ingredients[0].ingredient_id],
        "Yeni ad"
    );
    assert!(db
        .compare_recipe_versions(&v1.id, 1, None)
        .unwrap()
        .changes
        .iter()
        .any(|c| c.path[0] == "ingredientNames"));
    assert!(db
        .compare_recipe_versions(&v1.id, 2, Some(2))
        .unwrap()
        .changes
        .is_empty());
    for n in 0..5 {
        let mut val = RecipeSnapshot::from_recipe(saved.clone()).content();
        val.expected_revision = Some(saved.revision);
        val.description = Some(n.to_string());
        saved = db.save_recipe(val).unwrap();
    }
    let page = db.recipe_versions(&v1.id, None, 3).unwrap();
    assert_eq!(
        page.iter().map(|v| v.version_number).collect::<Vec<_>>(),
        vec![8, 7, 6]
    );
    let next = db.recipe_versions(&v1.id, Some(6), 3).unwrap();
    assert_eq!(
        next.iter().map(|v| v.version_number).collect::<Vec<_>>(),
        vec![5, 4, 3]
    );
    assert!(db.recipe_versions(&v1.id, None, 101).is_err());
    assert!(db.recipe_versions(&v1.id, Some(0), 3).is_err());
    assert!(db
        .recipe_versions("';DROP TABLE recipes;--", None, 3)
        .is_err());
    assert!(db.recipe_version(&v1.id, 99).is_err());
    healthy(&db);
}
#[test]
fn restore_rejects_stale_drafts_and_inactive_without_modification() {
    let (_dir, mut db, v1) = setup();
    let v2 = edit(&mut db, v1.clone());
    let before = bytes(&db, &v1.id);
    assert_eq!(
        db.restore_recipe_version(request(&v1, 1)).unwrap_err().code,
        "CONFLICT"
    );
    let mut val = RecipeSnapshot::from_recipe(v2.clone()).content();
    val.expected_revision = Some(v2.revision);
    let draft = db
        .save_draft(DraftWrite {
            id: id(),
            expected_revision: None,
            input: val,
        })
        .unwrap();
    assert_eq!(
        db.restore_recipe_version(request(&v2, 1)).unwrap_err().code,
        "RECIPE_RESTORE_DRAFT"
    );
    assert_eq!(db.draft(&draft.id).unwrap().revision, draft.revision);
    db.discard_draft(&draft.id, draft.revision).unwrap();
    let archived = db.archive_recipe(&v2.id, v2.revision, true).unwrap();
    assert_eq!(db.recipe_versions(&v2.id, None, 20).unwrap().len(), 2);
    assert_eq!(
        db.restore_recipe_version(request(&archived, 1))
            .unwrap_err()
            .code,
        "RECIPE_RESTORE_INACTIVE"
    );
    let active = db.archive_recipe(&v2.id, archived.revision, false).unwrap();
    let trash = db.set_deleted(&active.id, active.revision, true).unwrap();
    assert!(db.recipe_version(&trash.id, 1).is_ok());
    assert_eq!(
        db.restore_recipe_version(request(&trash, 1))
            .unwrap_err()
            .code,
        "RECIPE_RESTORE_INACTIVE"
    );
    assert_eq!(bytes(&db, &v1.id), before);
    healthy(&db);
}
#[test]
fn injected_insert_failure_and_id_collisions_roll_back_whole_restore() {
    let (_dir, mut db, v1) = setup();
    let mut value = RecipeSnapshot::from_recipe(v1.clone()).content();
    value.expected_revision = Some(1);
    value.ingredients[0].id = id();
    value.steps[0].ingredient_line_ids = vec![value.ingredients[0].id.clone()];
    value.steps[0].id = id();
    value.title = "V2".into();
    let v2 = db.save_recipe(value).unwrap();
    let before = bytes(&db, &v1.id);
    db.conn.execute_batch("CREATE TRIGGER fail_restore BEFORE INSERT ON recipe_versions WHEN NEW.provenance='restored' BEGIN SELECT RAISE(ABORT,'injected'); END;").unwrap();
    assert!(db.restore_recipe_version(request(&v2, 1)).is_err());
    assert_eq!(db.recipe(&v2.id).unwrap().title, "V2");
    assert_eq!(db.recipe(&v2.id).unwrap().revision, 2);
    assert_eq!(bytes(&db, &v1.id), before);
    db.conn.execute_batch("DROP TRIGGER fail_restore").unwrap();
    let mut other = input(&v1.ingredients[0].ingredient_id);
    other.ingredients[0].id = v1.ingredients[0].id.clone();
    other.steps[0].ingredient_line_ids = vec![other.ingredients[0].id.clone()];
    db.save_recipe(other).unwrap();
    assert!(db.restore_recipe_version(request(&v2, 1)).is_err());
    assert_eq!(db.recipe(&v2.id).unwrap().revision, 2);
    assert_eq!(bytes(&db, &v1.id), before);
    healthy(&db);
}
#[test]
fn corrupt_and_unsupported_payloads_never_restore_or_change_committed_rows() {
    for future in [false, true] {
        let (_dir, mut db, v1) = setup();
        let v2 = edit(&mut db, v1.clone());
        db.conn
            .execute_batch("DROP TRIGGER recipe_versions_immutable_update")
            .unwrap();
        if future {
            let mut payload: serde_json::Value =
                serde_json::from_str(&bytes(&db, &v1.id)[0]).unwrap();
            payload["formatVersion"] = 4.into();
            let text = payload.to_string();
            let hash = format!("{:x}", Sha256::digest(text.as_bytes()));
            db.conn.execute("UPDATE recipe_versions SET format_version=4,snapshot=?2,snapshot_checksum=?3 WHERE recipe_id=?1 AND version_number=1",params![v1.id,text,hash]).unwrap();
        } else {
            db.conn.execute("UPDATE recipe_versions SET snapshot_checksum=?2 WHERE recipe_id=?1 AND version_number=1",params![v1.id,"0".repeat(64)]).unwrap();
        }
        assert!(db.recipe_version(&v1.id, 1).is_err());
        assert!(db.compare_recipe_versions(&v1.id, 1, None).is_err());
        assert!(db.restore_recipe_version(request(&v2, 1)).is_err());
        assert_eq!(db.recipe(&v2.id).unwrap().revision, 2);
        assert_eq!(bytes(&db, &v1.id).len(), 2);
    }
}
#[test]
fn missing_historical_identity_is_rejected_without_name_substitution() {
    let (_dir, mut db, v1) = setup();
    let mut val = RecipeSnapshot::from_recipe(v1.clone()).content();
    val.expected_revision = Some(1);
    val.ingredients.clear();
    val.steps.clear();
    let v2 = db.save_recipe(val).unwrap();
    assert!(db
        .delete_ingredient(&v1.ingredients[0].ingredient_id, 1)
        .is_err());
    // Simulate damaged external DB: FK safeguards are bypassed ONLY in this fixture.
    db.conn.pragma_update(None, "foreign_keys", false).unwrap();
    db.conn
        .execute(
            "DELETE FROM ingredients WHERE id=?1",
            [&v1.ingredients[0].ingredient_id],
        )
        .unwrap();
    db.conn.pragma_update(None, "foreign_keys", true).unwrap();
    assert_eq!(
        db.restore_recipe_version(request(&v2, 1)).unwrap_err().code,
        "RECIPE_VERSION_REFERENCE"
    );
    assert_eq!(db.recipe(&v2.id).unwrap().revision, 2);
    assert_eq!(bytes(&db, &v1.id).len(), 2);
}

#[test]
fn uncommitted_restore_child() {
    let Ok(path) = std::env::var("M5B2_RESTORE_CRASH") else {
        return;
    };
    let mut db = Database::open(std::path::Path::new(&path)).unwrap();
    let rid: String = db
        .conn
        .query_row("SELECT id FROM recipes", [], |r| r.get(0))
        .unwrap();
    let historical = db.recipe_version(&rid, 1).unwrap();
    let mut val = historical.snapshot.content();
    val.expected_revision = Some(2);
    let tx = db
        .conn
        .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
        .unwrap();
    super::recipes::write_recipe_with_history(&tx, &val, Some(1)).unwrap();
    std::fs::write(
        std::path::Path::new(&path).join("restore-ready"),
        b"pending",
    )
    .unwrap();
    loop {
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
}
#[test]
fn killed_restore_rolls_back_current_content_and_new_snapshot_then_retry_succeeds() {
    let (dir, mut db, v1) = setup();
    let v2 = edit(&mut db, v1);
    let original = bytes(&db, &v2.id);
    drop(db);
    let mut child = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "persistence::recipe_history_tests::uncommitted_restore_child",
            "--nocapture",
        ])
        .env("M5B2_RESTORE_CRASH", dir.path())
        .stdout(std::process::Stdio::null())
        .spawn()
        .unwrap();
    let start = std::time::Instant::now();
    while !dir.path().join("restore-ready").exists() {
        if start.elapsed() > std::time::Duration::from_secs(20) {
            let _ = child.kill();
            let _ = child.wait();
            panic!("child restore did not start");
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    child.kill().unwrap();
    child.wait().unwrap();
    let mut db = Database::open(dir.path()).unwrap();
    assert_eq!(bytes(&db, &v2.id), original);
    assert_eq!(db.recipe(&v2.id).unwrap().title, v2.title);
    assert_eq!(db.recipe(&v2.id).unwrap().revision, 2);
    assert_eq!(
        db.restore_recipe_version(request(&v2, 1))
            .unwrap()
            .version_number,
        3
    );
    healthy(&db);
}
