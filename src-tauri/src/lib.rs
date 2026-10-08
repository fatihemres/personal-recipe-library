mod commands;
mod desktop_menu;
mod domain;
mod native_labels;
mod persistence;
mod services;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
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
            commands::save_preferences
        ])
        .run(tauri::generate_context!())
        .expect("desktop runtime could not start");
}
