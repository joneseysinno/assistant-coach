# 02 — Domain Model, Vocabulary and Rules

> Status: draft for review · Scope: *what the app's data is*, independent of UI and code.
> This is the document that stops drift: a **closed set of primitives**, a **vocabulary table**, **numbered rules**, and a **forbidden list**. New concepts need a decision record before they're added.

---

## 1. Design stance

- **Small primitives, composed.** A few kinds of things, combined into bigger things. No special-case "drill types."
- **Data before UI.** Every screen edits one of these structures. If a screen needs a concept that isn't here, this document changes first.
- **Sport-agnostic core, basketball "sport pack."** Court geometry, event names and the learning-target taxonomy are *data* in a basketball pack. Another sport later is a new pack, not a rewrite.

---

## 2. Primitives (closed set of 6)

| # | Primitive | What it is | Basketball example |
|---|---|---|---|
| P1 | **Space** | A court region plus zones painted on it. Zones have a *kind* (bonus, restricted, no-dribble, safe, start, target) and a shape. | Half court; corner "bonus" zones; no-dribble zone above the break |
| P2 | **Role** | A participant slot with a team (offense, defense, neutral) and optional per-role limits. | O1–O3, D1–D3, a neutral "joker" who plays offense for whoever has the ball, a coach-feeder |
| P3 | **Event** | An observable thing that happens in play, from the sport pack's vocabulary. | pass, catch, dribble, paint-touch, shot(zone, type), make, miss, off-reb, def-reb, turnover, deflection, charge, stop, closeout, screen, cut, time-expired |
| P4 | **Rule** | *When* (event pattern + state condition) → *Then* (consequence). Scoring, limits, restarts and possession changes are all Rules. | When `make` in zone `corner` → +3 · When `dribbles > 2` → turnover · When `stop` → defense gets possession *and* +1 |
| P5 | **Read** | A perception → action pair: the **cue** a player perceives and the **options** it affords. | Cue: "closeout is high and fast" → options: drive baseline / drive middle |
| P6 | **Dial** | A named, bounded parameter whose value changes a Space, Role or Rule. Difficulty lives in dials. | Closeout start distance: rim … elbow · Dribble limit: 0 … ∞ · Shot clock: 6 … 24 s |

Everything else is a **composition**:

| Composition | Built from | Notes |
|---|---|---|
| **Constraint** | Rules + Space edits + Role edits + Dials, with metadata | The unit of coaching intent. Has a *category* (task / environment / individual), *invites* (behaviors it should bring out), *preserves* (information it keeps), and *targets* (learning targets). |
| **Game** | Base format + ordered **Constraint stack** | Base format = Space + Roles + start/restart + possession/rotation + end condition. |
| **Variant** | Game + dial values + constraints suppressed/added | A progression step. *Never a copy* — a delta on its parent game. |
| **Play** | Alignment + **decision graph** of States and Transitions; transitions may be guarded by Reads | See section 6. |
| **Block** | A Game (or Variant) instance in time: duration, work:rest, basket/court, groups | One row in a practice plan. |
| **Session** | Ordered Blocks + theme + attendance | A practice plan. |
| **Season** | Weeks/phases × learning targets + sessions | Curriculum and coverage. |
| **Player** | Name, group/level, individual constraints (attributes, current pinpoints) | Individual constraints are *inputs* to games (handicaps, role assignment). |
| **Observation** | Timestamped Event tallies against a Block/Player | Fuel for charts, challenge-point suggestions, and calibration of estimates. |

### Why Event + Rule are the heart

Scoring is the most powerful constraint, and every useful scoring idea is "when *this* happens, *that* follows." Making scoring **structured** (Event → Rule) instead of free text makes possible:

- automatic **game cards** in plain English ("Corner threes are worth 3; everything else is worth 1"),
- **live scoring** in run mode,
- **observation pinpoints** that come straight from the same event list,
- later, authoring Rules as a **graph** (edges carry messages — a natural fit for joinn).

