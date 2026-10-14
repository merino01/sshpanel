//! Importación de Perfil e identidad del Miembro, a través de la interfaz del núcleo.

mod support;

use std::fs;
use std::path::Path;
use std::sync::Arc;

use sshpanel_core::{Core, Error, MemorySecretStore, Mode, SecretStore};
use support::{TestKey, TestServer};
use tempfile::TempDir;

const FINGERPRINT: &str = "SHA256:nThbg6kXUpJWGl7E1IGOCspRomTxdCARLviKw6E5SY8";

fn profile_json(extra: &str) -> String {
    format!(
        r#"{{"version":1,"host":"servidor.ejemplo.com","username":"usuario1","fingerprint":"{FINGERPRINT}","mode":"simple"{extra}}}"#
    )
}

struct Fixture {
    dir: TempDir,
    secrets: Arc<MemorySecretStore>,
    core: Core,
}

impl Fixture {
    fn new() -> Self {
        let dir = TempDir::new().unwrap();
        let secrets = Arc::new(MemorySecretStore::default());
        let core = Core::new(dir.path().join("data"), Box::new(secrets.clone()));
        Self { dir, secrets, core }
    }

    fn profile_file(&self, content: &str) -> std::path::PathBuf {
        let path = self.dir.path().join("perfil.dbdesp");
        fs::write(&path, content).unwrap();
        path
    }
}

#[test]
fn importing_a_valid_profile_makes_it_the_current_one() {
    let fx = Fixture::new();
    let file = fx.profile_file(&profile_json(r#","port":2222,"shared_folder":"/srv/compartida""#));

    let profile = fx.core.import_profile(&file).unwrap();

    assert_eq!(profile.host, "servidor.ejemplo.com");
    assert_eq!(profile.port, 2222);
    assert_eq!(profile.username, "usuario1");
    assert_eq!(profile.mode, Mode::Simple);
    assert_eq!(profile.shared_folder.as_deref(), Some("/srv/compartida"));
    assert_eq!(fx.core.current_profile().unwrap(), Some(profile));
}

#[test]
fn port_defaults_to_22_and_shared_folder_and_admin_are_optional() {
    let fx = Fixture::new();
    let file = fx.profile_file(&profile_json(""));

    let profile = fx.core.import_profile(&file).unwrap();

    assert_eq!(profile.port, 22);
    assert_eq!(profile.shared_folder, None);
    assert!(!profile.admin);
}

#[test]
fn there_is_no_current_profile_before_importing() {
    assert_eq!(Fixture::new().core.current_profile().unwrap(), None);
}

#[test]
fn invalid_profiles_are_rejected_and_nothing_is_saved() {
    let cases: Vec<(&str, String, Error)> = vec![
        ("no es JSON", "esto no es un perfil".into(), Error::ProfileMalformed),
        ("falta un campo", r#"{"version":1,"host":"a.com"}"#.into(), Error::ProfileMalformed),
        ("versión futura", profile_json("").replace(r#""version":1"#, r#""version":9"#), Error::ProfileUnsupportedVersion),
        ("host vacío", profile_json("").replace("servidor.ejemplo.com", " "), Error::ProfileInvalidField("host")),
        ("usuario con mayúsculas", profile_json("").replace("usuario1", "Root"), Error::ProfileInvalidField("username")),
        ("huella inválida", profile_json("").replace(FINGERPRINT, "SHA256:corta"), Error::ProfileInvalidField("fingerprint")),
        ("puerto cero", profile_json(r#","port":0"#), Error::ProfileInvalidField("port")),
        ("carpeta relativa", profile_json(r#","shared_folder":"compartida""#), Error::ProfileInvalidField("shared_folder")),
    ];
    for (name, content, expected) in cases {
        let fx = Fixture::new();
        let file = fx.profile_file(&content);

        let err = fx.core.import_profile(&file).unwrap_err();

        assert_eq!(err, expected, "caso: {name}");
        assert_eq!(fx.core.current_profile().unwrap(), None, "caso: {name}");
        assert_eq!(fx.secrets.get("identity").unwrap(), None, "caso: {name}: no debe crear identidad");
    }
}

#[test]
fn a_missing_file_is_reported_as_unreadable() {
    let fx = Fixture::new();

    let err = fx.core.import_profile(Path::new("/no/existe.dbdesp")).unwrap_err();

    assert_eq!(err, Error::ProfileUnreadable);
}

#[test]
fn importing_generates_an_identity_kept_in_the_secret_store() {
    let fx = Fixture::new();
    fx.core.import_profile(&fx.profile_file(&profile_json(""))).unwrap();

    let stored = fx.secrets.get("identity").unwrap().expect("la clave privada se guarda en el almacén");
    assert!(stored.contains("PRIVATE KEY"));
    assert!(fx.core.public_key().unwrap().starts_with("ssh-ed25519 "));
}

#[test]
fn the_private_key_never_leaves_the_secret_store() {
    let fx = Fixture::new();
    fx.core.import_profile(&fx.profile_file(&profile_json(""))).unwrap();

    let public = fx.core.public_key().unwrap();
    let saved_profile = fs::read_to_string(fx.dir.path().join("data/profile.json")).unwrap();

    assert!(!public.contains("PRIVATE"));
    assert!(!saved_profile.contains("PRIVATE"));
}

#[test]
fn the_identity_is_stable_across_imports() {
    let fx = Fixture::new();
    let file = fx.profile_file(&profile_json(""));
    fx.core.import_profile(&file).unwrap();
    let first = fx.core.public_key().unwrap();

    fx.core.import_profile(&file).unwrap();

    assert_eq!(fx.core.public_key().unwrap(), first);
}

#[test]
fn the_generated_public_key_opens_a_session_once_authorized() {
    let fx = Fixture::new();
    fx.core.import_profile(&fx.profile_file(&profile_json(""))).unwrap();
    let private = fx.secrets.get("identity").unwrap().unwrap();
    let server = TestServer::start();

    let key = TestKey::from_private(&private);
    assert!(!server.sftp("alice", &key, "pwd\n").status.success(), "antes de autorizar no entra");

    server.authorize("alice", &fx.core.public_key().unwrap());
    let out = server.sftp("alice", &key, "pwd\n");

    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
}
