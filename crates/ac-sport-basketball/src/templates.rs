//! Starting points for new documents, the starter constraint library, and
//! generate-game-from-read (02 §6, S7 §3).

use ac_domain::{
    Basket, Constraint, Dial, End, Format, Game, Gym, Play, PlayState, Practice, Restart, Role,
    Rotation, SpaceRef, Typed, Val,
};
use indexmap::IndexMap;

use crate::vocab;

pub fn default_gym() -> Gym {
    let b = |id: &str, label: &str, x: f64, y: f64, f: f64| Basket {
        id: id.into(),
        label: label.into(),
        position: [x, y],
        facing: f,
        ..Default::default()
    };
    Gym {
        id: "gym-home".into(),
        name: "Home gym".into(),
        court: "hs".into(),
        baskets: vec![
            b("A", "Main A", 0.0, 36.75, 180.0),
            b("B", "Main B", 0.0, -36.75, 0.0),
            b("S1", "Side 1", -19.75, 18.0, 90.0),
            b("S2", "Side 2", 19.75, 18.0, 270.0),
        ],
        ..Default::default()
    }
}

/// Roles for an `o`v`d` (+ neutral) format.
pub fn roles(o: usize, d: usize, neutral: usize) -> Vec<Role> {
    let mut v: Vec<Role> = (1..=o)
        .map(|i| Role::new(&format!("O{i}"), "offense"))
        .collect();
    v.extend((1..=d).map(|i| Role::new(&format!("D{i}"), "defense")));
    for i in 0..neutral {
        let mut r = Role::new(if i == 0 { "J" } else { "J2" }, "neutral");
        r.plays_for = Some("offense".into());
        v.push(r);
    }
    v
}

pub fn new_game(id: &str) -> Game {
    let mut start = Typed::new("check-ball");
    start.set_field("spot", Some("top"));
    let mut dials = IndexMap::new();
    let mut target = Dial::int(3, 11, 7);
    target.label = Some("Game to".into());
    dials.insert("target".into(), target);
    let mut base = IndexMap::new();
    base.insert("two".into(), Val::Int(1));
    base.insert("three".into(), Val::Int(1));
    Game {
        id: id.into(),
        name: "New game".into(),
        stage: "team-based".into(),
        format: Format {
            space: SpaceRef {
                court: "half".into(),
                ..Default::default()
            },
            roles: roles(3, 3, 0),
            start: Some(start),
            restart: Some(Restart {
                on_score: Some("switch".into()),
                on_stop: Some("switch".into()),
                ..Default::default()
            }),
            rotation: Some(Rotation {
                kind: "none".into(),
                ..Default::default()
            }),
            end: Some(End {
                kind: "score".into(),
                target: Some(Val::Text("$target".into())),
                ..Default::default()
            }),
            ..Default::default()
        },
        base_rules: base,
        dials,
        ..Default::default()
    }
}

/// Ready-made constraints a coach can drop into any game (each passes R2/R3).
const STARTERS: &str = r#"
- id: dribble-limit
  name: Dribble limit
  category: task
  invites: [quick-decisions, catch-and-attack]
  preserves: [defender-pressure, help-defense]
  targets: [off.extend.attack-closeout]
  dials:
    limit: { int: [0, 4], default: 2, harder: down, levels: [3, 2, 1], label: "Dribbles allowed" }
  rules:
    - id: over-limit
      label: Too many dribbles
      when: { count: dribble, by: player, op: ">", value: "$limit" }
      then: [ { turnover: true } ]
      judge: coach
  cues: ["Where is the help before you catch?"]
- id: kick-out-bonus
  name: Kick-out bonus
  category: task
  invites: [drive-and-kick, extra-pass]
  preserves: [help-defense, closeout]
  targets: [off.extend.drive-and-kick]
  rules:
    - id: kick-out-three
      label: Kick-out 3
      when: { event: make, value: 3 }
      after: [ { event: paint-touch } ]
      then: [ { points: 3 } ]
      overrides: [base.three]
      judge: coach
  cues: ["Who did the help leave?"]
- id: corner-zones
  name: Corner zones
  category: environment
  invites: [relocation, spacing]
  preserves: [help-defense]
  targets: [off.shape.relocation]
  zones:
    - { id: corner-L, kind: bonus, shape: { named: corner-L } }
    - { id: corner-R, kind: bonus, shape: { named: corner-R } }
  rules:
    - id: corner-catch
      label: Corner catch
      when: { event: catch, zone: [corner-L, corner-R] }
      after: [ { event: drive } ]
      then: [ { points: 1 } ]
      judge: coach
