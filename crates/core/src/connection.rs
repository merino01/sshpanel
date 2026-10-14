use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use russh::client::{self, Handle};
use russh::keys::{HashAlg, PrivateKeyWithHashAlg, PublicKeyOrCertificate};
use russh_sftp::client::SftpSession;
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

    pub(crate) async fn list(&self, path: &str) -> Result<Listing> {
        let dir = self.sftp.read_dir(path).await.map_err(|_| self.failure(Error::ListFailed))?;
        let mut entries: Vec<Entry> = dir
            .filter(|e| !matches!(e.file_name().as_str(), "." | ".."))
            .map(|e| {
                let meta = e.metadata();
                let file_type = e.file_type();
                let kind = if file_type.is_dir() {
                    EntryKind::Dir
                } else if file_type.is_symlink() {
                    EntryKind::Symlink
                } else {
                    EntryKind::File
                };
                Entry { name: e.file_name(), kind, size: meta.len(), modified: meta.mtime.map(u64::from) }
            })
            .collect();
        entries.sort_by(|a, b| (a.kind != EntryKind::Dir).cmp(&(b.kind != EntryKind::Dir)).then_with(|| a.name.cmp(&b.name)));
        Ok(Listing { path: path.to_string(), entries })
    }

    /// Distingue un fallo de la operación de una conexión caída.
    fn failure(&self, err: Error) -> Error {
        if self.ssh.is_closed() { Error::ConnectionLost } else { err }
    }
}