Free-text notes are still allowed, *alongside* the structure.

---

## 3. Learning targets

A **Learning Target** is a node in the sport pack's taxonomy (`01-theory.md` §10): game moment → principle → optional specific read. Games, constraints, reads, plays, blocks and sessions are all *tagged* with targets. Coverage, interleaving checks and season heat maps are computed from these tags.

```
offense
  create-advantage
    beat-1v1
    ball-screen
    off-ball-screen
    cut
  extend-advantage
    drive-and-kick
    attack-closeout        ← Read: "closeout high/fast → drive"
    extra-pass
  finish
  shape (spacing, timing, relocation)
defense
  deny-advantage
  scramble (help, rotate, closeout, x-out, talk)
  end-possession (contest, box-out, secure)
transition (offense / defense)
character (humble, hungry, effort, attitude)
```

---

## 4. Vocabulary table (coach word ↔ theory word ↔ app type)

| Coach says | Theory says | App type |
|---|---|---|
| drill | task (often non-representative) | Game with PoST stage = coordination, *flagged* if no live opposition |
| scrimmage | full representative game | Game, base 5v5 full court, no constraints |
| rule, condition, "you can only…" | task constraint | Constraint |
| points, scoring | task constraint (reward structure) | Rule(s) inside a Constraint |
| court, area, zones | environmental / task constraint | Space |
| "make it harder/easier" | challenge point | Dial |
| progression, level 2 | variant along a dial | Variant |
| read, look, "what did you see" | affordance / information–movement coupling | Read |
| set, play | structured constraint on spacing/timing | Play (decision graph) |
| cue | external-focus or questioning prompt | Cue (text on a Read or Constraint) |
| practice plan | session design | Session |
| groups, teams | role assignment / individual constraints | Block groups + Player |
| stats, charting | observation / pinpoints | Observation |

---

## 5. Rules (numbered, stable — refer to them by number)

**Game design**

