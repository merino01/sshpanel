//! Administración: las Claves públicas autorizadas de cada usuario. Están en
//! `/etc/ssh/authorized_keys/<usuario>`, de root y legibles por todos según la
//! guía del servidor (ADR 0006): se leen por SFTP, sin sudo, y se escriben con
//! `sudo` y la contraseña del Administrador por un canal SSH (ADR 0007).

use std::time::Duration;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use russh::ChannelMsg;
use serde::Serialize;
use tokio::io::AsyncReadExt;

use super::Connection;
use super::paths::join;
use crate::error::{Error, Result};
use crate::profile::{Mode, is_unix_username};

/// Lo más que se espera a una orden por el canal SSH.
const EXEC_TIMEOUT: Duration = Duration::from_secs(20);
/// Lo que imprime el script al terminar bien.
const DONE: &str = "sshpanel-ok";

/// La carpeta de `AuthorizedKeysFile /etc/ssh/authorized_keys/%u`.
const AUTHORIZED_KEYS_DIR: &str = "/etc/ssh/authorized_keys";

/// Un cambio en el fichero de claves de un usuario. Las líneas se comparan sin
/// los espacios de alrededor; el resto del fichero (comentarios, líneas vacías)
/// se conserva tal cual.
pub(crate) enum KeyChange<'a> {
    /// Añade la línea, si no está ya.
    Authorize(&'a str),
    /// Quita la línea.
    Revoke(&'a str),
    /// Cambia la primera línea por la segunda.
    Replace(&'a str, &'a str),
}

/// Lo que ha devuelto una orden.
struct Output {
    status: Option<u32>,
    stdout: String,
    stderr: String,
}

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

    /// Los usuarios de personas del Servidor (uid desde 1000), por nombre, para
    /// elegir a quién autorizar una clave.
    pub(crate) async fn server_users(&self) -> Result<Vec<String>> {
        let mut text = String::new();
        let mut file = self.sftp.open("/etc/passwd").await.map_err(|e| self.failure(e, Error::ReadFailed))?;
        file.read_to_string(&mut text).await.map_err(|e| self.io_failure(e, Error::ReadFailed))?;
        let mut users: Vec<String> = text
            .lines()
            .filter_map(|line| {
                let mut fields = line.split(':');
                let name = fields.next()?;
                let uid: u32 = fields.nth(1)?.parse().ok()?;
                (uid >= 1000 && uid != 65534 && name != "nobody").then(|| name.to_string())
            })
            .collect();
        users.sort();
        users.dedup();
        Ok(users)
    }

    /// Comprueba que se pueden cambiar las claves con `password`: que la clave
    /// del Administrador deja ejecutar órdenes (Modo avanzado) y que tiene sudo.
    pub(crate) async fn check_sudo(&self, password: &str) -> Result<()> {
        if self.mode == Mode::Simple {
            return Err(Error::AdminNeedsAdvanced);
        }
        // Con `command="internal-sftp"`, el Servidor abre SFTP en lugar de la orden y no responde nada.
        let probe = self.exec("echo sshpanel", b"").await?;
        if probe.stdout.trim() != "sshpanel" {
            return Err(Error::AdminNeedsAdvanced);
        }
        let output = self.sudo(password, "true").await?;
        if output.status == Some(0) { Ok(()) } else { Err(sudo_failure(&output)) }
    }

    /// Aplica `change` al fichero de claves de `user` con sudo, sustituyéndolo
    /// entero, de root y con permisos `644`, para que nunca quede a medias.
    pub(crate) async fn change_keys(&self, user: &str, password: &str, change: KeyChange<'_>) -> Result<()> {
        if !is_unix_username(user) {
            return Err(Error::InvalidUser);
        }
        if self.mode == Mode::Simple {
            return Err(Error::AdminNeedsAdvanced);
        }
        let path = join(AUTHORIZED_KEYS_DIR, user);
        let current = match self.sftp.open(&path).await {
            Ok(mut file) => {
                let mut text = String::new();
                file.read_to_string(&mut text).await.map_err(|e| self.io_failure(e, Error::KeysUnavailable))?;
                text
            }
            Err(e) => match self.failure(e, Error::KeysUnavailable) {
                Error::NotFound => String::new(),
                other => return Err(other),
            },
        };
        let Some(text) = apply(&current, &change)? else { return Ok(()) };
        let script = format!(
            "d={AUTHORIZED_KEYS_DIR}; t=$(mktemp \"$d/.{user}.XXXXXX\") || exit 1; \
             {{ printf %s {data} | base64 -d > \"$t\" && chown root:root \"$t\" && chmod 644 \"$t\" && mv -f \"$t\" \"$d/{user}\"; }} \
             || {{ rm -f \"$t\"; exit 1; }}; echo {DONE}",
            data = STANDARD.encode(text),
        );
        let output = self.sudo(password, &format!("sh -c '{script}'")).await?;
        if output.status == Some(0) && output.stdout.trim_end().ends_with(DONE) {
            return Ok(());
        }
        match sudo_failure(&output) {
            Error::KeyWriteFailed if output.status.is_none() => Err(Error::AdminNeedsAdvanced),
            error => Err(error),
        }
    }

    /// Ejecuta `command` como root, dando `password` a sudo por la entrada. `-k`
    /// hace que la pida siempre, y el resto de la entrada queda vacía.
    async fn sudo(&self, password: &str, command: &str) -> Result<Output> {
        let mut input = zeroize::Zeroizing::new(password.as_bytes().to_vec());
        input.push(b'\n');
        self.exec(&format!("LC_ALL=C sudo -S -k -p '' {command}"), &input).await
    }

    /// Ejecuta `command` por un canal nuevo, le pasa `input` y espera a que acabe.
    async fn exec(&self, command: &str, input: &[u8]) -> Result<Output> {
        let run = async {
            let mut channel = self.ssh.channel_open_session().await.map_err(|_| self.lost_or(Error::KeyWriteFailed))?;
            channel.exec(true, command).await.map_err(|_| self.lost_or(Error::KeyWriteFailed))?;
            channel.data(input).await.map_err(|_| self.lost_or(Error::KeyWriteFailed))?;
            channel.eof().await.map_err(|_| self.lost_or(Error::KeyWriteFailed))?;
            let (mut status, mut stdout, mut stderr) = (None, Vec::new(), Vec::new());
            while let Some(msg) = channel.wait().await {
                match msg {
                    ChannelMsg::Data { data } => stdout.extend_from_slice(&data),
                    ChannelMsg::ExtendedData { data, .. } => stderr.extend_from_slice(&data),
                    ChannelMsg::ExitStatus { exit_status } => status = Some(exit_status),
                    ChannelMsg::Failure => return Err(Error::AdminNeedsAdvanced),
                    ChannelMsg::Close => break,
                    _ => {}
                }
            }
            Ok(Output { status, stdout: String::from_utf8_lossy(&stdout).into(), stderr: String::from_utf8_lossy(&stderr).into() })
        };
        tokio::time::timeout(EXEC_TIMEOUT, run).await.map_err(|_| self.lost_or(Error::KeyWriteFailed))?
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

/// El fichero con `change` aplicado, o `None` si ya estaba así.
fn apply(current: &str, change: &KeyChange) -> Result<Option<String>> {
    let mut lines: Vec<&str> = current.lines().collect();
    let position = |lines: &[&str], line: &str| lines.iter().position(|l| l.trim() == line.trim());
    match *change {
        KeyChange::Authorize(line) => {
            if position(&lines, line).is_some() {
                return Ok(None);
            }
            lines.push(line.trim());
        }
        KeyChange::Revoke(line) => {
            let before = lines.len();
            lines.retain(|l| l.trim() != line.trim());
            if lines.len() == before {
                return Err(Error::KeyChanged);
            }
        }
        KeyChange::Replace(old, new) => {
            let i = position(&lines, old).ok_or(Error::KeyChanged)?;
            lines[i] = new.trim();
        }
    }
    let mut text = lines.join("\n");
    if !text.is_empty() {
        text.push('\n');
    }
    Ok(Some(text))
}

/// Por qué ha fallado sudo, según lo que dice (en inglés, por `LC_ALL=C`).
fn sudo_failure(output: &Output) -> Error {
    let stderr = output.stderr.to_lowercase();
    if stderr.contains("incorrect password") || stderr.contains("sorry, try again") {
        Error::SudoWrongPassword
    } else if stderr.contains("not in the sudoers") || stderr.contains("not allowed") || stderr.contains("may not run sudo") || output.status == Some(127) {
        Error::NoSudo
    } else {
        Error::KeyWriteFailed
    }
}
