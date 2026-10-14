//! Modo simple: el Explorador, la Papelera y las Transferencias solo usan la
//! carpeta personal y la Carpeta compartida, a través de la interfaz del núcleo
//! y contra el servidor OpenSSH de prueba.

mod support;

use std::fs;

use sshpanel_core::{Core, Error, MemorySecretStore, Transfer};
use support::TestServer;
use tempfile::TempDir;

const HOME: &str = "/home/alice";
const SHARED: &str = "/srv/compartida";

struct Session {
    dir: TempDir,
    core: Core,
}

/// `alice` conectada con un Perfil en `mode`, con la Carpeta compartida `/srv/compartida`.
async fn connect(server: &TestServer, mode: &str) -> Session {
    server.sh("mkdir -p /srv/compartida /srv/otra && chmod 777 /srv/compartida /srv/otra");
    let dir = TempDir::new().unwrap();
    let core = Core::new(dir.path().join(".data"), Box::new(MemorySecretStore::default()));
    let path = dir.path().join("perfil.dbdesp");
    fs::write(
        &path,
        format!(
            r#"{{"version":1,"host":"{}","port":{},"username":"alice","fingerprint":"{}","mode":"{mode}","shared_folder":"{SHARED}"}}"#,
            server.host(),
            server.port(),
            server.fingerprint()
        ),
    )
    .unwrap();
    core.import_profile(&path).unwrap();
    server.authorize("alice", &core.public_key().unwrap());
    core.connect().await.unwrap();
    Session { dir, core }
}

#[tokio::test]
async fn uses_the_personal_and_shared_folders() {
    let server = TestServer::start();
    server.mkdir("alice", "fotos");
    let s = connect(&server, "simple").await;

    s.core.list(HOME).await.unwrap();
    s.core.list("/home/alice/fotos").await.unwrap();
    s.core.list(SHARED).await.unwrap();
    s.core.create_folder(SHARED, "actas").await.unwrap();
    s.core.rename(SHARED, "actas", "reuniones").await.unwrap();
}

#[tokio::test]
async fn refuses_folders_outside_them() {
    let server = TestServer::start();
    let s = connect(&server, "simple").await;
    server.sh("echo -n x > /srv/otra/dato.txt");
    let outside = Error::OutsideAllowed;

    for path in ["/", "/home", "/home/bob", "/srv/otra", "/home/alicia", "/home/alice/../bob"] {
        assert_eq!(s.core.list(path).await.unwrap_err(), outside, "{path}");
    }
    assert_eq!(s.core.create_folder("/srv/otra", "mia").await.unwrap_err(), outside);
    assert_eq!(s.core.rename("/srv/otra", "dato.txt", "otro.txt").await.unwrap_err(), outside);
    assert_eq!(s.core.trash("/srv/otra", "dato.txt").await.unwrap_err(), outside);
    assert_eq!(server.sh("cat /srv/otra/dato.txt"), "x");
}

#[tokio::test]
async fn refuses_transfers_outside_them() {
    let server = TestServer::start();
    let s = connect(&server, "simple").await;
    server.sh("echo -n x > /srv/otra/dato.txt");
    let local = s.dir.path().join("nota.txt");
    fs::write(&local, "hola").unwrap();

    let upload = Transfer::upload(&local, "/srv/otra");
    let download = Transfer::download("/srv/otra", "dato.txt", s.dir.path());

    assert_eq!(s.core.run(&upload, |_| {}).await.unwrap_err(), Error::OutsideAllowed);
    assert_eq!(s.core.run(&download, |_| {}).await.unwrap_err(), Error::OutsideAllowed);
    assert_eq!(server.sh("ls /srv/otra"), "dato.txt");
}

#[tokio::test]
async fn does_not_restore_outside_them() {
    let server = TestServer::start();
    let trash = "/home/alice/.local/share/Trash";
    server.sh(&format!(
        "mkdir -p {trash}/files {trash}/info && echo -n x > {trash}/files/dato.txt \
         && printf '[Trash Info]\\nPath=/srv/otra/dato.txt\\nDeletionDate=2026-01-02T03:04:05\\n' > {trash}/info/dato.txt.trashinfo \
         && chown -R alice:alice /home/alice/.local"
    ));
    let s = connect(&server, "simple").await;

    assert_eq!(s.core.restore("dato.txt").await.unwrap_err(), Error::OutsideAllowed);
    assert_eq!(s.core.list_trash().await.unwrap().len(), 1);
}

#[tokio::test]
async fn has_no_terminal_nor_deleting_forever() {
    let server = TestServer::start();
    server.put_file("alice", "notas.txt", "hola");
    let s = connect(&server, "simple").await;
    s.core.trash(HOME, "notas.txt").await.unwrap();
    let id = s.core.list_trash().await.unwrap()[0].id.clone();

    assert_eq!(s.core.open_terminal(1, 80, 24, |_| {}).await.unwrap_err(), Error::AdvancedOnly);
    assert_eq!(s.core.terminal_input(1, b"ls\n").await.unwrap_err(), Error::NotConnected);
    assert_eq!(s.core.delete_forever(&id).await.unwrap_err(), Error::AdvancedOnly);
    assert_eq!(s.core.empty_trash().await.unwrap_err(), Error::AdvancedOnly);
    assert_eq!(s.core.list_trash().await.unwrap().len(), 1);
    assert!(!server.logs().contains("pty"), "no se llega a pedir un PTY");
}

#[tokio::test]
async fn advanced_mode_is_not_limited() {
    let server = TestServer::start();
    let s = connect(&server, "advanced").await;

    s.core.list("/").await.unwrap();
    s.core.list("/srv/otra").await.unwrap();
}
