//! Transferencias: subir y descargar archivos y carpetas, con progreso, a
//! través de la interfaz del núcleo y contra el servidor OpenSSH de prueba.

mod support;

use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

use sshpanel_core::{Core, Error, MemorySecretStore, Progress};
use support::TestServer;
use tempfile::TempDir;

const HOME: &str = "/home/alice";

/// `alice` conectada al servidor de prueba, con una carpeta local vacía.
struct Session {
    dir: TempDir,
    core: Core,
}

impl Session {
    fn local(&self) -> &Path {
        self.dir.path()
    }
}

async fn connect(server: &TestServer) -> Session {
    let dir = TempDir::new().unwrap();
    let core = Core::new(dir.path().join(".data"), Box::new(MemorySecretStore::default()));
    let path = dir.path().join(".perfil.dbdesp");
    fs::write(
        &path,
        format!(
            r#"{{"version":1,"host":"{}","port":{},"username":"alice","fingerprint":"{}","mode":"simple"}}"#,
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

fn names(core_listing: sshpanel_core::Listing) -> Vec<String> {
    core_listing.entries.into_iter().map(|e| e.name).collect()
}

#[tokio::test]
async fn uploads_a_file_reporting_progress() {
    let server = TestServer::start();
    let s = connect(&server).await;
    let file = s.local().join("notas.txt");
    fs::write(&file, "hola, servidor").unwrap();

    let mut reports = Vec::new();
    let name = s.core.upload(&file, HOME, |p| reports.push(p)).await.unwrap();

    assert_eq!(name, "notas.txt");
    assert_eq!(server.sh("cat /home/alice/notas.txt"), "hola, servidor");
    assert_eq!(reports.first(), Some(&Progress { done: 0, total: 14 }));
    assert_eq!(reports.last(), Some(&Progress { done: 14, total: 14 }));
}

#[tokio::test]
async fn uploads_a_folder_with_everything_inside() {
    let server = TestServer::start();
    let s = connect(&server).await;
    let folder = s.local().join("viaje");
    fs::create_dir_all(folder.join("fotos/vacía")).unwrap();
    fs::write(folder.join("itinerario.txt"), "día 1").unwrap();
    fs::write(folder.join("fotos/playa.jpg"), "jpg").unwrap();

    let mut last = None;
    s.core.upload(&folder, HOME, |p| last = Some(p)).await.unwrap();

    assert_eq!(server.sh("cat /home/alice/viaje/itinerario.txt"), "día 1");
    assert_eq!(server.sh("cat /home/alice/viaje/fotos/playa.jpg"), "jpg");
    assert_eq!(server.sh("test -d /home/alice/viaje/fotos/vacía && echo sí"), "sí");
    assert_eq!(last, Some(Progress { done: 9, total: 9 }));
}

#[tokio::test]
async fn downloads_a_file_and_a_folder() {
    let server = TestServer::start();
    server.put_file("alice", "informe.txt", "contenido");
    server.mkdir("alice", "proyecto/src");
    server.put_file("alice", "proyecto/src/main.rs", "fn main() {}");
    let s = connect(&server).await;

    let mut last = None;
    let file = s.core.download(HOME, "informe.txt", s.local(), |p| last = Some(p)).await.unwrap();
    let folder = s.core.download(HOME, "proyecto", s.local(), |_| {}).await.unwrap();

    assert_eq!(file, s.local().join("informe.txt"));
    assert_eq!(fs::read_to_string(&file).unwrap(), "contenido");
    assert_eq!(last, Some(Progress { done: 9, total: 9 }));
    assert_eq!(fs::read_to_string(folder.join("src/main.rs")).unwrap(), "fn main() {}");
}

#[tokio::test]
async fn never_replaces_what_already_exists() {
    let server = TestServer::start();
    server.put_file("alice", "notas.txt", "en el servidor");
    let s = connect(&server).await;
    let local = s.local().join("notas.txt");
    fs::write(&local, "en el equipo").unwrap();

    let uploaded = s.core.upload(&local, HOME, |_| {}).await.unwrap();
    let downloaded = s.core.download(HOME, "notas.txt", s.local(), |_| {}).await.unwrap();

    assert_eq!(uploaded, "notas (1).txt");
    assert_eq!(server.sh("cat /home/alice/notas.txt"), "en el servidor");
    assert_eq!(server.sh("cat '/home/alice/notas (1).txt'"), "en el equipo");
    assert_eq!(downloaded, s.local().join("notas (1).txt"));
    assert_eq!(fs::read_to_string(&local).unwrap(), "en el equipo");
    assert_eq!(fs::read_to_string(&downloaded).unwrap(), "en el servidor");
}

#[tokio::test]
async fn leaves_no_partial_files_after_a_transfer() {
    let server = TestServer::start();
    server.put_file("alice", "a.txt", "a");
    let s = connect(&server).await;
    let local = s.local().join("b.txt");
    fs::write(&local, "b").unwrap();

    s.core.upload(&local, HOME, |_| {}).await.unwrap();
    s.core.download(HOME, "a.txt", s.local(), |_| {}).await.unwrap();

    assert_eq!(server.sh("ls -A /home/alice | grep -v '^.ssh$' | sort"), "a.txt\nb.txt");
    let mut local_names: Vec<_> = fs::read_dir(s.local()).unwrap().map(|e| e.unwrap().file_name().into_string().unwrap()).collect();
    local_names.sort();
    assert_eq!(local_names, [".data", ".perfil.dbdesp", "a.txt", "b.txt"]);
}

#[tokio::test]
async fn reports_what_the_server_does_not_allow() {
    let server = TestServer::start();
    server.sh("mkdir /home/alice/privada && chmod 000 /home/alice/privada");
    let s = connect(&server).await;
    let local = s.local().join("notas.txt");
    fs::write(&local, "x").unwrap();

    assert_eq!(s.core.upload(&local, "/", |_| {}).await.unwrap_err(), Error::PermissionDenied);
    assert_eq!(s.core.download("/etc", "shadow", s.local(), |_| {}).await.unwrap_err(), Error::PermissionDenied);
    assert_eq!(s.core.download(HOME, "privada", s.local(), |_| {}).await.unwrap_err(), Error::PermissionDenied);
    assert_eq!(s.core.download(HOME, "fantasma", s.local(), |_| {}).await.unwrap_err(), Error::NotFound);
}

#[tokio::test]
async fn reports_local_files_that_cannot_be_read_or_written() {
    let server = TestServer::start();
    server.put_file("alice", "notas.txt", "x");
    let s = connect(&server).await;

    let missing = s.local().join("no-existe.txt");
    assert_eq!(s.core.upload(&missing, HOME, |_| {}).await.unwrap_err(), Error::LocalUnreadable);
    let not_a_folder = s.local().join("no-existe");
    assert_eq!(s.core.download(HOME, "notas.txt", &not_a_folder, |_| {}).await.unwrap_err(), Error::LocalWriteFailed);
}

#[tokio::test]
async fn a_cut_connection_during_a_download_is_reported_and_leaves_nothing_half_done() {
    let server = TestServer::start();
    server.sh("dd if=/dev/urandom of=/home/alice/grande.bin bs=1M count=32 2>/dev/null && chown alice /home/alice/grande.bin");
    let s = connect(&server).await;

    let cut = AtomicBool::new(false);
    let result = s
        .core
        .download(HOME, "grande.bin", s.local(), |p| {
            if p.done > 0 && !cut.swap(true, Ordering::SeqCst) {
                server.sh("pkill -u alice");
            }
        })
        .await;

    assert_eq!(result.unwrap_err(), Error::ConnectionLost);
    assert!(!s.local().join("grande.bin").exists());
    assert!(!s.local().join(".grande.bin.part").exists());
}

#[tokio::test]
async fn a_cut_connection_during_an_upload_is_reported() {
    let server = TestServer::start();
    let s = connect(&server).await;
    let local = s.local().join("grande.bin");
    fs::write(&local, vec![7u8; 32 * 1024 * 1024]).unwrap();

    let cut = AtomicBool::new(false);
    let result = s
        .core
        .upload(&local, HOME, |p| {
            if p.done > 0 && !cut.swap(true, Ordering::SeqCst) {
                server.sh("pkill -u alice");
            }
        })
        .await;

    assert_eq!(result.unwrap_err(), Error::ConnectionLost);
    assert_eq!(server.sh("test -e /home/alice/grande.bin || echo no está"), "no está");
}

#[tokio::test]
async fn transfers_require_a_connection() {
    let dir = TempDir::new().unwrap();
    let core = Core::new(dir.path(), Box::new(MemorySecretStore::default()));

    assert_eq!(core.upload(dir.path(), HOME, |_| {}).await.unwrap_err(), Error::NotConnected);
    assert_eq!(core.download(HOME, "x", dir.path(), |_| {}).await.unwrap_err(), Error::NotConnected);
}

#[tokio::test]
async fn the_explorer_keeps_working_after_a_transfer() {
    let server = TestServer::start();
    server.put_file("alice", "a.txt", "a");
    let s = connect(&server).await;

    s.core.download(HOME, "a.txt", s.local(), |_| {}).await.unwrap();

    assert!(names(s.core.list(HOME).await.unwrap()).contains(&"a.txt".to_string()));
}
