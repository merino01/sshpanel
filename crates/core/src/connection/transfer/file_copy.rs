//! Copia de un archivo, pasando por un provisional oculto y continuando
//! donde se quedó un intento anterior.

use std::io::{self, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

use russh_sftp::client::error::Error as SftpError;
use russh_sftp::protocol::OpenFlags;
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};

use super::super::Connection;
use super::{Progress, Report};
use crate::error::{Error, Result};

const CHUNK: usize = 256 * 1024;

/// Lleva la cuenta de los bytes copiados, avisa del avance y comprueba si se
/// ha pedido cancelar.
pub(super) struct Counter<'a> {
    pub(super) done: u64,
    pub(super) total: u64,
    pub(super) report: Report<'a>,
    pub(super) cancelled: &'a AtomicBool,
}

impl Counter<'_> {
    pub(super) fn report(&mut self) {
        (self.report)(Progress { done: self.done, total: self.total });
    }

    fn add(&mut self, bytes: usize) -> Result<()> {
        self.done += bytes as u64;
        self.report();
        self.check()
    }

    pub(super) fn check(&self) -> Result<()> {
        if self.cancelled.load(Ordering::SeqCst) { Err(Error::Cancelled) } else { Ok(()) }
    }
}

impl Connection {
    /// Copia `local` a `remote` pasando por el provisional, continuando desde
    /// el byte `offset` si un intento anterior lo dejó a medias.
    pub(super) async fn upload_file(&self, local: &Path, remote: &str, offset: u64, counter: &mut Counter<'_>) -> Result<()> {
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
    pub(super) async fn download_file(&self, remote: &str, local: &Path, offset: u64, counter: &mut Counter<'_>) -> Result<()> {
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

/// Nombre provisional, oculto, de un archivo a medio copiar: `dir/.nombre.part`.
pub(super) fn partial_name(path: &str) -> String {
    let split = path.rfind(['/', '\\']).map_or(0, |i| i + 1);
    format!("{}.{}.part", &path[..split], &path[split..])
}
