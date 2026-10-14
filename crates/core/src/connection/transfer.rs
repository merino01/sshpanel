//! Transferencias: subir y descargar archivos y carpetas enteras.
//!
//! Cada archivo se copia primero a un nombre provisional y solo toma su nombre
//! definitivo cuando está completo, así que un archivo con su nombre nunca está
//! a medias. Nunca se sustituye nada: si el nombre ya existe se usa
//! `nombre (1)`, `nombre (2)`…
//!
//! Una [`Transfer`] recuerda su destino y lo que lleva hecho: si falla, el
//! siguiente intento sigue donde se quedó, aprovechando los provisionales; si
//! se cancela, se borra lo que ha creado.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use serde::Serialize;
use tokio::fs;
use tokio::sync::Mutex;

use super::Connection;
use super::naming::{Reservation, Reserved, free_local_path};
use super::paths::{is_single_name, join};
use crate::error::{Error, Result};

mod file_copy;
mod plan;

use file_copy::{Counter, partial_name};
use plan::{Plan, local_plan};

/// Avance de una Transferencia, en bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Progress {
    pub done: u64,
    pub total: u64,
}

/// Dónde ha quedado una Transferencia terminada.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Outcome {
    /// Nombre con el que ha quedado en la carpeta del Servidor.
    Uploaded(String),
    /// Ruta en el equipo.
    Downloaded(PathBuf),
}

pub(crate) type Report<'a> = &'a mut (dyn FnMut(Progress) + Send);

/// Una subida o una descarga, de un archivo o de una carpeta entera.
pub struct Transfer {
    job: Job,
    cancelled: AtomicBool,
    /// Lo que dejaron los intentos anteriores; `None` si aún no ha empezado.
    started: Mutex<Option<Started>>,
}

enum Job {
    Upload { local: PathBuf, remote_dir: String },
    Download { remote_dir: String, name: String, local_dir: PathBuf },
}

impl Transfer {
    /// Subida del archivo o la carpeta `local` a la carpeta `remote_dir` del Servidor.
    pub fn upload(local: impl Into<PathBuf>, remote_dir: impl Into<String>) -> Self {
        Self::new(Job::Upload { local: local.into(), remote_dir: remote_dir.into() })
    }

    /// Descarga del archivo o la carpeta `name` de la carpeta `remote_dir` a la
    /// carpeta `local_dir` del equipo.
    pub fn download(remote_dir: impl Into<String>, name: impl Into<String>, local_dir: impl Into<PathBuf>) -> Self {
        Self::new(Job::Download { remote_dir: remote_dir.into(), name: name.into(), local_dir: local_dir.into() })
    }

    fn new(job: Job) -> Self {
        Self { job, cancelled: AtomicBool::new(false), started: Mutex::new(None) }
    }

    /// Pide que se pare: en cuanto avance, deja de copiar, borra lo que ha
    /// creado y falla con [`Error::Cancelled`]. No tiene vuelta atrás.
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }

    fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }
}

/// Una Transferencia que ya ha empezado: qué copia, adónde y qué lleva hecho.
struct Started {
    plan: Plan,
    /// Nombre con el que queda la raíz en su carpeta de destino.
    name: String,
    /// Ruta de la raíz en el destino, reservada mientras haga falta.
    root: Reservation,
    /// Qué elementos del plan están ya creados (carpetas) o completos (archivos).
    done: Vec<bool>,
}

impl Started {
    fn new(plan: Plan, name: String, root: Reservation) -> Self {
        let done = vec![false; plan.items.len()];
        Self { plan, name, root, done }
    }

    fn root(&self) -> &str {
        self.root.path()
    }

    /// Bytes de los archivos ya completos.
    fn copied(&self) -> u64 {
        self.plan.items.iter().zip(&self.done).filter(|(_, done)| **done).filter_map(|(item, _)| item.size).sum()
    }
}

