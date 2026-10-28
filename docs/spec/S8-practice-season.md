# S8 — Practice & Season

> Status: draft for review · Compositions: **Block, Session (practice), Season** · Rules R10–R12 · Decisions: D-015, D-020, D-022, D-031 · Needed by: live hub (the practice list, P1 live-first), planner (later)

## 1. Purpose

Define the practice plan the hub runs, how minutes are accounted for, and the active-ratio arithmetic. The live-first phase only needs §2–§4; §5–§6 are for the planner and season views.

## 2. Practice and Block

```yaml
practice:
  id: pr-2026-11-03
  date: 2026-11-03
  gym: gym-home                 # D-020
  start: "15:30"
  planned_minutes: 90
  policy: { overtime: firm, grace_s: 60, warn_s: 60 }   # D-022 defaults
  roster_present: [p-01, p-02, …]                       # attendance
  transition_s: 60              # default changeover between blocks
  blocks:
    - id: b1
      name: Warm-up 2v2
      game: s08-cross-court-2v2   # a game or variant id
      baskets: [S1, S2]
      minutes: 8
      intervals: null            # or { work_s: 240, rest_s: 60, reps: 3 }
      groups: { mode: random, team_size: 2 }    # or explicit teams below
      teams: null                # [{ id: red, color: "#d32f2f", players: [..] }, …]
      protected: false
      min_minutes: 4             # floor for the schedule guard (S13); default max(3, 50% of minutes)
      modifiers: []              # D-031
      prompt: "Where is the help coming from?"   # Board prompt line
    - id: b9
      name: Free-throw ladder
      game: g-ft-ladder
      baskets: [A, B]
      minutes: 6
      protected: true
```

## 3. Time accounting (R10)

```
planned_total = Σ block.minutes + (n_blocks − 1) × transition_s/60
unassigned    = practice.planned_minutes − planned_total          (show red if < 0)
block_active_minutes = intervals ? (work_s × reps) / 60 : minutes
```
Intervals: `work_s × reps + rest_s × (reps − 1)` must equal `minutes × 60` (± 30 s); the validator warns otherwise.

**Worked:** 9 blocks totalling 82 min, transition 60 s → 82 + 8 = 90 → unassigned 0.

## 4. Active ratio (R11)

Per block and per player:
```
active_ratio(block) = on_court_players / players_at_those_baskets      (per basket, then averaged by players)
player_active_min   = Σ over blocks of block_active_minutes × player_share(block, player)
```
`player_share` by rotation (S4 §5):

| Rotation | share for a player at that basket |
|---|---|
| none | 1 if on a playing team, 0 if sitting out |
| winners-stay / losers-stay / king / round-robin | `2 / n_teams` (estimate; refined from the log after practice) |
| three-team | `2 / 3` |
| continuous | (players in the phases' roles) / (players in the line) per phase |
| stations | 1 if the station's game has them on court |

Flags: any player whose active minutes < `target_active_ratio` × practice active minutes (default 0.6) is shown in amber.

**Worked (P0 6.4 will replace this with your real practice):** 12 players, 3v3 one basket, `none`, 10 min → ratio 0.5, 5 active min per player. Same block on two baskets → 1.0, 10 min.

## 5. Groups

| Mode | Behaviour |
|---|---|
| `explicit` | Teams listed |
| `random` | Shuffle present players into `team_size` teams (seeded per block so it is reproducible) |
| `balanced` | Snake draft by a player `level` (1–3) |
| `ability` | Teams of like level (for challenge-point work) |
| `keep` | Same teams as the previous block |

## 6. Season (planner, later)

- `season: { id, name, weeks: [ { index, phase, stage_mix: { team-based: 0.6, … }, targets: [..] } ] }`.
- Grid unit = **minutes of active play per learning target per week**, computed from blocks' tags × active minutes.
- Phases: pre-season / early / conference / tournament (renameable). The 14-week plan is imported as a Season (P6).

## 7. Invariants
1. Every block's baskets exist in the practice's gym.
2. A protected block's `min_minutes` = `minutes`.
3. Two blocks may share a time slot only on different baskets.
4. `game` resolves (game or variant).

## 8. Open questions
1. Do you plan transitions explicitly (water, setup) or as a flat 60 s between blocks?
2. Do players have a `level` you'd want to use for balanced/ability groups?
