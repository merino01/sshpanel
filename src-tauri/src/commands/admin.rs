//! Administración: leer las Claves públicas autorizadas de cada usuario.

use sshpanel_core::{Core, UserKeys};
use tauri::State;

use super::code;

#[tauri::command]
pub async fn authorized_keys(core: State<'_, Core>) -> Result<Vec<UserKeys>, String> {
    core.authorized_keys().await.map_err(code)
}

#[tauri::command]
pub async fn server_users(core: State<'_, Core>) -> Result<Vec<String>, String> {
    core.server_users().await.map_err(code)
}

/// La contraseña de sudo solo pasa por aquí: el núcleo la guarda en memoria.
#[tauri::command]
pub async fn unlock_admin(password: String, core: State<'_, Core>) -> Result<(), String> {
    let password = zeroize::Zeroizing::new(password);
    core.unlock_admin(&password).await.map_err(code)
}

#[tauri::command]
pub async fn authorize_key(user: String, line: String, core: State<'_, Core>) -> Result<(), String> {
    core.authorize_key(&user, &line).await.map_err(code)
}

#[tauri::command]
pub async fn revoke_key(user: String, line: String, core: State<'_, Core>) -> Result<(), String> {
    core.revoke_key(&user, &line).await.map_err(code)
}

#[tauri::command]
pub async fn replace_key(user: String, old: String, new: String, core: State<'_, Core>) -> Result<(), String> {
    core.replace_key(&user, &old, &new).await.map_err(code)
}
