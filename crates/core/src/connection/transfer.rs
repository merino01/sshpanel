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

use std::collections::HashSet;
use std::io::{self, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex as SyncMutex};

use russh_sftp::client::error::Error as SftpError;
use russh_sftp::protocol::{OpenFlags, StatusCode};
use serde::Serialize;
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};
use tokio::sync::Mutex;

use super::{Connection, is_single_name, join};
use crate::error::{Error, Result};

const CHUNK: usize = 256 * 1024;
const MAX_NUMBERED: u32 = 1000;

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
        &self.root.place.path
    }

    /// Bytes de los archivos ya completos.
    fn copied(&self) -> u64 {
        self.plan.items.iter().zip(&self.done).filter(|(_, done)| **done).filter_map(|(item, _)| item.size).sum()
    }
}

/// Algo que copiar, por su ruta dentro de la Transferencia (vacía para la
/// raíz). `size` es `None` para las carpetas.
struct Item {
    path: Vec<String>,
    size: Option<u64>,
}

impl Item {
    fn remote(&self, root: &str) -> String {
        self.path.iter().fold(root.to_string(), |p, c| join(&p, c))
    }

    fn local(&self, root: &Path) -> PathBuf {
        self.path.iter().fold(root.to_path_buf(), |p, c| p.join(c))
    }
}

/// Lo que hay que copiar, cada carpeta antes que su contenido.
struct Plan {
    items: Vec<Item>,
    total: u64,
}

impl Plan {
    fn new(root: Option<u64>) -> Self {
        Self { items: vec![Item { path: Vec::new(), size: root }], total: root.unwrap_or(0) }
    }

    fn is_dir(&self) -> bool {
        self.items[0].size.is_none()
    }

    fn push(&mut self, path: Vec<String>, size: Option<u64>) {
        self.total += size.unwrap_or(0);
        self.items.push(Item { path, size });
    }
}

/// Un destino en el Servidor o en el equipo.
#[derive(Clone, PartialEq, Eq, Hash)]
struct Place {
    remote: bool,
    path: String,
}

/// Destinos de las Transferencias empezadas, para que dos en paralelo con el
/// mismo nombre no elijan el mismo.
#[derive(Clone, Default)]
pub(crate) struct Reserved(Arc<SyncMutex<HashSet<Place>>>);

impl Reserved {
    fn reserve(&self, place: Place) -> Option<Reservation> {
        let mut set = self.0.lock().unwrap();
        set.insert(place.clone()).then(|| Reservation { set: self.clone(), place })
    }
}

/// Un destino reservado; se libera al soltarlo.
pub(super) struct Reservation {
    set: Reserved,
    place: Place,
}

impl Drop for Reservation {
    fn drop(&mut self) {
        self.set.0.lock().unwrap().remove(&self.place);
    }
}

struct Counter<'a> {
    done: u64,
    total: u64,
    report: Report<'a>,
    cancelled: &'a AtomicBool,
}

