# Spec v1: SSH Panel

## Problem Statement

Un grupo de personas comparte un Servidor (VPS) y necesita subir archivos, descargarlos y, a veces, ejecutar comandos. Algunos Miembros no tienen conocimientos técnicos y no pueden usar SSH, SFTP ni una terminal sin riesgo de romper algo. Las herramientas existentes (clientes SFTP genéricos) exigen configurar claves, huellas y rutas a mano, y no distinguen entre quien solo mueve archivos y quien administra.

## Solution

Una aplicación de escritorio multiplataforma (Linux, macOS, Windows) con interfaz moderna. El Miembro importa un Perfil que le da el Administrador y entra directamente a un Explorador de los archivos del Servidor, con arrastrar y soltar. Los Miembros avanzados activan además una Terminal. La app genera y custodia la clave del Miembro, verifica la Huella del Servidor sin molestar al usuario y protege contra borrados accidentales con una Papelera. El proyecto es genérico: cualquiera puede clonarlo y personalizar su Branding.

## User Stories

1. Como Miembro no técnico, quiero abrir la app e importar un solo fichero de Perfil, para poder entrar sin escribir direcciones ni usuarios.
2. Como Miembro, quiero que la app genere mi clave automáticamente, para no tener que usar la línea de comandos.
3. Como Miembro, quiero un botón "Copiar mi clave pública", para enviársela al Administrador y que me autorice.
4. Como Miembro, quiero que mi clave privada se guarde en el almacén seguro de mi sistema, para que nadie pueda copiarla fácilmente.
5. Como Miembro, quiero que mi clave privada nunca salga de mi equipo, para que ni el Administrador ni un tercero puedan suplantarme.
6. Como Miembro, quiero que la app se niegue a conectar si la Huella no coincide, con un mensaje claro, para no entregar mis archivos a un servidor falso.
7. Como Miembro no técnico, quiero no ver nunca avisos tipo "¿confías en este host?", para no tener que decidir algo que no entiendo.
8. Como Miembro, quiero ver al entrar mi carpeta personal en el Explorador, para empezar en un sitio conocido.
9. Como Miembro, quiero un acceso directo a la Carpeta compartida cuando el Perfil la defina, para encontrar rápido lo común.
10. Como Miembro, quiero navegar por carpetas, subir y entrar en ellas, para encontrar mis archivos.
11. Como Miembro, quiero subir archivos arrastrándolos a la ventana, para no tener que buscar menús.
12. Como Miembro, quiero subir archivos con un selector del explorador de mi sistema, para hacerlo sin arrastrar.
13. Como Miembro, quiero descargar archivos y carpetas a mi equipo, para trabajar con ellos en local.
14. Como Miembro, quiero crear carpetas y renombrar archivos, para mantener el orden.
15. Como Miembro, quiero ver una cola de Transferencias con barra de progreso, para saber cuánto falta.
16. Como Miembro, quiero cancelar una Transferencia, para parar algo que inicié por error.
17. Como Miembro, quiero reintentar una Transferencia fallida, para no empezar desde cero.
18. Como Miembro con mala conexión, quiero que una Transferencia interrumpida se reanude donde se quedó, para no repetir archivos grandes.
19. Como Miembro, quiero varias Transferencias a la vez, para no esperar a que acabe una para empezar otra.
20. Como Miembro, quiero que borrar pida confirmación, para no perder cosas por un clic accidental.
21. Como Miembro, quiero que lo borrado vaya a la Papelera del Servidor, para poder recuperarlo si me equivoqué.
22. Como Miembro, quiero restaurar un archivo desde la Papelera, para deshacer un borrado.
23. Como Miembro, quiero mensajes de error en lenguaje claro, sin jerga técnica, para entender qué hacer.
24. Como Miembro en Modo simple, quiero no ver la Terminal, para no poder romper nada con un comando.
25. Como Miembro avanzado, quiero una Terminal embebida, para ejecutar comandos y ver sus respuestas sin salir de la app.
26. Como Miembro avanzado, quiero vaciar la Papelera, para liberar espacio en el Servidor.
27. Como Miembro, quiero que la interfaz esté en español, para entenderla sin esfuerzo.
28. Como Miembro, quiero actualizar la app con un botón cuando haya versión nueva, para no reinstalar a mano.
29. Como Miembro, quiero instaladores nativos para mi sistema y una mini-guía, para instalarla pese a los avisos de desarrollador desconocido.
30. Como Administrador, quiero una pantalla "Crear Perfil", para generar el fichero sin escribirlo a mano.
31. Como Administrador, quiero que la app calcule la Huella del Servidor al crear el Perfil, para no obtenerla por línea de comandos.
32. Como Administrador, quiero definir la Carpeta compartida y el modo (simple o avanzado) en el Perfil, para adaptar la app a cada Miembro.
33. Como Administrador, quiero una guía de configuración del servidor (usuario por persona, `authorized_keys`, SFTP), para preparar el Servidor sin errores.
34. Como Administrador, quiero revocar a un Miembro quitando su clave pública del Servidor, para cortar su acceso sin afectar a los demás.
35. Como Administrador, quiero que cada Miembro tenga su propio usuario Linux, para que los permisos los gestione el Servidor.
36. Como persona que clona el proyecto, quiero cambiar nombre, identificador, extensión, icono y color en un solo fichero `branding.toml`, para adaptarlo a mi grupo.
37. Como persona que clona el proyecto, quiero que el repositorio no contenga ningún host, usuario, Huella ni clave, para poder usarlo y publicarlo sin riesgo.
38. Como persona que clona el proyecto, quiero una guía para configurar mis propias claves de firma del actualizador y mi repositorio de releases, para publicar mis propias versiones.

