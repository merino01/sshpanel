//! Editor: leer un archivo de texto y guardarlo de vuelta, a través de la
//! interfaz del núcleo y contra el servidor OpenSSH de prueba.

mod support;

use std::fs;

use sshpanel_core::{Core, Error, MemorySecretStore, Stamp};
use support::TestServer;
use tempfile::TempDir;

const NOTES: &str = "/home/alice/notas.txt";

struct Session {
    _dir: TempDir,
    core: Core,
}

/// `alice` conectada en Modo avanzado.
async fn connect(server: &TestServer) -> Session {
    let dir = TempDir::new().unwrap();
    let core = Core::new(dir.path().join("data"), Box::new(MemorySecretStore::default()));
    let path = dir.path().join("perfil.sshpanel");
    fs::write(
        &path,
        format!(
            r#"{{"version":1,"host":"{}","port":{},"username":"alice","fingerprint":"{}","mode":"advanced"}}"#,
            server.host(),
            server.port(),
            server.fingerprint()
        ),
    )
    .unwrap();
    core.import_profile(&path).unwrap();
    server.authorize("alice", &core.public_key().unwrap());
    core.connect().await.unwrap();
    Session { _dir: dir, core }
}

/// Los bytes del archivo `path` del Servidor, en hexadecimal.
fn hex_on_server(server: &TestServer, path: &str) -> String {
    server.sh(&format!("od -An -v -tx1 '{path}' | tr -d ' \\n'"))
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Tamaño y fecha de modificación del archivo `path` según el Servidor.
fn stamp_on_server(server: &TestServer, path: &str) -> Stamp {
    let out = server.sh(&format!("stat -c '%s %Y' '{path}'"));
    let (size, modified) = out.split_once(' ').unwrap();
    Stamp { size: size.parse().unwrap(), modified: Some(modified.parse().unwrap()) }
}

#[tokio::test]
async fn reads_a_text_file_and_saves_it() {
    let server = TestServer::start();
    server.put_file("alice", "notas.txt", "hola\nadiós");
    let s = connect(&server).await;

    let file = s.core.read_text(NOTES).await.unwrap();
    assert_eq!(file.text, "hola\nadiós");
    assert_eq!(file.stamp, stamp_on_server(&server, NOTES));

    let text = "línea uno\r\nlínea dos\r\nsin salto final";
    let stamp = s.core.write_text(NOTES, text).await.unwrap();

    assert_eq!(hex_on_server(&server, NOTES), hex(text.as_bytes()));
    assert_eq!(stamp, stamp_on_server(&server, NOTES));
    assert_eq!(s.core.read_text(NOTES).await.unwrap().text, text);
}

#[tokio::test]
async fn saving_a_shorter_text_leaves_nothing_of_the_old_one() {
    let server = TestServer::start();
    server.put_file("alice", "notas.txt", "un texto bastante largo");
    let s = connect(&server).await;

    s.core.write_text(NOTES, "corto").await.unwrap();
    s.core.write_text("/home/alice/notas.txt", "").await.unwrap();

    assert_eq!(server.sh("wc -c < /home/alice/notas.txt"), "0");
}

#[tokio::test]
async fn reads_and_saves_a_big_file() {
    let server = TestServer::start();
    let s = connect(&server).await;
    server.put_file("alice", "grande.txt", "");
    let text: String = (0..60_000).map(|n| format!("línea número {n}\n")).collect();
    assert!(text.len() > 512 * 1024);

    s.core.write_text("/home/alice/grande.txt", &text).await.unwrap();
    let file = s.core.read_text("/home/alice/grande.txt").await.unwrap();

    assert_eq!(file.text, text);
    assert_eq!(file.stamp.size, text.len() as u64);
    assert_eq!(server.sh("wc -l < /home/alice/grande.txt"), "60000");
}

#[tokio::test]
async fn saving_keeps_the_permissions_and_the_owner() {
    let server = TestServer::start();
    server.sh("mkdir -p /srv/compartida && chmod 777 /srv/compartida");
    server.sh("printf 'de bob' > /srv/compartida/acta.txt && chown bob:alice /srv/compartida/acta.txt && chmod 664 /srv/compartida/acta.txt");
    let s = connect(&server).await;

    s.core.write_text("/srv/compartida/acta.txt", "de alice").await.unwrap();

    assert_eq!(server.sh("cat /srv/compartida/acta.txt"), "de alice");
    assert_eq!(server.sh("stat -c '%a %U' /srv/compartida/acta.txt"), "664 bob");
}

#[tokio::test]
async fn reads_only_the_head_of_a_file() {
    let server = TestServer::start();
    server.sh("head -c 20000 /dev/zero > /home/alice/disco.img && chown alice /home/alice/disco.img");
    server.put_file("alice", "corto.txt", "hola");
    let s = connect(&server).await;

    let head = s.core.read_head("/home/alice/disco.img").await.unwrap();
    assert_eq!(head.size, 20000);
    assert_eq!(head.bytes.len(), 8 * 1024);
    assert!(head.bytes.iter().all(|&b| b == 0));

    let head = s.core.read_head("/home/alice/corto.txt").await.unwrap();
    assert_eq!((head.bytes.as_slice(), head.size), (b"hola".as_slice(), 4));
}

#[tokio::test]
async fn a_file_that_is_not_utf8_is_not_text() {
    let server = TestServer::start();
    server.sh("printf 'ma\\361ana' > /home/alice/latin1.txt && chown alice /home/alice/latin1.txt");
    let s = connect(&server).await;

    assert_eq!(s.core.read_text("/home/alice/latin1.txt").await.unwrap_err(), Error::NotText);
}

#[tokio::test]
async fn explains_what_went_wrong() {
    let server = TestServer::start();
    server.mkdir("alice", "fotos");
    server.sh("printf 'de root' > /srv/root.txt && chmod 644 /srv/root.txt");
    let s = connect(&server).await;

    assert_eq!(s.core.read_text("/home/alice/no-existe.txt").await.unwrap_err(), Error::NotFound);
    assert_eq!(s.core.read_head("/home/alice/no-existe.txt").await.unwrap_err(), Error::NotFound);
    assert_eq!(s.core.read_text("/home/alice/fotos").await.unwrap_err(), Error::ReadFailed);
    assert_eq!(s.core.read_head("/home/alice/fotos").await.unwrap_err(), Error::ReadFailed);
    assert_eq!(s.core.write_text("/srv/root.txt", "mío").await.unwrap_err(), Error::PermissionDenied);
    assert_eq!(server.sh("cat /srv/root.txt"), "de root");
    assert_eq!(s.core.read_text("notas.txt").await.unwrap_err(), Error::InvalidName);
}

#[tokio::test]
async fn saving_a_file_that_no_longer_exists_does_not_create_it() {
    let server = TestServer::start();
    server.put_file("alice", "notas.txt", "hola");
    let s = connect(&server).await;
    s.core.read_text(NOTES).await.unwrap();
    server.sh("rm /home/alice/notas.txt");

    assert_eq!(s.core.write_text(NOTES, "adiós").await.unwrap_err(), Error::NotFound);
    assert_eq!(server.sh("ls /home/alice"), "");
}

#[tokio::test]
async fn needs_a_connection() {
    let dir = TempDir::new().unwrap();
    let core = Core::new(dir.path().join("data"), Box::new(MemorySecretStore::default()));

    assert_eq!(core.read_head(NOTES).await.unwrap_err(), Error::NotConnected);
    assert_eq!(core.read_text(NOTES).await.unwrap_err(), Error::NotConnected);
    assert_eq!(core.write_text(NOTES, "hola").await.unwrap_err(), Error::NotConnected);
}
