//! El Servidor decide lo que puede hacer cada Miembro (ADR 0006): con las
//! opciones de `authorized_keys` de cada modo, editar el Perfil no da nada que
//! el Administrador no haya permitido. A través de la interfaz del núcleo y
//! contra el servidor OpenSSH de prueba.

mod support;

use std::fs;
use std::time::Duration;

use sshpanel_core::{Core, Error, MemorySecretStore, TerminalEvent, Transfer};
use support::TestServer;
use tempfile::TempDir;
use tokio::sync::mpsc::unbounded_channel;

/// Las opciones que genera «Autorizar clave» para cada modo.
const SIMPLE: &str = r#"restrict,command="internal-sftp""#;
const ADVANCED: &str = "restrict,pty";

struct Session {
    dir: TempDir,
    core: Core,
}

/// `alice` con un Perfil en Modo avanzado (como si lo hubiera editado a mano)
/// y su clave autorizada con `options`.
async fn connect(server: &TestServer, options: &str) -> Session {
    let dir = TempDir::new().unwrap();
    let core = Core::new(dir.path().join("data"), Box::new(MemorySecretStore::default()));
    let path = dir.path().join("perfil.dbdesp");
    fs::write(
        &path,
        format!(
            r#"{{"version":1,"host":"{}","port":{},"username":"alice","fingerprint":"{}","mode":"advanced","admin":true}}"#,
            server.host(),
            server.port(),
            server.fingerprint()
        ),
    )
    .unwrap();
    core.import_profile(&path).unwrap();
    server.authorize_with("alice", options, &core.public_key().unwrap());
    core.connect().await.unwrap();
    Session { dir, core }
}

#[tokio::test]
async fn a_simple_key_gets_files_but_no_terminal_whatever_the_profile_says() {
    let server = TestServer::start();
    server.put_file("alice", "notas.txt", "hola");
    let s = connect(&server, SIMPLE).await;

    assert_eq!(s.core.open_terminal(1, 80, 24, |_| {}).await.unwrap_err(), Error::TerminalFailed);

    let local = s.dir.path().join("subida.txt");
    fs::write(&local, "x").unwrap();
    s.core.run(&Transfer::upload(&local, "/home/alice"), |_| {}).await.unwrap();
    let id = s.core.trash("/home/alice", "notas.txt").await.unwrap();
    s.core.restore(&id).await.unwrap();
    assert_eq!(server.sh("cat /home/alice/subida.txt /home/alice/notas.txt"), "xhola");
}

#[tokio::test]
async fn an_advanced_key_gets_the_terminal() {
    let server = TestServer::start();
    let s = connect(&server, ADVANCED).await;
    let (tx, mut events) = unbounded_channel();
    s.core
        .open_terminal(1, 80, 24, move |event| {
            let _ = tx.send(event);
        })
        .await
        .unwrap();

    s.core.terminal_input(1, b"echo $((6*7))\n").await.unwrap();

    let mut text = String::new();
    let wait = async {
        while !text.contains("42") {
            match events.recv().await {
                Some(TerminalEvent::Output(data)) => text.push_str(&String::from_utf8_lossy(&data)),
                other => panic!("la Terminal se cerró: {other:?}"),
            }
        }
    };
    tokio::time::timeout(Duration::from_secs(10), wait).await.expect("no llegó la respuesta");
}
