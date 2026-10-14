//! Terminal: ejecutar comandos en el Servidor y ver sus respuestas, a través de
//! la interfaz del núcleo y contra el servidor OpenSSH de prueba.

mod support;

use std::fs;
use std::time::Duration;

use sshpanel_core::{Core, Error, MemorySecretStore, TerminalEvent};
use support::TestServer;
use tempfile::TempDir;
use tokio::sync::mpsc::{UnboundedReceiver, unbounded_channel};

struct Session {
    _dir: TempDir,
    core: Core,
}

/// `alice` conectada con un Perfil en `mode`.
async fn connect(server: &TestServer, mode: &str) -> Session {
    let dir = TempDir::new().unwrap();
    let core = Core::new(dir.path().join("data"), Box::new(MemorySecretStore::default()));
    let path = dir.path().join("perfil.dbdesp");
    fs::write(
        &path,
        format!(
            r#"{{"version":1,"host":"{}","port":{},"username":"alice","fingerprint":"{}","mode":"{mode}"}}"#,
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

/// Lo que va llegando de una Terminal abierta.
struct Screen {
    events: UnboundedReceiver<TerminalEvent>,
    text: String,
}

impl Screen {
    async fn open(core: &Core) -> Self {
        let (tx, events) = unbounded_channel();
        core.open_terminal(80, 24, move |event| {
            let _ = tx.send(event);
        })
        .await
        .unwrap();
        Self { events, text: String::new() }
    }

    /// Espera hasta 10 s a que la salida contenga `needle`.
    async fn wait_for(&mut self, needle: &str) {
        let wait = async {
            while !self.text.contains(needle) {
                match self.events.recv().await {
                    Some(TerminalEvent::Output(data)) => self.text.push_str(&String::from_utf8_lossy(&data)),
                    other => panic!("la Terminal se cerró esperando {needle:?}: {other:?}\n{}", self.text),
                }
            }
        };
        tokio::time::timeout(Duration::from_secs(10), wait).await.unwrap_or_else(|_| panic!("no llegó {needle:?}:\n{}", self.text));
    }

    /// Espera hasta 10 s a que la sesión termine.
    async fn wait_closed(&mut self) {
        let wait = async {
            loop {
                match self.events.recv().await {
                    Some(TerminalEvent::Output(_)) => {}
                    Some(TerminalEvent::Closed) | None => return,
                }
            }
        };
        tokio::time::timeout(Duration::from_secs(10), wait).await.expect("la Terminal no se cerró");
    }
}

#[tokio::test]
async fn runs_commands_and_shows_their_output() {
    let server = TestServer::start();
    server.put_file("alice", "notas.txt", "hola");
    let s = connect(&server, "advanced").await;
    let mut screen = Screen::open(&s.core).await;

    // Lo tecleado vuelve como eco; el resultado no coincide con lo tecleado.
    s.core.terminal_input(b"echo $((6*7)); cat notas.txt | tr a-z A-Z\n").await.unwrap();
    screen.wait_for("42").await;
    screen.wait_for("HOLA").await;
}

#[tokio::test]
async fn resizing_reaches_the_server() {
    let server = TestServer::start();
    let s = connect(&server, "advanced").await;
    let mut screen = Screen::open(&s.core).await;

    s.core.resize_terminal(100, 30).await.unwrap();
    s.core.terminal_input(b"stty size\n").await.unwrap();
    screen.wait_for("30 100").await;
}

#[tokio::test]
async fn exiting_closes_the_session() {
    let server = TestServer::start();
    let s = connect(&server, "advanced").await;
    let mut screen = Screen::open(&s.core).await;

    s.core.terminal_input(b"exit\n").await.unwrap();
    screen.wait_closed().await;
}

#[tokio::test]
async fn reconnecting_or_opening_another_closes_the_previous_one() {
    let server = TestServer::start();
    let s = connect(&server, "advanced").await;

    let mut first = Screen::open(&s.core).await;
    let mut second = Screen::open(&s.core).await;
    first.wait_closed().await;

    s.core.connect().await.unwrap();
    second.wait_closed().await;
    assert_eq!(s.core.terminal_input(b"ls\n").await.unwrap_err(), Error::NotConnected);
}

#[tokio::test]
async fn needs_a_connection() {
    let dir = TempDir::new().unwrap();
    let core = Core::new(dir.path().join("data"), Box::new(MemorySecretStore::default()));

    assert_eq!(core.open_terminal(80, 24, |_| {}).await.unwrap_err(), Error::NotConnected);
    assert_eq!(core.terminal_input(b"ls\n").await.unwrap_err(), Error::NotConnected);
}
