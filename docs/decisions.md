# Decision Record

> Every architectural decision for Assistant Coach, numbered and dated. A decision changes only through a new entry that supersedes it. When code and this file disagree, this file wins until a new decision is recorded.

Status key: **Accepted** · **Changed** (accepted in a modified form) · **Proposed** (drafted, waiting on AJ) · **Open** · **Superseded**

Format per entry: **ID · decision · why · alternatives considered · status · date**. The short table below is the index; entries D-020 onward also have a detail section further down.

## Index

| ID | Decision | Why | Status | Date |
|---|---|---|---|---|
| D-001 | Game designer is built before the play designer | CLA emphasis; plays depend on games; the court canvas is reused | Accepted | 2026-10-28 |
| D-002 | Plays are decision graphs; animation is derived from a path through the graph | Exposes reads; enables decompose/generate | Accepted | 2026-10-28 |
| D-003 | Closed primitive set: Space, Role, Event, Rule, Read, Dial; everything else is composition | Prevents drift | Accepted | 2026-10-28 |
| D-004 | Scoring and limits are structured Rules over Events | Cards, Remote buttons and pinpoints all come from one source | Accepted | 2026-10-28 |
| D-005 | Sport-agnostic core + basketball sport pack | Basketball first without lock-in | Accepted | 2026-10-28 |
| D-006 | **Persistence is InfiniteDB from day one** (not JSON files). See D-017–D-019, D-024 | Plays are graphs and animations are derived from graph paths. Games, constraints, reads, practices and the live log are all relational | **Changed** | 2026-10-28 |
| D-007 | Estimates are transparent heuristics (formula + confidence), calibrated from observations | Trust and correctability | Accepted | 2026-10-28 |
| D-008 | Measurement is decision/outcome based; no technique score | Aligned with CLA | Accepted | 2026-10-28 |
| D-009 | The laptop is the hub and the single source of truth in the gym | One clock, one score, no conflicts | Accepted | 2026-10-28 |
| D-010 | Three surfaces: Studio (JoInn shell), Board (TV), Remote (phone/tablet) | Each device gets the right UI | Accepted | 2026-10-28 |
| D-011 | Remote and Board are browser clients served by the hub (wasm); native Android later | No app-store install, one codebase | Accepted | 2026-10-28 |
| D-012 | Clocks run only on the hub, stored as start + paused + duration | No drift; survives reconnects | Accepted | 2026-10-28 |
| D-013 | All live actions are Events in an append-only log | Undo, crash recovery, observation data | Accepted | 2026-10-28 |
| D-014 | Remote scoring buttons are generated from the active game's Rules | Zero scoreboard setup; rules are the single source | Accepted | 2026-10-28 |
| D-015 | Schedule guard: soft/firm/locked policies, protected blocks, visible overtime cost | Keeps the coach from running long on one drill | Accepted | 2026-10-28 |
| D-016 | **Order: live first (option B).** The phase after P0 builds foundations + the live hub/Board/Remote running games written as fixture files; the Game designer comes after | Gets the scoreboard into practices sooner and tests the network risk (the biggest technical unknown) first. Fixtures already exist as the test corpus, so live mode has real games to run | **Accepted** | 2026-10-28 |
| D-017 | **Storage boundary:** `ac-domain` defines the store interface (save/load/query/append-log) in domain terms; a separate adapter crate implements it on InfiniteDB. App and UI code never call InfiniteDB directly | InfiniteDB is mid-redesign (identity, cells, ordering). A narrow adapter limits the blast radius | **Accepted** (G0) | 2026-10-28 |
| D-018 | **Fixtures stay as human-readable YAML.** They are the test corpus and an import/export format, not the storage. Tests load fixtures → write to InfiniteDB → read back → compare | Readable specs and diffs; proves the DB round-trip | **Accepted** (G0) | 2026-10-28 |
| D-019 | **Only the hub opens the database.** Board and Remote never touch it | Thin browser clients; no early wasm requirement on the DB | **Accepted** (G0) | 2026-10-28 |
| D-020 | **Gym profiles.** Courts vary by gym, so a *Gym* (court preset + list of baskets with position and facing) is settings data; a Practice names its gym; a Game's space is written in basket-local coordinates and placed on a basket by the Block | AJ: "varies by gym" | Accepted | 2026-10-28 |
| D-021 | **Shot clock per game.** A game may declare a possession clock (dial, e.g. 6–24 s) shown on the Board; reset triggers come from rules | AJ: yes, per game | Accepted | 2026-10-28 |
| D-022 | **Default overtime policy: Firm**, 60 s grace, warning at 1:00 | AJ: firm | Accepted | 2026-10-28 |
| D-023 | **Repo layout: one Cargo workspace** (`ac-domain`, `ac-sport-basketball`, `ac-store-idb`, `ac-live`, `ac-app`, `ac-web`). Studio app crate generated from the JoInn template, then moved into the workspace | Domain and hub logic testable without UI; reusable by an MCP server later; D-005 enforced by crate boundary | **Accepted** (G0; built in T01) | 2026-10-28 |
| D-024 | **InfiniteDB pin and id mapping.** ~~Pin `infinite-db = "=0.4.1"` (crates.io)~~ → pin amended by **D-037** (0.4.1 is not on crates.io). Because 0.4.1 node identity is a coordinate, the adapter mints opaque ids and stores every entity at `[kind, serial]` in an *entity space*; positions live in a separate *court space*. Coordinates in the entity space are ids, never meaning | Emulates the intrinsic-identity decision today; when InfiniteDB ships intrinsic ids only the adapter changes | Proposed | 2026-10-28 |
| D-025 | **Every rule has a `judge`:** `coach` (one tap on the Remote), `players` (self-officiated, shown on cards/Board, no button), `auto` (hub evaluates from clocks/logged state) | Stress set #1, #7, #12: many rules are for players to follow, not for the scorer to tap | Proposed | 2026-10-28 |
| D-026 | **Phases inside a Format.** A base format may have ordered phases (own roles/space/direction) with a transition on possession end and a role map | Stress set #2 (3v2 → 2v1 continuous) has no other clean encoding | Proposed | 2026-10-28 |
| D-027 | **Live dials.** A dial may be marked `live: true` (set during play from the Remote) with `visible_to: all / defense / offense / coach` | Stress set #3 (coach shows the coverage card); keeps "coach variable" inside the Dial primitive rather than adding a seventh | Proposed | 2026-10-28 |
| D-028 | **Constraint pool + `choose` consequence.** A game may list inactive *pool* constraints; a consequence lets a team (e.g. the loser) pick one; the coach records the pick | Stress set #4 (losing team picks the next rule) | Proposed | 2026-10-28 |
| D-029 | **Ball count is a format field** (default 1). With more than one ball, possession-scoped rules are rejected by the validator | Stress set #11 (two balls) — in scope, cheaply, with a clear limit | Proposed | 2026-10-28 |
| D-030 | **Zone enter/exit are events** (`zone.enter`, `zone.exit`, actor + zone) | Stress set #12 (help defender may not leave the paint); also covers "catch in corner" | Proposed | 2026-10-28 |
| D-031 | **Individual constraints are Block-level player modifiers** that compile to Role-level rules with an actor filter (answers `02` §10.4) | Stress set #9; keeps games reusable across rosters | Proposed | 2026-10-28 |
| D-032 | **Neutral players are a Role with `team: neutral` and a `plays_for` policy** (offense / defense / ball-side) (answers `02` §10.2) | Stress set #7; keeps primitives closed | Proposed | 2026-10-28 |
| D-033 | **Rule patterns stop at possession-scoped sequences** (`when` + up to two `after` steps, optional `within` seconds) plus counts and clock triggers (answers `02` §10.1) | Every stress game fits; anything more general waits for a real game | Proposed | 2026-10-28 |
| D-034 | **Rule precedence is explicit:** a rule lists what it `overrides`; matching rules otherwise all apply, in stack order. Zone *kind* is styling only — zone behavior always comes from rules | Predictable for coaches and testable; no hidden "most specific wins" | Proposed | 2026-10-28 |
| D-035 | **A Remote tap logs a rule firing, and the rule's pattern supplies the implied events.** "Kick-out 3 +3" logs one entry that implies `make(three)` and `paint-touch` in that possession | Scorekeeping and tallying stay one act (06 §4) without the coach entering raw events | Proposed | 2026-10-28 |
| D-036 | **Live log durability:** each log entry is written to InfiniteDB and `sync()`ed before the hub acks the tap; the hub also keeps the session log in memory. All InfiniteDB calls run on one dedicated store thread (InfiniteDB is a blocking API) | Crash recovery (D-013) needs durable taps; InfiniteDB's own blocking contract forbids calling it from async tasks | Proposed | 2026-10-28 |
| D-037 | **InfiniteDB pin = git commit `d193986` (0.4.1)**, 32-bit dimensions on every Assistant Coach space, and two adapter workarounds: log replay by full `query` + session filter (IDB-B1), incident counts by `query_hyperedges_for_endpoint(..).len()` (IDB-B2) | T11a: crates.io stops at 0.4.0; GitHub `main` = AJ's local 0.4.1. The default 8 bits/dim rejects coordinates > 255. `query_bbox` drops records (also in 0.4.0); `count_incident_edges_for_endpoint` returns 0 | **Accepted** (T11a result, within D-024's own rule: "pin the git commit instead and record it") | 2026-10-28 |

