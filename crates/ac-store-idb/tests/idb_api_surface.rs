//! T11a — InfiniteDB readiness: one test per behaviour S14 relies on, calling every API
//! listed in S14 §1.1 on the pinned version. If an InfiniteDB upgrade breaks S14's
//! assumptions, this file is where it shows up first.

use std::collections::BTreeMap;

use infinite_db::infinitedb_core::address::{DimensionVector, RevisionId, SpaceId};
use infinite_db::infinitedb_core::hyperedge::{
    Directionality, EndpointPolarity, EndpointRef, EndpointRole, Hyperedge, HyperedgeId,
    HyperedgeKind,
};
use infinite_db::infinitedb_core::query::DirectionFilter;
use infinite_db::infinitedb_core::space::{SpaceConfig, SpaceError};
use infinite_db::{EngineError, InfiniteDb, OpenOptions};
use tempfile::TempDir;

use ac_store_idb::spaces;

const ENTITY: SpaceId = SpaceId(spaces::ENTITY);
const EDGES: SpaceId = SpaceId(spaces::EDGES);
const LOG: SpaceId = SpaceId(spaces::LOG);

fn space(id: SpaceId, name: &str, dims: usize) -> SpaceConfig {
    SpaceConfig::new(id, name, dims).with_bits_per_dim(spaces::BITS_PER_DIM)
}

fn open() -> (InfiniteDb, TempDir) {
    let dir = TempDir::new().unwrap();
    let db = OpenOptions::default().open(dir.path()).unwrap();
    db.register_space(space(ENTITY, "ac.entity", 2)).unwrap();
    db.register_space(space(EDGES, "ac.edges", 2)).unwrap();
    db.register_space(space(LOG, "ac.log", 2)).unwrap();
    (db, dir)
}

fn at(kind: u32, serial: u32) -> DimensionVector {
    DimensionVector::new(vec![kind, serial])
}

fn ep(kind: u32, serial: u32, polarity: EndpointPolarity) -> EndpointRef {
    EndpointRef::new(EndpointRole::new("n"), ENTITY, at(kind, serial)).with_polarity(polarity)
}

fn edge(id: u64, kind: &str, tail: (u32, u32), head: (u32, u32), order: &str) -> Hyperedge {
    let mut metadata = BTreeMap::new();
    metadata.insert("order".to_string(), order.to_string());
    Hyperedge {
        id: HyperedgeId(id),
        kind: HyperedgeKind::new(kind),
        endpoints: vec![
            ep(tail.0, tail.1, EndpointPolarity::Tail),
            ep(head.0, head.1, EndpointPolarity::Head),
        ],
        weight_milli: None,
        metadata,
        valid_from: RevisionId::ZERO,
        valid_to: None,
        directionality: Directionality::Directed,
        authoring_frame: None,
        computation: None,
    }
}

/// S14 §2 on 0.4.0 (no `register_or_get_space`): registered spaces persist across reopen,
/// and registering again returns `DuplicateId`. The adapter's "open or register" treats
/// `DuplicateId` as "already there" (D-037).
#[test]
fn spaces_persist_and_re_registering_reports_duplicate_id() {
    let dir = TempDir::new().unwrap();
    {
        let db = InfiniteDb::open(dir.path()).unwrap();
        db.register_space(space(ENTITY, "ac.entity", 2)).unwrap();
        db.insert(ENTITY, at(1, 1), b"x".to_vec()).unwrap();
        db.sync().unwrap();
    }
    let db = InfiniteDb::open(dir.path()).unwrap();
    let again = db.register_space(space(ENTITY, "ac.entity", 2));
    assert!(
        matches!(
            again,
            Err(EngineError::RegistrySpace(SpaceError::DuplicateId(id))) if id == ENTITY
        ),
        "got {again:?}"
    );
    assert_eq!(
        db.query(ENTITY, None).unwrap().len(),
        1,
        "space and data survived reopen"
    );
}

/// D-024: serials and log seqs exceed 255, so spaces need more than the 8-bit default.
#[test]
fn large_coordinates_round_trip_with_32_bit_dims() {
    let (db, _dir) = open();
    let p = at(1, 4_000_000_000);
    db.insert(ENTITY, p.clone(), b"game".to_vec()).unwrap();
    db.sync().unwrap();
    let rows = db.query(ENTITY, None).unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].address.point, p);
}

