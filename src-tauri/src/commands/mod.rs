//! Comandos que la interfaz invoca, por área. Los errores llegan a la interfaz
//! como el código estable de [`sshpanel_core::Error::code`].

pub mod admin;
pub mod editor;
pub mod files;
pub mod preferences;
pub mod profile;
pub mod terminal;
pub mod transfers;
pub mod trash;
pub mod update;

fn code(err: sshpanel_core::Error) -> String {
    err.code().to_string()
}
