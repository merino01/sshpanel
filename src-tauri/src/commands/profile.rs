//! Perfil, identidad y conexión, y crear Perfiles (Administrador).

use std::path::PathBuf;

use serde::Serialize;
use sshpanel_core::{Core, Listing, NewProfile, Profile};
use tauri::State;

use super::code;
use crate::opened::Opened;

/// Lo que la interfaz necesita saber al arrancar.
#[derive(Serialize)]
pub struct AppState {
    profile: Option<Profile>,
    public_key: Option<String>,
}

#[tauri::command]
pub fn app_state(core: State<Core>) -> Result<AppState, String> {
    let profile = core.current_profile().map_err(code)?;
    let public_key = match profile {
        Some(_) => Some(core.public_key().map_err(code)?),
        None => None,
    };
    Ok(AppState { profile, public_key })
}

#[tauri::command]
pub fn import_profile(path: PathBuf, core: State<Core>) -> Result<AppState, String> {
    let profile = core.import_profile(&path).map_err(code)?;
    let public_key = core.public_key().map_err(code)?;
    Ok(AppState { profile: Some(profile), public_key: Some(public_key) })
}

/// Lee un fichero de Perfil sin importarlo, para enseñar qué contiene.
#[tauri::command]
pub fn read_profile(path: PathBuf, core: State<Core>) -> Result<Profile, String> {
    core.read_profile(&path).map_err(code)
}

/// El Perfil abierto con doble clic que la interfaz aún no ha recogido, si lo hay.
#[tauri::command]
pub fn take_opened_profile(opened: State<Opened>) -> Option<PathBuf> {
    opened.take()
}

#[tauri::command]
pub async fn connect(core: State<'_, Core>) -> Result<Listing, String> {
    core.connect().await.map_err(code)
}

/// Crea el fichero de Perfil `path` para un Miembro, calculando la Huella del Servidor.
#[tauri::command]
pub async fn create_profile(profile: NewProfile, path: PathBuf, core: State<'_, Core>) -> Result<Profile, String> {
    core.create_profile(profile, &path).await.map_err(code)
}
