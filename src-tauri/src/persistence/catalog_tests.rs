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
            category_ids: None,
        })
        .unwrap();
    assert_eq!(changed.revision, 2);
    assert_eq!(
        db.customize_catalog_ingredient(IngredientEdit {
            id: id.clone(),
            revision: 1,
            name: "Eski oturum".into(),
            notes: None,
            preferred_unit: None,
            category_ids: None,
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
    assert_eq!(db.bootstrap().unwrap().storage.schema_version, 8);
    assert_eq!(db.recipe(&r.id).unwrap().deleted_at, trashed.deleted_at);
    assert_eq!(db.draft(&draft.id).unwrap().revision, 1);
    db.import_catalog(&seed()).unwrap();
    db.install_bundled_catalog().unwrap();
    assert_eq!(db.catalog_status().unwrap().production_definitions, 854);
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
        8
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
    let package = if std::env::var_os("RECIPEATLAS_CATALOG_CRASH_PRODUCTION").is_some() {
        catalog_tool::bundled().unwrap()
    } else {
        seed()
    };
    db.import_catalog(&package).unwrap();
}
#[test]
fn killed_import_process_rolls_back_uncommitted_package_and_retry_recovers() {
    verify_killed_import(false);
}
#[test]
fn killed_production_import_rolls_back_and_recovers_full_offline_seed() {
    verify_killed_import(true);
}
fn verify_killed_import(production: bool) {
    let dir = tempfile::tempdir().unwrap();
    let marker = dir.path().join("ready");
    let mut command = std::process::Command::new(std::env::current_exe().unwrap());
    if production {
        command.env("RECIPEATLAS_CATALOG_CRASH_PRODUCTION", "1");
    }
    let mut child = command
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
    let package = if production {
        catalog_tool::bundled().unwrap()
    } else {
        seed()
    };
    assert_eq!(
        db.import_catalog(&package).unwrap().inserted,
        if production { 854 } else { 8 }
    );
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
    assert_eq!(seed.data.ingredients.len(), 854);
    assert_eq!(seed.data.categories.len(), 50);
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
        (854, 854, 854, 0)
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
    assert_eq!(report.category_coverage["vegetables"], 125);
    assert_eq!(report.category_coverage["spirits"], 26);
    assert_eq!(count(&db, "catalog_provenance"), 1659);
    assert_eq!(count(&db, "catalog_observations"), 0);
    assert_eq!(db.catalog_status().unwrap().production_definitions, 854);
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
        category_ids: None,
    })
    .unwrap();
    db.install_bundled_catalog().unwrap();
    assert_eq!(db.catalog_status().unwrap().production_definitions, 854);
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
    assert_eq!(count(&db, "ingredients"), 854);
    assert_eq!(count(&db, "catalog_sources"), 18);
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
    assert_eq!(count(&db, "ingredients"), 854);
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
            854
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
            "UPDATE catalog_releases SET manifest_sha256=?1 WHERE version=11",
            ["0".repeat(64)],
        )
        .unwrap();
    assert_eq!(
        db.install_bundled_catalog().unwrap_err().code,
        "CATALOG_VERSION_CHANGED"
    );
    assert_eq!(count(&db, "ingredients"), 854);
}

#[test]
fn bundled_startup_upgrades_successfully_installed_release8_preview() {
    let dir = tempfile::tempdir().unwrap();
    let mut db = Database::open(dir.path()).unwrap();
    let release8 = catalog_tool::load(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../catalog/releases/8/manifest.json"),
    )
    .unwrap();
    db.import_catalog(&release8).unwrap();
    let releases_before = count(&db, "catalog_import_runs");
    db.conn
        .execute(
            "UPDATE catalog_releases SET manifest_sha256=?1,artifact_sha256=?2 WHERE version=8",
            rusqlite::params![
                "86c0aac8e69ace4a3d75b7beb915a7990aac1fb8656d70bc1e2ea19a8d9b3749",
                "9a73d045c5f9addd913c128fa01d3759eeaeeb95599d022bc164ea5841dedc59"
            ],
        )
        .unwrap();
    db.install_bundled_catalog().unwrap();
    assert_eq!(count(&db, "catalog_import_runs"), releases_before + 1);
    assert_eq!(count(&db, "catalog_ingredients"), 854);
    assert_eq!(count(&db, "catalog_releases"), 2);
    db.install_bundled_catalog().unwrap();
    assert_eq!(count(&db, "catalog_import_runs"), releases_before + 1);
    // Merely knowing the manifest is insufficient: the matching artifact and
    // successful installation audit are also required.
    db.conn
        .execute(
            "UPDATE catalog_releases SET artifact_sha256=?1 WHERE version=11",
            ["0".repeat(64)],
        )
        .unwrap();
    assert_eq!(
        db.install_bundled_catalog().unwrap_err().code,
        "CATALOG_VERSION_CHANGED"
    );
}

#[test]
fn actual_v2_upgrade_preserves_every_identity_personal_draft_recipe_override_and_restart() {
    verify_historical_upgrade(2, 235, "Cin");
}

#[test]
fn actual_v3_upgrade_preserves_every_identity_personal_draft_recipe_override_and_restart() {
    verify_historical_upgrade(3, 482, "Bal şurubu");
}

#[test]
fn actual_v4_upgrade_preserves_every_identity_personal_draft_recipe_override_and_restart() {
    verify_historical_upgrade(4, 508, "Ailelik sos");
}

#[test]
fn actual_v5_upgrade_preserves_every_identity_personal_draft_recipe_override_and_restart() {
    verify_historical_upgrade(5, 508, "Ailelik sos");
}

#[test]
fn actual_v6_upgrade_preserves_every_identity_personal_draft_recipe_override_and_restart() {
    verify_historical_upgrade(6, 508, "Ailelik sos");
}

