# S9 — Estimates (predicted effects)

> Status: draft for review · Rules R15, D-007 · Needed by: predicted-effects panel (designer), practice metrics (planner), load view (later)

## 1. Purpose

Every number the app predicts is a **heuristic with its formula, its inputs and a confidence shown next to it** (R15). This spec fixes the formulas so they can be tested and later calibrated against logged observations.

Every estimator returns `Estimate { value, unit, formula_text, inputs, confidence: low | medium | high, basis: [refs] }`.

## 2. Area per player

```
area_per_player = area(court_region − restricted zones that no one may enter) / on_court_players
```
- Inputs: S3 region area, roles on court (neutrals count).
- Confidence: **high** (geometry).
- Bands (HS-age SSG rule of thumb from `01` §7): < 250 ft² tight · 250–450 typical · > 450 open.
- Example: 3v3 half HS → 350 ft²; 5v5 full HS → 420 ft²; 2v2 cross-court HS → 525 ft².

## 3. Intensity band

Ordinal band 1–5 (light … very high), not a heart-rate number.
```
band = clamp(round( B(n_per_side) + A(area_per_player) + W(work_s) + C(constraints) ), 1, 5)
B: 1v1 → 4.5 · 2v2 → 4 · 3v3 → 3.5 · 4v4 → 3 · 5v5 → 2.5
A: < 250 ft² → −0.25 · 250–450 → 0 · > 450 → +0.5          (bigger courts → more running, higher RPE — Klusemann 2012; Marcelino 2015)
W: continuous work > 4 min → +0.5 · work bouts ≤ 2.5 min with ≥ 60 s rest → −0.5   (Klusemann 2012)
C: shot clock ≤ 8 s → +0.25 · full-court phases → +0.5 · stage coordination with no live defense → −1
```
- Confidence: **low** until calibrated; **medium** after ≥ 5 logged blocks with session-RPE for the game.
- Basis: `01` §7 (Castagna 2011, Klusemann 2012, Marcelino 2015). Coefficients are **starting guesses**, stated as such on screen.

## 4. Involvement density

Estimated decisions/touches per player per minute.
```
involvement = K / on_court_players × Π modifiers
K = 12 (starting guess: possessions × touches per possession per minute across the floor)
modifiers: dribble limit ≤ 2 → ×1.2 · ≤ 1 → ×1.35 · paint-before-three → ×1.1 · neutral player → ×1.1 · shot clock ≤ 8 → ×1.15
```
- Confidence: **low**; calibrated from tallied events per minute (S10) — the calibration replaces `K` per game family.

## 5. Active ratio

Exactly S8 §4 (arithmetic, confidence **high** for `none`/`three-team`, **medium** for winner-stays-type rotations until the log refines it).

## 6. Load units

Session-RPE-style units so a week's load can be watched (fits the AWU/ACWR approach already used for athlete programs):
```
block_load(player) = player_active_minutes × rpe_estimate(band)
rpe_estimate: band 1 → 3 · 2 → 4 · 3 → 5 · 4 → 6.5 · 5 → 8
session_load(player) = Σ block_load
```
- After practice, a coach-entered session RPE (one number per player or per team) replaces the estimate in the record and feeds calibration.
- Weekly view (later): sum per week; ACWR = last-7-days / average of last-28-days, guardrail 0.8–1.3 shown, never enforced.

## 7. Calibration loop (R15)

For each estimator, store `(predicted, observed)` pairs per game family; refit the single multiplicative coefficient by least squares in log space after ≥ 5 pairs; show "calibrated from N blocks". Never silently change a formula's shape — a shape change is a new spec revision.

## 8. Tests

| Test | Input | Expected |
|---|---|---|
| S9-T1 | 3v3 half HS | area 350, band round(3.5 + 0) = 4 (3.5 rounds half-up), confidence low |
| S9-T2 | 5v5 full HS, 8 min continuous | area 420 → A 0; W +0.5; band round(2.5+0.5) = 3 |
| S9-T3 | 2v2 cross-court HS | area 525 → +0.5; band round(4 + 0.5) = 5 (4.5 → 5) |
| S9-T4 | involvement 3v3 + dribble limit 2 | 12/6 × 1.2 = 2.4 per min |
| S9-T5 | load: 20 active min at band 4 | 130 units |

Rounding is half-up (`floor(x + 0.5)`), stated in the formula text.
