mod commands;
mod opened;

use std::path::Path;

use commands::transfers::Transfers;
use commands::{files, profile, terminal, transfers, trash};
use opened::Opened;
use sshpanel_core::{Core, SystemSecretStore};
use tauri::Manager;

pub fn run() {
    tauri::Builder::default()
        .manage(Opened::default())
        // Abrir un Perfil con la app ya abierta: se lo lleva la primera instancia.
        .plugin(tauri_plugin_single_instance::init(|app, args, cwd| {
            if let Some(path) = opened::profile_arg(&args, &opened::extension(app), Path::new(&cwd)) {
                opened::deliver(app, path);
            }
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.unminimize();
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            let secrets = SystemSecretStore::new(app.config().identifier.clone());
            app.manage(Core::new(data_dir, Box::new(secrets)));
            app.manage(Transfers::default());
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
            transfers::run_transfer,
            transfers::cancel_transfer
        ])
        .build(tauri::generate_context!())
        .expect("error al iniciar la aplicación")
        .run(|_app, _event| {
            // En macOS, el Perfil abierto con doble clic llega como evento.
            #[cfg(target_os = "macos")]
            if let tauri::RunEvent::Opened { urls } = _event {
                if let Some(path) = urls.iter().find_map(|url| url.to_file_path().ok()) {
                    opened::deliver(_app, path);
                }
            }
        });
}
