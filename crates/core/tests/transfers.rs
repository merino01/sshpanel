//! Transferencias: subir y descargar archivos y carpetas, con progreso,
//! cancelar, reintentar, reanudar tras un corte y varias a la vez, a través de
//! la interfaz del núcleo y contra el servidor OpenSSH de prueba.

mod support;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};

use sshpanel_core::{Core, Error, MemorySecretStore, Outcome, Progress, Transfer};
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

async fn upload(core: &Core, local: &Path, remote_dir: &str) -> Result<Outcome, Error> {
    core.run(&Transfer::upload(local, remote_dir), |_| {}).await
}

async fn download(core: &Core, remote_dir: &str, name: &str, local_dir: &Path) -> Result<Outcome, Error> {
    core.run(&Transfer::download(remote_dir, name, local_dir), |_| {}).await
}

fn uploaded(name: &str) -> Outcome {
    Outcome::Uploaded(name.to_string())
}

fn downloaded(path: PathBuf) -> Outcome {
    Outcome::Downloaded(path)
}

/// Contenido variado, para que un trozo repetido o fuera de sitio se note.
fn pattern(len: usize) -> Vec<u8> {
    (0..len).map(|i| (i * 31 % 251) as u8).collect()
}

fn local_sha256(path: &Path) -> String {
    let out = Command::new("sha256sum").arg(path).output().unwrap();
    String::from_utf8(out.stdout).unwrap().split_whitespace().next().unwrap().to_string()
}

fn remote_sha256(server: &TestServer, path: &str) -> String {
    server.sh(&format!("sha256sum '{path}' | cut -d' ' -f1"))
}

/// Archivo aleatorio de `mib` MiB de `alice` en su carpeta personal.
fn put_big_file(server: &TestServer, name: &str, mib: u32) {
    server.sh(&format!("dd if=/dev/urandom of=/home/alice/{name} bs=1M count={mib} 2>/dev/null && chown alice /home/alice/{name}"));
}

/// Lo que hay en la carpeta personal de `alice`, sin `.ssh`.
fn remote_home(server: &TestServer) -> String {
    server.sh("ls -A /home/alice | grep -v '^.ssh$' | sort")
}

fn local_names(dir: &Path) -> Vec<String> {
    let mut names: Vec<_> = fs::read_dir(dir).unwrap().map(|e| e.unwrap().file_name().into_string().unwrap()).collect();
    names.sort();
    names
}

const CUT_AT: u64 = 8 * 1024 * 1024;

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
    let outcome = s.core.run(&Transfer::upload(&file, HOME), |p| reports.push(p)).await.unwrap();

    assert_eq!(outcome, uploaded("notas.txt"));
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
    s.core.run(&Transfer::upload(&folder, HOME), |p| last = Some(p)).await.unwrap();

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
    let file = s.core.run(&Transfer::download(HOME, "informe.txt", s.local()), |p| last = Some(p)).await.unwrap();
    let folder = download(&s.core, HOME, "proyecto", s.local()).await.unwrap();

    assert_eq!(file, downloaded(s.local().join("informe.txt")));
    assert_eq!(fs::read_to_string(s.local().join("informe.txt")).unwrap(), "contenido");
    assert_eq!(last, Some(Progress { done: 9, total: 9 }));
    assert_eq!(folder, downloaded(s.local().join("proyecto")));
    assert_eq!(fs::read_to_string(s.local().join("proyecto/src/main.rs")).unwrap(), "fn main() {}");
}

#[tokio::test]
async fn never_replaces_what_already_exists() {
    let server = TestServer::start();
    server.put_file("alice", "notas.txt", "en el servidor");
    let s = connect(&server).await;
    let local = s.local().join("notas.txt");
    fs::write(&local, "en el equipo").unwrap();

    let up = upload(&s.core, &local, HOME).await.unwrap();
    let down = download(&s.core, HOME, "notas.txt", s.local()).await.unwrap();

    assert_eq!(up, uploaded("notas (1).txt"));
    assert_eq!(server.sh("cat /home/alice/notas.txt"), "en el servidor");
    assert_eq!(server.sh("cat '/home/alice/notas (1).txt'"), "en el equipo");
    assert_eq!(down, downloaded(s.local().join("notas (1).txt")));
    assert_eq!(fs::read_to_string(&local).unwrap(), "en el equipo");
    assert_eq!(fs::read_to_string(s.local().join("notas (1).txt")).unwrap(), "en el servidor");
}