impl Counter<'_> {
    fn report(&mut self) {
        (self.report)(Progress { done: self.done, total: self.total });
    }

    fn add(&mut self, bytes: usize) -> Result<()> {
        self.done += bytes as u64;
        self.report();
        self.check()
    }

    fn check(&self) -> Result<()> {
        if self.cancelled.load(Ordering::SeqCst) { Err(Error::Cancelled) } else { Ok(()) }
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

    /// Copia `local` a `remote` pasando por el provisional, continuando desde
    /// el byte `offset` si un intento anterior lo dejó a medias.
    async fn upload_file(&self, local: &Path, remote: &str, offset: u64, counter: &mut Counter<'_>) -> Result<()> {
        let partial = partial_name(remote);
        let result = async {
            let mut source = fs::File::open(local).await.map_err(|_| Error::LocalUnreadable)?;
            let mut target = if offset > 0 {
                source.seek(SeekFrom::Start(offset)).await.map_err(|_| Error::LocalUnreadable)?;
                let mut target = self.sftp.open_with_flags(partial.as_str(), OpenFlags::WRITE).await.map_err(|e| self.failure(e, Error::TransferFailed))?;
                target.seek(SeekFrom::Start(offset)).await.map_err(|e| self.io_failure(e))?;
                target
            } else {
                self.sftp.create(partial.as_str()).await.map_err(|e| self.failure(e, Error::TransferFailed))?
            };
            let mut buf = vec![0; CHUNK];
            loop {
                let n = source.read(&mut buf).await.map_err(|_| Error::LocalUnreadable)?;
                if n == 0 {
                    break;
                }
                target.write_all(&buf[..n]).await.map_err(|e| self.io_failure(e))?;
                counter.add(n)?;
            }
            target.shutdown().await.map_err(|e| self.io_failure(e))?;
            self.ensure_free(remote).await?;
            self.sftp.rename(partial.as_str(), remote).await.map_err(|e| self.failure(e, Error::TransferFailed))
        }
        .await;
        // Sin conexión no se puede limpiar, y el provisional sirve para reanudar.
        if result.as_ref().is_err_and(|e| *e != Error::ConnectionLost) {
            let _ = self.sftp.remove_file(partial).await;
        }
        result
    }

    /// Copia `remote` a `local` pasando por el provisional, continuando desde
    /// el byte `offset` si un intento anterior lo dejó a medias.
    async fn download_file(&self, remote: &str, local: &Path, offset: u64, counter: &mut Counter<'_>) -> Result<()> {
        let partial = PathBuf::from(partial_name(local.to_str().ok_or(Error::LocalWriteFailed)?));
        let result = async {
            let mut source = self.sftp.open(remote).await.map_err(|e| self.failure(e, Error::TransferFailed))?;
            let mut target = if offset > 0 {
                source.seek(SeekFrom::Start(offset)).await.map_err(|e| self.io_failure(e))?;
                let mut target = fs::OpenOptions::new().write(true).open(&partial).await.map_err(|_| Error::LocalWriteFailed)?;
                target.seek(SeekFrom::Start(offset)).await.map_err(|_| Error::LocalWriteFailed)?;
                target
            } else {
                fs::File::create(&partial).await.map_err(|_| Error::LocalWriteFailed)?
            };
            let mut buf = vec![0; CHUNK];
            let copied = async {
                loop {
                    let n = source.read(&mut buf).await.map_err(|e| self.io_failure(e))?;
                    if n == 0 {
                        return Ok(());
                    }
                    target.write_all(&buf[..n]).await.map_err(|_| Error::LocalWriteFailed)?;
                    counter.add(n)?;
                }
            }
            .await;
            // Lo recibido queda escrito aunque falle, para poder reanudar.
            target.sync_all().await.map_err(|_| Error::LocalWriteFailed)?;
            drop(target);
            copied?;
            if fs::try_exists(local).await.unwrap_or(true) {
                return Err(Error::AlreadyExists);
            }
            fs::rename(&partial, local).await.map_err(|_| Error::LocalWriteFailed)
        }
        .await;
        if result.as_ref().is_err_and(|e| *e != Error::ConnectionLost) {
            let _ = fs::remove_file(&partial).await;
        }
        result
    }

    /// Recorre lo que hay que descargar. Dentro de una carpeta, los enlaces a
    /// carpetas y los enlaces rotos se omiten para no entrar en bucles.
    async fn remote_plan(&self, source: &str) -> Result<Plan> {
        let meta = self.sftp.metadata(source).await.map_err(|e| self.failure(e, Error::TransferFailed))?;
        if !meta.is_dir() {
            return Ok(Plan::new(Some(meta.len())));
        }
        let mut plan = Plan::new(None);
        let mut pending = vec![Vec::new()];
        while let Some(dir) = pending.pop() {
            let path = dir.iter().fold(source.to_string(), |p: String, c: &String| join(&p, c));
            let entries = self.sftp.read_dir(path.as_str()).await.map_err(|e| self.failure(e, Error::TransferFailed))?;
            for e in entries.filter(|e| !matches!(e.file_name().as_str(), "." | "..")) {
                let mut child = dir.clone();
                child.push(e.file_name());
                let meta = if e.file_type().is_symlink() {
                    match self.sftp.metadata(join(&path, &e.file_name())).await {
                        Ok(target) if !target.is_dir() => target,
                        _ => continue,
                    }
                } else {
                    e.metadata()
                };
                if meta.is_dir() {
                    pending.push(child.clone());
                    plan.push(child, None);
                } else {
                    plan.push(child, Some(meta.len()));
                }
            }
        }
        Ok(plan)
    }

    /// Primer nombre libre para `name` en `dir`, reservado. `fallback` es el
    /// error de la operación que lo pide, si el Servidor falla sin más.
    pub(super) async fn free_remote_name(
        &self,
        dir: &str,
        name: &str,
        is_dir: bool,
        reserved: &Reserved,
        fallback: Error,
    ) -> Result<(String, Reservation)> {
        for candidate in numbered(name, is_dir) {
            let path = join(dir, &candidate);
            let Some(reservation) = reserved.reserve(Place { remote: true, path: path.clone() }) else { continue };
            match self.sftp.symlink_metadata(path).await {
                Ok(_) => continue,
                Err(SftpError::Status(s)) if s.status_code == StatusCode::NoSuchFile => return Ok((candidate, reservation)),
                Err(e) => return Err(self.failure(e, fallback)),
            }
        }
        Err(Error::AlreadyExists)
    }

    /// Traduce un error de lectura o escritura en un archivo remoto.
    fn io_failure(&self, err: io::Error) -> Error {
        match err.into_inner().map(|inner| inner.downcast::<SftpError>()) {
            Some(Ok(sftp)) => self.failure(*sftp, Error::TransferFailed),
            _ if self.ssh.is_closed() => Error::ConnectionLost,
            // Estado de error del Servidor sin código, como un disco lleno.
            _ => Error::TransferFailed,
        }
    }
}

