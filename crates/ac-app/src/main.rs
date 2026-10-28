//! Assistant Coach — Studio.
//!
//! Design small-sided games (Game), plan practices from them (Practice), and draw plays as
//! decision graphs (Play). The layout comes from the JoInn `app-shell`; everything
//! app-specific is in `spec()`, the `docs` modules and the panes.

use std::collections::HashSet;
use std::sync::Mutex;

use app_shell::prelude::*;
use dioxus::prelude::*;

mod court;
mod docs;
mod library;
mod seed;
mod settings;
mod store;
mod ui;

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

/// Ids live as long as the app; interning makes them `Copy` for event handlers.
pub fn intern(s: &str) -> &'static str {
    static SET: Mutex<Option<HashSet<&'static str>>> = Mutex::new(None);
    let mut guard = SET.lock().unwrap();
    let set = guard.get_or_insert_with(HashSet::new);
    if let Some(x) = set.get(s) {
        return x;
    }
    let leaked: &'static str = Box::leak(s.to_string().into_boxed_str());
    set.insert(leaked);
    leaked
}

#[component]
fn App() -> Element {
    use_hook(|| {
        store::init();
        store::refresh_library();
        settings::load();
    });
    rsx! {
        style { {docs::APP_CSS} }
        AppShell { spec }
    }
}

/// The whole app, described as data.
fn spec() -> ShellSpec {
    let mut s = ShellSpec::new("Assistant Coach");
    s.context_label = Some("Studio");
    s.title_actions = vec![
        Tool::new("app.new-game", "New game", icons::TARGET),
        Tool::new("app.new-practice", "New practice", icons::TABLE),
        Tool::new("app.new-play", "New play", icons::PLAY),
    ];
    s.nav_rail = vec![
        RailItem::panel(
            "nav.library",
            "Library",
            icons::FOLDER,
            library::LibraryPane,
        ),
        RailItem::panel("nav.docs", "Open documents", icons::FILES, DocumentsPane),
        RailItem::panel(
            "nav.settings",
            "Settings",
            icons::SLIDERS,
            settings::SettingsPane,
        )
        .at_end(),
    ];
    s.default_nav = Some("nav.library");
    s.doc_types = vec![
        docs::game::doc_type(),
        docs::practice::doc_type(),
        docs::play::doc_type(),
    ];
    s.on_command = Some(docs::on_command);
    s
}
