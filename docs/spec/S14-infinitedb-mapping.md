# S14 — InfiniteDB Mapping

> Status: draft for review; §1.3/§2/§6 updated by the T11a result (2026-10-28) · Decisions: D-006, D-017, D-018, D-019, D-024, D-036, **D-037** · Needed by: P1 (blocking) · Read against the local checkout `D:\Rust\infinitedb\infinitedb\infinite-db` (Cargo version 0.4.1) on 2026-10-28

## 1. Readiness check

### 1.1 What exists today (0.4.1, read from the source)

| Capability | API on `InfiniteDb` | Status for us |
|---|---|---|
| Open / embedded engine | `InfiniteDb::open(dir)`, `OpenOptions` | **Stable** — default `embedded` feature |
| Spaces | `register_space(SpaceConfig::new(id, name, dims))`, `register_or_get_space` | **Stable** |
| Records | `insert(space, DimensionVector, Vec<u8>) → RevisionId`, `insert_many`, `delete`, `query(space, as_of)`, `query_bbox` | **Stable** |
| Durability | `sync()` (writes are fire-and-forget until `sync`), `flush` | **Stable** — note: must `sync()` before a write is visible to queries |
| Hyperedges | `insert_hyperedge`, `delete_hyperedge`, `fetch_hyperedge_by_id`, `query_hyperedges_by_kind`, `query_hyperedges_for_endpoint(_directed)`, `count_incident_edges_for_endpoint` | **Stable (M1–M7 tested)** — `Hyperedge { id: u64, kind: String, endpoints: [EndpointRef{role, space, node: DimensionVector, polarity}], metadata: BTreeMap<String,String>, weight_milli, … }` |
| Traversal | `traverse_hypergraph(_with_options)`, `check_hypergraph_acyclic`, frame traversals | Usable; P5 (plays) only |
| Branches | `create_branch`, `merge_branch` | Not needed in P1 |
| Server / sync features | `server`, `sync` | **Not used** (D-019: hub is the only process that opens the DB) |
| Blocking contract | SEMANTICS D-BLOCKING-CONTRACT: all methods block; async callers must offload | **Constraint on the hub** → one store thread (D-036) |
| Edition / MSRV | edition 2024, rust 1.85 | Fine; our crates can stay 2021 |

### 1.2 What is in redesign (AJ's decisions, Sept 2026 — not in the code)

| Redesign item | Today in 0.4.1 | Our exposure |
|---|---|---|
| **Intrinsic, opaque identity** | A node *is* `(SpaceId, DimensionVector)` — identity = coordinate | High → handled by the id-space mapping (§2) |
| Cells (subdividable regions) distinct from nodes | Spaces + placement/parent links | Low — court is a plain 2-D space |
| Relational / ML-optimized ordering | Hilbert-keyed | None — we never depend on storage order; order lives in edge metadata |
| Fractal architecture / "cell" entity | Not started | None for P1 |

### 1.3 Pin
- ~~Pin `infinite-db = "=0.4.1"`~~. **Result of the readiness task (P1 T11a) + AJ's call (D-037): pin crates.io `infinite-db = "=0.4.0"`** (0.4.1 is unpublished). `crates/ac-store-idb/tests/idb_api_surface.rs` calls every API in §1.1 that 0.4.0 has and checks the behaviours this spec relies on.
- Differences from §1.1 on 0.4.0: **no `register_or_get_space`** — "open or register" treats `SpaceError::DuplicateId` as already registered (spaces persist across reopen).
- InfiniteDB bugs found (D-037): **IDB-B1** `query_bbox` drops records → §6 replays with a full query; **IDB-B3** the 8-bit default aliases out-of-range coordinates → 32 bits/dim everywhere (§2). (IDB-B2 is a 0.4.1-only regression; a test guards the upgrade.)
- Only `ac-store-idb` depends on `infinite-db` (D-017). A contract test suite in `ac-domain` runs against both the in-memory store and the IDB store, so a later InfiniteDB upgrade is "make the contract tests pass again".

## 2. Spaces we register

