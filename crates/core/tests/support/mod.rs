//! Banco de pruebas: un servidor OpenSSH efímero en contenedor Docker.
//!
//! Usuarios disponibles: `alice` y `bob`, sin contraseña; solo entran con una
//! clave autorizada mediante [`TestServer::authorize`].

#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::Once;
use std::time::{Duration, Instant};

const IMAGE: &str = "sshpanel-test-sshd";

static BUILD: Once = Once::new();

fn docker(args: &[&str]) -> Output {
    Command::new("docker")
        .args(args)
        .output()
        .expect("no se pudo ejecutar docker; el banco de pruebas lo necesita")
}

fn docker_ok(args: &[&str]) -> String {
    let out = docker(args);
    assert!(
        out.status.success(),
        "docker {args:?} falló: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

fn build_image() {
    BUILD.call_once(|| {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/support");
        docker_ok(&["build", "-q", "-t", IMAGE, dir.to_str().unwrap()]);
    });
}

/// Par de claves de prueba generado con `ssh-keygen`, en un directorio temporal.
pub struct TestKey {
    dir: PathBuf,
}

impl TestKey {
    pub fn generate() -> Self {
        let dir = std::env::temp_dir().join(format!("sshpanel-key-{}", unique()));
        fs::create_dir_all(&dir).unwrap();
        let status = Command::new("ssh-keygen")
            .args(["-q", "-t", "ed25519", "-N", "", "-f"])
            .arg(dir.join("id"))
            .status()
            .expect("ssh-keygen no está disponible");
        assert!(status.success());
        Self { dir }
    }

    pub fn private_path(&self) -> PathBuf {
        self.dir.join("id")
    }

    pub fn public(&self) -> String {
        fs::read_to_string(self.dir.join("id.pub")).unwrap().trim().to_string()
    }
}

impl Drop for TestKey {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.dir);
    }
}

/// Servidor OpenSSH de prueba; el contenedor se elimina al soltarlo.
pub struct TestServer {
    container: String,
    port: u16,
}

impl TestServer {
    pub fn start() -> Self {
        build_image();
        let container = format!("sshpanel-test-{}", unique());
        docker_ok(&[
            "run", "-d", "--rm", "--name", &container, "-p", "127.0.0.1::22", IMAGE,
        ]);
        let mapping = docker_ok(&["port", &container, "22/tcp"]);
        let port = mapping.rsplit(':').next().unwrap().parse().unwrap();
        let server = Self { container, port };
        server.wait_ready();
        server
    }

    pub fn host(&self) -> &str {
        "127.0.0.1"
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    /// Huella SHA256 de la clave de host, en el formato de `ssh-keygen -l`.
    pub fn fingerprint(&self) -> String {
        let line = self.exec_ok(&["ssh-keygen", "-lf", "/etc/ssh/ssh_host_ed25519_key.pub"]);
        line.split_whitespace().nth(1).expect("huella").to_string()
    }

    /// Autoriza una clave pública para `user`.
    pub fn authorize(&self, user: &str, public_key: &str) {
        let script = format!(
            "echo '{public_key}' >> /home/{user}/.ssh/authorized_keys \
             && chown {user}:{user} /home/{user}/.ssh/authorized_keys \
             && chmod 600 /home/{user}/.ssh/authorized_keys"
        );
        self.exec_ok(&["sh", "-c", &script]);
    }

    /// Revoca todas las claves de `user`.
    pub fn revoke_all(&self, user: &str) {
        let script = format!("rm -f /home/{user}/.ssh/authorized_keys");
        self.exec_ok(&["sh", "-c", &script]);
    }

    /// Ejecuta `sftp` por lotes contra el servidor, con la clave dada.
    pub fn sftp(&self, user: &str, key: &TestKey, batch: &str) -> Output {
        let batch_file = key.dir.join("batch");
        fs::write(&batch_file, batch).unwrap();
        Command::new("sftp")
            .args(["-q", "-b"])
            .arg(&batch_file)
            .args(["-i"])
            .arg(key.private_path())
            .args([
                "-o", "BatchMode=yes",
                "-o", "IdentitiesOnly=yes",
                "-o", "StrictHostKeyChecking=no",
                "-o", "UserKnownHostsFile=/dev/null",
                "-P", &self.port.to_string(),
            ])
            .arg(format!("{user}@{}", self.host()))
            .output()
            .expect("sftp no está disponible")
    }

    fn exec_ok(&self, cmd: &[&str]) -> String {
        let mut args = vec!["exec", self.container.as_str()];
        args.extend_from_slice(cmd);
        docker_ok(&args)
    }

    fn wait_ready(&self) {
        let deadline = Instant::now() + Duration::from_secs(20);
        while Instant::now() < deadline {
            if std::net::TcpStream::connect((self.host(), self.port)).is_ok()
                && docker(&["exec", &self.container, "test", "-f", "/etc/ssh/ssh_host_ed25519_key.pub"])
                    .status
                    .success()
            {
                return;
            }
            std::thread::sleep(Duration::from_millis(200));
        }
        panic!("el servidor de prueba no arrancó a tiempo");
    }
}

impl Drop for TestServer {
    fn drop(&mut self) {
        let _ = docker(&["rm", "-f", &self.container]);
    }
}

fn unique() -> String {
    use std::sync::atomic::{AtomicU32, Ordering};
    static N: AtomicU32 = AtomicU32::new(0);
    format!(
        "{}-{}-{}",
        std::process::id(),
        N.fetch_add(1, Ordering::Relaxed),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .subsec_nanos()
    )
}