#[tokio::test]
async fn leaves_no_partial_files_after_a_transfer() {
    let server = TestServer::start();
    server.put_file("alice", "a.txt", "a");
    let s = connect(&server).await;
    let local = s.local().join("b.txt");
    fs::write(&local, "b").unwrap();

    upload(&s.core, &local, HOME).await.unwrap();
    download(&s.core, HOME, "a.txt", s.local()).await.unwrap();

    assert_eq!(remote_home(&server), "a.txt\nb.txt");
    assert_eq!(local_names(s.local()), [".data", ".perfil.dbdesp", "a.txt", "b.txt"]);
}

#[tokio::test]
async fn reports_what_the_server_does_not_allow() {
    let server = TestServer::start();
    server.sh("mkdir /home/alice/privada && chmod 000 /home/alice/privada");
    let s = connect(&server).await;
    let local = s.local().join("notas.txt");
    fs::write(&local, "x").unwrap();

    assert_eq!(upload(&s.core, &local, "/").await.unwrap_err(), Error::PermissionDenied);
    assert_eq!(download(&s.core, "/etc", "shadow", s.local()).await.unwrap_err(), Error::PermissionDenied);
    assert_eq!(download(&s.core, HOME, "privada", s.local()).await.unwrap_err(), Error::PermissionDenied);
    assert_eq!(download(&s.core, HOME, "fantasma", s.local()).await.unwrap_err(), Error::NotFound);
}

#[tokio::test]
async fn reports_local_files_that_cannot_be_read_or_written() {
    let server = TestServer::start();
    server.put_file("alice", "notas.txt", "x");
    let s = connect(&server).await;

    let missing = s.local().join("no-existe.txt");
    assert_eq!(upload(&s.core, &missing, HOME).await.unwrap_err(), Error::LocalUnreadable);
    let not_a_folder = s.local().join("no-existe");
    assert_eq!(download(&s.core, HOME, "notas.txt", &not_a_folder).await.unwrap_err(), Error::LocalWriteFailed);
}

#[tokio::test]
async fn transfers_require_a_connection() {
    let dir = TempDir::new().unwrap();
    let core = Core::new(dir.path(), Box::new(MemorySecretStore::default()));

    assert_eq!(upload(&core, dir.path(), HOME).await.unwrap_err(), Error::NotConnected);
    assert_eq!(download(&core, HOME, "x", dir.path()).await.unwrap_err(), Error::NotConnected);
}

#[tokio::test]
async fn the_explorer_keeps_working_after_a_transfer() {
    let server = TestServer::start();
    server.put_file("alice", "a.txt", "a");
    let s = connect(&server).await;

    download(&s.core, HOME, "a.txt", s.local()).await.unwrap();

    assert!(names(s.core.list(HOME).await.unwrap()).contains(&"a.txt".to_string()));
}

// Cancelar

#[tokio::test]
async fn cancelling_a_download_midway_leaves_nothing() {
    let server = TestServer::start();
    put_big_file(&server, "grande.bin", 32);
    let s = connect(&server).await;

    let transfer = Transfer::download(HOME, "grande.bin", s.local());
    let result = s.core.run(&transfer, |p| if p.done > 0 { transfer.cancel() }).await;

    assert_eq!(result.unwrap_err(), Error::Cancelled);
    assert_eq!(local_names(s.local()), [".data", ".perfil.dbdesp"]);
}

#[tokio::test]
async fn cancelling_a_folder_upload_midway_removes_what_it_created_and_nothing_else() {
    let server = TestServer::start();
    server.put_file("alice", "previo.txt", "ya estaba");
    let s = connect(&server).await;
    let folder = s.local().join("viaje");
    fs::create_dir_all(folder.join("fotos")).unwrap();
    for name in ["a.bin", "b.bin", "fotos/c.bin", "fotos/d.bin"] {
        fs::write(folder.join(name), pattern(2 * 1024 * 1024)).unwrap();
    }

    let transfer = Transfer::upload(&folder, HOME);
    let result = s.core.run(&transfer, |p| if p.done > 5 * 1024 * 1024 { transfer.cancel() }).await;

    assert_eq!(result.unwrap_err(), Error::Cancelled);
    assert_eq!(remote_home(&server), "previo.txt");
    assert_eq!(server.sh("cat /home/alice/previo.txt"), "ya estaba");
}

