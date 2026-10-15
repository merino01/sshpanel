//! Editor: leer un archivo de texto del Servidor y guardarlo de vuelta.
//! Qué cuenta como texto lo decide la interfaz con el principio del archivo
//! ([`Connection::read_head`]); aquí solo se exige que sea UTF-8, para poder
//! devolverlo como texto sin estropearlo.

use russh_sftp::client::fs::{File, Metadata};
use russh_sftp::protocol::OpenFlags;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use super::Connection;
use super::naming::Reserved;
use super::paths::{join, split_path};
use crate::error::{Error, Result};

/// Cuánto se lee del principio de un archivo para decidir si es de texto.
const HEAD: usize = 8 * 1024;

/// Tamaño y fecha de modificación de un archivo al leerlo o guardarlo. Si al
/// volver a guardar ya no coinciden, otro lo ha cambiado en el Servidor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

/// La copia que se ha guardado al lado de un archivo.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TextCopy {
    pub path: String,
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
    ///
    /// Con `expected`, falla con [`Error::Conflict`] sin tocar nada si el
    /// archivo ya no tiene ese [`Stamp`]. No es atómico: un cambio justo entre
    /// la comprobación y la escritura, o uno del mismo tamaño en el mismo
    /// segundo, no se detecta.
    pub(crate) async fn write_text(&self, path: &str, text: &str, expected: Option<Stamp>) -> Result<Stamp> {
        self.check_file(path)?;
        if let Some(expected) = expected {
            let meta = self.sftp.metadata(path).await.map_err(|e| self.failure(e, Error::WriteFailed))?;
            if Stamp::from(&meta) != expected {
                return Err(Error::Conflict);
            }
        }
        let file = self
            .sftp
            .open_with_flags(path, OpenFlags::WRITE | OpenFlags::TRUNCATE)
            .await
            .map_err(|e| self.failure(e, Error::WriteFailed))?;
        self.write_all(file, text).await
    }

    /// Guarda `text` en un archivo nuevo al lado de `path`, con el primer
    /// nombre libre (`notas (1).txt`…). Nunca sobrescribe nada.
    pub(crate) async fn write_text_copy(&self, path: &str, text: &str, reserved: &Reserved) -> Result<TextCopy> {
        self.check_file(path)?;
        let (dir, name) = split_path(path).ok_or(Error::InvalidName)?;
        let (name, _reservation) = self.free_remote_name(dir, name, false, reserved, Error::WriteFailed).await?;
        let path = join(dir, &name);
        let file = self
            .sftp
            .open_with_flags(&path, OpenFlags::CREATE | OpenFlags::EXCLUDE | OpenFlags::WRITE)
            .await
            .map_err(|e| self.failure(e, Error::WriteFailed))?;
        let stamp = self.write_all(file, text).await?;
        Ok(TextCopy { path, stamp })
    }

    /// Escribe `text` en `file`, abierto para escribir, lo cierra y devuelve su [`Stamp`].
    async fn write_all(&self, mut file: File, text: &str) -> Result<Stamp> {
        file.write_all(text.as_bytes()).await.map_err(|e| self.io_failure(e, Error::WriteFailed))?;
        file.flush().await.map_err(|e| self.io_failure(e, Error::WriteFailed))?;
        let meta = file.metadata().await.map_err(|e| self.failure(e, Error::WriteFailed))?;
        file.shutdown().await.map_err(|e| self.io_failure(e, Error::WriteFailed))?;
        Ok(Stamp::from(&meta))
    }
}
