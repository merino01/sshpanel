//! Transferencias: subir y descargar archivos y carpetas enteras.
//!
//! Cada archivo se copia primero a un nombre provisional y solo toma su nombre
//! definitivo cuando está completo, así que un archivo con su nombre nunca está
//! a medias. Nunca se sustituye nada: si el nombre ya existe se usa
//! `nombre (1)`, `nombre (2)`…

use std::io;
use std::path::{Path, PathBuf};

use russh_sftp::client::error::Error as SftpError;
use russh_sftp::protocol::StatusCode;
use serde::Serialize;
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

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

pub(crate) type Report<'a> = &'a mut (dyn FnMut(Progress) + Send);

/// Algo que copiar, por su ruta dentro de la Transferencia (vacía para la
/// raíz). `size` es `None` para las carpetas.
struct Item {
    path: Vec<String>,
    size: Option<u64>,
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

struct Counter<'a> {
    done: u64,
    total: u64,
    report: Report<'a>,
}

impl Counter<'_> {
    fn add(&mut self, bytes: usize) {
        self.done += bytes as u64;
        (self.report)(Progress { done: self.done, total: self.total });
    }
}

impl Connection {
    /// Sube el archivo o la carpeta `local` a la carpeta `remote_dir`. Devuelve
    /// el nombre con el que ha quedado en el Servidor.
    pub(crate) async fn upload(&self, local: &Path, remote_dir: &str, report: Report<'_>) -> Result<String> {
        let name = local.file_name().and_then(|n| n.to_str()).filter(|n| is_single_name(n)).ok_or(Error::LocalUnreadable)?;
        let plan = local_plan(local).await?;
        let name = self.free_remote_name(remote_dir, name, plan.is_dir()).await?;
        let root = join(remote_dir, &name);

        let mut counter = Counter { done: 0, total: plan.total, report };
        (counter.report)(Progress { done: 0, total: plan.total });
        for item in &plan.items {
            let remote = item.path.iter().fold(root.clone(), |p, c| join(&p, c));
            let local = item.path.iter().fold(local.to_path_buf(), |p, c| p.join(c));
            match item.size {
                None => self.sftp.create_dir(remote).await.map_err(|e| self.failure(e, Error::TransferFailed))?,
                Some(_) => self.upload_file(&local, &remote, &mut counter).await?,
            }
        }
        Ok(name)
    }

    /// Descarga el archivo o la carpeta `name` de `remote_dir` a la carpeta
    /// local `local_dir`. Devuelve dónde ha quedado.
    pub(crate) async fn download(&self, remote_dir: &str, name: &str, local_dir: &Path, report: Report<'_>) -> Result<PathBuf> {
        if !is_single_name(name) {
            return Err(Error::InvalidName);
        }
        let source = join(remote_dir, name);
        let plan = self.remote_plan(&source).await?;
        let root = free_local_path(local_dir, name, plan.is_dir()).await?;

        let mut counter = Counter { done: 0, total: plan.total, report };
        (counter.report)(Progress { done: 0, total: plan.total });
        for item in &plan.items {
            let remote = item.path.iter().fold(source.clone(), |p, c| join(&p, c));
            let local = item.path.iter().fold(root.clone(), |p, c| p.join(c));
            match item.size {
                None => fs::create_dir(&local).await.map_err(|_| Error::LocalWriteFailed)?,
                Some(_) => self.download_file(&remote, &local, &mut counter).await?,
            }
        }
        Ok(root)
    }

    async fn upload_file(&self, local: &Path, remote: &str, counter: &mut Counter<'_>) -> Result<()> {
        let partial = partial_name(remote);
        let result = async {
            let mut source = fs::File::open(local).await.map_err(|_| Error::LocalUnreadable)?;
            let mut target = self.sftp.create(partial.as_str()).await.map_err(|e| self.failure(e, Error::TransferFailed))?;
            let mut buf = vec![0; CHUNK];
            loop {
                let n = source.read(&mut buf).await.map_err(|_| Error::LocalUnreadable)?;
                if n == 0 {
                    break;
                }
                target.write_all(&buf[..n]).await.map_err(|e| self.io_failure(e))?;
                counter.add(n);
            }
            target.shutdown().await.map_err(|e| self.io_failure(e))?;
            self.ensure_free(remote).await?;
            self.sftp.rename(partial.as_str(), remote).await.map_err(|e| self.failure(e, Error::TransferFailed))
        }
        .await;
        // Sin conexión no se puede limpiar; el provisional queda oculto.
        if result.as_ref().is_err_and(|e| *e != Error::ConnectionLost) {
            let _ = self.sftp.remove_file(partial).await;
        }
        result
    }

    async fn download_file(&self, remote: &str, local: &Path, counter: &mut Counter<'_>) -> Result<()> {
        let partial = PathBuf::from(partial_name(local.to_str().ok_or(Error::LocalWriteFailed)?));
        let result = async {
            let mut source = self.sftp.open(remote).await.map_err(|e| self.failure(e, Error::TransferFailed))?;
            let mut target = fs::File::create(&partial).await.map_err(|_| Error::LocalWriteFailed)?;
            let mut buf = vec![0; CHUNK];
            loop {
                let n = source.read(&mut buf).await.map_err(|e| self.io_failure(e))?;
                if n == 0 {
                    break;
                }
                target.write_all(&buf[..n]).await.map_err(|_| Error::LocalWriteFailed)?;
                counter.add(n);
            }
            target.sync_all().await.map_err(|_| Error::LocalWriteFailed)?;
            drop(target);
            if fs::try_exists(local).await.unwrap_or(true) {
                return Err(Error::AlreadyExists);
            }
            fs::rename(&partial, local).await.map_err(|_| Error::LocalWriteFailed)
        }
        .await;
        if result.is_err() {
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

    async fn free_remote_name(&self, dir: &str, name: &str, is_dir: bool) -> Result<String> {
        for candidate in numbered(name, is_dir) {
            match self.sftp.symlink_metadata(join(dir, &candidate)).await {
                Ok(_) => continue,
                Err(SftpError::Status(s)) if s.status_code == StatusCode::NoSuchFile => return Ok(candidate),
                Err(e) => return Err(self.failure(e, Error::TransferFailed)),
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

async fn free_local_path(dir: &Path, name: &str, is_dir: bool) -> Result<PathBuf> {
    for candidate in numbered(name, is_dir) {
        let path = dir.join(candidate);
        if !fs::try_exists(&path).await.map_err(|_| Error::LocalWriteFailed)? {
            return Ok(path);
        }
    }
    Err(Error::AlreadyExists)
}

/// `name`, `name (1)`, `name (2)`… con el número antes de la extensión en los
/// archivos (`foto (1).jpg`).
fn numbered(name: &str, is_dir: bool) -> impl Iterator<Item = String> + '_ {
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