#[test]
fn terminology_v5_fresh_and_v4_upgrade_keep_ids_overrides_search_recipe_and_integrity() {
    let old = catalog_tool::load(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../catalog/releases/4/manifest.json"),
    )
    .unwrap();
    let new = catalog_tool::load(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../catalog/releases/5/manifest.json"),
    )
    .unwrap();
    let cases = [
        (
            "usda-sr-170918",
            "8f3055ee-2f8e-8300-87a4-2755af76b4e6",
            "Frenk kimyonu tohumu",
            "Caraway seed",
        ),
        (
            "usda-sr-168921",
            "3d663734-ceaf-8fcf-945f-74298021320a",
            "Tam tahıllı mavi mısır unu",
            "Whole-grain blue corn flour",
        ),
        (
            "usda-sr-168588",
            "6d96fb7b-4669-8b54-ad19-b006e3fb2873",
            "Badem püresi (sade, tuz ilavesiz)",
            "Almond butter (plain, no salt added)",
        ),
    ];
    assert_eq!(old.manifest.version, 4);
    assert_eq!(new.manifest.version, 5);
    assert_eq!(new.data.ingredients.len(), 508);
    assert_eq!(
        new.data
            .ingredients
            .iter()
            .map(|i| i.aliases.len())
            .sum::<usize>(),
        260
    );
    for (key, id, name, english) in cases {
        let entry = new.data.ingredients.iter().find(|i| i.key == key).unwrap();
        assert_eq!(
            (
                entry.id.as_str(),
                entry.names["tr"].as_str(),
                entry.names["en"].as_str()
            ),
            (id, name, english)
        );
    }
    let fresh = tempfile::tempdir().unwrap();
    let mut fresh_db = Database::open(fresh.path()).unwrap();
    fresh_db.import_catalog(&new).unwrap();
    for (_, id, name, _) in cases {
        assert_eq!(fresh_db.ingredients(name).unwrap()[0].id, id);
    }
    assert_eq!(count(&fresh_db, "catalog_ingredients"), 508);
    assert_eq!(count(&fresh_db, "catalog_aliases"), 260);
    drop(fresh_db);
    let dir = tempfile::tempdir().unwrap();
    let mut db = Database::open(dir.path()).unwrap();
    db.import_catalog(&old).unwrap();
    let personal = db.create_ingredient("Ev baharatı").unwrap();
    let butter = cases[2].1;
    let saved = db
        .save_recipe(recipe(&db.ingredients("Almond butter").unwrap().remove(0)))
        .unwrap();
    let caraway = cases[0].1;
    db.customize_catalog_ingredient(IngredientEdit {
        id: caraway.into(),
        revision: 1,
        name: "Benim Frenk kimyonum".into(),
        notes: Some("Kişisel not".into()),
        preferred_unit: Some("kg".into()),
        category_ids: None,
    })
    .unwrap();
    db.set_ingredient_favorite(caraway, "catalog", true)
        .unwrap();
    let revision = db
        .ingredient_detail(caraway, "catalog")
        .unwrap()
        .item
        .revision
        .unwrap();
    db.set_catalog_presentation(
        crate::domain::ingredient_library::CatalogPresentationInput {
            id: caraway.into(),
            revision,
            name: Some("Benim Frenk kimyonum".into()),
            alias: Some("Aile kimyonu".into()),
            preferred_unit: Some("kg".into()),
            notes: Some("Kişisel not".into()),
        },
    )
    .unwrap();
    let report = db.import_catalog(&new).unwrap();
    assert_eq!((report.inserted, report.updated), (0, 3));
    assert!(db.import_catalog(&new).unwrap().unchanged);
    assert_eq!(db.ingredients("Ev baharatı").unwrap()[0].id, personal.id);
    assert_eq!(
        db.ingredients("Benim Frenk kimyonum").unwrap()[0].id,
        caraway
    );
    assert_eq!(
        db.available_ingredients("Aile kimyonu").unwrap().items[0].id,
        caraway
    );
    assert!(db
        .ingredient_library(crate::domain::ingredient_library::LibraryQuery {
            view: Some("favorites".into()),
            origin: "catalog".into(),
            search: String::new(),
            category_id: None,
            uncategorized: false,
            offset: 0,
            limit: 30,
        })
        .unwrap()
        .items
        .iter()
        .any(|i| i.id == caraway));
    for (_, id, name, english) in cases {
        assert_eq!(
            db.conn
                .query_row(
                    "SELECT name FROM catalog_names WHERE ingredient_id=?1 AND locale='tr'",
                    [id],
                    |r| r.get::<_, String>(0)
                )
                .unwrap(),
            name
        );
        assert!(db
            .available_ingredients(name)
            .unwrap()
            .items
            .iter()
            .any(|i| i.id == id));
        assert!(db
            .available_ingredients(english)
            .unwrap()
            .items
            .iter()
            .any(|i| i.id == id));
    }
    assert!(db
        .available_ingredients("Frenk kimyonu")
        .unwrap()
        .items
        .iter()
        .any(|i| i.id == caraway));
    assert_eq!(
        db.conn
            .query_row(
                "SELECT name,notes,preferred_unit FROM ingredients WHERE id=?1",
                [caraway],
                |r| Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?
                ))
            )
            .unwrap(),
        (
            "Benim Frenk kimyonum".into(),
            "Kişisel not".into(),
            "kg".into()
        )
    );
    assert!(!db
        .available_ingredients("Tuz eklenmemiş sade badem ezmesi")
        .unwrap()
        .items
        .iter()
        .any(|i| i.id == butter));
    assert_ne!(butter, stable_id("ingredient", "usda-sr-170160"));
    drop(db);
    let mut db = Database::open(dir.path()).unwrap();
    assert!(db.import_catalog(&new).unwrap().unchanged);
    let retained = db.recipe(&saved.id).unwrap();
    assert_eq!(retained.ingredients[0].ingredient_id, butter);
    assert_eq!(
        (
            retained.ingredients[0].quantity.as_deref(),
            retained.ingredients[0].unit_code.as_str()
        ),
        (Some("35.000001"), "cc")
    );
    assert!(db
        .available_ingredients_mode("", "recent")
        .unwrap()
        .items
        .iter()
        .any(|i| i.id == butter));
    assert_eq!(
        db.conn
            .query_row("PRAGMA integrity_check", [], |r| r.get::<_, String>(0))
            .unwrap(),
        "ok"
    );
    assert_eq!(
        db.conn
            .query_row("SELECT count(*) FROM pragma_foreign_key_check", [], |r| r
                .get::<_, i64>(
                0
            ))
            .unwrap(),
        0
    );
}

#[test]
fn terminology_v5_name_collision_rolls_back_without_changing_personal_or_recipe_data() {
    let old = catalog_tool::load(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../catalog/releases/4/manifest.json"),
    )
    .unwrap();
    let dir = tempfile::tempdir().unwrap();
    let mut db = Database::open(dir.path()).unwrap();
    db.import_catalog(&old).unwrap();
    let personal = db.create_ingredient("Frenk kimyonu tohumu").unwrap();
    let saved = db.save_recipe(recipe(&personal)).unwrap();
    assert_eq!(
        db.import_catalog(
            &catalog_tool::load(
                &Path::new(env!("CARGO_MANIFEST_DIR")).join("../catalog/releases/5/manifest.json"),
            )
            .unwrap()
        )
        .unwrap_err()
        .code,
        "CATALOG_NAME_CONFLICT"
    );
    assert_eq!(count(&db, "catalog_releases"), 1);
    assert_eq!(
        db.ingredients("Frenk kimyonu tohumu").unwrap()[0].id,
        personal.id
    );
    assert_eq!(
        db.recipe(&saved.id).unwrap().ingredients[0].ingredient_id,
        personal.id
    );
    assert_eq!(
        db.conn
            .query_row(
                "SELECT name FROM catalog_names WHERE ingredient_id=?1 AND locale='tr'",
                ["8f3055ee-2f8e-8300-87a4-2755af76b4e6"],
                |r| r.get::<_, String>(0)
            )
            .unwrap(),
        "Karaman kimyonu tohumu"
    );
    assert_eq!(
        db.conn
            .query_row("PRAGMA integrity_check", [], |r| r.get::<_, String>(0))
            .unwrap(),
        "ok"
    );
}

