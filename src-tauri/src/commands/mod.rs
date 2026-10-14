//! Comandos que la interfaz invoca, por área. Los errores llegan a la interfaz
//! como el código estable de [`sshpanel_core::Error::code`].

pub mod files;
pub mod profile;
pub mod transfers;
pub mod trash;

fn code(err: sshpanel_core::Error) -> String {
    err.code().to_string()
}
