# P1 — Foundations + Live Practice (live-first, D-016 B)

> One task = one Cursor session = one PR. Each task names the specs and decisions it implements and the **acceptance tests** that must pass. Tests point at `docs/fixtures/`. Don't start a task until the ones it depends on are merged, and the gate decisions for it are Accepted (`P1-execution.md` §1).
>
> **How and when:** `P1-execution.md` (order, lanes, gates, gym milestones, session protocol). Proposals P-1…P-5 from that plan were accepted on 2026-10-28 and are applied below.
>
> **Exit criterion (from `04` P2, now P1):** two weeks of real practices run on the TV from the phone, no lost scores, no clock drift, planned-vs-actual minutes logged.
>
> **Ground rules for every PR:** no Dioxus or IO in `ac-domain`/`ac-live`; no `infinite-db` outside `ac-store-idb` (D-017); no basketball words in `ac-domain` types (D-005 — they live in `ac-sport-basketball`); new concepts need a decision first (D-003). Use the review checklist (`review-checklist.md`). `scripts/ci.ps1` runs the boundary check, the fixture oracle, fmt, clippy and tests.

## Status

| Task | State | Notes |
|---|---|---|
| T00 | ✅ built + cloud-verified · **gym run pending (M0)** | `spikes/tracer/README.md` has the gym script. wasm build not verified in the cloud; run `build.ps1` on the laptop |
| T01 | ✅ done 2026-10-28 | CI green; `cargo run -p ac-app` opens the shell (app-shell `v0.1.0`) |
| T11a | ✅ done 2026-10-28 | → **D-037**: pin git `d193986` (0.4.1 unpublished); 32-bit dims; IDB-B1 `query_bbox` drops records, IDB-B2 `count_incident_edges_for_endpoint` returns 0 — both worked around in the adapter |
| next | **Gate G1** (accept D-025…D-034), then T02 ∥ T04 | |

## Stage 0 — Tracer (P-1)

| # | Task | Specs / decisions | Depends | Acceptance |
|---|---|---|---|---|
| T00 | **Tracer bullet** (throwaway, `spikes/tracer`, own workspace). axum + `/ws` on 0.0.0.0:7878; plain-JS `/board` + `/remote` (+1/+2/+3, undo, clock); a Dioxus wasm `/wasm` page embedded in the exe; hub window with QR. No domain, no rules, no store | 06 §8; E9 | T01 | Two clients scoring with re-sends → exact score (test); the gym script in its README run on the floor, results in `validation/gym-network.md` |

## Stage A — Workspace and domain (no UI)

| # | Task | Specs / decisions | Depends | Acceptance |
|---|---|---|---|---|
| T01 | **Workspace scaffold.** Root `Cargo.toml` workspace; empty crates `ac-domain`, `ac-sport-basketball`, `ac-store-idb`, `ac-live`, `ac-app`, `ac-web`; JoInn template into `crates/ac-app` (app-shell pinned at a tag); CI script: boundary check, fixture oracle, `cargo fmt --check`, `clippy -D warnings`, `cargo test` | D-023 | — | `cargo test` green on an empty workspace; `cargo run -p ac-app` opens the shell |
| T02 | **Geometry.** `Point`, `Shape` (named, rect, circle, band, polygon, union, minus, mirror), `area`, `contains`, polygonize arcs at 2°; basket ↔ gym transform | S3 §4–6, §8 | T01 | S3-T1…T9, T8b |
| T03 | **Sport pack: court.** Presets hs/ms/ncaa/nba as data; named regions; `Gym`/`Basket` | S3 §3, §7; D-020 | T02 | S3-T3…T6 against the pack; pack values in one `const` table |
| T04 | **Events.** `EventKind` as pack data (string ids validated against the pack), `Event` struct, `source`, `entry` | S1 §3–4 | T01 | Vocabulary loads; unknown kind rejected |
| T05a | **Rules: types.** `Rule`, triggers, conditions, consequences, `judge`, `overrides`; `$dial` refs as `Value::Ref`; serde | S2 §2–5; D-025, D-033, D-034 | T04 | Types round-trip through serde; every S2 example parses |
| T05b | **Rules: validator** for S2 §8 | S2 §8 | T05a | Each S2 invariant has a failing-case test |
| T06 | **Format, Constraint, Dial, Game, Variant types** + role-edit sugar | S4 §2–7; S5 §2–6; D-026–D-032 | T05b | Types only; serde derives |
| T07 | **YAML fixture loader.** `serde_yaml` → domain for games, variants, practices, gyms, logs; error messages with file + path | S5; D-018 | T06 | Every file in `docs/fixtures/games/**` loads; round-trip YAML → domain → YAML → domain equal (S5-T6) |
| T08 | **Resolver.** `resolve(game, overrides) → ResolvedGame` (stack walk, role-edit expansion, dial binding, validation); rollback to N | S5 §5–7 | T07 | S5-T1…T5; every fixture resolves with 0 errors; s11 negative test (adding `after:` fails, D-029) |
| T09 | **Renderer.** Plain-English card lines + Board short form | S2 §7 | T08 | Every fixture's `expect.card` / `expect.board` matches exactly; **CI diffs the Rust output against `tools/p0_check.py` on every fixture** |