/// Lanza `transfer`, o la reanuda si ya hubo un intento. Al terminar o al
/// cancelarse olvida lo hecho y libera el destino; si falla, lo conserva para
/// el siguiente intento.
pub(crate) async fn run(
    connection: Option<Arc<Connection>>,
    transfer: &Transfer,
    reserved: &Reserved,
    report: Report<'_>,
) -> Result<Outcome> {
    let mut started = transfer.started.lock().await;
    let result = match &connection {
        _ if transfer.is_cancelled() => Err(Error::Cancelled),
        None => Err(Error::NotConnected),
        Some(c) => c.copy(&transfer.job, &mut started, reserved, &transfer.cancelled, report).await,
    };
    match result {
        Ok(_) => *started = None,
        Err(Error::Cancelled) => undo(connection.as_deref(), &transfer.job, started.take()).await,
        Err(_) => {}
    }
    result
}

/// Cancela `transfer` y borra lo que haya creado. Si está en curso, espera a
/// que se pare.
pub(crate) async fn discard(connection: Option<Arc<Connection>>, transfer: &Transfer) {
    transfer.cancel();
    let mut started = transfer.started.lock().await;
    undo(connection.as_deref(), &transfer.job, started.take()).await;
}

/// Deshace lo hecho, de dentro afuera: provisionales, archivos completos y
/// carpetas, que solo se borran si se han quedado vacías. Nunca toca lo que no
/// ha creado la Transferencia. En una subida sin conexión no se puede borrar
/// nada; los provisionales quedan ocultos.
async fn undo(connection: Option<&Connection>, job: &Job, started: Option<Started>) {
    let Some(started) = started else { return };
    let items = started.plan.items.iter().zip(&started.done).rev();
    match job {
        Job::Upload { .. } => {
            let Some(c) = connection else { return };
            for (item, &done) in items {
                let remote = item.remote(started.root());
                if item.size.is_none() {
                    if done {
                        let _ = c.sftp.remove_dir(remote).await;
                    }
                    continue;
                }
                let _ = c.sftp.remove_file(partial_name(&remote)).await;
                if done {
                    let _ = c.sftp.remove_file(remote).await;
                }
            }
        }
        Job::Download { .. } => {
            for (item, &done) in items {
                let local = item.local(Path::new(started.root()));
                if item.size.is_none() {
                    if done {
                        let _ = fs::remove_dir(local).await;
                    }
                    continue;
                }
                if let Some(partial) = local.to_str().map(partial_name) {
                    let _ = fs::remove_file(partial).await;
                }
                if done {
                    let _ = fs::remove_file(local).await;
                }
            }
        }
    }
}

impl Connection {
    async fn copy(
        &self,
        job: &Job,
        started: &mut Option<Started>,
        reserved: &Reserved,
        cancelled: &AtomicBool,
        report: Report<'_>,
    ) -> Result<Outcome> {
        let mut counter = Counter { done: 0, total: 0, report, cancelled };
        match job {
            Job::Upload { local, remote_dir } => {
                let started = self.upload(local, remote_dir, started, reserved, &mut counter).await?;
                Ok(Outcome::Uploaded(started.name.clone()))
            }
            Job::Download { remote_dir, name, local_dir } => {
                let started = self.download(remote_dir, name, local_dir, started, reserved, &mut counter).await?;
                Ok(Outcome::Downloaded(PathBuf::from(started.root())))
            }
        }
    }

