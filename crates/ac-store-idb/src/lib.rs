//! # ac-store-idb
//!
//! `Store` implementation on InfiniteDB (S14). This is the only crate that depends on
//! `infinite-db` (D-017), so an InfiniteDB upgrade is "make the contract tests pass here".
//!
//! Pinned to git commit `d193986` (0.4.1, unpublished on crates.io) — see D-037.
//! `tests/idb_api_surface.rs` is the T11a compile-and-behaviour test for every
//! InfiniteDB API S14 relies on.
//!
//! Filled in by P1 tasks T11 (entities + edges) and T12 (live log on a store thread, D-036).

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
