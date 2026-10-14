//! Papelera: lo que se borra desde la app se mueve a la Papelera del Miembro
//! en su carpeta personal del Servidor y se puede restaurar a su sitio.
//!
//! Sigue el formato de freedesktop.org (`~/.local/share/Trash`), así que las
//! herramientas del Servidor (`trash-cli`, `gio`) la entienden: cada elemento
//! está en `files/<id>` y de dónde vino, en `info/<id>.trashinfo`. Crear el
//! `.trashinfo` en exclusiva es lo que reserva `<id>`.

use std::time::{SystemTime, UNIX_EPOCH};

use russh_sftp::client::error::Error as SftpError;
use russh_sftp::client::fs::File;
use russh_sftp::protocol::{OpenFlags, StatusCode};
use serde::Serialize;
use tokio::io::AsyncWriteExt;

use super::transfer::{Reserved, numbered};
use super::{Connection, EntryKind, is_single_name, join};
use crate::error::{Error, Result};

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

    fn lost_or(&self, fallback: Error) -> Error {
        if self.ssh.is_closed() { Error::ConnectionLost } else { fallback }
    }

    fn check_connection(&self) -> Result<()> {
        if self.ssh.is_closed() { Err(Error::ConnectionLost) } else { Ok(()) }
    }
}

/// Si una ruta es la otra o está dentro de ella.
fn overlaps(a: &str, b: &str) -> bool {
    let (a, b) = (a.trim_end_matches('/'), b.trim_end_matches('/'));
    let inside = |inner: &str, outer: &str| inner.strip_prefix(outer).is_some_and(|rest| rest.is_empty() || rest.starts_with('/'));
    inside(a, b) || inside(b, a) || a.is_empty() || b.is_empty()
}

/// Carpeta y nombre de una ruta absoluta.
fn split_path(path: &str) -> Option<(&str, &str)> {
    let slash = path.rfind('/')?;
    let (dir, name) = (if slash == 0 { "/" } else { &path[..slash] }, &path[slash + 1..]);
    (path.starts_with('/') && is_single_name(name)).then_some((dir, name))
}

/// Ruta y fecha de borrado de un `.trashinfo`.
fn parse_info(text: &str) -> Option<(String, Option<u64>)> {
    let mut in_group = false;
    let (mut path, mut deleted) = (None, None);
    for line in text.lines().map(str::trim) {
        if line.starts_with('[') {
            in_group = line == "[Trash Info]";
        } else if in_group {
            match line.split_once('=') {
                Some(("Path", value)) => path = decode_path(value),
                Some(("DeletionDate", value)) => deleted = parse_date(value),
                _ => {}
            }
        }
    }
    path.map(|p| (p, deleted))
}

/// Ruta codificada como en una URL, salvo las `/`.
fn encode_path(path: &str) -> String {
    let mut out = String::new();
    for &b in path.as_bytes() {
        if b.is_ascii_alphanumeric() || b"-._~/".contains(&b) {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

fn decode_path(text: &str) -> Option<String> {
    let mut bytes = Vec::new();
    let mut rest = text.as_bytes();
    while let Some((&b, tail)) = rest.split_first() {
        if b == b'%' {
            let hex = std::str::from_utf8(tail.get(..2)?).ok()?;
            bytes.push(u8::from_str_radix(hex, 16).ok()?);
            rest = &tail[2..];
        } else {
            bytes.push(b);
            rest = tail;
        }
    }
    String::from_utf8(bytes).ok()
}

fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs())
}

/// Fecha en el formato de `DeletionDate` (`2026-10-14T09:30:00`). La
/// especificación la pide en hora local, pero aquí va en UTC: el equipo del
/// Miembro y el Servidor pueden estar en zonas distintas y la app la lee igual.
fn format_date(secs: u64) -> String {
    let (y, m, d) = civil_from_days((secs / 86_400) as i64);
    let s = secs % 86_400;
    format!("{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}", s / 3600, s % 3600 / 60, s % 60)
}

/// Lee una `DeletionDate` como UTC; ignora fracciones de segundo o zona horaria.
fn parse_date(text: &str) -> Option<u64> {
    let num = |range: std::ops::Range<usize>| text.get(range)?.parse::<i64>().ok();
    let (y, m, d) = (num(0..4)?, num(5..7)?, num(8..10)?);
    let (h, min, s) = (num(11..13)?, num(14..16)?, num(17..19)?);
    let separators = [(4, b'-'), (7, b'-'), (10, b'T'), (13, b':'), (16, b':')];
    if !separators.iter().all(|&(i, c)| text.as_bytes()[i] == c) || !(1..=12).contains(&m) || !(1..=31).contains(&d) {
        return None;
    }
    u64::try_from(days_from_civil(y, m, d) * 86_400 + h * 3600 + min * 60 + s).ok()
}

/// Días desde 1970-01-01 de una fecha del calendario gregoriano (algoritmo de Howard Hinnant).
fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let doy = (153 * (m + if m > 2 { -3 } else { 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

fn civil_from_days(z: i64) -> (i64, i64, i64) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (yoe + era * 400 + i64::from(m <= 2), m, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paths_round_trip_through_the_encoding() {
        for path in ["/home/alice/notas.txt", "/srv/compartida/mis fotos/año 100%.jpg", "/a/b+c&d#e?"] {
            assert_eq!(decode_path(&encode_path(path)).as_deref(), Some(path));
        }
        assert_eq!(encode_path("/home/alice/mis fotos"), "/home/alice/mis%20fotos");
        assert_eq!(decode_path("/x/%C3%B1"), Some("/x/ñ".to_string()));
        assert_eq!(decode_path("/x/%Z1"), None);
        assert_eq!(decode_path("/x/%4"), None);
    }

    #[test]
    fn dates_round_trip() {
        for secs in [0, 951_782_400, 1_791_970_200, 4_102_444_799] {
            assert_eq!(parse_date(&format_date(secs)), Some(secs));
        }
        assert_eq!(format_date(1_791_970_200), "2026-10-14T09:30:00");
        assert_eq!(parse_date("2026-10-14T09:30:00.123+02:00"), Some(1_791_970_200));
        assert_eq!(parse_date("ayer"), None);
        assert_eq!(parse_date("2026-13-14T09:30:00"), None);
    }

    #[test]
    fn reads_a_trashinfo() {
        let text = "[Trash Info]\nPath=/home/alice/mis%20notas.txt\nDeletionDate=2026-10-14T09:30:00\n";
        assert_eq!(parse_info(text), Some(("/home/alice/mis notas.txt".to_string(), Some(1_791_970_200))));
        assert_eq!(parse_info("[Otra]\nPath=/x\n"), None);
    }

    #[test]
    fn splits_absolute_paths_only() {
        assert_eq!(split_path("/home/alice/notas"), Some(("/home/alice", "notas")));
        assert_eq!(split_path("/notas"), Some(("/", "notas")));
        assert_eq!(split_path("notas"), None);
        assert_eq!(split_path("/home/"), None);
        assert_eq!(split_path("/home/.."), None);
    }

    #[test]
    fn detects_paths_inside_one_another() {
        let trash = "/home/alice/.local/share/Trash";
        assert!(overlaps("/home/alice/.local", trash));
        assert!(overlaps("/home/alice/.local/share/Trash/files/x", trash));
        assert!(overlaps("/", trash));
        assert!(!overlaps("/home/alice/.locale", trash));
        assert!(!overlaps("/home/alice/notas", trash));
    }
}