    async fn upload<'s>(
        &self,
        local: &Path,
        remote_dir: &str,
        started: &'s mut Option<Started>,
        reserved: &Reserved,
        counter: &mut Counter<'_>,
    ) -> Result<&'s Started> {
        let resuming = started.is_some();
        if !resuming {
            let name = local.file_name().and_then(|n| n.to_str()).filter(|n| is_single_name(n)).ok_or(Error::LocalUnreadable)?;
            let plan = local_plan(local).await?;
            let (name, root) = self.free_remote_name(remote_dir, name, plan.is_dir(), reserved, Error::TransferFailed).await?;
            *started = Some(Started::new(plan, name, root));
        }
        let started = started.as_mut().expect("empezada");

        // Lo que dejó a medias un intento anterior.
        let mut offsets = vec![0; started.plan.items.len()];
        if resuming {
            for (i, item) in started.plan.items.iter().enumerate() {
                let Some(size) = item.size.filter(|_| !started.done[i]) else { continue };
                let remote = item.remote(started.root());
                // Completo, aunque el intento anterior no llegó a saberlo.
                if self.sftp.symlink_metadata(remote.as_str()).await.is_ok_and(|m| m.is_regular() && m.len() == size) {
                    started.done[i] = true;
                } else if let Ok(m) = self.sftp.symlink_metadata(partial_name(&remote)).await
                    && m.is_regular()
                    && m.len() <= size
                {
                    offsets[i] = m.len();
                }
            }
        }

        counter.total = started.plan.total;
        counter.done = started.copied() + offsets.iter().sum::<u64>();
        counter.report();
        for (i, item) in started.plan.items.iter().enumerate() {
            if started.done[i] {
                continue;
            }
            counter.check()?;
            let remote = item.remote(started.root());
            match item.size {
                None => match self.sftp.create_dir(remote.as_str()).await {
                    Ok(()) => {}
                    // La creó un intento anterior, aunque no llegó a saberlo.
                    Err(_) if resuming && self.sftp.metadata(remote.as_str()).await.is_ok_and(|m| m.is_dir()) => {}
                    Err(e) => return Err(self.failure(e, Error::TransferFailed)),
                },
                Some(_) => self.upload_file(&item.local(local), &remote, offsets[i], counter).await?,
            }
            started.done[i] = true;
        }
        Ok(started)
    }

    async fn download<'s>(
        &self,
        remote_dir: &str,
        name: &str,
        local_dir: &Path,
        started: &'s mut Option<Started>,
        reserved: &Reserved,
        counter: &mut Counter<'_>,
    ) -> Result<&'s Started> {
        let source = join(remote_dir, name);
        let resuming = started.is_some();
        if !resuming {
            if !is_single_name(name) {
                return Err(Error::InvalidName);
            }
            let plan = self.remote_plan(&source).await?;
            let (name, root) = free_local_path(local_dir, name, plan.is_dir(), reserved).await?;
            *started = Some(Started::new(plan, name, root));
        }
        let started = started.as_mut().expect("empezada");

        let mut offsets = vec![0; started.plan.items.len()];
        if resuming {
            for (i, item) in started.plan.items.iter().enumerate() {
                let Some(size) = item.size.filter(|_| !started.done[i]) else { continue };
                let local = item.local(Path::new(started.root()));
                if fs::symlink_metadata(&local).await.is_ok_and(|m| m.is_file() && m.len() == size) {
                    started.done[i] = true;
                } else if let Some(partial) = local.to_str().map(partial_name)
                    && let Ok(m) = fs::symlink_metadata(partial).await
                    && m.is_file()
                    && m.len() <= size
                {
                    offsets[i] = m.len();
                }
            }
        }

        counter.total = started.plan.total;
        counter.done = started.copied() + offsets.iter().sum::<u64>();
        counter.report();
        for (i, item) in started.plan.items.iter().enumerate() {
            if started.done[i] {
                continue;
            }
            counter.check()?;
            let local = item.local(Path::new(started.root()));
            match item.size {
                None => match fs::create_dir(&local).await {
                    Ok(()) => {}
                    Err(_) if resuming && fs::metadata(&local).await.is_ok_and(|m| m.is_dir()) => {}
                    Err(_) => return Err(Error::LocalWriteFailed),
                },
                Some(_) => self.download_file(&item.remote(&source), &local, offsets[i], counter).await?,
            }
            started.done[i] = true;
        }
        Ok(started)
    }
}
