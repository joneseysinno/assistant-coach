//! The store interface in domain terms (S14 §7, D-017). Studio code saves and loads
//! documents through this trait; only `ac-store-idb` knows about InfiniteDB.

use std::collections::BTreeMap;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};

use crate::model::{Game, Play, Practice};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DocKind {
    Game,
    Practice,
    Play,
}

impl DocKind {
    pub const ALL: [DocKind; 3] = [DocKind::Game, DocKind::Practice, DocKind::Play];
    /// S14 §2 kind ordinals.
    pub fn ordinal(self) -> u32 {
        match self {
            DocKind::Game => 1,
            DocKind::Practice => 7,
            DocKind::Play => 5,
        }
    }
    pub fn from_ordinal(n: u32) -> Option<DocKind> {
        DocKind::ALL.into_iter().find(|k| k.ordinal() == n)
    }
    pub fn as_str(self) -> &'static str {
        match self {
            DocKind::Game => "game",
            DocKind::Practice => "practice",
            DocKind::Play => "play",
        }
    }
}

/// One stored document. (Variants differ in size; documents are few and cloned rarely.)
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "body", rename_all = "lowercase")]
pub enum Doc {
    Game(Game),
    Practice(Practice),
    Play(Play),
}

impl Doc {
    pub fn kind(&self) -> DocKind {
        match self {
            Doc::Game(_) => DocKind::Game,
            Doc::Practice(_) => DocKind::Practice,
            Doc::Play(_) => DocKind::Play,
        }
    }
    pub fn id(&self) -> &str {
        match self {
            Doc::Game(g) => &g.id,
            Doc::Practice(p) => &p.id,
            Doc::Play(p) => &p.id,
        }
    }
    pub fn name(&self) -> String {
        match self {
            Doc::Game(g) => g.name.clone(),
            Doc::Practice(p) => p.title(),
            Doc::Play(p) => p.name.clone(),
        }
    }
}

#[derive(Debug)]
pub struct StoreError(pub String);

impl std::fmt::Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for StoreError {}

pub type StoreResult<T> = Result<T, StoreError>;

pub trait Store: Send + Sync {
    /// Insert or replace (by kind + id). Durable when it returns.
    fn save(&self, doc: &Doc) -> StoreResult<()>;
    fn load(&self, kind: DocKind, id: &str) -> StoreResult<Option<Doc>>;
    /// Every document of a kind (or all), sorted by name.
    fn list(&self, kind: Option<DocKind>) -> StoreResult<Vec<Doc>>;
    fn delete(&self, kind: DocKind, id: &str) -> StoreResult<()>;
}

/// In-memory store for tests and previews.
#[derive(Default)]
pub struct MemStore {
    docs: Mutex<BTreeMap<(DocKind, String), Doc>>,
}

impl Store for MemStore {
    fn save(&self, doc: &Doc) -> StoreResult<()> {
        self.docs
            .lock()
            .unwrap()
            .insert((doc.kind(), doc.id().to_string()), doc.clone());
        Ok(())
    }
    fn load(&self, kind: DocKind, id: &str) -> StoreResult<Option<Doc>> {
        Ok(self
            .docs
            .lock()
            .unwrap()
            .get(&(kind, id.to_string()))
            .cloned())
    }
    fn list(&self, kind: Option<DocKind>) -> StoreResult<Vec<Doc>> {
        let mut v: Vec<Doc> = self
            .docs
            .lock()
            .unwrap()
            .values()
            .filter(|d| kind.is_none_or(|k| d.kind() == k))
            .cloned()
            .collect();
        v.sort_by_key(|d| d.name().to_lowercase());
        Ok(v)
    }
    fn delete(&self, kind: DocKind, id: &str) -> StoreResult<()> {
        self.docs.lock().unwrap().remove(&(kind, id.to_string()));
        Ok(())
    }
}

/// The contract every `Store` must meet (S14 §7: "both pass the same contract tests").
/// Call from each implementation's tests.
pub fn contract(store: &dyn Store) {
    let g = Game {
        id: "g-contract".into(),
        name: "Contract game".into(),
        stage: "team-based".into(),
        ..Default::default()
    };
    let p = Practice {
        id: "pr-contract".into(),
        name: "Contract practice".into(),
        planned_minutes: 90.0,
        ..Default::default()
    };
    store.save(&Doc::Game(g.clone())).unwrap();
    store.save(&Doc::Practice(p.clone())).unwrap();
    assert_eq!(
        store.load(DocKind::Game, "g-contract").unwrap(),
        Some(Doc::Game(g.clone()))
    );
    assert_eq!(
        store.load(DocKind::Play, "g-contract").unwrap(),
        None,
        "kinds are separate"
    );
    // replace
    let mut g2 = g.clone();
    g2.name = "Renamed".into();
    store.save(&Doc::Game(g2.clone())).unwrap();
    assert_eq!(
        store.load(DocKind::Game, "g-contract").unwrap(),
        Some(Doc::Game(g2))
    );
    assert_eq!(store.list(Some(DocKind::Game)).unwrap().len(), 1);
    assert_eq!(store.list(None).unwrap().len(), 2);
    // delete
    store.delete(DocKind::Game, "g-contract").unwrap();
    assert_eq!(store.load(DocKind::Game, "g-contract").unwrap(), None);
    assert_eq!(store.list(None).unwrap().len(), 1);
    store.delete(DocKind::Practice, "pr-contract").unwrap();
}

#[cfg(test)]
mod tests {
    #[test]
    fn mem_store_meets_the_contract() {
        super::contract(&super::MemStore::default());
    }
}
