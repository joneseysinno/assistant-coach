# S10 — Observation & Storage of Results

> Status: draft for review · Rules R13, R15 · Decisions: D-008, D-013, D-035 · Needed by: Review (later); the live log format is fixed now so data collected from the first live practice is usable later

## 1. Purpose

Define what a **pinpoint** is, how counts become rates, the **7/8 mastery criterion**, and how observations link to blocks, players and the live log.

## 2. Pinpoint

A pinpoint is a countable decision or outcome (R13), defined as:
```yaml
pinpoint:
  id: closeout-correct
  label: Closeout decisions correct
  counts: { rule: closeout-ok }           # or { event: closeout, where: { outcome: drive } }
  opportunities: { rules: [closeout-ok, closeout-miss] }   # optional: makes it a ratio pinpoint
  per: team | player | block
```
- **Rate pinpoint:** count per minute of *active* time (`count / active_minutes`).
- **Ratio pinpoint:** correct / opportunities (used for the 7/8 criterion).
- Because pinpoint buttons are rules (`then: [tally]`, D-035), every pinpoint tap is a log entry and the count is a fold over the log.

## 3. Timing for rates

`active_minutes` for a block = block running time (clock running, excluding pauses and interval rests), from the log's clock entries. For a player: × player_share (S8 §4) unless the coach tagged the player on the tap.

## 4. The 7/8 criterion — needs your definition

Precision-teaching programs use "7/8" in more than one way. **Pick one (or tell me yours):**

| Option | Definition | Good for |
|---|---|---|
| **A. 7 of last 8 opportunities** | Within a block, the most recent 8 opportunities contain ≥ 7 correct | Fast in-practice dial turns |
| **B. 7 of 8 sessions** | The pinpoint met its aim in ≥ 7 of the last 8 sessions | Mastery over time |
| **C. 7/8 accuracy at an aim rate** | Accuracy ≥ 87.5% **and** rate ≥ aim (count/min) in the session | Fluency (accuracy + speed) |

Proposed default: **C** for decision pinpoints shown on the Standard Celeration Chart, with **A** as the live "turn the dial" signal on the Remote. Whatever you choose becomes `criterion:` on the pinpoint and a P4 test.

## 5. Record shape (derived from the log; never typed in)

```yaml
observation:
  session: se-2026-11-03
  block: b4
  game: g-closeout-attack-3v3      # with the variant + dial values in force
  dials: { dribble-limit.limit: 2, closeout_dist: 6 }
  pinpoint: closeout-correct
  correct: 11
  opportunities: 14
  active_min: 9.5
  rate_per_min: 1.16
  by_team: { red: { correct: 6, opp: 7 }, blue: { correct: 5, opp: 7 } }
```
Observations are a **view** computed from the log (stored as a cached entity for charts; recomputable).

## 6. Links
- Observation → Block → Practice → Session (live) via S14 edges.
- Each dial change during the block (`dial.set`) splits the observation into segments, so success can be plotted **against the dial value** (metastability tuner, challenge point).

## 7. Tests (P4)
| Test | Log | Expected |
|---|---|---|
| S10-T1 | 14 closeout taps, 11 ok, 9.5 active min | rate 1.16/min, accuracy 0.786 |
| S10-T2 | Option A, last 8 = ✔✔✘✔✔✔✔✔ | criterion met (7/8) |
| S10-T3 | dial changed at entry 30 | two segments with separate accuracy |
