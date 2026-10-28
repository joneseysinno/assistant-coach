//! First-run library: the P0 fixture games and plays, the practice template, and the
//! free-throw ladder the template refers to.

use ac_domain::store::{Doc, Store};
use ac_domain::{Game, Play, Practice};

macro_rules! fixture {
    ($p:literal) => {
        include_str!(concat!("../../../docs/fixtures/", $p))
    };
}

const GAMES: &[&str] = &[
    fixture!("games/g-closeout-attack-3v3.yaml"),
    fixture!("games/stress/s01-delayed-defender.yaml"),
    fixture!("games/stress/s02-continuous-advantage.yaml"),
    fixture!("games/stress/s03-coach-calls-coverage.yaml"),
    fixture!("games/stress/s04-losers-pick-rule.yaml"),
    fixture!("games/stress/s05-shot-clock-scoring.yaml"),
    fixture!("games/stress/s06-king-of-the-court.yaml"),
    fixture!("games/stress/s07-neutral-joker.yaml"),
    fixture!("games/stress/s08-cross-court-2v2.yaml"),
    fixture!("games/stress/s09-player-handicap.yaml"),
    fixture!("games/stress/s10-defense-scores.yaml"),
    fixture!("games/stress/s11-two-balls.yaml"),
    fixture!("games/stress/s12-zone-restricted-defender.yaml"),
];

const PLAYS: &[&str] = &[
    fixture!("plays/pl-horns-elbow.yaml"),
    fixture!("plays/pl-5out-drive-kick.yaml"),
];

const PRACTICE: &str = fixture!("practices/practice-template.yaml");

const FT_LADDER: &str = r#"
id: g-ft-ladder
name: Free-throw ladder
stage: coordination
format:
  numbers: "1"
  space: { court: half }
  roles: [ { id: O1, team: offense } ]
  start: { type: dribble-at, from: paint }
  restart: { on_score: rotate, on_stop: rotate }
  rotation: { type: none }
  end: { type: block }
constraints:
  - id: ladder
    name: Make it, move up
    category: task
    invites: [routine-under-pressure]
    preserves: [game-fatigue]
    targets: [off.finish.shot-selection]
    rules:
      - { id: ft-make, label: Make, when: { event: make, value: ft }, then: [ { points: 1 } ], judge: coach, say: "Each free throw made = 1" }
notes: "No live defense on purpose: free throws at the end of practice, when tired (R1 coordination)."
"#;

/// Strip the P0 test-only keys so the Studio copies are clean documents.
fn clean_game(mut g: Game) -> Game {
    for k in ["expect", "stress", "block_example"] {
        g.extra.shift_remove(k);
    }
    g
}

pub fn seed(store: &dyn Store) {
    for y in GAMES.iter().chain(std::iter::once(&FT_LADDER)) {
        match ac_domain::from_yaml::<Game>(y) {
            Ok(g) => {
                let _ = store.save(&Doc::Game(clean_game(g)));
            }
            Err(e) => eprintln!("seed game: {e}"),
        }
    }
    for y in PLAYS {
        if let Ok(mut p) = ac_domain::from_yaml::<Play>(y) {
            p.extra.shift_remove("expect");
            let _ = store.save(&Doc::Play(p));
        }
    }
    if let Ok(mut p) = ac_domain::from_yaml::<Practice>(PRACTICE) {
        p.name = "Practice template".into();
        p.players = Some(12);
        let _ = store.save(&Doc::Practice(p));
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn seed_fills_a_store() {
        let s = ac_domain::MemStore::default();
        super::seed(&s);
        use ac_domain::store::Store;
        assert_eq!(s.list(None).unwrap().len(), 17);
    }
}
