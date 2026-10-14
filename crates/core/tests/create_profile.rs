//! Crear Perfil (Administrador): la Huella se calcula conectándose al Servidor
//! y el Perfil creado se puede importar, a través de la interfaz del núcleo y
//! contra el servidor OpenSSH de prueba.

mod support;

use std::fs;

use sshpanel_core::{Core, Error, MemorySecretStore, Mode, NewProfile};
use support::TestServer;
use tempfile::TempDir;

fn core(dir: &TempDir, name: &str) -> Core {
    Core::new(dir.path().join(name), Box::new(MemorySecretStore::default()))
}

fn new_profile(server: &TestServer, username: &str) -> NewProfile {
    NewProfile {
        host: server.host().to_string(),
        port: server.port(),
        username: username.to_string(),
        mode: Mode::Simple,
        shared_folder: Some("/srv/compartida".into()),
        admin: false,
    }
}

#[tokio::test]
async fn computes_the_fingerprint_and_writes_an_importable_profile() {
    let server = TestServer::start();
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("alice.dbdesp");

    let profile = core(&dir, "admin").create_profile(new_profile(&server, "alice"), &file).await.unwrap();

    assert_eq!(profile.fingerprint, server.fingerprint());
    let member = core(&dir, "alice");
    assert_eq!(member.import_profile(&file).unwrap(), profile);
    server.authorize("alice", &member.public_key().unwrap());
    assert_eq!(member.connect().await.unwrap().path, "/home/alice");
}

#[tokio::test]
async fn the_profile_contains_no_keys_and_marks_administrators() {
    let server = TestServer::start();
    let dir = TempDir::new().unwrap();
    let admin = core(&dir, "admin");
    let (member_file, admin_file) = (dir.path().join("alice.dbdesp"), dir.path().join("bob.dbdesp"));

    admin.create_profile(new_profile(&server, "alice"), &member_file).await.unwrap();
    let new = NewProfile { mode: Mode::Advanced, shared_folder: None, admin: true, ..new_profile(&server, "bob") };
    admin.create_profile(new, &admin_file).await.unwrap();

    let member_text = fs::read_to_string(&member_file).unwrap();
    assert!(!member_text.contains("KEY") && !member_text.contains("admin"), "{member_text}");
    assert!(member_text.contains(r#""mode": "simple""#) && member_text.contains("/srv/compartida"), "{member_text}");
    let imported = core(&dir, "bob").import_profile(&admin_file).unwrap();
    assert!(imported.admin);
    assert_eq!((imported.mode, imported.shared_folder), (Mode::Advanced, None));
}

#[tokio::test]
async fn does_not_authenticate_to_compute_the_fingerprint() {
    let server = TestServer::start();
    let dir = TempDir::new().unwrap();

    core(&dir, "admin").create_profile(new_profile(&server, "alice"), &dir.path().join("alice.dbdesp")).await.unwrap();

    let logs = server.logs();
    assert!(!logs.contains("alice"), "no se envía el usuario:\n{logs}");
}

#[tokio::test]
async fn an_unreachable_server_writes_nothing() {
    let dir = TempDir::new().unwrap();
    let port = std::net::TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port();
    let file = dir.path().join("alice.dbdesp");
    let new = NewProfile { host: "127.0.0.1".into(), port, username: "alice".into(), mode: Mode::Simple, shared_folder: None, admin: false };

    assert_eq!(core(&dir, "admin").create_profile(new, &file).await.unwrap_err(), Error::Unreachable);
    assert!(!file.exists());
}

#[tokio::test]
async fn invalid_choices_are_refused_before_connecting() {
    let dir = TempDir::new().unwrap();
    let admin = core(&dir, "admin");
    let file = dir.path().join("x.dbdesp");
    // Nadie escucha en esta dirección: si llegara a conectar, el error sería otro.
    let base = NewProfile { host: "192.0.2.1".into(), port: 22, username: "alice".into(), mode: Mode::Simple, shared_folder: None, admin: false };

    let cases = [
        (NewProfile { host: "  ".into(), ..base.clone() }, "host"),
        (NewProfile { host: "ejemplo.com/x".into(), ..base.clone() }, "host"),
        (NewProfile { port: 0, ..base.clone() }, "port"),
        (NewProfile { username: "Alice".into(), ..base.clone() }, "username"),
        (NewProfile { shared_folder: Some("compartida".into()), ..base.clone() }, "shared_folder"),
    ];
    for (new, field) in cases {
        assert_eq!(admin.create_profile(new, &file).await.unwrap_err(), Error::ProfileInvalidField(field));
    }
    assert!(!file.exists());
}
