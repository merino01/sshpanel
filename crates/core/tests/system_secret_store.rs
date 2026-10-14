//! Necesita un almacén seguro real del sistema (llavero desbloqueado), así que
//! no corre en la CI. Ejecutar a mano con `cargo test -p sshpanel-core -- --ignored`.

use sshpanel_core::{SecretStore, SystemSecretStore};

#[test]
#[ignore = "requiere el almacén seguro del sistema"]
fn round_trips_a_secret_through_the_system_store() {
    let store = SystemSecretStore::new(format!("sshpanel-test-{}", std::process::id()));

    assert_eq!(store.get("identity").unwrap(), None);
    store.set("identity", "valor").unwrap();
    assert_eq!(store.get("identity").unwrap().as_deref(), Some("valor"));
}
