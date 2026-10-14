//! Terminal: una consola interactiva en el Servidor, sobre un canal SSH con
//! PTY de la misma conexión que usa el Explorador. Solo en Modo avanzado.

use std::sync::Arc;

use russh::client::Msg;
use russh::{Channel, ChannelMsg, ChannelWriteHalf};
use tokio::sync::Notify;

use super::Connection;
use crate::error::{Error, Result};

/// Tipo de terminal que se anuncia al Servidor; es lo que emula xterm.js.
const TERM: &str = "xterm-256color";

/// Lo que llega de la Terminal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TerminalEvent {
    /// Lo que escribe el Servidor, tal cual (puede cortar un carácter UTF-8).
    Output(Vec<u8>),
    /// La sesión ha terminado: el Miembro salió o se perdió la conexión.
    Closed,
}

/// Terminal abierta: por aquí se le escribe; lo que responde llega por el
/// aviso que se dio al abrirla.
pub(crate) struct Terminal {
    writer: ChannelWriteHalf<Msg>,
    /// Para la lectura al cerrar desde aquí: russh no lo avisa por el canal.
    closed: Arc<Notify>,
}

impl Connection {
    pub(crate) async fn open_terminal(
        &self,
        cols: u32,
        rows: u32,
        mut on_event: impl FnMut(TerminalEvent) + Send + 'static,
    ) -> Result<Terminal> {
        self.check_advanced()?;
        let mut channel = self.ssh.channel_open_session().await.map_err(|_| self.lost_or(Error::TerminalFailed))?;
        channel.request_pty(true, TERM, cols, rows, 0, 0, &[]).await.map_err(|_| self.lost_or(Error::TerminalFailed))?;
        self.accepted(&mut channel).await?;
        channel.request_shell(true).await.map_err(|_| self.lost_or(Error::TerminalFailed))?;
        self.accepted(&mut channel).await?;

        let (mut reader, writer) = channel.split();
        let closed = Arc::new(Notify::new());
        let stop = closed.clone();
        tokio::spawn(async move {
            loop {
                let msg = tokio::select! {
                    msg = reader.wait() => msg,
                    () = stop.notified() => break,
                };
                match msg {
                    Some(ChannelMsg::Data { data }) | Some(ChannelMsg::ExtendedData { data, .. }) => {
                        on_event(TerminalEvent::Output(data.to_vec()))
                    }
                    Some(ChannelMsg::Eof | ChannelMsg::Close) | None => break,
                    Some(_) => {}
                }
            }
            on_event(TerminalEvent::Closed);
        });
        Ok(Terminal { writer, closed })
    }

    /// Espera la respuesta del Servidor a una petición sobre `channel`.
    async fn accepted(&self, channel: &mut Channel<Msg>) -> Result<()> {
        loop {
            match channel.wait().await {
                Some(ChannelMsg::Success) => return Ok(()),
                Some(ChannelMsg::Failure) => return Err(Error::TerminalFailed),
                Some(_) => {}
                None => return Err(self.lost_or(Error::TerminalFailed)),
            }
        }
    }
}

impl Terminal {
    /// Envía lo que teclea el Miembro.
    pub(crate) async fn input(&self, data: &[u8]) -> Result<()> {
        self.writer.data_bytes(data.to_vec()).await.map_err(|_| Error::ConnectionLost)
    }

    /// Avisa al Servidor del nuevo tamaño, en columnas y filas.
    pub(crate) async fn resize(&self, cols: u32, rows: u32) -> Result<()> {
        self.writer.window_change(cols, rows, 0, 0).await.map_err(|_| Error::ConnectionLost)
    }

    pub(crate) async fn close(&self) {
        self.closed.notify_one();
        let _ = self.writer.eof().await;
        let _ = self.writer.close().await;
    }
}
