//! Administración: leer y cambiar las Claves públicas autorizadas de cada
//! usuario en `/etc/ssh/authorized_keys/`, a través de la interfaz del núcleo y
//! contra el servidor OpenSSH de prueba (alice tiene sudo; bob, no).

mod support;

use std::fs;

use sshpanel_core::{Core, Error, MemorySecretStore, UserKeys};
use support::TestServer;
use tempfile::TempDir;

struct Session {
    _dir: TempDir,
    core: Core,
}

const SIMPLE: &str = r#"restrict,command="internal-sftp""#;
const ADVANCED: &str = "restrict,pty";

/// `alice` conectada en Modo `mode`, con su clave autorizada con `options`.
async fn connect(server: &TestServer, mode: &str, options: &str) -> Session {
    let s = member(server, "alice", mode);
    server.authorize_with("alice", options, &format!("{} alice-portatil", s.core.public_key().unwrap()));
    s.core.connect().await.unwrap();
    s
}

/// Un Miembro `user` con su Perfil importado (Modo `mode`), sin conectar ni autorizar.
fn member(server: &TestServer, user: &str, mode: &str) -> Session {
    let dir = TempDir::new().unwrap();
    let core = Core::new(dir.path().join("data"), Box::new(MemorySecretStore::default()));
    let path = dir.path().join("perfil.sshpanel");
    fs::write(
        &path,
        format!(
            r#"{{"version":1,"host":"{}","port":{},"username":"{user}","fingerprint":"{}","mode":"{mode}","admin":true}}"#,
            server.host(),
            server.port(),
            server.fingerprint()
        ),
    )
    .unwrap();
    core.import_profile(&path).unwrap();
    Session { _dir: dir, core }
}

/// alice, Administradora con clave avanzada y sudo ya desbloqueado.
async fn admin(server: &TestServer) -> Session {
    let s = connect(server, "advanced", ADVANCED).await;
    s.core.unlock_admin("alice-sudo").await.unwrap();
    s
}

fn stat(server: &TestServer, user: &str) -> String {
    server.sh(&format!("stat -c '%a %U:%G' /etc/ssh/authorized_keys/{user}")).trim().to_string()
}

fn lines(keys: &UserKeys) -> Vec<&str> {
    keys.lines.as_ref().unwrap().iter().map(String::as_str).collect()
}

#[tokio::test]
async fn lists_each_users_keys_without_blank_lines_or_comments() {
    let server = TestServer::start();
    let s = connect(&server, "advanced", "restrict,pty").await;
    server.sh(
        "printf '%s\\n' '# Equipo de Bob' '' 'restrict,command=\"internal-sftp\" ssh-ed25519 AAAAC3Nz bob-casa' '  ssh-ed25519 AAAAC3Nx  ' \
         > /etc/ssh/authorized_keys/bob && chmod 644 /etc/ssh/authorized_keys/bob",
    );

    let keys = s.core.authorized_keys().await.unwrap();

    let users: Vec<_> = keys.iter().map(|k| k.user.as_str()).collect();
    assert_eq!(users, ["alice", "bob"]);
    assert_eq!(lines(&keys[0]).len(), 1);
    assert!(lines(&keys[0])[0].starts_with("restrict,pty ssh-ed25519 ") && lines(&keys[0])[0].ends_with(" alice-portatil"));
    assert_eq!(lines(&keys[1]), ["restrict,command=\"internal-sftp\" ssh-ed25519 AAAAC3Nz bob-casa", "ssh-ed25519 AAAAC3Nx"]);
}

#[tokio::test]
async fn works_with_a_simple_mode_key_and_skips_what_is_not_a_file() {
    let server = TestServer::start();
    let s = connect(&server, "simple", "restrict,command=\"internal-sftp\"").await;
    server.sh("mkdir /etc/ssh/authorized_keys/carpeta && touch /etc/ssh/authorized_keys/bob && chmod 644 /etc/ssh/authorized_keys/bob");
    server.sh("printf 'ssh-ed25519 AAAA secreto' > /etc/ssh/authorized_keys/carol && chmod 600 /etc/ssh/authorized_keys/carol");

    let keys = s.core.authorized_keys().await.unwrap();

    assert_eq!(keys.iter().map(|k| k.user.as_str()).collect::<Vec<_>>(), ["alice", "bob", "carol"]);
    assert_eq!(keys[1].lines, Some(vec![]));
    assert_eq!(keys[2].lines, None);
}

#[tokio::test]
async fn explains_when_the_server_does_not_keep_keys_there() {
    let server = TestServer::start();
    let s = connect(&server, "advanced", "restrict,pty").await;
    // La clave de alice ya está aceptada en la conexión abierta.
    server.sh("chmod 700 /etc/ssh/authorized_keys");

    assert_eq!(s.core.authorized_keys().await.unwrap_err(), Error::KeysUnavailable);
}

