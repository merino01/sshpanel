# russh como biblioteca SSH/SFTP

Usamos `russh` y `russh-sftp` en lugar de `ssh2` (libssh2). Son Rust puro, sin libssh2 ni OpenSSL que compilar en Windows y macOS, son asíncronos sobre tokio (que Tauri ya usa) y la comprobación de la clave del Servidor es un callback que corre antes de autenticarse: si la Huella no coincide, la conexión se corta sin enviar usuario ni clave. Compilamos `russh` con el backend criptográfico `ring` en vez del predeterminado `aws-lc-rs`, que exige cmake y NASM en Windows.
