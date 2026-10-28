# S7 — Play Graph (light)

> Status: draft for review · Composition: **Play** · Rules R8, R9 · Decisions: D-002 · Needed by: P5 (full UX); written now only to prove the primitives cover plays

## 1. Purpose

Show that a play — linear set, read-based set, or looping concept — is just States, Transitions, Roles and Reads, so nothing about plays forces a new primitive later.

## 2. Structures

```yaml
play:
  id: pl-horns-elbow
  name: Horns — Elbow Ball Screen
  kind: read-set                 # linear-set | read-set | concept
  court: half
  roles: [O1, O2, O3, O4, O5]    # defenders optional: D1..D5
  states:
    - id: s0
      label: Horns
      at: { O1: [0, 28], O4: [-6, 14], O5: [6, 14], O2: [-21, 1], O3: [21, 1] }   # basket frame, ft (S3)
      ball: O1
  transitions:
    - id: t1
      from: s0
      to: s1
      actions: [ { screen: { by: O4, for: O1 } }, { dribble: { by: O1, to: [-8, 18] } } ]
      guard: null                # unguarded = always
      timing: "when O4 is set"
```

| Element | Fields | Notes |
|---|---|---|
| **State** | `id`, `label`, `at` (role → point), `ball` (role), optional `markers` (screens, cut arrows) | A court snapshot |
| **Transition** | `from`, `to`, `actions[]`, `guard` (Read id or null), `timing` (cue text), `participants` (derived: every role in actions + guard's `cue.source` role) | Stored as one hyperedge (S14 §4) |
| **Action** | `pass{from,to}`, `dribble{by,to}`, `screen{by,for}`, `cut{by,to}`, `handoff{from,to}`, `shot{by}` | Pack vocabulary = S1 ball events |
| **Loop** | A transition whose `to` is an earlier state | Concepts are graphs with cycles |

## 3. Derived things

- **Decompose(play) → reads:** the set of all non-null `guard`s.
- **Animation:** pick a path (sequence of transitions; at a branch, choose a guard); interpolate positions linearly over each transition (default 1.2 s per action). Never stored (R8).
- **Generate(read) → game:** roles = the transition's participants + the read's `cue.source`; space = bounding box of their positions + 6 ft, clipped to `half`; defense live only on `cue.source` and the participants' defenders; scoring rule: `make` after the correct option = 2, other makes = 1 (02 §6).

## 4. Invariants

1. Every state has a position for every play role; points inside the court region.
2. At most one unguarded transition leaves a state (otherwise the play is ambiguous — make them guarded).
3. A `concept` must contain at least one loop; a `linear-set` none.
4. R9: every guard links to ≥ 1 game in `trained_by` or is listed as untrained.

## 5. Worked examples

`docs/fixtures/plays/` holds two **format examples** (from `02` §8 and a generic 5-out concept). They are *not* your team's sets — W3.3 still needs 2–3 of those.

| Test | Input | Expected |
|---|---|---|
| S7-T1 | decompose `pl-horns-elbow` | 4 reads: drop, hedge, switch, help-on-roll |
| S7-T2 | R9 on `pl-horns-elbow` with `trained_by` for drop/hedge/switch only | 1 untrained (help-on-roll) |
| S7-T3 | `pl-5out-drive-kick` | kind concept, contains a loop |
| S7-T4 | path s0→t1→t-drop in horns | 2 transitions, animation length 3 actions × 1.2 s = 3.6 s |
