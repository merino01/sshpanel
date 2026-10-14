use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde::Serialize;
use sshpanel_core::{Core, Listing, Outcome, Profile, Progress, Restored, SystemSecretStore, Transfer, Trashed};
use tauri::ipc::Channel;
use tauri::{Manager, State};

/// Cada cuánto, como mucho, se avisa a la interfaz del avance de una Transferencia.
const PROGRESS_INTERVAL: Duration = Duration::from_millis(100);

/// Lo que la interfaz necesita saber al arrancar.
#[derive(Serialize)]
struct AppState {
    profile: Option<Profile>,
    public_key: Option<String>,
}

fn code(err: sshpanel_core::Error) -> String {
    err.code().to_string()
}

#[tauri::command]
fn app_state(core: State<Core>) -> Result<AppState, String> {
    let profile = core.current_profile().map_err(code)?;
    let public_key = match profile {
        Some(_) => Some(core.public_key().map_err(code)?),
        None => None,
    };
    Ok(AppState { profile, public_key })
}

#[tauri::command]
fn import_profile(path: PathBuf, core: State<Core>) -> Result<AppState, String> {
    let profile = core.import_profile(&path).map_err(code)?;
    let public_key = core.public_key().map_err(code)?;
    Ok(AppState { profile: Some(profile), public_key: Some(public_key) })
}

#[tauri::command]
async fn connect(core: State<'_, Core>) -> Result<Listing, String> {
    core.connect().await.map_err(code)
}

#[tauri::command]
async fn list_dir(path: String, core: State<'_, Core>) -> Result<Listing, String> {
    core.list(&path).await.map_err(code)
}

#[tauri::command]
async fn create_folder(parent: String, name: String, core: State<'_, Core>) -> Result<(), String> {
    core.create_folder(&parent, &name).await.map_err(code)
}

#[tauri::command]
async fn rename(dir: String, from: String, to: String, core: State<'_, Core>) -> Result<(), String> {
    core.rename(&dir, &from, &to).await.map_err(code)
}

#[tauri::command]
async fn trash(dir: String, name: String, core: State<'_, Core>) -> Result<(), String> {
    core.trash(&dir, &name).await.map_err(code)
}

#[tauri::command]
async fn list_trash(core: State<'_, Core>) -> Result<Vec<Trashed>, String> {
    core.list_trash().await.map_err(code)
}

#[tauri::command]
async fn restore(id: String, core: State<'_, Core>) -> Result<Restored, String> {
    core.restore(&id).await.map_err(code)
}

/// Envía el avance por `channel`, sin saturar la interfaz: el primero, el
/// último y como mucho uno cada [`PROGRESS_INTERVAL`].
fn throttled(channel: Channel<Progress>) -> impl FnMut(Progress) + Send {
    let mut last: Option<Instant> = None;
    move |progress| {
        if progress.done == progress.total || last.is_none_or(|t| t.elapsed() >= PROGRESS_INTERVAL) {
            last = Some(Instant::now());
            let _ = channel.send(progress);
        }
    }
}

/// Transferencias registradas por la interfaz, por el número que les da ella.
/// Se guardan hasta que terminan o se cancelan, para poder reanudarlas.
#[derive(Default)]
struct Transfers(Mutex<HashMap<u32, Arc<Transfer>>>);

impl Transfers {
    fn add(&self, id: u32, transfer: Transfer) {
        self.0.lock().unwrap().insert(id, Arc::new(transfer));
    }

    fn get(&self, id: u32) -> Option<Arc<Transfer>> {
        self.0.lock().unwrap().get(&id).cloned()
    }

    fn remove(&self, id: u32) -> Option<Arc<Transfer>> {
        self.0.lock().unwrap().remove(&id)
    }
}

#[tauri::command]
fn add_upload(id: u32, local: PathBuf, remote_dir: String, transfers: State<Transfers>) {
    transfers.add(id, Transfer::upload(local, remote_dir));
}

#[tauri::command]
fn add_download(id: u32, remote_dir: String, name: String, local_dir: PathBuf, transfers: State<Transfers>) {
    transfers.add(id, Transfer::download(remote_dir, name, local_dir));
}

/// Lanza la Transferencia `id`, o la reanuda donde se quedó.
#[tauri::command]
async fn run_transfer(
    id: u32,
    on_progress: Channel<Progress>,
    core: State<'_, Core>,
    transfers: State<'_, Transfers>,
) -> Result<Outcome, String> {
    let transfer = transfers.get(id).ok_or_else(|| code(sshpanel_core::Error::Cancelled))?;
    let result = core.run(&transfer, throttled(on_progress)).await;
    if result.is_ok() {
        transfers.remove(id);
    }
    result.map_err(code)
}

/// Cancela la Transferencia `id`, en curso o fallida, y borra lo que había creado.
#[tauri::command]
async fn cancel_transfer(id: u32, core: State<'_, Core>, transfers: State<'_, Transfers>) -> Result<(), String> {
    if let Some(transfer) = transfers.remove(id) {
        core.discard(&transfer).await;
    }
    Ok(())
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            let secrets = SystemSecretStore::new(app.config().identifier.clone());
            app.manage(Core::new(data_dir, Box::new(secrets)));
            app.manage(Transfers::default());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            app_state,
            import_profile,
            connect,
            list_dir,
            create_folder,
            rename,
            trash,
            list_trash,
            restore,
            add_upload,
            add_download,
            run_transfer,
            cancel_transfer
        ])
        .run(tauri::generate_context!())
        .expect("error al iniciar la aplicación");
}
