//! The Studio's connection to storage (D-017: only through the `Store` trait) and the
//! library index the panes read.

use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

use ac_domain::store::{Doc, DocKind, Store};
use ac_domain::{Game, Play, Practice};
use ac_store_idb::IdbStore;
use dioxus::prelude::*;

static STORE: OnceLock<Box<dyn Store>> = OnceLock::new();

/// Where the database lives: `AC_DATA_DIR`, else the OS data folder.
pub fn data_dir() -> std::path::PathBuf {
    if let Ok(d) = std::env::var("AC_DATA_DIR") {
        return d.into();
    }
    dirs::data_local_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join("AssistantCoach")
}

/// Open the database (once) and seed it with the fixture library on first run.
pub fn init() {
    STORE.get_or_init(|| {
        let dir = data_dir().join("db");
        match IdbStore::open(&dir) {
            Ok(s) => {
                if s.is_empty() {
                    crate::seed::seed(&s);
                }
                Box::new(s) as Box<dyn Store>
            }
            Err(e) => {
                eprintln!("Could not open the database at {dir:?}: {e}. Using memory only.");
                Box::new(ac_domain::MemStore::default())
            }
        }
    });
}

pub fn store() -> &'static dyn Store {
    init();
    STORE.get().unwrap().as_ref()
}

/// One row in the Library.
#[derive(Clone, Debug, PartialEq)]
pub struct Entry {
    pub kind: DocKind,
    pub id: String,
    pub name: String,
    pub detail: String,
    pub targets: Vec<String>,
    pub reads: Vec<String>,
}

pub static LIBRARY: GlobalSignal<Vec<Entry>> = Signal::global(Vec::new);
/// Bumped on every save, so views that read other documents refresh.
pub static REVISION: GlobalSignal<u64> = Signal::global(|| 0);

fn entry(d: &Doc) -> Entry {
    match d {
        Doc::Game(g) => Entry {
            kind: DocKind::Game,
            id: g.id.clone(),
            name: g.name.clone(),
            detail: format!(
                "{} · {} · {}",
                g.numbers(),
                ac_sport_basketball::vocab::region_label(&g.format.space.court),
                g.stage
            ),
            targets: g.targets(),
            reads: g.reads.clone(),
        },
        Doc::Practice(p) => Entry {
            kind: DocKind::Practice,
            id: p.id.clone(),
            name: p.title(),
            detail: format!(
                "{} · {} blocks · {} min",
                if p.date.is_empty() {
                    "no date"
                } else {
                    &p.date
                },
                p.blocks.len(),
                p.planned_minutes
            ),
            targets: vec![],
            reads: vec![],
        },
        Doc::Play(p) => Entry {
            kind: DocKind::Play,
            id: p.id.clone(),
            name: p.name.clone(),
            detail: format!(
                "{} · {} states · {} reads",
                p.kind,
                p.states.len(),
                ac_domain::play::reads(p).len()
            ),
            targets: vec![],
            reads: ac_domain::play::reads(p),
        },
    }
}

pub fn refresh_library() {
    let docs = store().list(None).unwrap_or_default();
    *LIBRARY.write() = docs.iter().map(entry).collect();
}

pub fn save(doc: Doc) {
    if let Err(e) = store().save(&doc) {
        eprintln!("save failed: {e}");
    }
    let e = entry(&doc);
    {
        let mut lib = LIBRARY.write();
        match lib.iter_mut().find(|x| x.kind == e.kind && x.id == e.id) {
            Some(x) => *x = e,
            None => lib.push(e),
        }
        lib.sort_by_key(|x| x.name.to_lowercase());
    }
    *REVISION.write() += 1;
}

pub fn delete(kind: DocKind, id: &str) {
    let _ = store().delete(kind, id);
    LIBRARY.write().retain(|e| !(e.kind == kind && e.id == id));
    *REVISION.write() += 1;
}

pub fn load_game(id: &str) -> Option<Game> {
    match store().load(DocKind::Game, id).ok()?? {
        Doc::Game(g) => Some(g),
        _ => None,
    }
}

pub fn load_practice(id: &str) -> Option<Practice> {
    match store().load(DocKind::Practice, id).ok()?? {
        Doc::Practice(p) => Some(p),
        _ => None,
    }
}

pub fn load_play(id: &str) -> Option<Play> {
    match store().load(DocKind::Play, id).ok()?? {
        Doc::Play(p) => Some(p),
        _ => None,
    }
}

pub fn all_games() -> Vec<Game> {
    store()
        .list(Some(DocKind::Game))
        .unwrap_or_default()
        .into_iter()
        .filter_map(|d| match d {
            Doc::Game(g) => Some(g),
            _ => None,
        })
        .collect()
}

/// A new unique document id: `g-`, `pr-`, `pl-` + a time-based suffix.
pub fn new_id(kind: DocKind) -> String {
    let prefix = match kind {
        DocKind::Game => "g",
        DocKind::Practice => "pr",
        DocKind::Play => "pl",
    };
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let mut id = format!("{prefix}-{t:x}");
    let mut n = 1;
    while store().load(kind, &id).ok().flatten().is_some() {
        n += 1;
        id = format!("{prefix}-{t:x}-{n}");
    }
    id
}

/// Today's date as YYYY-MM-DD (UTC; good enough for a default).
pub fn today() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let days = secs.div_euclid(86_400);
    // civil-from-days (Howard Hinnant)
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    format!("{}-{:02}-{:02}", if m <= 2 { y + 1 } else { y }, m, d)
}
