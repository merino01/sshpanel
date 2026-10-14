//! Papelera: lo que se borra desde la app se mueve a la Papelera del Miembro
//! en su carpeta personal del Servidor y se puede restaurar a su sitio.
//!
//! Sigue el formato de freedesktop.org (`~/.local/share/Trash`), así que las
//! herramientas del Servidor (`trash-cli`, `gio`) la entienden: cada elemento
//! está en `files/<id>` y de dónde vino, en `info/<id>.trashinfo`. Crear el
//! `.trashinfo` en exclusiva es lo que reserva `<id>`.

use russh_sftp::client::error::Error as SftpError;
use russh_sftp::client::fs::File;
use russh_sftp::protocol::{OpenFlags, StatusCode};
use serde::Serialize;
use tokio::io::AsyncWriteExt;

use super::naming::{Reserved, numbered};
use super::paths::{is_single_name, join, overlaps, split_path};
use super::{Connection, EntryKind};
use crate::error::{Error, Result};

mod info;

use info::{encode_path, format_date, now, parse_info};

/// Papelera dentro de la carpeta personal.
const TRASH: &str = ".local/share/Trash";
const INFO_SUFFIX: &str = ".trashinfo";

/// Un archivo o carpeta de la Papelera.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Trashed {
    /// Con qué nombre está guardado en la Papelera; sirve para restaurarlo.
    pub id: String,
    /// Nombre que tenía.
    pub name: String,
    /// Carpeta de la que vino.
    pub original_dir: String,
    pub kind: EntryKind,
    pub size: u64,
    /// Cuándo se borró, en segundos desde la época Unix.
    pub deleted: Option<u64>,
}

/// Dónde ha quedado un archivo o carpeta restaurado.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Restored {
    pub dir: String,
    /// Puede no ser el original si ya había algo con ese nombre.
    pub name: String,
}

impl Connection {
    fn trash_dir(&self, part: &str) -> String {
        join(&join(&self.home, TRASH), part)
    }

    pub(crate) async fn trash(&self, dir: &str, name: &str) -> Result<()> {
        if !is_single_name(name) {
            return Err(Error::InvalidName);
        }
        let source = join(dir, name);
        let meta = self.sftp.symlink_metadata(source.as_str()).await.map_err(|e| self.failure(e, Error::TrashFailed))?;
        if overlaps(&source, &join(&self.home, TRASH)) {
            return Err(Error::TrashProtected);
        }
        let (files, info) = (self.trash_dir("files"), self.trash_dir("info"));
        self.create_folders(&files, Error::TrashFailed).await?;
        self.create_folders(&info, Error::TrashFailed).await?;

        let (id, mut file) = self.claim(&files, &info, name, meta.is_dir()).await?;
        let info_path = join(&info, &format!("{id}{INFO_SUFFIX}"));
        let record = format!("[Trash Info]\nPath={}\nDeletionDate={}\n", encode_path(&source), format_date(now()));
        let result = async {
            file.write_all(record.as_bytes()).await.map_err(|_| self.lost_or(Error::TrashFailed))?;
            file.shutdown().await.map_err(|_| self.lost_or(Error::TrashFailed))?;
            self.sftp.rename(source.as_str(), join(&files, &id)).await.map_err(|e| self.failure(e, Error::TrashFailed))
        }
        .await;
        if result.is_err() {
            let _ = self.sftp.remove_file(info_path).await;
        }
        result
    }

    /// Reserva en la Papelera el primer nombre libre para `name`, creando su
    /// `.trashinfo`, que se devuelve abierto para escribirlo.
    async fn claim(&self, files: &str, info: &str, name: &str, is_dir: bool) -> Result<(String, File)> {
        for id in numbered(name, is_dir) {
            let info_path = join(info, &format!("{id}{INFO_SUFFIX}"));
            let flags = OpenFlags::CREATE | OpenFlags::EXCLUDE | OpenFlags::WRITE;
            match self.sftp.open_with_flags(info_path.as_str(), flags).await {
                // Puede quedar en `files/` algo sin su `.trashinfo`.
                Ok(file) => match self.ensure_free(&join(files, &id)).await {
                    Ok(()) => return Ok((id, file)),
                    Err(Error::AlreadyExists) => {
                        drop(file);
                        let _ = self.sftp.remove_file(info_path).await;
                    }
                    Err(e) => return Err(e),
                },
                // El Servidor no dice por qué no lo ha creado: si ya existe, se prueba el siguiente.
                Err(e) => match self.ensure_free(&info_path).await {
                    Err(Error::AlreadyExists) => {}
                    _ => return Err(self.failure(e, Error::TrashFailed)),
                },
            }
        }
        Err(Error::TrashFailed)
    }