#[test]
fn terminology_v6_fresh_and_v5_upgrade_preserve_users_recipes_and_search() {
    let old = catalog_tool::load(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../catalog/releases/5/manifest.json"),
    )
    .unwrap();
    let new = catalog_tool::load(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../catalog/releases/6/manifest.json"),
    )
    .unwrap();
    assert_eq!((old.manifest.version, new.manifest.version), (5, 6));
    let changes = [
        (
            "usda-sr-174842",
            "b824153c-c098-8309-ac77-8538c4fb3c4c",
            "Club soda (kokteyl sodası)",
            "Club soda",
        ),
        (
            "project-siyez-grain",
            "ee71c0f4-6d4f-858c-8ae0-32ccb416ea9e",
            "Siyez buğdayı",
            "Einkorn wheat (siyez)",
        ),
    ];
    for (key, id, tr, en) in changes {
        let record = new.data.ingredients.iter().find(|i| i.key == key).unwrap();
        assert_eq!(
            (
                &record.id[..],
                &record.names["tr"][..],
                &record.names["en"][..]
            ),
            (id, tr, en)
        );
        if key == "project-siyez-grain" {
            assert_eq!(record.canonical_name, en);
        }
    }
    let fresh = tempfile::tempdir().unwrap();
    let mut db = Database::open(fresh.path()).unwrap();
    db.import_catalog(&new).unwrap();
    assert_eq!(
        (
            count(&db, "catalog_ingredients"),
            count(&db, "catalog_aliases"),
            count(&db, "ingredient_categories")
        ),
        (508, 260, 50)
    );
    for (_, id, tr, en) in changes {
        for name in [tr, en] {
            assert!(db
                .available_ingredients(name)
                .unwrap()
                .items
                .iter()
                .any(|i| i.id == id));
        }
    }
    assert!(db
        .available_ingredients("KOKTEYL SODASI")
        .unwrap()
        .items
        .iter()
        .any(|i| i.id == changes[0].1));
    assert!(db
        .available_ingredients("einkorn wheat")
        .unwrap()
        .items
        .iter()
        .any(|i| i.id == changes[1].1));
    drop(db);

    let upgrade = tempfile::tempdir().unwrap();
    let mut db = Database::open(upgrade.path()).unwrap();
    db.import_catalog(&old).unwrap();
    let personal = db.create_ingredient("Aile aroması").unwrap();
    db.set_ingredient_favorite(&personal.id, "personal", true)
        .unwrap();
    let club = db
        .ingredients("Club soda")
        .unwrap()
        .into_iter()
        .find(|i| i.id == changes[0].1)
        .unwrap();
    let saved = db.save_recipe(recipe(&club)).unwrap();
    let draft = db
        .save_draft(DraftWrite {
            id: uuid::Uuid::new_v4().to_string(),
            expected_revision: None,
            input: recipe(&personal),
        })
        .unwrap();
    db.set_ingredient_favorite(&club.id, "catalog", true)
        .unwrap();
    let revision = db
        .ingredient_detail(&club.id, "catalog")
        .unwrap()
        .item
        .revision
        .unwrap();
    db.set_catalog_presentation(
        crate::domain::ingredient_library::CatalogPresentationInput {
            id: club.id.clone(),
            revision,
            name: Some("Kendi kokteyl sodam".into()),
            alias: Some("Ev sodası".into()),
            preferred_unit: Some("cc".into()),
            notes: Some("Özel".into()),
        },
    )
    .unwrap();
    let report = db.import_catalog(&new).unwrap();
    assert_eq!(
        (report.inserted, report.updated, report.validated),
        (0, 2, 508)
    );
    assert!(db.import_catalog(&new).unwrap().unchanged);
    assert_eq!(
        db.ingredient_detail(&club.id, "catalog")
            .unwrap()
            .canonical_tr
            .as_deref(),
        Some("Club soda (kokteyl sodası)")
    );
    assert_eq!(
        db.ingredients("Kendi kokteyl sodam").unwrap()[0].id,
        club.id
    );
    assert!(db
        .available_ingredients("Ev sodası")
        .unwrap()
        .items
        .iter()
        .any(|i| i.id == club.id));
    assert_eq!(db.ingredients("Aile aroması").unwrap()[0].id, personal.id);
    assert_eq!(
        db.draft(&draft.id).unwrap().input.ingredients[0].ingredient_id,
        personal.id
    );
    for (id, origin) in [(&club.id[..], "catalog"), (&personal.id[..], "personal")] {
        assert!(db
            .ingredient_library(crate::domain::ingredient_library::LibraryQuery {
                view: Some("favorites".into()),
                origin: origin.into(),
                search: String::new(),
                category_id: None,
                uncategorized: false,
                offset: 0,
                limit: 30,
            })
            .unwrap()
            .items
            .iter()
            .any(|i| i.id == id));
    }
    assert!(db
        .available_ingredients_mode("", "recent")
        .unwrap()
        .items
        .iter()
        .any(|i| i.id == club.id));
    for (key, name) in [
        ("usda-sr-171329", "Paprika"),
        ("usda-sr-170927", "Öğütülmüş muskat çiçeği"),
        ("usda-sr-170936", "Öğütülmüş sater otu"),
        ("usda-sr-171318", "Kuru Frenk maydanozu"),
        ("usda-sr-170160", "Badem ezmesi (pastacılık)"),
        ("usda-sr-170903", "Sade az yağlı Yunan tipi yoğurt"),
        ("usda-sr-175142", "Levrek (karışık türler, çiğ)"),
        ("project-salep-drink", "Salep (hazırlanmış içecek)"),
        ("project-aromatic-bitters", "Aromatik kokteyl bitteri"),
        ("project-creme-de-cassis", "Crème de cassis"),
        ("usda-sr-171319", "Chili tozu"),
        ("usda-sr-170874", "Fermente az yağlı buttermilk"),
        ("usda-sr-168412", "Hindiba (endivyen, çiğ)"),
    ] {
        assert_eq!(db.conn.query_row("SELECT n.name FROM catalog_names n JOIN catalog_ingredients i ON i.id=n.ingredient_id WHERE i.canonical_key=?1 AND n.locale='tr'", [key], |r| r.get::<_, String>(0)).unwrap(), name);
    }
    drop(db);
    let mut db = Database::open(upgrade.path()).unwrap();
    let runs = count(&db, "catalog_import_runs");
    assert!(db.import_catalog(&new).unwrap().unchanged);
    assert_eq!(count(&db, "catalog_import_runs"), runs + 1);
    let retained = db.recipe(&saved.id).unwrap();
    assert_eq!(
        (
            retained.ingredients[0].ingredient_id.as_str(),
            retained.ingredients[0].quantity.as_deref(),
            retained.ingredients[0].unit_code.as_str()
        ),
        (club.id.as_str(), Some("35.000001"), "cc")
    );
    assert_eq!(
        db.conn
            .query_row("PRAGMA integrity_check", [], |r| r.get::<_, String>(0))
            .unwrap(),
        "ok"
    );
    assert_eq!(
        db.conn
            .query_row("SELECT count(*) FROM pragma_foreign_key_check", [], |r| r
                .get::<_, i64>(
                0
            ))
            .unwrap(),
        0
    );
}

#[test]
fn terminology_v6_new_name_collision_rolls_back_release() {
    let old = catalog_tool::load(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../catalog/releases/5/manifest.json"),
    )
    .unwrap();
    let dir = tempfile::tempdir().unwrap();
    let mut db = Database::open(dir.path()).unwrap();
    db.import_catalog(&old).unwrap();
    let personal = db.create_ingredient("Club soda (kokteyl sodası)").unwrap();
    let saved = db.save_recipe(recipe(&personal)).unwrap();
    assert_eq!(
        db.import_catalog(&catalog_tool::bundled().unwrap())
            .unwrap_err()
            .code,
        "CATALOG_NAME_CONFLICT"
    );
    assert_eq!(count(&db, "catalog_releases"), 1);
    assert_eq!(
        db.ingredients("Club soda (kokteyl sodası)").unwrap()[0].id,
        personal.id
    );
    assert_eq!(
        db.recipe(&saved.id).unwrap().ingredients[0].ingredient_id,
        personal.id
    );
    assert_eq!(
        db.conn
            .query_row(
                "SELECT name FROM catalog_names WHERE ingredient_id=?1 AND locale='tr'",
                ["b824153c-c098-8309-ac77-8538c4fb3c4c"],
                |r| r.get::<_, String>(0)
            )
            .unwrap(),
        "Soda (club soda)"
    );
    assert_eq!(
        db.conn
            .query_row("PRAGMA integrity_check", [], |r| r.get::<_, String>(0))
            .unwrap(),
        "ok"
    );
}

