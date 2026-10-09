use crate::{
    domain::{AppError, Bootstrap, Preferences},
    services::StorageService,
};
#[tauri::command]
pub async fn bootstrap(state: tauri::State<'_, StorageService>) -> Result<Bootstrap, AppError> {
    state.bootstrap().await
}
#[tauri::command]
pub async fn save_preferences(
    preferences: Preferences,
    state: tauri::State<'_, StorageService>,
) -> Result<Preferences, AppError> {
    state.save(preferences).await
}

use crate::domain::recipes::{Ingredient, Recipe, RecipeInput, Unit};
#[tauri::command]
pub async fn list_recipes(
    trash: bool,
    kind: Option<String>,
    state: tauri::State<'_, StorageService>,
) -> Result<Vec<Recipe>, AppError> {
    state.execute(move |d| d.list_recipes(trash, kind)).await
}
#[tauri::command]
pub async fn get_recipe(
    id: String,
    state: tauri::State<'_, StorageService>,
) -> Result<Recipe, AppError> {
    state.execute(move |d| d.recipe(&id)).await
}
#[tauri::command]
pub async fn save_recipe(
    input: RecipeInput,
    state: tauri::State<'_, StorageService>,
) -> Result<Recipe, AppError> {
    state.execute(move |d| d.save_recipe(input)).await
}
#[tauri::command]
pub async fn set_recipe_deleted(
    id: String,
    revision: i64,
    deleted: bool,
    state: tauri::State<'_, StorageService>,
) -> Result<Recipe, AppError> {
    state
        .execute(move |d| d.set_deleted(&id, revision, deleted))
        .await
}
#[tauri::command]
pub async fn search_ingredients(
    query: String,
    state: tauri::State<'_, StorageService>,
) -> Result<Vec<Ingredient>, AppError> {
    state.execute(move |d| d.ingredients(&query)).await
}
#[tauri::command]
pub async fn create_ingredient(
    name: String,
    state: tauri::State<'_, StorageService>,
) -> Result<Ingredient, AppError> {
    state.execute(move |d| d.create_ingredient(&name)).await
}
#[tauri::command]
pub async fn list_units(state: tauri::State<'_, StorageService>) -> Result<Vec<Unit>, AppError> {
    state.execute(|d| d.units()).await
}

use crate::domain::reliability::{
    Draft, DraftWrite, IngredientEdit, IngredientSearch, PersonalIngredient,
};
#[tauri::command]
pub async fn list_drafts(state: tauri::State<'_, StorageService>) -> Result<Vec<Draft>, AppError> {
    state.execute(|d| d.drafts()).await
}
#[tauri::command]
pub async fn get_draft(
    id: String,
    state: tauri::State<'_, StorageService>,
) -> Result<Draft, AppError> {
    state.execute(move |d| d.draft(&id)).await
}
#[tauri::command]
pub async fn save_draft(
    write: DraftWrite,
    state: tauri::State<'_, StorageService>,
) -> Result<Draft, AppError> {
    state.execute(move |d| d.save_draft(write)).await
}
#[tauri::command]
pub async fn discard_draft(
    id: String,
    revision: i64,
    state: tauri::State<'_, StorageService>,
) -> Result<(), AppError> {
    state.execute(move |d| d.discard_draft(&id, revision)).await
}
#[tauri::command]
pub async fn commit_recipe(
    input: RecipeInput,
    draft_id: String,
    draft_revision: i64,
    as_copy: bool,
    state: tauri::State<'_, StorageService>,
) -> Result<Recipe, AppError> {
    state
        .execute(move |d| d.commit_recipe(input, &draft_id, draft_revision, as_copy))
        .await
}
#[tauri::command]
pub async fn duplicate_recipe(
    id: String,
    revision: i64,
    state: tauri::State<'_, StorageService>,
) -> Result<Recipe, AppError> {
    state
        .execute(move |d| d.duplicate_recipe(&id, revision))
        .await
}
#[tauri::command]
pub async fn scope_recipes(
    scope: String,
    kind: Option<String>,
    state: tauri::State<'_, StorageService>,
) -> Result<Vec<Recipe>, AppError> {
    state.execute(move |d| d.scope_recipes(&scope, kind)).await
}
#[tauri::command]
pub async fn archive_recipe(
    id: String,
    revision: i64,
    archived: bool,
    state: tauri::State<'_, StorageService>,
) -> Result<Recipe, AppError> {
    state
        .execute(move |d| d.archive_recipe(&id, revision, archived))
        .await
}
#[tauri::command]
pub async fn purge_recipe(
    id: String,
    revision: i64,
    state: tauri::State<'_, StorageService>,
) -> Result<(), AppError> {
    state.execute(move |d| d.purge_recipe(&id, revision)).await
}
#[tauri::command]
pub async fn search_personal_ingredients(
    query: String,
    state: tauri::State<'_, StorageService>,
) -> Result<IngredientSearch, AppError> {
    state.execute(move |d| d.search_personal(&query)).await
}
#[tauri::command]
pub async fn edit_ingredient(
    input: IngredientEdit,
    state: tauri::State<'_, StorageService>,
) -> Result<PersonalIngredient, AppError> {
    state.execute(move |d| d.edit_ingredient(input)).await
}
#[tauri::command]
pub async fn delete_ingredient(
    id: String,
    revision: i64,
    state: tauri::State<'_, StorageService>,
) -> Result<(), AppError> {
    state
        .execute(move |d| d.delete_ingredient(&id, revision))
        .await
}

