# S1 — Events

> Status: draft for review · Primitive: **P3 Event** · Decisions: D-004, D-013, D-030, D-035 · Needed by: rules (S2), live log (S11), Remote buttons (S12), observation (S10)

## 1. Purpose

Fix the basketball event vocabulary, the fields every event carries, and — most important for the gym — **which events a coach can actually record live**. Rules (S2) can only talk about events in this list.

## 2. Two layers: what happened vs. what was recorded

The coach does not enter raw events. A tap records a **rule firing**, and the rule's pattern supplies the events it implies (D-035).

```
 Remote tap "Kick-out 3 +3 (Red)"
        │
        ▼
 log entry: rule.fired { rule: kick-out-three, team: red, possession: 14 }
        │  implied (from the rule's when/after pattern)
        ▼
 events:  paint-touch {team: red, poss: 14}  →  make {value: 3, team: red, poss: 14}
```

So the **event stream** used by observation (S10) is: implied events from rule firings + direct tallies + hub-generated events (clocks, possessions). Every event says which of those it came from (`source`).

## 3. Fields every event carries

| Field | Type | Required | Notes |
|---|---|---|---|
| `kind` | event id (§4) | yes | Pack vocabulary |
| `t_ms` | u64 | yes | Hub time, ms since the **block** started (S11 clock math) |
| `block` | id | yes | Which block |
| `basket` | basket id | yes | Two games can run at once (D-020) |
| `possession` | u32 | when `balls = 1` | Hub-assigned counter, increments on every possession change (§6) |
| `team` | team id | usually | The team the event is *credited to* (the scoring team for `make`, the defending team for `stop`) |
| `actor` | role id or player id | optional | Rarely known live; present for player modifiers and video review |
| `zone` | zone id | optional | From the rule's filter, or a tap that names a zone |
| `attrs` | map | optional | Event-specific (§4) — e.g. `value: 3`, `hand: L` |
| `source` | `tap` · `implied` · `hub` · `review` | yes | Provenance |
| `entry` | log seq | yes | The log entry that produced it (undo removes all events of an entry) |

## 4. Vocabulary (basketball pack)

**Live** = a coach can record it with one tap mid-play. **Implied** = arrives through a rule firing. **Hub** = the hub produces it. **Review** = only from video/after practice. Most events can arrive several ways; the column says how they *usually* arrive.

### Possession and flow
| Event | Attrs | Usually | Notes |
|---|---|---|---|
| `possession.start` | `start: check-ball \| coach-pass \| closeout \| rebound \| transition \| inbound` | hub | From the format's start/restart (S4) |
| `possession.end` | `reason: score \| stop \| turnover \| violation \| time \| foul` | hub | Derived from the consequence that ended it |
| `phase.change` | `to` | hub | Multi-phase formats (D-026) |

### Ball actions
| Event | Attrs | Usually | Notes |
|---|---|---|---|
| `pass` | `kind: chest \| bounce \| skip \| lob \| handoff` | implied / review | Counting passes live is not realistic beyond 1–2 per possession |
| `catch` | `zone` | implied | "Catch in corner after a drive" |
| `dribble` | — | review | Never tallied live. **Dribble limits are judged as violations** |
| `drive` | `side: baseline \| middle` | implied / live | |
| `paint-touch` | `by: dribble \| pass \| post` | live / implied | Tallyable if it's the only pinpoint |
| `screen` | `kind: ball \| off-ball \| re-screen` | implied / review | |
| `cut` | `kind: back \| basket \| flare` | implied / review | |
| `closeout` | `outcome: drive \| shoot \| pass` | live | The closeout game's pinpoint |
| `extra-pass` | — | implied | Humble |
| `assist` | — | live (judged) | |

### Shots and rebounds
| Event | Attrs | Usually | Notes |
|---|---|---|---|
| `shot` | `value: 2 \| 3 \| ft`, `zone`, `hand: L \| R` | implied | Attempts, for shooting %: only when a rule or pinpoint asks |
| `make` | `value`, `zone`, `hand`, `after_catch_ms` | **live** | The basic scoring tap |
| `miss` | `value`, `zone` | review / live | Only if a pinpoint wants attempts |
| `off-reb` | — | **live** | Hungry |
| `def-reb` | — | implied | Usually just "stop" |
| `loose-ball` | — | live | Hungry |

