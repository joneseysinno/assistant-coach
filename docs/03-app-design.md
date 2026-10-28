# 03 — App Design on the JoInn Template

> Status: draft for review · Scope: how the domain model (`02`) maps onto the JoInn `app-shell` (`ShellSpec` → `DocType` → rails, panels, viewport, command bus). No code — structure and behavior only.

---

## 1. Why the JoInn shell fits this app

The shell was built around an Onshape-style workflow, and that workflow lines up with constraint-led design surprisingly well:

| Onshape idea | Assistant Coach equivalent |
|---|---|
| Sketch | **Base game** (space + roles + start/restart) |
| Feature tree | **Constraint stack** — ordered, each one can be suppressed, edited or reordered |
| Rollback bar | Drag to see the game *before* constraint N — "what does the game look like without the dribble limit?" |
| Configurations | **Variants / progressions** along dials |
| Part Studio → Assembly | Game → **Practice plan** (games placed in time and space) |
| Variables table | **Dials** panel |
| Command palette | "New 3v3 game", "Add constraint: dribble limit", "Open Tuesday practice" |

Other shell features this app uses directly:

- **Keep-alive viewports** — switching between a play, a game and tonight's practice keeps each one's state.
- **Panel memory per document type** — the Game designer remembers its constraint stack open; the Practice planner remembers its metrics panel.
- **One command bus + history** — every edit is a command; undo and "macro record a progression" are short steps from there.
- **Renderer-agnostic** — desktop for planning, web build for the courtside Remote and the TV Board (see `06-live-practice.md`).

---

## 2. `ShellSpec` for Assistant Coach

**Title bar:** app name, context label = team + season ("7th Girls · 2026–27"), actions: Run Practice (primary), Help, Account.

**Nav rail (global, far left)**

| Item | Pane contents |
|---|---|
| Documents | Shell's built-in documents pane (games, plays, practices, season) |
| Library | Every game, constraint, play and read, filterable by learning target, numbers (1v1…5v5), PoST stage, space, duration. Drag source for practices. |
| Roster | Players, groups/levels, individual constraints, attendance for today |
| Targets | The learning-target taxonomy tree with coverage counts (how many games/blocks train each) |
| Settings (end) | Court dimensions (HS 84×50, MS 74×42, custom), theme, units, sport pack |

**Doc types:** Game, Play, Practice, Season (+ Review in a later phase). Details below.

---

## 3. Doc type: **Game** (the heart of the app — build first)

```
┌ Toolbar: [Format: 1v1…5v5 · overload · court] [Space: zone tools] [Rules: + scoring · + limit · + restart] [Dials] [Card ⤓]
├────────────┬──────────────────────────────────────────┬──────────────┤
│ CONSTRAINT │                                          │ PROPERTIES   │
│ STACK      │        COURT CANVAS (SVG)                │ of selection │
│ ▣ Base 3v3 │   zones painted · roles placed · start   │ ─────────── │
│ ☑ Kick-out │   arrows · bonus areas glow              │ PREDICTED    │
│ ☑ 2-drib   │                                          │ EFFECTS      │
│ ☐ Corners  │                                          │ ─────────── │
│ ☑ Hungry   │                                          │ QUESTIONS &  │
│ ── rollback│                                          │ CUES         │
│ TARGETS    │                                          │              │
├────────────┴──────────────────────────────────────────┴──────────────┤
│ Bottom rail: [Progressions] [Card preview]   status: 3v3 · half · 4 constraints · team-based · ~350 ft²/pl
└───────────────────────────────────────────────────────────────────────┘
```

**Left rail**
- *Constraint stack* — feature-tree behavior: check to suppress, drag to reorder, rollback bar. Each row shows category icon (task / environment / individual / character).
- *Targets* — learning targets this game is tagged with; shows reads the game invites.

**Viewport — court canvas**
- SVG court from the sport pack, dimensions from Settings.
- Layers: space & zones → roles (O/D/neutral chips) → start/restart arrows → annotations.
- Click a zone/role/rule-region → its properties appear on the right.
- View toggle (bottom rail, like Iso/Top/Front in the demo): **Diagram | Card | Both**.

**Right rail**
- *Properties* — the selected constraint/zone/role: its Rules (rendered as plain-English sentences, editable as structured fields), its Dials (sliders with bounds), *invites* and *preserves* lists (R2, R3).
- *Predicted effects* — transparent estimates (R15):
  - players on court, relative area per player, intensity band,
  - estimated involvements/decisions per player per minute,
  - representativeness checklist: live defense? real basket? time pressure? help present?
  - "drill warning" badge if R1 fails.
- *Questions & cues* — coach prompts tied to the game's reads; external-focus checker (R14).

