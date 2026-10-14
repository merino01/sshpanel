//! Núcleo de SSH Panel. La interfaz (Tauri) solo invoca esta API.

mod connection;
mod error;
mod identity;
mod profile;
mod secrets;

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use tokio::sync::Mutex;

use connection::Connection;

pub use connection::{Entry, EntryKind, Listing, Progress};
pub use error::{Error, Result};
pub use profile::{Mode, Profile};
pub use secrets::{MemorySecretStore, SecretStore, SystemSecretStore};

const PROFILE_FILE: &str = "profile.json";

/// Punto de entrada del núcleo: Perfil importado, identidad del Miembro y
/// conexión con el Servidor.
pub struct Core {
    data_dir: PathBuf,
    secrets: Box<dyn SecretStore>,
    /// Compartida, para que una Transferencia larga no bloquee el Explorador.
    connection: Mutex<Option<Arc<Connection>>>,
}

impl Core {
    pub fn new(data_dir: impl Into<PathBuf>, secrets: Box<dyn SecretStore>) -> Self {
        Self { data_dir: data_dir.into(), secrets, connection: Mutex::new(None) }
    }

    /// Importa un fichero de Perfil. Genera la identidad del Miembro si todavía
    /// no existe. Si algo falla no se guarda nada.
    pub fn import_profile(&self, file: &Path) -> Result<Profile> {
        let text = fs::read_to_string(file).map_err(|_| Error::ProfileUnreadable)?;
        let profile = Profile::parse(&text)?;
        identity::ensure(self.secrets.as_ref())?;
        fs::create_dir_all(&self.data_dir).map_err(|_| Error::Storage)?;
        fs::write(self.data_dir.join(PROFILE_FILE), profile.to_json()).map_err(|_| Error::Storage)?;
        Ok(profile)
    }

    /// Perfil importado anteriormente, si lo hay.
    pub fn current_profile(&self) -> Result<Option<Profile>> {
        match fs::read_to_string(self.data_dir.join(PROFILE_FILE)) {
            Ok(text) => Profile::parse(&text).map(Some),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(_) => Err(Error::Storage),
        }
    }

    /// Clave pública del Miembro, en formato `authorized_keys`. Es la única
    /// parte de la identidad que sale de este equipo.
    pub fn public_key(&self) -> Result<String> {
        identity::ensure(self.secrets.as_ref())
    }

    /// Conecta con el Servidor del Perfil y lista la carpeta personal. Solo
    /// conecta si la Huella del Servidor coincide con la del Perfil; nunca
    /// pregunta al Miembro. Sustituye cualquier conexión anterior.
    pub async fn connect(&self) -> Result<Listing> {
        let profile = self.current_profile()?.ok_or(Error::NoProfile)?;
        let private_key = identity::private_key(self.secrets.as_ref())?;
        let mut slot = self.connection.lock().await;
        *slot = None;
        let connection = Connection::open(&profile, &private_key).await?;
        let listing = connection.list(&connection.home).await?;
        *slot = Some(Arc::new(connection));
        Ok(listing)
    }

    /// Lista una carpeta del Servidor (ruta absoluta).
    pub async fn list(&self, path: &str) -> Result<Listing> {
        self.connection().await?.list(path).await
    }

    /// Crea la carpeta `name` dentro de `parent`. Falla si ya existe algo con
    /// ese nombre.
    pub async fn create_folder(&self, parent: &str, name: &str) -> Result<()> {
        self.connection().await?.create_folder(parent, name).await
    }

    /// Renombra el archivo o carpeta `from` de la carpeta `dir` a `to`, sin
    /// moverlo de carpeta. Nunca sustituye algo que ya exista.
    pub async fn rename(&self, dir: &str, from: &str, to: &str) -> Result<()> {
        self.connection().await?.rename(dir, from, to).await
    }

    /// Sube el archivo o la carpeta `local` a la carpeta `remote_dir` del
    /// Servidor, avisando a `progress` de cada avance. Si el nombre está
    /// ocupado usa `nombre (1)`, `nombre (2)`…; devuelve el nombre elegido.
    pub async fn upload(&self, local: &Path, remote_dir: &str, mut progress: impl FnMut(Progress) + Send) -> Result<String> {
        self.connection().await?.upload(local, remote_dir, &mut progress).await
    }

    /// Descarga el archivo o la carpeta `name` de la carpeta `remote_dir` a la
    /// carpeta `local_dir` del equipo, avisando a `progress` de cada avance. Si
    /// el nombre está ocupado usa `nombre (1)`, `nombre (2)`…; devuelve dónde
    /// ha quedado.
    pub async fn download(&self, remote_dir: &str, name: &str, local_dir: &Path, mut progress: impl FnMut(Progress) + Send) -> Result<PathBuf> {
        self.connection().await?.download(remote_dir, name, local_dir, &mut progress).await
    }

    async fn connection(&self) -> Result<Arc<Connection>> {
        self.connection.lock().await.clone().ok_or(Error::NotConnected)
    }
}
