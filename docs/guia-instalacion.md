# Guía de instalación

Cómo instalar la app y entrar por primera vez. Necesitas el fichero de perfil que te ha enviado el administrador.

Descarga el instalador de tu sistema desde la página de versiones (Releases) del proyecto. La app no está firmada por Microsoft ni por Apple, porque esos certificados cuestan dinero cada año. Por eso tu sistema te avisará de que es de un «desarrollador desconocido». Es normal: aquí tienes cómo abrirla igualmente.

## Windows

1. Abre el instalador (`.msi` o `.exe`).
2. Si aparece «Windows protegió su PC», pulsa **Más información** y luego **Ejecutar de todas formas**.
3. Sigue los pasos del instalador.

## macOS

1. Abre el `.dmg` y arrastra la app a **Aplicaciones**.
2. Ábrela desde Aplicaciones. Si dice que **no se puede verificar el desarrollador**:
   1. Pulsa **OK** (o **Hecho**).
   2. Abre **Ajustes del Sistema → Privacidad y seguridad**.
   3. Baja hasta el aviso sobre la app y pulsa **Abrir igualmente**.
   4. Confírmalo con tu contraseña y pulsa **Abrir**.
3. Si en cambio dice que la app **está dañada** y propone moverla a la papelera, no está dañada: es el mismo aviso, más tajante. Abre la app **Terminal** y escribe esto, cambiando el nombre por el de la app si es otro:

   ```sh
   xattr -dr com.apple.quarantine "/Applications/DBDesp Panel.app"
   ```

   Luego ábrela con normalidad.

Solo hace falta la primera vez.

## Linux

- **`.deb`** (Debian, Ubuntu y derivados): ábrelo con el instalador de programas, o en una terminal:

  ```sh
  sudo apt install ./nombre-del-fichero.deb
  ```

- **`.AppImage`** (cualquier distribución): dale permiso para ejecutarse y ábrelo:

  ```sh
  chmod +x nombre-del-fichero.AppImage
  ./nombre-del-fichero.AppImage
  ```

## Primeros pasos

1. Abre la app y pulsa **Importar perfil**. Elige el fichero que te ha enviado el administrador.
2. La app crea tu clave y te la enseña. Pulsa **Copiar mi clave pública** y envíasela al administrador. Es seguro compartirla: la parte secreta nunca sale de tu equipo.
3. Cuando el administrador te avise de que ya tienes acceso, pulsa **Reintentar**.

Desde entonces, la app entra directamente al abrirla.
