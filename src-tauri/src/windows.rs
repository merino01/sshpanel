//! Las ventanas de la app: la principal, Ajustes y una por cada archivo abierto
//! en el Editor. Todas cargan la misma interfaz; el parámetro `window` de la URL
//! le dice cuál pintar. Comparten el núcleo y la conexión de la principal, y se
//! cierran con ella. Todas tienen el mismo aspecto (ver [`styled`]).

use std::collections::{HashMap, HashSet};
use std::sync::Mutex;

use tauri::{AppHandle, Manager, Runtime, State, WebviewUrl, WebviewWindow, WebviewWindowBuilder, WindowEvent};

/// Etiqueta de la ventana principal, la que crea `tauri.conf.json`.
pub const MAIN: &str = "main";

const SETTINGS: &str = "settings";

/// Abre la ventana principal con lo que dice `tauri.conf.json` (tamaño y el
/// título del Branding), que no la crea sola para darle el aspecto común.
pub fn open_main(app: &AppHandle) -> tauri::Result<()> {
    let config = app.config().app.windows.iter().find(|window| window.label == MAIN).cloned().unwrap_or_default();
    styled(WebviewWindowBuilder::from_config(app, &config)?).build().map(|_| ())
}

/// El aspecto de todas las ventanas: la barra de título dentro de la interfaz
/// (`TitleBar.svelte`) y, en macOS y Windows, fondo transparente para el marco
/// de cristal. En macOS se quedan los semáforos del sistema sobre la barra y el
/// cristal es el del sistema; en Windows, Mica. Con la Transparencia quitada, la
/// interfaz pinta el fondo opaco encima.
///
/// En Linux la ventana es opaca: con WebKitGTK transparente (al menos con NVIDIA
/// en Wayland) solo se pintan las zonas que cambian y el resto se queda vacío.
fn styled<'a, R: Runtime, M: Manager<R>>(builder: WebviewWindowBuilder<'a, R, M>) -> WebviewWindowBuilder<'a, R, M> {
    let builder = builder.transparent(!cfg!(target_os = "linux"));
    #[cfg(target_os = "macos")]
    let builder = {
        use tauri::window::{Effect, EffectState, EffectsBuilder};
        builder
            .title_bar_style(tauri::TitleBarStyle::Overlay)
            .hidden_title(true)
            // Centrados en la barra de título, que es más alta que la del sistema.
            .traffic_light_position(tauri::LogicalPosition::new(16.0, 18.0))
            .effects(EffectsBuilder::new().effect(Effect::Sidebar).state(EffectState::FollowsWindowActiveState).build())
    };
    #[cfg(target_os = "windows")]
    let builder = {
        use tauri::window::{Effect, EffectsBuilder};
        builder.decorations(false).effects(EffectsBuilder::new().effect(Effect::Mica).build())
    };
    #[cfg(target_os = "linux")]
    let builder = builder.decorations(false);
    builder
}

/// Las etiquetas de las ventanas del Editor empiezan así (`editor-1`…), como
/// dice `capabilities/default.json`.
const EDITOR_PREFIX: &str = "editor-";

/// Abre Ajustes, que es una ventana única: si ya está abierta, la pone delante.
#[tauri::command]
pub fn open_settings(title: String, app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(SETTINGS) {
        let _ = window.unminimize();
        let _ = window.show();
        return window.set_focus().map_err(failed);
    }
    styled(WebviewWindowBuilder::new(&app, SETTINGS, WebviewUrl::App("index.html?window=settings".into())))
        .title(title)
        .inner_size(640.0, 520.0)
        .min_inner_size(480.0, 360.0)
        .build()
        .map(|_| ())
        .map_err(failed)
}

/// Las ventanas del Editor abiertas: qué archivo tiene cada una y cuáles
/// tienen cambios sin guardar.
#[derive(Default)]
pub struct Editors(Mutex<EditorWindows>);

#[derive(Default)]
struct EditorWindows {
    /// Etiqueta de la ventana de cada archivo, por su ruta en el Servidor.
    by_path: HashMap<String, String>,
    unsaved: HashSet<String>,
    /// Para dar a cada ventana nueva una etiqueta que no se repita.
    next: u32,
}

impl EditorWindows {
    fn path_of(&self, label: &str) -> Option<&String> {
        self.by_path.iter().find(|(_, l)| *l == label).map(|(path, _)| path)
    }

    fn forget(&mut self, label: &str) {
        self.by_path.retain(|_, l| l != label);
        self.unsaved.remove(label);
    }
}

/// Abre el archivo `path` del Servidor en una ventana del Editor con el título
/// `title`. Cada archivo tiene la suya: si ya está abierto, la pone delante.
/// Es asíncrono porque en Windows crear una ventana desde un comando síncrono
/// bloquea la app.
#[tauri::command]
pub async fn open_editor(path: String, title: String, app: AppHandle, editors: State<'_, Editors>) -> Result<(), String> {
    let mut windows = editors.0.lock().unwrap();
    if let Some(window) = windows.by_path.get(&path).and_then(|label| app.get_webview_window(label)) {
        let _ = window.unminimize();
        let _ = window.show();
        return window.set_focus().map_err(failed);
    }
    windows.next += 1;
    let label = format!("{EDITOR_PREFIX}{}", windows.next);
    let window = styled(WebviewWindowBuilder::new(&app, &label, WebviewUrl::App("index.html?window=editor".into())))
        .title(title)
        .inner_size(900.0, 640.0)
        .min_inner_size(480.0, 320.0)
        .build()
        .map_err(failed)?;
    windows.by_path.insert(path, label.clone());
    let handle = app.clone();
    window.on_window_event(move |event| {
        if let WindowEvent::Destroyed = event {
            handle.state::<Editors>().0.lock().unwrap().forget(&label);
        }
    });
    Ok(())
}

/// El archivo del Servidor que tiene abierto esta ventana del Editor.
#[tauri::command]
pub fn editor_file(window: WebviewWindow, editors: State<Editors>) -> Option<String> {
    editors.0.lock().unwrap().path_of(window.label()).cloned()
}

/// Apunta si esta ventana del Editor tiene cambios sin guardar, para preguntar
/// antes de cerrar la principal o cambiar de Perfil.
#[tauri::command]
pub fn set_unsaved(unsaved: bool, window: WebviewWindow, editors: State<Editors>) {
    let mut windows = editors.0.lock().unwrap();
    if unsaved {
        windows.unsaved.insert(window.label().to_string());
    } else {
        windows.unsaved.remove(window.label());
    }
}

/// Cuántas ventanas del Editor tienen cambios sin guardar.
#[tauri::command]
pub fn unsaved_editors(editors: State<Editors>) -> u32 {
    editors.0.lock().unwrap().unsaved.len() as u32
}

/// Cierra todas las ventanas del Editor sin preguntar; la interfaz ya lo ha
/// hecho si alguna tenía cambios sin guardar.
#[tauri::command]
pub fn close_editors(app: AppHandle, editors: State<Editors>) {
    let labels: Vec<String> = editors.0.lock().unwrap().by_path.values().cloned().collect();
    for label in labels {
        if let Some(window) = app.get_webview_window(&label) {
            let _ = window.destroy();
        }
    }
}

fn failed(err: tauri::Error) -> String {
    eprintln!("ventanas: {err}");
    "window.failed".into()
}
