# P1 — Execution Plan

> Status: **in execution** (approved 2026-10-28: "execute") · Companion to `P1-tasks.md` (the *what*); this file is the *how and when*: order, lanes, gates, gym milestones, the Cursor session protocol and tripwires.
> It does **not** change scope. Proposals P-1…P-5 were **accepted** with the go-ahead and are applied in `P1-tasks.md`.

## Progress

| Date | What | Result |
|---|---|---|
| 2026-10-28 | Wave 0: G0 decisions (D-017, D-018, D-019, D-023) accepted; P-1…P-5 applied to `P1-tasks.md`; stale refs fixed | done (`git init` still to do on the laptop) |
| 2026-10-28 | **T01** workspace scaffold | ✅ CI green (boundary check, fixture oracle 13/13, fmt, clippy -D warnings, tests); `ac-app` opens the shell |
| 2026-10-28 | **T11a** InfiniteDB readiness | ✅ → D-037: git pin `d193986`; 32-bit dims; IDB-B1/B2 found, worked around, `#[ignore]` tests track them |
| 2026-10-28 | **T00** tracer bullet | ✅ built + verified in the cloud (tests, real-browser drop test, hub window). **Wasm build + gym run = M0, on the laptop** |
| next | **M0 at practice**, **Gate G1**, then Wave 2 (T02 ∥ T04) | |

---

## 1. Where we stand (P0 items that touch P1)

P1 can start before P0 is fully closed, because most open P0 items only block **late** P1 tasks. Each item below gets cleared just before the task that needs it, not all at once up front.

| P0 item | State | Blocks | Needed by |
|---|---|---|---|
| D-017, D-018, D-019, D-023 (storage boundary, YAML fixtures, hub-only DB, workspace) | Proposed | T01, T07, T10 | **Gate G0** (before T01) |
| D-025, D-033, D-034 (judge, pattern limit, precedence) | Proposed | T05 | Gate G1 |
| D-026 … D-032 (phases, live dials, pool, balls, zone events, modifiers, neutral) | Proposed | T06, T08 | Gate G1 |
| D-024 (pin + id mapping), S14 Q1 (local tree vs crates.io 0.4.1), S14 Q2 (one DB per install?) | Proposed / open | T11a, T11 | Gate G2 |
| D-035 (tap = rule firing), D-036 (sync per tap, store thread) | Proposed | T13, T12, T18 | Gate G3 |
| G-06 king-of-the-court queue order (default *back*) | Open | T13 (king ladder) | Gate G3 |
| Q2: how many scorers at once (proposed: coach + 1–2) | Proposed | T21 (role matrix) | Gate G4 |
| **E9 gym network test** | Pending | T19 type sizes, T25 | Replaced by **M0** below (proposal) |
| E10 live dry-run | Pending | Only tunes S13 defaults; D-022 (Firm) is already accepted | Nice before T15, not blocking |
| **E3 ten real games** | Waiting on AJ | **T26**: a real practice can't run on stress games alone | Must be encoded **before M5** |
| E6 practice-01 | Template only | T23/T26 need one real practice YAML | Before M2 (as a YAML file) |

**The real long pole is E3, not code.** The exit criterion is "two weeks of *real* practices." That needs the games you actually run, encoded as fixtures and passing `p0_check.py`. Encoding them is about one evening of describing the games and then my encoding work, and it can happen alongside Stage A. Start it now.

---

## 2. Proposed changes to the task graph

### P-1 · Add **T00: tracer bullet** (network spike in real code) · ✅ *Accepted, applied*
The roadmap chose live-first "to test the network risk first," but in `P1-tasks.md` the first network code is T18, roughly two-thirds of the way through. T00 moves that test to week 1.

- **What:** a throwaway `spikes/tracer/` (outside the workspace crates, deleted at T18). axum + WebSocket on `0.0.0.0:7878`; a Dioxus **web** build served by the desktop exe; `/board` shows two scores and a countdown; `/remote` has `+1 / +2 / +3 / undo` per team; hub state in memory only.
- **Why this specific shape:** it proves the four things that could sink P1 and that no unit test catches:
  1. phone → laptop over your router, with Windows Firewall in the way (this *is* E9 §2, done with the real stack);
  2. **building the wasm bundle and embedding it in the desktop exe** (`dx build --platform web` → `include_dir!`/`rust-embed` → axum static files). T18 assumes this works. It's fiddly, so find the problems first;
  3. TV readability with real fonts and real rendering (E9 §3, using the mock-board sizes);
  4. a tap round-trip you can *feel* on the floor (latency).
