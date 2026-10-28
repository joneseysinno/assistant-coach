# Assistant Coach — Design Docs

An app on the JoInn `app-shell` template that helps a coach **design small-sided games with a constraints-led approach**, **turn plays into the reads that games train**, and **build practice plans and seasons** from those games, then **run practice live**: score and clocks on the TV, phones as remotes, and drills kept on schedule. Basketball first; laptop, tablet and Android phone.

> Theory and plan only — no code yet. P0 (spec + paper validation) is in progress; see `05-p0-plan.md` for status. Architecture lives in these documents, not in chat. When something changes, update the doc or add a decision record (`04` §2).

| Doc | What it answers |
|---|---|
| [01-theory.md](01-theory.md) | Why games and constraints; ecological dynamics, Newell, affordances, attractors/metastability, nonlinear pedagogy, SSG evidence, PoST, how plays fit, measurement, character as constraint |
| [02-domain-model.md](02-domain-model.md) | The closed primitive set, compositions, learning targets, vocabulary table, rules R1–R15, play model, worked examples, forbidden list |
| [03-app-design.md](03-app-design.md) | How it maps onto the JoInn shell: ShellSpec, doc types (Game, Play, Practice, Season, Review), panels, run mode, app modules, template gaps, more ambitious ideas |
| [04-roadmap.md](04-roadmap.md) | Phases P0–P7 with exit criteria, decision record D-001…D-016, open questions, risks |
| [05-p0-plan.md](05-p0-plan.md) | P0 plan and status: decisions, specs S1–S14, fixtures, stress set, paper prototype, gym network test, P1/P2 handoff |
| [decisions.md](decisions.md) | The decision record: D-001…D-036 with status and date |
| [06-live-practice.md](06-live-practice.md) | Live practice: laptop hub, TV Board, phone/tablet Remote, clocks, scoring from rules, keeping drills on schedule |
| [spec/](spec/) | S1 Events · S2 Rule grammar · S3 Space & court · S4 Format · S5 Constraint/Dial/Variant + fixture format · S6 Targets & reads · S7 Play graph · S8 Practice & season · S9 Estimates · S10 Observation · S11 Live protocol · S12 Remote & Board · S13 Schedule guard · S14 InfiniteDB mapping |
| [fixtures/](fixtures/) | The test corpus (YAML): games (worked example + 12-game stress set), plays (format examples), practice template, a live-log replay |
| [validation/](validation/) | Gap log, gym network test, paper prototype, live dry-run, practice-01 sheets |
| [plans/](plans/) | P1 tasks (foundations + live), P2 tasks (designer), PR review checklist |
| [printables/](printables/) | Mock TV Board for the gym test; paper prototype (constraint deck, game cards, Remote button sheets, tally sheet) |
| `../tools/` | `p0_check.py` (fixture checker / oracle for P1 golden tests), `make_printables.py` |

**The core loop**

```
   PLAY ──decompose──► READS ──generate──► GAMES ──sequence──► PRACTICE ──observe──► DATA
    ▲                                        │                                       │
    └──────── harvest emergent patterns ◄────┘◄──────── tune dials (challenge point) ┘
```
