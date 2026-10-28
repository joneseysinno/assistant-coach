//! Document types and the plumbing they share: per-document state with undo/redo and
//! autosave, opening documents from the Library, and command routing.

use std::collections::HashMap;
use std::time::Instant;

use ac_domain::store::{Doc, DocKind};
use app_shell::prelude::*;
use dioxus::prelude::*;

pub mod game;
pub mod play;
pub mod practice;

pub const APP_CSS: &str = include_str!("app.css");

/// One open document: its value, undo/redo stacks and edit coalescing.
#[derive(Clone)]
pub struct Editing<T> {
    pub value: T,
    undo: Vec<T>,
    redo: Vec<T>,
    last_key: Option<(String, Instant)>,
}

impl<T: Clone + PartialEq> Editing<T> {
    pub fn new(value: T) -> Self {
        Editing {
            value,
            undo: Vec::new(),
            redo: Vec::new(),
            last_key: None,
        }
    }

    /// Apply `f`; typing in the same field within 1.5 s is one undo step.
    pub fn edit(&mut self, key: &str, f: impl FnOnce(&mut T)) -> bool {
        let before = self.value.clone();
        f(&mut self.value);
        if self.value == before {
            return false;
        }
        let coalesce = self
            .last_key
            .as_ref()
            .is_some_and(|(k, t)| k == key && t.elapsed().as_millis() < 1500 && !key.is_empty());
        if !coalesce {
            self.undo.push(before);
            if self.undo.len() > 200 {
                self.undo.remove(0);
            }
        }
        self.redo.clear();
        self.last_key = Some((key.to_string(), Instant::now()));
        true
    }

    pub fn undo(&mut self) -> bool {
        match self.undo.pop() {
            Some(prev) => {
                self.redo.push(std::mem::replace(&mut self.value, prev));
                self.last_key = None;
                true
            }
            None => false,
        }
    }

    pub fn redo(&mut self) -> bool {
        match self.redo.pop() {
            Some(next) => {
                self.undo.push(std::mem::replace(&mut self.value, next));
                self.last_key = None;
                true
            }
            None => false,
        }
    }
}

/// Ctrl+Z / Ctrl+Y (Ctrl+Shift+Z) undo and redo the active document. The shell only
/// matches single-key shortcuts, and text fields keep their own native undo, so this
/// listens in the page and ignores keys pressed inside inputs.
pub fn use_undo_keys() {
    use std::sync::atomic::{AtomicBool, Ordering};
    static INSTALLED: AtomicBool = AtomicBool::new(false);
    let shell = use_shell();
    use_hook(move || {
        if INSTALLED.swap(true, Ordering::SeqCst) {
            return;
        }
        spawn(async move {
            let mut ev = document::eval(
                r#"
                document.addEventListener('keydown', (e) => {
                  if (!(e.ctrlKey || e.metaKey)) return;
                  const k = e.key.toLowerCase();
                  if (k !== 'z' && k !== 'y') return;
                  const t = (e.target && e.target.tagName) || '';
                  if (t === 'INPUT' || t === 'TEXTAREA' || t === 'SELECT') return;
                  e.preventDefault();
                  dioxus.send((k === 'y' || e.shiftKey) ? 'redo' : 'undo');
                });
                "#,
            );
            while let Ok(what) = ev.recv::<String>().await {
                let mut sh = shell;
                let Some(kind) = sh.active_type().map(|t| t.id) else {
                    continue;
                };
                let id: &'static str = match (kind, what.as_str()) {
                    (game::ID, "undo") => "gm.undo",
                    (game::ID, _) => "gm.redo",
                    (practice::ID, "undo") => "pr.undo",
                    (practice::ID, _) => "pr.redo",
                    (play::ID, "undo") => "pl.undo",
                    (play::ID, _) => "pl.redo",
                    _ => continue,
                };
                sh.run(id);
            }
        });
    });
}

/// Shell document id for a stored document.
pub fn shell_kind(kind: DocKind) -> &'static str {
    match kind {
        DocKind::Game => game::ID,
        DocKind::Practice => practice::ID,
        DocKind::Play => play::ID,
    }
}

/// Open (or switch to) a stored document.
pub fn open(mut shell: Shell, kind: DocKind, id: &str, name: &str) {
    let open = shell.documents.read().iter().any(|d| d.id == id);
    if !open {
        shell
            .documents
            .write()
            .push(Document::new(id, name, shell_kind(kind)));
    }
    shell.activate(id);
}

/// Keep the shell's tab name in step with the document's name.
pub fn rename(mut shell: Shell, id: &str, name: &str) {
    let mut docs = shell.documents.write();
    if let Some(d) = docs.iter_mut().find(|d| d.id == id) {
        if d.name != name {
            d.name = name.to_string();
        }
    }
}