## Stage B — Storage

| # | Task | Specs / decisions | Depends | Acceptance |
|---|---|---|---|---|
| T10 | **Store trait + MemStore + contract test suite**; the store-thread channel API the hub uses (so MemStore and IdbStore are interchangeable behind it) | S14 §7; D-036 | T08 | Contract tests pass on MemStore |
| T11a | ✅ **InfiniteDB readiness** (done — D-037) | S14 §1.3 | T01 | `crates/ac-store-idb/tests/idb_api_surface.rs` |
| T11 | **IdbStore: entities + edges.** Spaces `ac.entity/edges/meta` (32-bit dims), id minting, envelope, stack order in edge metadata; incident counts via the incidence query (IDB-B2) | S14 §2–4; D-024, D-037 | T10, T11a, **Gate G2** | Contract tests pass on IdbStore; S14-T1…T3, T6 |
| T12 | **IdbStore: live log.** `ac.log`, append+sync, read (full query + session filter, IDB-B1), open sessions; dedicated store thread with a channel API | S14 §6; D-036, D-037 | T11 | S14-T4 (crash), S14-T5 (p95 < 150 ms on AJ's laptop, printed by a bench) |

## Stage C — Hub logic (pure Rust, `ac-live`)

| # | Task | Specs / decisions | Depends | Acceptance |
|---|---|---|---|---|
| T13a | **Log + fold core.** `LogEntry`, `EntryKind`, `fold(practice, entries) → LiveState`; scores, possession counter, undo, implied events (D-035) | S11 §6–7; S1 §2, §6; S2 §6 | T08, **Gate G3** | S11-T5 on `fixtures/logs/closeout-block.yaml`; S1-T1…T4; S2-F1…F4 |
| T13b | **Game end + rotation** (king ladder, G-06) + the fold property test | S4 §5 | T13a | S4-T3; property test: fold(prefix) then fold(rest) = fold(all); every gym log in `fixtures/logs/` replays |
| T14 | **Clocks.** Block, interval, shot (per basket), game; pause/resume/+N; expiry events with an injected clock | S11 §8; D-012, D-021 | T13b | S11-T1; interval boundaries; shot reset on possession change |
| T15 | **Schedule guard.** Policies, warn/horn/grace, cut algorithm, planned-vs-actual record | S13 | T14 | S13-T1…T7 |
| T16 | **Buttons.** `buttons(resolved_game)` + calls + pinpoints + players' rules drawer + live dials | S12 §2 | T09 | Every fixture's `expect.buttons_per_team` / `expect.calls` |
| T17 | **Request handling.** Validate request by role → entry; idempotency map; per-device undo | S11 §3–4, §9 | T13a | S11-T2, T3, T4; role matrix tests |

## Stage D — Hub server and clients

| # | Task | Specs / decisions | Depends | Acceptance |
|---|---|---|---|---|
| T18 | **Embedded hub server in `ac-app`.** tokio + axum on 0.0.0.0:7878 on its own thread (T00 pattern); `/ws` JSON; serves the embedded `ac-web` build; hub task owns `LiveState`; store thread **on MemStore** (P-2 — IdbStore swaps in at M4); ping/pong. Delete `spikes/tracer` in this PR | S11 §2, §5; D-009, D-036 | T10, T14, T17 | Integration test with two fake WebSocket clients scoring at once (port of T00's `two_clients.rs`) |
| T19a | **Board (wasm) layouts** — `/board`: running / two baskets / last minute / overtime / rest / between blocks / pairing; type sizes from S12 §5 (updated by the gym test) | S12 §4–6 | T18, T09 | Visual check against `printables/mock-board.html` |
| T19b | **Board clock sync** — offset-corrected clock (lowest-RTT ping sample, T00 pattern); drift log | S11 §8 | T19a, T14 | Clock stays within 100 ms of the hub over 10 min (logged) |
| T20a | **Remote (wasm) core** — `/remote`; pairing (QR URL + PIN → token), local tap queue, ack states, reconnect with a 4 s heartbeat that **abandons** the dead socket (T00 finding) | S12 §3; S11 §3, §10 | T18, T16 | Manual script on the Android phone: pair, score 20 taps, airplane-mode 10 s mid-taps, reconnect → no lost/duplicate taps |
| T20b | **Remote layouts** A–D, undo list, vibrate | S12 §2–3 | T20a | Layouts match S12; undo list per device |
| T21 | **Pairing & roles.** Head PIN on laptop, scorer PIN + QR on TV, lockout, revoke, basket lock | S11 §3 | T20a, **Gate G4** | Role matrix enforced server-side (tests) |
| T22 | **Crash recovery end-to-end** — includes swapping IdbStore in behind the store thread | S11 §11 | T21, T12 | S11-T6 run as a real process kill |
| T23 | **Studio minimal: Practice doc type.** List of blocks (pick fixture games/variants), minutes, baskets, teams/colors, protected, policy; gym picker; **Run ▶** hands the practice to the hub | S8 §2; D-020 | T11, T18, T09 | Plan a practice from fixtures, save, reopen, run |
| T24 | **Second screen + sound.** Open the Board full-screen on the second display from Studio; horn/chirp; audio unlock step | S12 §7 | T19a | Horn audible from TV at block end |
| T25 | **First-run & gym checklist.** Firewall: **port rule** for 7878 on Private networks (survives rebuilds), IP + QR display, lid/sleep reminder, "test from phone" button | 06 §8; gym-network results | T21 | Fresh Windows user account: phone connects in < 2 min following the screen |

## Stage E — Field release

| # | Task | Depends | Acceptance |
|---|---|---|---|
| T26 | **Practice-week build:** release build, run one real practice, log issues | T15, T22, T23, T24, T25 + **E3 real games encoded** | Practice runs start to finish; issues filed |
| T27 | **Fix-up from week 1** | T26 | Second week runs with no lost scores, no clock drift (exit criterion) |

## Dependency sketch
```
T01 ─┬─ T00 (M0, gym)                         T11a ✅
     ├─ T02 ─ T03
     └─ T04 ─ T05a ─ T05b ─ T06 ─ T07 ─ T08 ─┬─ T09 ─ T16 ───────────────┐
                                             ├─ T10 ─ T11 ─ T12 ─────┐   │
                                             └─ T13a ─ T13b ─ T14 ─ T15  │
                                                  └─ T17 ─┐   │       │   │
                                        T10 + T14 + T17 ─ T18 ─┬─ T19a ─ T19b, T24        (M2)
                                                               ├─ T20a ─ T20b ─ T21 ─ T25 (M3)
                                                               │            T21 + T12 ─ T22 (M4)
                                                               └─ T23 (T11, T09)
                                     T15 + T22 + T23 + T24 + T25 + E3 ─ T26 ─ T27         (M5)
```
