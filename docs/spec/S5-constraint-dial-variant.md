# S5 — Constraint, Dial, Variant (composition) + the fixture format

> Status: draft for review · Primitives: **P6 Dial** + compositions Constraint, Game, Variant · Rules R2–R7 · Decisions: D-018, D-027, D-028, D-031 · Needed by: everything that loads a game

## 1. Purpose

Say exactly how a game is assembled from a base format and a stack of constraints, how difficulty dials plug into rules and zones, and what a variant (progression step) stores. This spec also defines the **YAML fixture format** that P1 loads (D-018).

## 2. Constraint

```yaml
- id: dribble-limit
  name: Dribble limit
  category: task                 # task | environment | individual | character
  invites: [quick-decisions, catch-and-attack]          # R2: ≥ 1
  preserves: [defender-pressure, help-defense]          # R3: ≥ 1
  targets: [off.extend.attack-closeout]                 # R2: ≥ 1 (S6 ids)
  stage: null                     # optional override; the game declares its stage (R7)
  dials:
    limit: { int: [0, 4], default: 2, harder: down, levels: [3, 2, 1] }
  zones: []                       # Space edits (S3 §6)
  roles: []                       # Role edits: add a role, or restrict one (§2.1)
  rules:
    - id: over-limit
      when: { count: dribble, by: player, op: ">", value: "$limit" }
      then: [ { turnover: true } ]
      judge: coach
  cues: ["Where is the help before you catch?"]          # R14: external / question
  notes: ""
```

### 2.1 Role edits
| Edit | YAML | Compiles to |
|---|---|---|
| Add role | `{ add: { id: D3, team: defense, start: { zone: baseline-out } } }` | a role in the resolved format |
| Restrict to zone | `{ restrict: D3, stay_in: paint, penalty: 1 }` | rule `zone.exit(D3, paint) → points 1 to offense`, `judge: coach` (it awards points, so the scorer taps it — S2 §8.5; found by the P0 checker, gap G-12b) |
| Can't shoot | `{ restrict: J, cannot: shoot }` | rule `shot by J → turnover`, `judge: players` |
| Can't dribble | `{ restrict: O5, cannot: dribble }` | rule `dribble by O5 → turnover`, `judge: players` |

Role edits are **sugar**: the resolver turns them into ordinary zones/roles/rules, so the primitive set stays closed (D-003).

## 3. Dial

| Type | YAML | Value | Example |
|---|---|---|---|
| `int` | `{ int: [min, max], step: 1 }` | integer | dribble limit |
| `distance` | `{ distance: [min_ft, max_ft], stops: { rim: 0, block: 6, elbow: 14 } }` | feet | closeout start |
| `duration` | `{ duration: [min_s, max_s] }` | seconds | shot clock |
| `enum` | `{ enum: [drop, hedge, switch] }` | one option | coverage |
| `bool` | `{ bool: true }` | true/false | "assists count" |

Common fields: `default` (required), `harder: up | down` (which way is harder — used by challenge-point suggestions), `levels` (2–4 values; the paper deck prints these as **Level 1/2/3**), `live` + `visible_to` (D-027), `label`.

## 4. How a dial is referenced

- Anywhere a **number, duration or enum** appears in a rule, zone or format field, the string `"$name"` may stand in for it.
- **Scope:** `$name` resolves first in the enclosing constraint's dials, then the game's dials (`game.dials`). Constraint dials are namespaced in the resolved game as `constraint-id.name` (shown on the Dials panel), so two constraints can both have `limit`.
- **Type check:** an `int` dial may fill an int slot; `distance` fills geometry and distance slots; `duration` fills seconds; `enum` only fills `is:` conditions and enum fields. Mismatch = validator error.
- A dial may also be referenced by a *game* field: `end.target: "$target"`.

## 5. Game

