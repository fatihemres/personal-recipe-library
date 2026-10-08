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
            1
        );
    }
}
