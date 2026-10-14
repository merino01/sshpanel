# 03: Importar Perfil y generar identidad

**What to build:** El Miembro abre la app, pulsa "Importar perfil", elige el fichero y la app genera su clave, la guarda en el almacén seguro del sistema y le deja copiar su Clave pública.

**Blocked by:** 01-esqueleto-tauri-y-branding

**Status:** done

- [x] Una pantalla de primer arranque con un único botón "Importar perfil"
- [x] El Perfil se valida y un Perfil inválido da un mensaje claro en lenguaje simple
- [x] La clave privada se guarda en el almacén seguro del sistema, sin passphrase, y nunca sale del equipo
- [x] El botón "Copiar mi clave pública" copia solo la Clave pública
- [x] Pruebas a través de la interfaz del núcleo Rust
