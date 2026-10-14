//! Transferencias: la interfaz las registra con un número, las lanza o
//! reanuda y las cancela.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use sshpanel_core::{Core, Error, Outcome, Progress, Transfer};
use tauri::State;
use tauri::ipc::Channel;

use super::code;

/// Cada cuánto, como mucho, se avisa a la interfaz del avance de una Transferencia.
const PROGRESS_INTERVAL: Duration = Duration::from_millis(100);

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
pub struct Transfers(Mutex<HashMap<u32, Arc<Transfer>>>);

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
pub fn add_upload(id: u32, local: PathBuf, remote_dir: String, transfers: State<Transfers>) {
    transfers.add(id, Transfer::upload(local, remote_dir));
}

#[tauri::command]
pub fn add_download(id: u32, remote_dir: String, name: String, local_dir: PathBuf, transfers: State<Transfers>) {
    transfers.add(id, Transfer::download(remote_dir, name, local_dir));
}

/// Lanza la Transferencia `id`, o la reanuda donde se quedó.
#[tauri::command]
pub async fn run_transfer(
    id: u32,
    on_progress: Channel<Progress>,
    core: State<'_, Core>,
    transfers: State<'_, Transfers>,
) -> Result<Outcome, String> {
    let transfer = transfers.get(id).ok_or_else(|| code(Error::Cancelled))?;
    let result = core.run(&transfer, throttled(on_progress)).await;
    if result.is_ok() {
        transfers.remove(id);
    }
    result.map_err(code)
}

/// Cancela la Transferencia `id`, en curso o fallida, y borra lo que había creado.
#[tauri::command]
pub async fn cancel_transfer(id: u32, core: State<'_, Core>, transfers: State<'_, Transfers>) -> Result<(), String> {
    if let Some(transfer) = transfers.remove(id) {
        core.discard(&transfer).await;
    }
    Ok(())
}