| Space | Id | Dims | Coordinates | Payload |
|---|---|---|---|---|
| `ac.entity` | 100 | 2 | `[kind_ordinal, serial]` | versioned envelope (§3) of one domain entity |
| `ac.edges` | 101 | 2 | (InfiniteDB hyperedge storage) | hyperedges (§4) |
| `ac.court` | 102 | 3 | `[x, y, layer]`: x, y in **tenths of a foot + 10 000** (any point within ±1000 ft maps to a non-negative integer; check `bits_per_dim` covers 20 000); layer = context ordinal | spatial index rows: `{ entity, role? }` (§5) |
| `ac.log` | 103 | 2 | `[session_serial, seq]` | one live log entry (S11) |
| `ac.meta` | 104 | 1 | `[key]` | schema version, id counters |

**Bits per dimension (D-037):** every space above registers with `.with_bits_per_dim(32)`. InfiniteDB's default is 8; on 0.4.0 a coordinate above 255 is silently aliased onto a smaller one and **overwrites** it (IDB-B3) — serials and seqs pass 255 in the first practice. 32 bits × at most 3 dims = 96 ≤ InfiniteDB's 128-bit key.

**Identity (D-024):** `EntityId(u64)` is minted by the adapter from `ac.meta` counters. Its storage address is `[kind_ordinal, serial]` — a coordinate that carries **no meaning** beyond "this is entity N of kind K". Domain code sees only `GameId`, `ConstraintId`, … newtypes. When InfiniteDB gains intrinsic ids, only the adapter's address function changes.

Kind ordinals: 1 Game · 2 Constraint · 3 Variant · 4 Read · 5 Play · 6 State · 7 Practice · 8 Block · 9 Season · 10 Player · 11 Gym · 12 Session(live) · 13 Observation.

## 3. Payload envelope

```rust
struct Envelope { schema: u16, kind: u16, id: u64, body: Vec<u8> }  // bincode 2
```
- ~~`body` = the domain struct serialized with serde (bincode).~~ **Amended by D-038:** the record is a JSON envelope `{schema, doc}` (serde_json), so documents written by an older build still load after fields are added (every domain struct keeps unknown fields in `extra`). YAML is never stored.
- **Migration:** on load, `schema < CURRENT` runs `migrate_<kind>_<from>_<to>` functions in order; the store writes back the upgraded record lazily (on next save). Migration functions get fixture tests.
- Update = insert a new record at the same address (latest revision wins); delete = `delete` (tombstone). History is available as-of revision for free.

## 4. Edges (hyperedges in `ac.edges`)

Every relationship is a hyperedge; `kind` is `ac.<relation>`; endpoints point at `ac.entity` addresses; ordering and small data are in `metadata`.

| Kind | Endpoints (role: entity) | Metadata | Why a hyperedge |
|---|---|---|---|
| `ac.game.has_constraint` | tail `game`, head `constraint` | `order` (zero-padded `0003`), `suppressed` | Stack order is data, not position |
| `ac.variant.of` | tail `variant`, head `parent` | — (delta lives in the variant body) | R6 |
| `ac.tagged` | tail `subject` (game/constraint/block/read), head ×N `target` | — | One edge, many targets |
| `ac.read.trained_by` | tail `read`, head ×N `game` | — | R9 query |
| `ac.practice.has_block` | tail `practice`, head `block` | `order`, `basket` | |
| `ac.block.runs` | tail `block`, head `game\|variant` | — | |
| `ac.block.group` | tail `block`, head ×N `player` | `team`, `color` | Groups |
| `ac.play.transition` | tail `from_state`, head `to_state`, neutral ×N `role-participant`, neutral 0..1 `guard_read` | `actions` (JSON-ish text), `timing` | The case for hyperedges (02 §11) — P5 |
| `ac.session.of_practice` | tail `session`, head `practice` | `started_at` | Live session ↔ plan |

Edge ids: `HyperedgeId(u64)` minted from the same counter family.

## 5. Spatial rows (`ac.court`)

