#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // WebKitGTK falla con "Error 71 dispatching to Wayland display" en algunos
    // compositores; desactivar el renderizador DMABUF lo evita.
    #[cfg(target_os = "linux")]
    if std::env::var_os("WEBKIT_DISABLE_DMABUF_RENDERER").is_none() {
        // SAFETY: se ejecuta al inicio de `main`, antes de crear ningún hilo.
        unsafe { std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1") };
    }
    sshpanel::run();
}
