# Spec v2: SSH Panel

## Problem Statement

La v1 cubre lo básico: entrar con un Perfil, mover archivos, la Papelera y la Terminal. Al usarla a diario se notan cuatro carencias:

- **Los archivos no se pueden editar en el Servidor.** Para cambiar un fichero de configuración hay que descargarlo, editarlo y volver a subirlo, o usar vim en la Terminal, que un Miembro no técnico no sabe manejar.
- **La app no se puede adaptar.** No hay atajos de teclado configurables, ni temas, ni otro idioma que el español.
- **El Administrador sigue autorizando claves a mano.** La app le genera la línea de `authorized_keys`, pero tiene que copiarla y pegarla con `sudo nano` en el Servidor.
- **No hay un sitio fijo para las preferencias ni para la administración.** «Crear perfil» está suelto en la barra lateral.

## Solution

Una barra superior, presente en todas las pantallas, da acceso a dos apartados nuevos: Ajustes, en su propia ventana, y la Administración, solo para el Administrador.

- **Ajustes:** el Miembro elige Tema, Atajos e idioma (español o inglés), y todo se guarda en su equipo.
- **Abrir archivos:** el doble clic abre un archivo de texto en el Editor de la app, en su propia ventana, y el resto en la aplicación del equipo. La app vigila la copia local y sube cada cambio.
- **Abrir con:** el menú contextual permite elegir entre el Editor, la aplicación del equipo o, en Modo avanzado, vim en la Terminal.
- **Administración:** reúne «Crear perfil» y la gestión de Claves públicas. Lista las claves autorizadas de cada usuario, y autoriza, revoca o cambia de modo una clave escribiendo en el Servidor con sudo.

## User Stories

### Barra superior y Ajustes

1. Como Miembro, quiero ver arriba con qué usuario y Servidor estoy conectado, para saber siempre dónde estoy.
2. Como Miembro, quiero abrir Ajustes desde la barra superior o con Ctrl+, (Cmd+, en macOS), para cambiar mis preferencias sin buscar menús.
3. Como Miembro, quiero que Ajustes se abra en su propia ventana y que, si ya está abierta, se ponga delante, para no tener ventanas repetidas.
4. Como Miembro, quiero que los cambios de Ajustes se apliquen al momento en la ventana principal, para ver el efecto sin reiniciar.
5. Como Miembro, quiero que mis preferencias se mantengan al cerrar la app y al importar otro Perfil, porque son mías y de mi equipo, no del Perfil.
6. Como Miembro, quiero poder abrir Ajustes también desde la pantalla de bienvenida, antes de conectar, para elegir idioma o Tema desde el principio.

### Temas

7. Como Miembro, quiero elegir entre los Temas Sistema, Claro, Oscuro y Alto contraste, para trabajar cómodo con cualquier luz.
8. Como Miembro, quiero que el Tema Sistema siga el modo claro u oscuro de mi equipo, para no tener que cambiarlo a mano.
9. Como Miembro, quiero que la Terminal use los colores del Tema, para que la app se vea uniforme.
10. Como persona que clona el proyecto, quiero que el color de acento de mi Branding se mantenga en los Temas básicos, para que la app siga siendo reconocible.
11. Como persona que mantiene el proyecto, quiero añadir un Tema creando un solo fichero, para ampliar el catálogo sin tocar el resto del código.

### Atajos

12. Como Miembro, quiero atajos de teclado para las acciones del Explorador, para trabajar más rápido sin el ratón.
13. Como Miembro, quiero cambiar cualquier Atajo en Ajustes, para adaptarlos a mi costumbre.
14. Como Miembro, quiero que Ajustes me avise si un Atajo ya lo usa otra acción y no lo guarde hasta que lo resuelva, para que ninguna acción se quede sin Atajo por error.
15. Como Miembro, quiero un botón «Restablecer», para volver a los Atajos por defecto.
16. Como Miembro avanzado, quiero que los Atajos de la app no funcionen dentro de la Terminal, para que la consola reciba todas mis teclas.
17. Como Miembro de macOS, quiero que los Atajos usen Cmd donde en los demás sistemas se usa Ctrl, para que sigan las costumbres de mi sistema.

### Idioma

18. Como Miembro que no habla español, quiero la app en inglés, para entenderla.
19. Como Miembro, quiero que la primera vez la app use el idioma de mi sistema (y el inglés si no es ni español ni inglés), para no tener que configurarla.
20. Como Miembro, quiero cambiar el idioma en Ajustes, para usar el que prefiera.
21. Como Miembro, quiero que fechas y tamaños sigan el formato de mi idioma, para leerlos sin esfuerzo.
22. Como Administrador o Miembro que habla inglés, quiero las guías de instalación y del servidor en inglés, para preparar todo sin traducir.

