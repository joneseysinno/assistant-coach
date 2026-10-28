//! App settings (03 §2 Settings pane): court lines, where files go. Stored as JSON next to
//! the database.

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

use crate::ui::*;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Settings {
    /// Court preset whose lines are painted in your gym (S3 §3).
    pub court: String,
}

impl Default for Settings {
    fn default() -> Self {
        Settings { court: "hs".into() }
    }
}

pub static SETTINGS: GlobalSignal<Settings> = Signal::global(Settings::default);

fn path() -> std::path::PathBuf {
    crate::store::data_dir().join("settings.json")
}

pub fn load() {
    if let Ok(text) = std::fs::read_to_string(path()) {
        if let Ok(s) = serde_json::from_str(&text) {
            *SETTINGS.write() = s;
        }
    }
}

fn save(s: &Settings) {
    let _ = std::fs::create_dir_all(crate::store::data_dir());
    let _ = std::fs::write(path(), serde_json::to_string_pretty(s).unwrap_or_default());
}

/// The shell reuses one scope for whatever view sits in a slot (a lone keyed child),
/// so this wrapper has no hooks and renders a distinct component type.
#[allow(non_snake_case)]
pub fn SettingsPane() -> Element {
    rsx! { SettingsPaneBody {} }
}

#[component]
fn SettingsPaneBody() -> Element {
    let s = SETTINGS.read().clone();
    let presets: Vec<(String, String)> = ac_sport_basketball::court::PRESETS
        .iter()
        .map(|c| (c.id.to_string(), c.label.to_string()))
        .collect();
    let db = crate::store::data_dir().join("db");
    let exports = crate::docs::export_dir();
    rsx! {
        div { class: "app-pane app-form",
            h3 { class: "app-pane-title", "Settings" }
            Field { label: "Court lines in your gym", hint: "Used for zones, space per player and printed cards (S3 §3)",
                Pick { value: s.court.clone(), options: presets, on_change: move |v: String| {
                    let mut n = SETTINGS.read().clone();
                    n.court = v;
                    save(&n);
                    *SETTINGS.write() = n;
                } }
            }
            Field { label: "Theme",
                button { class: "app-btn app-sm", onclick: move |_| app_shell::use_shell().run(app_shell::cmd::TOGGLE_THEME), "Toggle light / dark" }
            }
            Field { label: "Database (InfiniteDB)", p { class: "app-mono app-small", "{db.display()}" } }
            Field { label: "Printed cards and plans go to", p { class: "app-mono app-small", "{exports.display()}" } }
            p { class: "app-muted app-small", "Everything saves as you edit. Undo with Ctrl+Z." }
        }
    }
}
