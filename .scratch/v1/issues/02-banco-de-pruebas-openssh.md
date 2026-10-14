# 02: Banco de pruebas OpenSSH

**What to build:** Un servidor OpenSSH efímero en contenedor, con usuarios y claves de prueba, y un arnés para que las pruebas del núcleo Rust lo levanten y lo apaguen. Es el único punto de prueba del spec.

**Blocked by:** None (can start immediately)

**Status:** done

- [x] Una prueba de ejemplo levanta el contenedor, se conecta por SFTP con una clave de prueba y lo apaga
- [x] El arnés expone la Huella del servidor de prueba y permite autorizar y revocar claves
- [x] Funciona en la CI sin intervención manual

## Comments

- 2026-10-14: El usuario confirma que las GitHub Actions están en verde.