## Implementation Decisions

- **Marco de UI:** Tauri, con frontend en TypeScript y backend en Rust (ADR 0001). La terminal usa xterm.js.
- **Núcleo Rust:** un módulo único, independiente de la interfaz, que expone al frontend comandos de alto nivel: importar Perfil, generar identidad, conectar, listar, transferir, borrar a la Papelera, restaurar, abrir Terminal y crear Perfil. La lógica SSH/SFTP está detrás de esa interfaz. La biblioteca concreta (`russh` o `ssh2`) se decide al implementar.
- **Identidad:** el par de claves lo genera la app en el equipo del Miembro. La clave privada se guarda en el almacén seguro del sistema (Keychain, Credential Manager, Secret Service), sin passphrase adicional. Solo la clave pública se entrega al Administrador.
- **Perfil:** fichero importable (extensión por defecto `.dbdesp`, definida por el Branding) con dirección, usuario, Huella, modo (simple o avanzado) y, opcionalmente, Carpeta compartida. No contiene claves privadas. El formato exacto se define al implementar.
- **Verificación del Servidor:** la Huella va en el Perfil. La app la comprueba en cada conexión y rechaza la conexión si no coincide, sin preguntar al usuario.
- **Explorador:** arranca en la carpeta personal del Miembro y permite navegar donde los permisos del usuario Linux lo permitan. La app no gestiona permisos.
- **Papelera:** los borrados desde la app mueven los archivos a una carpeta de la Papelera en el Servidor, tras confirmación. Solo el Modo avanzado puede vaciarla.
- **Modos:** el modo se fija en el Perfil. El Modo simple no incluye la Terminal.
- **Transferencias:** cola con progreso, cancelar, reintentar y varias en paralelo; la reanudación entra en la v1 si el coste es razonable.
- **Administración:** pantalla mínima "Crear Perfil", visible solo con un Perfil de administrador, que calcula la Huella conectándose al Servidor. El resto de la administración del Servidor es manual, según la guía.
- **Branding y genericidad (ADR 0002):** el proyecto se llama `sshpanel`. `branding.toml` contiene nombre, identificador, extensión, icono y color de acento, y un script lo vuelca en la configuración de Tauri. "DBDesp Panel" es el valor por defecto. El código no contiene valores de ninguna instancia.
- **Textos:** solo español en la v1, separados del código para poder traducir después.
- **Distribución:** instaladores nativos (.msi/.exe, .dmg, .AppImage/.deb) generados con GitHub Actions, sin firma de código al principio. Actualizador integrado de Tauri sobre GitHub Releases si resulta viable sin firma.
- **Repositorio:** público, licencia MIT. `.gitignore` excluye `*.dbdesp`, `*.pem`, `id_*` y `.env`.
- **Entregables no-código:** guía de configuración del servidor (con datos ficticios) y mini-guía de instalación.

## Testing Decisions

- **Qué es una buena prueba:** verifica comportamiento externo a través de la interfaz del núcleo, no detalles internos.
- **Punto de prueba único:** la interfaz del núcleo Rust, ejecutada contra un servidor OpenSSH efímero en contenedor. Cubre importación de Perfil, Huella, listado, Transferencias (incluidos corte y reanudación), Papelera y Terminal.
- **Interfaz gráfica:** un único smoke test de extremo a extremo (arrancar la app, importar un Perfil de prueba y listar), sin más puntos de prueba.
- **Casos clave:** Huella que no coincide, clave no autorizada, borrado que va a la Papelera y se restaura, Transferencia cortada que se reanuda, y Modo simple sin acceso a la Terminal.
- **Referencia previa:** ninguna; el repositorio parte vacío.

## Out of Scope

- Registro de actividad en la app.
- Monitorización del Servidor y gestión de servicios.
- Editor de texto remoto y túneles.
- Varios idiomas (la v1 es solo español).
- Gestión de permisos desde la app.
- Firma de código de los instaladores.
- Administración del Servidor desde la app, más allá de crear Perfiles.

## Further Notes

- Vocabulario según `GLOSSARY.md`: Servidor, Miembro, Administrador, Perfil, Modo simple, Modo avanzado, Explorador, Terminal, Huella, Clave pública, Carpeta compartida, Papelera, Transferencia, Branding.
- Hitos propuestos: base y Branding; identidad y conexión; Explorador; Transferencias; Terminal; administración y guías; empaquetado y actualizador.
- Pendiente de tracker: este spec no se ha publicado como issue porque no hay tracker configurado (`/setup-matt-pocock-skills`).