/// IDB-B3 (found on 0.4.0): a space at the default 8 bits/dim *accepts* out-of-range
/// coordinates and aliases them — `[1,300]` overwrites `[1,44]` (300 − 256). Silent data
/// loss, which is why every Assistant Coach space uses `BITS_PER_DIM = 32` and why the
/// adapter must never mint a coordinate above `u32::MAX`. (0.4.1 rejects the insert instead.)
/// This test only prints what the pinned version does; the guard is the next test.
#[test]
fn default_8_bit_space_behaviour_is_recorded() {
    let dir = TempDir::new().unwrap();
    let db = InfiniteDb::open(dir.path()).unwrap();
    let s = SpaceId(900);
    db.register_space(SpaceConfig::new(s, "narrow", 2)).unwrap();
    let first = db.insert(s, DimensionVector::new(vec![1, 44]), b"a".to_vec());
    let second = db.insert(s, DimensionVector::new(vec![1, 300]), b"b".to_vec());
    db.sync().unwrap();
    let live = db.query(s, None).unwrap().len();
    println!(
        "8-bit space: insert [1,44] ok={} · insert [1,300] ok={} · live records = {live} (2 = safe, 1 = aliased)",
        first.is_ok(),
        second.is_ok()
    );
}

/// Guard for IDB-B3: with the adapter's 32 bits/dim, coordinates that would alias at 8 or
/// 16 bits stay distinct.
#[test]
fn adapter_dims_never_alias() {
    let (db, _dir) = open();
    for serial in [44u32, 44 + (1 << 8), 44 + (1 << 16), 44 + (1 << 24)] {
        db.insert(ENTITY, at(1, serial), serial.to_le_bytes().to_vec())
            .unwrap();
    }
    db.sync().unwrap();
    assert_eq!(db.query(ENTITY, None).unwrap().len(), 4);
}

/// S14 §3: update = insert at the same address; the latest revision wins.
/// Delete = tombstone; the record disappears from `query`.
#[test]
fn update_replaces_and_delete_tombstones() {
    let (db, _dir) = open();
    db.insert(ENTITY, at(1, 7), b"v1".to_vec()).unwrap();
    db.sync().unwrap();
    db.insert(ENTITY, at(1, 7), b"v2".to_vec()).unwrap();
    db.sync().unwrap();
    let rows = db.query(ENTITY, None).unwrap();
    assert_eq!(rows.len(), 1, "one live record per address");
    assert_eq!(rows[0].data, b"v2");

    db.delete(ENTITY, at(1, 7)).unwrap();
    db.sync().unwrap();
    assert!(db.query(ENTITY, None).unwrap().is_empty());
}

fn fill_log(db: &InfiniteDb) {
    let rows: Vec<_> = (0..50u32)
        .rev()
        .map(|seq| {
            (
                DimensionVector::new(vec![3, seq]),
                seq.to_le_bytes().to_vec(),
            )
        })
        .collect();
    db.insert_many(LOG, rows).unwrap();
    db.insert(LOG, DimensionVector::new(vec![4, 0]), vec![0xFF])
        .unwrap();
    db.flush(LOG).unwrap();
    db.sync().unwrap();
}

/// S14 §6 as P1 implements it (D-037 workaround for IDB-B1): replay = full `query` of
/// `ac.log`, filtered to the session and sorted by seq. Correct at P1 scale (one log per
/// practice, a few thousand entries).
#[test]
fn log_replay_by_full_query_returns_one_session_in_seq_order() {
    let (db, _dir) = open();
    fill_log(&db);
    let mut hits: Vec<_> = db
        .query(LOG, None)
        .unwrap()
        .into_iter()
        .filter(|r| r.address.point.coords[0] == 3)
        .collect();
    hits.sort_by_key(|r| r.address.point.coords[1]);
    let seqs: Vec<u32> = hits.iter().map(|r| r.address.point.coords[1]).collect();
    assert_eq!(seqs, (0..50).collect::<Vec<_>>());
}

/// IDB-B1 (found by T11a; present in 0.4.0, the pinned version, and in 0.4.1): `query_bbox`
/// silently drops records. A `[3,0]..[3,19]` box over a dense 6×20 grid misses (3,1..3) and
/// (3,16..18); the whole-space box is complete. Points to the Hilbert `range_decompose`
/// boundary handling. Run with `cargo test -- --ignored` after an InfiniteDB upgrade; when
/// it passes, S14 §6 can go back to bbox replay.
#[test]
#[ignore = "IDB-B1: query_bbox drops records (InfiniteDB bug, D-037)"]
fn log_replay_by_bbox_returns_one_session_in_seq_order() {
    let (db, _dir) = open();
    fill_log(&db);

    let mut hits = db
        .query_bbox(
            LOG,
            DimensionVector::new(vec![3, 0]),
            DimensionVector::new(vec![3, u32::MAX]),
            None,
        )
        .unwrap();
    hits.sort_by_key(|r| r.address.point.coords[1]);
    assert_eq!(hits.len(), 50);
    let seqs: Vec<u32> = hits.iter().map(|r| r.address.point.coords[1]).collect();
    assert_eq!(seqs, (0..50).collect::<Vec<_>>());
}

