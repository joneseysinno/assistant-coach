# S11 — Live Protocol (hub ↔ devices)

> Status: draft for review · Decisions: D-009, D-011, D-012, D-013, D-019, D-035, D-036 · Needed by: **P1 (live-first)** — `ac-live` and the hub server

## 1. Purpose

Everything that crosses the network in the gym: who may say what, how the hub orders it, how clocks are computed, and how a phone or the laptop itself recovers.

## 2. Transport

- HTTP on the hub (`0.0.0.0:<port>`, default **7878**) serves the Board/Remote wasm app and a WebSocket at `/ws`.
- Messages are **JSON** (serde, tagged enums: `{"t": "score", …}`) — readable in browser dev tools; compact enough at tap rates.
- One WebSocket per device. The Board on the laptop's TV uses `http://localhost:7878/board` (same protocol, `viewer` role, auto-paired).

## 3. Roles and pairing

| Role | Can | Paired by |
|---|---|---|
| `head` | everything: clocks, next block, extend, policy, plan edits, promote devices | **head PIN** shown only on the laptop screen (not the TV) |
| `scorer` | score/undo/tally/violations for its assigned basket(s); pause/resume the block clock **if** the head allows | **scorer PIN** shown on the TV with the QR code |
| `viewer` | read-only | QR alone |

- PINs are 4 digits, new each session. Wrong PIN 5× from one device → 30 s lockout.
- On success the hub issues a `token` (128-bit random, hex) the client keeps in `localStorage`; reconnects send the token, not the PIN. Tokens die with the session.
- The head can revoke a device from the laptop.

## 4. Client → hub messages (requests)

Every request carries `id` (client-generated ULID — unique, time-sortable) and `basket` where relevant.

