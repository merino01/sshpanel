//! Administración: leer las Claves públicas autorizadas de cada usuario de
//! `/etc/ssh/authorized_keys/`, a través de la interfaz del núcleo y contra el
//! servidor OpenSSH de prueba.

mod support;

use std::fs;

use sshpanel_core::{Core, Error, MemorySecretStore, UserKeys};
use support::TestServer;
use tempfile::TempDir;

struct Session {
    _dir: TempDir,
    core: Core,
}

/// `alice` conectada en Modo `mode`, con su clave autorizada con `options`.
async fn connect(server: &TestServer, mode: &str, options: &str) -> Session {
    let dir = TempDir::new().unwrap();
    let core = Core::new(dir.path().join("data"), Box::new(MemorySecretStore::default()));
    let path = dir.path().join("perfil.sshpanel");
    fs::write(
        &path,
        format!(
            r#"{{"version":1,"host":"{}","port":{},"username":"alice","fingerprint":"{}","mode":"{mode}","admin":true}}"#,
            server.host(),
            server.port(),
            server.fingerprint()
        ),
    )
    .unwrap();
    core.import_profile(&path).unwrap();
    server.authorize_with("alice", options, &format!("{} alice-portatil", core.public_key().unwrap()));
    core.connect().await.unwrap();
    Session { _dir: dir, core }
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