## Answers to the open questions in `04` §4

| # | Question | Answer | Source |
|---|---|---|---|
| 1 | Courtside device | Laptop hub + TV via HDMI; Android phone over a local router; tablet supported | AJ (earlier) |
| 2 | Users | **Proposed:** coach + 1–2 assistants score at once; players get **paper/TV** game cards, not phones | Claude default — AJ to confirm |
| 3 | Sets vs. concepts | **Open** — AJ to supply the team's sets/concepts for W3.3 | — |
| 4 | Courts | **Varies by gym** → D-020 gym profiles | AJ |
| 5 | Existing material | **Proposed:** start clean; import the 14-week plan and the precision-teaching sheet as P6/P4 tasks | Claude default |
| 6 | Persistence | InfiniteDB from day one (D-006) | AJ (earlier) |
| 7 | Rule editor | **Proposed:** structured forms for v1; joinn later (`03` §9.10) | Claude default |
| 8 | Load tracking | **Proposed:** per-session estimated load in P3; weekly load/ACWR view in P4 using the AWU/ACWR approach already used for athlete programs | Claude default |
| 9 | Standalone phone mode | **Proposed:** later (P7) | Claude default |
| 10 | D-016 | **Live first (B)** | AJ |
| 11 | Shot clock | **Yes, per game** → D-021 | AJ |
| 12 | Overtime default | **Firm** → D-022 | AJ |

