# 13: Actualizador integrado

**What to build:** La app comprueba las releases al arrancar y ofrece "Actualizar" con un botón.

**Blocked by:** 12-instaladores-en-releases

**Status:** in-progress

- [ ] Detecta una versión nueva en GitHub Releases al arrancar
- [ ] Ofrece actualizar con un botón y aplica la actualización
- [x] Si no es viable sin firma de código, se documenta la reinstalación manual y se acuerda con el usuario
- [x] README: cómo configurar las claves de firma del actualizador y el repositorio de releases (viene del ticket 11)

## Comments

- 2026-10-14: Es viable sin firma de código: el actualizador de Tauri usa sus propias claves (`tauri signer generate`). `branding.toml` gana `releases_repo` y `updater_pubkey`; con los dos, `apply-branding.mjs` escribe `plugins.updater` con el endpoint `…/releases/latest/download/latest.json` (los borradores no cuentan como `latest`), y sin ellos el plugin no se carga y la app no busca nada. Comandos `check_update`, `install_update` (descarga, instala y reinicia) y `open_releases`; la interfaz busca al arrancar, en silencio si falla, y ofrece «Actualizar» en un aviso fijo, con confirmación si hay Transferencias sin terminar y «Descargar» si falla. El workflow `Release` firma solo si existe el secret `TAURI_SIGNING_PRIVATE_KEY` (`--config src-tauri/tauri.release.conf.json`), así que `tauri build` en local y los forks sin claves siguen igual. Comprobado en local con una clave desechable: salen `.deb.sig` y `.AppImage.sig`, y la app consulta el endpoint al arrancar. Falta: que el usuario genere las claves y suba los secrets, y probar de verdad una actualización de una versión a otra (casillas 1 y 2).
