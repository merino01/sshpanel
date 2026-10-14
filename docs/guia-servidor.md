# Guía del servidor

Para el Administrador: cómo preparar el Servidor para que los Miembros entren con la app, cómo darlos de alta y de baja, y cómo hacer que cada uno solo pueda hacer lo que le toca.

Los comandos son para Debian y Ubuntu, y se ejecutan en el Servidor con `sudo`. Todos los nombres de esta guía son de ejemplo: el Servidor `vps.example.com` (203.0.113.10) y los Miembros `alice` y `bob`. Cámbialos por los tuyos.

## Cómo funciona, en una frase

Cada Miembro tiene su propio usuario Linux en el Servidor y entra con una clave que genera la app en su equipo. Tú autorizas esa clave en el Servidor, y **es el Servidor el que decide qué puede hacer**: el Perfil que le das solo configura qué le enseña la app (ver [Modos y seguridad](#modos-y-seguridad)).

## 1. Qué necesitas

- Un Servidor Linux con OpenSSH (`sudo apt install openssh-server`), al que ya entres tú con `sudo`.
- El paquete `acl`, para la Carpeta compartida: `sudo apt install acl`.

## 2. Guardar las claves donde los Miembros no lleguen

Por defecto, OpenSSH lee las claves de cada usuario de su propia carpeta (`~/.ssh/authorized_keys`). Eso no sirve aquí: el Miembro puede cambiar ese fichero con la propia app, quitarse los límites de su modo y tener una consola. Las claves irán en `/etc/ssh/authorized_keys/<usuario>`, que solo puede tocar root.

**Antes de cambiarlo, copia allí tu propia clave**, o te quedarás fuera:

```sh
sudo install -d -m 755 /etc/ssh/authorized_keys
sudo cp ~/.ssh/authorized_keys /etc/ssh/authorized_keys/$USER
sudo chmod 644 /etc/ssh/authorized_keys/$USER
```

Después, en `/etc/ssh/sshd_config`, deja esta línea (sustituye la de `AuthorizedKeysFile` si ya existe):

```
AuthorizedKeysFile /etc/ssh/authorized_keys/%u
```

Comprueba la configuración y aplícala **sin cerrar la sesión que tienes abierta**:

```sh
sudo sshd -t && sudo systemctl reload ssh
```

Abre otra terminal y comprueba que sigues entrando antes de cerrar la primera.

## 3. Un usuario Linux por Miembro

```sh
sudo adduser --disabled-password --gecos "" alice
sudo chmod 750 /home/alice
```

Uno por persona, para que cada Miembro tenga sus propios permisos y puedas darle de baja sin afectar a los demás. `--disabled-password` hace que solo se pueda entrar con clave. `chmod 750` hace que el resto de Miembros no pueda ver su carpeta personal.

## 4. Carpeta compartida (opcional)

Una carpeta en la que todos los Miembros pueden subir, cambiar y borrar:

```sh
sudo groupadd compartida
sudo mkdir -p /srv/compartida
sudo chgrp compartida /srv/compartida
sudo chmod 2770 /srv/compartida
sudo setfacl -d -m g:compartida:rwx /srv/compartida
sudo usermod -aG compartida alice
sudo usermod -aG compartida bob
```

- `2770`: solo el grupo `compartida` entra, y lo que se crea dentro sigue siendo del grupo.
- `setfacl -d`: lo que sube uno lo puede cambiar o borrar otro, también lo que haya dentro de carpetas nuevas.

Lo que se borra desde la Carpeta compartida va a la Papelera de quien lo borra, en su carpeta personal. Para eso, **la Carpeta compartida tiene que estar en el mismo disco que `/home`**; si no, borrar desde ella falla con un aviso y no se borra nada.

La ruta (`/srv/compartida`) es la que pondrás en el campo «Carpeta compartida» al crear los Perfiles.

## 5. Tu Perfil de Administrador

1. Instala la app (ver la [guía de instalación](guia-instalacion.md)).
2. En la pantalla de bienvenida, pulsa «¿Eres el administrador? Crea un perfil».
3. Rellena el Servidor, tu usuario, Modo avanzado y la Carpeta compartida, y marca «Perfil de administrador». La app se conecta al Servidor para calcular su Huella; no tienes que copiarla.
4. Guarda el fichero y vuelve a la bienvenida para importarlo con «Importar perfil».
5. La app te enseña tu clave pública. En «Autorizar clave» (en la pantalla «Crear perfil»), pégala, elige Modo avanzado y añade la línea que sale a tu fichero de claves:

```sh
sudo nano /etc/ssh/authorized_keys/tu-usuario
```

6. Pulsa «Reintentar» en la app.

## 6. Dar de alta a un Miembro

1. Crea su usuario (paso 3) y, si hace falta, añádelo al grupo `compartida` (paso 4).
2. En la app, «Crear perfil» (abajo en la barra lateral): rellena sus datos y su modo, y guarda el fichero, por ejemplo `alice.dbdesp`.
3. Envíale el fichero. Al importarlo, la app le enseña su clave pública con un botón para copiarla, y te la manda.
4. En «Autorizar clave», pega su clave, elige su modo y copia la línea. Tiene este aspecto:

```
restrict,command="internal-sftp" ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIEjEmPlOdEcLaVeNoReAlXxXxXxXxXxXxXxXxXxXxXxX alice@portatil
```

5. Añádela a su fichero de claves:

```sh
sudo nano /etc/ssh/authorized_keys/alice
sudo chmod 644 /etc/ssh/authorized_keys/alice
```

6. Dile que pulse «Reintentar».

El Perfil no contiene ninguna clave, así que puedes enviarlo por cualquier medio. La clave pública tampoco es secreta.

## Modos y seguridad

El modo del Perfil solo decide qué enseña la app. Lo que cada Miembro puede hacer de verdad lo deciden las opciones que van delante de su clave, y por eso las pone «Autorizar clave»:

| Modo | Opciones | Qué puede hacer |
| --- | --- | --- |
| Simple | `restrict,command="internal-sftp"` | Solo archivos (SFTP). Sin consola ni túneles. |
| Avanzado | `restrict,pty` | Archivos y Terminal. Sin túneles ni reenvío de agente. |

Si un Miembro edita su Perfil para ponerse Modo avanzado, la app le enseñará la Terminal, pero el Servidor no le dejará abrirla y la app se lo explicará.

Lo mismo con la marca de administrador: solo muestra «Crear perfil». Cualquiera puede crear Perfiles, pero un Perfil no da acceso a nada. Ser Administrador de verdad es poder editar `/etc/ssh/authorized_keys`.

## 7. Encerrar el Modo simple en sus carpetas

En Modo simple, la app solo deja usar la carpeta personal y la Carpeta compartida. Es una ayuda para no perderse, no un límite: con su clave, el Miembro puede usar cualquier programa de SFTP y ver el resto del Servidor donde Linux le dé permiso de lectura. Hay dos formas de limitarlo.

### Opción A: permisos de Linux (recomendada)

Es lo que ya hacen los pasos 3 y 4: nadie ve las carpetas personales de los demás (`chmod 750`) y la Carpeta compartida es solo del grupo. El Miembro podrá ver archivos del sistema que son públicos en cualquier Linux, como los de `/etc` o `/usr`, pero no lo de otras personas.

### Opción B: encierro con `ChrootDirectory` (avanzada)

El Miembro solo ve su carpeta y la compartida; el resto del Servidor no existe para él. Exige más mantenimiento y tiene un inconveniente: **no podrá mover a la Papelera lo que esté en la Carpeta compartida**, porque dentro del encierro está montada aparte y es otro sistema de ficheros. Se le avisará y no se borrará nada.

Para encerrar a `alice`:

```sh
sudo groupadd sftp-solo
sudo usermod -aG sftp-solo alice

# La raíz del encierro tiene que ser de root y Alice no puede escribir en ella.
sudo chown root:alice /home/alice
sudo chmod 750 /home/alice

# Sus archivos, dentro. Si ya tenía archivos en /home/alice, muévelos aquí.
sudo install -d -m 700 -o alice -g alice /home/alice/archivos

# La Carpeta compartida, montada dentro.
sudo install -d -m 755 /home/alice/compartida
echo '/srv/compartida /home/alice/compartida none bind 0 0' | sudo tee -a /etc/fstab
sudo mount /home/alice/compartida
```

Y al final de `/etc/ssh/sshd_config`:

```
Match Group sftp-solo
    ChrootDirectory %h
    ForceCommand internal-sftp -d /archivos
    AllowTcpForwarding no
    X11Forwarding no
```

```sh
sudo sshd -t && sudo systemctl reload ssh
```

En su Perfil, la Carpeta compartida es la ruta **de dentro del encierro**: `/compartida`. Su carpeta personal será `/archivos`.

## 8. Dar de baja a un Miembro

Quita su línea de su fichero de claves (o el fichero entero) y corta las sesiones que tenga abiertas:

```sh
sudo rm /etc/ssh/authorized_keys/alice
sudo pkill -u alice
```

Desde ese momento no puede entrar, ni con la app ni con ningún otro programa. Los demás Miembros no se ven afectados. Si además quieres bloquear el usuario o borrarlo con sus archivos:

```sh
sudo usermod -L alice              # bloquearlo
sudo deluser --remove-home alice   # borrarlo con su carpeta personal
```

## 9. Comprobar la Huella (opcional)

La Huella identifica al Servidor: la app la guarda en el Perfil y se niega a conectar si alguna vez no coincide, por si alguien se hace pasar por él. «Crear perfil» la enseña al terminar. Para comprobar que es la de tu Servidor:

```sh
for f in /etc/ssh/ssh_host_*_key.pub; do ssh-keygen -lf "$f"; done
```

Una de las líneas, normalmente la que acaba en `(ED25519)`, debe tener la misma Huella, por ejemplo `SHA256:EjEmPlOdEhUeLlAnOeSrEaLxXxXxXxXxXxXxXxXxXxX`.

Si reinstalas el Servidor o cambias sus claves de host, la Huella cambia y la app de todos dejará de conectar. Tendrás que crear Perfiles nuevos.
