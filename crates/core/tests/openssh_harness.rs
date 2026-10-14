//! Prueba de humo del banco de pruebas OpenSSH.

mod support;
use support::{TestKey, TestServer};

#[test]
fn connects_over_sftp_with_an_authorized_key() {
    let server = TestServer::start();
    let key = TestKey::generate();
    server.authorize("alice", &key.public());

    let out = server.sftp("alice", &key, "pwd\nls\n");

    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    assert!(String::from_utf8_lossy(&out.stdout).contains("/home/alice"));
}

#[test]
fn rejects_a_key_that_was_never_authorized() {
    let server = TestServer::start();
    let key = TestKey::generate();

    let out = server.sftp("alice", &key, "pwd\n");

    assert!(!out.status.success());
}

#[test]
fn revoking_keys_cuts_access() {
    let server = TestServer::start();
    let key = TestKey::generate();
    server.authorize("bob", &key.public());
    assert!(server.sftp("bob", &key, "pwd\n").status.success());

    server.revoke_all("bob");

    assert!(!server.sftp("bob", &key, "pwd\n").status.success());
}

#[test]
fn exposes_the_host_fingerprint() {
    let server = TestServer::start();

    assert!(server.fingerprint().starts_with("SHA256:"));
}
