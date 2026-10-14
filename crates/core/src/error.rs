/// Errores del núcleo. La interfaz traduce cada [`Error::code`] a un mensaje
/// en lenguaje simple; aquí solo viven descripciones técnicas.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum Error {
    #[error("no se pudo leer el fichero del Perfil")]
    ProfileUnreadable,
    #[error("el Perfil no es un fichero válido")]
    ProfileMalformed,
    #[error("versión de Perfil no soportada")]
    ProfileUnsupportedVersion,
    #[error("el Perfil tiene un valor inválido en `{0}`")]
    ProfileInvalidField(&'static str),
    #[error("el almacén seguro del sistema no está disponible")]
    SecretStoreUnavailable,
    #[error("no se pudo guardar el estado de la aplicación")]
    Storage,
    #[error("no hay ningún Perfil importado")]
    NoProfile,
    #[error("la Huella del Servidor no coincide con la del Perfil")]
    FingerprintMismatch,
    #[error("el Servidor no acepta la clave del Miembro")]
    Unauthorized,
    #[error("no se pudo contactar con el Servidor")]
    Unreachable,
    #[error("no hay conexión abierta con el Servidor")]
    NotConnected,
    #[error("se perdió la conexión con el Servidor")]
    ConnectionLost,
    #[error("no se pudo listar la carpeta")]
    ListFailed,
    #[error("el Servidor no da permiso para esta operación")]
    PermissionDenied,
    #[error("el archivo o carpeta no existe")]
    NotFound,
    #[error("ya existe un archivo o carpeta con ese nombre")]
    AlreadyExists,
    #[error("el nombre está vacío o contiene `/`")]
    InvalidName,
    #[error("no se pudo crear la carpeta")]
    CreateFolderFailed,
    #[error("no se pudo renombrar")]
    RenameFailed,
    #[error("no se pudo leer el archivo o carpeta del equipo")]
    LocalUnreadable,
    #[error("no se pudo escribir en la carpeta del equipo")]
    LocalWriteFailed,
    #[error("la Transferencia falló")]
    TransferFailed,
    #[error("la Transferencia se ha cancelado")]
    Cancelled,
}

impl Error {
    /// Código estable que la interfaz usa para elegir el mensaje.
    pub fn code(&self) -> &'static str {
        match self {
            Self::ProfileUnreadable => "profile.unreadable",
            Self::ProfileMalformed => "profile.malformed",
            Self::ProfileUnsupportedVersion => "profile.unsupported_version",
            Self::ProfileInvalidField(_) => "profile.invalid_field",
            Self::SecretStoreUnavailable => "secrets.unavailable",
            Self::Storage => "storage.failed",
            Self::NoProfile => "connection.no_profile",
            Self::FingerprintMismatch => "connection.fingerprint_mismatch",
            Self::Unauthorized => "connection.unauthorized",
            Self::Unreachable => "connection.unreachable",
            Self::NotConnected => "connection.not_connected",
            Self::ConnectionLost => "connection.lost",
            Self::ListFailed => "files.list_failed",
            Self::PermissionDenied => "files.permission_denied",
            Self::NotFound => "files.not_found",
            Self::AlreadyExists => "files.already_exists",
            Self::InvalidName => "files.invalid_name",
            Self::CreateFolderFailed => "files.create_folder_failed",
            Self::RenameFailed => "files.rename_failed",
            Self::LocalUnreadable => "transfer.local_unreadable",
            Self::LocalWriteFailed => "transfer.local_write_failed",
            Self::TransferFailed => "transfer.failed",
            Self::Cancelled => "transfer.cancelled",
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;
