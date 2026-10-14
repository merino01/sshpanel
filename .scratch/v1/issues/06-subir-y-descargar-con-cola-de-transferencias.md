# 06: Subir y descargar con cola de Transferencias

**What to build:** El Miembro sube archivos arrastrándolos a la ventana o con el selector del sistema, descarga archivos y carpetas, y ve la cola de Transferencias con barra de progreso.

**Blocked by:** 05-explorador

**Status:** done

- [x] Subida por arrastrar y soltar y por selector
- [x] Descarga de archivos y carpetas
- [x] Cola visible con progreso por Transferencia
- [x] Un fallo de red muestra un mensaje claro

## Comments

- 2026-10-14: Núcleo con `upload` y `download` (archivos y carpetas enteras, con progreso por callback). Cada archivo se copia a un provisional oculto `.nombre.part` y solo toma su nombre al completarse; nunca se sustituye nada: si el nombre está ocupado queda como `nombre (1)`. Dentro de carpetas se omiten los enlaces a carpetas (evita bucles). La conexión pasa a `Arc` para que una Transferencia no bloquee el Explorador. Un corte o un Servidor que no responde (10 s) da `connection.lost`. Errores nuevos: `transfer.local_unreadable`, `transfer.local_write_failed`, `transfer.failed`. Probado contra el banco de pruebas (`tests/transfers.rs`), incluido el corte a mitad de subida y de descarga.
- 2026-10-14: Interfaz: botón «Subir archivos», arrastrar y soltar (también carpetas) sobre la carpeta en pantalla, «Descargar» en el menú contextual (elige carpeta de destino) y panel de Transferencias con barra. La cola va de una en una; el paralelo, cancelar y reintentar quedan para el ticket 07.
- 2026-10-14: Probado a mano: subida con selector y arrastrando, descarga y cola con progreso funcionan.
