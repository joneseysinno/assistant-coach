//! App-side document types. Add a module per `DocType` and register it in
//! `spec()` in `main.rs`.

use app_shell::prelude::*;

pub mod example;

pub const APP_CSS: &str = include_str!("app.css");

/// Every command from every document type lands here (after the shell
/// has handled its own built-ins). Route by the id prefix of each doc type.
pub fn on_command(_shell: Shell, ev: &CommandEvent) {
    let Some(doc) = ev.doc.clone() else { return };
    if ev.id.starts_with("ex.") {
        example::handle(&doc, ev.id);
    }
}
