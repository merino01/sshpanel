# Installation guide

*[Versión en español](guia-instalacion.md)*

How to install the app and sign in for the first time. You need the profile file your administrator sent you.

Download the installer for your system from the project's Releases page. The app isn't signed by Microsoft or Apple, because those certificates cost money every year. That's why your system will warn you that it comes from an "unknown developer". This is normal: here's how to open it anyway.

## Windows

1. Open the installer (`.msi` or `.exe`).
2. If "Windows protected your PC" appears, click **More info** and then **Run anyway**.
3. Follow the installer's steps.

## macOS

1. Open the `.dmg` and drag the app into **Applications**.
2. Open it from Applications. If it says the **developer cannot be verified**:
   1. Click **OK** (or **Done**).
   2. Open **System Settings → Privacy & Security**.
   3. Scroll down to the message about the app and click **Open Anyway**.
   4. Confirm with your password and click **Open**.
3. If it says instead that the app **is damaged** and offers to move it to the Bin, it isn't damaged: it's the same warning, only blunter. Open the **Terminal** app and type this, replacing the name with the app's if it's different:

   ```sh
   xattr -dr com.apple.quarantine "/Applications/SSH Panel.app"
   ```

   Then open it as usual.

You only need to do this the first time.

## Linux

- **`.deb`** (Debian, Ubuntu and derivatives): open it with the software installer, or in a terminal:

  ```sh
  sudo apt install ./file-name.deb
  ```

- **`.AppImage`** (any distribution): make it executable and open it:

  ```sh
  chmod +x file-name.AppImage
  ./file-name.AppImage
  ```

## First steps

1. Double-click the profile file your administrator sent you, or open the app, click **Import profile** and choose it. Double-clicking works if you installed with the Windows installer, the macOS `.dmg` or the Linux `.deb`/`.rpm`; with the `.AppImage`, use **Import profile**.
2. The app creates your key and shows it to you. Click **Copy my public key** and send it to the administrator. It's safe to share: the secret part never leaves your computer.
3. When the administrator tells you that you have access, click **Retry**.

From then on, the app signs in directly when you open it and lets you know when there's a new version: click **Update** and it restarts already updated. If you're sent another profile later, double-click it: the app will ask before switching.

The app follows your computer's language if it's Spanish or English, and uses English otherwise. You can change it in **Settings → Language**.
