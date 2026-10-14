//! Terminal: abrirla, escribirle, cambiar su tamaño y cerrarla.

use sshpanel_core::{Core, TerminalEvent};
use tauri::State;
use tauri::ipc::{Channel, Response};

use super::code;

/// Abre la Terminal. Lo que escribe el Servidor llega por `on_output` como
/// bytes, sin pasar por JSON; el fin de la sesión, por `on_closed`.
#[tauri::command]
pub async fn open_terminal(
    cols: u32,
    rows: u32,
    on_output: Channel<Response>,
    on_closed: Channel<()>,
    core: State<'_, Core>,
) -> Result<(), String> {
    let on_event = move |event| match event {
        TerminalEvent::Output(data) => {
            let _ = on_output.send(Response::new(data));
        }
        TerminalEvent::Closed => {
            let _ = on_closed.send(());
        }
    };
    core.open_terminal(cols, rows, on_event).await.map_err(code)
}

#[tauri::command]
pub async fn terminal_input(data: String, core: State<'_, Core>) -> Result<(), String> {
    core.terminal_input(data.as_bytes()).await.map_err(code)
}

#[tauri::command]
pub async fn resize_terminal(cols: u32, rows: u32, core: State<'_, Core>) -> Result<(), String> {
    core.resize_terminal(cols, rows).await.map_err(code)
}

#[tauri::command]
pub async fn close_terminal(core: State<'_, Core>) -> Result<(), String> {
    core.close_terminal().await;
    Ok(())
}
