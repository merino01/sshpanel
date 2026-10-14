//! Explorador: listar carpetas, crear carpetas y renombrar.

use serde::Serialize;

use super::Connection;
use super::paths::{is_single_name, join, new_name};
use crate::error::{Error, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum EntryKind {
    File,
    Dir,
    Symlink,
}

/// Un archivo o carpeta del Servidor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Entry {
    pub name: String,
    pub kind: EntryKind,
    pub size: u64,
    /// Segundos desde la época Unix.
    pub modified: Option<u64>,
}

/// Contenido de una carpeta del Servidor: carpetas primero, luego por nombre.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Listing {
    pub path: String,
    pub entries: Vec<Entry>,
}

impl Connection {
    /// Lista una carpeta. Los enlaces se muestran como lo que apuntan, para
    /// poder entrar en un enlace a una carpeta; un enlace roto queda como enlace.
    pub(crate) async fn list(&self, path: &str) -> Result<Listing> {
        self.check_allowed(path)?;
        let dir = self.sftp.read_dir(path).await.map_err(|e| self.failure(e, Error::ListFailed))?;
        let mut entries = Vec::new();
        for e in dir.filter(|e| !matches!(e.file_name().as_str(), "." | "..")) {
            let mut meta = e.metadata();
            let mut kind = EntryKind::File;
            if e.file_type().is_symlink() {
                match self.sftp.metadata(join(path, &e.file_name())).await {
                    Ok(target) => meta = target,
                    Err(_) => kind = EntryKind::Symlink,
                }
            }
            if meta.is_dir() {
                kind = EntryKind::Dir;
            }
            entries.push(Entry { name: e.file_name(), kind, size: meta.len(), modified: meta.mtime.map(u64::from) });
        }
        entries.sort_by(|a, b| (a.kind != EntryKind::Dir).cmp(&(b.kind != EntryKind::Dir)).then_with(|| a.name.cmp(&b.name)));
        Ok(Listing { path: path.to_string(), entries })
    }

    pub(crate) async fn create_folder(&self, parent: &str, name: &str) -> Result<()> {
        self.check_allowed(parent)?;
        let path = join(parent, new_name(name)?);
        self.ensure_free(&path).await?;
        self.sftp.create_dir(path).await.map_err(|e| self.failure(e, Error::CreateFolderFailed))
    }

    pub(crate) async fn rename(&self, dir: &str, from: &str, to: &str) -> Result<()> {
        self.check_allowed(dir)?;
        if !is_single_name(from) {
            return Err(Error::InvalidName);
        }
        let to = new_name(to)?;
        let source = join(dir, from);
        self.sftp.symlink_metadata(source.as_str()).await.map_err(|e| self.failure(e, Error::RenameFailed))?;
        if from == to {
            return Ok(());
        }
        let target = join(dir, to);
        self.ensure_free(&target).await?;
        self.sftp.rename(source, target).await.map_err(|e| self.failure(e, Error::RenameFailed))
    }
}