/// Recorre lo que hay que subir, siguiendo los enlaces salvo los que llevan a
/// carpetas dentro de una carpeta, para no entrar en bucles.
async fn local_plan(local: &Path) -> Result<Plan> {
    let meta = fs::metadata(local).await.map_err(|_| Error::LocalUnreadable)?;
    if !meta.is_dir() {
        return Ok(Plan::new(Some(meta.len())));
    }
    let mut plan = Plan::new(None);
    let mut pending = vec![Vec::new()];
    while let Some(dir) = pending.pop() {
        let path = dir.iter().fold(local.to_path_buf(), |p: PathBuf, c: &String| p.join(c));
        let mut entries = fs::read_dir(&path).await.map_err(|_| Error::LocalUnreadable)?;
        while let Some(e) = entries.next_entry().await.map_err(|_| Error::LocalUnreadable)? {
            let name = e.file_name().into_string().map_err(|_| Error::LocalUnreadable)?;
            let link = e.file_type().await.map_err(|_| Error::LocalUnreadable)?.is_symlink();
            let Ok(meta) = fs::metadata(e.path()).await else {
                if link {
                    continue; // enlace roto
                }
                return Err(Error::LocalUnreadable);
            };
            let mut child = dir.clone();
            child.push(name);
            if meta.is_dir() {
                if !link {
                    pending.push(child.clone());
                    plan.push(child, None);
                }
            } else {
                plan.push(child, Some(meta.len()));
            }
        }
    }
    Ok(plan)
}

/// Primer nombre libre para `name` en la carpeta `dir` del equipo, reservado.
async fn free_local_path(dir: &Path, name: &str, is_dir: bool, reserved: &Reserved) -> Result<(String, Reservation)> {
    for candidate in numbered(name, is_dir) {
        let path = dir.join(&candidate).into_os_string().into_string().map_err(|_| Error::LocalWriteFailed)?;
        let Some(reservation) = reserved.reserve(Place { remote: false, path: path.clone() }) else { continue };
        if !fs::try_exists(&path).await.map_err(|_| Error::LocalWriteFailed)? {
            return Ok((candidate, reservation));
        }
    }
    Err(Error::AlreadyExists)
}

/// `name`, `name (1)`, `name (2)`… con el número antes de la extensión en los
/// archivos (`foto (1).jpg`).
pub(super) fn numbered(name: &str, is_dir: bool) -> impl Iterator<Item = String> + '_ {
    let (stem, ext) = match name.rfind('.') {
        Some(i) if i > 0 && !is_dir => name.split_at(i),
        _ => (name, ""),
    };
    std::iter::once(name.to_string()).chain((1..MAX_NUMBERED).map(move |n| format!("{stem} ({n}){ext}")))
}

/// Nombre provisional, oculto, de un archivo a medio copiar: `dir/.nombre.part`.
fn partial_name(path: &str) -> String {
    let split = path.rfind(['/', '\\']).map_or(0, |i| i + 1);
    format!("{}.{}.part", &path[..split], &path[split..])
}
