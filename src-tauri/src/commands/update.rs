//! Actualizar la app desde las Releases del Branding. Solo existe si
//! `branding.toml` tiene clave pública y repositorio (ver [`enabled`]).

use std::sync::Mutex;

use serde::Serialize;
use tauri::{AppHandle, Manager, Runtime, State};
use tauri_plugin_opener::OpenerExt;
use tauri_plugin_updater::{Update, UpdaterExt};

/// La versión encontrada por [`check_update`], lista para instalarse.
#[derive(Default)]
pub struct Pending(Mutex<Option<Update>>);

#[derive(Serialize)]
pub struct Available {
    version: String,
}

/// Si el Branding configura el actualizador: sin él, el plugin no se carga.
pub fn enabled<R: Runtime>(app: &AppHandle<R>) -> bool {
    app.config().plugins.0.contains_key("updater")
}

#[tauri::command]
pub async fn check_update(app: AppHandle, pending: State<'_, Pending>) -> Result<Option<Available>, String> {
    let update = app.updater().map_err(failed)?.check().await.map_err(failed)?;
    let available = update.as_ref().map(|update| Available { version: update.version.clone() });
    *pending.0.lock().unwrap() = update;
    Ok(available)
}

/// Descarga e instala la versión encontrada y reinicia la app con ella.
#[tauri::command]
pub async fn install_update(app: AppHandle, pending: State<'_, Pending>) -> Result<(), String> {
    let update = pending.0.lock().unwrap().take().ok_or("update.unavailable")?;
    update.download_and_install(|_, _| {}, || {}).await.map_err(failed)?;
    app.restart()
}

/// Abre la página de la última Release, para instalarla a mano si falla.
#[tauri::command]
pub fn open_releases(app: AppHandle) -> Result<(), String> {
    let config = app.config().plugins.0.get("updater").ok_or("update.unavailable")?;
    let endpoint = config["endpoints"][0].as_str().ok_or("update.unavailable")?;
    // El endpoint es …/releases/latest/download/latest.json (lo genera el Branding).
    let page = endpoint.strip_suffix("/download/latest.json").ok_or("update.unavailable")?;
    app.opener().open_url(page, None::<&str>).map_err(failed)
}

fn failed(err: impl std::fmt::Display) -> String {
    eprintln!("actualizador: {err}");
    "update.failed".into()
}

pub fn manage<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    app.manage(Pending::default());
    if enabled(app) {
        app.plugin(tauri_plugin_updater::Builder::new().build())?;
    }
    Ok(())
}
