use std::path::PathBuf;

use serde::Serialize;
use sshpanel_core::{Core, Listing, Profile, SystemSecretStore};
use tauri::{Manager, State};

/// Lo que la interfaz necesita saber al arrancar.
#[derive(Serialize)]
struct AppState {
    profile: Option<Profile>,
    public_key: Option<String>,
}

fn code(err: sshpanel_core::Error) -> String {
    err.code().to_string()
}

#[tauri::command]
fn app_state(core: State<Core>) -> Result<AppState, String> {
    let profile = core.current_profile().map_err(code)?;
    let public_key = match profile {
        Some(_) => Some(core.public_key().map_err(code)?),
        None => None,
    };
    Ok(AppState { profile, public_key })
}

#[tauri::command]
fn import_profile(path: PathBuf, core: State<Core>) -> Result<AppState, String> {
    let profile = core.import_profile(&path).map_err(code)?;
    let public_key = core.public_key().map_err(code)?;
    Ok(AppState { profile: Some(profile), public_key: Some(public_key) })
}

#[tauri::command]
async fn connect(core: State<'_, Core>) -> Result<Listing, String> {
    core.connect().await.map_err(code)
}

#[tauri::command]
async fn list_dir(path: String, core: State<'_, Core>) -> Result<Listing, String> {
    core.list(&path).await.map_err(code)
}

#[tauri::command]
async fn create_folder(parent: String, name: String, core: State<'_, Core>) -> Result<(), String> {
    core.create_folder(&parent, &name).await.map_err(code)
}

#[tauri::command]
async fn rename(dir: String, from: String, to: String, core: State<'_, Core>) -> Result<(), String> {
    core.rename(&dir, &from, &to).await.map_err(code)
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            let secrets = SystemSecretStore::new(app.config().identifier.clone());
            app.manage(Core::new(data_dir, Box::new(secrets)));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![app_state, import_profile, connect, list_dir, create_folder, rename])
        .run(tauri::generate_context!())
        .expect("error al iniciar la aplicación");
}