### Editor

23. Como Miembro, quiero que el doble clic en un archivo de texto lo abra en el Editor, para cambiarlo sin descargarlo.
24. Como Miembro, quiero que cada archivo se abra en su propia ventana, para editar varios a la vez sin perder de vista el Explorador.
25. Como Miembro, quiero resaltado de sintaxis según la extensión, para leer configuraciones y scripts con claridad.
26. Como Miembro, quiero guardar con Ctrl+S, para trabajar como en cualquier editor.
27. Como Miembro, quiero que la app me pregunte antes de cerrar un archivo con cambios sin guardar, para no perder trabajo.
28. Como Miembro, quiero que, si el archivo cambió en el Servidor desde que lo abrí, la app me pregunte si sobrescribir o guardarlo como copia, para no pisar el trabajo de otro.
29. Como Miembro con mala conexión, quiero que si se cae la conexión mi texto se quede en la ventana y pueda guardar al reconectar, para no perder lo escrito.
30. Como Miembro, quiero que un archivo de más de 5 MB me avise antes de abrirse, para saber que puede tardar y ir lento.
31. Como Miembro en Modo simple, quiero que el Editor respete las mismas carpetas que el Explorador, para que todo funcione igual.

### Abrir con

32. Como Miembro, quiero que el doble clic en un archivo que no es de texto lo abra con la aplicación de mi equipo, para ver imágenes, documentos y PDF sin descargarlos a mano.
33. Como Miembro, quiero que, al guardar en la aplicación de mi equipo, el cambio se suba solo al Servidor, para no tener que acordarme de subirlo.
34. Como Miembro, quiero que esa subida salga en la cola de Transferencias, para saber cuándo ha llegado.
35. Como Miembro, quiero ver en el panel de Transferencias qué archivos tengo abiertos en el equipo y poder dejar de vigilar cada uno, para controlar qué se sube.
36. Como Miembro, quiero que las copias temporales se borren al desconectar o cerrar la app, para no dejar archivos del Servidor en mi equipo.
37. Como Miembro, quiero «Abrir con…» en el menú contextual, para elegir entre el Editor y la aplicación de mi equipo.
38. Como Miembro avanzado, quiero «Abrir con → vim», que abre el archivo en una pestaña nueva de la Terminal, para usar mi editor de siempre.

### Administración

39. Como Administrador, quiero un apartado «Administración» en la barra superior, solo visible con mi Perfil, para tener todo lo de administrar en un sitio.
40. Como Administrador, quiero que «Crear perfil» esté dentro de la Administración, para no tenerlo suelto en la barra lateral.
41. Como Administrador, quiero ver las Claves públicas autorizadas de cada usuario, con su nombre y su modo, para saber quién tiene acceso y cómo.
42. Como Administrador, quiero ver como «Personalizada» una clave cuyas opciones no son las de ningún modo, para saber que alguien la configuró a mano y que la app no la va a tocar.
43. Como Administrador, quiero autorizar la Clave pública que me envía un Miembro eligiendo usuario y modo, para darle acceso sin abrir una consola.
44. Como Administrador, quiero revocar una clave, para cortar el acceso de un equipo perdido o de alguien que se va.
45. Como Administrador, quiero cambiar el modo de una clave, para dar o quitar la Terminal a un Miembro.
46. Como Administrador, quiero que la app me pida la contraseña de sudo la primera vez que cambio algo en la sesión y no la guarde en ningún sitio, para no teclearla en cada cambio sin dejarla en disco.
47. Como Administrador, quiero una confirmación que me explique las consecuencias si voy a revocar o pasar a Modo simple la clave con la que estoy conectado, para no dejarme fuera sin querer.

## Implementation Decisions

