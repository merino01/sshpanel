use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use russh::client::{self, Handle};
use russh::keys::{HashAlg, PrivateKeyWithHashAlg, PublicKeyOrCertificate};
use russh_sftp::client::SftpSession;
use russh_sftp::client::error::Error as SftpError;
use russh_sftp::protocol::StatusCode;
use serde::Serialize;

use crate::error::{Error, Result};
use crate::profile::Profile;

const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

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

/// Acepta la clave del Servidor solo si su Huella es la del Perfil. Se ejecuta
/// antes de autenticarse, así que si no coincide no se envía usuario ni clave.
struct Verifier {
    fingerprint: String,
    mismatch: Arc<AtomicBool>,
}

impl client::Handler for Verifier {
    type Error = russh::Error;

    async fn check_server_key(&mut self, server_key: &PublicKeyOrCertificate) -> std::result::Result<bool, Self::Error> {
        let key = match server_key {
            PublicKeyOrCertificate::PublicKey { key, .. } => key,
            PublicKeyOrCertificate::Certificate(_) => {
                self.mismatch.store(true, Ordering::SeqCst);
                return Ok(false);
            }
        };
        let matches = key.fingerprint(HashAlg::Sha256).to_string() == self.fingerprint;
        self.mismatch.store(!matches, Ordering::SeqCst);
        Ok(matches)
    }
}

/// Sesión SFTP abierta con el Servidor.
pub(crate) struct Connection {
    ssh: Handle<Verifier>,
    sftp: SftpSession,
    pub(crate) home: String,
}

impl Connection {
    pub(crate) async fn open(profile: &Profile, private_key_pem: &str) -> Result<Self> {
        let key = russh::keys::decode_secret_key(private_key_pem, None).map_err(|_| Error::Storage)?;
        let mismatch = Arc::new(AtomicBool::new(false));
        let verifier = Verifier { fingerprint: profile.fingerprint.clone(), mismatch: mismatch.clone() };
        let config = Arc::new(client::Config::default());

        let handshake = client::connect(config, (profile.host.as_str(), profile.port), verifier);
        let mut ssh = match tokio::time::timeout(CONNECT_TIMEOUT, handshake).await {
            Ok(Ok(ssh)) => ssh,
            _ if mismatch.load(Ordering::SeqCst) => return Err(Error::FingerprintMismatch),
            _ => return Err(Error::Unreachable),
        };

        let auth = ssh.authenticate_publickey(profile.username.as_str(), PrivateKeyWithHashAlg::new(Arc::new(key), None));
        match tokio::time::timeout(CONNECT_TIMEOUT, auth).await {
            Ok(Ok(result)) if result.success() => {}
            Ok(Ok(_)) => return Err(Error::Unauthorized),
            _ => return Err(Error::Unreachable),
        }

        let sftp = async {
            let channel = ssh.channel_open_session().await.ok()?;
            channel.request_subsystem(true, "sftp").await.ok()?;
            SftpSession::new(channel.into_stream()).await.ok()
        };
        let sftp = tokio::time::timeout(CONNECT_TIMEOUT, sftp).await.ok().flatten().ok_or(Error::ConnectionLost)?;
        let home = sftp.canonicalize(".").await.map_err(|_| Error::ConnectionLost)?;
        Ok(Self { ssh, sftp, home })
    }

    /// Lista una carpeta. Los enlaces se muestran como lo que apuntan, para
    /// poder entrar en un enlace a una carpeta; un enlace roto queda como enlace.
    pub(crate) async fn list(&self, path: &str) -> Result<Listing> {
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
        let path = join(parent, new_name(name)?);
        self.ensure_free(&path).await?;
        self.sftp.create_dir(path).await.map_err(|e| self.failure(e, Error::CreateFolderFailed))
    }

    pub(crate) async fn rename(&self, dir: &str, from: &str, to: &str) -> Result<()> {
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

    /// Falla si ya hay algo en `path`. El Servidor sustituiría sin avisar una
    /// carpeta vacía al renombrar, así que se comprueba antes.
    async fn ensure_free(&self, path: &str) -> Result<()> {
        match self.sftp.symlink_metadata(path).await {
            Ok(_) => Err(Error::AlreadyExists),
            Err(SftpError::Status(s)) if s.status_code == StatusCode::NoSuchFile => Ok(()),
            Err(e) => Err(self.failure(e, Error::ListFailed)),
        }
    }

    /// Traduce un error SFTP, distinguiendo una conexión caída, un permiso
    /// denegado o algo que no existe de un fallo genérico de la operación.
    fn failure(&self, err: SftpError, fallback: Error) -> Error {
        if self.ssh.is_closed() {
            return Error::ConnectionLost;
        }
        match err {
            SftpError::Status(s) if s.status_code == StatusCode::PermissionDenied => Error::PermissionDenied,
            SftpError::Status(s) if s.status_code == StatusCode::NoSuchFile => Error::NotFound,
            _ => fallback,
        }
    }
}

/// Ruta de `name` dentro de la carpeta `dir`.
fn join(dir: &str, name: &str) -> String {
    format!("{}/{name}", dir.trim_end_matches('/'))
}

fn is_single_name(name: &str) -> bool {
    !name.is_empty() && name != "." && name != ".." && !name.contains(['/', '\0'])
}

/// Nombre que escribe el Miembro, sin espacios alrededor.
fn new_name(name: &str) -> Result<&str> {
    let name = name.trim();
    if is_single_name(name) { Ok(name) } else { Err(Error::InvalidName) }
}
