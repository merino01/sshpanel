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
Miembro que configura el Servidor, autoriza a otros Miembros y genera sus Perfiles.
_Avoid_: Owner, root

**Perfil**:
Fichero importable que contiene los datos para que un Miembro se conecte al Servidor (dirección, usuario y clave).
_Avoid_: Config, ajustes de conexión

**Modo simple**:
Nivel de la app que solo muestra el explorador de archivos y acciones con nombres claros, sin terminal, y limitado a la carpeta personal del Miembro y la Carpeta compartida.
_Avoid_: Modo básico, modo fácil

**Modo avanzado**:
Nivel de la app que añade la Terminal al explorador de archivos. Se elige al configurar el Perfil.
_Avoid_: Modo experto, modo pro

**Explorador**:
Vista de los archivos y carpetas del Servidor, con subida y descarga por arrastrar y soltar o por selector.
_Avoid_: File manager, gestor de archivos

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
Carpeta del Servidor a la que van los archivos borrados desde la app en lugar de eliminarse. Solo se vacía en Modo avanzado.
_Avoid_: Basura, recycle bin

**Transferencia**:
Subida o descarga de un archivo entre el equipo del Miembro y el Servidor, gestionada en una cola con progreso, cancelación, reintento y reanudación.
_Avoid_: Upload, descarga/subida suelta

**Branding**:
Conjunto de cinco valores fijados al compilar (nombre, identificador, extensión del Perfil, icono y color de acento) que personalizan la app para una instancia concreta. "DBDesp Panel" es el Branding por defecto.
_Avoid_: Tema, skin, configuración
