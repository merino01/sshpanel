# SSH Panel

Aplicación de escritorio multiplataforma con la que un grupo de personas gestiona un VPS compartido: explorar y transferir archivos y, opcionalmente, ejecutar comandos.

## Language

**Servidor**:
El VPS compartido al que se conectan todos los usuarios de la app.
_Avoid_: VPS (en la interfaz), máquina, host

**Miembro**:
Persona del grupo que usa la app para conectarse al Servidor con su propia identidad.
_Avoid_: Cliente, usuario (ambiguo con usuario Linux)

**Administrador**:
Miembro que configura el Servidor, autoriza a otros Miembros y genera sus Perfiles. Su Perfil lleva la marca de administrador, que solo le muestra la Administración en la app: su poder real es poder editar como root los `authorized_keys` del Servidor.
_Avoid_: Owner, root

**Perfil**:
Fichero importable que contiene los datos para que un Miembro se conecte al Servidor (dirección, usuario, Huella y modo). Nunca contiene claves y solo configura la app: lo que el Miembro puede hacer lo decide el Servidor (ADR 0006).
_Avoid_: Config, ajustes de conexión

**Modo simple**:
Nivel de la app que solo muestra el explorador de archivos y acciones con nombres claros, sin terminal, y limitado a la carpeta personal del Miembro y la Carpeta compartida. Lo hace cumplir el Servidor si el Administrador autoriza la clave con las opciones de este modo.
_Avoid_: Modo básico, modo fácil

**Modo avanzado**:
Nivel de la app que añade la Terminal al explorador de archivos. Se elige al configurar el Perfil, y el Servidor solo permite la Terminal si la clave está autorizada con las opciones de este modo.
_Avoid_: Modo experto, modo pro

**Administración**:
Apartado de la app, visible solo con Perfil de administrador, donde se crean Perfiles y se autorizan, revocan o cambian de modo las Claves públicas de los Miembros en el Servidor.
_Avoid_: Panel de admin, backoffice

**Explorador**:
Vista de los archivos y carpetas del Servidor, con subida y descarga por arrastrar y soltar o por selector.
_Avoid_: File manager, gestor de archivos

**Editor**:
Ventana de la app para leer y modificar un archivo de texto del Servidor y guardarlo de vuelta en él.
_Avoid_: Visor, bloc de notas

**Abrir con**:
Elección de con qué se abre un archivo del Servidor: el Editor, una aplicación del equipo del Miembro (la app vigila la copia local y sube cada cambio) o vim en la Terminal.
_Avoid_: Ejecutar, lanzar

**Terminal**:
Consola embebida para ejecutar comandos en el Servidor y ver sus respuestas. Solo existe en Modo avanzado.
_Avoid_: Consola, shell

**Huella**:
Identificador único del Servidor incluido en el Perfil; la app solo conecta si coincide con el del Servidor real.
_Avoid_: Fingerprint, certificado

**Clave pública**:
Mitad no secreta de la identidad de un Miembro, que este entrega al Administrador para que lo autorice. La clave privada nunca sale del equipo del Miembro.
_Avoid_: Contraseña, token

**Carpeta compartida**:
Carpeta del Servidor opcionalmente definida en el Perfil, con acceso directo fijo en el Explorador.
_Avoid_: Carpeta de grupo, share

**Papelera**:
Carpeta del Servidor a la que van los archivos borrados desde la app en lugar de eliminarse. Solo en Modo avanzado se vacía o se borra algo de ella para siempre.
_Avoid_: Basura, recycle bin

**Transferencia**:
Subida o descarga de un archivo entre el equipo del Miembro y el Servidor, gestionada en una cola con progreso, cancelación, reintento y reanudación.
_Avoid_: Upload, descarga/subida suelta

**Branding**:
Conjunto de valores fijados al compilar (nombre, identificador, extensión del Perfil, icono, color de acento y, opcionalmente, el repositorio de releases y la clave pública del actualizador) que personalizan la app para una instancia concreta. "SSH Panel" es el Branding por defecto; cada instancia es un fork con el suyo.
_Avoid_: Tema, skin, configuración

**Ajustes**:
Ventana de la app con las preferencias del Miembro, guardadas en su equipo y no en el Perfil.
_Avoid_: Configuración, opciones (el Perfil también configura la app)

**Tema**:
Paleta completa de colores de la interfaz y de la Terminal que el Miembro elige en Ajustes. Los Temas básicos (Sistema, Claro, Oscuro, Alto contraste) usan el color de acento del Branding; los Temas con nombre propio traen el suyo.
_Avoid_: Skin, apariencia (como nombre de cosa)

**Transparencia**:
Preferencia de Ajustes que hace de cristal el marco de la ventana (la barra de título y la lateral), dejando ver lo que hay detrás. Solo en macOS y Windows; en Linux y con Alto contraste la ventana es siempre opaca.
_Avoid_: Vibrancy, efecto cristal (como nombre de la preferencia)

**Atajo**:
Combinación de teclas asignada a una acción de la app, configurable en Ajustes.
_Avoid_: Shortcut, acceso directo (son los de la barra lateral)

## En inglés

Cómo se dicen estos términos en la interfaz y las guías en inglés (`i18n/en.json`, `docs/*-guide.md`). Se escriben con mayúscula igual que en español.

| Español | Inglés | _Avoid_ |
| --- | --- | --- |
| Servidor | Server | VPS, host |
| Miembro | Member | Client, user |
| Administrador | Administrator | Owner, root |
| Perfil | Profile | Config |
| Modo simple | Simple mode | Basic mode |
| Modo avanzado | Advanced mode | Expert mode, pro mode |
| Administración | Administration | Admin panel |
| Explorador | Explorer | File manager |
| Editor | Editor | Viewer |
| Abrir con | Open with | Run, launch |
| Terminal | Terminal | Console, shell |
| Huella | Fingerprint | Certificate |
| Clave pública | Public key | Password, token |
| Carpeta compartida | Shared folder | Group folder, share |
| Papelera | Trash | Recycle bin, bin |
| Transferencia | Transfer | — |
| Branding | Branding | Theme, skin |
| Ajustes | Settings | Preferences, options |
| Tema | Theme | Skin |
| Transparencia | Transparency | Vibrancy, glass |
| Atajo | Shortcut | Hotkey; y nunca para los accesos directos de la barra lateral |