- id: hungry
  name: Hungry (offensive boards)
  category: character
  invites: [crash-the-glass]
  preserves: [box-out-contest]
  targets: [char.hungry, off.finish.off-reb]
  rules:
    - { id: off-reb, label: Off reb, when: { event: off-reb }, then: [ { points: 1 } ], judge: coach }
- id: stops-count
  name: Stops count
  category: character
  invites: [defensive-effort, help]
  preserves: [live-offense]
  targets: [char.effort, def.scramble.help]
  rules:
    - { id: stop, label: Stop, when: { event: stop }, then: [ { points: 1, to: defense } ], judge: coach }
- id: charges-count
  name: Take a charge
  category: character
  invites: [help-positioning, sacrifice]
  preserves: [live-drives]
  targets: [char.effort, def.scramble.help]
  rules:
    - { id: charge, label: Charge, when: { event: charge }, then: [ { points: 2, to: defense }, { turnover: true } ], judge: coach }
- id: no-dribble-up-top
  name: No dribbles above the break
  category: environment
  invites: [ball-movement, cutting]
  preserves: [closeouts, help-defense]
  targets: [off.extend.extra-pass]
  zones:
    - { id: no-drib-top, kind: no-dribble, shape: { named: above-break } }
  rules:
    - { id: drib-top, label: Dribble up top, when: { event: dribble, zone: no-drib-top }, then: [ { turnover: true } ], judge: coach }
- id: pass-before-shot
  name: One pass before a shot
  category: task
  invites: [two-player-game, cut]
  preserves: [on-ball-pressure, help]
  targets: [off.create.cut]
  rules:
    - { id: no-pass, label: No pass first, when: { event: shot }, if: [ { count: pass, by: team, op: "<", value: 1 } ], then: [ { turnover: true } ], judge: players, say: "Make one pass before you shoot" }
- id: quick-score
  name: Catch-and-go bonus
  category: task
  invites: [catch-and-attack, quick-decisions]
  preserves: [closeouts, help-defense]
  targets: [off.extend.attack-closeout]
  dials:
    window: { duration: [3, 10], default: 6, harder: down, levels: [8, 6, 4], label: "Quick window" }
  rules:
    - { id: quick, label: Quick score, when: { event: make }, after: [ { event: catch } ], within: "$window", then: [ { points: 2 } ], overrides: [base.two], judge: coach }
- id: extra-pass
  name: Extra-pass bonus
  category: character
  invites: [extra-pass, unselfish-play]
  preserves: [help-defense, closeouts]
  targets: [char.humble, off.extend.extra-pass]
  rules:
    - { id: extra-pass-score, label: Extra-pass score, when: { event: make }, after: [ { event: extra-pass } ], then: [ { points: 2 } ], overrides: [base.two], judge: coach }
- id: deflections
  name: Deflections count
  category: character
  invites: [active-hands, ball-pressure]
  preserves: [live-passing]
  targets: [char.effort, def.deny.gap]
  rules:
    - { id: deflection, label: Deflection, when: { event: deflection }, then: [ { points: 1, to: defense } ], judge: coach }
- id: talk-tally
  name: Talk (tally)
  category: character
  invites: [communication]
  preserves: [full-game-structure]
  targets: [char.attitude, def.scramble.talk]
  rules:
    - { id: talk, label: Talk, when: { event: talk }, then: [ { tally: true } ], judge: coach, say: "Coaches count talk" }
"#;

pub fn starter_constraints() -> Vec<Constraint> {
    serde_yaml::from_str(STARTERS).expect("starter constraints parse")
}

/// An empty constraint with one scoring rule, ready to edit.
pub fn blank_constraint(game: &Game) -> Constraint {
    let id = game.fresh_constraint_id("rule");
    let rid = game.fresh_rule_id(&format!("{id}-score"));
    Constraint {
        name: "New constraint".into(),
        category: "task".into(),
        rules: vec![ac_domain::Rule {
            id: rid,
            label: Some("Bonus".into()),
            when: ac_domain::Trigger::event("make"),
            then: vec![ac_domain::Consequence::points(2)],
            judge: Some(ac_domain::Judge::Coach),
            ..Default::default()
        }],
        id,
        ..Default::default()
    }
}

/// Re-key a constraint so its ids don't clash with the game it's added to.
pub fn adopt(game: &Game, c: &Constraint) -> Constraint {
    let mut c = c.clone();
    c.id = game.fresh_constraint_id(&c.id);
    let mut taken: Vec<String> = Vec::new();
    for r in &mut c.rules {
        let mut id = game.fresh_rule_id(&r.id);
        let mut n = 2;
        while taken.contains(&id) {
            id = format!("{}-{n}", r.id);
            n += 1;
        }
        taken.push(id.clone());
        r.id = id;
    }
    c
}

