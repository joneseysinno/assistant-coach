# S3 — Space & Court

> Status: draft for review · Primitive: **P1 Space** · Decisions: D-005, D-020, D-030, D-034 · Needed by: court canvas, area/player (S9), zone rules (S2), Board two-basket layout (S12)

## 1. Purpose

Give every game, play and logged event one unambiguous geometry: where the basket is, which way is "up", what "the corner" means, and how a game written once runs on any basket in any gym.

## 2. Definitions

| Term | Meaning |
|---|---|
| **Gym** | Settings data: a court preset (or custom) + the list of baskets in it (D-020) |
| **Basket** | A hoop in a gym: `id`, `label`, `position`, `facing`. Main baskets A/B are on the baselines; side baskets are on the sidelines |
| **Gym frame** | Coordinates for the whole gym: feet, origin at **center court**, +y toward basket **A**, +x to the right of a person standing at center court facing A |
| **Basket frame** | Coordinates for one basket: feet, origin at the **center of the rim** (projected to the floor), **+y toward half court** (away from the baseline), **+x to the right of a player facing the basket from half court** — the "right wing" side. Diagrams draw the basket at the top |
| **Space** | What a game plays on: a *court region* (the playable area, a shape in basket frame) + *zones* |
| **Zone** | A named shape inside the space with a `kind` for styling. **Zones do nothing by themselves** — rules reference them (D-034) |
| **Named region** | A zone shape supplied by the sport pack (paint, corners, …), computed from the gym's line set |

All game data is in **basket frame**. Only a Block (placing a game on a basket) and the Board's two-basket view need the gym frame.

## 3. Court presets (sport pack data)

Values in feet. *Rim offset* = rim center to baseline (4 ft backboard + 15 in). Every value is pack data, not code, so a gym with odd lines is a custom preset.

| Preset | Length × width | Rim offset | Lane width | FT line (from rim) | 3-pt arc radius | Corner 3 (|x|) | Restricted arc |
|---|---|---|---|---|---|---|---|
| `hs` NFHS high school | 84 × 50 | 5.25 | 12 | 13.75 | 19.75 | 19.75 (straight to baseline) | none |
| `ms` middle school | 74 × 42 | 5.25 | 12 | 13.75 | 19.75 | 19.75 | none |
| `ncaa` | 94 × 50 | 5.25 | 12 | 13.75 | 22.15 | 21.65 | 4 |
| `nba` | 94 × 50 | 5.25 | 16 | 13.75 | 23.75 | 22.0 | 4 |
| `custom` | any | any | any | any | any | any | any |

> **Verify at each gym:** many middle-school gyms are full HS size, and some have only HS lines painted on a short court. The gym profile records *what is painted*, because players react to painted lines. Measurements above are from rulebook values as I know them; confirm NFHS/NCAA current lines before P1 locks the pack.

**Derived:** half-court line at `y = L/2 − rim_offset` (HS: 36.75). Baseline at `y = −5.25`. Sidelines at `x = ±W/2`.

## 4. Basket placement in a gym

```yaml
gym:
  id: gym-home
  name: Home gym
  court: hs            # preset id, or a custom block
  baskets:
    - { id: A,  label: "Main A", position: [0,  36.75], facing: 180 }   # gym frame; facing = compass direction the backboard faces (deg; 0 = +y, 90 = +x)
    - { id: B,  label: "Main B", position: [0, -36.75], facing: 0 }
    - { id: S1, label: "Side 1", position: [-19.75, 18], facing: 90 }    # side hoop on the left sideline (rim 5.25 ft in from x = -25)
    - { id: S2, label: "Side 2", position: [ 19.75, 18], facing: 270 }
```

Transform basket frame → gym frame: with facing θ, forward `f = (sin θ, cos θ)` and right `r = (−f.y, f.x)`, `gym = position + local.y·f + local.x·r`. (Basket A: θ = 180 → f = (0, −1), r = (1, 0), so a player attacking A has gym +x on their right.) Side baskets have no painted lines unless the profile says `lines: hs`; their default court region is `cross-court` (§5).

## 5. Court regions

| Region id | Shape (basket frame) | Area (HS) | Use |
|---|---|---|---|
| `half` | rect x ∈ [−W/2, W/2], y ∈ [−rim_off, L/2 − rim_off] (HS: ±25, −5.25…36.75) | 2100 ft² | Most SSGs |
| `full` | rect x ∈ [−W/2, W/2], y ∈ [−rim_off, L − rim_off] (HS: to 78.75) | 4200 ft² | Transition, 5v5 |
| `three-quarter` | rect, y to 57.75 | 3150 ft² | Press break, advantage games |
| `cross-court` | rect, the gym's width as length: y ∈ [−rim_off, W − rim_off] (HS: to 44.75), x ∈ [−half_length/2, +half_length/2] (HS: ±21) | ≈ 2100 ft² | Side baskets (stress #8) |
| `inside-arc` | the 3-pt arc + straight segments | ≈ 750 ft² | Post / finishing games |
| `custom` | any shape (§6) | computed | — |

