pub mod catalog_tool;
mod commands;
mod desktop_menu;
mod domain;
mod native_labels;
mod persistence;
mod services;
use tauri::{Emitter, Manager};
#[derive(Default)]
pub struct ExitPermission(pub std::sync::atomic::AtomicBool);

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(ExitPermission::default())
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if !window
                    .state::<ExitPermission>()
                    .0
                    .load(std::sync::atomic::Ordering::SeqCst)
                {
                    api.prevent_close();
                    let _ = window.emit("request-exit", ());
                }
            }
        })
        .menu(desktop_menu::build)
        .setup(|app| {
            let directory = app
                .path()
                .app_data_dir()
                .map_err(|_| domain::AppError::storage());
            app.manage(services::StorageService::new(directory));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::bootstrap,
            commands::save_preferences,
            commands::list_recipes,
            commands::get_recipe,
            commands::save_recipe,
            commands::set_recipe_deleted,
            commands::search_ingredients,
            commands::create_ingredient,
            commands::list_units,
            commands::list_drafts,
            commands::get_draft,
            commands::save_draft,
            commands::discard_draft,
            commands::commit_recipe,
            commands::duplicate_recipe,
            commands::scope_recipes,
            commands::archive_recipe,
            commands::purge_recipe,
            commands::search_personal_ingredients,
            commands::edit_ingredient,
            commands::delete_ingredient,
            commands::catalog_status,
            commands::search_available_ingredients,
            commands::link_personal_catalog,
            commands::customize_catalog_ingredient,
            commands::create_personal_category,
            commands::finish_exit
        ])
        .build(tauri::generate_context!())
        .expect("desktop runtime could not start")
        .run(|app, event| {
            if let tauri::RunEvent::ExitRequested { api, .. } = event {
                if !app
                    .state::<ExitPermission>()
                    .0
                    .load(std::sync::atomic::Ordering::SeqCst)
                {
                    api.prevent_exit();
                    let _ = app.emit("request-exit", ());
                }
            }
        });
}
