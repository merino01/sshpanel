# 08: Papelera

**What to build:** Al borrar, la app pide confirmación y mueve el archivo a la Papelera del Servidor; el Miembro puede restaurarlo.

**Blocked by:** 05-explorador

**Status:** done

- [x] «Eliminar» disponible en el menú contextual de cada fila (ticket 14)
- [x] Borrar siempre pide confirmación
- [x] El archivo borrado va a la Papelera y no se elimina
- [x] Restaurar un archivo desde la Papelera lo devuelve a su sitio
- [x] Conflictos de nombre al restaurar tratados con un mensaje claro

## Comments

- 2026-10-14: Decidido con el usuario: Papelera en formato freedesktop en la carpeta personal (`~/.local/share/Trash`), una por Miembro; lo borrado de la Carpeta compartida va a la Papelera de quien lo borra. Al restaurar sobre un nombre ocupado queda como `nombre (1)` y se avisa; si la carpeta original ya no existe, se vuelve a crear. Acceso directo fijo «Papelera». Ver ADR 0004.
- 2026-10-14: Núcleo: `Core::trash`, `Core::list_trash` y `Core::restore` (tipos `Trashed` y `Restored`). Nunca se borra la Papelera ni lo que la contiene. Errores nuevos: `files.trash_failed`, `files.trash_protected`, `files.restore_failed`. Probado contra el banco de pruebas (`tests/trash.rs`): archivos y carpetas, restaurar, conflicto de nombre, carpeta desaparecida, dos con el mismo nombre, nombres con espacios, acentos y `%`, Carpeta compartida, permisos denegados y una Papelera escrita por otra herramienta.
- 2026-10-14: Interfaz: «Eliminar» en el menú contextual y con la tecla Supr, con confirmación en un diálogo. La vista Papelera muestra de dónde vino cada elemento y cuándo se borró, y tiene «Restaurar» en el menú contextual. Pendiente: probarlo a mano en la app.
