//! Administración: leer las Claves públicas autorizadas de cada usuario.

use sshpanel_core::{Core, UserKeys};
use tauri::State;

use super::code;

#[tauri::command]
pub async fn authorized_keys(core: State<'_, Core>) -> Result<Vec<UserKeys>, String> {
    core.authorized_keys().await.map_err(code)
}
