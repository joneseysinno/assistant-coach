# 06 — Live Practice: Scoreboard, Clock and Remote

> Status: draft for review · Scope: running a practice live, across laptop, TV, phone and tablet. Covers what it does, how the devices connect, the clock and score model, and the guard against drills running long.

---

## 1. What it has to do

In the gym, during practice:

1. **Keep score** for the current game, using that game's own scoring rules. The coach or an assistant can both score.
2. **Track time**: the clock for the current block, intervals inside the block (e.g., 3 × 4 min with 1 min rest), team rotations, and an optional shot clock.
3. **Show it on the TV** so players can see the score, the time, the game name and the key rules.
4. **Keep the coach on schedule**: warn before a block ends, make overtime obvious, show the cost to the rest of practice, and make "next block" a single tap.
5. **Work on a laptop, a tablet and an Android phone.** It must not depend on the internet: the router only provides a local network.

---

## 2. The gym setup

```
                     ┌──────────────── HDMI ────────────────┐
                     │                                      ▼
  ┌──────────────────┴───┐     local Wi-Fi      ┌────────────────────┐
  │  LAPTOP  = the HUB   │◄────(your router)───►│  TV  = the BOARD   │
  │  · runs the practice │                      │  (laptop's 2nd     │
  │  · holds the clock   │                      │   screen, full     │
  │  · holds the score   │                      │   screen browser)  │
  │  · saves the log     │                      └────────────────────┘
  └──────────▲───────────┘
             │ local Wi-Fi (browser → http://laptop-ip:port)
     ┌───────┴────────┬────────────────┐
     │ PHONE = REMOTE │ PHONE = REMOTE │  TABLET = REMOTE or STUDIO
     │ (head coach)   │ (assistant)    │
     └────────────────┴────────────────┘
```

- The **laptop is the hub**. It runs the desktop app, which includes a small local web server.
- The **TV** is just a screen attached to the laptop. The Board view runs full-screen on it.
- **Phones and tablets join through the browser.** Nothing is installed from an app store. The coach scans a QR code on the TV or laptop and enters a short PIN.
- **No internet is needed.** Everything stays inside the router's local network.

---

## 3. Three surfaces, one app

