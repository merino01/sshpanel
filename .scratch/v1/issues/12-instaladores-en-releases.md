# 12: Instaladores en releases

**What to build:** Al etiquetar una versión, la CI publica instaladores nativos de los tres sistemas en GitHub Releases.

**Blocked by:** 01-esqueleto-tauri-y-branding; 09-modo-avanzado; 10-crear-perfil

**Status:** done

- [x] Instaladores para Windows, macOS y Linux (.msi/.exe, .dmg, .AppImage/.deb)
- [x] La extensión del Perfil queda asociada a la app al instalar
- [x] Sin firma de código; la mini-guía cubre los avisos

## Comments

- 2026-10-14: Desde el ticket 11: la extensión ya está declarada en `tauri.conf.json`, pero la app no recoge el fichero cuando se abre con doble clic (no lee los argumentos ni el evento de fichero abierto de macOS). Al hacerlo, añadir a la guía de instalación que el Perfil se puede abrir con doble clic.
- 2026-10-14: `.github/workflows/release.yml`: con una etiqueta `vX.Y.Z` comprueba que coincide con la versión (`scripts/check-version.mjs`), crea un borrador de Release y le sube los instaladores de Linux (ubuntu-22.04: `.deb`, `.rpm`, `.AppImage`), Windows (`.msi`, `.exe`) y macOS (`.dmg` Apple Silicon e Intel, este compilado en cruzado porque ya no hay runners Intel). «Run workflow» compila lo mismo y deja los instaladores como artefactos, sin crear Release.
- 2026-10-14: Doble clic: el fichero llega como argumento (Windows y Linux), a la instancia ya abierta con `tauri-plugin-single-instance`, o como `RunEvent::Opened` (macOS). La interfaz lo recoge con `take_opened_profile` y, si ya había otro Perfil, pregunta antes de cambiar (`Core::read_profile` lee sin importar). En Linux, `apply-branding.mjs` genera una plantilla de `.desktop` con `%f` y el XML de shared-mime-info: comprobado instalando el `.deb` en Debian 12 (`xdg-mime` reconoce `.dbdesp` y elige la app). La AppImage no se integra en el sistema, así que con ella se usa «Importar perfil».
- 2026-10-14: Pendiente: ver el workflow funcionando en GitHub y probar el doble clic en Windows y macOS.
