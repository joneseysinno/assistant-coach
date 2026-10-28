# S12 — Remote & Board

> Status: draft for review · Decisions: D-010, D-014, D-021, D-025, D-027 · Needed by: **P1 (live-first)** · Layouts are provisional until the gym network/readability test (W4.6) and the live dry-run (W4.7)

## 1. Purpose

Exactly what the phone and the TV show, how buttons come from rules, and the type sizes, so the wasm screens can be built without guessing.

## 2. Button generation (pure function in `ac-live`)

`buttons(resolved_game) -> Vec<ButtonView>`:

1. **Scoring buttons** = rules with a non-zero `points` consequence and `judge: coach`. One per rule **per team column** — the tap says which team it's for.
   - Order: **base rules first** (Basket, Three — same place in every game, so thumbs learn it), then constraint rules in stack order.
   - Label = `rule.label` (≤ 16 chars) + the points: "Kick-out 3 +3". Defense-credited rules show the team that *defends* — the coach taps the column of the team that earned it.
2. **Calls tray** = `judge: coach` rules with no points (violations, turnovers): one row, team-neutral; tapping asks "which team?" only if possession is unknown, otherwise applies to the team in possession.
3. **Pinpoints** = `tally` rules: at most 2 shown, in a separate strip with a count.
4. **Players' rules** (`judge: players`) are not buttons; they appear under a collapsed "Rules" drawer so the coach can enforce one (logs `rule.fired` like any other).
5. **Live dials** = an enum picker strip (coverage card) above the scoreboard.
6. **Overflow:** more than 5 scoring buttons per team → the 5th becomes "More…" (sheet).

Golden tests: each fixture's `expect.buttons_per_team` / `expect.calls`.

## 3. Remote layouts (portrait phone, one thumb)

**A. One basket (default)**
```
┌──────────────────────────────┐
│ Closeout Attack 3v3   A  ⟲ ⓘ │  title · basket · sync status
│ 06:12 ▶      Practice −2:00  │  block clock · schedule balance
├──────────────┬───────────────┤
│  RED   5     │  BLUE   3     │  tap a score to see its history
│ [Basket +1]  │ [Basket +1]   │  ≥ 56 px tall, full column width
│ [Three  +1]  │ [Three  +1]   │
│ [Kick-out3+3]│ [Kick-out3+3] │
│ [Corner  +1] │ [Corner  +1]  │
│ [Off reb +1] │ [Off reb +1]  │
├──────────────┴───────────────┤
│ Calls: [Too many dribbles] [Stop ⇄]          │
│ Pinpoint: [Closeout ✔ 6] [Closeout ✘ 2]      │
├──────────────────────────────┤
│ [ Undo ]  [ ⏸ ]  [ +1 min ]  [ Next ▸ ]       │  head role only for the last three
└──────────────────────────────┘
```
**B. Two baskets:** a segmented control `A | B` at the top; each scorer can be *locked* to one basket by the head.
**C. Between blocks:** next game card (rules from the renderer), teams with colors (tap to reshuffle), `Start ▶`.
**D. Overtime:** the bottom bar turns red and shows the choices: **Next ▸ · +1 min (from: Shell 4v4 → 7 min) · Finish possession · Keep going** (firm policy countdown visible).

Interaction rules: every tap gives instant local feedback (pressed state + 10 ms vibrate), then shows ✓ when acked; un-acked > 2 s shows a small clock icon. Undo lists the last 5 entries with time and label.

## 4. Board layouts (TV, 16:9, landscape)

| State | Content |
|---|---|
| **Running — one game** | Game name · block n of N · team scores huge · block clock · shot clock (if any) · 3 rule phrases · next block |
| **Running — two baskets** | Two halves, each: basket label, game name, two scores, its shot clock; one shared block clock center-top |
| **Last minute** | Clock and frame amber |
| **Overtime** | Red frame, `+1:23` counter, "Practice +3 min behind" |
| **Interval rest** | Calm blue, rest countdown, "Next interval: 2 of 3" |
| **Between blocks** | Next game name, its card rules (up to 5), teams with colors, `Starts in 0:45` |
| **Water break** | Countdown only |
| **Prompt line** | Optional question under the rules ("Where is the help coming from?") |
| **Pairing** | QR + scorer PIN (shown between blocks and on demand; hidden while running) |

## 5. Type sizes (provisional — set by the gym test)

Sized in `vh` so they scale with the TV:

| Element | Size | Approx. height on a 55″ TV (27″ tall) |
|---|---|---|
| Team scores | 30 vh | 8.1″ |
| Block clock | 22 vh | 5.9″ |
| Shot clock | 14 vh | 3.8″ |
| Game name | 7 vh | 1.9″ |
| Rule phrases | 6 vh | 1.6″ |
| Next / prompt | 4.5 vh | 1.2″ |

Rule of thumb for signage: roughly 1 inch of letter height per 10 ft for comfortable reading (much more distance is readable, but not at a glance). By that rule scores read across a full court, clocks to about half court plus, and **rules only near the TV**. The W4.6 test confirms where each line stops being readable; the result sets these numbers.

## 6. Colors and themes

- Team colors from the Block's pinnies; text on a team color is auto black/white by contrast (WCAG ≥ 4.5:1).
- Board theme: near-black background, off-white text, no thin fonts (weight ≥ 600), tabular numerals for clocks.
- Never color alone: amber/red states also change the frame thickness and add a label.

## 7. Sound

Horn at block/interval end (from the TV via HDMI, triggered by the Board page on the hub's `alert`); optional chirp at 1:00. The Board page needs one user click after load to unlock audio (browser autoplay rule) — the setup checklist includes it.

## 8. Open questions
1. Do scorers mostly hold the phone **portrait** one-handed? (Assumed.)
2. Team names: colors only ("Red/Blue") or player-chosen names?
