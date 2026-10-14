# 14: Menú contextual en el Explorador

**What to build:** Clic derecho (o tecla Menú / Shift+F10) sobre un archivo, una carpeta o el hueco de la carpeta abre un menú con las acciones disponibles, en lugar de enlaces sueltos en cada fila.

**Blocked by:** 05-explorador

**Status:** done

- [x] Menú de fila: Abrir (solo carpetas), Renombrar y Nueva carpeta
- [x] Menú del hueco de la carpeta: Nueva carpeta
- [x] Se maneja con el teclado: filas enfocables, flechas para moverse, Escape para cerrar
- [x] El menú nativo de la webview no aparece, salvo en los campos de texto (para poder pegar)
- [x] El botón «Nueva carpeta» de la barra se mantiene

## Comments

- 2026-10-14: «Eliminar» se añade a este menú en el ticket 08, con confirmación y Papelera.
- 2026-10-14: Probado a mano: menú con ratón y teclado, Renombrar, Nueva carpeta y Abrir funcionan.
- 2026-10-14: Cambio decidido con el usuario: el botón «Nueva carpeta» sale de la barra (antes se mantenía). El hueco de la carpeta ocupa el resto de la ventana, se enfoca con Tab y su menú tiene «Nueva carpeta» y «Subir archivos». Las filas muestran la mano y subrayan el nombre al pasar por encima, y un clic en cualquier parte de una carpeta la abre.