pub fn new_practice(id: &str, date: &str) -> Practice {
    Practice {
        id: id.into(),
        name: String::new(),
        date: date.into(),
        gym: "gym-home".into(),
        start: "15:30".into(),
        planned_minutes: 90.0,
        transition_s: 60.0,
        players: Some(12),
        gyms: vec![default_gym()],
        ..Default::default()
    }
}

/// 5-out alignment for a new play.
pub fn new_play(id: &str) -> Play {
    let mut at = IndexMap::new();
    for (r, p) in [
        ("O1", [0.0, 24.0]),
        ("O2", [-17.0, 15.0]),
        ("O3", [17.0, 15.0]),
        ("O4", [-21.0, 1.0]),
        ("O5", [21.0, 1.0]),
    ] {
        at.insert(r.to_string(), p);
    }
    Play {
        id: id.into(),
        name: "New play".into(),
        kind: "read-set".into(),
        court: "half".into(),
        roles: ["O1", "O2", "O3", "O4", "O5"].map(String::from).to_vec(),
        states: vec![PlayState {
            id: "s0".into(),
            label: "Alignment".into(),
            ball: Some("O1".into()),
            at,
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Generate a game from a read in a play (02 §6, S7 §3): the transition's participants
/// play live, with a defender each; the right option scores extra.
pub fn game_from_read(play: &Play, transition: &str, read_id: &str, id: &str) -> Game {
    let mut g = new_game(id);
    let def = vocab::read(read_id);
    let cue = def.map(|r| r.cue).unwrap_or(read_id);
    g.name = format!("Read: {cue}");
    g.stage = "complex".into();
    let mut offense = ac_domain::play::participants(play, transition);
    if let Some(t) = play.transitions.iter().find(|t| t.id == transition) {
        if let Some(s) = play.state(&t.from) {
            if let Some(b) = &s.ball {
                if !offense.contains(b) {
                    offense.insert(0, b.clone());
                }
            }
        }
    }
    if offense.is_empty() {
        offense = vec!["O1".into(), "O2".into()];
    }
    offense.truncate(4);
    let mut roles: Vec<Role> = offense.iter().map(|r| Role::new(r, "offense")).collect();
    roles.extend(
        offense
            .iter()
            .map(|r| Role::new(&r.replacen('O', "D", 1), "defense")),
    );
    g.format.roles = roles;
    g.format.numbers = None;
    g.reads = vec![read_id.to_string()];
    let source = def.map(|r| r.source).unwrap_or("defender");
    let right = def
        .and_then(|r| r.options.iter().find(|o| o.1))
        .map(|o| o.0)
        .unwrap_or("the right option");
    g.constraints.push(Constraint {
        id: "reward-the-read".into(),
        name: "Reward the read".into(),
        category: "task".into(),
        invites: vec![format!("read: {cue}")],
        preserves: vec![format!("live {source}")],
        targets: def.map(|r| vec![r.target.to_string()]).unwrap_or_default(),
        rules: vec![ac_domain::Rule {
            id: "right-read".into(),
            label: Some("Right read".into()),
            when: ac_domain::Trigger::event("make"),
            then: vec![ac_domain::Consequence::points(2)],
            overrides: vec!["base.two".into(), "base.three".into()],
            judge: Some(ac_domain::Judge::Coach),
            say: Some(format!(
                "Score off the right read ({}) = 2",
                right.to_lowercase()
            )),
            ..Default::default()
        }],
        cues: vec![format!("What did the {source} do?")],
        ..Default::default()
    });
    g.notes = format!(
        "Generated from play '{}' ({}). Live defense on the {source}: their choice is the cue.",
        play.name, transition
    );
    g
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::BasketballPack;
    use ac_domain::{resolve, ResolveOptions, Severity};

    #[test]
    fn starters_parse_and_validate_inside_a_new_game() {
        let mut g = new_game("g-test");
        for c in starter_constraints() {
            let c = adopt(&g, &c);
            g.constraints.push(c);
        }
        let rg = resolve(&g, &BasketballPack, &ResolveOptions::default());
        let errs: Vec<_> = rg
            .issues
            .iter()
            .filter(|i| i.severity == Severity::Error)
            .collect();
        assert!(errs.is_empty(), "{errs:#?}");
    }

    #[test]
    fn new_game_is_valid_and_renders() {
        let g = new_game("g-new");
        let rg = resolve(&g, &BasketballPack, &ResolveOptions::default());
        assert!(rg.errors().next().is_none(), "{:?}", rg.issues);
        assert_eq!(
            crate::render::card_lines(&rg),
            vec!["Basket = 1", "Three = 1"]
        );
        assert_eq!(crate::render::end_text(&rg).unwrap(), "First to 7");
    }
}