```yaml
id: g-closeout-attack-3v3
name: Closeout Attack 3v3
stage: team-based               # R7
format: { … S4 … }
base_rules: default-basketball  # pack preset: base.two = 1, base.three = 2 (or a list)
dials: { target: { int: [3, 11], default: 7 } }
constraints: [ …ordered stack… ]
pool: []                        # D-028 inactive constraints that can be chosen in play
reads: [read.closeout.high-fast-drive, read.closeout.short-shoot]
observables: [ { pinpoint: closeout-correct, over: closeout } ]
notes: ""
```

**Resolution** `resolve(game, overrides) → ResolvedGame`:
1. Start from the format; add base rules.
2. Walk the constraint stack top to bottom, skipping suppressed ones; add zones, roles, rules; expand role edits.
3. Bind dial values: defaults ← game overrides ← variant overrides ← live `dial.set` entries (latest wins).
4. Substitute `$refs`; validate (S2 §8, S3 §8, S4 §8).
5. Output a flat, fully-bound game: roles, zones, rules (with stack order), clocks. The hub, the renderer and the button generator only ever see a `ResolvedGame`.

The *rollback bar* in Studio is `resolve` with the stack truncated at N.

## 6. Variant (progression step) — a delta, never a copy (R6)

```yaml
id: v-closeout-attack-L2
parent: g-closeout-attack-3v3    # a game or another variant
name: Closeout Attack — Level 2
rung: 2                          # position on the progression ladder
dials: { dribble-limit.limit: 1, closeout-start.dist: 12 }
toggles: { corner-zones: off }
add: [ { ref: c-shot-clock-8 } ] # constraints appended to the stack (inline or by ref)
format: { end: { target: 5 } }   # optional format field overrides (shallow, per key)
```

**Inheritance when the parent changes:**
| Parent change | Effect on the variant |
|---|---|
| Changes a dial default the variant doesn't override | Variant follows |
| Changes a dial's bounds so the override is out of range | Override clamped; warning on the variant |
| Removes a constraint the variant toggles | Toggle becomes dangling; warning; ignored |
| Adds a constraint | Variant gets it (on) |
| Reorders the stack | Variant follows; `add` constraints stay at the end |

Variants may chain (variant of a variant). Resolution folds deltas root → leaf.

## 7. Block-level modifiers (D-031)

```yaml
block:
  game: v-closeout-attack-L2
  modifiers:
    - { player: p-avery, rule: { when: { event: make, actor: { player: p-avery }, hand: R }, then: [ { points: 0, cancel: base } ], judge: players, say: "Avery: left-hand finishes only" } }
```

The resolver appends modifiers after the stack, for this block only.

## 8. Invariants

1. R2/R3: every constraint has ≥ 1 `invites`, ≥ 1 `preserves`, ≥ 1 `targets`.
2. R4: no constraint has hard-coded easy/hard copies — two constraints that differ only in a number are a linter warning ("make this a dial").
3. R6: a variant's file never contains a full stack.
4. R7: every game has `stage`.
5. Every dial default is inside its bounds; `levels` are inside bounds and ordered easy → hard.
6. Resolution is pure and deterministic (same input → byte-identical output).

## 9. Worked examples (P1 tests)

| Test | Input | Expected |
|---|---|---|
| S5-T1 | `g-closeout-attack-3v3` resolved with defaults | rules: base.two, base.three, kick-out-three, over-limit (limit 2), corner-catch, hungry; card line "More than 2 dribbles = turnover" |
| S5-T2 | `v-closeout-attack-L2` | limit 1; corner-catch absent; end target 5 |
| S5-T3 | Change parent `dribble-limit.limit` bounds to [2, 4] | L2's override 1 → clamped 2, one warning |
| S5-T4 | Rollback to constraint 1 | rules: base.two, base.three, kick-out-three |
| S5-T5 | `$limit` used as a zone radius | validator error (int in a distance slot) |
| S5-T6 | Round-trip | fixture YAML → domain → YAML → domain: equal (D-018) |

## 10. Open questions

1. Should `levels` be required on every dial (so every constraint prints as a Level 1/2/3 card), or only when you want it on paper?
2. Base scoring for 7th grade: is your default 2s = 1 and 3s = 2 (as assumed above), or 2 and 3?
