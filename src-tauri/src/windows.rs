//! Ventanas de la app además de la principal. Todas cargan la misma interfaz;
//! el parámetro `window` de la URL le dice cuál pintar. Comparten el núcleo y la
//! conexión de la principal, y se cierran con ella.

use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

/// Etiqueta de la ventana principal, la que crea `tauri.conf.json`.
pub const MAIN: &str = "main";

const SETTINGS: &str = "settings";

/// Abre Ajustes, que es una ventana única: si ya está abierta, la pone delante.
#[tauri::command]
pub fn open_settings(title: String, app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(SETTINGS) {
        let _ = window.unminimize();
        let _ = window.show();
        return window.set_focus().map_err(failed);
    }
    WebviewWindowBuilder::new(&app, SETTINGS, WebviewUrl::App("index.html?window=settings".into()))
        .title(title)
        .inner_size(640.0, 520.0)
        .min_inner_size(480.0, 360.0)
        .build()
        .map(|_| ())
        .map_err(failed)
}

fn failed(err: tauri::Error) -> String {
    eprintln!("ventanas: {err}");
    "window.failed".into()
}