Domain data keeps coordinates in the entity body (authoritative). `ac.court` holds **derived index rows** so spatial questions are bbox queries:
- Zones of a game (centroid + bbox corners) — "games with a corner bonus zone".
- Player positions in play States (P5) — "states with O5 in the dunker spot".
- Logged events with a zone (S1) — "where do our stops happen".

`layer` separates contexts (1 zones, 2 states, 3 events). Rows are rebuilt from bodies on save; they are never the source of truth. (This matches the "derived render of the source of truth" direction AJ set for InfiniteDB.)

## 6. Live log (`ac.log`)

- One record per log entry at `[session_serial, seq]`; `seq` is assigned by the hub (S11).
- Write path (D-036): store thread `insert` → `sync()` → reply to hub → hub acks the tap.
- Replay = ~~`query_bbox` over `[session, 0..∞]`~~ **`query(ac.log)` filtered to `session`, sorted by `seq`** (D-037, IDB-B1: `query_bbox` drops records). Fine at P1 scale (a practice is a few thousand entries); return to bbox when the ignored IDB-B1 test passes.
- Undo entries are appended, never deletes.

## 7. Store interface (in `ac-domain`, domain terms only — D-017)

```rust
pub trait Store: Send + Sync {
    // entities
    fn save<E: Entity>(&self, e: &E) -> Result<E::Id>;
    fn load<E: Entity>(&self, id: E::Id) -> Result<Option<E>>;
    fn list<E: Entity>(&self, filter: &ListFilter) -> Result<Vec<Summary<E>>>;
    fn delete<E: Entity>(&self, id: E::Id) -> Result<()>;
    // composition
    fn set_stack(&self, game: GameId, stack: &[(ConstraintId, bool /*suppressed*/)]) -> Result<()>;
    fn set_tags(&self, subject: SubjectRef, targets: &[TargetId]) -> Result<()>;
    // queries (02 §11)
    fn query(&self, q: &Query) -> Result<QueryResult>;
    // live
    fn append_log(&self, session: SessionId, entry: &LogEntry) -> Result<Seq>;  // durable on return
    fn read_log(&self, session: SessionId, from: Seq) -> Result<Vec<LogEntry>>;
    fn open_sessions(&self) -> Result<Vec<SessionId>>;   // crash recovery
}

pub enum Query {
    GamesForTarget { target: TargetId, include_children: bool },
    ReadsOfPlay(PlayId),
    GamesTrainingRead(ReadId),
    UntrainedReads,                       // R9
    BlocksByTargetBetween { target: TargetId, from: Date, to: Date },  // R12, season
    PathStates { play: PlayId, path: Vec<TransitionPick> },             // animation
    EntitiesInZone { layer: SpatialLayer, bbox: Rect },
}
```

`ac-domain` ships `MemStore` (HashMap-based) for tests; `ac-store-idb` ships `IdbStore`. Both pass the same contract tests.

**P1 scope (live-first, D-016):** Game, Constraint, Variant, Gym, Practice, Block, Session + `append_log/read_log/open_sessions` + `GamesForTarget`. Play/State/transition edges and spatial rows wait for P2/P5.

## 8. Worked examples (contract tests)

| Test | Steps | Expected |
|---|---|---|
| S14-T1 | load every `fixtures/games/*.yaml` → `save` → `load` | equal to the loaded domain value |
| S14-T2 | `set_stack(g, [c1, c3, c2])` then `load` | stack order c1, c3, c2 regardless of ids |
| S14-T3 | tag g with 2 targets; `GamesForTarget(off.extend, children)` | contains g |
| S14-T4 | append 500 log entries, drop the store without `close`, reopen, `read_log` | 500 entries, in seq order |
| S14-T5 | p95 `append_log` latency on AJ's laptop | < 150 ms (else D-036 fallback) |
| S14-T6 | entity saved with `schema = 1`, current = 2 | load returns migrated value |

## 9. Open questions

1. Is the local `infinite-db` tree ahead of the crates.io 0.4.1 package? (Decides crates.io pin vs. git-commit pin.)
2. Do you want the Assistant Coach DB to live in its own directory per team/season, or one DB for everything? Proposed: one DB per installation, `Team` as an entity later.
