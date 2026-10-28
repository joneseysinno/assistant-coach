# 05 — P0 Plan: Theory & Spec to "Ready to Build"

> Status: **in progress** (desk work drafted 2026-10-28) · Scope: everything that has to be true before the first line of P1 code.
>
> **Progress against the exit criteria**
>
> | # | State | Where |
> |---|---|---|
> | E1 | Drafted: D-016, D-020–D-022 accepted from AJ's answers; D-017–D-019, D-023–D-036 proposed; remaining open questions given proposed defaults | `decisions.md` |
> | E2 | Drafted: S1–S14 all written (S7 light) — need AJ's review | `spec/` |
> | E3 | **Waiting on AJ:** the 10 real games | `validation/gap-log.md` (bottom table) |
> | E4 | Done on paper: 12/12 stress games encoded; every gap has a decision or spec change; 2 small items open for AJ | `fixtures/games/stress/`, `validation/gap-log.md` |
> | E5 | Format examples only (Horns, 5-out concept); **waiting on AJ's real sets** | `fixtures/plays/` |
> | E6 | Template ready; needs real games + roster | `validation/practice-01.md` |
> | E7 | Printables ready; **floor test pending** | `printables/paper-prototype.html`, `validation/paper-prototype.md` |
> | E8 | Drafted (live-first order, D-016 B) | `plans/P1-tasks.md`, `plans/P2-tasks.md`, `plans/review-checklist.md` |
> | E9 | Mock Board + checklist ready; **gym test pending** | `printables/mock-board.html`, `validation/gym-network.md` |
> | E10 | S11–S13 drafted; dry-run sheets ready; **dry-run pending** | `validation/live-dry-run.md` |
>
> Checker: `python tools/p0_check.py docs/fixtures/games` (13 fixtures, 0 failing). Printables: `python tools/make_printables.py`.
> P0 is done when an implementer (you or Cursor) can build P1 **from the docs alone**, without needing to ask "what did we mean by…?"

---

## 1. Why P0 deserves real work

Past projects drifted when the idea was only fuzzy at build time. Docs 01–04 and 06 settle the *direction*. They do not yet settle the *details* that code will need:
- the exact event list,
- the rule grammar,
- the court coordinates,
- how a dial plugs into a rule,
- what a variant delta contains,
- how files are laid out.

If those are left open, Cursor will invent answers, and each one becomes a small drift. P0 closes them on paper, then **tests the paper model against real practices**. A wrong primitive caught at that stage costs an hour; caught in P2 it costs a refactor.

**P0 has three outputs:**
1. **Decisions** — every open question answered and recorded.
2. **Specs** — one short spec per part of the model, each ending in worked examples that become P1's test data.
3. **Validation** — real games, plays and a real practice, encoded and run on paper with no code.

---

## 2. Exit criteria (P0 is done when…)

| # | Criterion | Evidence |
|---|---|---|
| E1 | Every decision D-001…D-016 is accepted, changed or rejected, and the open questions (`04` §4) are answered | `docs/decisions.md` with status and date for each |
| E2 | Specs S1–S10 and S14 (below) are written and reviewed | `docs/spec/` folder |
| E3 | **10 real games** from current practices are encoded in the model **with no "notes-only" workarounds** for anything that matters | `docs/fixtures/games/*.yaml` + a gap log |
| E4 | The **stress set** (12 tricky games, §5.2) is encoded, or each failure has a decision record | Fixtures + decisions |
| E5 | **2–3 plays or concepts** are encoded as decision graphs, and each read is linked to a game | `docs/fixtures/plays/*.yaml` |
| E6 | **One real practice** is planned in the model. Active time and estimated load are worked out by hand, the practice is run, and the plan is compared with what happened | `docs/validation/practice-01.md` |
| E7 | **Paper prototype tested at practice** (constraint deck + game cards + a tally sheet), with findings logged | `docs/validation/paper-prototype.md` |
| E8 | P1 **and P2** (live first, then designer — D-016) are broken into **Cursor-sized tasks** (one session/PR each), with acceptance tests that point at the fixtures | `docs/plans/P1-tasks.md`, `docs/plans/P2-tasks.md` |
| E9 | **Gym network test** done with your real laptop, router, Android phone and TV: phone reaches the laptop, firewall handled, latency acceptable, test Board readable from the far baseline | `docs/validation/gym-network.md` |
| E10 | Specs S11–S13 (live protocol, Remote/Board layouts, schedule guard) written; **one practice "dry-run" on paper**: scoring with a mock button sheet, timing by stopwatch, overtime decisions logged | `docs/spec/`, `docs/validation/live-dry-run.md` |

