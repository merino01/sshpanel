mod commands;

use commands::transfers::Transfers;
use commands::{files, profile, terminal, transfers, trash};
use sshpanel_core::{Core, SystemSecretStore};
use tauri::Manager;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            let secrets = SystemSecretStore::new(app.config().identifier.clone());
            app.manage(Core::new(data_dir, Box::new(secrets)));
            app.manage(Transfers::default());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            profile::app_state,
            profile::import_profile,
            profile::connect,
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
        .run(tauri::generate_context!())
        .expect("error al iniciar la aplicación");
}
