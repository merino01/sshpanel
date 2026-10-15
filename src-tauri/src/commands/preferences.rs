//! Preferencias del Miembro (Ajustes): un fichero en la carpeta de datos de su
//! equipo, aparte del Perfil, así que sobreviven a importar otro. Para Rust son
//! JSON sin más; los valores por defecto los pone la interfaz. Cada cambio se
//! avisa a todas las ventanas con el evento `preferences-changed`.

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;
use tauri::{AppHandle, Emitter, Manager};

const FILE: &str = "preferences.json";

/// Evento con el que se avisa a todas las ventanas de las preferencias nuevas.
const EVENT: &str = "preferences-changed";

/// El mismo código que usa el núcleo cuando no puede escribir en el equipo.
const STORAGE: &str = "storage.failed";

#[tauri::command]
pub fn read_preferences(app: AppHandle) -> Result<Value, String> {
    Ok(load(&file(&app)?))
}

#[tauri::command]
pub fn save_preferences(preferences: Value, app: AppHandle) -> Result<(), String> {
    store(&file(&app)?, &preferences).map_err(|_| STORAGE.to_string())?;
    let _ = app.emit(EVENT, preferences);
    Ok(())
}

fn file(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(app.path().app_data_dir().map_err(|_| STORAGE)?.join(FILE))
}

/// Las preferencias guardadas en `path`, o `null` si no hay o no se entienden.
fn load(path: &Path) -> Value {
    fs::read_to_string(path).ok().and_then(|text| serde_json::from_str(&text).ok()).unwrap_or(Value::Null)
}

/// Guarda `preferences` en `path` de una vez: se escribe al lado y se renombra,
/// para que un cierre a medias no deje el fichero roto.
fn store(path: &Path, preferences: &Value) -> std::io::Result<()> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    let partial = path.with_extension("json.partial");
    fs::write(&partial, serde_json::to_vec_pretty(preferences)?)?;
    fs::rename(&partial, path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn temp_dir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!("sshpanel-preferences-{}-{:?}", std::process::id(), std::thread::current().id()));
        let _ = fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn stores_and_loads_the_preferences() {
        let path = temp_dir().join("datos").join(FILE);
        store(&path, &json!({ "theme": "dark" })).unwrap();

        assert_eq!(load(&path), json!({ "theme": "dark" }));
        store(&path, &json!({ "theme": "light" })).unwrap();
        assert_eq!(load(&path), json!({ "theme": "light" }));
    }

    #[test]
    fn a_missing_or_broken_file_is_null() {
        let dir = temp_dir();
        assert_eq!(load(&dir.join(FILE)), Value::Null);

        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join(FILE), "{ roto").unwrap();
        assert_eq!(load(&dir.join(FILE)), Value::Null);
    }
}
