use rand_core::OsRng;
use ssh_key::{Algorithm, LineEnding, PrivateKey};

use crate::error::{Error, Result};
use crate::secrets::SecretStore;

const SECRET_NAME: &str = "identity";
const COMMENT: &str = "sshpanel";

/// Devuelve la clave pública de la identidad, generándola y guardando la
/// privada en el almacén seguro la primera vez.
pub(crate) fn ensure(store: &dyn SecretStore) -> Result<String> {
    let key = match store.get(SECRET_NAME)? {
        Some(pem) => PrivateKey::from_openssh(&pem).map_err(|_| Error::Storage)?,
        None => {
            let mut key = PrivateKey::random(&mut OsRng, Algorithm::Ed25519).map_err(|_| Error::Storage)?;
            key.set_comment(COMMENT);
            let pem = key.to_openssh(LineEnding::LF).map_err(|_| Error::Storage)?;
            store.set(SECRET_NAME, pem.as_str())?;
            key
        }
    };
    key.public_key().to_openssh().map_err(|_| Error::Storage)
}

/// Clave privada en formato OpenSSH, generándola si aún no existe. Solo la usa
/// la conexión para autenticarse; nunca se entrega fuera del núcleo.
pub(crate) fn private_key(store: &dyn SecretStore) -> Result<String> {
    ensure(store)?;
    store.get(SECRET_NAME)?.ok_or(Error::Storage)
}