#[tauri::command]
pub fn finish_exit(app: tauri::AppHandle, state: tauri::State<'_, crate::ExitPermission>) {
    state.0.store(true, std::sync::atomic::Ordering::SeqCst);
    app.exit(0);
}

#[tauri::command]
pub async fn search_available_ingredients(
    query: String,
    state: tauri::State<'_, StorageService>,
) -> Result<IngredientSearch, AppError> {
    state
        .execute(move |d| d.available_ingredients(&query))
        .await
}
#[tauri::command]
pub async fn catalog_status(
    state: tauri::State<'_, StorageService>,
) -> Result<crate::domain::catalog::CatalogStatus, AppError> {
    state.execute(|d| d.catalog_status()).await
}
#[tauri::command]
pub async fn link_personal_catalog(
    id: String,
    revision: i64,
    catalog_id: String,
    state: tauri::State<'_, StorageService>,
) -> Result<(), AppError> {
    state
        .execute(move |d| d.link_personal_catalog(&id, revision, &catalog_id))
        .await
}
#[tauri::command]
pub async fn customize_catalog_ingredient(
    input: IngredientEdit,
    state: tauri::State<'_, StorageService>,
) -> Result<PersonalIngredient, AppError> {
    state
        .execute(move |d| d.customize_catalog_ingredient(input))
        .await
}
#[tauri::command]
pub async fn create_personal_category(
    tr: String,
    en: String,
    parent: Option<String>,
    state: tauri::State<'_, StorageService>,
) -> Result<String, AppError> {
    state
        .execute(move |d| d.create_personal_category(&tr, &en, parent.as_deref()))
        .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};
    use tauri::test::{get_ipc_response, mock_builder, mock_context, noop_assets, INVOKE_KEY};
    fn invoke(
        window: &tauri::WebviewWindow<tauri::test::MockRuntime>,
        command: &str,
        body: Value,
    ) -> Result<Value, Value> {
        get_ipc_response(
            window,
            tauri::webview::InvokeRequest {
                cmd: command.into(),
                callback: tauri::ipc::CallbackFn(0),
                error: tauri::ipc::CallbackFn(1),
                url: if cfg!(windows) {
                    "http://tauri.localhost"
                } else {
                    "tauri://localhost"
                }
                .parse()
                .unwrap(),
                body: tauri::ipc::InvokeBody::Json(body),
                headers: Default::default(),
                invoke_key: INVOKE_KEY.into(),
            },
        )
        .map(|response| response.deserialize::<Value>().unwrap())
    }
    #[test]
    fn registered_ipc_handlers_use_real_worker_and_persist_across_restart() {
        let dir = tempfile::tempdir().unwrap();
        let app = mock_builder()
            .manage(StorageService::without_catalog(Ok(dir.path().into())))
            .invoke_handler(tauri::generate_handler![bootstrap, save_preferences])
            .build(mock_context(noop_assets()))
            .unwrap();
        let window = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
            .build()
            .unwrap();
        let bootstrap = invoke(&window, "bootstrap", json!({})).unwrap();
        assert_eq!(bootstrap["storage"]["fts5"], true);
        assert_eq!(
            invoke(
                &window,
                "save_preferences",
                json!({"preferences":{"theme":"dark","locale":"tr"}})
            )
            .unwrap()["theme"],
            "dark"
        );
        let app2 = mock_builder()
            .manage(StorageService::without_catalog(Ok(dir.path().into())))
            .invoke_handler(tauri::generate_handler![super::bootstrap, save_preferences])
            .build(mock_context(noop_assets()))
            .unwrap();
        let window2 = tauri::WebviewWindowBuilder::new(&app2, "main", Default::default())
            .build()
            .unwrap();
        assert_eq!(
            invoke(&window2, "bootstrap", json!({})).unwrap()["preferences"]["theme"],
            "dark"
        );
        assert_eq!(
            invoke(
                &window,
                "save_preferences",
                json!({"preferences":{"theme":"light","locale":"en"}})
            )
            .unwrap_err()["code"],
            "INVALID_INPUT"
        );
        assert_eq!(
            invoke(&window, "bootstrap", json!({})).unwrap()["preferences"]["theme"],
            "dark"
        );
    }
    #[test]
    fn recipe_commands_round_trip_real_sqlite_across_independent_workers() {
        let directory = tempfile::tempdir().unwrap();
        let create_app = || {
            mock_builder()
                .manage(StorageService::without_catalog(Ok(directory.path().into())))
                .invoke_handler(tauri::generate_handler![
                    list_recipes,
                    get_recipe,
                    save_recipe,
                    set_recipe_deleted,
                    search_ingredients,
                    create_ingredient,
                    list_units
                ])
                .build(mock_context(noop_assets()))
                .unwrap()
        };
        let app = create_app();
        let window = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
            .build()
            .unwrap();
        let ingredient = invoke(&window, "create_ingredient", json!({"name":"İçme suyu"})).unwrap();
        assert_eq!(
            invoke(&window, "create_ingredient", json!({"name":"içme suyu"})).unwrap()["id"],
            ingredient["id"]
        );
        assert_eq!(
            invoke(&window, "search_ingredients", json!({"query":"içme"})).unwrap()[0]["id"],
            ingredient["id"]
        );
        assert_eq!(
            invoke(&window, "list_units", json!({}))
                .unwrap()
                .as_array()
                .unwrap()
                .len(),
            6
        );
        let id = uuid::Uuid::new_v4().to_string();
        let mut input = json!({"id":id,"expectedRevision":null,"title":"İçecek","description":null,"kind":"beverage","servings":"2.50","prepMinutes":2,"cookMinutes":null,"notes":"Çok soğuk", "ingredients":[{"id":uuid::Uuid::new_v4().to_string(),"ingredientId":ingredient["id"],"quantity":"35.000001","unitCode":"cc","note":null}], "steps":[{"id":uuid::Uuid::new_v4().to_string(),"instructions":"Karıştırın"}]});
        let saved = invoke(&window, "save_recipe", json!({"input":input})).unwrap();
        assert_eq!(saved["ingredients"][0]["quantity"], "35.000001");
        drop(window);
        drop(app);
        let app = create_app();
        let window = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
            .build()
            .unwrap();
        let loaded = invoke(&window, "get_recipe", json!({"id":id})).unwrap();
        assert_eq!(loaded["steps"][0]["instructions"], "Karıştırın");
        input["expectedRevision"] = json!(1);
        input["title"] = json!("İçecek güncel");
        assert_eq!(
            invoke(&window, "save_recipe", json!({"input":input})).unwrap()["revision"],
            2
        );
        assert_eq!(
            invoke(&window, "save_recipe", json!({"input":input})).unwrap_err()["code"],
            "CONFLICT"
        );
        invoke(
            &window,
            "set_recipe_deleted",
            json!({"id":id,"revision":2,"deleted":true}),
        )
        .unwrap();
        assert!(
            invoke(&window, "list_recipes", json!({"trash":false,"kind":null}))
                .unwrap()
                .as_array()
                .unwrap()
                .is_empty()
        );
        invoke(
            &window,
            "set_recipe_deleted",
            json!({"id":id,"revision":3,"deleted":false}),
        )
        .unwrap();
        assert_eq!(
            invoke(
                &window,
                "list_recipes",
                json!({"trash":false,"kind":"beverage"})
            )
            .unwrap()[0]["title"],
            "İçecek güncel"
        );
    }
    #[test]
    fn initialization_failure_can_retry_after_directory_is_repaired() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("app");
        std::fs::write(&path, "obstacle").unwrap();
        let app = mock_builder()
            .manage(StorageService::without_catalog(Ok(path.clone())))
            .invoke_handler(tauri::generate_handler![bootstrap])
            .build(mock_context(noop_assets()))
            .unwrap();
        let window = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
            .build()
            .unwrap();
        assert_eq!(
            invoke(&window, "bootstrap", json!({})).unwrap_err()["code"],
            "STORAGE_UNAVAILABLE"
        );
        std::fs::rename(&path, dir.path().join("preserved-obstacle")).unwrap();
        assert_eq!(
            invoke(&window, "bootstrap", json!({})).unwrap()["storage"]["schemaVersion"],
            4
        );
    }
    #[test]
    fn reliability_ipc_uses_actual_sqlite_for_drafts_management_and_ingredient_conflicts() {
        let dir = tempfile::tempdir().unwrap();
        let app = mock_builder()
            .manage(StorageService::without_catalog(Ok(dir.path().into())))
            .invoke_handler(tauri::generate_handler![
                create_ingredient,
                search_personal_ingredients,
                edit_ingredient,
                delete_ingredient,
                save_draft,
                get_draft,
                list_drafts,
                discard_draft,
                commit_recipe,
                duplicate_recipe,
                scope_recipes,
                archive_recipe,
                set_recipe_deleted,
                purge_recipe
            ])
            .build(mock_context(noop_assets()))
            .unwrap();
        let window = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
            .build()
            .unwrap();
        let ingredient = invoke(&window, "create_ingredient", json!({"name":"Şeker"})).unwrap();
        for query in ["Şek", "şek", "ŞEK", "şeker"] {
            assert_eq!(
                invoke(
                    &window,
                    "search_personal_ingredients",
                    json!({"query":query})
                )
                .unwrap()["items"][0]["id"],
                ingredient["id"]
            );
        }
        let id = uuid::Uuid::new_v4().to_string();
        let draft_id = uuid::Uuid::new_v4().to_string();
        let mut input = json!({"id":id,"expectedRevision":null,"title":"Şerbet","description":null,"kind":"beverage","servings":"1","prepMinutes":null,"cookMinutes":null,"notes":null,"ingredients":[{"id":uuid::Uuid::new_v4().to_string(),"ingredientId":ingredient["id"],"quantity":"35.","unitCode":"cc","note":null}],"steps":[]});
        let draft = invoke(
            &window,
            "save_draft",
            json!({"write":{"id":draft_id,"expectedRevision":null,"input":input}}),
        )
        .unwrap();
        assert_eq!(
            invoke(&window, "get_draft", json!({"id":draft_id})).unwrap()["input"]["ingredients"]
                [0]["quantity"],
            "35."
        );
        assert_eq!(
            invoke(&window, "list_drafts", json!({}))
                .unwrap()
                .as_array()
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            invoke(
                &window,
                "delete_ingredient",
                json!({"id":ingredient["id"],"revision":1})
            )
            .unwrap_err()["code"],
            "INGREDIENT_REFERENCED"
        );
        input["ingredients"][0]["quantity"] = json!("35.000001");
        let saved=invoke(&window,"commit_recipe",json!({"input":input,"draftId":draft_id,"draftRevision":draft["revision"],"asCopy":false})).unwrap();
        assert!(invoke(&window, "list_drafts", json!({}))
            .unwrap()
            .as_array()
            .unwrap()
            .is_empty());
        assert_eq!(
            invoke(
                &window,
                "discard_draft",
                json!({"id":draft_id,"revision":1})
            )
            .unwrap_err()["code"],
            "DRAFT_CONFLICT"
        );
        let copy = invoke(&window, "duplicate_recipe", json!({"id":id,"revision":1})).unwrap();
        assert_ne!(copy["id"], saved["id"]);
        invoke(
            &window,
            "archive_recipe",
            json!({"id":id,"revision":1,"archived":true}),
        )
        .unwrap();
        assert_eq!(
            invoke(
                &window,
                "scope_recipes",
                json!({"scope":"archived","kind":null})
            )
            .unwrap()
            .as_array()
            .unwrap()
            .len(),
            1
        );
        invoke(
            &window,
            "set_recipe_deleted",
            json!({"id":id,"revision":2,"deleted":true}),
        )
        .unwrap();
        assert_eq!(
            invoke(&window, "purge_recipe", json!({"id":id,"revision":3})).unwrap(),
            Value::Null
        );
        assert_eq!(
            invoke(
                &window,
                "scope_recipes",
                json!({"scope":"active","kind":null})
            )
            .unwrap()[0]["id"],
            copy["id"]
        );
        let renamed=invoke(&window,"edit_ingredient",json!({"input":{"id":ingredient["id"],"revision":1,"name":"Toz şeker","notes":"Kişisel","preferredUnit":"g"}})).unwrap();
        assert_eq!(renamed["revision"], 2);
        assert_eq!(
            invoke(
                &window,
                "delete_ingredient",
                json!({"id":ingredient["id"],"revision":2})
            )
            .unwrap_err()["code"],
            "INGREDIENT_REFERENCED"
        );
    }
    #[test]
    fn catalog_ipc_uses_installed_sqlite_data_and_real_worker() {
        let dir = tempfile::tempdir().unwrap();
        let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../catalog/validation/manifest.json");
        crate::catalog_tool::import(dir.path(), &manifest, true).unwrap();
        let app = mock_builder()
            .manage(StorageService::without_catalog(Ok(dir.path().into())))
            .invoke_handler(tauri::generate_handler![
                catalog_status,
                search_ingredients,
                customize_catalog_ingredient,
                create_personal_category
            ])
            .build(mock_context(noop_assets()))
            .unwrap();
        let window = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
            .build()
            .unwrap();
        assert_eq!(
            invoke(&window, "catalog_status", json!({})).unwrap()["validationDefinitions"],
            8
        );
        assert_eq!(
            invoke(&window, "catalog_status", json!({})).unwrap()["productionDefinitions"],
            0
        );
        let honey =
            invoke(&window, "search_ingredients", json!({"query":"Honey"})).unwrap()[0].clone();
        assert_eq!(invoke(&window,"customize_catalog_ingredient",json!({"input":{"id":honey["id"],"revision":1,"name":"Özel bal","notes":"Kişisel","preferredUnit":"g"}})).unwrap()["origin"],"catalog");
        assert_eq!(
            invoke(&window, "search_ingredients", json!({"query":"Honey"})).unwrap()[0]["name"],
            "Özel bal"
        );
        assert!(invoke(
            &window,
            "create_personal_category",
            json!({"tr":"Özel","en":"Custom","parent":null})
        )
        .unwrap()
        .as_str()
        .is_some());
    }
}
