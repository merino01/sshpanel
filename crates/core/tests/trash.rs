//! Papelera: borrar mueve a la Papelera y se puede restaurar, a través de la
//! interfaz del núcleo y contra el servidor OpenSSH de prueba.

mod support;

use std::fs;

use sshpanel_core::{Core, EntryKind, Error, Listing, MemorySecretStore, Restored};
use support::TestServer;
use tempfile::TempDir;

const HOME: &str = "/home/alice";
const TRASH: &str = "/home/alice/.local/share/Trash";

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
async fn deleting_moves_files_and_folders_to_the_trash() {
    let server = TestServer::start();
    server.put_file("alice", "notas.txt", "hola");
    server.mkdir("alice", "fotos");
    server.put_file("alice", "fotos/playa.jpg", "jpg");
    let s = connect(&server).await;

    s.core.trash(HOME, "notas.txt").await.unwrap();
    s.core.trash(HOME, "fotos").await.unwrap();

    let home = s.core.list(HOME).await.unwrap();
    assert_eq!(kind_of(&home, "notas.txt"), None);
    assert_eq!(kind_of(&home, "fotos"), None);
    assert_eq!(server.sh(&format!("cat {TRASH}/files/notas.txt {TRASH}/files/fotos/playa.jpg")), "holajpg");
    let info = server.sh(&format!("cat {TRASH}/info/notas.txt.trashinfo"));
    assert!(info.starts_with("[Trash Info]\nPath=/home/alice/notas.txt\nDeletionDate=20"), "{info}");

    let trash = s.core.list_trash().await.unwrap();
    let notas = trash.iter().find(|t| t.name == "notas.txt").unwrap();
    assert_eq!((notas.original_dir.as_str(), notas.kind, notas.size), (HOME, EntryKind::File, 4));
    assert!(notas.deleted.is_some());
    let fotos = trash.iter().find(|t| t.name == "fotos").unwrap();
    assert_eq!(fotos.kind, EntryKind::Dir);
}

#[tokio::test]
async fn restoring_puts_it_back_where_it_was() {
    let server = TestServer::start();
    server.mkdir("alice", "proyectos");
    server.put_file("alice", "proyectos/plan.txt", "plan");
    let s = connect(&server).await;
    s.core.trash("/home/alice/proyectos", "plan.txt").await.unwrap();

    let id = s.core.list_trash().await.unwrap()[0].id.clone();
    let restored = s.core.restore(&id).await.unwrap();

    assert_eq!(restored, Restored { dir: "/home/alice/proyectos".into(), name: "plan.txt".into() });
    assert_eq!(server.sh("cat /home/alice/proyectos/plan.txt"), "plan");
    assert!(s.core.list_trash().await.unwrap().is_empty());
    assert_eq!(server.sh(&format!("ls -A {TRASH}/files {TRASH}/info | grep -v : | wc -w")), "0");
}

#[tokio::test]
async fn restoring_onto_an_occupied_name_keeps_both() {
    let server = TestServer::start();
    server.put_file("alice", "notas.txt", "vieja");
    let s = connect(&server).await;
    s.core.trash(HOME, "notas.txt").await.unwrap();
    server.put_file("alice", "notas.txt", "nueva");

    let id = s.core.list_trash().await.unwrap()[0].id.clone();
    let restored = s.core.restore(&id).await.unwrap();

    assert_eq!(restored.name, "notas (1).txt");
    assert_eq!(server.sh("cat '/home/alice/notas.txt' '/home/alice/notas (1).txt'"), "nuevavieja");
}

#[tokio::test]
async fn restoring_recreates_a_folder_that_no_longer_exists() {
    let server = TestServer::start();
    server.mkdir("alice", "a/b");
    server.put_file("alice", "a/b/dentro.txt", "x");
    let s = connect(&server).await;
    s.core.trash("/home/alice/a/b", "dentro.txt").await.unwrap();
    server.sh("rm -r /home/alice/a");

    let id = s.core.list_trash().await.unwrap()[0].id.clone();
    s.core.restore(&id).await.unwrap();

    assert_eq!(server.sh("cat /home/alice/a/b/dentro.txt"), "x");
}

#[tokio::test]
async fn two_deleted_items_with_the_same_name_go_back_to_their_own_place() {
    let server = TestServer::start();
    server.mkdir("alice", "uno");
    server.mkdir("alice", "dos");
    server.put_file("alice", "uno/informe.pdf", "1");
    server.put_file("alice", "dos/informe.pdf", "2");
    let s = connect(&server).await;
    s.core.trash("/home/alice/uno", "informe.pdf").await.unwrap();
    s.core.trash("/home/alice/dos", "informe.pdf").await.unwrap();

    let trash = s.core.list_trash().await.unwrap();
    assert_eq!(trash.len(), 2);
    assert_ne!(trash[0].id, trash[1].id);
    for item in &trash {
        assert_eq!(s.core.restore(&item.id).await.unwrap().name, "informe.pdf");
    }

    assert_eq!(server.sh("cat /home/alice/uno/informe.pdf /home/alice/dos/informe.pdf"), "12");
}