---

## 3. Workstreams

```
W1 Decisions ──┐
               ├──► W2 Specs ──► W3 Fixtures (paper encoding) ──► W5 P1 handoff
W4 Field tests ┘         ▲                │
 (at practice)           └── gap log ◄────┘
```

W1 and W4 can start right away. W2 and W3 loop: every gap found while encoding goes back into a spec or a decision.

---

## 4. W1 — Decisions

Turn `04` §2–3 into a living `docs/decisions.md`. Format per entry: **ID · decision · why · alternatives considered · status · date**.

| Step | Item | Notes |
|---|---|---|
| 1.1 | Answer the open questions in `04` §4 (users, sets vs. concepts, baskets/court size, imports, persistence, rule editor, load tracking, standalone phone mode, P1/P2 order, shot clock, overtime default). The courtside device is already answered | Five minutes of your answers settle a lot of the specs |
| 1.2 | Accept, change or reject D-001…D-016 | |
| 1.3 | Settle the modeling questions in `02` §10 (rule expressiveness, neutral players, scout packs, individual constraints) | Likely answered during W3 encoding — leave open until then |
| 1.4 | New decisions from the gap log | Expect 5–15 |
| 1.5 | **Repo layout decision**: one app crate generated from the template, or a workspace | Proposal: workspace with `ac-domain` (pure data + store interface, no Dioxus), `ac-store-idb` (InfiniteDB adapter), `ac-live` (hub logic), `ac-app` (Dioxus Studio + hub server), `ac-web` (Board/Remote wasm). The domain crate can be tested and reused (e.g., by an MCP server later) without UI |

---

## 5. W2 — Specs

Each spec is short (1–3 pages) and has the same shape: **purpose · definitions · fields · invariants · examples · open questions**. The examples are written as fixtures so they become P1 unit tests.

