//! Administración: leer las Claves públicas autorizadas de cada usuario. Están
//! en `/etc/ssh/authorized_keys/<usuario>`, de root y legibles por todos según
//! la guía del servidor (ADR 0006), así que basta con SFTP, sin sudo.

use serde::Serialize;
use tokio::io::AsyncReadExt;

use super::Connection;
use super::paths::join;
use crate::error::{Error, Result};

/// La carpeta de `AuthorizedKeysFile /etc/ssh/authorized_keys/%u`.
const AUTHORIZED_KEYS_DIR: &str = "/etc/ssh/authorized_keys";

/// Las líneas de clave del fichero de un usuario, sin las vacías ni los comentarios.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct UserKeys {
    pub user: String,
    /// `None` si el fichero no se ha podido leer.
    pub lines: Option<Vec<String>>,
}

impl Connection {
    /// Las claves de cada usuario, por nombre. No pasa por las carpetas del
    /// Modo simple: solo lee, y lo que se puede leer lo decide el Servidor.
    pub(crate) async fn authorized_keys(&self) -> Result<Vec<UserKeys>> {
        let dir = self.sftp.read_dir(AUTHORIZED_KEYS_DIR).await.map_err(|e| match self.failure(e, Error::KeysUnavailable) {
            Error::NotFound | Error::PermissionDenied => Error::KeysUnavailable,
            other => other,
        })?;
        let mut users = Vec::new();
        for entry in dir {
            let user = entry.file_name();
            if !entry.file_type().is_file() || user.starts_with('.') {
                continue;
            }
            let lines = self.read_key_lines(&join(AUTHORIZED_KEYS_DIR, &user)).await?;
            users.push(UserKeys { user, lines });
        }
        users.sort_by(|a, b| a.user.cmp(&b.user));
        Ok(users)
    }

    /// Las líneas de clave de `path`, o `None` si no se puede leer. Solo falla si se cae la conexión.
    async fn read_key_lines(&self, path: &str) -> Result<Option<Vec<String>>> {
        let mut text = String::new();
        let read = async {
            let mut file = self.sftp.open(path).await.ok()?;
            file.read_to_string(&mut text).await.ok()
        }
        .await;
        self.check_connection()?;
        Ok(read.map(|_| {
            text.lines().map(str::trim).filter(|line| !line.is_empty() && !line.starts_with('#')).map(String::from).collect()
        }))
    }
}
