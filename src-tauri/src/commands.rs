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
            .manage(StorageService::new(Ok(dir.path().into())))
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
            .manage(StorageService::new(Ok(dir.path().into())))
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
                .manage(StorageService::new(Ok(directory.path().into())))
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
            .manage(StorageService::new(Ok(path.clone())))
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
            2
        );
    }
}