fn verify_historical_upgrade(version: u32, previous_count: usize, collision_name: &str) {
    let old = catalog_tool::load(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(format!("../catalog/releases/{version}/manifest.json")),
    )
    .unwrap();
    assert_eq!(old.data.ingredients.len(), previous_count);
    let dir = tempfile::tempdir().unwrap();
    let mut db = Database::open(dir.path()).unwrap();
    db.import_catalog(&old).unwrap();
    let personal = db.create_ingredient("Aile baharatım").unwrap();
    let collision = db.create_ingredient(collision_name).unwrap();
    let custom_recipe = db.save_recipe(recipe(&personal)).unwrap();
    let honey = db.ingredients("Honey").unwrap().remove(0);
    let saved = db.save_recipe(recipe(&honey)).unwrap();
    let draft = db
        .save_draft(DraftWrite {
            id: uuid::Uuid::new_v4().to_string(),
            expected_revision: None,
            input: recipe(&honey),
        })
        .unwrap();
    db.customize_catalog_ingredient(IngredientEdit {
        id: honey.id.clone(),
        revision: 1,
        name: "Özel aile balım".into(),
        notes: Some("Değişmesin".into()),
        preferred_unit: Some("kg".into()),
        category_ids: None,
    })
    .unwrap();
    let upgraded = catalog_tool::bundled().unwrap();
    let start = std::time::Instant::now();
    let report = db.import_catalog(&upgraded).unwrap();
    let import_time = start.elapsed();
    assert_eq!(
        (
            report.validated,
            report.inserted,
            report.updated,
            report.collisions
        ),
        (
            854,
            854 - previous_count,
            if version == 4 {
                5
            } else if version == 6 {
                0
            } else if version == 5 {
                2
            } else {
                previous_count
            },
            if version >= 4 { 0 } else { 1 }
        )
    );
    for item in &old.data.ingredients {
        let id: String = db
            .conn
            .query_row(
                "SELECT id FROM catalog_ingredients WHERE canonical_key=?1",
                [&item.key],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(id, item.id);
    }
    assert_eq!(
        db.recipe(&custom_recipe.id).unwrap().ingredients[0].ingredient_id,
        personal.id
    );
    assert_eq!(
        db.recipe(&saved.id).unwrap().ingredients[0].ingredient_id,
        honey.id
    );
    assert_eq!(
        db.draft(&draft.id).unwrap().input.ingredients[0].ingredient_id,
        honey.id
    );
    assert_eq!(
        db.conn
            .query_row(
                "SELECT preferred_unit FROM ingredients WHERE id=?1",
                [&honey.id],
                |r| r.get::<_, String>(0)
            )
            .unwrap(),
        "kg"
    );
    assert_eq!(
        db.ingredients(collision_name)
            .unwrap()
            .iter()
            .find(|i| i.id == collision.id)
            .unwrap()
            .name,
        collision_name
    );
    let runs = count(&db, "catalog_import_runs");
    let changes: i64 = db
        .conn
        .query_row("SELECT total_changes()", [], |r| r.get(0))
        .unwrap();
    db.install_bundled_catalog().unwrap();
    assert_eq!(
        db.conn
            .query_row("SELECT total_changes()", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        changes
    );
    assert_eq!(count(&db, "catalog_import_runs"), runs);
    drop(db);
    let mut db = Database::open(dir.path()).unwrap();
    db.install_bundled_catalog().unwrap();
    assert_eq!(
        db.recipe(&saved.id).unwrap().ingredients[0]
            .quantity
            .as_deref(),
        Some("35.000001")
    );
    assert_eq!(
        db.catalog_status().unwrap().pending_collisions,
        if version >= 4 { 0 } else { 1 }
    );
    assert_eq!(
        db.conn
            .query_row("PRAGMA integrity_check", [], |r| r.get::<_, String>(0))
            .unwrap(),
        "ok"
    );
    assert_eq!(
        db.conn
            .query_row("SELECT count(*) FROM pragma_foreign_key_check", [], |r| r
                .get::<_, i64>(
                0
            ))
            .unwrap(),
        0
    );
    eprintln!("actual v{version}->v7 import: {import_time:?}");
}

#[test]
fn expanded_catalog_search_forms_aliases_source_boundaries_and_real_performance() {
    let dir = tempfile::tempdir().unwrap();
    let mut db = Database::open(dir.path()).unwrap();
    db.install_bundled_catalog().unwrap();
    for (key, queries) in [
        (
            "project-siyez-grain",
            vec!["SİYEZ BUĞ", "siyez buğ", "Einkorn wheat"],
        ),
        ("project-raki", vec!["RAKI", "rakı", "Rakı"]),
        ("project-india-pale-ale", vec!["IPA", "India pale ale"]),
        ("project-orgeat-syrup", vec!["orgeat", "Orgeat almond"]),
        ("project-nar-eksisi", vec!["NAR EKŞİSİ", "nar ekşisi"]),
        (
            "project-fine-koftelik-bulgur",
            vec!["ÇİĞ KÖFTELİK", "fine bulgur"],
        ),
        ("usda-sr-169236", vec!["Yer elması", "Jerusalem artichoke"]),
        (
            "project-creme-de-cassis",
            vec!["Siyah frenk üzümü likörü", "Crème de cassis"],
        ),
        ("usda-sr-170918", vec!["Frenk kimyonu", "caraway"]),
    ] {
        for q in queries {
            let found = db.available_ingredients(q).unwrap();
            assert_eq!(
                found
                    .items
                    .iter()
                    .filter(|i| i.id == stable_id("ingredient", key))
                    .count(),
                1,
                "{q}"
            );
        }
    }
    assert_ne!(
        db.ingredients("Salep tozu").unwrap()[0].id,
        db.ingredients("Salep (hazırlanmış").unwrap()[0].id
    );
    for key in ["usda-sr-174133", "usda-sr-173230"] {
        assert_eq!(
            db.conn
                .query_row(
                    "SELECT preferred_unit FROM ingredients WHERE id=?1",
                    [stable_id("ingredient", key)],
                    |r| r.get::<_, String>(0)
                )
                .unwrap(),
            "g"
        );
    }
    let seed = catalog_tool::bundled().unwrap();
    assert_eq!(
        seed.data
            .ingredients
            .iter()
            .filter(|i| i.key.starts_with("project-"))
            .count(),
        54
    );
    for item in seed
        .data
        .ingredients
        .iter()
        .filter(|i| i.key.starts_with("project-"))
    {
        assert_eq!(
            item.provenance.len(),
            if item.key == "project-siyez-grain" {
                2
            } else {
                1
            }
        );
        assert_eq!(item.provenance[0].source_id, "recipeatlas-curation");
        assert!(item.provenance[0].description.contains("https://"));
    }
    let start = std::time::Instant::now();
    for _ in 0..100 {
        db.available_ingredients("şek").unwrap();
    }
    let elapsed = start.elapsed();
    eprintln!("100 real SQLite Turkish partial searches: {elapsed:?}");
    assert!(
        elapsed < std::time::Duration::from_secs(10),
        "Gross search regression: {elapsed:?}"
    );
}

#[test]
fn failed_actual_v2_upgrade_keeps_previous_release_and_retries_atomically() {
    let dir = tempfile::tempdir().unwrap();
    let mut db = Database::open(dir.path()).unwrap();
    let old = catalog_tool::load(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../catalog/releases/2/manifest.json"),
    )
    .unwrap();
    db.import_catalog(&old).unwrap();
    db.conn.execute_batch("CREATE TRIGGER fail_expansion BEFORE INSERT ON catalog_ingredients WHEN NEW.canonical_key='project-gin' BEGIN SELECT RAISE(ABORT,'upgrade interrupted'); END;").unwrap();
    assert!(db.install_bundled_catalog().is_err());
    assert_eq!(count(&db, "catalog_ingredients"), 235);
    assert_eq!(count(&db, "catalog_releases"), 1);
    assert_eq!(db.ingredients("Honey").unwrap()[0].name, "Bal");
    db.conn
        .execute_batch("DROP TRIGGER fail_expansion")
        .unwrap();
    drop(db);
    let mut db = Database::open(dir.path()).unwrap();
    db.install_bundled_catalog().unwrap();
    assert_eq!(count(&db, "catalog_ingredients"), 854);
    assert_eq!(count(&db, "catalog_releases"), 2);
}

#[test]
fn m3d1_expansion_preserves_v6_and_rolls_back_before_retry_with_real_search() {
    let old = catalog_tool::load(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../catalog/releases/6/manifest.json"),
    )
    .unwrap();
    let new = catalog_tool::load(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../catalog/releases/7/manifest.json"),
    )
    .unwrap();
    assert_eq!(new.manifest.version, 7);
    for previous in &old.data.ingredients {
        let current = new
            .data
            .ingredients
            .iter()
            .find(|i| i.id == previous.id)
            .unwrap();
        assert_eq!(
            serde_json::to_value(previous).unwrap(),
            serde_json::to_value(current).unwrap()
        );
    }
    let additions: Vec<_> = new
        .data
        .ingredients
        .iter()
        .filter(|i| !old.data.ingredients.iter().any(|p| p.id == i.id))
        .collect();
    assert_eq!(additions.len(), 60);
    assert_eq!(
        new.data
            .ingredients
            .iter()
            .map(|i| i.aliases.len())
            .sum::<usize>(),
        269
    );
    let dir = tempfile::tempdir().unwrap();
    let mut db = Database::open(dir.path()).unwrap();
    db.import_catalog(&old).unwrap();
    let personal = db.create_ingredient("M3D aile çeşnisi").unwrap();
    let honey = db.ingredients("Honey").unwrap().remove(0);
    db.set_ingredient_favorite(&personal.id, "personal", true)
        .unwrap();
    db.set_ingredient_favorite(&honey.id, "catalog", true)
        .unwrap();
    let saved = db.save_recipe(recipe(&personal)).unwrap();
    let honey_recipe = db.save_recipe(recipe(&honey)).unwrap();
    let draft = db
        .save_draft(DraftWrite {
            id: uuid::Uuid::new_v4().to_string(),
            expected_revision: None,
            input: recipe(&personal),
        })
        .unwrap();
    db.set_catalog_presentation(
        crate::domain::ingredient_library::CatalogPresentationInput {
            id: honey.id.clone(),
            revision: 1,
            name: Some("M3D özel bal".into()),
            alias: Some("M3D aile balı".into()),
            preferred_unit: Some("kg".into()),
            notes: Some("Korunsun".into()),
        },
    )
    .unwrap();
    let source_count = count(&db, "catalog_sources");
    db.conn.execute_batch("CREATE TRIGGER fail_m3d1 BEFORE INSERT ON catalog_ingredients WHEN NEW.canonical_key='usda-sr-170283' BEGIN SELECT RAISE(ABORT,'forced expansion failure'); END;").unwrap();
    assert!(db.import_catalog(&new).is_err());
    assert_eq!(count(&db, "catalog_ingredients"), 508);
    assert_eq!(count(&db, "catalog_aliases"), 260);
    assert_eq!(count(&db, "catalog_sources"), source_count);
    assert_eq!(count(&db, "catalog_releases"), 1);
    assert_eq!(
        db.recipe(&saved.id).unwrap().ingredients[0].ingredient_id,
        personal.id
    );
    db.conn.execute_batch("DROP TRIGGER fail_m3d1").unwrap();
    let report = db.import_catalog(&new).unwrap();
    assert_eq!(
        (report.inserted, report.updated, report.collisions),
        (60, 0, 0)
    );
    assert!(db.import_catalog(&new).unwrap().unchanged);
    let start = std::time::Instant::now();
    for item in additions {
        assert_eq!(item.id, stable_id("ingredient", &item.key));
        for name in item
            .names
            .values()
            .chain(item.aliases.iter().map(|a| &a.name))
        {
            assert!(
                db.available_ingredients(name)
                    .unwrap()
                    .items
                    .iter()
                    .any(|i| i.id == item.id),
                "Missing search: {name}"
            );
        }
        let detail = db.ingredient_detail(&item.id, "catalog").unwrap();
        assert_eq!(detail.sources.len(), 2);
        assert!(detail.metadata.is_empty());
        assert_eq!(
            detail.canonical_tr.as_deref(),
            Some(item.names["tr"].as_str())
        );
        let page = db
            .ingredient_library(crate::domain::ingredient_library::LibraryQuery {
                search: item.names["tr"].clone(),
                origin: "catalog".into(),
                category_id: Some(stable_id("category", &item.categories[0])),
                offset: 0,
                limit: 30,
                view: None,
                uncategorized: false,
            })
            .unwrap();
        assert!(page.items.iter().any(|i| i.id == item.id));
    }
    for q in [
        "ŞEKERSİZ TURNA",
        "şekersiz turna",
        "S\u{0327}EKERSİZ TURNA",
        "KIRMI",
        "kırmı",
    ] {
        assert!(
            !db.available_ingredients(q).unwrap().items.is_empty(),
            "Missing Turkish search: {q}"
        );
    }
    eprintln!(
        "60 new identity name/alias/detail/category checks: {:?}",
        start.elapsed()
    );
    let new_ingredient = db.ingredients("Enoki mushroom").unwrap().remove(0);
    let mut expanded_input = recipe(&new_ingredient);
    expanded_input.ingredients[0].unit_code = "g".into();
    let expanded_recipe = db.save_recipe(expanded_input).unwrap();
    drop(db);
    let mut db = Database::open(dir.path()).unwrap();
    db.install_bundled_catalog().unwrap();
    assert_eq!(
        (
            count(&db, "catalog_ingredients"),
            count(&db, "catalog_aliases"),
            count(&db, "ingredient_categories")
        ),
        (854, 269, 50)
    );
    for (id, ingredient_id, unit) in [
        (&saved.id, &personal.id, "cc"),
        (&honey_recipe.id, &honey.id, "cc"),
        (&expanded_recipe.id, &new_ingredient.id, "g"),
    ] {
        let retained = db.recipe(id).unwrap();
        assert_eq!(&retained.ingredients[0].ingredient_id, ingredient_id);
        assert_eq!(
            retained.ingredients[0].quantity.as_deref(),
            Some("35.000001")
        );
        assert_eq!(retained.ingredients[0].unit_code, unit);
    }
    assert!(db.drafts().unwrap().iter().any(|d| d.id == draft.id));
    assert!(
        db.ingredient_detail(&personal.id, "personal")
            .unwrap()
            .item
            .favorite
    );
    let retained = db.ingredient_detail(&honey.id, "catalog").unwrap();
    assert!(retained.item.favorite && retained.item.last_used_at.is_some());
    assert_eq!(retained.item.name, "M3D özel bal");
    assert_eq!(retained.personal_alias.as_deref(), Some("M3D aile balı"));
    assert_eq!(retained.item.preferred_unit.as_deref(), Some("kg"));
    assert_eq!(
        db.conn
            .query_row("PRAGMA integrity_check", [], |r| r.get::<_, String>(0))
            .unwrap(),
        "ok"
    );
    assert_eq!(
        db.conn
            .query_row("SELECT count(*) FROM pragma_foreign_key_check", [], |r| r
                .get::<_, i64>(
                0
            ))
            .unwrap(),
        0
    );
}

#[test]
fn m3d2b_release8_upgrades_v7_atomically_and_preserves_user_state() {
    let old = catalog_tool::load(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../catalog/releases/7/manifest.json"),
    )
    .unwrap();
    let new = catalog_tool::load(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../catalog/releases/8/manifest.json"),
    )
    .unwrap();
    assert_eq!(new.manifest.version, 8);
    assert_eq!(old.data.ingredients.len(), 568);
    assert_eq!(new.data.ingredients.len(), 647);
    for previous in &old.data.ingredients {
        let current = new
            .data
            .ingredients
            .iter()
            .find(|item| item.id == previous.id)
            .unwrap();
        let mut current = current.clone();
        for provenance in &mut current.provenance {
            match provenance.source_id.as_str() {
                "usda-sr-legacy-m3d1" => {
                    assert_eq!(provenance.source_version, "2018-04-selection-5-reviewed");
                    provenance.source_version = "2018-04-selection-5".into();
                }
                "recipeatlas-curation-m3d1" => {
                    assert_eq!(provenance.source_version, "production-4-reviewed");
                    provenance.source_version = "production-4".into();
                }
                _ => {}
            }
        }
        assert_eq!(
            serde_json::to_value(previous).unwrap(),
            serde_json::to_value(current).unwrap()
        );
    }
    let additions: Vec<_> = new
        .data
        .ingredients
        .iter()
        .filter(|item| !old.data.ingredients.iter().any(|prior| prior.id == item.id))
        .collect();
    assert_eq!(additions.len(), 79);
    assert!(additions.iter().all(|item| {
        item.id == stable_id("ingredient", &item.key)
            && item.aliases.is_empty()
            && item.observations.is_empty()
            && item.provenance.len() == 2
            && item.provenance[0].source_id == "usda-sr-legacy-m3d2b"
    }));
    let dir = tempfile::tempdir().unwrap();
    let mut db = Database::open(dir.path()).unwrap();
    db.import_catalog(&old).unwrap();
    let personal = db.create_ingredient("Aile M3D-2B çeşnisi").unwrap();
    db.set_ingredient_favorite(&personal.id, "personal", true)
        .unwrap();
    let honey = db.ingredients("Honey").unwrap().remove(0);
    db.set_ingredient_favorite(&honey.id, "catalog", true)
        .unwrap();
    let saved = db.save_recipe(recipe(&personal)).unwrap();
    let draft = db
        .save_draft(DraftWrite {
            id: uuid::Uuid::new_v4().to_string(),
            expected_revision: None,
            input: recipe(&honey),
        })
        .unwrap();
    db.set_catalog_presentation(
        crate::domain::ingredient_library::CatalogPresentationInput {
            id: honey.id.clone(),
            revision: 1,
            name: Some("Aile balı M3D-2B".into()),
            alias: Some("Özel bal araması".into()),
            preferred_unit: Some("kg".into()),
            notes: Some("Sakla".into()),
        },
    )
    .unwrap();
    let source_count = count(&db, "catalog_sources");
    db.conn.execute_batch("CREATE TRIGGER fail_release8 BEFORE INSERT ON catalog_ingredients WHEN NEW.canonical_key='usda-sr-167714' BEGIN SELECT RAISE(ABORT,'forced release 8 failure'); END;").unwrap();
    assert!(db.import_catalog(&new).is_err());
    assert_eq!(count(&db, "catalog_ingredients"), 568);
    assert_eq!(count(&db, "catalog_releases"), 1);
    assert_eq!(count(&db, "catalog_sources"), source_count);
    assert_eq!(
        db.recipe(&saved.id).unwrap().ingredients[0].ingredient_id,
        personal.id
    );
    db.conn.execute_batch("DROP TRIGGER fail_release8").unwrap();
    let report = db.import_catalog(&new).unwrap();
    assert_eq!(
        (report.inserted, report.updated, report.collisions),
        (79, 60, 0)
    );
    assert!(db.import_catalog(&new).unwrap().unchanged);
    for item in additions {
        for name in item.names.values() {
            assert!(
                db.available_ingredients(name)
                    .unwrap()
                    .items
                    .iter()
                    .any(|found| found.id == item.id),
                "Missing bilingual search: {name}"
            );
        }
        let detail = db.ingredient_detail(&item.id, "catalog").unwrap();
        assert_eq!(detail.sources.len(), 2);
        assert!(detail.metadata.is_empty());
        let page = db
            .ingredient_library(crate::domain::ingredient_library::LibraryQuery {
                search: item.names["tr"].clone(),
                origin: "catalog".into(),
                category_id: Some(stable_id("category", &item.categories[0])),
                offset: 0,
                limit: 30,
                view: None,
                uncategorized: false,
            })
            .unwrap();
        assert!(page.items.iter().any(|found| found.id == item.id));
    }
    for query in ["KUDRET", "kudret", "Kudret", "PUMMELO", "pummelo"] {
        assert!(!db.available_ingredients(query).unwrap().items.is_empty());
    }
    let palm = db.ingredients("Palmiye göbeği").unwrap().remove(0);
    let mut input = recipe(&palm);
    input.ingredients[0].quantity = Some("0.123456".into());
    input.ingredients[0].unit_code = "g".into();
    let new_recipe = db.save_recipe(input).unwrap();
    drop(db);
    let mut db = Database::open(dir.path()).unwrap();
    db.install_bundled_catalog().unwrap();
    assert_eq!(
        (
            count(&db, "catalog_ingredients"),
            count(&db, "catalog_aliases"),
            count(&db, "ingredient_categories")
        ),
        (854, 269, 50)
    );
    assert_eq!(count(&db, "catalog_sources"), source_count + 12);
    let kept = db.recipe(&saved.id).unwrap();
    assert_eq!(kept.ingredients[0].ingredient_id, personal.id);
    assert_eq!(kept.ingredients[0].quantity.as_deref(), Some("35.000001"));
    assert_eq!(kept.ingredients[0].unit_code, "cc");
    assert_eq!(
        db.draft(&draft.id).unwrap().input.ingredients[0].ingredient_id,
        honey.id
    );
    let new_saved = db.recipe(&new_recipe.id).unwrap();
    assert_eq!(new_saved.ingredients[0].ingredient_id, palm.id);
    assert_eq!(
        new_saved.ingredients[0].quantity.as_deref(),
        Some("0.123456")
    );
    assert_eq!(new_saved.ingredients[0].unit_code, "g");
    assert!(
        db.ingredient_detail(&personal.id, "personal")
            .unwrap()
            .item
            .favorite
    );
    let honey_detail = db.ingredient_detail(&honey.id, "catalog").unwrap();
    assert_eq!(honey_detail.item.name, "Aile balı M3D-2B");
    assert!(honey_detail.item.favorite);
    assert!(honey_detail.item.last_used_at.is_none());
    assert_eq!(
        honey_detail.personal_alias.as_deref(),
        Some("Özel bal araması")
    );
    assert_eq!(honey_detail.item.preferred_unit.as_deref(), Some("kg"));
    assert_eq!(
        db.conn
            .query_row("PRAGMA integrity_check", [], |r| r.get::<_, String>(0))
            .unwrap(),
        "ok"
    );
    assert_eq!(
        db.conn
            .query_row("SELECT count(*) FROM pragma_foreign_key_check", [], |r| r
                .get::<_, i64>(
                0
            ))
            .unwrap(),
        0
    );
}

#[test]
fn release8_accepts_earlier_m3d1_snapshot_without_rewriting_its_provenance() {
    let old = catalog_tool::load(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../catalog/releases/7/manifest.json"),
    )
    .unwrap();
    let dir = tempfile::tempdir().unwrap();
    let mut db = Database::open(dir.path()).unwrap();
    db.import_catalog(&old).unwrap();
    // A locally installed 61-record M3D-1 preview used these same source/version
    // pairs with different evidence. Preserve those immutable snapshots.
    let preview = [
        (
            "usda-sr-legacy-m3d1",
            "2018-04-selection-5",
            "f6a9b1d959554177f4ccde64cd75892fba1e0acfca7a17077e06668572ac5042",
        ),
        (
            "recipeatlas-curation-m3d1",
            "production-4",
            "fecf768a94cff14b0eb343e49e2376c7fedac8ef95c131eed8b289d8d35c11b4",
        ),
    ];
    for (source, version, hash) in preview {
        db.conn
            .execute(
                "UPDATE catalog_sources SET artifact_sha256=?3 WHERE id=?1 AND version=?2",
                rusqlite::params![source, version, hash],
            )
            .unwrap();
    }
    let report = db.install_bundled_catalog();
    assert!(
        report.is_ok(),
        "preview source snapshot blocked startup: {report:?}"
    );
    assert_eq!(count(&db, "catalog_ingredients"), 854);
    assert_eq!(count(&db, "catalog_aliases"), 269);
    for (source, version, hash) in preview {
        let kept: String = db
            .conn
            .query_row(
                "SELECT artifact_sha256 FROM catalog_sources WHERE id=?1 AND version=?2",
                rusqlite::params![source, version],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(kept, hash);
        let reviewed = format!("{version}-reviewed");
        assert_eq!(
            db.conn
                .query_row(
                    "SELECT count(*) FROM catalog_sources WHERE id=?1 AND version=?2",
                    rusqlite::params![source, reviewed],
                    |r| r.get::<_, i64>(0),
                )
                .unwrap(),
            1
        );
    }
    assert_eq!(
        db.conn
            .query_row("PRAGMA integrity_check", [], |r| r.get::<_, String>(0))
            .unwrap(),
        "ok"
    );
    assert_eq!(
        db.conn
            .query_row("SELECT count(*) FROM pragma_foreign_key_check", [], |r| r
                .get::<_, i64>(
                0
            ))
            .unwrap(),
        0
    );
}

#[test]
fn production_allowlist_rejects_foodon_source_before_database_open() {
    let dir = tempfile::tempdir().unwrap();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../catalog/production");
    let mut manifest: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(root.join("manifest.json")).unwrap())
            .unwrap();
    for artifact in manifest["evidence"]
        .as_object()
        .unwrap()
        .values()
        .chain(std::iter::once(&manifest["artifact"]))
    {
        let file = artifact["file"].as_str().unwrap();
        std::fs::copy(root.join(file), dir.path().join(file)).unwrap();
    }
    manifest["sources"]
        .as_array_mut()
        .unwrap()
        .last_mut()
        .unwrap()["id"] = serde_json::json!("foodon");
    let path = dir.path().join("manifest.json");
    std::fs::write(&path, serde_json::to_vec(&manifest).unwrap()).unwrap();
    let profile = dir.path().join("profile");
    assert_eq!(
        catalog_tool::import(&profile, &path, false)
            .unwrap_err()
            .code,
        "CATALOG_LICENSE"
    );
    assert!(!profile.exists());
}

#[test]
fn m3d3_release9_upgrades_v8_and_rejects_reused_source_version_atomically() {
    let previous = catalog_tool::load(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../catalog/releases/8/manifest.json"),
    )
    .unwrap();
    let current = catalog_tool::load(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../catalog/releases/9/manifest.json"),
    )
    .unwrap();
    assert_eq!(
        (previous.manifest.version, current.manifest.version),
        (8, 9)
    );
    assert_eq!(
        (
            previous.data.ingredients.len(),
            current.data.ingredients.len()
        ),
        (647, 745)
    );
    for old in &previous.data.ingredients {
        let now = current
            .data
            .ingredients
            .iter()
            .find(|item| item.id == old.id)
            .unwrap();
        assert_eq!(
            serde_json::to_value(old).unwrap(),
            serde_json::to_value(now).unwrap()
        );
    }
    let dir = tempfile::tempdir().unwrap();
    let mut db = Database::open(dir.path()).unwrap();
    db.import_catalog(&previous).unwrap();
    let own = db.create_ingredient("Aile için özel malzeme").unwrap();
    let saved = db.save_recipe(recipe(&own)).unwrap();
    let honey = db.ingredients("Honey").unwrap().remove(0);
    db.set_ingredient_favorite(&honey.id, "catalog", true)
        .unwrap();
    db.set_catalog_presentation(
        crate::domain::ingredient_library::CatalogPresentationInput {
            id: honey.id.clone(),
            revision: 1,
            name: Some("Özel bal adım".into()),
            alias: None,
            preferred_unit: Some("kg".into()),
            notes: None,
        },
    )
    .unwrap();
    let source = current
        .manifest
        .sources
        .iter()
        .find(|s| s.id == "usda-sr-legacy-m3d3")
        .unwrap();
    db.conn.execute("INSERT INTO catalog_sources(id,version,name,url,license,attribution,retrieved_at,artifact_sha256) VALUES(?1,?2,?3,?4,?5,?6,?7,?8)",
        rusqlite::params![source.id,source.version,source.name,source.url,source.license,source.attribution,source.retrieved_at,"0".repeat(64)]).unwrap();
    assert_eq!(
        db.import_catalog(&current).unwrap_err().code,
        "CATALOG_SOURCE_CHANGED"
    );
    assert_eq!(count(&db, "catalog_ingredients"), 647);
    assert_eq!(count(&db, "catalog_releases"), 1);
    assert_eq!(
        db.recipe(&saved.id).unwrap().ingredients[0]
            .quantity
            .as_deref(),
        Some("35.000001")
    );
    db.conn
        .execute(
            "DELETE FROM catalog_sources WHERE id=?1 AND version=?2",
            rusqlite::params![source.id, source.version],
        )
        .unwrap();
    let report = db.import_catalog(&current).unwrap();
    assert_eq!(
        (report.inserted, report.updated, report.collisions),
        (98, 0, 0)
    );
    assert!(db.import_catalog(&current).unwrap().unchanged);
    drop(db);
    let mut db = Database::open(dir.path()).unwrap();
    db.install_bundled_catalog().unwrap();
    assert_eq!(count(&db, "catalog_ingredients"), 854);
    assert_eq!(count(&db, "catalog_aliases"), 269);
    assert_eq!(
        db.recipe(&saved.id).unwrap().ingredients[0].ingredient_id,
        own.id
    );
    assert_eq!(db.recipe(&saved.id).unwrap().ingredients[0].unit_code, "cc");
    let detail = db.ingredient_detail(&honey.id, "catalog").unwrap();
    assert_eq!(detail.item.name, "Özel bal adım");
    assert!(detail.item.favorite);
    for name in [
        "Taze biber nanesi",
        "Fresh peppermint",
        "Konserve bakla",
        "Canned fava beans",
    ] {
        assert!(
            !db.available_ingredients(name).unwrap().items.is_empty(),
            "{name}"
        );
    }
    let search_started = std::time::Instant::now();
    for item in current.data.ingredients.iter().filter(|item| {
        !previous
            .data
            .ingredients
            .iter()
            .any(|old| old.id == item.id)
    }) {
        for name in item.names.values() {
            let page = db.available_ingredients(name).unwrap();
            assert!(page.items.iter().any(|found| found.id == item.id), "{name}");
            assert_eq!(
                page.items
                    .iter()
                    .map(|found| &found.id)
                    .collect::<std::collections::HashSet<_>>()
                    .len(),
                page.items.len(),
                "Duplicate search identity for {name}"
            );
        }
    }
    eprintln!(
        "M3D-3 196 bilingual lookups: {:?}",
        search_started.elapsed()
    );
    assert_eq!(
        db.conn
            .query_row("PRAGMA integrity_check", [], |r| r.get::<_, String>(0))
            .unwrap(),
        "ok"
    );
    assert_eq!(
        db.conn
            .query_row("SELECT count(*) FROM pragma_foreign_key_check", [], |r| r
                .get::<_, i64>(
                0
            ))
            .unwrap(),
        0
    );
}

#[test]
fn m3d4_release10_fresh_and_historical_upgrades_preserve_user_state() {
    let current = catalog_tool::load(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../catalog/releases/10/manifest.json"),
    )
    .unwrap();
    assert_eq!(
        (current.manifest.version, current.data.ingredients.len()),
        (10, 795)
    );
    for previous_version in [7, 8, 9] {
        let previous = catalog_tool::load(&Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
            "../catalog/releases/{previous_version}/manifest.json"
        )))
        .unwrap();
        let directory = tempfile::tempdir().unwrap();
        let mut db = Database::open(directory.path()).unwrap();
        db.import_catalog(&previous).unwrap();
        let own = db.create_ingredient("Aile mutfağı malzemesi").unwrap();
        let saved = db.save_recipe(recipe(&own)).unwrap();
        let draft = db
            .save_draft(DraftWrite {
                id: uuid::Uuid::new_v4().to_string(),
                expected_revision: None,
                input: recipe(&own),
            })
            .unwrap();
        db.set_ingredient_favorite(&own.id, "personal", true)
            .unwrap();
        let honey = db.ingredients("Honey").unwrap().remove(0);
        db.set_ingredient_favorite(&honey.id, "catalog", true)
            .unwrap();
        db.set_catalog_presentation(
            crate::domain::ingredient_library::CatalogPresentationInput {
                id: honey.id.clone(),
                revision: 1,
                name: Some("Evimizin balı".into()),
                alias: None,
                preferred_unit: Some("kg".into()),
                notes: None,
            },
        )
        .unwrap();
        let before_ids = previous
            .data
            .ingredients
            .iter()
            .map(|i| (&i.key, &i.id))
            .collect::<std::collections::HashMap<_, _>>();
        let report = db.import_catalog(&current).unwrap();
        assert_eq!(report.inserted, 795 - previous.data.ingredients.len());
        assert_eq!(report.collisions, 0);
        assert!(db.import_catalog(&current).unwrap().unchanged);
        for item in &previous.data.ingredients {
            assert_eq!(before_ids[&item.key], &item.id);
            let now = current
                .data
                .ingredients
                .iter()
                .find(|i| i.key == item.key)
                .unwrap();
            assert_eq!(item.id, now.id);
            assert_eq!(item.names, now.names);
            assert_eq!(item.categories, now.categories);
            assert_eq!(
                serde_json::to_value(&item.aliases).unwrap(),
                serde_json::to_value(&now.aliases).unwrap()
            );
            assert_eq!(item.preferred_unit, now.preferred_unit);
            if previous_version == 9 {
                assert_eq!(
                    serde_json::to_value(item).unwrap(),
                    serde_json::to_value(now).unwrap()
                );
            }
        }
        drop(db);
        let mut db = Database::open(directory.path()).unwrap();
        db.install_bundled_catalog().unwrap();
        assert_eq!(count(&db, "catalog_ingredients"), 854);
        assert_eq!(count(&db, "catalog_aliases"), 269);
        assert_eq!(count(&db, "ingredient_categories"), 50);
        assert_eq!(
            db.recipe(&saved.id).unwrap().ingredients[0].ingredient_id,
            own.id
        );
        assert_eq!(
            db.recipe(&saved.id).unwrap().ingredients[0]
                .quantity
                .as_deref(),
            Some("35.000001")
        );
        assert_eq!(db.recipe(&saved.id).unwrap().ingredients[0].unit_code, "cc");
        assert_eq!(
            db.draft(&draft.id).unwrap().input.ingredients[0].ingredient_id,
            own.id
        );
        assert!(
            db.ingredient_detail(&own.id, "personal")
                .unwrap()
                .item
                .favorite
        );
        let detail = db.ingredient_detail(&honey.id, "catalog").unwrap();
        assert_eq!(detail.item.name, "Evimizin balı");
        assert!(detail.item.favorite);
        for name in ["İSOT", "isot", "Urfa biber", "Fonio", "fonio", "TULUM"] {
            assert!(
                !db.available_ingredients(name).unwrap().items.is_empty(),
                "Missing search: {name}"
            );
        }
        assert_eq!(
            db.conn
                .query_row("PRAGMA integrity_check", [], |r| r.get::<_, String>(0))
                .unwrap(),
            "ok"
        );
        assert_eq!(
            db.conn
                .query_row("SELECT count(*) FROM pragma_foreign_key_check", [], |r| r
                    .get::<_, i64>(
                    0
                ))
                .unwrap(),
            0
        );
    }
}