**Bottom rail**
- *Progressions* — a ladder of variants (easier ⇄ harder), each a delta on the base (R6). Click a rung to load it; the stack and dials update.
- *Card preview* — the printable/shareable game card.

**Game card (export)** — one page: name, diagram, "How to score," "What's not allowed," start/restart, rotation, the 1–2 questions the coach will ask. Written for *players*, generated from the Rules (R5). Useful for posting in the gym or handing to an assistant coach.

---

## 4. Doc type: **Play**

```
┌ Toolbar: [Align] [Actions: pass · dribble · screen · cut · handoff] [Read: + branch] [Playback ▶] [Decompose ⇢ reads]
├────────────┬──────────────────────────────────────────┬──────────────┤
│ STATES &   │  COURT  |  GRAPH  |  BOTH                │ SELECTED     │
│ BRANCHES   │                                          │ READ         │
│ State0     │  court view: positions for current state │ cue · options│
│  ├ drop →  │  graph view: nodes = states,             │ TRAINED BY   │
│  ├ hedge → │              edges = actions (+read guard)│ ▸ 2v2 Elbow  │
│  └ switch →│                                          │ [Generate    │
│            │                                          │  game ⇢]     │
├────────────┴──────────────────────────────────────────┴──────────────┤
│ Bottom rail: [Timeline scrubber]   status: 5 states · 4 reads · 1 untrained
```

- **Graph-first** (R8). Court view shows the current state; graph view shows the whole decision structure; *Both* is a split.
- **Playback** walks one chosen path to make an animation (derived, not stored).
- **Decompose** lists all reads; each shows its *trained by* games or an "untrained" flag (R9).
- **Generate game from read** creates a new Game document pre-built around that read (`02` §6) and links it back.
- **Harvest (bottom-up loop):** a Play can also be *created from a game* — in run mode the coach taps "capture" and quickly drags chips to record a pattern the players found. It lands in the Library as a draft play.

**Shell note:** the play designer has modes (aligning vs. drawing actions vs. adding reads). The current `DocType.toolbar` is static. Options: (a) put modes in the toolbar as toggle tools and keep mode in app state — works today; (b) implement the README's "context-sensitive toolbar" idea (`fn(&Shell) -> Vec<ToolGroup>`) in the shell. Start with (a); revisit if it gets clumsy.

---

## 5. Doc type: **Practice**

```
┌ Toolbar: [+ Block from library] [Split groups] [Rotation: winners-stay · round-robin · stations] [Transition time] [Run ▶]
├────────────┬──────────────────────────────────────────┬──────────────┤
│ SESSION    │  TIMELINE                                │ SESSION      │
│ OUTLINE    │  rows = baskets/courts, columns = time    │ METRICS      │
│ Theme      │  ┌─Warm-up 2v2 ─┐┌─Closeout 3v3──┐       │ active time/ │
│ Blocks     │  │ basket A      ││ A + B         │       │ player       │
│ ...        │  └──────────────┘└───────────────┘       │ est. load    │
│            │                                          │ target mix   │
│            │                                          │ interleaving │
├────────────┴──────────────────────────────────────────┴──────────────┤
│ Bottom rail: [Run mode] [Notes]   status: 90 min · 7 blocks · 0 min unassigned · 78% avg active
```

- **Blocks** are game instances placed on a timeline across available baskets. Drag from the Library pane. Each block carries duration, work:rest, rotation format and groups.
- **Groups** come from the Roster (balanced teams, ability groups, or random). Individual constraints can be applied per player for a block (`02` §10.4).
- **Metrics panel**
  - *Time accounting* — total, unassigned, transitions (R10).
  - *Active time per player* — bar per player; flags anyone below a threshold (R11).
  - *Estimated load* — session-RPE-style units from block intensity bands × minutes, so the coach can watch weekly load (fits the AWU/ACWR thinking already used for athlete programs).
  - *Target mix* — which learning targets the session hits; interleaving warning (R12).
  - *PoST stage mix* — e.g., early season heavier on simplified/variability; game week heavier on performance.
- **Run mode** is now its own system; see `06-live-practice.md` (laptop hub, TV Board, phone Remote). In Studio, the Practice doc just has a **Run ▶** button that hands the session to the hub. Originally sketched as: big clock, current block card, next block, live score from the game's Rules, tally buttons for the block's pinpoints, "capture pattern" button (harvest loop), and a one-tap "turn dial up/down" that records the change.

---

## 6. Doc type: **Season**

- **Viewport:** grid of weeks (columns) × learning targets (rows), cells colored by minutes spent; overlay of PoST stage per week.
- **Left:** phases/blocks of the season (e.g., pre-season, early, conference, tournament). Can import the structure of the existing 14-week season plan.
- **Right:** coverage analysis — targets never trained, targets massed in one week (spacing), program-word coverage (Humble/Hungry/Effort/Attitude).
- **Bottom:** calendar with practices and games.

