//! Assistant Coach
//!
//! The layout (title bar, rails, panels, palette) comes from `app-shell`.
//! Everything app-specific lives in `spec()` and the `docs` modules.

use app_shell::prelude::*;
use dioxus::prelude::*;

mod docs;

fn main() {
    #[cfg(feature = "desktop")]
    {
        use dioxus::desktop::{Config, LogicalSize, WindowBuilder};
        let window = WindowBuilder::new()
            .with_title("Assistant Coach")
            .with_inner_size(LogicalSize::new(1440.0, 900.0));
        dioxus::LaunchBuilder::desktop()
            .with_cfg(Config::new().with_window(window).with_menu(None))
            .launch(App);
    }
    #[cfg(not(feature = "desktop"))]
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        style { {docs::APP_CSS} }
        AppShell { spec }
    }
}

/// The whole app, described as data.
fn spec() -> ShellSpec {
    let mut s = ShellSpec::new("Assistant Coach");
    s.title_actions = vec![
        Tool::new("app.help", "Help", icons::HELP),
        Tool::new("app.account", "Account", icons::USER),
    ];
    s.nav_rail = vec![
        RailItem::panel("nav.docs", "Documents", icons::FILES, DocumentsPane),
        RailItem::panel("nav.settings", "Settings", icons::SLIDERS, SettingsPane).at_end(),
    ];
    s.default_nav = Some("nav.docs");
    s.doc_types = vec![docs::example::doc_type()];
    s.documents = vec![Document::new("d1", "Untitled", docs::example::ID)];
    s.on_command = Some(docs::on_command);
    s
}
