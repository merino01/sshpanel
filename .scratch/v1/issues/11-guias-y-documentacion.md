# 11: Guías y documentación

**What to build:** Guía de configuración del servidor, mini-guía de instalación y README para quien clone el proyecto.

**Blocked by:** 10-crear-perfil

**Status:** done

- [x] Guía de servidor: usuario Linux por persona, `authorized_keys`, SFTP y revocación de un Miembro, con datos ficticios
- [x] Mini-guía de instalación que explica los avisos de desarrollador desconocido en Windows y macOS
- [x] README que explica el Branding (las claves de firma del actualizador y el repositorio de releases pasan al ticket 13)
- [x] Ningún host, usuario, Huella ni clave reales en el repo
- [x] La guía de servidor explica que limitar el Modo simple a la carpeta personal y la Carpeta compartida es solo cosa de la app: para que un Miembro no pueda salir de verdad, hay que configurarlo en el Servidor (`ChrootDirectory` con `internal-sftp` en `sshd_config`, o permisos de Linux)

## Comments

- 2026-10-14: En Modo simple la app solo deja usar la carpeta personal y la Carpeta compartida (lo comprueba el núcleo), pero no es una barrera de seguridad: el Miembro tiene su clave y puede usar cualquier cliente SFTP. La guía debe explicar cómo restringirlo en el Servidor si hace falta.
- 2026-10-14: Desde el ticket 10: la guía del servidor debe explicar cómo crea el primer Administrador su Perfil (enlace «¿Eres el administrador? Crea un perfil» en la Bienvenida, marcando «Perfil de administrador») y cómo crea luego los de los Miembros.
- 2026-10-14: Desde la ADR 0006: la guía del servidor debe explicar que el modo lo hace cumplir el Servidor con las opciones de `authorized_keys` (`restrict,command="internal-sftp"` para el Modo simple, `restrict,pty` para el avanzado; las genera «Autorizar clave» en «Crear perfil»), `ChrootDirectory` para encerrar el Modo simple, y que «admin» y el modo del Perfil no son permisos.
- 2026-10-14: Hecho: `docs/guia-servidor.md`, `docs/guia-instalacion.md` y `README.md`, en español (decidido con el usuario). Lo del actualizador pasa al 13, porque aún no existe. Los comandos de la guía del servidor se han probado en un Debian 12 limpio, con el propio núcleo: Carpeta compartida con setgid y ACL (lo que sube uno lo cambia y lo borra otro), `chmod 750` en las carpetas personales, las opciones de cada modo y el encierro con `ChrootDirectory` y montaje desde `fstab` (borrar desde la compartida montada falla con `files.trash_failed`, como dice la ADR 0004).
- 2026-10-14: Al probarlo apareció un fallo en la recomendación de la ADR 0006: con las claves en `~/.ssh/authorized_keys`, un Miembro en Modo simple puede reescribir las suyas por SFTP y tener consola. Las claves van en `/etc/ssh/authorized_keys/%u`, de root. Corregidos la ADR, los textos de «Autorizar clave» y el banco de pruebas, que ahora usa esa configuración, con una prueba del ataque (`tests/server_enforced.rs`).
- 2026-10-14: Datos del repo revisados: solo `example.com`, IPs de documentación y Huellas y claves visiblemente falsas.
