mod commands;
mod local_copies;
mod opened;
mod windows;

use std::path::Path;

use commands::transfers::Transfers;
use commands::{admin, editor, files, preferences, profile, terminal, transfers, trash, update};
use local_copies::LocalCopies;
use opened::Opened;
use sshpanel_core::{Core, SystemSecretStore};
use tauri::Manager;
use windows::Editors;

pub fn run() {
    tauri::Builder::default()
        .manage(Opened::default())
        .manage(Editors::default())
        // Abrir un Perfil con la app ya abierta: se lo lleva la primera instancia.
        .plugin(tauri_plugin_single_instance::init(|app, args, cwd| {
            if let Some(path) = opened::profile_arg(&args, &opened::extension(app), Path::new(&cwd)) {
                opened::deliver(app, path);
            }
            if let Some(window) = app.get_webview_window(windows::MAIN) {
                let _ = window.unminimize();
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            let secrets = SystemSecretStore::new(app.config().identifier.clone());
            app.manage(Core::new(data_dir, Box::new(secrets)));
            app.manage(Transfers::default());
            app.manage(LocalCopies::new(&app.config().identifier));
            local_copies::watch(app.handle());
            update::manage(app.handle())?;
            windows::open_main(app.handle())?;
            let cwd = std::env::current_dir().unwrap_or_default();
            if let Some(path) = opened::profile_arg(std::env::args_os(), &opened::extension(app.handle()), &cwd) {
                opened::deliver(app.handle(), path);
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            profile::app_state,
            profile::import_profile,
            profile::read_profile,
            profile::take_opened_profile,
            profile::connect,
            profile::create_profile,
            files::list_dir,
            files::create_folder,
            files::rename,
            files::file_stamp,
            trash::trash,
            trash::list_trash,
            trash::restore,
            trash::delete_forever,
            trash::empty_trash,
            terminal::open_terminal,
            terminal::terminal_input,
            terminal::resize_terminal,
            terminal::close_terminal,
            transfers::add_upload,
            transfers::add_download,
            transfers::add_replace,
            transfers::run_transfer,
            transfers::cancel_transfer,
            update::check_update,
            update::install_update,
            update::open_releases,
            preferences::read_preferences,
            preferences::save_preferences,
            windows::open_settings,
            editor::read_head,
            editor::read_text,
            editor::write_text,
            editor::write_text_copy,
            windows::open_editor,
            windows::editor_file,
            windows::move_editor,
            windows::set_unsaved,
            windows::unsaved_editors,
            windows::close_editors,
            local_copies::prepare_local_copy,
            local_copies::watch_local_copy,
            local_copies::open_local_copy,
            local_copies::forget_local_copy,
            local_copies::clear_local_copies,
            admin::authorized_keys
        ])
        .build(tauri::generate_context!())
        .expect("error al iniciar la aplicación")
        .run(|_app, _event| {
            // La ventana principal manda: al cerrarla se cierran las demás (Ajustes, Editor…).
            if let tauri::RunEvent::WindowEvent { label, event: tauri::WindowEvent::Destroyed, .. } = &_event {
                if label == windows::MAIN {
                    _app.exit(0);
                }
            }
            // Las copias de archivos del Servidor no se quedan en el equipo.
            if let tauri::RunEvent::Exit = _event {
                _app.state::<LocalCopies>().clear();
            }
            // En macOS, el Perfil abierto con doble clic llega como evento.
            #[cfg(target_os = "macos")]
            if let tauri::RunEvent::Opened { urls } = _event {
                if let Some(path) = urls.iter().find_map(|url| url.to_file_path().ok()) {
                    opened::deliver(_app, path);
                }
            }
        });
}
