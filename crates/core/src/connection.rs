//! Conexión con el Servidor: SSH con la Huella verificada y sesión SFTP. Las
//! operaciones sobre archivos viven en los submódulos, por área.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use russh::client::{self, Handle};
use russh::keys::{HashAlg, PrivateKeyWithHashAlg, PublicKeyOrCertificate};
use russh_sftp::client::SftpSession;
use russh_sftp::client::error::Error as SftpError;
use russh_sftp::protocol::StatusCode;

use crate::error::{Error, Result};
use crate::profile::{Mode, Profile};

mod explorer;
mod naming;
mod paths;
mod terminal;
mod transfer;
mod trash;

pub use explorer::{Entry, EntryKind, Listing};
pub(crate) use naming::Reserved;
use paths::is_within;
pub(crate) use terminal::Terminal;
pub use terminal::TerminalEvent;
pub(crate) use transfer::{discard, run};
pub use transfer::{Outcome, Progress, Transfer};
pub use trash::{Restored, Trashed};

const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

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

/// Apunta la Huella del Servidor y corta la conexión antes de autenticarse.
struct FingerprintReader {
    fingerprint: Arc<std::sync::Mutex<Option<String>>>,
}

impl client::Handler for FingerprintReader {
    type Error = russh::Error;

    async fn check_server_key(&mut self, server_key: &PublicKeyOrCertificate) -> std::result::Result<bool, Self::Error> {
        if let PublicKeyOrCertificate::PublicKey { key, .. } = server_key {
            *self.fingerprint.lock().unwrap() = Some(key.fingerprint(HashAlg::Sha256).to_string());
        }
        Ok(false)
    }
}

/// Huella SHA256 de la clave del Servidor en `host:port`, en el formato de
/// `ssh-keygen -l`. No se autentica ni envía ningún usuario ni clave.
pub(crate) async fn server_fingerprint(host: &str, port: u16) -> Result<String> {
    let fingerprint = Arc::new(std::sync::Mutex::new(None));
    let reader = FingerprintReader { fingerprint: fingerprint.clone() };
    let handshake = client::connect(Arc::new(client::Config::default()), (host, port), reader);
    // Siempre falla, porque se rechaza la clave a propósito.
    let _ = tokio::time::timeout(CONNECT_TIMEOUT, handshake).await;
    fingerprint.lock().unwrap().take().ok_or(Error::Unreachable)
}

/// Sesión SFTP abierta con el Servidor.
pub(crate) struct Connection {
    ssh: Handle<Verifier>,
    sftp: SftpSession,
    pub(crate) home: String,
    /// En Modo simple, las únicas carpetas que se pueden usar (con todo lo que
    /// contienen): la personal y la Carpeta compartida. `None` en Modo avanzado.
    allowed: Option<Vec<String>>,
    mode: Mode,
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
        let allowed = match profile.mode {
            Mode::Simple => Some([Some(home.clone()), profile.shared_folder.clone()].into_iter().flatten().collect()),
            Mode::Advanced => None,
        };
        Ok(Self { ssh, sftp, home, allowed, mode: profile.mode })
    }

    /// En Modo simple, falla si `dir` queda fuera de la carpeta personal y de
    /// la Carpeta compartida. Es una ayuda para no perderse ni tocar lo que no
    /// toca, no una barrera de seguridad: eso se configura en el Servidor.
    fn check_allowed(&self, dir: &str) -> Result<()> {
        match &self.allowed {
            Some(roots) if !roots.iter().any(|root| is_within(dir, root)) => Err(Error::OutsideAllowed),
            _ => Ok(()),
        }
    }

    /// Falla en Modo simple: la Terminal y borrar para siempre son solo del
    /// Modo avanzado.
    fn check_advanced(&self) -> Result<()> {
        match self.mode {
            Mode::Advanced => Ok(()),
            Mode::Simple => Err(Error::AdvancedOnly),
        }
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

    /// Traduce un error SFTP, distinguiendo una conexión caída o que no
    /// responde, un permiso denegado o algo que no existe de un fallo genérico
    /// de la operación.
    fn failure(&self, err: SftpError, fallback: Error) -> Error {
        if self.ssh.is_closed() {
            return Error::ConnectionLost;
        }
        match err {
            SftpError::Status(s) if s.status_code == StatusCode::PermissionDenied => Error::PermissionDenied,
            SftpError::Status(s) if s.status_code == StatusCode::NoSuchFile => Error::NotFound,
            SftpError::Timeout | SftpError::IO(_) | SftpError::UnexpectedBehavior(_) => Error::ConnectionLost,
            _ => fallback,
        }
    }

    /// `fallback`, salvo si se ha caído la conexión. Para errores que el
    /// Servidor no explica.
    fn lost_or(&self, fallback: Error) -> Error {
        if self.ssh.is_closed() { Error::ConnectionLost } else { fallback }
    }

    fn check_connection(&self) -> Result<()> {
        if self.ssh.is_closed() { Err(Error::ConnectionLost) } else { Ok(()) }
    }
}
