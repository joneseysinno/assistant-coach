# 01 — Theory: Constraints, Games, and How Players Learn

> Status: draft for review · Scope: the learning theory the app is built on.
> Rule of thumb for every later design decision: **if a feature contradicts this document, the feature is wrong or this document needs a decision record.**

---

## 1. The one-sentence thesis

**A coach does not install movements in players; a coach designs environments in which the right movements and decisions are the easiest solutions to find.**

Everything in the app follows from that sentence. The app is a tool for *designing environments* (games), *shaping them* (constraints), *sequencing them* (practice plans and seasons), and *checking whether they worked* (observation). Plays are included, but they are reinterpreted through this lens (section 9).

---

## 2. Where this comes from: ecological dynamics in plain terms

The Constraints-Led Approach (CLA) comes out of **ecological dynamics**, which joins two ideas:

1. **Ecological psychology (Gibson).** Players don't see a neutral world and then compute a plan; they perceive *opportunities for action* directly — a gap to drive, a defender's high hip to cross over, a shooter's open window. Gibson called these **affordances**. An affordance belongs to the *player–environment pair*: the same gap is a driving lane for a quick 5'4" guard and not for a slow 6'0" forward.
2. **Dynamical systems (Bernstein, Kelso, Newell).** Movement is not a stored program replayed on cue. It **self-organizes** from many interacting parts (body, task, surroundings) and settles into stable patterns called **attractors**. Learning means building new attractors and loosening unhelpful ones.

Put together: **skill is a tight coupling between what a player perceives and what they do, tuned to a specific environment.** That is the opposite of the traditional view, where skill is a technique you perfect in isolation and then "apply" in games.

### Why this matters for practice design

If skill is perception coupled to action, then practice that removes the perception (stationary form shooting, two-line layups, three-man weave with no defense) trains a movement **detached from the information that will trigger it in a game**. The player gets good at the drill. Carry-over to the game is a hope, not a design.

This is called **representative learning design** (Brunswik; Pinder, Davids, Renshaw & Araújo, 2011): practice must *sample the information sources players will use in competition* — defenders, teammates, spacing, time pressure, the score.

---

## 3. Newell's constraints: the coach's three levers

Karl Newell (1986) proposed that behavior emerges from the interaction of three families of constraints:

| Family | What it includes (basketball) | Can the coach change it? |
|---|---|---|
| **Individual** | Height, wingspan, strength, speed, fatigue, confidence, current skill, attention, emotions | Slowly (training), or temporarily (fatigue, role assignment, handicaps) |
| **Task** | Rules, scoring, number of players, dribble limits, time limits, goals, equipment (ball size, rim height) | **Directly and instantly — this is the main lever** |
| **Environment** | Court size and shape, baskets available, lighting, noise, crowd, referees, game situation (score/clock) | Partly (court area, baskets, simulated score/clock) |

The coach's job becomes **choosing and tuning task and environmental constraints so that the behaviors you want emerge.** The app's core object is therefore not a "drill" — it is a **game plus a stack of constraints**.

### The five classic task-constraint dials

Nearly every constraint a coach invents is one of these:

1. **Space** — court length/width, zones (bonus zones, no-go zones, safe zones), number of baskets.
2. **Numbers** — 1v1 … 5v5, overloads (3v2), underloads, neutral players / jokers.
3. **Time** — shot clock, possession clock, "score within 3 passes," game length, work:rest.
4. **Rules & scoring** — what counts, how much, what's banned, what triggers a turnover. *Scoring is the most powerful constraint there is: players do what gets rewarded.*
5. **Equipment & bodies** — ball size, rim height, a player's non-dominant hand only, a defender's hands behind the back.

---

## 4. Attractors, fluctuations, and the "sweet spot" for decisions

- **Attractor:** a stable, preferred way of doing something. Example: a player who always drives right, or a team that stands and watches when the ball is in the post.
- **To change an attractor you have to destabilize it** — make the old solution stop working or stop paying. Example: right-hand finishes count 1, left-hand finishes count 3.
- **Fluctuations:** while a pattern is changing, behavior gets messy. That mess is *learning*, not failure. The app must not show it as failure (see measurement, section 11).
- **Metastability:** a region where two or more options are equally viable. A 2v1 where the defender is exactly far enough away that *both* the drive and the pass are good choices. **This is the most valuable place to practice decision-making**, because the player has to actually read the defender instead of following a habit.

