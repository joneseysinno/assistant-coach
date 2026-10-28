# S13 — Schedule Guard

> Status: draft for review · Decisions: D-015, D-022 · Needed by: **P1 (live-first)** · Defaults: firm, 60 s grace, warn at 1:00

## 1. Purpose

Keep practice on time without taking control away from the coach: warn early, make overtime cost something visible, and record planned vs. actual so habits show up over a season.

## 2. Settings (per practice, defaults from Settings)

| Setting | Default | Notes |
|---|---|---|
| `policy` | **firm** | soft / firm / locked |
| `warn_s` | 60 | amber + chirp + vibrate |
| `grace_s` | 60 | firm only: time after zero before auto-advance |
| `hard_end` | false | if true, practice may not run past `start + planned_minutes`; overtime must be recovered |
| `min_block_s` | max(180, 50% of block) | per block unless the block sets `min_minutes` |

## 3. Timeline of one block

```
 planned end − warn_s      planned end (0:00)        + grace_s
 ───────────●──────────────────────●─────────────────────●──────▶
          amber, chirp,          horn, red,           firm: auto-advance
          vibrate                overtime counter     (unless Keep going)
                                 starts               locked: advance at 0:00
                                                      soft: nothing
```
- **Keep going** (head): cancels auto-advance for this block; the overtime counter keeps running; the Remote shows the live cut preview.
- **Finish possession**: advance at the next `possession.end` on that basket (or after 60 s, whichever first).
- **+1 min (from X)**: adds 60 s to this block and takes 60 s from block X now.

## 4. The cut algorithm

Overtime `O` seconds (or an explicit +N) has to come from somewhere unless `hard_end` is false and the coach accepts running late.

```
fn plan_cut(blocks_after_current, O, pick: Option<BlockId>) -> Vec<Cut>:
    if let Some(x) = pick:                         # coach chose "from X"
        take = min(O, x.planned − x.min);  cut x by take; O −= take
    for b in blocks_after_current.reversed():      # latest flexible block first
        if b.protected: continue
        take = min(O, b.planned − b.min)
        if take > 0: cut b by take; O −= take
        if O == 0: break
    if O > 0: report "runs late by O" (hard_end → must choose: skip a block or accept)
```

- **Why latest first:** the last flexible block is usually the one the coach is least attached to by the time it arrives, and cutting it keeps the next block exactly as planned. Proportional cutting spreads small losses across everything and changes every later block — harder to run.
- The Remote's default `+1 min (from: X)` names the block this algorithm would cut first, so the cost is visible before the tap.
- Cuts round to whole 30 s.

## 5. Planned-vs-actual record (per block, in the session)

```yaml
- block: b4
  planned_s: 600
  started_at: "15:52:10"
  ended_at: "15:56:40"
  actual_s: 870
  overtime_s: 270
  extensions: [ { at_s: 600, seconds: 60, from: b7 }, { at_s: 660, seconds: 60, from: b7 } ]
  cuts_received: 0
  advanced_by: head-next | auto-firm | auto-locked | finish-possession
  overtime_note: null        # optional coach tag: "still learning" | "lost track" | "setup issue"
```
The optional tag after a long overrun feeds the review's "signal vs. habit" split (06 §7.4).

## 6. Tests (P1)

| Test | Plan (minutes; P = protected; min) | Event | Expected |
|---|---|---|---|
| S13-T1 | b5=10, b6=8, b7=6, b8=6P | 3 min overtime in b5 | b7 6→3 (min 3); b6, b8 unchanged |
| S13-T2 | same | 5 min overtime in b5 | b7 6→3, b6 8→6 (min 4) ; 0 late |
| S13-T3 | same, `+1 min (from b6)` | — | b6 8→7 |
| S13-T4 | firm, grace 60 | no tap | auto-advance at +60 s, `advanced_by: auto-firm` |
| S13-T5 | firm, Keep going at +30 s | — | no auto-advance; overtime counter continues |
| S13-T6 | locked | — | advance at 0:00 |
| S13-T7 | all later blocks protected, hard_end | 2 min over | "runs late by 2:00" prompt |