#[test]
fn m3d4_reused_source_version_rolls_back_and_recovers_without_mutating_release9() {
    let old = catalog_tool::load(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../catalog/releases/9/manifest.json"),
    )
    .unwrap();
    let current = catalog_tool::load(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../catalog/releases/10/manifest.json"),
    )
    .unwrap();
    let directory = tempfile::tempdir().unwrap();
    let mut db = Database::open(directory.path()).unwrap();
    db.import_catalog(&old).unwrap();
    let own = db.create_ingredient("Saklanacak aile malzemesi").unwrap();
    let saved = db.save_recipe(recipe(&own)).unwrap();
    let source = current
        .manifest
        .sources
        .iter()
        .find(|s| s.id == "wikidata-m3d4")
        .unwrap();
    db.conn.execute("INSERT INTO catalog_sources(id,version,name,url,license,attribution,retrieved_at,artifact_sha256) VALUES(?1,?2,?3,?4,?5,?6,?7,?8)",
        rusqlite::params![source.id,source.version,source.name,source.url,source.license,source.attribution,source.retrieved_at,"0".repeat(64)]).unwrap();
    assert_eq!(
        db.import_catalog(&current).unwrap_err().code,
        "CATALOG_SOURCE_CHANGED"
    );
    assert_eq!(count(&db, "catalog_ingredients"), 745);
    assert_eq!(count(&db, "catalog_releases"), 1);
    assert_eq!(
        db.recipe(&saved.id).unwrap().ingredients[0]
            .quantity
            .as_deref(),
        Some("35.000001")
    );
    db.conn
        .execute(
            "DELETE FROM catalog_sources WHERE id=?1 AND version=?2",
            rusqlite::params![source.id, source.version],
        )
        .unwrap();
    assert_eq!(db.import_catalog(&current).unwrap().inserted, 50);
    assert!(db.import_catalog(&current).unwrap().unchanged);
    assert_eq!(count(&db, "catalog_ingredients"), 795);
    assert_eq!(
        db.conn
            .query_row("PRAGMA integrity_check", [], |r| r.get::<_, String>(0))
            .unwrap(),
        "ok"
    );
}