/// Documents created by the shell's own "+" menu get ids like `game-1000`, which could
/// collide with stored ids from an earlier session. Swap in a fresh store id.
pub fn adopt_shell_id(mut shell: Shell, kind: DocKind, old: &str) -> Option<String> {
    let k = shell_kind(kind);
    let is_shell_id = old
        .strip_prefix(k)
        .and_then(|r| r.strip_prefix('-'))
        .is_some_and(|n| n.chars().all(|c| c.is_ascii_digit()));
    if !is_shell_id {
        return None;
    }
    let new = crate::store::new_id(kind);
    let mut docs = shell.documents.write();
    if let Some(d) = docs.iter_mut().find(|d| d.id == old) {
        *d = Document::new(new.clone(), d.name.clone(), k);
    }
    drop(docs);
    if shell.active.read().as_deref() == Some(old) {
        shell.active.set(Some(new.clone()));
    }
    Some(new)
}

/// Create a new stored document of `kind`, open it, and return its id.
pub fn create(shell: Shell, kind: DocKind) -> String {
    let id = crate::store::new_id(kind);
    let doc = match kind {
        DocKind::Game => Doc::Game(ac_sport_basketball::templates::new_game(&id)),
        DocKind::Practice => Doc::Practice(ac_sport_basketball::templates::new_practice(
            &id,
            &crate::store::today(),
        )),
        DocKind::Play => Doc::Play(ac_sport_basketball::templates::new_play(&id)),
    };
    let name = doc.name();
    crate::store::save(doc);
    open(shell, kind, &id, &name);
    id
}

/// Every command from every document type lands here (after the shell's built-ins).
pub fn on_command(shell: Shell, ev: &CommandEvent) {
    match ev.id {
        "app.new-game" => {
            create(shell, DocKind::Game);
            return;
        }
        "app.new-practice" => {
            create(shell, DocKind::Practice);
            return;
        }
        "app.new-play" => {
            create(shell, DocKind::Play);
            return;
        }
        _ => {}
    }
    let Some(doc) = ev.doc.clone() else { return };
    if ev.id.starts_with("gm.") {
        game::handle(shell, &doc, ev.id);
    } else if ev.id.starts_with("pr.") {
        practice::handle(shell, &doc, ev.id);
    } else if ev.id.starts_with("pl.") {
        play::handle(shell, &doc, ev.id);
    }
}

/// Per-kind state maps (the template's pattern: keyed by document id).
pub type Map<T> = GlobalSignal<HashMap<String, T>>;

/// Write the value of an `ac_domain::Doc` to the file system as YAML (export).
pub fn export_yaml(kind: DocKind, doc: &Doc) -> Option<std::path::PathBuf> {
    let text = match doc {
        Doc::Game(g) => ac_domain::to_yaml(g).ok()?,
        Doc::Practice(p) => ac_domain::to_yaml(p).ok()?,
        Doc::Play(p) => ac_domain::to_yaml(p).ok()?,
    };
    let dir = export_dir();
    let path = dir.join(format!("{}-{}.yaml", kind.as_str(), doc.id()));
    std::fs::write(&path, text).ok()?;
    Some(path)
}

pub fn export_dir() -> std::path::PathBuf {
    let dir = dirs::document_dir()
        .unwrap_or_else(crate::store::data_dir)
        .join("Assistant Coach");
    let _ = std::fs::create_dir_all(&dir);
    dir
}

/// Write an HTML page and open it in the default browser (for printing).
pub fn export_html(name: &str, html: String) -> Option<std::path::PathBuf> {
    let safe: String = name
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' {
                c
            } else {
                '-'
            }
        })
        .collect();
    let path = export_dir().join(format!("{safe}.html"));
    std::fs::write(&path, html).ok()?;
    if std::env::var("AC_NO_OPEN").is_err() {
        let _ = open::that(&path);
    }
    Some(path)
}

/// Shared print stylesheet for exported pages.
pub const PRINT_CSS: &str = r#"
body{font-family:system-ui,-apple-system,"Segoe UI",sans-serif;color:#1c1e22;margin:24px;max-width:900px}
h1{font-size:24px;margin:0 0 4px} h2{font-size:18px;margin:18px 0 6px} h3{font-size:14px;margin:12px 0 4px;text-transform:uppercase;letter-spacing:.04em;color:#555}
.sub{color:#5d626c;margin:0 0 12px} ul{margin:4px 0 8px 18px;padding:0} li{margin:2px 0}
table{border-collapse:collapse;width:100%;margin:8px 0} td,th{border-bottom:1px solid #ddd;padding:6px 8px;text-align:left;vertical-align:top;font-size:14px}
.card{border:1px solid #ccc;border-radius:8px;padding:12px 16px;margin:14px 0;page-break-inside:avoid;display:grid;grid-template-columns:300px 1fr;gap:16px}
.board{font-family:ui-monospace,Consolas,monospace;font-size:13px;background:#f3f4f6;padding:4px 8px;border-radius:4px;display:inline-block}
.muted{color:#777} @media print{body{margin:10mm}.noprint{display:none}}
"#;

pub fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