- **Barra superior:** aparece en todas las pantallas, también en Bienvenida. A la izquierda muestra `usuario @ servidor` cuando hay Perfil. A la derecha, «Administración» (solo con la marca de administrador) y «Ajustes». La barra lateral sigue con los accesos directos, la Papelera y la Terminal, y pierde «Crear perfil». El diseño de la barra y de los apartados nuevos se hace con la skill `frontend-design`.
- **Nombres en el código:** el componente `Shortcuts.svelte` de la barra lateral pasa a llamarse `Places.svelte`, como el tipo `Place` de `lib/paths`. Así «Atajo» (de teclado) y «acceso directo» (de carpeta) no se confunden.
- **Ventanas:** Tauri 2 permite varias ventanas. Ajustes es una ventana única: abrirla de nuevo la pone delante. Cada archivo del Editor tiene su propia ventana. Las ventanas comparten el núcleo y la conexión de la principal. Los cambios de preferencias llegan a todas las ventanas mediante un evento.
- **Preferencias:** se guardan en un fichero de la carpeta de datos de la app, en el equipo del Miembro. Son independientes del Perfil y sobreviven a importar otro. Si falta un valor, se usa el de por defecto, para que añadir preferencias en el futuro no rompa ficheros antiguos.
- **Temas:**
  - Cada Tema es una paleta completa (fondo, texto, bordes, estados y los colores de la Terminal) definida en su propio fichero. La app descubre todos los ficheros de Tema al compilar, sin una lista que mantener a mano.
  - Temas de la v2: Sistema (por defecto, sigue a `prefers-color-scheme`), Claro, Oscuro y Alto contraste. Los cuatro usan el acento del Branding.
  - El formato ya permite que un Tema traiga su propio acento, para los Temas con nombre propio de más adelante.
