//! Explorador: navegar, crear carpetas, renombrar y permisos denegados por el
//! Servidor, a través de la interfaz del núcleo y contra el servidor OpenSSH de prueba.

mod support;

use std::fs;

use sshpanel_core::{Core, EntryKind, Error, Listing, MemorySecretStore};
use support::TestServer;
use tempfile::TempDir;

const HOME: &str = "/home/alice";

/// `alice` conectada al servidor de prueba.
struct Session {
    _dir: TempDir,
    core: Core,
}

async fn connect(server: &TestServer) -> Session {
    let dir = TempDir::new().unwrap();
    let core = Core::new(dir.path().join("data"), Box::new(MemorySecretStore::default()));
    let path = dir.path().join("perfil.dbdesp");
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
    Session { _dir: dir, core }
}

fn kind_of(listing: &Listing, name: &str) -> Option<EntryKind> {
    listing.entries.iter().find(|e| e.name == name).map(|e| e.kind)
}

#[tokio::test]
async fn navigates_into_a_folder_and_back_up() {
    let server = TestServer::start();
    server.mkdir("alice", "fotos");
    let s = connect(&server).await;

    let fotos = s.core.list("/home/alice/fotos").await.unwrap();
    let home = s.core.list("/home").await.unwrap();

    assert_eq!(fotos.path, "/home/alice/fotos");
    assert_eq!(kind_of(&home, "alice"), Some(EntryKind::Dir));
}

#[tokio::test]
async fn a_link_to_a_folder_can_be_entered_like_a_folder() {
    let server = TestServer::start();
    server.sh("ln -s /tmp /home/alice/temporal && ln -s /nada /home/alice/roto");
    let s = connect(&server).await;

    let listing = s.core.list(HOME).await.unwrap();

    assert_eq!(kind_of(&listing, "temporal"), Some(EntryKind::Dir));
    assert_eq!(kind_of(&listing, "roto"), Some(EntryKind::Symlink));
    assert!(s.core.list("/home/alice/temporal").await.is_ok());
}

#[tokio::test]
async fn creates_a_folder() {
    let server = TestServer::start();
    let s = connect(&server).await;

    s.core.create_folder(HOME, "proyectos").await.unwrap();

    let listing = s.core.list(HOME).await.unwrap();
    assert_eq!(kind_of(&listing, "proyectos"), Some(EntryKind::Dir));
}

#[tokio::test]
async fn refuses_to_create_a_folder_that_already_exists() {
    let server = TestServer::start();
    server.put_file("alice", "notas", "hola");
    let s = connect(&server).await;

    assert_eq!(s.core.create_folder(HOME, "notas").await.unwrap_err(), Error::AlreadyExists);
}

#[tokio::test]
async fn renames_files_and_folders() {
    let server = TestServer::start();
    server.put_file("alice", "notas.txt", "hola");
    server.mkdir("alice", "fotos");
    let s = connect(&server).await;

    s.core.rename(HOME, "notas.txt", "apuntes.txt").await.unwrap();
    s.core.rename(HOME, "fotos", "imágenes").await.unwrap();

    let listing = s.core.list(HOME).await.unwrap();
    assert_eq!(kind_of(&listing, "apuntes.txt"), Some(EntryKind::File));
    assert_eq!(kind_of(&listing, "imágenes"), Some(EntryKind::Dir));
    assert_eq!(kind_of(&listing, "notas.txt"), None);
    assert_eq!(kind_of(&listing, "fotos"), None);
}

#[tokio::test]
async fn renaming_onto_an_existing_name_changes_nothing() {
    let server = TestServer::start();
    server.mkdir("alice", "a");
    server.mkdir("alice", "b");
    server.put_file("alice", "a/dentro.txt", "x");
    let s = connect(&server).await;

    // Sin la comprobación previa, el Servidor sustituiría la carpeta vacía `b`.
    assert_eq!(s.core.rename(HOME, "a", "b").await.unwrap_err(), Error::AlreadyExists);

    let listing = s.core.list(HOME).await.unwrap();
    assert_eq!(kind_of(&listing, "a"), Some(EntryKind::Dir));
    assert_eq!(s.core.list("/home/alice/a").await.unwrap().entries.len(), 1);
}

#[tokio::test]
async fn renaming_to_the_same_name_does_nothing() {
    let server = TestServer::start();
    server.put_file("alice", "notas.txt", "hola");
    let s = connect(&server).await;

    s.core.rename(HOME, "notas.txt", "notas.txt").await.unwrap();
}

#[tokio::test]
async fn rejects_names_that_are_not_a_single_name() {
    let server = TestServer::start();
    server.put_file("alice", "notas.txt", "hola");
    let s = connect(&server).await;

    for name in ["", "   ", ".", "..", "a/b", "../fuera"] {
        assert_eq!(s.core.create_folder(HOME, name).await.unwrap_err(), Error::InvalidName, "{name:?}");
        assert_eq!(s.core.rename(HOME, "notas.txt", name).await.unwrap_err(), Error::InvalidName, "{name:?}");
    }
}

#[tokio::test]
async fn trims_spaces_around_new_names() {
    let server = TestServer::start();
    let s = connect(&server).await;

    s.core.create_folder(HOME, "  facturas ").await.unwrap();

    assert_eq!(kind_of(&s.core.list(HOME).await.unwrap(), "facturas"), Some(EntryKind::Dir));
}

#[tokio::test]
async fn reports_what_the_server_does_not_allow() {
    let server = TestServer::start();
    server.sh("chmod 700 /home/bob");
    let s = connect(&server).await;

    assert_eq!(s.core.list("/home/bob").await.unwrap_err(), Error::PermissionDenied);
    assert_eq!(s.core.create_folder("/", "mia").await.unwrap_err(), Error::PermissionDenied);
    assert_eq!(s.core.rename("/etc", "hostname", "otro").await.unwrap_err(), Error::PermissionDenied);
}

#[tokio::test]
async fn reports_a_folder_that_does_not_exist() {
    let server = TestServer::start();
    let s = connect(&server).await;

    assert_eq!(s.core.list("/srv/compartida").await.unwrap_err(), Error::NotFound);
    assert_eq!(s.core.rename(HOME, "fantasma", "otro").await.unwrap_err(), Error::NotFound);
}

#[tokio::test]
async fn changing_files_requires_a_connection() {
    let dir = TempDir::new().unwrap();
    let core = Core::new(dir.path(), Box::new(MemorySecretStore::default()));

    assert_eq!(core.create_folder(HOME, "x").await.unwrap_err(), Error::NotConnected);
    assert_eq!(core.rename(HOME, "x", "y").await.unwrap_err(), Error::NotConnected);
}