- **Bonus:** it's a usable dumb scoreboard from week 1. Use it at practice while P1 is being built. Every time a scorer wishes it had a button, that's free S12 data. Log it in `validation/gym-network.md` + `gap-log.md`.
- **Guardrail against drift:** T00 has **no domain types, no rules, no store**. It must not grow. If it's tempting to add a feature, the feature belongs in a real task.

### P-2 · Decouple the hub (T18) from InfiniteDB (T12) · ✅ *Accepted, applied*
Today T18 waits for T10 → T11 → T12. That puts all of storage on the critical path. The hub only needs the `Store` trait and the store-thread *channel API*:
- T18 depends on **T10 + T14 + T17** and runs on `MemStore` behind the same store-thread channel.
- `IdbStore` (T11, T12) drops in behind the channel when it's ready. Swapping it in is a one-line constructor change plus re-running T18's integration test.
- T12 becomes a prerequisite of **T22** (crash recovery), where durability actually matters, and of **M4**.
- Effect: the storage lane comes off the critical path, saving about 3 task-lengths, and the M2 gym milestone happens earlier.

### P-3 · Hidden dependencies to add to `P1-tasks.md` · ✅ *Accepted, applied*
| Task | Add dependency | Why |
|---|---|---|
| T18 hub server | **T14** clocks | The hub runs blocks, so it needs clocks |
| T19 Board | **T09** renderer, **T14** clocks | The Board shows the S2 §7 short form and the offset-corrected clock |
| T20 Remote | **T16** buttons | Remote buttons come from T16 (D-014) |
| T23 Studio Practice | **T09** | Blocks show game card lines |
| T26 practice week | **T15, T22, T23, T24, T25** + **E3** real games | Everything must be in the build that goes to the gym |

### P-4 · Split the four tasks that are too big for one session · ✅ *Accepted, applied*
One task = one session = one PR only works if the task fits. These four won't:
| Task | Split |
|---|---|
| T05 Rules | **T05a** types + serde · **T05b** validator (S2 §8), one failing-case test per invariant |
| T13 Log + fold | **T13a** entries, fold, scores, possession, undo, implied events (D-035) · **T13b** game end + rotation + king ladder + the prefix/rest property test |
| T19 Board | **T19a** layouts (running / two baskets / last minute / OT / rest / between) · **T19b** clock offset sync + the 10-minute drift log |
| T20 Remote | **T20a** pairing, tap queue, ack states, reconnect · **T20b** layouts A–D, undo list, vibrate |