- **Atajos:**

  | Acción | Atajo por defecto |
  |---|---|
  | Nueva carpeta | Ctrl+Shift+N |
  | Subir archivos | Ctrl+U |
  | Renombrar | F2 |
  | Mover a la Papelera | Supr |
  | Abrir / entrar | Intro |
  | Carpeta anterior | Alt+← o Retroceso |
  | Recargar | F5 |
  | Copiar ruta | Ctrl+Shift+C |
  | Ajustes | Ctrl+, |
  | Terminal (solo Modo avanzado) | Ctrl+` |

  - En macOS, Ctrl se cambia por Cmd.
  - Un Atajo repetido no se guarda, y se dice qué acción ya lo usa. «Restablecer» vuelve a los Atajos por defecto.
  - Con el foco en la Terminal, la app no aplica ningún Atajo. Los Atajos propios de las pestañas de la Terminal (`lib/terminalTabs`) no cambian.
  - Comprobar conflictos y convertir un Atajo a texto legible es lógica sin DOM en `lib/`, probada con Vitest.
- **Idioma:**
  - `i18n` deja de cargar `es.json` de forma fija: carga el idioma elegido, con `en.json` al lado.
  - La primera vez se usa el idioma del sistema si es español o inglés, y si no, inglés.
  - `lib/format` formatea fechas y tamaños según el idioma activo.
  - Un test comprueba que `es.json` y `en.json` tienen las mismas claves.
  - Las guías de `docs/` se traducen al inglés.
- **Editor:**
  - Abre cualquier archivo de texto, sin límite de tamaño. Por encima de 5 MB avisa antes de cargarlo.
  - Un archivo es de texto si no hay bytes nulos en su primer trozo.
  - El resaltado de sintaxis depende de la extensión.
  - Ctrl+S guarda.
  - Al cerrar con cambios sin guardar, pregunta.
  - Si se cae la conexión, el texto se queda en la ventana y «Guardar» vuelve a funcionar al reconectar.
  - En Modo simple solo abre archivos de las carpetas permitidas, igual que el Explorador. El núcleo también lo comprueba.
- **Conflicto al guardar:**
  - El núcleo guarda la fecha de modificación y el tamaño del archivo al abrirlo. Al guardar, si han cambiado, devuelve un error de conflicto.
  - La interfaz pregunta entonces si sobrescribir o guardar como copia. La copia usa el `naming` de nombres libres, que nunca sobrescribe.
  - Vale igual para el Editor y para la aplicación del equipo.
- **Abrir con la aplicación del equipo:**
  - La app descarga el archivo a una carpeta temporal propia y lo abre con la aplicación predeterminada del sistema.
  - Vigila la copia mientras haya conexión y sube cada guardado como una Transferencia normal de la cola.
  - El panel de Transferencias muestra los «archivos abiertos en el equipo» con «Dejar de vigilar».
  - Las copias temporales se borran al desconectar o al cerrar la app.
- **Doble clic y «Abrir con…»:**
  - El doble clic abre los archivos de texto en el Editor y el resto en la aplicación del equipo.
  - El menú contextual tiene «Abrir con…» con Editor y Aplicación del equipo, más vim solo en Modo avanzado.
  - vim abre una pestaña nueva de la Terminal con `vim <ruta>` y la muestra. Si el Servidor no tiene vim, se ve el error de la propia consola.
- **Administración: lectura.** Leer los ficheros de `/etc/ssh/authorized_keys/` no necesita sudo (son legibles por todos, según la guía del servidor), así que la lista se lee por SFTP.
  - Cada línea se analiza en `lib/authorizedKeys`, donde ya está la generación: el modo se deduce de las opciones y el nombre es el comentario de la clave.
  - Si las opciones no corresponden a ningún modo, la clave sale como «Personalizada». Sus opciones nunca se reescriben; solo se puede revocar.
- **Administración: escritura (ADR 0007).**
  - Autorizar, revocar y cambiar de modo escriben el fichero del usuario con `sudo` por un canal SSH. Exige que el Administrador tenga clave de Modo avanzado.
  - La contraseña de sudo se pide la primera vez que se cambia algo en la sesión y vive solo en memoria, nunca en disco ni en el almacén de secretos. Si es incorrecta o el usuario no tiene sudo, se explica con un error claro.
  - La escritura sustituye el fichero entero con permisos `644` y propietario root, para que no quede a medias.
  - Revocar la clave con la que se está conectado, o pasarla a Modo simple, pide una confirmación reforzada que explica que se perderá el acceso o la Administración. No se bloquea.
  - La Administración absorbe la pantalla «Crear perfil», incluido el «Autorizar clave» actual, que pasa de copiar la línea a escribirla en el Servidor.
- **Publicación:** cada ticket terminado sale en su propia release (0.3.0, 0.4.0…) a través del actualizador, sin esperar al conjunto. Orden previsto:
  1. barra superior, Ajustes y guardado de preferencias;
  2. Temas;
  3. Atajos;
  4. inglés;
  5. Editor;
  6. Abrir con;
  7. Administración.

  Inglés, Editor y Administración solo dependen del primero.

## Testing Decisions

- **Qué es una buena prueba:** como en la v1, verifica comportamiento externo a través de la interfaz pública del núcleo (`Core`) contra el banco de pruebas de OpenSSH de `tests/`, y la lógica sin DOM de `ui/src/lib/` con Vitest.
- **Núcleo:**
  - leer un archivo de texto y guardarlo;
  - conflicto cuando el archivo cambió en el Servidor, y guardar como copia sin sobrescribir;
  - en Modo simple, el Editor no puede leer ni escribir fuera de las carpetas permitidas;
  - listar las claves de `/etc/ssh/authorized_keys/`;
  - autorizar, revocar y cambiar de modo con sudo, y comprobar que después el Servidor acepta o rechaza la clave en el modo esperado;
  - contraseña de sudo incorrecta y usuario sin sudo.

  El banco de pruebas necesita un usuario con sudo y `AuthorizedKeysFile` apuntando a `/etc/ssh/authorized_keys/%u`.
- **Interfaz (Vitest):**
  - conflictos y normalización de Atajos (incluido Ctrl↔Cmd);
  - análisis de líneas de `authorized_keys` (modo simple, avanzado y Personalizada, con y sin comentario);
  - qué cuenta como archivo de texto;
  - preferencias con valores que faltan;
  - mismas claves en `es.json` y `en.json`;
  - formato de fechas y tamaños por idioma.
- **Referencia previa:** las pruebas de la v1 en `tests/` y en `ui/src/lib/*.test.ts`, en particular `authorizedKeys.test.ts` y `terminalTabs.test.ts`.

## Out of Scope

- Temas con nombre propio (Kanagawa Dragon, Dracula…) y Temas importados por el Miembro. El formato ya los permite.
- Leer ficheros enormes por trozos o ver solo el final de un log.
- Crear usuarios Linux desde la Administración.
- Otras preferencias: carpeta de descargas por defecto, número de Transferencias en paralelo y si confirmar al mandar a la Papelera.
- Más idiomas aparte de español e inglés.
- Lo que la v1 ya dejó fuera y sigue fuera: registro de actividad, monitorización y gestión de servicios, túneles, gestión de permisos desde la app y firma de código.

## Further Notes

- Vocabulario según `GLOSSARY.md`. Nuevos en la v2: Administración, Editor, Abrir con, Ajustes, Tema y Atajo.
- ADR nuevo: 0007, la Administración escribe las claves con sudo y la contraseña del Administrador.
- Los tickets de la v2 son GitHub Issues. El ticket local de Ajustes y atajos que existía antes de esta spec quedó absorbido en los de Ajustes y Atajos.