| `t` | Fields | Role | Effect (log entry kind) |
|---|---|---|---|
| `hello` | `pin?`, `token?`, `last_seq?`, `device_name` | any | pairing / resume (not logged) |
| `ping` | `client_ms` | any | clock offset (not logged) |
| `fire` | `rule`, `team`, `basket`, `player?` | scorer+ | `rule.fired` (scoring, violations, tallies — all rules) |
| `possession` | `basket`, `to_team?` | scorer+ | `possession.switch` ("Stop ⇄" with no rule) |
| `undo` | `target_seq?` (default: this device's last undoable entry) | scorer+ (own) / head (any) | `undo` |
| `clock` | `op: start\|pause\|resume\|reset_shot`, `clock`, `basket?` | head (scorer if allowed) | `clock.*` |
| `extend` | `seconds`, `from_block` | head | `schedule.extend` |
| `next` | — | head | `block.end` + `block.start` |
| `keep_going` | — | head | `schedule.keep_going` (cancels firm auto-advance) |
| `finish_possession` | — | head | `schedule.finish_possession` (advance at next possession end) |
| `dial` | `dial`, `value`, `basket` | head / scorer (live dials) | `dial.set` |
| `choose` | `constraint`, `basket` | scorer+ | `constraint.on` (D-028) |
| `teams` | `basket`, `teams` | head | `teams.set` |
| `capture` | `note` | scorer+ | `capture` |
| `note` | `text` | head | `coach.note` |

## 5. Hub → client messages

| `t` | Fields | When |
|---|---|---|
| `welcome` | `role`, `token`, `session`, `hub_ms` | after `hello` |
| `state` | full `LiveState` snapshot (§6) | on connect, then on every change (≤ 10/s, coalesced) |
| `ack` | `id`, `seq` or `error` | for each request |
| `pong` | `client_ms`, `hub_ms` | for each ping |
| `alert` | `kind: warn_1min \| horn \| overtime \| auto_advance_in`, `seconds?` | schedule guard (S13) |

`state` is a whole snapshot, not a diff: < 4 KB for two baskets; simpler than patching and safe after drops.

## 6. LiveState

```rust
struct LiveState {
    session: SessionId, seq: Seq,                 // last applied log seq
    hub_ms: u64,                                  // hub clock at send time
    block: BlockView { index, of, name, baskets, protected, planned_s, next: Option<NextView> },
    clocks: Vec<Clock>,                           // block, interval, shot (per basket), game
    games: Vec<BasketGame> {                      // one per active basket
        basket, game_name, teams: Vec<TeamScore { id, color, score }>,
        possession: Option<TeamId>, possession_index: u32, phase: Option<String>,
        board_rules: Vec<String>, buttons: Vec<ButtonView>,  // S12
        live_dials: Vec<DialView>, ladder: Option<Ladder>,
    },
    schedule: ScheduleView { balance_s: i64, policy, overtime_s, cut_preview: Vec<CutLine>, auto_advance_at: Option<u64> },
    recent: Vec<EntryView>,                       // last 10, for the undo list
    recovered: bool,
}
```

## 7. Log entry

```rust
struct LogEntry {
    seq: Seq,              // hub-assigned, gapless per session, starts at 1
    id: RequestId,         // the client's ULID (idempotency key); hub-generated for hub events
    hub_ms: u64,           // wall-clock ms (UTC) at the hub when appended
    device: DeviceId, role: Role,
    kind: EntryKind,       // tagged enum mirroring §4 plus hub events (clock.expired, block.auto_advance, session.start/end)
    undoes: Option<Seq>,
}
```
- Append path (D-036): validate → `Store::append_log` (durable) → fold into `LiveState` → `ack` + broadcast.
- **Fold** is a pure function `fold(&ResolvedPractice, &[LogEntry]) -> LiveState` in `ac-live`. Undo = entries whose seq is in a later `undoes` are skipped. A fold of the whole log after a crash must equal the live state before it (property test).

## 8. Clock math (D-012)

```rust
struct Clock { id, kind, duration_ms: u64, started_at: Option<u64>, paused_total_ms: u64, paused_at: Option<u64> }

fn elapsed(c, now) = match (c.started_at, c.paused_at) {
    (None, _)        => 0,
    (Some(s), None)  => now - s - c.paused_total_ms,
    (Some(s), Some(p)) => p - s - c.paused_total_ms,
}
fn remaining(c, now) = c.duration_ms as i64 - elapsed(c, now) as i64   // negative = overtime
```
- `start`: `started_at = now`. `pause`: `paused_at = now`. `resume`: `paused_total += now − paused_at; paused_at = None`. `+N s`: `duration += N`.
- Intervals: the interval clock restarts on each work/rest boundary; the boundary is a hub event computed from the block clock (so it can't drift from it).
- Shot clock: one per basket; `reset_shot` and any possession change set `started_at = now, paused_total = 0`.

**Client display:** each client keeps `offset = hub_ms − (client_send + client_recv)/2` from the ping with the **smallest round trip** of the last 8 (ping every 2 s, every 0.5 s for the first 5 s). Displayed time = `remaining(clock, client_now + offset)`, rendered every animation frame. Clients never decide expiry; the hub emits `clock.expired`.

## 9. Idempotency and ordering

- The hub keeps `HashMap<RequestId, Seq>` for the session (rebuilt from the log on recovery). A repeated `id` returns the original `ack` and appends nothing.
- Requests from all devices go through one channel into the hub task, so the log order is the arrival order.
- A scorer's `undo` with no target undoes **its own** last undoable entry, never another scorer's.

## 10. Reconnect

1. Remote shows **Reconnecting…**, keeps tapping into a local queue (max 50), buttons dimmed but live.
2. On reconnect: `hello { token, last_seq }` → `welcome` + `state`.
3. Client resends its queue in order; duplicates are dropped by id (§9).
4. Taps older than 60 s in the queue are shown to the scorer to confirm or discard (a basket two minutes ago may be stale).

## 11. Laptop restart mid-block

1. On launch, `Store::open_sessions()` finds a session without `session.end`.
2. The app asks: **Resume practice?** (default yes, 10 s countdown).
3. Replay: `read_log` → `fold` → `LiveState { recovered: true }`.
4. Clocks: a running clock's `started_at` is wall time, so time that passed during the crash **counts as played** (players kept playing). The Board shows a small "recovered" badge for 30 s.
5. Devices reconnect with their tokens (tokens are stored in the session record, so they survive).

## 12. Tests (P1)

| Test | Scenario | Expected |
|---|---|---|
| S11-T1 | start at 0, pause at 60 000, resume at 90 000, now 120 000, duration 480 000 | remaining 390 000 |
| S11-T2 | duplicate `fire` with the same id | one log entry, two acks with the same seq |
| S11-T3 | two scorers fire on basket A at the same ms | both applied, consecutive seqs |
| S11-T4 | scorer B `undo` with no target after A's tap | undoes B's last entry, not A's |
| S11-T5 | replay fixture log `fixtures/logs/closeout-block.yaml` | final score red 7 blue 5, game ended, rotation applied |
| S11-T6 | kill the hub process after entry 40, restart | state equals fold of 40 entries; tokens valid |
| S11-T7 | client offset with pings (rtt 30, 12, 80 ms) | offset from the 12 ms sample |