### P-5 · Fix stale cross-references (doc hygiene, no behaviour change) · ✅ *done*
- `decisions.md` D-036: "Measured in P1 task **T10**" → **T12** (S14-T5 is T12's acceptance).
- `S14` §1.3: "Readiness task (P1 **T09a**)" → **T11a**.
- `validation/gym-network.md` §4: "First-run setup step in P1 (**T16**)" → **T25**.
- `05-p0-plan.md` §8 W5.3/5.4 still describe the pre-D-016 order (designer first). Mark it superseded by `P1-tasks.md`.

---

## 3. Milestones: each one ends in the gym

Each milestone produces something you take to a real practice. Unit tests prove the code matches the spec; only the gym proves the spec matches the floor.

| M | Name | Tasks (with P-1…P-4) | What you take to practice | What it proves / unlocks |
|---|---|---|---|---|
| **M0** | Tracer bullet | T01, **T00**, T11a | Dumb scoreboard: phone taps +1/+2/+3, TV shows score + countdown | E9 closed with real code; wasm-embed pipeline works; firewall procedure known |
| **M1** | Domain spine | T02–T09 | — (desk milestone) | All 13 fixtures load, resolve, render; **Rust renderer output = `p0_check.py` output, byte for byte** |
| **M2** | Clock Board | T10, T13a/b, T14, T15, T18, T19a/b, T24 | TV runs the **practice timer** from a practice YAML: block names, game short-form rules, warn/horn/grace (Firm, D-022). Head coach drives it from the laptop keyboard. **No phone yet** | Schedule guard on the floor; the planned-vs-actual record starts **here**, weeks before the exit test |
| **M3** | Phone scoring | T16, T17, T20a/b, T21 | Rule-generated buttons on your Android phone; assistant gets a scorer PIN | D-014 on the floor: are the generated buttons the ones scorers need? |
| **M4** | Durable hub | T11, T12, T22, T25 | Same as M3, but now kill the app mid-block on purpose at practice and recover | D-036 latency on your laptop; crash recovery; fresh-account setup < 2 min |
| **M5** | Field release | T23, T26, T27 (+ E3 real games) | Plan in Studio, Run ▶, whole practice on the TV from the phone | **Exit criterion:** two weeks of practices, no lost scores, no drift, planned vs. actual logged, every fixture round-trips through InfiniteDB |

M2 before M3 is deliberate. The clock and the schedule guard are what keep practice moving (D-015), and they're useful with zero phone risk. If M3 slips, practices still benefit from M2.

---

## 4. Lanes and waves

Four lanes. At any moment aim for **2–3 tasks unblocked**, so a session never sits idle waiting on a review.

- **D: Domain** (`ac-domain`, `ac-sport-basketball`): pure, the spine
- **S: Storage** (`ac-store-idb`): off the critical path after P-2
- **L: Live logic** (`ac-live`): pure, test-heavy
- **C: Clients/server** (`ac-app` hub, `ac-web` Board/Remote, Studio)

| Wave | D | S | L | C | Gate before the wave | Milestone |
|---|---|---|---|---|---|---|
| 0 | — | — | — | — | **G0**: accept D-017/18/19/23; `git init`; repo on GitHub; CI runner chosen | — |
| 1 | T01 | T11a | — | **T00** | — | **M0** at the next practice |
| 2 | T02 ∥ T04 | — | — | — | **G1**: accept D-025…D-034 | — |
| 3 | T03 ∥ T05a → T05b | — | — | — | — | — |
| 4 | T06 → T07 → T08 | — | — | — | — | (spine: strictly serial) |
| 5 | T09 | T10 | T13a | — | **G3**: accept D-035, D-036; answer G-06 | M1 when T09 lands |
| 6 | T16 | T11 *(G2 first)* | T13b → T14 | — | **G2**: D-024 + S14 Q1/Q2 | — |
| 7 | — | T12 | T15 ∥ T17 | T18 (MemStore) | — | — |
| 8 | — | — | — | T19a → T19b → T24 | — | **M2** |
| 9 | — | — | — | T20a → T20b → T21 | **G4**: Q2 scorers | **M3** |
| 10 | — | — | — | swap IdbStore in · T22 · T25 | — | **M4** |
| 11 | — | — | — | T23 | E3 real games encoded + one real practice YAML | — |
| 12 | — | — | — | T26 → T27 | — | **M5 / exit** |

### Critical path (with P-1…P-4)
`T01 → T04 → T05a → T05b → T06 → T07 → T08 → T13a → T13b → T14 → T18 → T20a → T20b → T21 → T22 → T26 → T27`

Storage (T10 → T11 → T12) runs alongside and must land before T22. The domain spine T04→T08 is pure serial work, and roughly the first third of P1 rides on it. That's where review discipline matters most: a drift in T05 propagates into everything.

---

## 5. Size and pace

Sizes are in Cursor sessions, *including* fix-ups after review. S ≈ 1, M ≈ 1–2, L ≈ 2–3.

| Size | Tasks |
|---|---|
| S | T03, T04, T11a, T24 |
| M | T00, T01, T02, T05a, T06, T07, T09, T10, T12, T13b, T14, T15, T16, T17, T19b, T20b, T21, T22, T25 |
| L | T05b, T08, T11, T13a, T18, T19a, T20a, T23 |

Total ≈ **45–50 sessions** before T26. M2 lands at about session 30.

| Pace | M0 | M2 (Clock Board in the gym) | M5 starts (T26) |
|---|---|---|---|
| 3 sessions / week | week 1 | ~week 10 | ~week 16 |
| 5 sessions / week | week 1 | ~week 6 | ~week 10 |
| 7 sessions / week | week 1 | ~week 4–5 | ~week 7 |

These are rough guesses, not commitments. Recalibrate after Wave 4: if the spine took twice the estimate, the estimate is wrong, not the plan. **Map these weeks onto the season calendar.** The exit criterion needs two weeks of real practices, so M5 must start with at least 3 weeks of season left: week 1 is T26, the T27 fixes go in between, week 2 must run clean, and the third week is slack in case week 2 doesn't.

---

## 6. Session protocol: how each task runs without drift

Past projects drifted when Cursor filled gaps with its own inventions. The protocol makes gaps loud instead.

### 6.1 Task packet (paste into each Cursor session)
```
TASK  Txx — <title>                         branch: p1/txx-<slug>
READ  docs/spec/<Sx>.md §<n>…  ·  docs/decisions.md <D-ids>  ·  docs/plans/review-checklist.md
DO    <the task row from P1-tasks.md, verbatim>
DONE  <acceptance tests, verbatim> + `cargo fmt --check`, `clippy -D warnings`, `cargo test` green
NOT   no new crates/deps beyond: <list> · no edits to docs/fixtures/** · no types outside the named crate(s)
STOP  If the spec is silent or ambiguous: do NOT choose. Write the question at the end of
      docs/validation/gap-log.md under "P1 implementation gaps", stub with `todo!("G-xx")`,
      and finish the rest of the task.
```

### 6.2 Review loop
1. Cursor opens the PR; the description lists the acceptance tests and any `G-xx` stubs.
2. **Claude reviews** against `review-checklist.md` + the task's specs, the same way you review InfiniteDB PRs. Paste the PR link or diff here. Findings are ranked; boundary violations (D-005, D-017, D-019) are always blocking.
3. Answer any `G-xx` gaps. Each one becomes a decision or a spec edit **before** the next task that touches it.
4. Merge. Tick the task. Update `P1-tasks.md` status.

### 6.3 Standing rules
- **Oracle in CI from T09 on:** CI runs `python tools/p0_check.py docs/fixtures/games` and diffs its card/board/button output against the Rust renderer. Two implementations agreeing is a much stronger test than one implementation matching hand-written `expect:` blocks.
- **Fixtures are frozen** (checklist rule). Changing one is its own PR, with a reason and `p0_check.py` still passing.
- **Gym logs become fixtures.** Every real practice from M2 on writes a session log. Export each one to `docs/fixtures/logs/<date>.yaml`, and the fold property test replays all of them. By T27 you'll have a regression suite of *your own practices*.
- **Gate checks are explicit.** A task whose decisions are still *Proposed* doesn't start. Acceptance is a 2-minute read of the decision rows, done in batches at G0–G4.

---

## 7. Tripwires (decide in advance what triggers a rethink)

| Tripwire | Threshold | Pre-agreed response |
|---|---|---|
| T00 fails in the gym (phone can't reach laptop reliably) | > 1 failed reload in 10 at center court after the firewall fix | Stop C-lane planning. Try the router's 5 GHz/2.4 GHz split, a fixed DHCP lease for the laptop, and a port-based firewall rule (not program-based, since rebuilds change the exe). Write a decision before T18 |
| wasm-embed pipeline in T00 is painful | > 2 sessions | Fall back to serving `ac-web/dist` from disk next to the exe for P1; embedding becomes a P7 polish item (new decision) |
| Rule grammar is bigger than it looked | T05b > 3 sessions | Re-read D-033; the grammar is probably being generalised beyond S2 §3. Cut back to the spec; don't extend |
| Resolver/renderer mismatch with the oracle | Any fixture differs after 2 fix attempts | Decide which one is wrong *in the spec* first, then fix the code (or the Python) |
| D-036 latency | S14-T5 p95 > 150 ms on your laptop | Pre-agreed: batch syncs every 250 ms (D-036 fallback) + superseding decision |
| Clock drift | Board > 100 ms from the hub over 10 min (T19b) | Check offset estimation (ping/pong RTT/2) before blaming the network |
| InfiniteDB API shifts under you | The local tree moves during P1 | Pinned version wins (D-024). Upgrades are a separate PR that only touches `ac-store-idb` and must pass the contract suite |
| Phone screen sleeps mid-scoring | Happens at M3 | Android Chrome setting or a screen timeout increase for v1 (`06` §8); add a "screen asleep?" banner on reconnect |
| Scope creep in T00 | Any domain concept appears in `spikes/` | Delete it. It belongs in a task |

---

## 8. Answers needed from AJ, in the order they're needed

| When | Question |
|---|---|
| ~~Now (G0)~~ | ✅ D-017, D-018, D-019, D-023 accepted; P-1…P-5 accepted (2026-10-28) |
| Now | **Season dates:** first practice, last practice, and how many practices per week? (Sets where M2/M5 must land) |
| Now | Rough Cursor pace: sessions per week? |
| Now (E3) | Describe the 10 games you actually run; I'll encode them and log any gaps |
| Before T05 (G1) | Accept D-025…D-034 (most came straight from the stress set) |
| Before T11 (G2) | ~~S14 Q1~~ answered by T11a (D-037: local = GitHub `d193986`, crates.io has no 0.4.1). Still open: accept D-024 as amended; S14 Q2: one DB per install? |
| Before T13 (G3) | Accept D-035/D-036; king-of-the-court loser goes to the **back** of the queue (G-06 default)? |
| Before T21 (G4) | Confirm coach + 1–2 scorers at once (Q2) |
| Before M5 | One real practice written as YAML (E6) using real games |

---

## 9. First three sessions

1. **Wave 0 (no code, about 30 min):** accept G0 decisions + proposals; `git init` at `D:\Rust\Dioxus\assistant-coach`, push to GitHub, add `.gitignore` for `target/` and `dist/`.
2. **T01 workspace scaffold:** root workspace, six empty crates, JoInn template into `crates/ac-app` with `app-shell` pinned at a tag, CI script. Acceptance: `cargo test` green; `cargo run -p ac-app` opens the shell.
3. **T00 tracer bullet + T11a readiness**, in parallel. Then take the dumb scoreboard to the next practice and fill in `gym-network.md`.
