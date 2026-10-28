# S2 — Rule Grammar (When → Then)

> Status: draft for review · Primitive: **P4 Rule** · Decisions: D-004, D-014, D-025, D-028, D-033, D-034, D-035 · Needed by: game resolution (S5), cards, Remote buttons (S12), live fold (S11)

## 1. Purpose

One small, structured language for everything a game "says": scoring, limits, restarts, role changes, character rewards and pinpoints. It must be (a) renderable as plain English for a 7th-grade player, (b) turnable into one-tap buttons, (c) foldable by the hub into a score, and (d) **not** a programming language.

## 2. Shape of a rule

```yaml
- id: kick-out-three            # unique in the resolved game
  label: Kick-out 3             # button text (≤ 16 chars); defaults from the renderer
  when:  { event: make, value: 3 }
  after: [ { event: paint-touch } ]       # optional, possession-scoped, ≤ 2 steps
  within: null                  # optional seconds from the last `after` step
  if:    []                     # optional conditions (§4)
  then:  [ { points: 3 } ]      # consequences (§5)
  overrides: [three]            # rules this one suppresses for the same event (D-034)
  judge: coach                  # coach | players | auto  (D-025)
  side: either                  # offense | defense | either — whose button it is
  say: null                     # optional hand-written sentence; else rendered (§7)
```

Rules live inside constraints (S5). The base format contributes **base rules** (e.g. `two`, `three`, restarts) with ids prefixed `base.`.

## 3. Triggers (`when`) — the closed list (D-033)

| Form | YAML | Example |
|---|---|---|
| **Event** with filters | `{ event: K, value?, zone?, actor?, team?, hand?, attrs? }` | `{ event: make, zone: corner-L }` |
| **Sequence** | `when` + `after: [steps]` (+ `within: s`) | three *after* paint-touch; make *within 6 s* of catch |
| **Count** | `{ count: K, by: player \| team, op: ">" \| ">=" \| "==", value: N }` within the possession | `{ count: dribble, by: player, op: ">", value: "$limit" }` |
| **Clock** | `{ clock: block \| interval \| shot \| game, reaches: s }` | `{ clock: shot, reaches: 0 }` |
| **State** | `{ score: team \| any, reaches: N }` · `{ game: end }` · `{ possession: end, reason? }` | first to 7 |

Filters:
- `actor: { role: J }` · `{ team: neutral }` · `{ player: P }` (block modifiers only, D-031)
- `zone:` a zone id or named region; `zone_not:` the complement
- `team:` `offense` / `defense` / a team id — default: the team the event is credited to

**Sequence semantics:** steps must occur in order inside the **same possession**; `within` bounds the time between the last `after` step and `when`. With `judge: coach` the coach asserts the whole pattern with one tap; with `auto` the hub checks it against logged events.

**Count semantics:** counts reset at each possession start. Counts of events nobody logs live (`dribble`) are only usable with `judge: coach` or `players` — they become *violation* buttons, not counters.

## 4. Conditions (`if`)

All must hold. Conditions read **game state**, never the future.