> **Design implication (original to this app):** a decision game should be *tuned toward metastability* — its dials adjusted until the read is genuinely close to 50/50. If shooters shoot every time, the closeout starts too far away; move the defender closer. The app should help the coach find that point (see `03-app-design.md`, "Metastability tuner").

---

## 5. Repetition without repetition

Bernstein's phrase. Athletes should solve the *same problem* many times, but **never in exactly the same way**. Good games produce hundreds of repetitions of a *decision* in slightly different situations. That variability is useful:

- **Functional variability** — the movement adapts to the situation instead of being a rigid copy.
- **Degeneracy** — there are many ways to reach the same goal (a floater, a reverse layup, a pull-up). Good players have many; games should reward finding them.

Contrast with the blocked drill: 50 identical repetitions. It looks great in practice and is retained poorly. This matches the **contextual interference** effect (Shea & Morgan, 1979): mixed, interleaved practice feels worse in the moment and is retained better.

---

## 6. Nonlinear pedagogy: the teaching principles

Chow, Davids, Button & Renshaw put CLA into a teaching framework called **nonlinear pedagogy**. Its working principles, rewritten as rules for the app:

1. **Representativeness** — keep the key information (defender, teammate, basket, time) in the task.
2. **Simplify, don't decompose** — make the whole task easier (slower defender, bigger space, fewer players) instead of cutting it into pieces (shooting form with no defense, then defense later).
3. **Manipulate constraints** — change the task, not the instruction.
4. **External focus of attention** — cue the effect ("send the ball to the back of the rim," "beat the help"), not the body part ("snap your wrist"). (Wulf's research on attentional focus.)
5. **Functional variability** — vary on purpose.
6. **Individualize** — each player brings different individual constraints, so the same game is a different problem for each of them.

### Simplification vs. decomposition, concretely

| Learning goal | Decomposition (avoid as default) | Simplification (prefer) |
|---|---|---|
| Shooting off a closeout | Form shooting, no defense | 1v1 closeout from a *real* defender who starts at the rim and closes at reduced speed |
| Passing out of a drive | Two-line pass-and-catch | 2v1 to the rim; defender must commit before the pass counts |
| Help defense rotation | Walk-through shell drill | 4v4 with offense limited to 2 dribbles, so drives are short and rotations are learnable |

### Teaching Games for Understanding (TGfU)

TGfU (Bunker & Thorpe, 1982) adds two ways to modify a game:

- **Representation** — a smaller game that keeps the tactical structure of the full game (3v3 has pick, help, rotate, and closeout).
- **Exaggeration** — change rules so one tactical problem appears much more often than it would in a real game (offensive rebounds worth 3 → constant box-out battles).

Both are first-class operations in the app.

---

## 7. Small-sided games (SSGs) in basketball: what the evidence says

SSGs are the main delivery vehicle. Research findings to build into the app's effect estimates (as **rules of thumb, not physics**):

- **Fewer players → higher intensity per player.** 3v3 produced higher heart rate (~88% HRmax) and blood lactate than 5v5 on the same court (Castagna et al., 2011, cited in Marcelino et al., 2015). 2v2 produced higher HR and RPE than 4v4, with more sprints, high-intensity shuffling and jumps (Klusemann et al., 2012).
- **Fewer players → more involvements per player** (touches, shots, decisions). Every player in a 3v3 is always within one action of the ball; in 5v5 some players can hide.
- **Court size matters less than player number for internal load in some studies**, but larger courts tend to produce higher perceived exertion and more repeated-sprint decline (Marcelino et al., 2015; Klusemann et al., 2012: full court RPE ≈ 7 vs half court ≈ 6).
- **Work:rest shapes intensity.** Klusemann et al. found 2 × 5 min (30 s rest) produced higher HR and RPE than 4 × 2.5 min (60 s rest).

### Translating to app concepts

- **Relative area per player** — court area ÷ players on court. A standard SSG load indicator in football research, and a useful readout here. Example (high-school 84 × 50 ft court): 3v3 half court ≈ 350 ft²/player; 5v5 full court ≈ 420 ft²/player.
- **Active ratio** — players on court ÷ players at that basket. A 3v3 with 12 players waiting at one hoop has a 50% active ratio; with "winners stay, next team on" plus a second basket it can be ~100%. **For a coach with one gym and 12–15 kids, this number is often the biggest single lever on practice quality.**
- **Involvement density** — estimated touches/decisions per player per minute, scaling roughly with 1/(players on court), adjusted by constraints (dribble limits increase pass decisions; "must touch paint before a three" increases drives).

---

## 8. Periodization of skill: where SSGs sit (PoST)

Otte, Millar & Klatt (2019) proposed the **Periodisation of Skill Training (PoST)** framework. It is useful because it legitimizes simplified tasks *without* falling back on decontextualized drills:

1. **Coordination training** — find a functional movement pattern. Low variability, simplified but still representative (a slow, real closeout).
2. **Skill adaptability training**
   - *Movement variability* (1–2 athletes): same skill, deliberate variation.
   - *Complex training* (up to ~4 athletes): several linked skills, time pressure, fatigue.
   - *Team-based training* (larger groups): small-sided, conditioned and larger games close to match demands, with questioning.
3. **Performance training** — prepare for competition: opponent-specific games early in the week, stability and confidence close to the game.

> **App use:** every game declares its PoST stage. The season view can then show whether the team spends its time in the right stages for the time of year.

---

## 9. Where plays fit: the reconciliation

This is the main tension the app must solve. Strict CLA voices say "don't run plays." Real coaches run sets because sets organize spacing and timing. Both are right, once you reframe what a play is:

> **A play is a structured constraint.** It places players so that certain affordances appear in predictable places at predictable times. It does not — and should not — decide *what* the player does when the affordance appears. That is a **read**.

So in this app:

- A **play is a decision graph**, not an animation. Nodes are court states; edges are actions *and the reads that trigger them* ("if X₂ trails the curl → curl to the rim; if X₂ goes under → pop").
- **Every read in a play is a learning target** that should be trained in a game where that read appears again and again, with live defense.
- **"Plays create the question; games teach the answer."**

### The two-way loop (original to this app)

```
        top-down                                  bottom-up
   PLAY  ──decompose──►  READS  ──generate──►  GAMES
    ▲                                            │
    └────────── harvest emergent patterns ◄──────┘
```

- **Top-down:** design a set → app extracts its reads → app proposes SSGs that make those reads appear often (with representative defense).
- **Bottom-up:** during games, players invent good solutions (a spacing pattern, a re-screen, a cut). The coach captures them quickly and promotes them into the playbook. **The team's sets grow out of what the team finds, rather than being imposed on it.** For a young team this builds ownership (Humble/Attitude) and produces sets that suit *these* players' individual constraints.

---

## 10. A basketball learning-target taxonomy: game moments and principles

To tag games, plays and sessions consistently, the app uses **game moments × principles** instead of skill lists (dribbling, passing, shooting). Draft taxonomy:

**Offense**
- *Neutral → Advantage (create):* beat your player 1v1, ball screen, off-ball screen, cut, post seal, transition push.
- *Advantage → Bigger advantage (extend):* drive-and-kick, extra pass, attack the closeout, short roll, skip pass.
- *Finish (convert):* finish at rim vs. contest, shot selection, offensive rebound.
- *Shape:* spacing, timing, floor balance, relocation.

**Defense**
- *Deny advantage:* on-ball containment, screen coverage, deny/gap position.
- *Scramble (recover from disadvantage):* help, rotate, closeout, X-out, talk.
- *End possession:* contest without fouling, block out, secure rebound.

**Transition**
- *Offense:* outlet, fill lanes, early advantage.
- *Defense:* get back, stop the ball, match up.

**Character (program words)** — Humble, Hungry, Effort, Attitude, treated as **observable behaviors that constraints can reward** (section 12).

---

## 11. Measuring learning without bringing back technique coaching

Measurement is where many CLA programs quietly slip back into technique coaching ("count elbows in"). Rules for this app:

1. **Measure functional outcomes and decisions, not form.** Count "correct read at the closeout" / opportunities, "advantages created per minute," "paint touches per possession," "deflections."
2. **Expect fluctuation.** When a new constraint goes in, success can drop before it rises. Charts must show trend over sessions, not judge single sessions.
3. **Precision Teaching fits CLA well if pinpoints are decision-based.** Count-per-minute and the Standard Celeration Chart show *rate of learning*; a 7/8 mastery criterion on decision accuracy can trigger a progression (harder dial). This is the bridge to the challenge-point idea:
4. **Challenge point** (Guadagnoli & Lee, 2004): learning is best when difficulty is matched to the learner's current level. Practical use: if success on a decision is high and stable (e.g., 7/8 met across sessions), turn the difficulty dial up; if it collapses, turn it down. The dial is a constraint, so the response is *change the game*, not *add instructions*.

---

## 12. Character as constraint

Program values can be *designed into games* instead of only talked about:

| Word | Observable behavior | Constraint that rewards it |
|---|---|---|
| **Hungry** | Loose balls, offensive rebounds | Offensive rebound = +2; loose-ball recovery = possession + 1 |
| **Effort** | Sprinting back, closeouts, help | Defensive stop where all 3 defenders touch the paint = +1 |
| **Humble** | Extra pass, screening for others | Assist = +1; basket after a screener's re-screen = +1 |
| **Attitude** | Next-play response, encouragement | Team that's losing picks the next rule; "talk" calls counted by a player-observer |

---

## 13. The coach's role during games

- **Design, then step back.** Most of the teaching is already in the constraints.
- **Questions over instructions.** "What did the help defender do?" "Where was the open space?"
- **External-focus, analogy cues.** "Throw it to his outside hand," "be a door that closes."
- **Freeze-replay sparingly.** Short, question-led, and back into play quickly.
- **Change a dial instead of explaining more.** If the game isn't producing the behavior, the design is the problem.

---

## References

- Bernstein, N. (1967). *The Co-ordination and Regulation of Movements.* Pergamon.
- Bunker, D., & Thorpe, R. (1982). A model for the teaching of games in secondary schools. *Bulletin of Physical Education*, 18(1).
- Chow, J. Y., Davids, K., Button, C., & Renshaw, I. (2016; 2nd ed. 2021). *Nonlinear Pedagogy in Skill Acquisition.* Routledge.
- Gibson, J. J. (1979). *The Ecological Approach to Visual Perception.* Houghton Mifflin.
- Guadagnoli, M. A., & Lee, T. D. (2004). Challenge point: a framework for conceptualizing the effects of various practice conditions in motor learning. *Journal of Motor Behavior*, 36(2).
- Klusemann, M. J., Pyne, D. B., Foster, C., & Drinkwater, E. J. (2012). [Optimising technical skills and physical loading in small-sided basketball games.](https://www.bisp-surf.de/Record/PU201403001762) *Journal of Sports Sciences*, 30(14).
- Marcelino, P. R., et al. (2015). [Does small-sided-games' court area influence metabolic, perceptual, and physical performance parameters of young elite basketball players?](https://pmc.ncbi.nlm.nih.gov/articles/PMC4786584) *Journal of Human Kinetics.* (also summarizes Castagna et al., 2011).
- Newell, K. M. (1986). Constraints on the development of coordination. In Wade & Whiting (Eds.), *Motor Development in Children.*
- Otte, F. W., Millar, S.-K., & Klatt, S. (2019). [Skill training periodization in "specialist" sports coaching — an introduction of the "PoST" framework for skill development.](https://www.frontiersin.org/journals/sports-and-active-living/articles/10.3389/fspor.2019.00061/pdf) *Frontiers in Sports and Active Living*, 1:61.
- Pinder, R. A., Davids, K., Renshaw, I., & Araújo, D. (2011). Representative learning design and functionality of research and practice in sport. *Journal of Sport & Exercise Psychology*, 33(1).
- Renshaw, I., Davids, K., Newcombe, D., & Roberts, W. (2019). *The Constraints-Led Approach: Principles for Sports Coaching and Practice Design.* Routledge.
- Shea, J. B., & Morgan, R. L. (1979). Contextual interference effects on the acquisition, retention, and transfer of a motor skill. *Journal of Experimental Psychology: Human Learning and Memory*, 5(2).
- Wulf, G. (2013). Attentional focus and motor learning: a review of 15 years. *International Review of Sport and Exercise Psychology*, 6(1).
