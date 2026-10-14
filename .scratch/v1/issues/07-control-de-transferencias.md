# 07: Control de Transferencias

**What to build:** El Miembro puede cancelar, reintentar, lanzar varias Transferencias a la vez y reanudar una interrumpida por un corte de red.

**Blocked by:** 06-subir-y-descargar-con-cola-de-transferencias

**Status:** done

- [x] Cancelar una Transferencia en curso
- [x] Reintentar una fallida
- [x] Varias Transferencias en paralelo
- [x] Una Transferencia cortada se reanuda donde se quedó, verificado contra el banco de pruebas
- [x] Si la reanudación resulta demasiado costosa, se documenta y se deja fuera con el acuerdo del usuario (no hace falta: entra)

## Comments

- 2026-10-14: Decidido con el usuario: cancelar deshace la Transferencia (borra lo que ha creado ella, nunca lo que ya estaba); lo cortado se reanuda solo al volver a conectar, y también hay «Reintentar»; 3 a la vez; la reanudación solo dura mientras la app está abierta (la cola vive en memoria).
- 2026-10-14: Núcleo: tipo `Transfer`, que recuerda su destino y lo que lleva hecho; `Core::run` la lanza o la reanuda y `Core::discard` la cancela y deshace. Al reanudar, los archivos ya completos no se repiten y los `.part` continúan desde su tamaño; tras un corte, el `.part` se conserva también en las descargas. Los destinos se reservan para que dos Transferencias en paralelo con el mismo nombre queden como `nombre` y `nombre (1)`. Error nuevo: `transfer.cancelled`. Probado contra el banco de pruebas: cancelar a mitad (archivo y carpeta), reanudar subida, descarga y carpeta cortadas (comparando sha256), descartar una cortada, reintentar tras un permiso denegado y varias en paralelo.
- 2026-10-14: Interfaz: botones «Cancelar» y «Reintentar» en cada fila; la cola lanza hasta 3 a la vez. Si se corta la conexión, las que iban quedan «cortadas» y siguen solas al reconectar. Límite conocido: cancelar una subida cortada sin conexión no puede borrar su provisional del Servidor, que queda oculto.
- 2026-10-14: La carpeta en pantalla se refresca también al cancelar o al fallar una subida, para que desaparezca el provisional. Probado a mano: cancelar, reintentar, paralelo y reanudar al reconectar funcionan.
