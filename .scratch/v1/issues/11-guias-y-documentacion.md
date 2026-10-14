# 11: Guías y documentación

**What to build:** Guía de configuración del servidor, mini-guía de instalación y README para quien clone el proyecto.

**Blocked by:** 10-crear-perfil

**Status:** ready-for-agent

- [ ] Guía de servidor: usuario Linux por persona, `authorized_keys`, SFTP y revocación de un Miembro, con datos ficticios
- [ ] Mini-guía de instalación que explica los avisos de desarrollador desconocido en Windows y macOS
- [ ] README que explica el Branding y cómo configurar claves de firma del actualizador y repositorio de releases
- [ ] Ningún host, usuario, Huella ni clave reales en el repo
- [ ] La guía de servidor explica que limitar el Modo simple a la carpeta personal y la Carpeta compartida es solo cosa de la app: para que un Miembro no pueda salir de verdad, hay que configurarlo en el Servidor (`ChrootDirectory` con `internal-sftp` en `sshd_config`, o permisos de Linux)

## Comments

- 2026-10-14: En Modo simple la app solo deja usar la carpeta personal y la Carpeta compartida (lo comprueba el núcleo), pero no es una barrera de seguridad: el Miembro tiene su clave y puede usar cualquier cliente SFTP. La guía debe explicar cómo restringirlo en el Servidor si hace falta.
