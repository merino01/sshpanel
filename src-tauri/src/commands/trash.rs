//! Papelera: mover a ella, listarla y restaurar.

use sshpanel_core::{Core, Restored, Trashed};
use tauri::State;

use super::code;

#[tauri::command]
pub async fn trash(dir: String, name: String, core: State<'_, Core>) -> Result<(), String> {
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