    /// Contenido de la Papelera, lo último borrado primero. Lo que no se puede
    /// interpretar se omite.
    pub(crate) async fn list_trash(&self) -> Result<Vec<Trashed>> {
        let (files, info) = (self.trash_dir("files"), self.trash_dir("info"));
        let entries = match self.sftp.read_dir(info.as_str()).await {
            Ok(entries) => entries,
            Err(SftpError::Status(s)) if s.status_code == StatusCode::NoSuchFile => return Ok(Vec::new()),
            Err(e) => return Err(self.failure(e, Error::ListFailed)),
        };
        let mut items = Vec::new();
        for e in entries {
            let file_name = e.file_name();
            let Some(id) = file_name.strip_suffix(INFO_SUFFIX).filter(|id| is_single_name(id)) else { continue };
            let Ok(text) = self.sftp.read(join(&info, &file_name)).await else {
                self.check_connection()?;
                continue;
            };
            let Some((path, deleted)) = parse_info(&String::from_utf8_lossy(&text)) else { continue };
            let Some((dir, name)) = split_path(&path) else { continue };
            let Ok(meta) = self.sftp.symlink_metadata(join(&files, id)).await else {
                self.check_connection()?;
                continue;
            };
            let kind = match () {
                _ if meta.is_dir() => EntryKind::Dir,
                _ if meta.file_type().is_symlink() => EntryKind::Symlink,
                _ => EntryKind::File,
            };
            items.push(Trashed { id: id.to_string(), name: name.to_string(), original_dir: dir.to_string(), kind, size: meta.len(), deleted });
        }
        items.sort_by(|a, b| b.deleted.cmp(&a.deleted).then_with(|| a.name.cmp(&b.name)));
        Ok(items)
    }

    pub(crate) async fn restore(&self, id: &str, reserved: &Reserved) -> Result<Restored> {
        if !is_single_name(id) {
            return Err(Error::InvalidName);
        }
        let info_path = join(&self.trash_dir("info"), &format!("{id}{INFO_SUFFIX}"));
        let text = self.sftp.read(info_path.as_str()).await.map_err(|e| self.failure(e, Error::RestoreFailed))?;
        let (path, _) = parse_info(&String::from_utf8_lossy(&text)).ok_or(Error::RestoreFailed)?;
        let (dir, name) = split_path(&path).ok_or(Error::RestoreFailed)?;
        let source = join(&self.trash_dir("files"), id);
        let meta = self.sftp.symlink_metadata(source.as_str()).await.map_err(|e| self.failure(e, Error::RestoreFailed))?;

        self.create_folders(dir, Error::RestoreFailed).await?;
        let (name, _reservation) = self.free_remote_name(dir, name, meta.is_dir(), reserved, Error::RestoreFailed).await?;
        self.sftp.rename(source, join(dir, &name)).await.map_err(|e| self.failure(e, Error::RestoreFailed))?;
        let _ = self.sftp.remove_file(info_path).await;
        Ok(Restored { dir: dir.to_string(), name })
    }

    /// Crea las carpetas que falten hasta `path` (ruta absoluta).
    async fn create_folders(&self, path: &str, fallback: Error) -> Result<()> {
        let mut current = String::new();
        for part in path.split('/').filter(|p| !p.is_empty()) {
            current = format!("{current}/{part}");
            match self.sftp.metadata(current.as_str()).await {
                Ok(meta) if meta.is_dir() => {}
                Ok(_) => return Err(fallback),
                Err(SftpError::Status(s)) if s.status_code == StatusCode::NoSuchFile => {
                    self.sftp.create_dir(current.as_str()).await.map_err(|e| self.failure(e, fallback.clone()))?
                }
                Err(e) => return Err(self.failure(e, fallback)),
            }
        }
        Ok(())
    }
}
