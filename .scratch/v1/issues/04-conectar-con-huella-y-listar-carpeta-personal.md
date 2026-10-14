# 04: Conectar verificando la Huella y listar la carpeta personal

**What to build:** Con un Perfil importado y la Clave pública autorizada, la app se conecta al Servidor, verifica la Huella sin preguntar al usuario y muestra los archivos de su carpeta personal.

**Blocked by:** 02-banco-de-pruebas-openssh; 03-importar-perfil-y-generar-identidad

**Status:** done

- [x] Conecta y lista la carpeta personal contra el banco de pruebas
- [x] Si la Huella no coincide, rechaza la conexión con un mensaje claro y no envía nada
- [x] Si la clave no está autorizada, muestra un mensaje claro que invita a contactar al Administrador
- [x] El Miembro nunca ve avisos técnicos del tipo "¿confías en este host?"

## Comments

- 2026-10-14: Probado a mano contra un Servidor real. Al importar se genera la clave; tras autorizarla en el Servidor, la app conecta y lista la carpeta personal.