- **R1 — Live opposition by default.** Every game has at least one active defender (or opponent behavior source) unless its PoST stage is *coordination*, and then it must state *why* in its notes. The app shows a "non-representative" badge on games without live opposition.
- **R2 — Every constraint states what it invites.** A constraint has at least one *invites* entry (behavior expected to emerge) and one *target* tag. "Because I like it" is not a constraint.
- **R3 — Every constraint states what it preserves.** Name the information sources kept (defender's position, help, time pressure). This is the representativeness check.
- **R4 — Difficulty lives in dials.** No hard-coded "easy" and "hard" versions; a variant is dial values plus constraint toggles.
- **R5 — Scoring is structured.** Every point source is a Rule over Events. Notes may explain, not replace.
- **R6 — Variants are deltas.** A progression references its parent game; editing the parent updates every variant that didn't override that part.
- **R7 — Every game declares a PoST stage** (coordination / variability / complex / team-based / performance).

**Plays**

- **R8 — A play is a graph, not a movie.** Animation is *derived* from a path through the graph.
- **R9 — No orphan reads.** Each Read in a play links to ≥ 1 game that trains it, or is flagged "untrained." The playbook view shows the untrained count.

**Practice**

- **R10 — Every minute is accounted for.** Session time = Σ block time + transitions. The planner shows the gap.
- **R11 — Per-player active time is visible.** Active ratio is computed per block and per player from groups and roles.
- **R12 — Interleave by default.** The planner warns when one learning target fills more than a set share of consecutive blocks, unless the session is tagged *performance* stage.

**Measurement and coaching language**

- **R13 — Measure decisions and outcomes, not form.** Pinpoints are Events or Read outcomes; there is no "technique score" field.
- **R14 — Cues are external or questions.** The cue editor gently flags body-part language ("elbow," "wrist," "knees") and suggests an external rewrite. A soft rule: it warns, it doesn't block.
- **R15 — Estimates are labeled as estimates.** Intensity/involvement predictions show their formula and confidence, and are calibrated against Observations over time.

---

## 6. The Play model in detail

```
Play
 ├─ alignment: State₀  (positions of O1–O5, D1–D5 optional, ball)
 ├─ states: [State]    (court snapshot: positions + ball + optional screen/cut markers)
 └─ transitions: [Transition]
       from: State, to: State
       actions: [pass | dribble | screen | cut | handoff | shot]   (who, to where)
       guard: Option<Read>     ← "only if this is what you see"
       timing: Option<cue>     ← "when the ball crosses the free-throw line extended"
```

- A **linear set** is a chain of unguarded transitions.
- A **read-based set** branches: from one state, several guarded transitions leave, one per Read.
- An **action/concept** (e.g., "Horns", "Chicago", "5-out drive-and-kick rules") is a play whose graph *loops* — it's a rule system, not a sequence.

**Decompose(play) → reads** walks the graph and collects every guard. **Generate(read) → game** builds a base game around the read's state: the roles involved, a Space cropped to the action, live defense on exactly the players whose choices create the cue, and a scoring rule that rewards the correct option on each branch. The coach then edits it like any other game.

---

## 7. Worked example A — a game

**"Closeout Attack 3v3"**

- **Base:** half court · O1–O3 vs D1–D3 · start: D passes to O on the perimeter and closes out (live) · make-it-take-it off, defense rebound = switch · first to 7.
- **Constraint stack**
  1. *Kick-out bonus* (task) — Rule: `make three after paint-touch in same possession → +3, else three → +1`. Invites: drive-and-kick, extra pass. Preserves: help defense, closeout. Targets: extend-advantage/drive-and-kick.
  2. *Two-dribble limit* (task) — Rule: `dribbles > 2 by one player → turnover`. Dial: limit 1–4 (default 2). Invites: quick decisions, catch-and-attack.
  3. *Corner zones* (environment) — Space: two corner bonus zones; Rule: `catch in corner zone after a drive → +1`. Invites: relocation / spacing.
  4. *Hungry* (character) — Rule: `off-reb → +1`.
- **Reads invited:** closeout high & fast → drive; closeout short → shoot; help steps up → kick to the help defender's player.
- **Dials:** closeout start distance (rim → block → elbow), dribble limit, game target score.
- **Observables:** closeout decisions correct / total; paint touches per possession; extra passes.
- **Metastability tuning:** if > 80% of closeouts produce a catch-and-shoot, start defenders closer; if > 80% are drives, start them farther.
- **Estimated readout (HS court):** 6 on court, ~350 ft²/player, high intensity; with 12 players at one basket → 50% active ratio → planner suggests "winners stay, 3 teams rotate" or a second basket.

## 8. Worked example B — a play decomposed

**"Horns – Elbow Ball Screen"** (simplified)

```
State0 (Horns alignment) ──O1 uses O4 screen──► State1
State1 ──guard: "D4 drops"   ──► O1 pull-up / pocket pass to rolling O4
State1 ──guard: "D4 hedges"  ──► O1 hits O4 popping / O5 dives
State1 ──guard: "switch"     ──► O1 attacks big / O4 seals small
State1 ──guard: "D5 helps on roll" ──► lift to O5's side → kick
```

**Decompose →** 4 reads. **Generate →** e.g., "2v2 Elbow Screen + Weak-side 1v1": O1/O4 vs D1/D4 from horns, defense *chooses* coverage each possession (coach shows a card to the defense so offense must read it), weak-side O5 vs D5 who may help. Scoring: correct counter to the coverage that scores = +2; any other score = +1. The play is then *trained by* this game (R9 satisfied).

---

## 9. Forbidden list

| Don't | Because | Instead |
|---|---|---|
| A "drill" type separate from Game | Splits the model and invites decontextualized practice | Game with stage = coordination and R1 justification |
| Plays stored only as animations/frames | Hides the reads; can't decompose | Decision graph; animation is derived |
| Free-text-only scoring | Can't generate cards, score live, or measure | Rules over Events + optional notes |
| Copy-paste progressions | Drift between versions | Variants as deltas (R6) |
| A "technique score" | Pulls coaching back to form | Decision/outcome pinpoints (R13) |
| Hidden estimate formulas | Coaches can't trust or calibrate them | Show formula + confidence (R15) |
| Basketball names in core types | Blocks other sports later | Sport pack data |

---

## 10. Open modeling questions

> **P0 answers (proposed):** 1 → D-033 (possession-scoped sequences + `within` + count conditions); 2 → D-032 (Role with `team: neutral` + `plays_for`); 3 → still open (fits as constraints on defense roles; confirm before P7); 4 → D-031 (Block-level player modifiers). Stress-set evidence: `validation/gap-log.md`.

1. **Rule expressiveness.** Is "When event-pattern [+ condition] → consequence" enough, or do we need sequences ("three *after* a paint touch *in the same possession*")? Current answer: allow a small *possession-scoped sequence* pattern; nothing more general until a real game needs it.
2. **Neutral players.** Model as Role with team = neutral and a rule "joins team in possession," or as a special case? Proposal: Role + Rule (keeps primitives closed).
3. **Opponent scouting as constraints.** A "scout pack" = a saved constraint set describing an opponent's habits (they hedge every ball screen; they zone after timeouts). Fits as Constraints on the *defense* roles. Confirm before modeling.
4. **Individual constraints inside team games** (e.g., Player A can only finish left in this block). Proposal: per-Block player modifiers that compile to Role-level Rules.

---

## 11. Mapping onto InfiniteDB (draft — detailed in spec S14)

Per D-006, the model is stored as a graph in InfiniteDB from day one. Domain code stays in domain terms (D-017); this section only shows *why the model fits a hypergraph* and how it will be laid out.

| Domain | InfiniteDB shape | Note |
|---|---|---|
| Game, Constraint, Rule, Dial, Read, Play, State, Block, Session, Season, Player | **Nodes** with intrinsic, opaque ids | Identity is not position or order (fits the InfiniteDB identity decision) |
| Game ─has→ Constraint (ordered) | Typed edge with an order value | The constraint stack's order is data on the edge, not array position |
| Variant ─delta of→ Game | Typed edge carrying the delta (dial overrides, toggles) | R6: variants are deltas |
| Constraint ─invites/targets→ Learning targets | **Hyperedge**: one constraint to many targets | Many-to-many tagging is native |
| Read ─trained by→ Games | Hyperedge | R9 "no orphan reads" becomes a graph query |
| **Play transition** | **Hyperedge joining from-State, to-State, every participating Role (screener, user, both defenders) and the guarding Read** | A ball screen involves four players and two states. A plain edge can't carry that; a hyperedge can. This is the main reason plays are graphs |
| Court, zones | A spatial region subdivided into named zones (paint, corners, wings…) | The court is a subdividable region; positions live inside it |
| Player positions in a State | Points in court space | Spatial queries: "every play that ends with a catch in the right corner," "every state with O5 in the dunker spot" |
| Animation | **Derived:** a traversal along a path of States, interpolating positions over each transition's timing | Never stored (D-002) |
| Live event log | Append-only, time-ordered Event nodes linked to Block, Game, Role and Player | Undo, replay/crash recovery, observation charts (D-013) |

**Graph queries the app will want** (these drive the S14 spec):
- All reads in a play (decompose); all games that train a read; reads with none (R9).
- All games/blocks tagged with a learning target, by date (season coverage, interleaving R12).
- A play's path → ordered states → animation frames.
- Spatial: plays, states or logged events by court zone.
- Live: replay a session's log to reconstruct the score and clocks at any time.
