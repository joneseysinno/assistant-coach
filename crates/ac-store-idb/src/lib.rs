//! # ac-store-idb
//!
//! `Store` implementation on InfiniteDB (S14). This is the only crate that depends on
//! `infinite-db` (D-017), so an InfiniteDB upgrade is "make the contract tests pass here".
//!
//! Pinned to crates.io `infinite-db = "=0.4.0"` — see D-037. 0.4.0 has no
//! `register_or_get_space`; "open or register" treats `SpaceError::DuplicateId` as
//! "already registered" (spaces persist across reopen).
//! `tests/idb_api_surface.rs` is the T11a compile-and-behaviour test for every
//! InfiniteDB API S14 relies on.
//!
//! **Layout (S14 §2–3, D-024, D-038):** each document is one record in `ac.entity` at
//! `[kind_ordinal, serial]`. The serial is minted here and means nothing; the domain id
//! (`g-closeout-attack-3v3`) lives in the envelope. Bodies are JSON (not bincode) while
//! the model is still changing — new optional fields load from old records (D-038).
//! On open the whole entity space is read once (a full `query`, which also sidesteps
//! IDB-B1) into an index; every `save` is `insert` + `sync` before it returns.

use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;

use ac_domain::store::{Doc, DocKind, Store, StoreError, StoreResult};
use infinite_db::infinitedb_core::address::{DimensionVector, SpaceId};
use infinite_db::infinitedb_core::space::{SpaceConfig, SpaceError};
use infinite_db::{EngineError, InfiniteDb};
use serde::{Deserialize, Serialize};

/// The InfiniteDB space ids registered by Assistant Coach (S14 §2).
pub mod spaces {
    /// `[kind_ordinal, serial]` → versioned envelope of one domain entity.
    pub const ENTITY: u64 = 100;
    /// Hyperedges (relationships, stack order in metadata).
    pub const EDGES: u64 = 101;
    /// `[x, y, layer]` derived spatial index rows.
    pub const COURT: u64 = 102;
    /// `[session_serial, seq]` → one live log entry.
    pub const LOG: u64 = 103;
    /// `[key]` → schema version, id counters.
    pub const META: u64 = 104;

    /// Bits per dimension for every Assistant Coach space. InfiniteDB's default is 8,
    /// which caps a coordinate at 255 — far too small for serials and log sequence
    /// numbers. 32 bits fits `dims * bits <= 128` for every space we register (max 3 dims).
    pub const BITS_PER_DIM: u32 = 32;
}

/// Current envelope schema (S14 §3).
pub const SCHEMA: u16 = 1;

#[derive(Serialize, Deserialize)]
struct Envelope {
    schema: u16,
    doc: Doc,
}

fn err(e: impl std::fmt::Debug) -> StoreError {
    StoreError(format!("{e:?}"))
}

/// Register a space, or accept it if it already exists (0.4.0 has no idempotent register).
pub fn open_or_register(db: &InfiniteDb, id: u64, name: &str, dims: usize) -> StoreResult<()> {
    let config = SpaceConfig::new(SpaceId(id), name, dims).with_bits_per_dim(spaces::BITS_PER_DIM);
    match db.register_space(config) {
        Ok(()) => Ok(()),
        Err(EngineError::RegistrySpace(SpaceError::DuplicateId(_))) => Ok(()),
        Err(e) => Err(err(e)),
    }
}

struct Index {
    docs: HashMap<(DocKind, String), (u32, Doc)>,
    next_serial: u32,
}

pub struct IdbStore {
    db: InfiniteDb,
    index: Mutex<Index>,
}

const ENTITY: SpaceId = SpaceId(spaces::ENTITY);

impl IdbStore {
    pub fn open(dir: impl AsRef<Path>) -> StoreResult<Self> {
        std::fs::create_dir_all(dir.as_ref()).map_err(err)?;
        let db = InfiniteDb::open(dir.as_ref()).map_err(err)?;
        open_or_register(&db, spaces::ENTITY, "ac.entity", 2)?;
        let mut docs = HashMap::new();
        let mut next_serial = 1;
        for rec in db.query(ENTITY, None).map_err(err)? {
            if rec.tombstone || rec.data.is_empty() {
                continue;
            }
            let coords = &rec.address.point.coords;
            let serial = coords.get(1).copied().unwrap_or(0);
            next_serial = next_serial.max(serial + 1);
            match serde_json::from_slice::<Envelope>(&rec.data) {
                Ok(env) => {
                    docs.insert(
                        (env.doc.kind(), env.doc.id().to_string()),
                        (serial, env.doc),
                    );
                }
                Err(e) => eprintln!("ac-store-idb: skipping unreadable record {coords:?}: {e}"),
            }
        }
        Ok(IdbStore {
            db,
            index: Mutex::new(Index { docs, next_serial }),
        })
    }

    pub fn len(&self) -> usize {
        self.index.lock().unwrap().docs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Store for IdbStore {
    fn save(&self, doc: &Doc) -> StoreResult<()> {
        let mut ix = self.index.lock().unwrap();
        let key = (doc.kind(), doc.id().to_string());
        let serial = match ix.docs.get(&key) {
            Some((s, _)) => *s,
            None => {
                let s = ix.next_serial;
                ix.next_serial += 1;
                s
            }
        };
        let body = serde_json::to_vec(&Envelope {
            schema: SCHEMA,
            doc: doc.clone(),
        })
        .map_err(err)?;
        let at = DimensionVector::new(vec![doc.kind().ordinal(), serial]);
        self.db.insert(ENTITY, at, body).map_err(err)?;
        self.db.sync().map_err(err)?;
        ix.docs.insert(key, (serial, doc.clone()));
        Ok(())
    }

    fn load(&self, kind: DocKind, id: &str) -> StoreResult<Option<Doc>> {
        Ok(self
            .index
            .lock()
            .unwrap()
            .docs
            .get(&(kind, id.to_string()))
            .map(|(_, d)| d.clone()))
    }

    fn list(&self, kind: Option<DocKind>) -> StoreResult<Vec<Doc>> {
        let mut v: Vec<Doc> = self
            .index
            .lock()
            .unwrap()
            .docs
            .values()
            .map(|(_, d)| d)
            .filter(|d| kind.is_none_or(|k| d.kind() == k))
            .cloned()
            .collect();
        v.sort_by_key(|d| d.name().to_lowercase());
        Ok(v)
    }

    fn delete(&self, kind: DocKind, id: &str) -> StoreResult<()> {
        let mut ix = self.index.lock().unwrap();
        if let Some((serial, _)) = ix.docs.remove(&(kind, id.to_string())) {
            let at = DimensionVector::new(vec![kind.ordinal(), serial]);
            self.db.delete(ENTITY, at).map_err(err)?;
            self.db.sync().map_err(err)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn space_ids_are_distinct() {
        use super::spaces::*;
        let ids = [ENTITY, EDGES, COURT, LOG, META];
        for (i, a) in ids.iter().enumerate() {
            for b in &ids[i + 1..] {
                assert_ne!(a, b);
            }
        }
    }
}