#[tokio::test]
async fn a_transfer_cancelled_before_it_starts_does_nothing() {
    let server = TestServer::start();
    let s = connect(&server).await;
    let local = s.local().join("notas.txt");
    fs::write(&local, "x").unwrap();

    let transfer = Transfer::upload(&local, HOME);
    transfer.cancel();

    assert_eq!(s.core.run(&transfer, |_| {}).await.unwrap_err(), Error::Cancelled);
    assert_eq!(remote_home(&server), "");
}

// Cortes, reanudar y reintentar

#[tokio::test]
async fn a_cut_download_resumes_where_it_stopped() {
    let server = TestServer::start();
    put_big_file(&server, "grande.bin", 32);
    let s = connect(&server).await;
    let transfer = Transfer::download(HOME, "grande.bin", s.local());

    let cut = AtomicBool::new(false);
    let first = s
        .core
        .run(&transfer, |p| {
            if p.done > CUT_AT && !cut.swap(true, Ordering::SeqCst) {
                server.sh("pkill -u alice");
            }
        })
        .await;
    assert_eq!(first.unwrap_err(), Error::ConnectionLost);
    assert!(!s.local().join("grande.bin").exists());
    assert!(s.local().join(".grande.bin.part").exists(), "lo recibido se guarda para reanudar");

    s.core.connect().await.unwrap();
    let mut reports = Vec::new();
    let second = s.core.run(&transfer, |p| reports.push(p)).await.unwrap();

    assert_eq!(second, downloaded(s.local().join("grande.bin")));
    assert!(reports[0].done >= CUT_AT, "sigue donde se quedó: {:?}", reports[0]);
    assert_eq!(local_sha256(&s.local().join("grande.bin")), remote_sha256(&server, "/home/alice/grande.bin"));
    assert_eq!(local_names(s.local()), [".data", ".perfil.dbdesp", "grande.bin"]);
}

#[tokio::test]
async fn a_cut_upload_resumes_where_it_stopped_with_the_same_name() {
    let server = TestServer::start();
    server.put_file("alice", "grande.bin", "ya estaba");
    let s = connect(&server).await;
    let local = s.local().join("grande.bin");
    fs::write(&local, pattern(32 * 1024 * 1024)).unwrap();
    let transfer = Transfer::upload(&local, HOME);

    let cut = AtomicBool::new(false);
    let first = s
        .core
        .run(&transfer, |p| {
            if p.done > CUT_AT && !cut.swap(true, Ordering::SeqCst) {
                server.sh("pkill -u alice");
            }
        })
        .await;
    assert_eq!(first.unwrap_err(), Error::ConnectionLost);
    assert_eq!(server.sh("test -e '/home/alice/grande (1).bin' || echo no está"), "no está");

    s.core.connect().await.unwrap();
    let mut reports = Vec::new();
    let second = s.core.run(&transfer, |p| reports.push(p)).await.unwrap();

    assert_eq!(second, uploaded("grande (1).bin"));
    assert!(reports[0].done > 0, "sigue donde se quedó: {:?}", reports[0]);
    assert_eq!(remote_sha256(&server, "/home/alice/grande (1).bin"), local_sha256(&local));
    assert_eq!(server.sh("cat /home/alice/grande.bin"), "ya estaba");
    assert_eq!(remote_home(&server), "grande (1).bin\ngrande.bin");
}

