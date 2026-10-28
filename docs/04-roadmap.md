# 04 — Roadmap, Decisions and Open Questions

> Status: draft for review (rev 3: D-016 decided — live first; P1 and P2 swapped) · Scope: build order, exit criteria, decision record, and the questions that need an answer before code starts.

---

## 1. Scope in one paragraph

Assistant Coach is used in two places:
- **At the desk (Studio):** design constraint-led games, turn plays into reads, plan practices and seasons.
- **In the gym (Live):** the laptop is the hub, the TV is the scoreboard players read, and phones/tablets are remotes. Coach and assistants keep score with buttons generated from each game's rules. The app runs every clock and keeps practice on schedule. Everything scored becomes observation data.

Basketball first. Laptop, tablet and Android phone. No internet required in the gym.

---

## 2. Build order and why

Two parts must exist before anything is usable: the **domain model** (games, constraints, rules) and the **live hub**. The networking (phone → laptop → TV) is the largest *technical* risk. The game designer is the largest *product* idea. D-016 chose **live first**: games are written as fixture files at first, so a real practice runs on the TV as early as possible and the network risk is tested first; the designer follows.

| Phase | Deliverable | Exit criterion |
|---|---|---|
| **P0 — Theory & spec** | Docs 01–06, specs S1–S14, fixtures, paper prototype, gym network test. See `05-p0-plan.md` | All P0 exit criteria (E1–E10) met |
| **P1 — Foundations + Studio** (D-038) | Workspace; `ac-domain` (types, resolver, practice metrics, play graph), basketball pack (court, vocab, renderer, estimates, templates), `Store` + InfiniteDB adapter; **Studio**: Game designer (stack + rollback, format/rule/dial forms, zones, checks, predicted effects, card export), Practice planner (blocks, metrics, printed plan), Play designer (graph, drag, playback, reads → game), Library, Settings. Tasks: `plans/P1-tasks.md`; execution: `plans/P1-execution.md` | **AJ builds his real games (E3), one real practice (E6) and the team's sets in the app**, prints cards/plans and uses them at practice; everything round-trips through InfiniteDB |
| **P2 — Live practice** (was P1, D-016 → D-038) | `ac-live` hub (log, fold, clocks, schedule guard, buttons from rules); hub server in the laptop app; Board (TV) and Remote (phone/tablet), pairing + roles, crash recovery; Studio “Run” on a Practice doc. Tasks: former live tasks in `plans/P1-tasks.md` (T12–T26) | **Two weeks of real practices run on the TV from the phone**, with no lost scores and no clock drift; planned-vs-actual minutes logged |
| **P3 — Practice planner depth + Roster** (v1 planner shipped in P1) | Timeline across baskets, groups from roster, rotation formats, active time per player, estimated load, target mix, interleaving check; Library/Roster/Targets panes; drag from library | A real week planned in the planner; every player at or above the target active ratio |
| **P4 — Review** | Observation charts from the live log (count per minute, Standard Celeration Chart, 7/8 markers); challenge-point suggestions; overtime report (planned vs. actual by game); estimate calibration | At least one progression and one schedule change made from the data |
| **P5 — Play designer depth** (v1 shipped in P1) | Play doc type as decision graph; court/graph views; playback; decompose → reads; generate game from read; harvest from live "capture" | The team's sets/concepts entered; every read trained or knowingly flagged |
| **P6 — Season** | Weeks × targets heat map, PoST stages, coverage/spacing, 14-week import | Season plan with no untrained priority targets |
| **P7 — Assist & reach** | Problem-first entry, constraint recipes, metastability tuner, scout packs, MCP/AI generation, optional joinn editor; **standalone phone mode** (no laptop); optional native Android / local HTTPS | Coach goes from "we stand in the post" to a runnable session in under 5 minutes |

Deliberately **not** in v1: video, cloud sync between homes and gyms, opponent game stats, other sports.

---

## 3. Decision record