#[tokio::test]
async fn authorizing_a_key_lets_the_server_accept_it_in_that_mode() {
    let server = TestServer::start();
    let s = admin(&server).await;
    // bob se ha puesto Modo avanzado en el Perfil: lo que manda es su línea.
    let bob = member(&server, "bob", "advanced");
    assert_eq!(bob.core.connect().await.unwrap_err(), Error::Unauthorized);

    let line = format!("{SIMPLE} {} bob-portatil", bob.core.public_key().unwrap());
    s.core.authorize_key("bob", &line).await.unwrap();

    bob.core.connect().await.unwrap();
    assert_eq!(bob.core.open_terminal(1, 80, 24, |_| {}).await.unwrap_err(), Error::TerminalFailed);
    assert_eq!(stat(&server, "bob"), "644 root:root");
    assert_eq!(server.sh("cat /etc/ssh/authorized_keys/bob"), line);

    // Autorizarla otra vez no la repite.
    s.core.authorize_key("bob", &line).await.unwrap();
    assert_eq!(server.sh("cat /etc/ssh/authorized_keys/bob"), line);
}

#[tokio::test]
async fn changing_the_mode_changes_what_the_server_allows() {
    let server = TestServer::start();
    let s = admin(&server).await;
    let bob = member(&server, "bob", "advanced");
    let key = bob.core.public_key().unwrap();
    server.authorize_with("bob", SIMPLE, &format!("{key} bob-portatil"));
    bob.core.connect().await.unwrap();
    assert_eq!(bob.core.open_terminal(1, 80, 24, |_| {}).await.unwrap_err(), Error::TerminalFailed);

    s.core.replace_key("bob", &format!("{SIMPLE} {key} bob-portatil"), &format!("{ADVANCED} {key} bob-portatil")).await.unwrap();

    bob.core.connect().await.unwrap();
    bob.core.open_terminal(1, 80, 24, |_| {}).await.unwrap();
    assert_eq!(stat(&server, "bob"), "644 root:root");
}

#[tokio::test]
async fn revoking_a_key_makes_the_server_reject_it_and_keeps_the_rest() {
    let server = TestServer::start();
    let s = admin(&server).await;
    let bob = member(&server, "bob", "simple");
    let line = format!("{SIMPLE} {} bob-portatil", bob.core.public_key().unwrap());
    server.sh(&format!(
        "printf '%s\\n' '# Equipos de Bob' '{line}' '' '{ADVANCED} ssh-ed25519 AAAAC3Nz bob-casa' > /etc/ssh/authorized_keys/bob"
    ));
    bob.core.connect().await.unwrap();

    s.core.revoke_key("bob", &line).await.unwrap();

    assert_eq!(bob.core.connect().await.unwrap_err(), Error::Unauthorized);
    assert_eq!(server.sh("cat /etc/ssh/authorized_keys/bob"), format!("# Equipos de Bob\n\n{ADVANCED} ssh-ed25519 AAAAC3Nz bob-casa"));
    assert_eq!(stat(&server, "bob"), "644 root:root");
    assert_eq!(s.core.revoke_key("bob", &line).await.unwrap_err(), Error::KeyChanged);
    assert_eq!(s.core.replace_key("bob", &line, "otra").await.unwrap_err(), Error::KeyChanged);
}

#[tokio::test]
async fn the_sudo_password_is_needed_once_and_forgotten_with_the_profile() {
    let server = TestServer::start();
    let s = connect(&server, "advanced", ADVANCED).await;
    let line = format!("{SIMPLE} ssh-ed25519 AAAAC3Nz bob-casa");
    assert_eq!(s.core.authorize_key("bob", &line).await.unwrap_err(), Error::SudoPasswordNeeded);

    assert_eq!(s.core.unlock_admin("no-es-esta").await.unwrap_err(), Error::SudoWrongPassword);
    s.core.unlock_admin("alice-sudo").await.unwrap();
    s.core.authorize_key("bob", &line).await.unwrap();
    s.core.connect().await.unwrap();
    s.core.revoke_key("bob", &line).await.unwrap();

    let profile = s._dir.path().join("perfil.sshpanel");
    s.core.import_profile(&profile).unwrap();
    s.core.connect().await.unwrap();
    assert_eq!(s.core.authorize_key("bob", &line).await.unwrap_err(), Error::SudoPasswordNeeded);
}

#[tokio::test]
async fn explains_a_user_without_sudo_and_a_simple_admin_key() {
    let server = TestServer::start();
    let bob = member(&server, "bob", "advanced");
    server.authorize_with("bob", ADVANCED, &bob.core.public_key().unwrap());
    bob.core.connect().await.unwrap();
    assert_eq!(bob.core.unlock_admin("bob-sudo").await.unwrap_err(), Error::NoSudo);

    // El Perfil dice Modo avanzado, pero la clave es de Modo simple.
    let s = connect(&server, "advanced", SIMPLE).await;
    assert_eq!(s.core.unlock_admin("alice-sudo").await.unwrap_err(), Error::AdminNeedsAdvanced);
    let simple = connect(&server, "simple", SIMPLE).await;
    assert_eq!(simple.core.unlock_admin("alice-sudo").await.unwrap_err(), Error::AdminNeedsAdvanced);
}

#[tokio::test]
async fn lists_the_servers_people_and_rejects_odd_user_names() {
    let server = TestServer::start();
    let s = admin(&server).await;

    assert_eq!(s.core.server_users().await.unwrap(), ["alice", "bob"]);
    assert_eq!(s.core.authorize_key("../shadow", "ssh-ed25519 AAAA").await.unwrap_err(), Error::InvalidUser);
}
