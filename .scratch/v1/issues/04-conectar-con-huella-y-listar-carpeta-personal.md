# 04: Conectar verificando la Huella y listar la carpeta personal

**What to build:** Con un Perfil importado y la Clave pública autorizada, la app se conecta al Servidor, verifica la Huella sin preguntar al usuario y muestra los archivos de su carpeta personal.

**Blocked by:** 02-banco-de-pruebas-openssh; 03-importar-perfil-y-generar-identidad

**Status:** ready-for-agent

- [ ] Conecta y lista la carpeta personal contra el banco de pruebas
- [ ] Si la Huella no coincide, rechaza la conexión con un mensaje claro y no envía nada
- [ ] Si la clave no está autorizada, muestra un mensaje claro que invita a contactar al Administrador
- [ ] El Miembro nunca ve avisos técnicos del tipo "¿confías en este host?"
