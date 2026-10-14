use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

const SUPPORTED_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    Simple,
    Advanced,
}

/// Datos para que un Miembro se conecte al Servidor. Nunca contiene claves privadas.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Profile {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub fingerprint: String,
    pub mode: Mode,
    pub shared_folder: Option<String>,
    /// Perfil de un Administrador: la app le ofrece «Crear perfil».
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub admin: bool,
}

/// Lo que elige el Administrador al crear un Perfil; la Huella la calcula la app.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct NewProfile {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub mode: Mode,
    pub shared_folder: Option<String>,
    pub admin: bool,
}

#[derive(Deserialize)]
struct RawProfile {
    version: u32,
    host: String,
    #[serde(default = "default_port")]
    port: u16,
    username: String,
    fingerprint: String,
    mode: Mode,
    #[serde(default)]
    shared_folder: Option<String>,
    #[serde(default)]
    admin: bool,
}

fn default_port() -> u16 {
    22
}

impl Profile {
    /// Interpreta el contenido de un fichero de Perfil.
    pub fn parse(text: &str) -> Result<Self> {
        let raw: RawProfile = serde_json::from_str(text).map_err(|_| Error::ProfileMalformed)?;
        if raw.version != SUPPORTED_VERSION {
            return Err(Error::ProfileUnsupportedVersion);
        }
        let profile = Self {
            host: raw.host.trim().to_string(),
            port: raw.port,
            username: raw.username.trim().to_string(),
            fingerprint: raw.fingerprint.trim().to_string(),
            mode: raw.mode,
            shared_folder: raw.shared_folder.map(|f| f.trim().to_string()).filter(|f| !f.is_empty()),
            admin: raw.admin,
        };
        profile.validate()?;
        Ok(profile)
    }

    /// Perfil a partir de lo que elige el Administrador, aún sin la Huella,
    /// que se pone con [`Profile::with_fingerprint`] tras conectar.
    pub(crate) fn from_new(new: NewProfile) -> Result<Self> {
        let profile = Self {
            host: new.host.trim().to_string(),
            port: new.port,
            username: new.username.trim().to_string(),
            fingerprint: String::new(),
            mode: new.mode,
            shared_folder: new.shared_folder.map(|f| f.trim().to_string()).filter(|f| !f.is_empty()),
            admin: new.admin,
        };
        profile.validate_except_fingerprint()?;
        Ok(profile)
    }

    pub(crate) fn with_fingerprint(self, fingerprint: String) -> Result<Self> {
        let profile = Self { fingerprint, ..self };
        profile.validate()?;
        Ok(profile)
    }

    /// Serializa el Perfil en el formato de fichero.
    pub fn to_json(&self) -> String {
        #[derive(Serialize)]
        struct File<'a> {
            version: u32,
            #[serde(flatten)]
            profile: &'a Profile,
        }
        serde_json::to_string_pretty(&File { version: SUPPORTED_VERSION, profile: self })
            .expect("un Perfil siempre se puede serializar")
    }

    fn validate(&self) -> Result<()> {
        self.validate_except_fingerprint()?;
        if !is_sha256_fingerprint(&self.fingerprint) {
            return Err(Error::ProfileInvalidField("fingerprint"));
        }
        Ok(())
    }

    fn validate_except_fingerprint(&self) -> Result<()> {
        if self.host.is_empty() || self.host.chars().any(|c| c.is_whitespace() || c == '/') {
            return Err(Error::ProfileInvalidField("host"));
        }
        if self.port == 0 {
            return Err(Error::ProfileInvalidField("port"));
        }
        if !is_unix_username(&self.username) {
            return Err(Error::ProfileInvalidField("username"));
        }
        if let Some(folder) = &self.shared_folder {
            if !folder.starts_with('/') {
                return Err(Error::ProfileInvalidField("shared_folder"));
            }
        }
        Ok(())
    }
}

fn is_unix_username(name: &str) -> bool {
    let mut chars = name.chars();
    matches!(chars.next(), Some(c) if c.is_ascii_lowercase() || c == '_')
        && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-')
}

/// `SHA256:` seguido de 43 caracteres base64 sin relleno, como imprime `ssh-keygen -l`.
fn is_sha256_fingerprint(value: &str) -> bool {
    value.strip_prefix("SHA256:").is_some_and(|b64| {
        b64.len() == 43 && b64.chars().all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '/')
    })
}
