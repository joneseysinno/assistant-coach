# S6 — Learning Targets & Reads

> Status: draft for review · Primitive: **P5 Read** + the learning-target taxonomy · Rules R2, R9, R12, R13 · Needed by: tagging, coverage, plays (S7), season grid (S8)

## 1. Purpose

Give every learning target and read a **stable id**, so tags written today still mean the same thing in a season's coverage chart. Ids never change; labels can.

## 2. Id rules

- Targets: `<moment>.<principle>[.<specific>]`, lowercase, hyphens, dots for levels. Example `off.extend.attack-closeout`.
- Reads: `read.<principle-slug>.<cue-slug>`. Example `read.closeout.high-fast`.
- Character: `char.<word>.<behavior>`.
- Retiring an id: mark `retired: true` with `replaced_by`. Never reuse an id.

## 3. Target tree (basketball pack v1)

| Id | Label |
|---|---|
| `off` | Offense |
| `off.create` | Create an advantage |
| `off.create.beat-1v1` | Beat your player |
| `off.create.ball-screen` | Use a ball screen |
| `off.create.off-ball-screen` | Use / set an off-ball screen |
| `off.create.cut` | Cut |
| `off.create.post-seal` | Post seal |
| `off.create.transition-push` | Push in transition |
| `off.extend` | Extend the advantage |
| `off.extend.drive-and-kick` | Drive and kick |
| `off.extend.attack-closeout` | Attack the closeout |
| `off.extend.extra-pass` | Extra pass |
| `off.extend.short-roll` | Short roll |
| `off.extend.skip-pass` | Skip pass |
| `off.finish` | Finish |
| `off.finish.rim-vs-contest` | Finish at the rim vs. a contest |
| `off.finish.shot-selection` | Shot selection |
| `off.finish.off-reb` | Offensive rebound |
| `off.shape` | Shape |
| `off.shape.spacing` | Spacing |
| `off.shape.timing` | Timing |
| `off.shape.relocation` | Relocation |
| `off.shape.balance` | Floor balance |
| `def` | Defense |
| `def.deny` | Deny the advantage |
| `def.deny.contain` | On-ball containment |
| `def.deny.screen-coverage` | Screen coverage |
| `def.deny.gap` | Deny / gap position |
| `def.scramble` | Scramble |
| `def.scramble.help` | Help |
| `def.scramble.rotate` | Rotate |
| `def.scramble.closeout` | Close out |
| `def.scramble.x-out` | X-out |
| `def.scramble.talk` | Talk |
| `def.end` | End the possession |
| `def.end.contest` | Contest without fouling |
| `def.end.box-out` | Box out |
| `def.end.secure` | Secure the rebound |
| `trans.off` | Transition offense |
| `trans.off.outlet` | Outlet |
| `trans.off.fill-lanes` | Fill lanes |
| `trans.off.early-advantage` | Early advantage |
| `trans.def` | Transition defense |
| `trans.def.get-back` | Get back |
| `trans.def.stop-ball` | Stop the ball |
| `trans.def.match-up` | Match up |
| `char.humble` | Humble — extra pass, screening for others |
| `char.hungry` | Hungry — loose balls, offensive rebounds |
| `char.effort` | Effort — sprint back, closeouts, help |
| `char.attitude` | Attitude — next-play response, encouragement, talk |

Interleaving (R12) and coverage count at the **principle level** (two dots) by default.

## 4. Read

```yaml
id: read.closeout.high-fast
target: off.extend.attack-closeout
actor: catcher                       # the role who must read (a role alias, bound by the game or play)
cue:
  text: "The closeout is high and fast"
  source: closing-defender           # whose behaviour *is* the cue
  look_for: "defender's feet still moving, hands high, weight forward"
options:
  - { id: drive, text: "Rip and drive past the top foot", correct: true }
  - { id: shoot, text: "Shoot", correct: false }
  - { id: pass,  text: "Swing it", correct: when-help-is-there }
outcome_pinpoint: { event: closeout, correct_when: { outcome: drive } }
```

| Field | Notes |
|---|---|
| `actor` | Role alias: `ball-handler`, `catcher`, `screener`, `weak-side`, `help-defender` … bound to concrete roles by the game/play |
| `cue.source` | **Which player's behaviour is the cue.** Generate-from-read (02 §6) puts live defense exactly on this role |
| `options` | ≥ 2. `correct` may be `true`, `false` or a condition — reads are rarely one-answer |
| `outcome_pinpoint` | How the read is measured (R13): the event + which outcomes count as correct. S10 counts "correct / opportunities" from this |

**Read pairs.** Most cues come in pairs that make a metastable decision (01 §4): `read.closeout.high-fast` ↔ `read.closeout.short`. A read may list `pair:` so the metastability tuner can plot the split.

## 5. Starter reads (pack v1)

| Id | Cue (source) | Options |
|---|---|---|
| `read.closeout.high-fast` | Closeout high and fast (closing defender) | drive ✔ · shoot |
| `read.closeout.short` | Closeout short / late (closing defender) | shoot ✔ · drive |
| `read.help.steps-up` | Help defender steps up to the drive (help defender) | kick to the help's player ✔ · finish |
| `read.help.stays` | Help stays home (help defender) | finish ✔ · kick |
| `read.ballscreen.drop` | Screener's defender drops (D-screener) | pull-up / pocket pass ✔ |
| `read.ballscreen.hedge` | Hedge / show (D-screener) | hit the popper / short roll ✔ |
| `read.ballscreen.switch` | Switch (both defenders) | attack the big / seal the small ✔ |
| `read.ballscreen.help-on-roll` | Weak-side defender helps on the roller (help defender) | lift / kick to the helper's player ✔ |
| `read.denial.overplay` | Defender denies the wing (on-ball wing defender) | back cut ✔ |
| `read.2v1.commit` | Defender commits to the ball (lone defender) | pass ✔ |
| `read.2v1.stays-home` | Defender stays with the receiver (lone defender) | finish ✔ |

## 6. Invariants

1. Every tag on a game/constraint/block resolves to a non-retired target.
2. Every read has ≥ 2 options and a `cue.source`.
3. R9: every read in a play links to ≥ 1 game (`trained_by`) or is flagged untrained.
4. R13: `outcome_pinpoint` refers to a decision/outcome event, never a body-form field.
5. R14: `cue.text` and option texts are checked by the cue linter (warn on body-part words).

## 7. Worked examples

| Test | Input | Expected |
|---|---|---|
| S6-T1 | Tag `off.extend.attack-closeout` | rolls up to `off.extend` for interleaving |
| S6-T2 | Read option text "snap your wrist" | R14 warning |
| S6-T3 | Closeout game log: 10 `closeout` events, 7 `outcome: drive` on high-fast cues | correct 7 / 10 opportunities (needs cue known — §8.1) |

## 8. Open questions

1. **Measuring a read needs the cue.** A coach tapping "closeout → drive" doesn't record whether the closeout was high or short. Options: (a) two buttons per outcome ("high → drive", "short → shoot"), (b) tap only *correct / incorrect* and let the coach judge, (c) measure the split only (metastability) and not correctness. Proposed: (b) for live (one tap: ✔ / ✘), (a) for video review. Your call — it shapes S10 and S12.
2. Do you want your own words for the character behaviours (Humble/Hungry/Effort/Attitude) as they appear on the 14-week plan? I can replace the labels above.