| Surface | Device | Purpose | UI style |
|---|---|---|---|
| **Studio** | Laptop (and tablet) | Design games, plays, practices, seasons | The JoInn shell: rails, panels, viewport |
| **Board** | TV (laptop's second screen) | What players see | Display only. Huge type, high contrast, readable from half court |
| **Remote** | Phone, tablet, laptop | Run the practice: score, clock, next block | Touch-first: big buttons, one hand, no panels |

All three share the same domain code (`ac-domain`). Studio uses the JoInn shell. **Board and Remote do not.** They are their own simple layouts, because a dense CAD-style shell is the wrong tool for scoring with one thumb.

**How they're built (proposal):** the laptop app embeds the hub server. The Remote and Board are compiled to web (wasm) and served by the hub, so any browser can be a Remote or a Board. One codebase, no Android toolchain needed for v1. A native Android build (Dioxus mobile) stays a later option if the browser version hits limits (§8).

---

## 4. The hub: one source of truth

**Rule: the laptop owns the state. Every other device sends requests and displays what the hub tells it.**

- **Requests go up:** `score(team A, rule "corner three")`, `start clock`, `pause`, `next block`, `+1 min`, `undo last`.
- **State comes down:** the current block, clocks, scores and the schedule status, pushed to every device over a WebSocket.
- **One ordered log.** Every request that changes something is appended to a timestamped event log on the laptop. That one log gives:
  - **Undo:** "undo last score" removes a log entry instead of guessing.
  - **Crash recovery:** if the app restarts, replaying the log restores the scores and clocks.
  - **Observation data for free:** the scoring taps *are* Events (`02` P3), so after practice the log already holds paint touches, stops, offensive rebounds and so on. Scorekeeping and tallying become the same act.
  - **Scheduling history:** how long each block actually ran compared with the plan (§7.4).
- **Two scorers at once** is safe, because the hub puts requests in order. Each tap carries a unique id, so a phone that reconnects and resends doesn't double-score.

### Clocks

- Clocks run **only on the laptop**. Phones and the TV *display* the hub's time; they never count on their own.
- Each device measures its delay to the hub with a ping, so displays don't visibly lag (the same idea as NTP, much simpler).
- A clock is stored as "started at T, paused total P, duration D", not as "seconds left" ticking down. Remaining time is always calculated from those values, so it can't drift and it survives reconnects.

### Pairing and roles

- The TV shows a QR code (the hub's URL) and a 4-digit PIN that changes each session.
- **Head** role: everything (clock, next block, change the plan live).
- **Scorer** role: score and undo for one basket/game.
- **Viewer** role: read-only (an assistant's tablet, a manager).
- This keeps a curious player's phone from controlling the scoreboard.

---

## 5. Scoring buttons come from the game's rules

This is the payoff of structured scoring rules (`02` R5). The Remote's buttons are **generated from the active game's rules**, so the coach never sets up a scoreboard:

```
 Closeout Attack 3v3 · Basket A          06:12 ▶
 ┌───────────────┐   ┌───────────────┐
 │  RED   5      │   │  BLUE   3     │
 ├───────────────┤   ├───────────────┤
 │ Kick-out 3 +3 │   │ Kick-out 3 +3 │
 │ Basket   +1   │   │ Basket   +1   │
 │ Off reb  +1   │   │ Off reb  +1   │
 │ Corner catch+1│   │ Corner catch+1│
 └───────────────┘   └───────────────┘
 [ Undo ]   [ Turnover ]   [ Stop ⇄ ]
 ───────────────────────────────────────
 [ ⏸ Pause ]   [ +1 min ]   [ Next ▸ ]
```

- Simple games just show **+1 / +2 / +3**.
- Rules with conditions that need a judgment ("three *after* a paint touch") are still one tap: **the coach judges, the app adds it up.** The app does not try to detect events on its own.
- **Limit rules** (dribble limit, shot clock) can appear as one-tap "violation" buttons that apply the consequence (turnover/possession change).
- Two baskets running at once (cross-court 2v2s, two 3v3s) show **two scoreboards**. The Remote switches between them, or each assistant takes one basket.

---

## 6. The Board (TV)

Players read it from across the gym, so it shows few things in very large type:

```
┌──────────────────────────────────────────────────────────────┐
│  CLOSEOUT ATTACK 3v3                          BLOCK 4 of 9   │
│                                                              │
│        RED    5          4:12          3    BLUE             │
│                                                              │
│   Kick-out 3 = 3   ·   Off reb = +1   ·   2 dribbles max     │
│                                                              │
│   NEXT ▸ Transition 3v2 → 2v1  (8 min)                       │
└──────────────────────────────────────────────────────────────┘
```

- **Rules on the TV are coaching.** In a constraints-led practice, players knowing the constraint *is* the instruction. The top 2–3 rules come straight from the plain-English rule renderings.
- **Team colors follow pinnies** (set per block or per session).
- **Layouts:** one game · two baskets side by side · rest/transition screen (next game, its rules, the teams) · water break countdown.
- **Optional prompt line:** the question the coach wants players thinking about ("Where is the help coming from?").
- **States the TV shows:** last minute = amber, overtime = red with the overtime counter, interval rest = a calm color.
- **Sound** comes out of the TV speakers through HDMI: a horn at end of block/interval, an optional chirp at 1 minute.

---

## 7. Keeping drills from running long

This is a habit problem as much as a timer problem, so it uses several layers.

### 7.1 Make the time visible
- The Board and Remote always show the block clock and **"Practice: +3 min behind"**, a running schedule balance.

### 7.2 Warn early
- At a set warning time (default 1 min): amber, a chirp, and the phone vibrates.
- At zero: horn, red, and the overtime counter starts.

### 7.3 Make overtime cost something
- Overtime eats into later blocks, and the Remote shows **exactly what will be cut**: "Staying 2 more minutes cuts *Free-throw ladder* to 3 min."
- The coach's choices are always one tap: **Next ▸** · **+1 min (from: block X)** · **Finish this possession**.
- **Policy setting** (per practice):
  - *Soft:* warn only.
  - *Firm:* auto-advance after a grace period (e.g., 60 s) unless the head coach taps "keep going."
  - *Locked:* auto-advance at zero. Useful when an assistant runs a station.
- **Protected blocks:** some blocks (e.g., the end-of-practice game, free throws) can be marked uncuttable, so overtime comes out of the flexible ones.

### 7.4 Learn from it
- The log records planned vs. actual minutes for every block. After practice: "Closeout Attack ran +4 min (3rd time this month)." Over a season this shows which games the coach consistently over-runs. That points to either a habit or a plan that keeps budgeting too little time.
- **Out-of-the-box use:** some overtime is a *signal*. A game that keeps running long may be one players are still learning from (metastable, engaged). The review can show the overtime next to that game's success rates, so the coach can tell "I lost track of time" apart from "this game was still producing learning."

---

## 8. Phones, tablets, browsers: practical gotchas

| Issue | Detail | Plan |
|---|---|---|
| **Screen sleeping** | The Screen Wake Lock API needs a *secure context* (HTTPS or localhost). A phone reaching the laptop over plain `http://192.168.x.x` isn't one | v1: set the phone's screen timeout longer during practice, or whitelist the hub address in Chrome on your own phone (`chrome://flags` → "Insecure origins treated as secure"). Later: a local HTTPS certificate, or a native Android build |
| **Vibration** | Supported in Chrome on Android; not on iPhone Safari | Fine for an Android phone; always pair vibration with a visual and sound cue |
| **Windows firewall** | Will block the hub's port on first run | First-run setup step: allow on private networks |
| **Laptop sleep / lid** | Lid-close or sleep would stop the hub | Run mode blocks sleep while active; the setup checklist says to set lid-close to "do nothing" |
| **Wi-Fi drop** | Phone loses connection briefly | Remote shows "reconnecting," resends queued taps with ids (no double score), and reloads state on reconnect. The laptop's own keyboard can always run the clock |
| **Laptop IP changes** | The router gives the laptop a new address | QR code always shows the current address; optionally reserve the laptop's IP in the router |
| **Readability** | TV across the gym | Board type sized by distance; high-contrast theme separate from the Studio theme |

---

## 9. Standalone mode (no laptop)

Sometimes there won't be a laptop, TV or router (an away gym, an outdoor court). A **phone-only mode** would run the hub logic inside the phone's browser: clock, score and schedule on one device, no Board. This could be a later phase. The architecture allows it because the hub logic lives in `ac-domain`/`ac-live` (pure Rust) and can compile to wasm. **Decision needed:** v1 or later?

---

## 10. New spec and decision items

**Specs to add in P0**
- **S11 Live protocol:** requests, state messages, the log format, clock math, ids/idempotency, roles/pairing, reconnect.
- **S12 Remote & Board layouts:** the screens in §5–§6; generating buttons from rules; Board layouts and states; type sizes for viewing distance.
- **S13 Schedule guard:** overtime policies, the cut algorithm (which blocks shrink and by how much), protected blocks, warning settings.

**New decisions**

| ID | Decision |
|---|---|
| D-009 | Laptop is the hub and single source of truth; other devices send requests and display state |
| D-010 | Three surfaces (Studio / Board / Remote); Board and Remote don't use the JoInn shell |
| D-011 | Remote and Board are browser clients served by the hub (wasm); native Android is later |
| D-012 | Clocks run only on the hub, stored as start + paused + duration; clients display |
| D-013 | Every scoring/clock action is an Event in an append-only log; observation data comes from scorekeeping |
| D-014 | Remote scoring buttons are generated from the active game's Rules |
| D-015 | Schedule guard with soft/firm/locked policies and protected blocks |