#[test]
fn m3d5_release11_fresh_historical_upgrades_search_and_user_state() {
    let current = catalog_tool::bundled().unwrap();
    assert_eq!(
        (current.manifest.version, current.data.ingredients.len()),
        (11, 854)
    );
    let previous = catalog_tool::load(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../catalog/releases/10/manifest.json"),
    )
    .unwrap();
    assert_eq!(previous.data.ingredients.len(), 795);
    for item in &previous.data.ingredients {
        let now = current
            .data
            .ingredients
            .iter()
            .find(|x| x.key == item.key)
            .unwrap();
        assert_eq!(
            serde_json::to_value(item).unwrap(),
            serde_json::to_value(now).unwrap()
        );
    }
    for previous_version in [7, 8, 9, 10] {
        let prior = catalog_tool::load(&Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
            "../catalog/releases/{previous_version}/manifest.json"
        )))
        .unwrap();
        let directory = tempfile::tempdir().unwrap();
        let mut db = Database::open(directory.path()).unwrap();
        db.import_catalog(&prior).unwrap();
        let own = db.create_ingredient("Aile içeceği girdisi").unwrap();
        let saved = db.save_recipe(recipe(&own)).unwrap();
        let draft = db
            .save_draft(DraftWrite {
                id: uuid::Uuid::new_v4().to_string(),
                expected_revision: None,
                input: recipe(&own),
            })
            .unwrap();
        db.set_ingredient_favorite(&own.id, "personal", true)
            .unwrap();
        let honey = db.ingredients("Honey").unwrap().remove(0);
        db.set_ingredient_favorite(&honey.id, "catalog", true)
            .unwrap();
        db.set_catalog_presentation(
            crate::domain::ingredient_library::CatalogPresentationInput {
                id: honey.id.clone(),
                revision: 1,
                name: Some("Ev balı".into()),
                alias: None,
                preferred_unit: Some("kg".into()),
                notes: None,
            },
        )
        .unwrap();
        let imported = db.import_catalog(&current).unwrap();
        assert_eq!(imported.inserted, 854 - prior.data.ingredients.len());
        assert!(db.import_catalog(&current).unwrap().unchanged);
        drop(db);
        let mut db = Database::open(directory.path()).unwrap();
        db.install_bundled_catalog().unwrap();
        assert_eq!(count(&db, "catalog_ingredients"), 854);
        assert_eq!(count(&db, "catalog_aliases"), 269);
        assert_eq!(count(&db, "ingredient_categories"), 50);
        let retained = db.recipe(&saved.id).unwrap();
        assert_eq!(retained.ingredients[0].ingredient_id, own.id);
        assert_eq!(
            retained.ingredients[0].quantity.as_deref(),
            Some("35.000001")
        );
        assert_eq!(retained.ingredients[0].unit_code, "cc");
        assert_eq!(
            db.draft(&draft.id).unwrap().input.ingredients[0].ingredient_id,
            own.id
        );
        assert!(
            db.ingredient_detail(&own.id, "personal")
                .unwrap()
                .item
                .favorite
        );
        let honey_detail = db.ingredient_detail(&honey.id, "catalog").unwrap();
        assert_eq!(honey_detail.item.name, "Ev balı");
        assert!(honey_detail.item.favorite);
        for query in [
            "MATCHA",
            "matcha",
            "Arabica",
            "Portakal kokteyl bitteri",
            "Orange cocktail bitters",
            "Salep unu",
            "ginger beer",
        ] {
            assert!(
                !db.available_ingredients(query).unwrap().items.is_empty(),
                "Missing Release-11 search: {query}"
            );
        }
        assert_eq!(
            db.conn
                .query_row("PRAGMA integrity_check", [], |r| r.get::<_, String>(0))
                .unwrap(),
            "ok"
        );
        assert_eq!(
            db.conn
                .query_row("SELECT count(*) FROM pragma_foreign_key_check", [], |r| r
                    .get::<_, i64>(
                    0
                ))
                .unwrap(),
            0
        );
    }
}

