//! Papelera: mover a ella, listarla, restaurar y, en Modo avanzado, borrar
//! para siempre.

use sshpanel_core::{Core, Restored, Trashed};
use tauri::State;

use super::code;

/// Devuelve el id en la Papelera, para «Deshacer».
#[tauri::command]
pub async fn trash(dir: String, name: String, core: State<'_, Core>) -> Result<String, String> {
    core.trash(&dir, &name).await.map_err(code)
}

#[tauri::command]
pub async fn list_trash(core: State<'_, Core>) -> Result<Vec<Trashed>, String> {
    core.list_trash().await.map_err(code)
}

#[tauri::command]
pub async fn restore(id: String, core: State<'_, Core>) -> Result<Restored, String> {
    core.restore(&id).await.map_err(code)
}

#[tauri::command]
pub async fn delete_forever(id: String, core: State<'_, Core>) -> Result<(), String> {
    core.delete_forever(&id).await.map_err(code)
}

#[tauri::command]
pub async fn empty_trash(core: State<'_, Core>) -> Result<(), String> {
    core.empty_trash().await.map_err(code)
}
