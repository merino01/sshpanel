//! Rutas del Servidor: siempre con `/`, sea cual sea el sistema del Miembro.

/// Ruta de `name` dentro de la carpeta `dir`.
pub(super) fn join(dir: &str, name: &str) -> String {
    format!("{}/{name}", dir.trim_end_matches('/'))
}

/// Si `name` es un único nombre, sin `/` y distinto de `.` y `..`.
pub(super) fn is_single_name(name: &str) -> bool {
    !name.is_empty() && name != "." && name != ".." && !name.contains(['/', '\0'])
}

/// Nombre que escribe el Miembro, sin espacios alrededor.
pub(super) fn new_name(name: &str) -> crate::Result<&str> {
    let name = name.trim();
    if is_single_name(name) { Ok(name) } else { Err(crate::Error::InvalidName) }
}

/// Carpeta y nombre de una ruta absoluta.
pub(super) fn split_path(path: &str) -> Option<(&str, &str)> {
    let slash = path.rfind('/')?;
    let (dir, name) = (if slash == 0 { "/" } else { &path[..slash] }, &path[slash + 1..]);
    (path.starts_with('/') && is_single_name(name)).then_some((dir, name))
}

/// Si una ruta es la otra o está dentro de ella.
pub(super) fn overlaps(a: &str, b: &str) -> bool {
    let (a, b) = (a.trim_end_matches('/'), b.trim_end_matches('/'));
    let inside = |inner: &str, outer: &str| inner.strip_prefix(outer).is_some_and(|rest| rest.is_empty() || rest.starts_with('/'));
    inside(a, b) || inside(b, a) || a.is_empty() || b.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_absolute_paths_only() {
        assert_eq!(split_path("/home/alice/notas"), Some(("/home/alice", "notas")));
        assert_eq!(split_path("/notas"), Some(("/", "notas")));
        assert_eq!(split_path("notas"), None);
        assert_eq!(split_path("/home/"), None);
        assert_eq!(split_path("/home/.."), None);
    }

    #[test]
    fn detects_paths_inside_one_another() {
        let trash = "/home/alice/.local/share/Trash";
        assert!(overlaps("/home/alice/.local", trash));
        assert!(overlaps("/home/alice/.local/share/Trash/files/x", trash));
        assert!(overlaps("/", trash));
        assert!(!overlaps("/home/alice/.locale", trash));
        assert!(!overlaps("/home/alice/notas", trash));
    }
}