#[test]
fn m3d5_source_snapshot_conflict_rolls_back_without_losing_personal_data() {
    let old = catalog_tool::load(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../catalog/releases/10/manifest.json"),
    )
    .unwrap();
    let current = catalog_tool::bundled().unwrap();
    let directory = tempfile::tempdir().unwrap();
    let mut db = Database::open(directory.path()).unwrap();
    db.import_catalog(&old).unwrap();
    let own = db.create_ingredient("Saklanan içecek girdisi").unwrap();
    let saved = db.save_recipe(recipe(&own)).unwrap();
    let source = current
        .manifest
        .sources
        .iter()
        .find(|s| s.id == "wikidata-m3d5")
        .unwrap();
    db.conn.execute("INSERT INTO catalog_sources(id,version,name,url,license,attribution,retrieved_at,artifact_sha256) VALUES(?1,?2,?3,?4,?5,?6,?7,?8)",
        rusqlite::params![source.id,source.version,source.name,source.url,source.license,source.attribution,source.retrieved_at,"0".repeat(64)]).unwrap();
    assert_eq!(
        db.import_catalog(&current).unwrap_err().code,
        "CATALOG_SOURCE_CHANGED"
    );
    assert_eq!(count(&db, "catalog_ingredients"), 795);
    assert_eq!(
        db.recipe(&saved.id).unwrap().ingredients[0]
            .quantity
            .as_deref(),
        Some("35.000001")
    );
    db.conn
        .execute(
            "DELETE FROM catalog_sources WHERE id=?1 AND version=?2",
            rusqlite::params![source.id, source.version],
        )
        .unwrap();
    assert_eq!(db.import_catalog(&current).unwrap().inserted, 59);
    assert!(db.import_catalog(&current).unwrap().unchanged);
    assert_eq!(count(&db, "catalog_ingredients"), 854);
}