### Defense
| Event | Attrs | Usually | Notes |
|---|---|---|---|
| `stop` | — | **live** | Defense ends a possession without a score |
| `deflection` | — | **live** | |
| `steal` | — | live | Also ends the possession |
| `block` | — | live | |
| `charge` | — | **live** | |
| `contest` | — | review | |

### Rules, roles, space
| Event | Attrs | Usually | Notes |
|---|---|---|---|
| `turnover` | `kind: steal \| out \| travel \| other` | **live** | |
| `violation` | `rule` | **live** | A limit rule fired (dribble limit, zone, shot clock) |
| `foul` | `on: offense \| defense` | live | |
| `role.enter` / `role.leave` | `role` | hub / implied | Delayed defender (stress #1) |
| `zone.enter` / `zone.exit` | `zone` | implied (judge: players) | D-030 |
| `dial.set` | `dial, value` | live | Includes live dials (D-027) |
| `constraint.on` / `constraint.off` | `constraint` | live | Pool choices (D-028) |

### Clock and character
| Event | Attrs | Usually | Notes |
|---|---|---|---|
| `clock.expired` | `clock: block \| interval \| shot \| game` | hub | |
| `clock.warning` | `clock` | hub | Schedule guard |
| `talk` | — | live (assistant / player-observer) | Attitude/Effort |
| `sprint-back` | — | live (judged) | Effort |
| `capture` | `note` | live | Harvest loop: "players found something" |

## 5. What is realistically tallyable live

Working assumption (to be checked by the tally-sheet test): one coach **scoring** a game can hold the scoring taps plus **one** extra pinpoint; a second person can hold one or two more. The Remote therefore:
- always shows the scoring rules (they *are* the game);
- shows at most **2 pinpoint buttons** per basket beyond scoring (S12);
- never asks for `dribble`, `pass` counts, `contest` or `shot` attempts live.

**Validated in P0 by the tally-sheet test (W4.3)** — update this section with what actually worked.

## 6. Possession counter

`possession` increments when any consequence changes possession (`turnover`, `possession: switch`, a `stop`, a make with `restart: switch`). A coach tap of **Stop ⇄** with no rule also increments it. For `balls > 1` the counter is absent and possession-scoped patterns are invalid (D-029).

## 7. Invariants

1. Every event kind used in a rule is in the pack vocabulary (validator error otherwise).
2. Events are never edited. Undo removes a *log entry*, and with it all events whose `entry` is that seq.
3. `t_ms` is hub time; clients never stamp events.
4. An implied event is derivable from (rule, entry) alone — re-deriving after a rule edit is allowed for review but never rewrites the log.

## 8. Worked examples (P1 tests)

| Test | Log entry | Expected events |
|---|---|---|
| S1-T1 | `rule.fired kick-out-three red poss 14` | `paint-touch{red,14,implied}`, `make{value:3,red,14,implied}` |
| S1-T2 | `rule.fired basket blue poss 15` (rule: `when make value 2`) | `make{value:2,blue,15}` |
| S1-T3 | `rule.fired def-stop blue poss 16` (defense scores +1) | `stop{blue,16}`, `possession.end{reason: stop}`, `possession.start{start: rebound}` — possession becomes 17 |
| S1-T4 | `undo 42` where entry 42 was S1-T1 | both implied events gone; red score −3 |
| S1-T5 | `tally closeout outcome=drive` | `closeout{outcome: drive, source: tap}` |

## 9. Open questions

1. Is `paint-touch` tallyable for you live along with scoring, or only when it is the scoring rule? (Paper test W4.3.)
2. Do you want `miss` taps at all (for shot quality pinpoints), or is "makes per minute" enough?
3. Player-observer: a benched player tallying `talk` on a tablet — wanted? It fits the Attitude constraint and the roles model (a *viewer+tally* role).
