# P2 — Game Designer (pulled into P1 by D-038)

> **D-038 (2026-10-28): Studio first.** These designer tasks were pulled forward into P1 and built at v1 depth: D01–D05 and D08–D13 are in the Studio (forms, stack + rollback, checks, predicted effects, card + YAML export, Library). **Still open:** D06 (drawing zones on the canvas), D07 (variants + progression ladder UI), YAML *import*, and D14 (AJ encodes the 10 real games in the app, which is now the main P1 test). The live phase (old P1 T12–T27) is now P2.

> One task = one Cursor session/PR. Builds on P1's domain, resolver, renderer and store. Exit criterion: **the 10 real fixture games are built in the app (not typed as YAML), round-trip through InfiniteDB, and render readable cards; new games export to YAML and run live.**

| # | Task | Specs / decisions | Depends | Acceptance |
|---|---|---|---|---|
| D01 | **Court SVG component** (`court` module): lines from the gym preset, zones layer, role chips, start arrows, hit-testing via S3 `contains` | S3 | P1 T03 | Snapshot SVGs for hs/ms half and full; click on (−22, 1) selects `corner-L` |
| D02 | **Game doc type shell**: toolbar groups (Format · Space · Rules · Dials · Card), left rail (Constraint stack, Targets), right rail (Properties, Predicted effects, Questions & cues), bottom rail (Progressions, Card preview) | 03 §3 | D01 | Opens a fixture game; panels remember state per doc type |
| D03 | **Constraint stack panel**: list, suppress toggle, drag reorder, **rollback bar** | S5 §5; 03 §3 | D02 | Rollback to N matches `resolve` with stack truncated (S5-T4) |
| D04 | **Format editor** (forms): numbers/roles, start/restart, rotation, end, shot clock, balls, phases (basic) | S4 | D02 | Build Closeout Attack 3v3's format by hand → equal to the fixture's format |
| D05 | **Rule editor** (forms, not text): trigger form picker, filters, after/within, conditions, consequences, judge, overrides; live plain-English preview | S2; 04 Q7 | D02 | Build every rule in the fixtures by form; rendered sentence matches `expect.card` |
| D06 | **Zones on the canvas**: draw rect/circle/band/polygon, pick named regions, mirror L/R, dial-driven sizes update live | S3 §6 | D01, D03 | `corner-zones` constraint built by clicking |
| D07 | **Dials panel + variants**: sliders with bounds/levels; progression ladder; variant = delta (R6); parent-change warnings | S5 §3–6 | D03 | S5-T2, S5-T3 through the UI |
| D08 | **Linter** ("Is this a drill?"): R1–R3, R7, R12–R14 messages in coaching language | 02 §5; 03 §9.9 | D05 | Each rule has a fixture that triggers it |
| D09 | **Predicted effects panel**: area/player, intensity band, involvement, active ratio, each with formula + confidence | S9 | D04 | S9-T1…T4 shown with formula text |
| D10 | **Targets + reads tagging**: target tree picker (S6 ids), reads on the game, cue linter (R14) | S6 | D02 | S6-T2 warning appears |
| D11 | **Game card export**: printable page (HTML → print/PDF) matching the paper-prototype game card | 03 §3; S2 §7 | D05 | Card for Closeout Attack matches `printables/` wording |
| D12 | **Library pane**: all games/constraints/variants, filter by target, numbers, stage; drag into Practice doc | 03 §2, §8.3 | D02 | Drag a game into a practice block |
| D13 | **YAML export/import** of a game with its constraints (D-018) | S5 | D05 | Export → `p0_check.py` passes → import → equal |
| D14 | **Encode the 10 real games in the UI** (field task) | E3 | D01–D13 | All 10 saved, round-trip, cards printed |
