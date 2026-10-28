# S4 — Format (the base game)

> Status: draft for review · Composition: **Base format** = Space + Roles + start/restart + possession/rotation + end · Decisions: D-021, D-026, D-029, D-032 · Needed by: Game (S5), practice time (S8), live fold (S11)

## 1. Purpose

Describe "how the game runs" before any constraint is added: who plays, how a possession starts and restarts, how teams rotate, and when it ends. Everything here is data the hub can run and the card can explain.

## 2. Fields

```yaml
format:
  numbers: "3v3"                 # display; derived from roles, validated
  space: { court: half, basket: primary }      # S3 region; extra zones belong to constraints
  roles:
    - { id: O1, team: offense }
    - { id: D1, team: defense }
    - { id: J,  team: neutral, plays_for: offense }     # D-032
  teams: 2                       # teams in the rotation (≥ 2); see §5
  balls: 1                       # D-029
  start:   { type: closeout, passer: defense, closeout: live }
  restart: { on_score: switch, on_stop: switch, on_violation: switch }
  rotation: { type: winners-stay, rotate_on: score }
  end:     { type: score, target: 7, win_by: 1 }
  shot_clock: null               # or { seconds: "$shot", resets_on: [possession.change] }  (D-021)
  phases: null                   # D-026, see §7
  rim_height_ft: 10
```

`offense` / `defense` are **per-possession states**, not team names. Teams are Red/Blue (/Green…) and their colors come from the Block (pinnies). A role's `team: offense` means "on the team that has the ball when the possession starts."

## 3. Start types

| Type | Fields | Meaning | Card text |
|---|---|---|---|
| `check-ball` | `spot` | Defender checks to offense at the spot | "Check ball at the top" |
| `coach-pass` | `from: zone`, `to: role` | Coach feeds the ball | "Coach passes to a wing" |
| `closeout` | `passer: defense`, `closeout: live`, `distance: "$d"` | Defender passes out and closes out | "Defense passes out and closes out" |
| `rebound` | `shooter: coach \| offense` | Live missed shot | "Coach shoots; play the rebound" |
| `transition` | `from: basket-B`, `trigger: outlet` | Play starts going the other way | "Outlet and go" |
| `inbound` | `spot: baseline \| sideline` | Dead-ball inbound | "Baseline out of bounds" |
| `dribble-at` | `from: zone` | Offense attacks from a spot | "Attack from half court" |

`start` is the first possession; `restart` says what happens after each kind of possession end.

## 4. Restarts

| Key | Values | Meaning |
|---|---|---|
| `on_score` | `switch` · `keep` (make-it-take-it) · `rotate` · `restart: <start type>` | |
| `on_stop` | `switch` (live, defense goes on offense) · `check` · `rotate` | A defensive rebound or turnover |
| `on_violation` | same | Limit rules |

A rule's `restart:` consequence overrides these for one possession (S2 §5).

## 5. Rotation formats

`n_teams` = teams in the rotation at this basket; `n_court` = teams on the floor (2).

| Type | Behaviour | `rotate_on` | Active ratio (S9) |
|---|---|---|---|
| `none` | Same two teams the whole time | — | `n_court / n_teams` |
| `winners-stay` | Winner of a mini-game stays; loser off; next team on | `game-end` / `score` | ≈ `2 / n_teams` early, rises for the strong team |
| `losers-stay` | Loser stays (more reps for whoever is struggling) | `game-end` | ≈ `2 / n_teams` |
| `make-it-take-it` | Possession rule, not a rotation (`on_score: keep`) | — | — |
| `three-team` | O → D → off → O on each possession (offense comes off after defending) | `possession` | `2/3` |
| `round-robin` | Scheduled matchups for `n_teams`; mini-game per matchup | `game-end` / `time` | `2 / n_teams` |
| `king` | King of the court: ladder of `n_teams`; winner stays as king; challengers queue | `game-end` | `2 / n_teams`; ladder state kept |
| `stations` | Groups rotate between baskets on a timer (Block level) | `time` | per station |
| `continuous` | Possessions chain; roles switch team on possession end (uses phases) | `possession` | depends on phases |

**Ladder state** (king, winners-stay) is *derived from the log* — the hub folds game-end entries into a queue; nothing is stored separately.

## 6. End conditions

| Type | Fields | Notes |
|---|---|---|
| `score` | `target`, `win_by` | Mini-games inside a block; triggers rotation |
| `time` | `seconds` | Mini-game by time |
| `possessions` | `count` | "Each team gets 5 possessions" |
| `block` | — | Play until the block clock ends (default) |

The **block clock always ends the block** regardless of `end` (S13 decides what happens at zero).

## 7. Phases (D-026)

For formats whose structure changes during play (stress #2):

```yaml
phases:
  - id: three-on-two
    roles: [O1, O2, O3, D1, D2]
    attack: A
  - id: two-on-one
    roles: [O4, O5, D3]              # the two defenders become O4/O5; one of the shooters stays back as D3
    attack: B
phase_flow:
  - { from: three-on-two, on: possession.end, to: two-on-one, map: { D1: O4, D2: O5, shooter: D3 }, others: off }
  - { from: two-on-one,  on: possession.end, to: three-on-two, map: { "*": off }, next_group: true }
```

`map` names roles; `shooter` / `rebounder` are **actor aliases** the coach handles on the floor (the app doesn't track who shot). The hub only needs the phase (for the Board and the basket direction) and the team that scored.

## 8. Invariants

1. Every role id is unique; at least one offense role; R1: at least one defense role or `stage: coordination` with a note.
2. `numbers` matches the roles (`3v3` = 3 offense, 3 defense; neutrals shown as `+1`).
3. `rotation` with `n_teams > 2` needs `rotate_on`.
4. `phases` ⇒ every role in a `map` exists in the target phase.
5. `shot_clock.seconds` is a Duration dial or a literal 3–35 s.

## 9. Worked examples

| Test | Format | Check |
|---|---|---|
| S4-T1 | 3v3, 12 players at the basket, `none` | active ratio 6/12 = 0.5 (`01` §7) |
| S4-T2 | 3v3, 12 players, `three-team` + 4th team waiting | 6/12 → planner suggests second basket |
| S4-T3 | `king`, 4 teams R B G Y, R starts as king, queue [B, G, Y], losers to the **back** (default, see §10.1) | R beats B → queue [G, Y, B]; R beats G → [Y, B, G]; Y beats R → king Y, queue [B, G, R] |
| S4-T4 | `on_score: keep` | after red scores, possession stays red; possession counter still increments (new possession) |
| S4-T5 | 3v2 → 2v1 phases | after possession end in phase 1, phase = two-on-one, attack = B |

## 10. Open questions

1. King of the court: when the king loses, does the old king go to the **back** of the queue or the **front**? (Gyms do both.) S4-T3 assumes the back; change the test if you play it the other way.
2. Do you use `losers-stay` at all, or only `winners-stay`?
3. "Make-it-take-it" with a 7th-grade group — do you cap it (max 2 in a row)? That is a counting rule; the grammar supports it.
