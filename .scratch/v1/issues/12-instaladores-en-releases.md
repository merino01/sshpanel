# 12: Instaladores en releases

**What to build:** Al etiquetar una versión, la CI publica instaladores nativos de los tres sistemas en GitHub Releases.

**Blocked by:** 01-esqueleto-tauri-y-branding; 09-modo-avanzado; 10-crear-perfil

**Status:** ready-for-agent

- [ ] Instaladores para Windows, macOS y Linux (.msi/.exe, .dmg, .AppImage/.deb)
- [ ] La extensión del Perfil queda asociada a la app al instalar
- [ ] Sin firma de código; la mini-guía cubre los avisos

## Comments

- 2026-10-14: Desde el ticket 11: la extensión ya está declarada en `tauri.conf.json`, pero la app no recoge el fichero cuando se abre con doble clic (no lee los argumentos ni el evento de fichero abierto de macOS). Al hacerlo, añadir a la guía de instalación que el Perfil se puede abrir con doble clic.