`area(region)` is exact for rects/circles/polygons; arcs are polygonized at 2° steps for area and hit-testing.

## 6. Zone shapes

```yaml
zones:
  - { id: corner-L, kind: bonus, shape: { named: corner-L } }
  - { id: no-drib,  kind: no-dribble, shape: { rect: [-25, 22, 25, 36.75] } }        # x0,y0,x1,y1
  - { id: start-ring, kind: start, shape: { circle: { center: [0,0], r: "$closeout_dist" } } }
  - { id: lane-plus, kind: restricted, shape: { band: { r: [0, 8], deg: [-90, 90] } } } # annular sector around the rim, 0° = +y
  - { id: dunker, kind: target, shape: { polygon: [[6,-5.25],[12,-5.25],[12,3],[6,3]] } }
  - { id: perimeter, kind: safe, shape: { minus: [ { named: half }, { named: inside-arc } ] } }
```

| Shape | Fields | Notes |
|---|---|---|
| `named` | region id from the pack | Recomputed when the gym's lines change |
| `rect` | x0, y0, x1, y1 | Axis-aligned in basket frame |
| `circle` | center, r | |
| `band` | r: [min, max], deg: [from, to] | Annular sector about the rim; covers "within 8 ft", "beyond the arc", "restricted arc" |
| `polygon` | ≥ 3 points | Counter-clockwise |
| `union` / `minus` | list of shapes | Only these two set operations |
| `mirror` | shape, axis: x | Makes L/R pairs from one definition |

Any number in a shape may be a dial reference `"$name"` (S5 §4). Zones are clipped to the court region.

## 7. Named regions (basketball pack)

Computed from the line set; `L`/`R` follow +x = right.

| Id | Definition (HS lines) |
|---|---|
| `paint` | rect x ∈ [−6, 6], y ∈ [−5.25, 13.75] |
| `restricted` | band r ∈ [0, 4], deg [−90, 90] (only when the preset has one) |
| `block-L/R` | circle r 2 at (∓7, 1.75) — the first lane space |
| `short-corner-L/R` | rect x ∈ ±[6, 14], y ∈ [−5.25, 4] |
| `dunker-L/R` | rect x ∈ ±[6, 10], y ∈ [−5.25, 2] |
| `elbow-L/R` | circle r 2.5 at (∓6, 13.75) |
| `corner-L/R` | outside the arc, y < 3 ("below the break"), |x| > corner value |
| `wing-L/R` | outside the arc, 3 ≤ y, 30° ≤ |angle from +y| ≤ 75° |
| `top` | outside the arc, |angle| < 30° |
| `above-break` | outside the arc and y ≥ 3 |
| `mid-range` | inside the arc, outside the paint |
| `inside-arc` | inside the arc (incl. paint) |
| `beyond-arc` | outside the arc, inside the court region |
| `backcourt` | y > half-court line (full-court regions only) |

## 8. Invariants

1. All stored geometry is basket frame, feet, `f32` in the domain (`i32` tenths in storage, S14).
2. A zone id is unique within a resolved game (constraint zones are namespaced `constraint-id.zone-id` if they clash).
3. A zone's `kind` only styles it. Validator warns if a zone is referenced by no rule (dead zone) — except `start` zones, which formats use.
4. Named regions are recomputed from the gym's line set; a game never stores computed polygons.
5. `area` and point-in-zone are pure functions with unit tests on the values in §5/§7.

## 9. Worked examples (become P1 tests)

| Test | Input | Expected |
|---|---|---|
| S3-T1 | `area(half, hs)` | 2100.0 |
| S3-T2 | `area(half, ms)` | 42 × 37 = 1554.0 |
| S3-T3 | point (−22, 1) in `corner-L` (hs) | true |
| S3-T4 | point (−18, 1) in `corner-L` (hs) | false (inside the arc) |
| S3-T5 | point (0, 20) in `top` (hs) | true (r = 20 > 19.75) |
| S3-T6 | point (5, 10) in `paint` | true |
| S3-T7 | `area_per_player(half, hs, 6 players)` | 350.0 (matches `01` §7) |
| S3-T8 | basket S1 of `gym-home`, local (0, 10) → gym frame | (−9.75, 18) |
| S3-T8b | basket A, local (5, 0) → gym frame | (5, 36.75) |
| S3-T9 | `circle r: "$closeout_dist"` with dial = 12 → contains (0, 11.9) | true |

## 10. Open questions

1. Do your gyms' side hoops have painted lanes? (Decides whether cross-court games can use `paint`.)
2. Is a three-dimensional "rim height" dial needed for 7th grade (lowered rims)? Proposed: a format field `rim_height_ft`, not geometry.
