# 09: Modo avanzado

**What to build:** Un Perfil en Modo avanzado añade la Terminal embebida y permite vaciar la Papelera; el Modo simple no muestra ninguna de las dos.

**Blocked by:** 08-papelera

**Status:** done

- [x] Terminal con xterm.js que ejecuta comandos en el Servidor y muestra sus respuestas
- [x] Vaciar la Papelera solo disponible en Modo avanzado
- [x] «Eliminar definitivamente» un elemento de la Papelera, solo en Modo avanzado
- [x] En Modo simple no hay Terminal ni opción de vaciar la Papelera
- [x] Probado a través de la interfaz del núcleo Rust

## Comments

- 2026-10-14: Decidido con el usuario: la Terminal va en una pantalla propia, con acceso fijo «Terminal» en la barra lateral bajo «Papelera»; la sesión sigue viva al ir al Explorador o la Papelera y volver. En la Papelera, además de «Vaciar Papelera», «Eliminar definitivamente» en el menú contextual de cada elemento. Las dos piden confirmación.
- 2026-10-14: Núcleo: `Core::open_terminal` (PTY `xterm-256color` sobre la misma conexión SSH; la salida y el cierre llegan como `TerminalEvent`), `terminal_input`, `resize_terminal`, `close_terminal`; una sola Terminal a la vez, y conectar de nuevo cierra la anterior. `Core::delete_forever` y `Core::empty_trash`: vaciar borra todo lo de `files/` (también lo que no tiene `.trashinfo`) y los `.trashinfo`, sin seguir enlaces simbólicos y solo dentro de la Papelera. El núcleo lo rechaza en Modo simple. Errores nuevos: `mode.advanced_only`, `files.delete_failed`, `terminal.failed`. Probado contra el banco de pruebas (`tests/terminal.rs`, `tests/trash.rs`, `tests/simple_mode.rs`).
- 2026-10-14: Interfaz: la salida de la Terminal cruza como bytes crudos (`tauri::ipc::Response`), sin JSON. Al terminar la sesión se ofrece «Abrir otra». Pendiente: probarlo a mano en la app.
