//! Núcleo de SSH Panel. La interfaz (Tauri) solo invoca esta API.

mod connection;
mod error;
mod identity;
mod profile;
mod secrets;

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use tokio::sync::Mutex;

use connection::{Connection, Reserved, Terminal};

pub use connection::{Entry, EntryKind, Listing, Outcome, Progress, Restored, TerminalEvent, Transfer, Trashed};
pub use error::{Error, Result};
pub use profile::{Mode, NewProfile, Profile};
pub use secrets::{MemorySecretStore, SecretStore, SystemSecretStore};

const PROFILE_FILE: &str = "profile.json";

/// Punto de entrada del núcleo: Perfil importado, identidad del Miembro y
/// conexión con el Servidor.
pub struct Core {
    data_dir: PathBuf,
    secrets: Box<dyn SecretStore>,
    /// Compartida, para que una Transferencia larga no bloquee el Explorador.
    connection: Mutex<Option<Arc<Connection>>>,
    /// Destinos elegidos por las operaciones en curso.
    reserved: Reserved,
    /// Terminales abiertas, por el número que les da la interfaz.
    terminals: Mutex<HashMap<u32, Arc<Terminal>>>,
}

impl Core {
    pub fn new(data_dir: impl Into<PathBuf>, secrets: Box<dyn SecretStore>) -> Self {
        Self { data_dir: data_dir.into(), secrets, connection: Mutex::new(None), reserved: Reserved::default(), terminals: Mutex::default() }
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

    /// Crea el fichero de Perfil `file` para un Miembro. La Huella la calcula
    /// conectándose al Servidor, sin autenticarse; si no se puede contactar con
    /// él no se escribe nada. El Perfil nunca contiene claves.
    pub async fn create_profile(&self, new: NewProfile, file: &Path) -> Result<Profile> {
        let profile = Profile::from_new(new)?;
        let fingerprint = connection::server_fingerprint(&profile.host, profile.port).await?;
        let profile = profile.with_fingerprint(fingerprint)?;
        fs::write(file, profile.to_json()).map_err(|_| Error::Storage)?;
        Ok(profile)
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
        self.close_all_terminals().await;
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

    /// Mueve el archivo o carpeta `name` de la carpeta `dir` a la Papelera del
    /// Miembro, recordando de dónde vino. Nunca elimina nada. Devuelve su id en
    /// la Papelera, para deshacerlo con [`Core::restore`].
    pub async fn trash(&self, dir: &str, name: &str) -> Result<String> {
        self.connection().await?.trash(dir, name).await
    }

    /// Contenido de la Papelera, lo último borrado primero.
    pub async fn list_trash(&self) -> Result<Vec<Trashed>> {
        self.connection().await?.list_trash().await
    }

    /// Devuelve a su sitio el elemento `id` de la Papelera, volviendo a crear
    /// su carpeta si ya no existe. Nunca sustituye nada: si el nombre está
    /// ocupado usa `nombre (1)`, `nombre (2)`…
    pub async fn restore(&self, id: &str) -> Result<Restored> {
        self.connection().await?.restore(id, &self.reserved).await
    }

    /// Borra para siempre el elemento `id` de la Papelera. Solo en Modo avanzado.
    pub async fn delete_forever(&self, id: &str) -> Result<()> {
        self.connection().await?.delete_forever(id).await
    }

    /// Borra para siempre todo lo que hay en la Papelera. Solo en Modo avanzado.
    pub async fn empty_trash(&self) -> Result<()> {
        self.connection().await?.empty_trash().await
    }

    /// Abre la Terminal `id` en el Servidor, de `cols` columnas y `rows`
    /// filas, cerrando la que tuviera ese número. Lo que responde el Servidor,
    /// y el cierre de la sesión, llegan por `on_event`. Puede haber varias a la
    /// vez sobre la misma conexión. Solo en Modo avanzado.
    pub async fn open_terminal(&self, id: u32, cols: u32, rows: u32, on_event: impl FnMut(TerminalEvent) + Send + 'static) -> Result<()> {
        let connection = self.connection().await?;
        self.close_terminal(id).await;
        let terminal = connection.open_terminal(cols, rows, on_event).await?;
        if let Some(old) = self.terminals.lock().await.insert(id, Arc::new(terminal)) {
            old.close().await;
        }
        Ok(())
    }

    /// Envía a la Terminal `id` lo que teclea el Miembro.
    pub async fn terminal_input(&self, id: u32, data: &[u8]) -> Result<()> {
        self.terminal(id).await?.input(data).await
    }

    /// Cambia el tamaño de la Terminal `id`, en columnas y filas.
    pub async fn resize_terminal(&self, id: u32, cols: u32, rows: u32) -> Result<()> {
        self.terminal(id).await?.resize(cols, rows).await
    }

    /// Cierra la Terminal `id`, si está abierta.
    pub async fn close_terminal(&self, id: u32) {
        let terminal = self.terminals.lock().await.remove(&id);
        if let Some(terminal) = terminal {
            terminal.close().await;
        }
    }

    async fn close_all_terminals(&self) {
        let terminals: Vec<_> = self.terminals.lock().await.drain().map(|(_, t)| t).collect();
        for terminal in terminals {
            terminal.close().await;
        }
    }

    async fn terminal(&self, id: u32) -> Result<Arc<Terminal>> {
        self.terminals.lock().await.get(&id).cloned().ok_or(Error::NotConnected)
    }

    /// Lanza una Transferencia, avisando a `progress` de cada avance, o la
    /// reanuda donde se quedó si un intento anterior falló. Nunca sustituye
    /// nada: si el nombre está ocupado usa `nombre (1)`, `nombre (2)`…
    /// Varias Transferencias pueden ir a la vez sobre la misma conexión.
    pub async fn run(&self, transfer: &Transfer, mut progress: impl FnMut(Progress) + Send) -> Result<Outcome> {
        connection::run(self.connection().await.ok(), transfer, &self.reserved, &mut progress).await
    }

    /// Cancela una Transferencia, en curso o fallida, y borra lo que había
    /// creado; si está en curso, espera a que se pare. Lo que ya existía antes
    /// nunca se toca.
    pub async fn discard(&self, transfer: &Transfer) {
        connection::discard(self.connection().await.ok(), transfer).await
    }

    async fn connection(&self) -> Result<Arc<Connection>> {
        self.connection.lock().await.clone().ok_or(Error::NotConnected)
    }
}
