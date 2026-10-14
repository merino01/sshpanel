//! Conexión verificando la Huella y listado de la carpeta personal, a través de
//! la interfaz del núcleo y contra el servidor OpenSSH de prueba.

mod support;

use std::fs;

use sshpanel_core::{Core, EntryKind, Error, MemorySecretStore};
use support::{TestKey, TestServer};
use tempfile::TempDir;

/// Huella con formato válido que no es la de ningún servidor de prueba.
const OTHER_FINGERPRINT: &str = "SHA256:EjEmPlOdEhUeLlAnOeSrEaLxXxXxXxXxXxXxXxXxXxX";

struct Fixture {
    dir: TempDir,
    core: Core,
}

impl Fixture {
    fn new() -> Self {
        let dir = TempDir::new().unwrap();
        let core = Core::new(dir.path().join("data"), Box::new(MemorySecretStore::default()));
        Self { dir, core }
    }

    /// Importa un Perfil de `alice` para el servidor dado.
    fn import(&self, host: &str, port: u16, fingerprint: &str) {
        let path = self.dir.path().join("perfil.sshpanel");
        fs::write(
            &path,
            format!(
                r#"{{"version":1,"host":"{host}","port":{port},"username":"alice","fingerprint":"{fingerprint}","mode":"simple"}}"#
            ),
        )
        .unwrap();
        self.core.import_profile(&path).unwrap();
    }

    fn import_for(&self, server: &TestServer) {
        self.import(server.host(), server.port(), &server.fingerprint());
    }
}

#[tokio::test]
async fn connects_and_lists_the_home_folder() {
    let server = TestServer::start();
    let fx = Fixture::new();
    fx.import_for(&server);
    server.authorize("alice", &fx.core.public_key().unwrap());
    server.put_file("alice", "notas.txt", "hola");
    server.mkdir("alice", "fotos");

    let listing = fx.core.connect().await.unwrap();

    assert_eq!(listing.path, "/home/alice");
    let names: Vec<_> = listing.entries.iter().map(|e| (e.name.as_str(), e.kind)).collect();
    let fotos = names.iter().position(|e| *e == ("fotos", EntryKind::Dir)).expect("lista la carpeta");
    let notas = names.iter().position(|e| *e == ("notas.txt", EntryKind::File)).expect("lista el archivo");
    assert!(fotos < notas, "las carpetas van primero: {names:?}");
    assert_eq!(listing.entries[notas].size, 4);
    assert!(!names.iter().any(|(n, _)| *n == "." || *n == ".."));
}

#[tokio::test]
async fn lists_a_folder_once_connected() {
    let server = TestServer::start();
    let fx = Fixture::new();
    fx.import_for(&server);
    server.authorize("alice", &fx.core.public_key().unwrap());
    server.mkdir("alice", "fotos");
    server.put_file("alice", "fotos/playa.jpg", "jpg");
    fx.core.connect().await.unwrap();

    let listing = fx.core.list("/home/alice/fotos").await.unwrap();

    assert_eq!(listing.entries.len(), 1);
    assert_eq!(listing.entries[0].name, "playa.jpg");
}

#[tokio::test]
async fn refuses_a_server_whose_fingerprint_does_not_match_without_sending_anything() {
    let server = TestServer::start();
    let fx = Fixture::new();
    fx.import(server.host(), server.port(), OTHER_FINGERPRINT);
    server.authorize("alice", &fx.core.public_key().unwrap());

    let err = fx.core.connect().await.unwrap_err();

    assert_eq!(err, Error::FingerprintMismatch);
    // Un intento posterior de `bob` marca hasta dónde ha llegado el registro.
    server.sftp("bob", &TestKey::generate(), "pwd\n");
    assert!(server.wait_for_log("bob"), "{}", server.logs());
    assert!(!server.logs().contains("alice"), "no debe enviar el usuario:\n{}", server.logs());
    assert_eq!(fx.core.list("/home/alice").await.unwrap_err(), Error::NotConnected);
}

#[tokio::test]
async fn reports_a_key_that_is_not_authorized_yet() {
    let server = TestServer::start();
    let fx = Fixture::new();
    fx.import_for(&server);

    assert_eq!(fx.core.connect().await.unwrap_err(), Error::Unauthorized);
    // Contraste con la prueba de la Huella: un intento de autenticación sí deja rastro.
    assert!(server.wait_for_log("alice"), "{}", server.logs());
}

#[tokio::test]
async fn connecting_requires_a_profile() {
    assert_eq!(Fixture::new().core.connect().await.unwrap_err(), Error::NoProfile);
}

#[tokio::test]
async fn reports_an_unreachable_server() {
    let (host, port, fingerprint) = {
        let server = TestServer::start();
        (server.host().to_string(), server.port(), server.fingerprint())
    };
    let fx = Fixture::new();
    fx.import(&host, port, &fingerprint);

    assert_eq!(fx.core.connect().await.unwrap_err(), Error::Unreachable);
}

#[tokio::test]
async fn listing_requires_a_connection() {
    assert_eq!(Fixture::new().core.list("/home/alice").await.unwrap_err(), Error::NotConnected);
}
