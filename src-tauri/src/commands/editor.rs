//! Editor: leer el principio de un archivo, leerlo entero y guardarlo. Las
//! ventanas del Editor las abre `windows`.

use sshpanel_core::{Core, Head, Stamp, TextFile};
use tauri::State;

use super::code;

#[tauri::command]
pub async fn read_head(path: String, core: State<'_, Core>) -> Result<Head, String> {
    core.read_head(&path).await.map_err(code)
}

#[tauri::command]
pub async fn read_text(path: String, core: State<'_, Core>) -> Result<TextFile, String> {
    core.read_text(&path).await.map_err(code)
}

#[tauri::command]
pub async fn write_text(path: String, text: String, core: State<'_, Core>) -> Result<Stamp, String> {
    core.write_text(&path, &text).await.map_err(code)
}
