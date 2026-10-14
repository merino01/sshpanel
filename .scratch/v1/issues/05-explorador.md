# 05: Explorador

**What to build:** El Miembro navega por las carpetas del Servidor, crea carpetas, renombra y accede a la Carpeta compartida cuando el Perfil la define.

**Blocked by:** 04-conectar-con-huella-y-listar-carpeta-personal

**Status:** done

- [x] Navegar entrando y subiendo de carpeta, empezando en la carpeta personal
- [x] Crear carpeta y renombrar archivos y carpetas
- [x] Acceso directo fijo a la Carpeta compartida solo si el Perfil la define
- [x] Los permisos denegados por el Servidor se muestran como un mensaje claro

## Comments

- 2026-10-14: Núcleo con `create_folder` y `rename` (sin mover de carpeta; nunca sustituyen algo existente, porque el Servidor reemplazaría sin avisar una carpeta vacía al renombrar). Los enlaces a carpetas se listan como carpetas para poder entrar. Errores nuevos: `files.permission_denied`, `files.not_found`, `files.already_exists`, `files.invalid_name`. Probado contra el banco de pruebas (`tests/explorer.rs`).
- 2026-10-14: Probado a mano contra un Servidor real: navegación, crear carpeta, renombrar, Carpeta compartida y permisos denegados funcionan.
