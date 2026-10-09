use super::*;
use crate::{
    catalog_tool,
    domain::{catalog::*, recipes::*, reliability::*},
};
use std::path::Path;
fn seed() -> VerifiedSeed {
    catalog_tool::load(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../catalog/validation/manifest.json"),
    )
    .unwrap()
}
fn count(db: &Database, table: &str) -> i64 {
    db.conn
        .query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r.get(0))
        .unwrap()
}
fn next(seed: &mut VerifiedSeed) {
    seed.manifest.version += 1;
    seed.manifest_hash = super::migrations::checksum(&serde_json::to_string(&seed.data).unwrap());
}
fn recipe(i: &Ingredient) -> RecipeInput {
    RecipeInput {
        id: uuid::Uuid::new_v4().to_string(),
        expected_revision: None,
        title: "Şerbet".into(),
        description: None,
        kind: "beverage".into(),
        servings: "1".into(),
        prep_minutes: None,
        cook_minutes: None,
        notes: None,
        ingredients: vec![RecipeIngredient {
            id: uuid::Uuid::new_v4().to_string(),
            ingredient_id: i.id.clone(),
            quantity: Some("35.000001".into()),
            unit_code: "cc".into(),
            note: None,
        }],
        steps: vec![RecipeStep {
            id: uuid::Uuid::new_v4().to_string(),
            instructions: "Karıştırın".into(),
        }],
    }
}
#[test]
fn installation_idempotency_counts_aliases_sources_and_restart() {
    let dir = tempfile::tempdir().unwrap();
    let mut db = Database::open(dir.path()).unwrap();
    let seed = seed();
    assert_eq!(db.catalog_status().unwrap().definitions, 0);
    let report = db.import_catalog(&seed).unwrap();
    assert_eq!(
        (
            report.validated,
            report.inserted,
            report.available,
            report.collisions
        ),
        (8, 8, 8, 0)
    );
    assert_eq!(report.category_coverage["sweeteners"], 3);
    assert_eq!(report.category_coverage["beverage"], 3);
    assert_eq!(count(&db, "ingredient_categories"), 8);
    assert_eq!(count(&db, "catalog_provenance"), 16);
    assert_eq!(count(&db, "catalog_observations"), 0);
    assert!(db.import_catalog(&seed).unwrap().unchanged);
    assert_eq!(count(&db, "ingredients"), 8);
    assert_eq!(db.catalog_status().unwrap().production_definitions, 0);
    assert_eq!(
        db.create_ingredient("ICING SUGAR").unwrap().id,
        stable_id("ingredient", "sugar-powdered")
    );
    assert_eq!(count(&db, "ingredients"), 8);
    drop(db);
    let db = Database::open(dir.path()).unwrap();
    for q in ["Şek", "şek", "ŞEK", "şeker"] {
        assert!(db
            .ingredients(q)
            .unwrap()
            .iter()
            .any(|i| i.id == stable_id("ingredient", "sugar-granulated")));
    }
    for q in ["Icing", "ICING", "icing", "Confectioners"] {
        assert_eq!(
            db.ingredients(q).unwrap()[0].id,
            stable_id("ingredient", "sugar-powdered")
        );
    }
    assert_eq!(db.ingredients("Whole milk").unwrap().len(), 1);
    let available = db.available_ingredients("ICING").unwrap();
    assert_eq!(available.total, 8);
    assert_eq!(available.items.len(), 1);

    assert_eq!(db.ingredients("S\u{0327}EK").unwrap().len(), 2);
    assert_eq!(
        db.conn
            .query_row(
                "SELECT license FROM catalog_sources WHERE id='usda-sr-legacy'",
                [],
                |r| r.get::<_, String>(0)
            )
            .unwrap(),
        "CC0-1.0"
    );
    assert_eq!(
        db.conn
            .query_row(
                "SELECT count(*) FROM catalog_import_runs WHERE status='succeeded'",
                [],
                |r| r.get::<_, i64>(0)
            )
            .unwrap(),
        2
    );
}
#[test]
fn personal_collision_preserves_ids_recipes_drafts_and_explicit_link() {
    let dir = tempfile::tempdir().unwrap();
    let mut db = Database::open(dir.path()).unwrap();
    let personal = db.create_ingredient("Şeker").unwrap();
    let saved = db.save_recipe(recipe(&personal)).unwrap();
    let draft = db
        .save_draft(DraftWrite {
            id: uuid::Uuid::new_v4().to_string(),
            expected_revision: None,
            input: recipe(&personal),
        })
        .unwrap();
    let report = db.import_catalog(&seed()).unwrap();
    assert_eq!((report.available, report.collisions), (7, 1));
    assert_eq!(count(&db, "ingredients"), 8);
    assert_eq!(
        db.recipe(&saved.id).unwrap().ingredients[0].ingredient_id,
        personal.id
    );
    assert_eq!(
        db.draft(&draft.id).unwrap().input.ingredients[0].ingredient_id,
        personal.id
    );
    let canonical = stable_id("ingredient", "sugar-granulated");
    db.link_personal_catalog(&personal.id, 1, &canonical)
        .unwrap();
    assert_eq!(
        db.ingredients("şeker")
            .unwrap()
            .iter()
            .find(|i| i.id == personal.id)
            .unwrap()
            .name,
        "Şeker"
    );
    assert_eq!(
        db.link_personal_catalog(&personal.id, 1, &canonical)
            .unwrap_err()
            .code,
        "CONFLICT"
    );
    let mut newer = seed();
    next(&mut newer);
    newer
        .data
        .ingredients
        .iter_mut()
        .find(|i| i.key == "sugar-granulated")
        .unwrap()
        .names
        .insert("tr".into(), "Kristal şeker".into());
    db.import_catalog(&newer).unwrap();
    assert_eq!(db.search_personal("şeker").unwrap().items[0].name, "Şeker");
    assert_eq!(db.ingredients("white sugar").unwrap()[0].id, personal.id);
    assert_eq!(
        db.delete_ingredient(&personal.id, 2).unwrap_err().code,
        "INGREDIENT_REFERENCED"
    );
    drop(db);
    let db = Database::open(dir.path()).unwrap();
    assert_eq!(
        db.recipe(&saved.id).unwrap().ingredients[0]
            .quantity
            .as_deref(),
        Some("35.000001")
    );
    assert_eq!(
        db.draft(&draft.id).unwrap().input.ingredients[0].ingredient_id,
        personal.id
    );
}
#[test]
fn catalog_upgrade_preserves_customizations_and_unchanged_revisions() {
    let dir = tempfile::tempdir().unwrap();
    let mut db = Database::open(dir.path()).unwrap();
    let mut seed = seed();
    db.import_catalog(&seed).unwrap();
    let id = stable_id("ingredient", "honey");
    let changed = db
        .customize_catalog_ingredient(IngredientEdit {
            id: id.clone(),
            revision: 1,
            name: "Benim balım".into(),
            notes: Some("Kişisel not".into()),
            preferred_unit: Some("kg".into()),
        })
        .unwrap();
    assert_eq!(changed.revision, 2);
    assert_eq!(
        db.customize_catalog_ingredient(IngredientEdit {
            id: id.clone(),
            revision: 1,
            name: "Eski oturum".into(),
            notes: None,
            preferred_unit: None
        })
        .unwrap_err()
        .code,
        "CONFLICT"
    );
    next(&mut seed);
    seed.data
        .ingredients
        .iter_mut()
        .find(|i| i.key == "honey")
        .unwrap()
        .names
        .insert("tr".into(), "Bal (güncel)".into());
    db.import_catalog(&seed).unwrap();
    let (name, notes, unit, revision): (String, String, String, i64) = db
        .conn
        .query_row(
            "SELECT name,notes,preferred_unit,revision FROM ingredients WHERE id=?1",
            [&id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )
        .unwrap();
    assert_eq!(
        (name, notes, unit, revision),
        ("Benim balım".into(), "Kişisel not".into(), "kg".into(), 2)
    );
    assert_eq!(db.ingredients("Honey").unwrap()[0].id, id);
    let mut changed_package = super::catalog_tests::seed();
    changed_package.manifest_hash = "f".repeat(64);
    assert_eq!(
        db.import_catalog(&changed_package).unwrap_err().code,
        "CATALOG_DOWNGRADE"
    );
    seed.manifest_hash = "a".repeat(64);
    assert_eq!(
        db.import_catalog(&seed).unwrap_err().code,
        "CATALOG_VERSION_CHANGED"
    );
}
#[test]
fn failed_import_rolls_back_all_catalog_writes_and_logs_failure() {
    let dir = tempfile::tempdir().unwrap();
    let mut db = Database::open(dir.path()).unwrap();
    let personal = db.create_ingredient("Kişisel un").unwrap();
    db.conn.execute_batch("CREATE TRIGGER fail_catalog BEFORE INSERT ON catalog_names WHEN NEW.name='Toz şeker' BEGIN SELECT RAISE(ABORT,'injected failure'); END;").unwrap();
    assert!(db.import_catalog(&seed()).is_err());
    for table in [
        "catalog_ingredients",
        "catalog_sources",
        "catalog_releases",
        "ingredient_categories",
    ] {
        assert_eq!(count(&db, table), 0);
    }
    assert_eq!(db.ingredients("kişisel").unwrap()[0].id, personal.id);
    assert_eq!(
        db.conn
            .query_row("SELECT status FROM catalog_import_runs", [], |r| r
                .get::<_, String>(0))
            .unwrap(),
        "failed"
    );
    db.conn.execute_batch("DROP TRIGGER fail_catalog").unwrap();
    assert_eq!(db.import_catalog(&seed()).unwrap().inserted, 8);
}
#[test]
fn validation_rejects_duplicates_bad_units_cycles_and_unknown_metadata() {
    let dir = tempfile::tempdir().unwrap();
    let mut db = Database::open(dir.path()).unwrap();
    let mut duplicate = seed();
    duplicate
        .data
        .ingredients
        .push(duplicate.data.ingredients[0].clone());
    assert!(db.import_catalog(&duplicate).is_err());
    assert_eq!(count(&db, "catalog_import_runs"), 0);
    let mut bad = seed();
    bad.data.ingredients[0].preferred_unit = Some("bogus".into());
    assert_eq!(
        db.import_catalog(&bad).unwrap_err().code,
        "CATALOG_INVALID_UNIT"
    );
    assert_eq!(count(&db, "catalog_ingredients"), 0);
    let mut cycle = seed();
    cycle.data.categories[0].parent = Some(cycle.data.categories[0].key.clone());
    assert_eq!(
        db.import_catalog(&cycle).unwrap_err().code,
        "CATALOG_CATEGORY_CYCLE"
    );
    let mut density = seed();
    let p = &density.data.ingredients[0].provenance[0];
    density.data.ingredients[0].observations = vec![Observation {
        kind: "density".into(),
        code: "bulk".into(),
        value: "0".into(),
        unit: Some("g/mL".into()),
        basis: "unknown".into(),
        source_id: p.source_id.clone(),
        source_version: p.source_version.clone(),
        external_id: p.external_id.clone(),
    }];
    assert!(db.import_catalog(&density).is_err());
}
#[test]
fn hierarchy_and_personal_categories_enforce_foreign_keys_and_cycles() {
    let dir = tempfile::tempdir().unwrap();
    let mut db = Database::open(dir.path()).unwrap();
    db.import_catalog(&seed()).unwrap();
    let food = stable_id("category", "food");
    let category = db
        .create_personal_category("Özel baharatlar", "Special spices", Some(&food))
        .unwrap();
    assert!(db
        .conn
        .execute(
            "UPDATE ingredient_categories SET parent_id=?1 WHERE id=?2",
            rusqlite::params![category, food]
        )
        .is_err());
    assert!(db
        .conn
        .execute("DELETE FROM ingredient_categories WHERE id=?1", [&food])
        .is_err());
    assert!(db.create_personal_category("", "Empty", None).is_err());
}
#[test]
fn schema_three_upgrade_preserves_m2b_state_and_migration_checksums() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("library.sqlite3");
    let mut conn = Connection::open(&path).unwrap();
    migrations::apply(&mut conn, &path, &migrations::MIGRATIONS[..3]).unwrap();
    conn.pragma_update(None, "foreign_keys", true).unwrap();
    let mut db = Database { conn };
    // Historical schema-3 fixture uses its original ingredient insertion contract.
    let ingredient = Ingredient {
        id: uuid::Uuid::new_v4().to_string(),
        name: "İçme suyu".into(),
    };
    db.conn
        .execute(
            "INSERT INTO ingredients(id,name,search_name) VALUES(?1,?2,?3)",
            rusqlite::params![
                ingredient.id,
                ingredient.name,
                search_name(&ingredient.name)
            ],
        )
        .unwrap();
    let r = db.save_recipe(recipe(&ingredient)).unwrap();
    let archived = db.archive_recipe(&r.id, 1, true).unwrap();
    let trashed = db.set_deleted(&r.id, archived.revision, true).unwrap();
    let draft = db
        .save_draft(DraftWrite {
            id: uuid::Uuid::new_v4().to_string(),
            expected_revision: None,
            input: recipe(&ingredient),
        })
        .unwrap();
    drop(db);
    let mut db = Database::open(dir.path()).unwrap();
    assert_eq!(db.bootstrap().unwrap().storage.schema_version, 4);
    assert_eq!(db.recipe(&r.id).unwrap().deleted_at, trashed.deleted_at);
    assert_eq!(db.draft(&draft.id).unwrap().revision, 1);
    db.import_catalog(&seed()).unwrap();
    db.install_bundled_catalog().unwrap();
    assert_eq!(db.catalog_status().unwrap().production_definitions, 235);
    assert_eq!(
        db.recipe(&r.id).unwrap().ingredients[0].ingredient_id,
        ingredient.id
    );
    assert!(db
        .ingredients("içme")
        .unwrap()
        .iter()
        .any(|i| i.id == ingredient.id));
    assert_eq!(
        migrations::validate(&db.conn, migrations::MIGRATIONS).unwrap(),
        4
    );
}
#[test]
fn package_checksums_evidence_license_and_validation_opt_in_are_enforced() {
    let dir = tempfile::tempdir().unwrap();
    let original = Path::new(env!("CARGO_MANIFEST_DIR")).join("../catalog/validation");
    for name in [
        "manifest.json",
        "ingredients.json",
        "usda-records.json",
        "curation-records.json",
    ] {
        std::fs::copy(original.join(name), dir.path().join(name)).unwrap();
    }
    let manifest = dir.path().join("manifest.json");
    assert!(catalog_tool::load(&manifest).is_ok());
    let db = tempfile::tempdir().unwrap();
    assert_eq!(
        catalog_tool::import(db.path(), &manifest, false)
            .unwrap_err()
            .code,
        "CATALOG_VALIDATION_ONLY"
    );
    assert!(!db.path().join("library.sqlite3").exists());
    let mut value: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&manifest).unwrap()).unwrap();
    value["sources"][0]["license"] = serde_json::json!("Unverified");
    std::fs::write(&manifest, serde_json::to_vec(&value).unwrap()).unwrap();
    assert!(
        matches!(catalog_tool::load(&manifest),Err(AppError{code,..}) if code=="CATALOG_LICENSE")
    );
    std::fs::copy(original.join("manifest.json"), &manifest).unwrap();
    std::fs::write(dir.path().join("ingredients.json"), "{}").unwrap();
    assert!(
        matches!(catalog_tool::load(&manifest),Err(AppError{code,..}) if code=="CATALOG_CHECKSUM")
    );
}
#[test]
fn catalog_crash_child() {
    let Ok(directory) = std::env::var("RECIPEATLAS_CATALOG_CRASH_DIRECTORY") else {
        return;
    };
    let mut db = Database::open(Path::new(&directory)).unwrap();
    db.import_catalog(&seed()).unwrap();
}
#[test]
fn killed_import_process_rolls_back_uncommitted_package_and_retry_recovers() {
    let dir = tempfile::tempdir().unwrap();
    let marker = dir.path().join("ready");
    let mut child = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "persistence::catalog_tests::catalog_crash_child",
            "--nocapture",
        ])
        .env("RECIPEATLAS_CATALOG_CRASH_DIRECTORY", dir.path())
        .env("RECIPEATLAS_CATALOG_CRASH_MARKER", &marker)
        .stdout(std::process::Stdio::null())
        .spawn()
        .unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
    while !marker.exists() && std::time::Instant::now() < deadline {
        if child.try_wait().unwrap().is_some() {
            panic!("import helper exited before readiness");
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    let ready = marker.exists();
    child.kill().unwrap();
    child.wait().unwrap();
    assert!(ready, "import helper readiness timeout");
    let mut db = Database::open(dir.path()).unwrap();
    assert_eq!(count(&db, "catalog_ingredients"), 0);
    assert_eq!(count(&db, "catalog_releases"), 0);
    assert_eq!(
        db.conn
            .query_row("SELECT status FROM catalog_import_runs", [], |r| r
                .get::<_, String>(0))
            .unwrap(),
        "running"
    );
    assert_eq!(db.import_catalog(&seed()).unwrap().inserted, 8);
}

#[test]
fn updated_system_name_collision_rejects_package_without_overwriting_personal_data() {
    let dir = tempfile::tempdir().unwrap();
    let mut db = Database::open(dir.path()).unwrap();
    let mut seed = seed();
    db.import_catalog(&seed).unwrap();
    let personal = db.create_ingredient("Anadolu balı").unwrap();
    next(&mut seed);
    seed.data
        .ingredients
        .iter_mut()
        .find(|i| i.key == "honey")
        .unwrap()
        .names
        .insert("tr".into(), "Anadolu balı".into());
    assert_eq!(
        db.import_catalog(&seed).unwrap_err().code,
        "CATALOG_NAME_CONFLICT"
    );
    assert_eq!(db.ingredients("Anadolu").unwrap()[0].id, personal.id);
    assert_eq!(db.ingredients("Honey").unwrap()[0].name, "Bal");
    assert_eq!(count(&db, "catalog_releases"), 1);
}

#[test]
fn production_embedded_package_offline_counts_attribution_search_and_recipe_restart() {
    let seed = catalog_tool::bundled().unwrap();
    let disk = catalog_tool::load(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../catalog/production/manifest.json"),
    )
    .unwrap();
    assert_eq!(seed.manifest_hash, disk.manifest_hash);
    assert_eq!(seed.data.ingredients.len(), 235);
    assert_eq!(seed.data.categories.len(), 39);
    let dir = tempfile::tempdir().unwrap();
    let mut db = Database::open(dir.path()).unwrap();
    let report = db.import_catalog(&seed).unwrap();
    assert_eq!(
        (
            report.validated,
            report.inserted,
            report.available,
            report.collisions
        ),
        (235, 235, 235, 0)
    );
    let coverage: serde_json::Value =
        serde_json::from_str(include_str!("../../../catalog/production/coverage.json")).unwrap();
    assert_eq!(
        coverage["productionReadyRecords"].as_i64(),
        Some(i64::try_from(report.validated).unwrap())
    );
    for (key, value) in coverage["categoryMembershipCounts"].as_object().unwrap() {
        assert_eq!(
            report.category_coverage[key],
            usize::try_from(value.as_u64().unwrap()).unwrap()
        );
    }
    assert!(seed
        .manifest
        .sources
        .iter()
        .all(|s| s.license == "CC0-1.0" && !s.attribution.is_empty()));
    assert!(seed
        .data
        .ingredients
        .iter()
        .all(|i| i.names.contains_key("tr")
            && i.names.contains_key("en")
            && i.observations.is_empty()));
    assert_eq!(report.category_coverage["vegetables"], 41);
    assert_eq!(report.category_coverage["spirits"], 3);
    assert_eq!(count(&db, "catalog_provenance"), 470);
    assert_eq!(count(&db, "catalog_observations"), 0);
    assert_eq!(db.catalog_status().unwrap().production_definitions, 235);
    for q in ["Şek", "şek", "ŞEK", "şeker", "S\u{0327}EK"] {
        assert!(db
            .available_ingredients(q)
            .unwrap()
            .items
            .iter()
            .any(|i| i.id == stable_id("ingredient", "sugar-granulated")));
    }
    for q in ["ICING", "Icing", "icing"] {
        assert_eq!(
            db.ingredients(q).unwrap()[0].id,
            stable_id("ingredient", "sugar-powdered")
        );
    }
    for q in ["Ispanak", "ıspanak", "ISPANAK"] {
        assert_eq!(db.ingredients(q).unwrap()[0].name, "Ispanak (çiğ)");
    }
    let tomato = db.ingredients("Domates (çiğ").unwrap().remove(0);
    assert_eq!(db.ingredients("Tomato (raw").unwrap()[0].id, tomato.id);
    let saved = db.save_recipe(recipe(&tomato)).unwrap();
    assert!(db.import_catalog(&seed).unwrap().unchanged);
    let runs = count(&db, "catalog_import_runs");
    db.install_bundled_catalog().unwrap();
    assert_eq!(count(&db, "catalog_import_runs"), runs);
    drop(db);
    let mut db = Database::open(dir.path()).unwrap();
    db.install_bundled_catalog().unwrap();
    assert_eq!(
        db.recipe(&saved.id).unwrap().ingredients[0].ingredient_id,
        tomato.id
    );
    assert_eq!(
        db.recipe(&saved.id).unwrap().ingredients[0]
            .quantity
            .as_deref(),
        Some("35.000001")
    );
    let broken: i64 = db
        .conn
        .query_row("SELECT count(*) FROM pragma_foreign_key_check", [], |r| {
            r.get(0)
        })
        .unwrap();
    assert_eq!(broken, 0);
    assert_eq!(
        db.conn
            .query_row("PRAGMA integrity_check", [], |r| r.get::<_, String>(0))
            .unwrap(),
        "ok"
    );
}

#[test]
fn production_upgrade_retains_personal_collisions_overrides_drafts_and_recipes() {
    let dir = tempfile::tempdir().unwrap();
    let mut db = Database::open(dir.path()).unwrap();
    let personal = db.create_ingredient("Şeker").unwrap();
    let saved = db.save_recipe(recipe(&personal)).unwrap();
    let draft = db
        .save_draft(DraftWrite {
            id: uuid::Uuid::new_v4().to_string(),
            expected_revision: None,
            input: recipe(&personal),
        })
        .unwrap();
    db.import_catalog(&seed()).unwrap();
    let honey = stable_id("ingredient", "honey");
    db.customize_catalog_ingredient(IngredientEdit {
        id: honey.clone(),
        revision: 1,
        name: "Benim balım".into(),
        notes: Some("Korunsun".into()),
        preferred_unit: Some("kg".into()),
    })
    .unwrap();
    db.install_bundled_catalog().unwrap();
    assert_eq!(db.catalog_status().unwrap().production_definitions, 235);
    assert_eq!(db.catalog_status().unwrap().pending_collisions, 1);
    assert_eq!(
        db.recipe(&saved.id).unwrap().ingredients[0].ingredient_id,
        personal.id
    );
    assert_eq!(
        db.draft(&draft.id).unwrap().input.ingredients[0].ingredient_id,
        personal.id
    );
    assert_eq!(db.ingredients("Benim balım").unwrap()[0].id, honey);
    let preserved: (String, String) = db
        .conn
        .query_row(
            "SELECT notes,preferred_unit FROM ingredients WHERE id=?1",
            [honey],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert_eq!(preserved, ("Korunsun".into(), "kg".into()));
    assert_eq!(count(&db, "ingredients"), 235);
    assert_eq!(count(&db, "catalog_sources"), 4);
}

#[test]
fn production_invalid_duplicate_and_failure_roll_back_package() {
    let dir = tempfile::tempdir().unwrap();
    let mut db = Database::open(dir.path()).unwrap();
    let mut bad = catalog_tool::bundled().unwrap();
    bad.data.ingredients.push(bad.data.ingredients[0].clone());
    assert!(db.import_catalog(&bad).is_err());
    assert_eq!(count(&db, "catalog_ingredients"), 0);
    // Fail after writes have begun, using a real SQLite constraint/trigger.
    db.conn.execute_batch("CREATE TRIGGER fail_production BEFORE INSERT ON catalog_ingredients WHEN NEW.canonical_key='usda-sr-170457' BEGIN SELECT RAISE(ABORT,'forced rollback'); END;").unwrap();
    assert!(db
        .import_catalog(&catalog_tool::bundled().unwrap())
        .is_err());
    assert_eq!(count(&db, "catalog_ingredients"), 0);
    assert_eq!(count(&db, "ingredient_categories"), 0);
    assert_eq!(count(&db, "catalog_releases"), 0);
    db.conn
        .execute_batch("DROP TRIGGER fail_production")
        .unwrap();
    db.install_bundled_catalog().unwrap();
    assert_eq!(count(&db, "ingredients"), 235);
}

#[test]
fn actual_default_worker_installs_offline_and_reopens_without_reimport() {
    let dir = tempfile::tempdir().unwrap();
    let runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(async {
        let service = crate::services::StorageService::new(Ok(dir.path().into()));
        service.bootstrap().await.unwrap();
        assert_eq!(
            service
                .execute(|db| Ok(db.catalog_status()?.production_definitions))
                .await
                .unwrap(),
            235
        );
        let recipe_id = service
            .execute(|db| {
                let ingredient = db.ingredients("Domates (çiğ")?.remove(0);
                Ok(db.save_recipe(recipe(&ingredient))?.id)
            })
            .await
            .unwrap();
        drop(service);
        let service = crate::services::StorageService::new(Ok(dir.path().into()));
        service.bootstrap().await.unwrap();
        assert_eq!(
            service
                .execute(move |db| Ok(db.recipe(&recipe_id)?.title))
                .await
                .unwrap(),
            "Şerbet"
        );
        assert_eq!(
            service
                .execute(|db| Ok(count(db, "catalog_import_runs")))
                .await
                .unwrap(),
            1
        );
    });
}

#[test]
fn bundled_startup_retains_newer_catalog_and_rejects_same_version_drift() {
    let dir = tempfile::tempdir().unwrap();
    let mut db = Database::open(dir.path()).unwrap();
    db.install_bundled_catalog().unwrap();
    let mut newer = catalog_tool::bundled().unwrap();
    next(&mut newer);
    db.import_catalog(&newer).unwrap();
    let runs = count(&db, "catalog_import_runs");
    db.install_bundled_catalog().unwrap();
    assert_eq!(count(&db, "catalog_import_runs"), runs);
    assert_eq!(
        db.import_catalog(&catalog_tool::bundled().unwrap())
            .unwrap_err()
            .code,
        "CATALOG_DOWNGRADE"
    );
    let fresh = tempfile::tempdir().unwrap();
    let mut db = Database::open(fresh.path()).unwrap();
    db.install_bundled_catalog().unwrap();
    db.conn
        .execute(
            "UPDATE catalog_releases SET manifest_sha256=?1 WHERE version=2",
            ["0".repeat(64)],
        )
        .unwrap();
    assert_eq!(
        db.install_bundled_catalog().unwrap_err().code,
        "CATALOG_VERSION_CHANGED"
    );
    assert_eq!(count(&db, "ingredients"), 235);
}