---

## Detail: D-020 … D-036

### D-020 Gym profiles
- **Decision:** A `Gym` = court preset (or custom length × width + line set) + a list of `Basket { id, label, position, facing }` in gym coordinates (origin center court). A Game's Space is in basket-local coordinates (S3). A Block places a game on one or more baskets.
- **Alternatives:** one global court setting (fails in away gyms); per-practice court (duplicated data).
- **Consequence:** Practice gets a `gym` field; Settings gets a Gyms list. Board two-basket layout comes from the Block's basket list.

### D-021 Shot clock per game
- **Decision:** `format.shot_clock: { seconds: $shot, resets_on: [possession.change, off-reb?] }`. The Board shows it only when the active game declares it. Expiry is a `clock.expired{clock: shot}` event; the game's rule decides the consequence (usually turnover). Reset is a Remote tap ("Reset ⟲") plus automatic reset on any consequence that changes possession.
- **Alternatives:** a global shot clock toggle (not tied to rules).

### D-022 Firm by default
- Warning at 60 s (amber + chirp + vibrate), horn at zero, then 60 s grace, then auto-advance unless the head coach taps **Keep going**. A practice may override. Protected blocks are never cut.

### D-023 Workspace
```
assistant-coach/
  Cargo.toml                 [workspace]
  crates/
    ac-domain/               pure types, rules, resolver, renderer, store trait. no Dioxus, no IO
    ac-sport-basketball/     court presets, named regions, event vocabulary, targets (data)
    ac-store-idb/            Store impl on infinite-db =0.4.1
    ac-live/                 hub logic: log, fold, clocks, guard, buttons. pure, no tokio
    ac-app/                  Dioxus desktop: Studio (JoInn shell) + embedded hub server (axum, tokio)
    ac-web/                  Dioxus web (wasm): Board + Remote routes, served by ac-app
  docs/                      (these docs)
  fixtures/ -> docs/fixtures (test corpus)
```
- **Alternatives:** single app crate from the template (fast start, but domain becomes untestable without UI and an MCP server can't reuse it).
- **Note:** the template is `cargo generate`d once into `crates/ac-app`, its `app-shell` git dependency kept pinned at a tag.

### D-024 InfiniteDB pin and id mapping
- See S14 §2–3. Entity space `ac.entity` (2 dims: `kind`, `serial`); court space `ac.court` (3 dims: x, y in tenths of a foot + 1000 ft offset, `layer`); log space `ac.log` (2 dims: `session serial`, `seq`). Edges are hyperedges in `ac.edges` with kind strings `ac.<relation>` and order in metadata.
- **Alternatives:** wait for intrinsic identity (blocks P1); use coordinates as real positions for entities (couples identity to placement — the thing the InfiniteDB redesign is undoing).

### D-025 Rule judge
- `judge: coach | players | auto`. Remote buttons are generated only for `coach` rules (and `players` rules may appear under a collapsed "Calls" tray so a coach can enforce one). `auto` rules are evaluated by the hub (clock expiry, score targets).
- Default: `coach` if the rule awards points, `players` otherwise.

### D-026 Phases
- `format.phases: [ {id, roles, space, attack: basket-A | basket-B}, … ]`, `format.phase_flow: { on: possession.end, next: <phase>, role_map: {...} }`. A single-phase format omits both. Role maps may name event actors ("shooter leaves") — the coach handles that in the gym; the app only needs to know who is on which team for scoring.

### D-027 Live dials
- Same Dial primitive, plus `live: true` and `visible_to`. A live dial's value is logged as an entry (`dial.set`) so the observation data knows which coverage was shown on each possession. Enum live dials render as a picker on the Remote; the Board hides them unless `visible_to: all`.

### D-028 Constraint pool + choose
- `game.pool: [constraint…]` (inactive until chosen). Consequence `choose: { by: losing-team | winning-team | coach, from: pool, count: 1, replaces: previous-choice? }`. The Remote shows the pool as cards; the tap logs `constraint.on`.

### D-029 Ball count
- `format.balls: 1` default. `balls > 1` ⇒ validator error on any `after`/`count` pattern with `scope: possession`.

### D-030 Zone events
- `zone.enter` / `zone.exit` with `{actor, zone}`; usually `judge: players`. Also allows "catch in corner" as `catch` with `zone:` filter (already in S1), so zone events are only needed for *movement* limits.

### D-031 Block-level player modifiers
- `block.modifiers: [{ player, rule }]` where the rule's actor filter is `player`. The resolver compiles them into the resolved game for that block only. Cards for the block list them under the player's name.

### D-032 Neutral roles
- `Role { team: neutral, plays_for: offense | defense | ball-side }`. Limits (e.g. "joker can't shoot") are ordinary rules with `actor: { role: J }`.

### D-033 Pattern limit
- Allowed triggers: single event (+ filters), event `after` event(s) within possession (max 2 steps, optional `within` seconds), count of event by player/team within possession compared to a value, clock trigger, score/state trigger. Nothing else until a real game needs it (new decision).

### D-034 Explicit precedence
- Resolution order for one triggering event in S2 §6. `overrides` is the only way one rule suppresses another.

### D-035 Tap = rule firing
- Log entry `rule.fired { rule, team, implied: [events] }`. Pinpoints that award nothing are rules whose only consequence is `tally` (so a pinpoint is also "a rule", and its button comes from the same generator).

### D-036 Live log durability
- Tap → hub validates → store thread appends to `ac.log` → `sync()` → hub folds into state → broadcast + ack. Measured in P1 task T12 (S14-T5); if sync latency at tap rates exceeds 150 ms p95 on AJ's laptop, fall back to batching syncs every 250 ms and record a superseding decision.

### D-037 InfiniteDB pin and adapter workarounds (T11a result)
- **Pin:** `infinite-db = { git = "https://github.com/joneseysinno/infinitedb.git", rev = "d193986e96331f0cf9bfc1bc0d9cc15e72b8c8e4" }` (Cargo version 0.4.1; GitHub `main` on 2026-09-05; identical to AJ's local tree on 2026-10-28). crates.io's latest is 0.4.0, so D-024's `=0.4.1` could not resolve.
- **Dimensions:** every space registers with `.with_bits_per_dim(32)` (`ac_store_idb::spaces::BITS_PER_DIM`). The default is 8, and 0.4.1 rejects a coordinate of 300 with `InvalidSpaceConfig { "coordinate 300 exceeds bits_per_dim 8 (max 255)" }`. 32 × 3 dims = 96 ≤ 128, so it fits every space in S14 §2.
- **IDB-B1:** `query_bbox` silently drops records. Repro: a dense 6×20 grid in a 2-D space; box `[3,0]..[3,19]` returns 14 of 20, missing (3,1..3) and (3,16..18); `[2,0]..[4,19]` misses the same y-values in each column; the whole-space box is complete. Same on 0.4.0, independent of `bits_per_dim`. Likely the Hilbert `range_decompose` boundary handling. **Workaround:** `read_log` = `query(ac.log)` filtered to the session and sorted by seq (fine at P1 scale). Spatial rows (S14 §5, P2/P5) wait for the fix.
- **IDB-B2:** `count_incident_edges_for_endpoint` returns 0 once the endpoint's own space is registered, while `query_hyperedges_for_endpoint` returns the edges. **Workaround:** count with the incidence query.
- Both bugs have `#[ignore]`d tests in `crates/ac-store-idb/tests/idb_api_surface.rs` asserting the correct behaviour. After an InfiniteDB upgrade run `cargo test -p ac-store-idb -- --ignored`; when they pass, remove the workarounds (and this note) in a superseding decision.
- **For the InfiniteDB project:** B1 and B2 are filed here with repros; they belong in that repo's issue list / SEMANTICS review.

