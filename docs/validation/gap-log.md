# Gap Log

> Every place a real or stress game did not fit the model cleanly. Each gap goes to a decision (W1) or a spec change (W2). **Target: zero open gaps for anything that changes how a game is played.**
>
> Check tool: `python tools/p0_check.py docs/fixtures/games` — loads every fixture, validates it against S1–S6 and compares rendered cards/buttons with each fixture's `expect:` block. Last run 2026-10-28: **13 fixtures, 0 failing.**

Status: **Open** · **→ D-xxx** (became a decision) · **→ Sxx** (spec changed) · **Closed** (no change needed) · **Out of scope**

## Stress set (§6.2 of the P0 plan)

| # | Game | Result | Gap | Resolution |
|---|---|---|---|---|
| 1 | Delayed defender 3v2(+1) | Clean with decision | G-01: a rule players follow ("D3 joins after the first pass") needs no scorer button | → D-025 `judge: players` |
| 2 | Continuous 3v2 → 2v1 | Needed extension | G-02a: possessions chain, roles change team and the attack direction flips | → D-026 phases in Format |
| 2 | 〃 | Open | G-02b: teams are "lines of three", not Red/Blue — does anyone keep score? | **Open — AJ:** if scored, score by group; if not, it's a `tally` game (stops/makes per minute) |
| 3 | Coach calls the coverage | Needed extension | G-03: a variable the coach sets each possession, hidden from the offense | → D-027 live dials with `visible_to` |
| 4 | Losing team picks the next rule | Needed extension | G-04: constraints that switch on during play; a choice by players | → D-028 pool + `choose` |
| 5 | Score within 6 s of the catch | Clean | — (sequence + `within`, judged by coach; shot clock per game) | D-021, D-033 |
| 6 | King of the court, 4 teams | Clean | G-06: queue order when the king loses (back vs. front) | **Open — AJ** (S4 §10.1); default *back* |
| 7 | Neutral joker who can't shoot | Clean with decision | G-07: neutral role semantics | → D-032 |
| 8 | Cross-court 2v2 on side baskets | Clean with decision | G-08a: baskets vary by gym | → D-020 gym profiles |
| 8 | 〃 | → S2 | G-08b: "make one pass before you shoot" needs a negative pattern | → S2 §4 *count condition* (no negation needed) |
| 9 | Left-hand finishes only (one player) | Clean with decision | G-09a: individual constraints in team games | → D-031 block modifiers |
| 9 | 〃 | Open (minor) | G-09b: a game with *only* block modifiers has no constraint, so R2 can't be checked on the game | Proposed: R2 lint runs on the Block for modifiers |
| 10 | Defense scores (stop/deflection/charge) | Clean | — (`points … to: defense`) | — |
| 11 | Two balls | In scope with a limit | G-11: possession is undefined with 2 balls | → D-029 ball count; possession rules invalid |
| 12 | Help defender may not leave the paint | Clean with decision | G-12a: needs a zone-exit event | → D-030 |
| 12 | 〃 | → S5 | G-12b: **found by the checker** — the role-edit sugar compiled to a points rule judged by players, which S2 §8.5 forbids | → S5 §2.1: compiles to `judge: coach`, offense +1 |

## Found while writing the specs

| Id | Gap | Resolution |
|---|---|---|
| G-S1 | A tap is a rule firing, not a raw event — how does observation get events? | → D-035 implied events |
| G-S2 | Reads can't be scored correct/incorrect live unless the cue is known | **Open — AJ** (S6 §8.1); proposed ✔/✘ tap live, cue-split on video |
| G-S3 | Board needs short phrases, card needs full sentences | → S2 §7 Board short form (first 3 constraint rules, stack order) |
| G-S4 | Remote button order: by constraint or by base rule first? | → S12 §2 base rules first (same place in every game) |
| G-S5 | Two possession-changing consequences on one tap (stop +1 then charge) | → S2 §6.4 first wins + validator warning |
| G-S6 | Live log needs durability; InfiniteDB is a blocking API | → D-036 store thread + sync per tap |

## P1 implementation gaps

> Cursor's STOP rule (`P1-execution.md` §6.1): when a spec is silent, log the question here as `G-Pxx`, stub with `todo!("G-Pxx")`, and finish the rest of the task. Each gap gets a decision or spec edit before the next task that touches it.

| Id | Task | Gap | Resolution |
|---|---|---|---|
| G-P01 | T11a | crates.io has no `infinite-db` 0.4.1 | → D-037: AJ pinned crates.io `=0.4.0` |
| G-P02 | T11a | Default 8 bits/dim: 0.4.0 aliases coordinates > 255 onto smaller ones and overwrites them (IDB-B3) | → D-037: 32 bits/dim; S14 §2 |
| G-P03 | T11a | IDB-B1: `query_bbox` drops records | → D-037 workaround; S14 §6 |
| G-P04 | T11a | IDB-B2: `count_incident_edges_for_endpoint` returns 0 — 0.4.1 only | Not in 0.4.0; regression test guards the upgrade |
| G-P06 | T11a | 0.4.0 has no `register_or_get_space` | → D-037: `DuplicateId` = already registered |
| G-P05 | T00 | Chrome keeps a dead WebSocket "open" for > 5 s with Wi-Fi off, so the Remote kept saying *connected* | Abandon the socket on a 4 s heartbeat timeout → carried into T20a's spec line |

## Real games (W3.1) — waiting on AJ

| # | Game | Encoded | Gaps |
|---|---|---|---|
| 1–10 | *Describe 10 games you already run (1v1, 2v2, 3v3, an overload, a transition game, a defense-first game, a character-scoring game).* | — | — |