---

## 7. Later doc type: **Review** (observation and charts)

- Pinpoint charts per player and per team from run-mode tallies — count per minute on a Standard Celeration Chart; 7/8 criterion markers.
- **Challenge-point suggestions:** "Closeout reads are 7/8 for 3 sessions → suggest moving closeout start to the elbow."
- **Metastability tuner:** for decision games, plots option split (drive vs. shoot %) against the dial setting and suggests the dial value nearest 50/50.
- **Estimate calibration:** compares predicted involvements/intensity with what was tallied, and adjusts the estimate coefficients (R15).

---

## 8. App-side architecture (inside the generated app, not the shell)

The shell owns layout. The app owns everything else. Proposed modules, following the shell's flat-module style:

| Module | Responsibility |
|---|---|
| `domain` | The types from `02` (Space, Role, Event, Rule, Read, Dial, Constraint, Game, Variant, Play, Block, Session, Season, Player, Observation). Pure data, serde. No Dioxus. |
| `sport_basketball` | The sport pack: court geometry by level, event vocabulary, learning-target taxonomy, starter constraint library. |
| `rules` | Rule evaluation (live scoring in run mode, plain-English rendering for cards). |
| `estimate` | The transparent heuristic models (area/player, intensity band, involvement density, active ratio, load). Every function returns value + formula text + confidence. |
| `store` (interface in `ac-domain`) + `ac-store-idb` (adapter) | Persistence on **InfiniteDB from day one** (D-006). The domain defines save/load/query/append-log in domain terms; the adapter maps them to nodes, hyperedges and spatial regions (`02` §11, spec S14). UI code never calls InfiniteDB directly (D-017). YAML fixtures are for tests and import/export only (D-018). |
| `court` | Shared SVG court component: court lines, zones, role chips, arrows, hit-testing. Used by Game, Play and Run mode. |
| `docs/game.rs`, `docs/play.rs`, `docs/practice.rs`, `docs/season.rs` | One `DocType` each, using the template's pattern: per-document state in a `GlobalSignal<HashMap<doc_id, State>>`, commands handled in `on_command`. |
| `panes` | Library, Roster, Targets nav panes. |

**Template gaps to plan for**
1. **Persistence** — the shell has none; `Document` is only id/name/kind. The app's `store` loads into the per-doc state map on open and saves to InfiniteDB on command (and autosave). Only the hub/Studio process opens the database (D-019).
2. **Cross-document references** — practices reference games; plays reference games through reads. These are edges and hyperedges in InfiniteDB; the Library pane is a graph query. The shell doesn't need to know.
3. **Drag from nav pane into a viewport** — Library → Practice timeline. Needs a small app-level drag state (the shell already has its own for splitters; don't overload it).
4. **Mode-dependent toolbars** — Play designer (section 4 note).
5. **Courtside layout** — resolved by `06-live-practice.md`: Board and Remote are separate touch/display layouts, not the shell.

---

## 9. Ideas beyond the obvious (to pick from, not all for v1)

1. **Problem-first entry ("What's going wrong?").** The coach types or picks a problem — "we stand when the ball is in the post," "we only drive right." The app maps it to learning targets and proposes *constraint recipes* that destabilize that attractor. Turns the library into a diagnostic tool.
2. **Constraint mixer.** A sound-board view of all dials for the current game, with the predicted-effects panel updating live — "push the dribble limit down, watch decisions/min go up and intensity go down."
3. **Metastability tuner** (section 7) — tune decision games toward the 50/50 point where reading is required.
4. **Plays harvested from games** — the bottom-up loop; the playbook becomes a record of what *this* team discovered.
5. **Scout packs** — save an opponent's habits as constraints on the defense; drop them into game-week practices (PoST performance stage).
6. **Individual constraints inside team games** — "Player A scores only with her left hand this block," "Player B can't post up, must face up." The same 3v3 becomes a different problem for each player.
7. **Players vote on a rule.** The losing team picks the next constraint from three cards. Builds ownership and Attitude, and players start thinking like designers.
8. **Character scoring library** — ready-made constraints that reward Humble/Hungry/Effort/Attitude behaviors (`01` §12).
9. **"Is this a drill?" linter** — runs R1–R3 and R12–R14 over a game or session and explains each warning in coaching language.
10. **joinn authoring (later).** Rules and plays are already graphs whose edges carry meaning and messages. Once joinn is ready, it could become the advanced editor for both — the simple form UI stays for most coaches.
11. **AI assist through MCP (later).** "Generate three 2v2 games that train help-side rotation for 12 players on one basket" — the AI writes domain objects, the coach edits them in the same UI.
