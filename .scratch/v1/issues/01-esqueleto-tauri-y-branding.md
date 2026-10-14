# 01: Esqueleto Tauri y Branding

**What to build:** Una app Tauri que arranca en Linux, macOS y Windows, con el Branding aplicado desde `branding.toml` y textos de interfaz en español separados del código (ADR 0001 y 0002).

**Blocked by:** None (can start immediately)

**Status:** in-progress

- [x] Crate y proyecto renombrados a `sshpanel`; sin valores de ninguna instancia en el repo
- [x] `branding.toml` define nombre, identificador, extensión del Perfil, icono y color de acento, y un script los aplica a la app
- [x] Los textos de la interfaz viven en ficheros de idioma, no en el código
- [ ] La CI compila la app para Linux, macOS y Windows (workflow escrito; falta verlo ejecutar en GitHub)
- [x] `.gitignore` excluye `*.dbdesp`, `*.pem`, `id_*` y `.env`; el repo incluye la licencia MIT
