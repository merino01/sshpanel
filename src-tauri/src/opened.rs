//! Perfil abierto con doble clic. En Windows y Linux llega como argumento, a
//! esta instancia o, si la app ya estaba abierta, a la primera (que lo recibe
//! del plugin de instancia única); en macOS, como evento del sistema. Se guarda
//! aquí hasta que la interfaz lo recoge con `take_opened_profile`, y se le avisa
//! con el evento `profile-opened` por si ya estaba en marcha.

use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use tauri::{AppHandle, Emitter, Manager};

/// Evento con el que se avisa a la interfaz de que hay un Perfil por abrir.
const EVENT: &str = "profile-opened";

#[derive(Default)]
pub struct Opened(Mutex<Option<PathBuf>>);

impl Opened {
    pub fn take(&self) -> Option<PathBuf> {
        self.0.lock().unwrap().take()
    }
}

/// Extensión de los ficheros de Perfil, tal como la declara el Branding.
pub fn extension(app: &AppHandle) -> String {
    app.config()
        .bundle
        .file_associations
        .as_ref()
        .and_then(|associations| associations.first())
        .and_then(|association| association.ext.first())
        .map(|ext| ext.to_string())
        .unwrap_or_default()
}

/// El primer argumento, sin contar el programa, que es un fichero con la
/// extensión `ext`. Las rutas relativas se resuelven desde `cwd`.
pub fn profile_arg<S: AsRef<OsStr>>(args: impl IntoIterator<Item = S>, ext: &str, cwd: &Path) -> Option<PathBuf> {
    if ext.is_empty() {
        return None;
    }
    args.into_iter().skip(1).map(|arg| PathBuf::from(arg.as_ref())).find(|path| {
        path.extension().and_then(OsStr::to_str).is_some_and(|e| e.eq_ignore_ascii_case(ext))
    }).map(|path| cwd.join(path))
}

/// Deja `path` pendiente de abrir y avisa a la interfaz.
pub fn deliver(app: &AppHandle, path: PathBuf) {
    *app.state::<Opened>().0.lock().unwrap() = Some(path);
    let _ = app.emit(EVENT, ());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_the_profile_among_the_arguments() {
        let cwd = Path::new("/home/alice/Descargas");
        let args = |list: &[&str]| profile_arg(list.iter().copied(), "sshpanel", cwd);

        assert_eq!(args(&["/usr/bin/app", "/tmp/alice.sshpanel"]), Some(PathBuf::from("/tmp/alice.sshpanel")));
        assert_eq!(args(&["app", "--flag", "alice.SSHPANEL"]), Some(PathBuf::from("/home/alice/Descargas/alice.SSHPANEL")));
        assert_eq!(args(&["app", "notas.txt", "b.sshpanel"]), Some(PathBuf::from("/home/alice/Descargas/b.sshpanel")));
    }

    #[test]
    fn ignores_the_program_and_other_files() {
        let cwd = Path::new("/");

        assert_eq!(profile_arg(["/opt/app.sshpanel"], "sshpanel", cwd), None);
        assert_eq!(profile_arg(["app", "notas.txt", "sshpanel"], "sshpanel", cwd), None);
        assert_eq!(profile_arg(["app", "a.sshpanel"], "", cwd), None);
    }
}
