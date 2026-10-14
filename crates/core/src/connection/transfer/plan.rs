//! Qué copia una Transferencia: el archivo o la carpeta raíz y, si es una
//! carpeta, todo lo que contiene.

use std::path::{Path, PathBuf};

use tokio::fs;

use super::super::Connection;
use super::super::paths::join;
use crate::error::{Error, Result};

/// Algo que copiar, por su ruta dentro de la Transferencia (vacía para la
/// raíz). `size` es `None` para las carpetas.
pub(super) struct Item {
    pub(super) path: Vec<String>,
    pub(super) size: Option<u64>,
}

impl Item {
    pub(super) fn remote(&self, root: &str) -> String {
        self.path.iter().fold(root.to_string(), |p, c| join(&p, c))
    }

    pub(super) fn local(&self, root: &Path) -> PathBuf {
        self.path.iter().fold(root.to_path_buf(), |p, c| p.join(c))
    }
}

/// Lo que hay que copiar, cada carpeta antes que su contenido.
pub(super) struct Plan {
    pub(super) items: Vec<Item>,
    pub(super) total: u64,
}

impl Plan {
    fn new(root: Option<u64>) -> Self {
        Self { items: vec![Item { path: Vec::new(), size: root }], total: root.unwrap_or(0) }
    }

    pub(super) fn is_dir(&self) -> bool {
        self.items[0].size.is_none()
    }

    fn push(&mut self, path: Vec<String>, size: Option<u64>) {
        self.total += size.unwrap_or(0);
        self.items.push(Item { path, size });
    }
}

impl Connection {
    /// Recorre lo que hay que descargar. Dentro de una carpeta, los enlaces a
    /// carpetas y los enlaces rotos se omiten para no entrar en bucles.
    pub(super) async fn remote_plan(&self, source: &str) -> Result<Plan> {
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
}

/// Recorre lo que hay que subir, siguiendo los enlaces salvo los que llevan a
/// carpetas dentro de una carpeta, para no entrar en bucles.
pub(super) async fn local_plan(local: &Path) -> Result<Plan> {
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
