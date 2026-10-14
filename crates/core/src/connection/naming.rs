//! Reserva de nombres: elegir `nombre`, `nombre (1)`, `nombre (2)`… sin
//! sustituir nada, y sin que dos operaciones en paralelo elijan el mismo.

use std::collections::HashSet;
use std::path::Path;
use std::sync::{Arc, Mutex};

use russh_sftp::client::error::Error as SftpError;
use russh_sftp::protocol::StatusCode;
use tokio::fs;

use super::Connection;
use super::paths::join;
use crate::error::{Error, Result};

const MAX_NUMBERED: u32 = 1000;

/// Un destino en el Servidor o en el equipo.
#[derive(Clone, PartialEq, Eq, Hash)]
struct Place {
    remote: bool,
    path: String,
}

/// Destinos elegidos por las operaciones en curso (Transferencias, restaurar),
/// para que dos en paralelo con el mismo nombre no elijan el mismo.
#[derive(Clone, Default)]
pub(crate) struct Reserved(Arc<Mutex<HashSet<Place>>>);

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

impl Reservation {
    /// Ruta reservada.
    pub(super) fn path(&self) -> &str {
        &self.place.path
    }
}

impl Connection {
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
}

/// Primer nombre libre para `name` en la carpeta `dir` del equipo, reservado.
pub(super) async fn free_local_path(dir: &Path, name: &str, is_dir: bool, reserved: &Reserved) -> Result<(String, Reservation)> {
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