> Summary only. The full record with dates is `decisions.md`, which now runs to D-037 (D-017–D-036 from P0: storage, gyms, shot clock, overtime default, workspace, rule-grammar extensions from the stress set; D-037 from P1 T11a: InfiniteDB pin + workarounds).

| ID | Decision | Rationale | Status |
|---|---|---|---|
| D-001 | Game designer comes before the play designer | CLA emphasis; plays depend on games | Accepted |
| D-002 | Plays are decision graphs; animation is derived | Exposes reads; enables decompose/generate | Accepted |
| D-003 | Closed primitive set: Space, Role, Event, Rule, Read, Dial | Prevents drift | Accepted |
| D-004 | Scoring and limits are structured Rules over Events | Cards, live scoring buttons and pinpoints come from one source | Accepted |
| D-005 | Sport-agnostic core + basketball sport pack | Basketball first without lock-in | Accepted |
| D-006 | **Changed:** persistence is InfiniteDB from day one | Plays are graphs and animations derive from them; avoids a later migration | Changed |
| D-007 | Estimates are transparent heuristics, calibrated from observations | Trust and correctability | Accepted |
| D-008 | Measurement is decision/outcome based; no technique score | Aligned with CLA | Accepted |
| D-009 | Laptop is the hub and single source of truth | One clock, one score, no conflicts | Accepted |
| D-010 | Three surfaces: Studio (JoInn shell), Board, Remote (own touch/display layouts) | Each device gets the right UI | Accepted |
| D-011 | Remote and Board are browser clients served by the hub (wasm); native Android later | No app-store install, one codebase | Accepted |
| D-012 | Clocks only on the hub; stored as start + paused + duration | No drift; survives reconnect | Accepted |
| D-013 | All live actions are Events in an append-only log | Undo, crash recovery, observation data | Accepted |
| D-014 | Remote scoring buttons are generated from the game's Rules | Zero scoreboard setup; rules stay the single source | Accepted |
| D-015 | Schedule guard: soft/firm/locked policies, protected blocks, visible overtime cost | Keeps the coach from running long on one drill | Accepted |
| D-016 | ~~Order: live first (B)~~ — superseded by D-038 | — | Superseded |
| D-038 | **Order: Studio first.** P1 = foundations + Studio designers; P2 = live | AJ: can't test an app that isn't built; live needs real games made in the app | **Accepted 2026-10-28** |

---

## 4. Open questions

Answered or given a proposed default in `decisions.md` → *Answers to the open questions*. Still open for AJ: sets vs. concepts (Q3, needs the team's sets), and the proposed defaults for Q2, Q5, Q7, Q8, Q9. Spec-level questions are listed at the end of each spec and in `validation/gap-log.md`.

## 5. Risks and how the design handles them

| Risk | Mitigation |
|---|---|
| Gym network problems (firewall, Wi-Fi drops, IP changes) | P0 gym network test (W4.6); reconnect with idempotent taps; QR always shows the current address; the laptop can run everything alone |
| Phone screen sleeps / no wake lock over plain HTTP | Phone setting or Chrome flag for v1; local HTTPS or native Android later (`06` §8) |
| Scorekeeping too slow mid-practice | Buttons generated from rules, one tap each, undo; tested on paper in P0 |
| Scope drift into a general sports platform | Closed primitives, sport pack boundary, phase exit criteria |
| App quietly becomes a drill library | R1 linter, library sorted by learning target |
| Rule language grows into a programming language | Possession-scoped sequences only until a real game needs more |
| Estimates look authoritative but are guesses | Show formula + confidence; calibrate from logs |
| **InfiniteDB is mid-redesign** (intrinsic identity, cells, ordering), so its API may shift under P1 | Narrow adapter crate (D-017); pin a specific InfiniteDB version/commit for P1; fixtures round-trip tests catch breakage; S14 lists exactly which DB features P1 needs |
| Shell limitations (static toolbars, no persistence, no cross-doc drag) | App-side `store` and drag state; Board/Remote don't use the shell |