/// S14-T4 precursor: synced writes survive dropping the handle without a clean close.
#[test]
fn synced_writes_survive_reopen() {
    let dir = TempDir::new().unwrap();
    {
        let db = InfiniteDb::open(dir.path()).unwrap();
        db.register_space(space(LOG, "ac.log", 2)).unwrap();
        for seq in 0..20u32 {
            db.insert(LOG, DimensionVector::new(vec![1, seq]), vec![seq as u8])
                .unwrap();
            db.sync().unwrap();
        }
        // dropped here: no explicit close
    }
    let db = InfiniteDb::open(dir.path()).unwrap();
    assert_eq!(db.query(LOG, None).unwrap().len(), 20);
}

/// S14 §4: every relationship is a hyperedge; stack order lives in metadata, not position.
#[test]
fn hyperedges_carry_order_metadata_and_are_queryable_every_way_s14_needs() {
    let (db, _dir) = open();
    // game (1,1) has constraints (2,10), (2,30), (2,20) stacked in that order
    db.insert_hyperedge(
        EDGES,
        edge(1, "ac.game.has_constraint", (1, 1), (2, 10), "0001"),
    )
    .unwrap();
    db.insert_hyperedge(
        EDGES,
        edge(2, "ac.game.has_constraint", (1, 1), (2, 30), "0002"),
    )
    .unwrap();
    db.insert_hyperedge(
        EDGES,
        edge(3, "ac.game.has_constraint", (1, 1), (2, 20), "0003"),
    )
    .unwrap();
    db.insert_hyperedge(EDGES, edge(4, "ac.variant.of", (3, 1), (1, 1), ""))
        .unwrap();
    db.sync().unwrap();

    // by kind, ordered by metadata (S14-T2 shape)
    let mut stack = db
        .query_hyperedges_by_kind(EDGES, "ac.game.has_constraint", None)
        .unwrap();
    stack.sort_by(|a, b| a.metadata["order"].cmp(&b.metadata["order"]));
    let heads: Vec<u32> = stack
        .iter()
        .map(|e| e.endpoints[1].node.coords[1])
        .collect();
    assert_eq!(heads, vec![10, 30, 20]);

    // by id
    let fetched = db
        .fetch_hyperedge_by_id(EDGES, HyperedgeId(4), None)
        .unwrap()
        .unwrap();
    assert_eq!(fetched.kind.as_str(), "ac.variant.of");

    // incident to the game, any direction and directed
    let game = EndpointRef::new(EndpointRole::new("n"), ENTITY, at(1, 1));
    assert_eq!(
        db.query_hyperedges_for_endpoint(EDGES, &game, None)
            .unwrap()
            .len(),
        4
    );
    let outgoing = db
        .query_hyperedges_for_endpoint_directed(EDGES, &game, None, DirectionFilter::Outgoing)
        .unwrap();
    assert_eq!(
        outgoing.len(),
        3,
        "the game is the tail of its three stack edges"
    );

    // delete
    db.delete_hyperedge(EDGES, HyperedgeId(2)).unwrap();
    db.sync().unwrap();
    assert_eq!(
        db.query_hyperedges_by_kind(EDGES, "ac.game.has_constraint", None)
            .unwrap()
            .len(),
        2
    );
}

/// Regression guard for IDB-B2: on 0.4.1 (git `d193986`) `count_incident_edges_for_endpoint`
/// returns 0 once the endpoint's own space is registered. 0.4.0 — the pinned version — is
/// correct. If this fails after an upgrade, count with `query_hyperedges_for_endpoint(..).len()`.
#[test]
fn count_incident_edges_matches_incidence_query() {
    let (db, _dir) = open();
    db.insert_hyperedge(
        EDGES,
        edge(1, "ac.game.has_constraint", (1, 1), (2, 10), "0001"),
    )
    .unwrap();
    db.sync().unwrap();
    let game = EndpointRef::new(EndpointRole::new("n"), ENTITY, at(1, 1));
    let by_query = db
        .query_hyperedges_for_endpoint(EDGES, &game, None)
        .unwrap()
        .len();
    assert_eq!(by_query, 1);
    assert_eq!(
        db.count_incident_edges_for_endpoint(&game, None).unwrap(),
        by_query
    );
}
