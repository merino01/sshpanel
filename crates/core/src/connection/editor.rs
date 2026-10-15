//! Editor: leer un archivo de texto del Servidor y guardarlo de vuelta.
//! Qué cuenta como texto lo decide la interfaz con el principio del archivo
//! ([`Connection::read_head`]); aquí solo se exige que sea UTF-8, para poder
//! devolverlo como texto sin estropearlo.

use russh_sftp::client::fs::Metadata;
use russh_sftp::protocol::OpenFlags;
use serde::Serialize;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use super::Connection;
use super::paths::split_path;
use crate::error::{Error, Result};

/// Cuánto se lee del principio de un archivo para decidir si es de texto.
const HEAD: usize = 8 * 1024;

/// Tamaño y fecha de modificación de un archivo al leerlo o guardarlo. Si al
/// volver a guardar ya no coinciden, otro lo ha cambiado en el Servidor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Stamp {
    pub size: u64,
    /// Segundos desde la época Unix.
    pub modified: Option<u64>,
}

impl From<&Metadata> for Stamp {
    fn from(meta: &Metadata) -> Self {
        Self { size: meta.len(), modified: meta.mtime.map(u64::from) }
    }
}

/// El principio de un archivo, para decidir si es de texto antes de abrirlo.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Head {
    /// Como mucho los primeros 8 KB.
    pub bytes: Vec<u8>,
    /// Tamaño del archivo entero.
    pub size: u64,
}

/// Un archivo de texto leído entero.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TextFile {
    pub text: String,
    pub stamp: Stamp,
}

impl Connection {
    /// En Modo simple, falla si el archivo `path` queda fuera de las carpetas
    /// permitidas. `path` tiene que ser una ruta absoluta a un nombre.
    fn check_file(&self, path: &str) -> Result<()> {
        let (dir, _) = split_path(path).ok_or(Error::InvalidName)?;
        self.check_allowed(dir)
    }

    pub(crate) async fn read_head(&self, path: &str) -> Result<Head> {
        self.check_file(path)?;
        let mut file = self.sftp.open(path).await.map_err(|e| self.failure(e, Error::ReadFailed))?;
        let meta = file.metadata().await.map_err(|e| self.failure(e, Error::ReadFailed))?;
        if meta.is_dir() {
            return Err(Error::ReadFailed);
        }
        let mut bytes = vec![0; HEAD];
        let mut filled = 0;
        while filled < HEAD {
            let n = file.read(&mut bytes[filled..]).await.map_err(|e| self.io_failure(e, Error::ReadFailed))?;
            if n == 0 {
                break;
            }
            filled += n;
        }
        bytes.truncate(filled);
        Ok(Head { bytes, size: meta.len() })
    }

    pub(crate) async fn read_text(&self, path: &str) -> Result<TextFile> {
        self.check_file(path)?;
        let mut file = self.sftp.open(path).await.map_err(|e| self.failure(e, Error::ReadFailed))?;
        let meta = file.metadata().await.map_err(|e| self.failure(e, Error::ReadFailed))?;
        if meta.is_dir() {
            return Err(Error::ReadFailed);
        }
        let mut bytes = Vec::with_capacity(meta.len().try_into().unwrap_or(0));
        file.read_to_end(&mut bytes).await.map_err(|e| self.io_failure(e, Error::ReadFailed))?;
        let text = String::from_utf8(bytes).map_err(|_| Error::NotText)?;
        Ok(TextFile { text, stamp: Stamp::from(&meta) })
    }

    /// Sustituye el contenido del archivo `path`, que tiene que existir. Se
    /// escribe en el mismo archivo, no en uno nuevo, para que conserve sus
    /// permisos y su propietario (en la Carpeta compartida suele ser de otro).
    pub(crate) async fn write_text(&self, path: &str, text: &str) -> Result<Stamp> {
        self.check_file(path)?;
        let mut file = self
            .sftp
            .open_with_flags(path, OpenFlags::WRITE | OpenFlags::TRUNCATE)
            .await
            .map_err(|e| self.failure(e, Error::WriteFailed))?;
        file.write_all(text.as_bytes()).await.map_err(|e| self.io_failure(e, Error::WriteFailed))?;
        file.flush().await.map_err(|e| self.io_failure(e, Error::WriteFailed))?;
        let meta = file.metadata().await.map_err(|e| self.failure(e, Error::WriteFailed))?;
        file.shutdown().await.map_err(|e| self.io_failure(e, Error::WriteFailed))?;
        Ok(Stamp::from(&meta))
    }
}
