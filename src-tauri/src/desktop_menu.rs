use crate::native_labels as tr;
use tauri::{
    menu::{Menu, PredefinedMenuItem as Item, Submenu},
    AppHandle, Runtime,
};
pub fn build<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<Menu<R>> {
    Menu::with_items(
        app,
        &[
            &Submenu::with_items(
                app,
                tr::APP,
                true,
                &[
                    #[cfg(target_os = "macos")]
                    &Item::hide(app, Some(tr::HIDE))?,
                    &Item::quit(app, Some(tr::QUIT))?,
                ],
            )?,
            &Submenu::with_items(
                app,
                tr::EDIT,
                true,
                &[
                    &Item::undo(app, Some(tr::UNDO))?,
                    &Item::redo(app, Some(tr::REDO))?,
                    &Item::separator(app)?,
                    &Item::cut(app, Some(tr::CUT))?,
                    &Item::copy(app, Some(tr::COPY))?,
                    &Item::paste(app, Some(tr::PASTE))?,
                    &Item::select_all(app, Some(tr::SELECT_ALL))?,
                ],
            )?,
            &Submenu::with_items(
                app,
                tr::WINDOW,
                true,
                &[
                    &Item::minimize(app, Some(tr::MINIMIZE))?,
                    &Item::fullscreen(app, Some(tr::FULLSCREEN))?,
                    &Item::close_window(app, Some(tr::CLOSE))?,
                ],
            )?,
        ],
    )
}
