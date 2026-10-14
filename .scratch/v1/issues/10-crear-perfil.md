# 10: Crear Perfil (Administrador)

**What to build:** El Administrador usa una pantalla mínima para generar un Perfil con host, usuario, modo y Carpeta compartida, calculando la Huella al conectarse al Servidor.

**Blocked by:** 04-conectar-con-huella-y-listar-carpeta-personal

**Status:** done

- [x] Formulario que produce un fichero de Perfil con la extensión del Branding
- [x] La Huella se calcula conectándose al Servidor, sin que el Administrador la copie a mano
- [x] La pantalla solo es visible con un Perfil de administrador
- [x] El Perfil generado no contiene claves privadas
- [x] Un Perfil generado se puede importar con el ticket 03

## Comments

- 2026-10-14: Decidido con el usuario: el Perfil gana el campo opcional `admin` (los Perfiles anteriores siguen valiendo y no son de administrador; si es `false` no se escribe). «Crear perfil» aparece en la barra lateral con un Perfil de administrador y, para que el primero pueda crearse el suyo, como enlace «¿Eres el administrador? Crea un perfil» en la Bienvenida. Crear un Perfil no da acceso a nada: eso lo decide el Servidor.
- 2026-10-14: Núcleo: `Core::create_profile(NewProfile, fichero)`. Valida los datos antes de conectar (`profile.invalid_field`), calcula la Huella con el apretón de manos SSH rechazando la clave a propósito, así que no se autentica ni envía el usuario, y solo escribe el fichero si lo consigue (`connection.unreachable` si no). Probado contra el banco de pruebas (`tests/create_profile.rs`): el Perfil creado se importa y conecta, no lleva claves, marca administradores, no se envía el usuario y un servidor inalcanzable no deja fichero.
- 2026-10-14: Interfaz: el formulario explica campo a campo lo que está mal (`lib/profileForm.ts`, mismas reglas que el núcleo) y propone el mismo Servidor y Carpeta compartida del Perfil importado. Al guardar muestra la Huella y el siguiente paso, y permite «Crear otro». Abierta desde una pantalla de trabajo conserva la barra lateral y las Terminales.