#[tokio::test]
async fn a_cut_folder_download_resumes_without_repeating_finished_files() {
    let server = TestServer::start();
    server.mkdir("alice", "fotos");
    for name in ["a.bin", "b.bin", "c.bin"] {
        put_big_file(&server, &format!("fotos/{name}"), 8);
    }
    let s = connect(&server).await;
    let transfer = Transfer::download(HOME, "fotos", s.local());

    let cut = AtomicBool::new(false);
    let first = s
        .core
        .run(&transfer, |p| {
            if p.done > 12 * 1024 * 1024 && !cut.swap(true, Ordering::SeqCst) {
                server.sh("pkill -u alice");
            }
        })
        .await;
    assert_eq!(first.unwrap_err(), Error::ConnectionLost);

    s.core.connect().await.unwrap();
    let mut reports = Vec::new();
    s.core.run(&transfer, |p| reports.push(p)).await.unwrap();

    assert!(reports[0].done >= 12 * 1024 * 1024, "sigue donde se quedó: {:?}", reports[0]);
    for name in ["a.bin", "b.bin", "c.bin"] {
        let local = s.local().join("fotos").join(name);
        assert_eq!(local_sha256(&local), remote_sha256(&server, &format!("/home/alice/fotos/{name}")));
    }
    assert_eq!(local_names(&s.local().join("fotos")), ["a.bin", "b.bin", "c.bin"]);
}

#[tokio::test]
async fn discarding_a_cut_transfer_removes_what_it_left() {
    let server = TestServer::start();
    put_big_file(&server, "grande.bin", 32);
    let s = connect(&server).await;
    let transfer = Transfer::download(HOME, "grande.bin", s.local());

    let cut = AtomicBool::new(false);
    let first = s
        .core
        .run(&transfer, |p| {
            if p.done > 0 && !cut.swap(true, Ordering::SeqCst) {
                server.sh("pkill -u alice");
            }
        })
        .await;
    assert_eq!(first.unwrap_err(), Error::ConnectionLost);

    s.core.discard(&transfer).await;

    assert_eq!(local_names(s.local()), [".data", ".perfil.dbdesp"]);
    assert_eq!(s.core.run(&transfer, |_| {}).await.unwrap_err(), Error::Cancelled);
}

#[tokio::test]
async fn a_failed_transfer_can_be_retried() {
    let server = TestServer::start();
    server.sh("mkdir /home/alice/privada && chown alice /home/alice/privada && chmod 500 /home/alice/privada");
    let s = connect(&server).await;
    let local = s.local().join("notas.txt");
    fs::write(&local, "por fin").unwrap();
    let transfer = Transfer::upload(&local, "/home/alice/privada");

    assert_eq!(s.core.run(&transfer, |_| {}).await.unwrap_err(), Error::PermissionDenied);
    server.sh("chmod 700 /home/alice/privada");

    assert_eq!(s.core.run(&transfer, |_| {}).await.unwrap(), uploaded("notas.txt"));
    assert_eq!(server.sh("cat /home/alice/privada/notas.txt"), "por fin");
}

// Varias a la vez

#[tokio::test]
async fn parallel_uploads_with_the_same_name_get_different_names() {
    let server = TestServer::start();
    let s = connect(&server).await;
    let (one, two) = (s.local().join("uno/notas.txt"), s.local().join("dos/notas.txt"));
    for (file, content) in [(&one, "primero"), (&two, "segundo")] {
        fs::create_dir(file.parent().unwrap()).unwrap();
        fs::write(file, content).unwrap();
    }

    let (a, b) = tokio::join!(upload(&s.core, &one, HOME), upload(&s.core, &two, HOME));

    let mut outcomes = [a.unwrap(), b.unwrap()];
    outcomes.sort_by_key(|o| format!("{o:?}"));
    assert_eq!(outcomes, [uploaded("notas (1).txt"), uploaded("notas.txt")]);
    let mut contents = [server.sh("cat /home/alice/notas.txt"), server.sh("cat '/home/alice/notas (1).txt'")];
    contents.sort();
    assert_eq!(contents, ["primero", "segundo"]);
}

#[tokio::test]
async fn parallel_downloads_all_complete() {
    let server = TestServer::start();
    for name in ["a.bin", "b.bin", "c.bin"] {
        put_big_file(&server, name, 8);
    }
    let s = connect(&server).await;

    let (a, b, c) = tokio::join!(
        download(&s.core, HOME, "a.bin", s.local()),
        download(&s.core, HOME, "b.bin", s.local()),
        download(&s.core, HOME, "c.bin", s.local()),
    );

    for (outcome, name) in [(a, "a.bin"), (b, "b.bin"), (c, "c.bin")] {
        assert_eq!(outcome.unwrap(), downloaded(s.local().join(name)));
        assert_eq!(local_sha256(&s.local().join(name)), remote_sha256(&server, &format!("/home/alice/{name}")));
    }
}
