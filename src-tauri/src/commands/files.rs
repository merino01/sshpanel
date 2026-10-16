//! Explorador: listar, crear carpetas, renombrar y ver si un archivo ha cambiado.

use sshpanel_core::{Core, Listing, Stamp};
use tauri::State;

use super::code;

#[tauri::command]
pub async fn list_dir(path: String, core: State<'_, Core>) -> Result<Listing, String> {
    core.list(&path).await.map_err(code)
}

#[tauri::command]
pub async fn create_folder(parent: String, name: String, core: State<'_, Core>) -> Result<(), String> {
    core.create_folder(&parent, &name).await.map_err(code)
}

#[tauri::command]
pub async fn rename(dir: String, from: String, to: String, core: State<'_, Core>) -> Result<(), String> {
    core.rename(&dir, &from, &to).await.map_err(code)
}

#[tauri::command]
pub async fn file_stamp(path: String, core: State<'_, Core>) -> Result<Stamp, String> {
    core.stamp(&path).await.map_err(code)
}