| Condition | YAML | Example use |
|---|---|---|
| Score | `{ score: losing \| winning \| tied }` / `{ margin: { op, value } }` | catch-up rules |
| Clock | `{ clock: block, left: { op: "<", value: 60 } }` | "last minute: threes are 4" |
| Dial / live dial | `{ dial: coverage, is: hedge }` | coach card game (stress #3) |
| Constraint state | `{ constraint: X, on: true }` | |
| Possession index | `{ possession_in_game: { op: ">", value: 5 } }` | rarely |
| Phase | `{ phase: two-on-one }` | D-026 |
| Count so far | `{ count: pass, by: team \| player, op, value }` (this possession) | "Make one pass before you shoot" (stress #8) — counts as a *condition*, so no negation is needed |

## 5. Consequences (`then`) — applied in order

| Consequence | YAML | Notes |
|---|---|---|
| Points | `{ points: N }` · `{ points: N, to: defense \| offense \| <team> }` | N may be negative or `"$dial"`. Default `to`: the credited team |
| No basket | `{ points: 0, cancel: base }` | Cancels the base points of the same tap (handicap rules) |
| Turnover | `{ turnover: true }` | Possession changes; implies `possession.end{turnover}` |
| Possession | `{ possession: switch \| keep \| coach }` | `keep` = make-it-take-it |
| Restart | `{ restart: check-ball \| coach-pass \| closeout \| rebound \| inbound, ball?: offense \| defense \| challenger \| loser }` | Overrides the format's default restart for this possession; `ball` says who gets it |
| Role | `{ role: { enter: D3, at: start-zone } }` · `{ leave: O1 }` · `{ switch_team: [D1, D2] }` | Delayed defender, continuous games |
| Constraint | `{ constraint: { on: X } }` · `{ off: X }` · `{ choose: { by: losing-team, from: pool } }` | D-028 |
| Dial | `{ dial: { set: limit, value: 1 } }` · `{ step: limit, by: -1 }` | In-game progression |
| Clock | `{ clock: { reset: shot } }` · `{ add: block, seconds: 30 }` | |
| End | `{ end: game }` · `{ end: block }` | |
| Rotate | `{ rotate: winners-stay \| next-team }` | Triggers the format's rotation (S4) |
| Phase | `{ phase: next }` | D-026 |
| Tally | `{ tally: true }` | No game effect. **This is how a pinpoint is a rule** |

## 6. Precedence (one triggering tap or event)

1. Collect every **on** rule whose trigger matches.
2. Remove any rule listed in another matching rule's `overrides` (D-034). Overrides are not transitive unless written.
3. Apply the remaining rules **in stack order** (base rules first, then constraints top to bottom).
4. Points add. At most **one** possession change per event: the first `turnover`/`possession` consequence wins; later ones are ignored (validator warns when two can collide).
5. `end` and score-target checks run last, after all points are applied.

**Coach taps:** a tap fires exactly the tapped rule *plus* any `auto` rules triggered by the resulting state (e.g. score reaches 7 → `end: game`). It never also fires the base rule the tapped rule overrides.

## 7. Plain-English rendering

Renderer = templates per trigger form + per consequence, joined. Players read these on cards and on the Board, so they are short and second-person-free.

| Pattern | Template | Example |
|---|---|---|
| event + points | "{Event phrase} = {N}" | "Corner three = 3" |
| sequence + points | "{Event} after {step} = {N}" | "Three after a paint touch = 3" |
| within | "… within {s} seconds of {step}" | "Score within 6 seconds of the catch = 2" |
| count + turnover | "More than {N} {event}s = turnover" | "More than 2 dribbles = turnover" |
| defense points | "{Event} = +{N} for the defense" | "Stop = +1 for the defense" |
| role enter | "{Role} joins after {event}" | "The third defender joins after the first pass" |
| zone exit limit | "{Role} must stay in the {zone}" | "The help defender must stay in the paint" |
| choose | "{Team} picks the next rule" | "The losing team picks the next rule" |
| tally | (not rendered on cards) | — |

Rules: dial values are rendered as their **current value** ("2 dribbles max"), never `$limit`; plural/singular handled; `say:` overrides the template. Golden tests store the expected sentence for every fixture rule.

**Board short form:** the TV has room for about three short phrases, so the Board uses the game's **first three constraint rules in stack order** (base rules and `tally` rules excluded — the coach's stack order *is* the priority). Short templates: points rule → "{label} = {N}"; count limit → "{N} {event}s max"; anything else → its `label`. A rule may set `board:` to override. Example: "Kick-out 3 = 3 · 2 dribbles max · Corner catch = 1".

## 8. Invariants (validator)

1. Every `event:` is in the pack vocabulary; every `zone:` resolves.
2. Every `$dial` resolves in scope and has the right type (S5).
3. `after` length ≤ 2; `within` only with `after`.
4. Possession-scoped triggers are invalid when `format.balls > 1` (D-029).
5. A rule with `points` must have `judge: coach` or `auto` (or the game can't keep score); `players` rules may not award points.
6. `overrides` targets exist in the same resolved game.
7. Two rules that both change possession on the same trigger → warning.

## 9. Worked examples (fixture rules → expected rendering)

| Test | Rule (YAML) | Rendered |
|---|---|---|
| S2-T1 | `when: {event: make, value: 2}`, `then: [{points: 1}]`, id `base.two` | "Basket = 1" |
| S2-T2 | `when: {event: make, value: 3}`, `after: [{event: paint-touch}]`, `then: [{points: 3}]`, `overrides: [base.three]` | "Three after a paint touch = 3" |
| S2-T3 | `when: {count: dribble, by: player, op: ">", value: "$limit"}`, `then: [{turnover: true}]`, `judge: coach`, dial limit = 2 | "More than 2 dribbles = turnover" |
| S2-T4 | `when: {event: stop}`, `then: [{points: 1, to: defense}]` | "Stop = +1 for the defense" |
| S2-T5 | `when: {event: make}`, `after: [{event: catch}]`, `within: 6`, `then: [{points: 2}]`, `overrides: [base.two]` | "Score within 6 seconds of the catch = 2" |
| S2-T6 | `when: {count: pass, by: team, op: "==", value: 1}`, `then: [{role: {enter: D3}}]`, `judge: players` | "The third defender joins after the first pass" |
| S2-T7 | `when: {clock: shot, reaches: 0}`, `then: [{turnover: true}]`, `judge: auto` | "Shot clock: 8 seconds" (shown as a clock, not a sentence) |

**Fold tests (precedence):**
| Test | Active rules | Tap | Expected |
|---|---|---|---|
| S2-F1 | base.three (1), kick-out-three (3, overrides base.three) | kick-out-three | +3, not +4 |
| S2-F2 | base.two (1), hungry `off-reb → +1` | off-reb then base.two | +2 total |
| S2-F3 | first-to-7, red at 6 | base.two (red) | red 7, `end: game`, rotation fires |
| S2-F4 | def-stop (+1 defense, possession switch), charge (+2 defense, turnover) | charge | defense +2, one possession change |

## 10. Open questions

1. Do you ever score **negative** points for the offense (e.g. "turnover = −1")? Allowed by the grammar; confirm it's a real use.
2. "Assist" is judged. Should the Remote show it as a modifier on a basket tap ("Basket + assist") rather than its own button? That's a layout question for S12 but changes how many taps a possession takes.
