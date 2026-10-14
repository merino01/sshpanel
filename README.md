# SSH Panel

App de escritorio (Linux, macOS y Windows) para que un grupo de personas use un servidor compartido sin saber de SSH: explorar sus archivos, subirlos y descargarlos arrastrando, recuperar lo borrado de una papelera y, quien lo necesite, abrir una terminal.

El administrador del servidor le da a cada persona un fichero de perfil. Ella lo importa y entra: la app genera y guarda su clave, comprueba que el servidor es el auténtico y nunca le pide configurar nada.

El proyecto es genérico: cualquiera puede hacer un fork y publicar su propia versión, con su nombre, icono y color (ver [Tu propia versión](#tu-propia-versión)).

## Qué hace

- **Explorador** de los archivos del servidor: navegar, crear carpetas, renombrar, menú contextual.
- **Transferencias** con cola, progreso, varias a la vez, cancelar, reintentar y reanudar tras un corte.
- **Papelera**: lo borrado se puede restaurar, también con «Deshacer» justo después de borrar.
- **Modo simple**, solo archivos con nombres claros, y **Modo avanzado**, con Terminal en pestañas y vaciar la Papelera.
- **Crear perfil**, para el administrador: calcula la huella del servidor y genera la línea de `authorized_keys` de cada persona según su modo.

Quien decide lo que puede hacer cada persona es el servidor, no la app (ver la [ADR 0006](docs/adr/0006-el-servidor-es-la-barrera-de-seguridad.md)).

## Guías

- [Guía del servidor](docs/guia-servidor.md), para el administrador: usuarios, claves, Carpeta compartida, modos, dar de alta y de baja.
- [Guía de instalación](docs/guia-instalacion.md), para cada persona: instalar pese a los avisos de «desarrollador desconocido» y entrar por primera vez.

El diseño está en el [spec de la v1](docs/spec-v1.md), las decisiones en [`docs/adr/`](docs/adr/) y el vocabulario en el [glosario](GLOSSARY.md).

## Desarrollo

Necesitas:

- [Rust](https://rustup.rs) estable.
- Node.js 22.
- Docker, para las pruebas del núcleo: levantan un servidor OpenSSH de usar y tirar.
- En Linux, las dependencias de Tauri:

  ```sh
  sudo apt install libwebkit2gtk-4.1-dev libayatana-appindicator3-dev librsvg2-dev patchelf
  ```

Comandos:

```sh
npm ci                       # dependencias
npm run tauri dev            # la app, recargándose al cambiar el código
cargo test -p sshpanel-core  # pruebas del núcleo contra el servidor de prueba
npm test                     # pruebas de la lógica de la interfaz
npm run check                # comprobación de tipos de la interfaz
```

### Estructura

- `crates/core`: el núcleo en Rust. Es el único sitio con lógica de SSH y SFTP, y se prueba a través de su API pública contra el servidor de prueba de `crates/core/tests/`.
- `src-tauri`: la app Tauri. Comandos finos, uno por área, en `src/commands/`.
- `ui/src`: la interfaz en Svelte 5.
  - `lib/api.ts` es lo único que llama al núcleo.
  - `lib/` tiene la lógica sin DOM, probada con Vitest.
  - `state/` guarda el estado de la app.
  - `screens/` y `components/` tienen las pantallas y sus piezas.
  - Los textos están en `i18n/es.json`.

## Publicar una versión

1. Sube la versión en `Cargo.toml` (`[workspace.package]`), `src-tauri/tauri.conf.json` y `package.json`. `node scripts/check-version.mjs` comprueba que coinciden.
2. Haz commit, etiqueta y súbelo:

   ```sh
   git tag v0.2.0
   git push origin master v0.2.0
   ```

3. El workflow `Release` compila los instaladores de Windows (`.msi`, `.exe`), macOS (`.dmg` para Apple Silicon y para Intel) y Linux (`.deb`, `.rpm`, `.AppImage`) y los deja en un **borrador** de Release. Si la etiqueta no coincide con la versión, falla sin crear nada.
4. Revisa el borrador en la página de Releases de GitHub y publícalo.

Para probar los instaladores sin publicar nada, lanza el workflow `Release` a mano («Run workflow» en la pestaña Actions): los deja como artefactos del run.

Los instaladores no están firmados; la [guía de instalación](docs/guia-instalacion.md) explica cómo abrirlos pese a los avisos.

## Branding

El nombre, el icono y los colores de la app salen de `branding.toml`:

| Campo | Qué es |
| --- | --- |
| `name` | Nombre de la app, en la ventana, los instaladores y los textos. |
| `identifier` | Identificador único de la app, en formato de dominio al revés (`com.example.panel`). Cada versión debe tener el suyo. |
| `profile_extension` | Extensión de los ficheros de perfil (por defecto, `sshpanel`). Solo minúsculas y dígitos. |
| `icon` | Imagen PNG cuadrada de la que se generan todos los iconos. |
| `accent_color` | Color de acento, en `#rrggbb`. |

Tras editarlo, aplica los cambios:

```sh
npm run branding
```

Esto regenera `src-tauri/tauri.conf.json`, `ui/src/branding.generated.ts`, los ficheros de Linux de `src-tauri/linux/` (para abrir los perfiles con doble clic), la línea de los perfiles en `.gitignore` (para que nunca acaben en el repositorio) y los iconos de `src-tauri/icons/`; commitea el resultado. La CI comprueba que lo commiteado coincide con `branding.toml`.

El identificador y la extensión se fijan al compilar ([ADR 0002](docs/adr/0002-proyecto-generico-con-branding-en-compilacion.md)). Si los cambias, es otra app: los perfiles y las claves guardadas con la anterior no se comparten.

## Tu propia versión

Para dar la app a tu grupo con vuestro nombre, icono y color:

1. Haz un fork de este repositorio.
2. En tu fork, edita `branding.toml`. Ponle un `identifier` propio: si usaras el de este proyecto, tu app y la genérica compartirían datos y clave en el mismo equipo.
3. Ejecuta `npm run branding` y commitea el resultado.
4. Publica una versión como se explica en [Publicar una versión](#publicar-una-versión): el workflow de tu fork compila tus instaladores y los deja en tus Releases.

Para traer las mejoras de este repositorio, sincroniza tu fork: solo cambian tus valores de `branding.toml` y los ficheros que genera.

## Licencia

[MIT](LICENSE).