| Spec | Covers | Key questions it must answer | Needed for |
|---|---|---|---|
| **S1 Events** | The basketball event vocabulary | Final list. Fields each event carries (actor role, zone, time, possession id). Which events a coach can **tally live** and which can only be **derived** or **reviewed on video** | Rules, observation |
| **S2 Rule grammar** | When → Then | Pattern forms (single event; event + condition; possession-scoped sequence). Conditions (zone, role, count, clock, score). Consequences (points, turnover, possession change, restart, role enters/leaves, constraint on/off, dial change). Precedence when two rules fire. Plain-English rendering templates | Game, cards, Remote buttons |
| **S3 Space & court** | Geometry | Coordinate system: feet, origin at the center of the basket, +y toward half court. Court presets (HS 84×50, MS 74×42, NBA/NCAA 94×50, custom). Named regions from the pack (paint, corners, wings, top, above-the-break, restricted arc). Zone shapes (rect, circle/arc, polygon, named region). Cross-court and multi-basket layouts | Court canvas, area/player |
| **S4 Format** | Base game | Roles and teams. Start types (check-ball, coach-pass, closeout start, rebound start, transition start). Restarts. Rotation formats (winners stay, make-it-take-it, 3-team rotate, round robin, stations, king of the court). End conditions (score, time, possessions) | Game, practice |
| **S5 Constraint, Dial, Variant** | Composition | Constraint metadata (category, invites, preserves, targets, stage). Dial types (int range, distance, enum, bool, duration). **How a dial is referenced inside a rule or zone.** What a variant delta holds (dial overrides, constraint toggles, added constraints). Inheritance when the parent changes | Game, progressions |
| **S6 Learning targets & reads** | Taxonomy | Final tree with **stable ids**. Read structure (cue, options, actor, which defender's behavior is the cue). The character targets | Tagging, coverage, plays |
| **S7 Play graph** *(light)* | Plays | State, transition, guard, loop. Enough to prove the primitives cover plays; full UX deferred to P4 | P4 (validates D-002 now) |
| **S8 Practice & season** | Time | Block fields. Time accounting. **Active-ratio formula** with rotation formats. Group generation. Season grid units | P2 |
| **S9 Estimates** | Heuristics | Exact formulas for area/player, intensity band, involvement density, active ratio and load units. Every input, assumption and confidence rule, with the research behind each | Predicted-effects panel |
| **S10 Observation & storage** | Data | Pinpoint definition (event or read outcome + opportunity). Count-per-minute timing. **Your exact 7/8 criterion definition**. How observations link to blocks, players and the live log | P4 |
| **S11 Live protocol** | Hub ↔ devices | Request and state message types. Log entry format. Clock math (start + paused + duration; client offset). Ids and idempotency. Roles and PIN pairing. Reconnect and replay. What happens if the laptop restarts mid-block | P2 |
| **S12 Remote & Board** | Live screens | Remote layouts (one basket, two baskets, between blocks). How buttons are generated from rules, including violations and judgment-call rules. Board layouts and states (running, last minute, overtime, rest, water break). Type sizes for viewing distance. Team colors | P2 |
| **S13 Schedule guard** | Time discipline | Warning settings. Soft/firm/locked policies. **The cut algorithm** (which blocks shrink, by how much, minimum lengths). Protected blocks. What the planned-vs-actual record holds | P2, P4 |

| **S14 InfiniteDB mapping** | Storage | **First, an InfiniteDB readiness check:** which features are stable today vs. in redesign; which version/commit P1 pins. Then node kinds, edge and hyperedge kinds, how the court and zones map to spatial regions, play transitions as hyperedges, path traversal → animation frames, the live log, schema versioning/migration. The store interface (D-017) in domain terms, and the list of graph queries the app needs (`02` §11) | P1 (blocking) |

**Suggested writing order:** S3 → S1 → S2 → S4 → S5 → S6. That is the minimum needed to encode games. **S14 starts in stage 1 alongside them**, because the InfiniteDB readiness check could change P1's scope. S7–S10 can follow in parallel with W3.

---

## 6. W3 — Fixtures: encoding real things by hand

This is the core of P0: **use the model before building it.**

### 6.1 Ten real games

Pick 10 games already used at practice, covering a spread: 1v1, 2v2, 3v3, overloads, a transition game, a defense-first game and a character-scoring game. For each, write a fixture file (YAML is fine; it's for reading, not a final format):

```yaml
id: g-closeout-attack-3v3
base:
  space: { court: half, zones: [corner-L: bonus, corner-R: bonus] }
  roles: [O1, O2, O3, D1, D2, D3]
  start: closeout-start { passer: D, closeout: live }
  restart: { on_stop: switch, on_score: switch }
  end: { score: 7 }
constraints:
  - id: kick-out-bonus
    category: task
    rules:
      - when: make(shot.three) after paint-touch in possession
        then: points(3)
    invites: [drive-and-kick, extra-pass]
    preserves: [help-defense, closeout]
    targets: [offense.extend.drive-and-kick]
  - id: dribble-limit
    dials: { limit: { int: [1, 4], default: 2 } }
    rules:
      - when: dribbles(player) > $limit
        then: turnover
stage: team-based
reads: [read.closeout.high-fast→drive, read.closeout.short→shoot, read.help-steps→kick]
observables: [closeout-decision-correct / closeout]
```

Each game gets a **gap log** entry for any part that only fits in a free-text note. Gaps go to W1 or W2. **Target: zero gaps for anything that changes how the game is played.**

### 6.2 Stress set: 12 games built to break the model

These are common in constraints-led coaching and are the most likely to expose missing primitives:

| # | Game | What it tests |
|---|---|---|
| 1 | **Delayed defender**: 3v2, the third defender starts on the baseline and enters after the first pass | Role entry triggered by an event |
| 2 | **Continuous advantage**: 3v2 one way → 2v1 back (or an 11-man-break style rotation) | Possessions that chain; roles that switch teams |
| 3 | **Coach calls the coverage**: 2v2 ball screen where the coach shows the defense a card | Hidden information; a coach-controlled variable |
| 4 | **Losing team picks the next rule** | Constraints that change during the game; player choice |
| 5 | **Shot clock scoring**: a score within 6 s of the catch = 2 | Time-based conditions |
| 6 | **King of the court** with 3–4 teams | Rotation, ladder state across games |
| 7 | **Neutral joker** who can't shoot | Neutral role + role-level limit |
| 8 | **Cross-court 2v2s** on side baskets | Multi-basket space layout |
| 9 | **Player handicap**: one player left-hand finishes only | Individual constraint within a team game |
| 10 | **Defense scores**: stop = +1, deflection = +1, charge = +2 | Defense-side scoring, character rules |
| 11 | **Two balls** (passing-pressure game) | More than one ball — in or out of scope? |
| 12 | **Zone-restricted defender**: the help defender may not leave the paint | Role restricted to a zone |

Each one either encodes cleanly or produces a decision: extend the grammar, or put it explicitly out of scope.

### 6.3 Plays

Encode 2–3 of the team's real sets or concepts as decision graphs (S7). At least one must be a looping **concept** (e.g., 5-out drive-and-kick rules), not just a linear set. For each read, link a game from 6.1, or write a new fixture using the "generate from read" recipe in `02` §6. This tests D-002 and R9 on real material.

### 6.4 One real practice

Plan one actual practice in the model:
- Blocks are fixtures from 6.1, with real groups from the roster.
- Calculate by hand: total time, unassigned time, **active time per player**, estimated load, and the target mix.
- Run the practice, then log what really happened (actual times, who sat, what changed).
- The comparison feeds S8 and S9.

---

## 7. W4 — Field tests at practice (paper prototype)

Test the *product idea* on paper before the code exists:

1. **Constraint deck.** Index cards: one base-game card plus one card per constraint (rule in plain English, dial written as "Level 1/2/3"). Stack them physically to build the game. *Tests:* are the constraints separable? Does the order matter? Do the plain-English rule renderings read clearly?
2. **Game cards for players.** Hand an assistant (or a player) a card and see whether they can run the game with no explanation. *Tests:* S2 rendering templates and the game card format.
3. **Tally sheet.** For one block, tally 2–3 pinpoints live (e.g., closeout decisions correct/total, paint touches). *Tests:* which S1 events are realistically tallyable mid-practice, and how many pinpoints a coach can track at once (probably 1–2).
4. **Dial turn.** Change one dial mid-game based on the success rate and note the effect. *Tests:* the challenge-point / metastability idea in practice.
5. **Capture.** When players invent something good, sketch it in 30 seconds on a court sheet. *Tests:* whether "harvest" is realistic during practice or belongs after it.

6. **Gym network test** (environment only, no app code). At the gym, with your laptop, router, Android phone and TV:
   - Laptop and phone on the router's Wi-Fi. Start any throwaway local web server on the laptop (e.g., `python -m http.server`) and open it from the phone using the laptop's IP. This checks the firewall, the IP and how reliable the Wi-Fi is in that gym.
   - Put a **static mock Board** (an HTML page or image I can make from `06` §6) full screen on the TV. Check what's readable from the far baseline and the far corner. Note the size where the score, clock and rule text stop being readable.
   - Note how far the phone can be from the router before it drops.
7. **Live dry-run on paper.** Run one practice with a printed **Remote button sheet** per game (tick marks instead of taps), a stopwatch for blocks, and the schedule guard rules applied by hand (warn at 1 min; at zero decide "next" or "+1 from block X"). Record planned vs. actual minutes. *Tests:* whether the button layout matches what scorers actually need, how often you'd want to overrun, and which policy (soft/firm/locked) you'd choose.

Log findings in `docs/validation/paper-prototype.md`, `gym-network.md` and `live-dry-run.md`. These results decide the Remote and Board layouts (S12) and the guard defaults (S13).

---

## 8. W5 — P1 handoff

| Step | Output |
|---|---|
| 5.1 | Update 02/03 with everything W1–W4 changed (the docs stay the source of truth) |
| 5.2 | Lock the fixture format as the P1 test corpus: every fixture must load, round-trip (save → load → equal) and render its game card text |
| 5.3 | *(Superseded by D-016 B and `plans/P1-tasks.md` + `plans/P1-execution.md` — the draft orders in 5.3/5.4 are the pre-swap ones.)* **P1 task list** in `docs/plans/P1-tasks.md`, each task sized for one Cursor session/PR, with acceptance tests pointing at fixtures. Draft order: generate app from template → `ac-domain` types from S1–S6 → store interface → `ac-store-idb` adapter on the pinned InfiniteDB → fixture loader + YAML → DB → domain round-trip tests → rule renderer (plain English) → court SVG (S3) → Game doc type: base + constraint stack → zones/roles on canvas → dials + variants → predicted effects (S9) → game card export |
| 5.4 | **P2 task list** in `docs/plans/P2-tasks.md`. Draft order: `ac-live` hub logic as pure Rust (log, clocks, requests → state) with tests replaying fixture logs → embedded server + WebSocket in the laptop app → Board (wasm) → Remote (wasm) with buttons from rules → pairing/roles → schedule guard → crash recovery → second-screen Board window and sound |
| 5.5 | **Review checklist** for checking what Cursor lands: rule numbers (R1–R15) and decision ids (D-001…D-016) to verify on each PR |

---

## 9. Sequence

P0 work is mostly writing and practice-floor testing. Practices set the pace, so the steps below are stages, not calendar dates.

| Stage | Desk work | Practice-floor work |
|---|---|---|
| **1** | W1.1–1.2 answers (incl. D-016); S3, S1, S2 drafted | **Gym network test** (W4.6), done first because it's the biggest technical unknown; constraint deck + one game card (W4.1–2) |
| **2** | S4–S6; encode the 10 real games; gap log; S11 drafted | Tally sheet + dial turn on 2–3 games (W4.3–4) |
| **3** | Stress set; S5 revised from gaps; S12–S13 drafted; S7 + plays encoded | **Live dry-run on paper** (W4.7); capture test (W4.5) |
| **4** | S8–S10; real practice planned in the model | Run that practice; compare (6.4) |
| **5** | Decisions closed; docs updated; P1 + P2 task lists | — |

---

## 10. Files P0 will add

```
docs/
  05-p0-plan.md            ← this file
  decisions.md             W1
  spec/
    S1-events.md … S14-infinitedb-mapping.md
  fixtures/
    games/*.yaml           10 real + 12 stress
    plays/*.yaml           2–3
    practices/practice-01.yaml
  validation/
    gap-log.md
    paper-prototype.md
    practice-01.md
    gym-network.md
    live-dry-run.md
  plans/
    P1-tasks.md            W5
    P2-tasks.md            W5
```

---

## 11. Where I can do the heavy lifting

- **Drafting specs S1–S14** from these docs, for you to review and correct.
- **Turning your game descriptions into fixtures.** Describe a game the way you'd tell an assistant coach, and I'll encode it and log the gaps.
- **Making the paper prototype printables:** constraint-deck cards, game cards, tally sheets, Remote button sheets, and a static mock Board for the TV test.
- **Writing the P1 task list** in the phased, one-PR-at-a-time format that works with Cursor.

**What only you can supply:** the answers to the open questions, the 10 real games, the team's sets, and what actually happens on the floor.
