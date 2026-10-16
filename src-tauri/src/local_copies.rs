//! Copias en el equipo de los archivos del Servidor abiertos con su aplicación.
//! Viven en una carpeta temporal propia, una subcarpeta por archivo para que
//! conserven su nombre, y se vigilan por sondeo: así da igual si la aplicación
//! guarda en el mismo archivo o escribe uno nuevo y lo renombra. Subir los
//! cambios es cosa de la interfaz, como una Transferencia más de la cola. Se
//! borran al dejar de vigilarlas, al cambiar de Perfil y al cerrar la app.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, SystemTime};

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_opener::OpenerExt;

use crate::windows;

/// Evento a la ventana principal con el número de la copia que ha cambiado.
const EVENT: &str = "local-copy-changed";
/// Cada cuánto se miran las copias.
const POLL: Duration = Duration::from_secs(1);

/// Tamaño y fecha de modificación de una copia en el equipo.
#[derive(Clone, Copy, PartialEq, Eq)]
struct Seen {
    size: u64,
    modified: Option<SystemTime>,
}

fn seen(path: &Path) -> Option<Seen> {
    fs::metadata(path).ok().filter(|m| m.is_file()).map(|m| Seen { size: m.len(), modified: m.modified().ok() })
}

struct Watched {
    /// Su subcarpeta en la carpeta temporal.
    dir: PathBuf,
    /// El archivo, desde que la interfaz pide vigilarlo.
    file: Option<PathBuf>,
    /// Como estaba la última vez que se avisó (o al descargarlo).
    reported: Option<Seen>,
    /// Lo visto en la última vuelta, si ha cambiado: se avisa cuando se queda
    /// igual una vuelta entera, para no subir un guardado a medias.
    pending: Option<Seen>,
}

#[derive(Default)]
struct Inner {
    next: u32,
    copies: HashMap<u32, Watched>,
}

pub struct LocalCopies {
    root: PathBuf,
    inner: Mutex<Inner>,
}

/// Una subcarpeta nueva y vacía para descargar una copia.
#[derive(Serialize)]
pub struct Prepared {
    id: u32,
    dir: PathBuf,
}

impl LocalCopies {
    /// Vacía lo que dejara una sesión anterior que acabó de golpe; con una
    /// sola instancia de la app, nadie más la usa.
    pub fn new(identifier: &str) -> Self {
        let root = std::env::temp_dir().join(format!("{identifier}-copias"));
        let _ = fs::remove_dir_all(&root);
        Self { root, inner: Mutex::default() }
    }

    /// Deja de vigilar todas las copias y las borra.
    pub fn clear(&self) {
        self.inner.lock().unwrap().copies.clear();
        let _ = fs::remove_dir_all(&self.root);
    }

    /// Los números de las copias que han cambiado y ya no cambian.
    fn changed(&self) -> Vec<u32> {
        let mut inner = self.inner.lock().unwrap();
        let mut changed = Vec::new();
        for (&id, copy) in &mut inner.copies {
            let Some(file) = &copy.file else { continue };
            let now = seen(file);
            if now == copy.reported || now.is_none() {
                copy.pending = None;
            } else if now == copy.pending {
                copy.reported = now;
                copy.pending = None;
                changed.push(id);
            } else {
                copy.pending = now;
            }
        }
        changed
    }
}

/// Mira las copias cada [`POLL`] y avisa a la ventana principal de las que han cambiado.
pub fn watch(app: &AppHandle) {
    let app = app.clone();
    thread::spawn(move || {
        loop {
            thread::sleep(POLL);
            for id in app.state::<LocalCopies>().changed() {
                let _ = app.emit_to(windows::MAIN, EVENT, id);
            }
        }
    });
}

#[tauri::command]
pub fn prepare_local_copy(copies: State<LocalCopies>) -> Result<Prepared, String> {
    let mut inner = copies.inner.lock().unwrap();
    inner.next += 1;
    let id = inner.next;
    let dir = copies.root.join(id.to_string());
    fs::create_dir_all(&dir).map_err(|_| "local_copy.unavailable")?;
    inner.copies.insert(id, Watched { dir: dir.clone(), file: None, reported: None, pending: None });
    Ok(Prepared { id, dir })
}

/// Abre con su aplicación el archivo `file` descargado en la subcarpeta de la
/// copia `id`, y empieza a vigilarlo.
#[tauri::command]
pub fn watch_local_copy(id: u32, file: PathBuf, app: AppHandle, copies: State<LocalCopies>) -> Result<(), String> {
    {
        let mut inner = copies.inner.lock().unwrap();
        let copy = inner.copies.get_mut(&id).ok_or("local_copy.unavailable")?;
        // Solo lo descargado en su subcarpeta, nunca otra ruta del equipo.
        if file.parent() != Some(copy.dir.as_path()) {
            return Err("local_copy.unavailable".into());
        }
        copy.reported = seen(&file);
        copy.file = Some(file);
    }
    open_local_copy(id, app, copies)
}

/// Vuelve a abrir con su aplicación la copia `id`.
#[tauri::command]
pub fn open_local_copy(id: u32, app: AppHandle, copies: State<LocalCopies>) -> Result<(), String> {
    let file = copies.inner.lock().unwrap().copies.get(&id).and_then(|copy| copy.file.clone()).ok_or("local_copy.unavailable")?;
    let file = file.to_str().ok_or("local_copy.open_failed")?;
    app.opener().open_path(file, None::<&str>).map_err(|_| "local_copy.open_failed".into())
}

/// Deja de vigilar la copia `id` y la borra.
#[tauri::command]
pub fn forget_local_copy(id: u32, copies: State<LocalCopies>) {
    if let Some(copy) = copies.inner.lock().unwrap().copies.remove(&id) {
        let _ = fs::remove_dir_all(copy.dir);
    }
}

/// Deja de vigilar todas las copias y las borra, como al cambiar de Perfil.
#[tauri::command]
pub fn clear_local_copies(copies: State<LocalCopies>) {
    copies.clear();
}