#[tokio::test]
async fn names_with_spaces_accents_and_percent_signs_survive() {
    let server = TestServer::start();
    server.sh("mkdir '/home/alice/mis cosas' && echo -n x > '/home/alice/mis cosas/año 100%.txt' && chown -R alice:alice '/home/alice/mis cosas'");
    let s = connect(&server).await;
    s.core.trash("/home/alice/mis cosas", "año 100%.txt").await.unwrap();

    let trash = s.core.list_trash().await.unwrap();
    assert_eq!((trash[0].name.as_str(), trash[0].original_dir.as_str()), ("año 100%.txt", "/home/alice/mis cosas"));
    s.core.restore(&trash[0].id).await.unwrap();

    assert_eq!(server.sh("cat '/home/alice/mis cosas/año 100%.txt'"), "x");
}

#[tokio::test]
async fn deleting_from_the_shared_folder_goes_to_the_members_trash() {
    let server = TestServer::start();
    server.sh("mkdir -p /srv/compartida && chmod 777 /srv/compartida && echo -n acta > /srv/compartida/acta.txt");
    let s = connect(&server).await;

    s.core.trash("/srv/compartida", "acta.txt").await.unwrap();
    assert_eq!(kind_of(&s.core.list("/srv/compartida").await.unwrap(), "acta.txt"), None);

    let id = s.core.list_trash().await.unwrap()[0].id.clone();
    assert_eq!(s.core.restore(&id).await.unwrap().dir, "/srv/compartida");
    assert_eq!(server.sh("cat /srv/compartida/acta.txt"), "acta");
}

#[tokio::test]
async fn refuses_to_delete_the_trash_or_what_contains_it() {
    let server = TestServer::start();
    server.put_file("alice", "notas.txt", "hola");
    let s = connect(&server).await;
    s.core.trash(HOME, "notas.txt").await.unwrap();

    assert_eq!(s.core.trash(HOME, ".local").await.unwrap_err(), Error::TrashProtected);
    assert_eq!(s.core.trash("/home/alice/.local/share", "Trash").await.unwrap_err(), Error::TrashProtected);
    assert_eq!(s.core.trash(&format!("{TRASH}/files"), "notas.txt").await.unwrap_err(), Error::TrashProtected);
    assert_eq!(s.core.list_trash().await.unwrap().len(), 1);
}

#[tokio::test]
async fn a_refused_delete_leaves_nothing_behind() {
    let server = TestServer::start();
    let s = connect(&server).await;

    assert_eq!(s.core.trash("/etc", "hostname").await.unwrap_err(), Error::PermissionDenied);
    assert_eq!(s.core.trash(HOME, "fantasma").await.unwrap_err(), Error::NotFound);

    assert!(s.core.list_trash().await.unwrap().is_empty());
    assert_eq!(server.sh("cat /etc/hostname | wc -l"), "1");
}

#[tokio::test]
async fn reads_a_trash_written_by_other_tools() {
    let server = TestServer::start();
    server.sh(&format!(
        "mkdir -p {TRASH}/files {TRASH}/info && echo -n x > {TRASH}/files/viejo.txt \
         && printf '[Trash Info]\\nPath=/home/alice/docs/viejo%%20archivo.txt\\nDeletionDate=2026-01-02T03:04:05\\n' > {TRASH}/info/viejo.txt.trashinfo \
         && printf 'basura' > {TRASH}/info/roto.trashinfo \
         && chown -R alice:alice /home/alice/.local"
    ));
    let s = connect(&server).await;

    let trash = s.core.list_trash().await.unwrap();

    assert_eq!(trash.len(), 1, "el .trashinfo roto se omite");
    assert_eq!((trash[0].name.as_str(), trash[0].original_dir.as_str()), ("viejo archivo.txt", "/home/alice/docs"));
    assert_eq!(trash[0].deleted, Some(1_767_323_045));
    s.core.restore(&trash[0].id).await.unwrap();
    assert_eq!(server.sh("cat '/home/alice/docs/viejo archivo.txt'"), "x");
}

#[tokio::test]
async fn an_empty_trash_lists_nothing() {
    let server = TestServer::start();
    let s = connect(&server).await;

    assert!(s.core.list_trash().await.unwrap().is_empty());
    assert_eq!(s.core.restore("fantasma").await.unwrap_err(), Error::NotFound);
}

#[tokio::test]
async fn the_trash_requires_a_connection() {
    let dir = TempDir::new().unwrap();
    let core = Core::new(dir.path(), Box::new(MemorySecretStore::default()));

    assert_eq!(core.trash(HOME, "x").await.unwrap_err(), Error::NotConnected);
    assert_eq!(core.list_trash().await.unwrap_err(), Error::NotConnected);
    assert_eq!(core.restore("x").await.unwrap_err(), Error::NotConnected);
}
