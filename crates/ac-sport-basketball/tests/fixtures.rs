//! Every fixture in docs/fixtures against its `expect:` block (the outputs of
//! `tools/p0_check.py` are the oracle). Covers S2-T1…T6 rendering, S5-T1/T4/T6, S7-T1…T4,
//! the D-029 negative test and the practice template.

use std::path::{Path, PathBuf};

use ac_domain::{play, resolve, Game, Play, Practice, ResolveOptions, Rule, Severity};
use ac_sport_basketball::{render, BasketballPack};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/fixtures")
}

fn yaml_files(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for e in std::fs::read_dir(dir).unwrap() {
        let p = e.unwrap().path();
        if p.is_dir() {
            out.extend(yaml_files(&p));
        } else if p.extension().is_some_and(|x| x == "yaml") {
            out.push(p);
        }
    }
    out.sort();
    out
}

fn expect<'a>(g: &'a Game, key: &str) -> Option<&'a serde_yaml::Value> {
    g.extra.get("expect").and_then(|e| e.get(key))
}

fn strings(v: &serde_yaml::Value) -> Vec<String> {
    v.as_sequence()
        .unwrap()
        .iter()
        .map(|x| x.as_str().unwrap().to_string())
        .collect()
}

fn modifiers(g: &Game) -> Vec<Rule> {
    g.extra
        .get("block_example")
        .and_then(|b| b.get("modifiers"))
        .and_then(|m| m.as_sequence())
        .map(|seq| {
            seq.iter()
                .map(|m| serde_yaml::from_value(m.get("rule").unwrap().clone()).unwrap())
                .collect()
        })
        .unwrap_or_default()
}

#[test]
fn every_game_fixture_matches_its_golden_output() {
    let files = yaml_files(&root().join("games"));
    assert_eq!(files.len(), 13, "fixture count changed");
    for f in files {
        let text = std::fs::read_to_string(&f).unwrap();
        let g: Game = ac_domain::from_yaml(&text).unwrap();
        let opts = ResolveOptions {
            modifiers: modifiers(&g),
            ..Default::default()
        };
        let rg = resolve(&g, &BasketballPack, &opts);
        let errors: Vec<_> = rg
            .issues
            .iter()
            .filter(|i| i.severity == Severity::Error)
            .collect();
        assert!(errors.is_empty(), "{}: {errors:#?}", g.id);

        let card = render::card_lines(&rg);
        if let Some(want) = expect(&g, "card").or(expect(&g, "card_block")) {
            assert_eq!(card, strings(want), "{}: card", g.id);
        }
        if let Some(want) = expect(&g, "board") {
            assert_eq!(
                render::board_line(&rg),
                want.as_str().unwrap(),
                "{}: board",
                g.id
            );
        }
        if let Some(want) = expect(&g, "buttons_per_team") {
            assert_eq!(render::buttons(&rg), strings(want), "{}: buttons", g.id);
        }
        if let Some(want) = expect(&g, "calls") {
            assert_eq!(render::calls(&rg), strings(want), "{}: calls", g.id);
        }
    }
}

/// S5-T6 / D-018: YAML → domain → YAML → domain is equal, for every fixture kind.
#[test]
fn every_fixture_round_trips() {
    for f in yaml_files(&root()) {
        let text = std::fs::read_to_string(&f).unwrap();
        let dir = f.parent().unwrap().to_string_lossy().to_string();
        if dir.contains("games") {
            let a: Game = ac_domain::from_yaml(&text).unwrap();
            let b: Game = ac_domain::from_yaml(&ac_domain::to_yaml(&a).unwrap()).unwrap();
            assert_eq!(a, b, "{f:?}");
        } else if dir.contains("plays") {
            let a: Play = ac_domain::from_yaml(&text).unwrap();
            let b: Play = ac_domain::from_yaml(&ac_domain::to_yaml(&a).unwrap()).unwrap();
            assert_eq!(a, b, "{f:?}");
        } else if dir.contains("practices") {
            let a: Practice = ac_domain::from_yaml(&text).unwrap();
            let b: Practice = ac_domain::from_yaml(&ac_domain::to_yaml(&a).unwrap()).unwrap();
            assert_eq!(a, b, "{f:?}");
        }
    }
}

fn closeout() -> Game {
    ac_domain::from_yaml(
        &std::fs::read_to_string(root().join("games/g-closeout-attack-3v3.yaml")).unwrap(),
    )
    .unwrap()
}

/// S5-T1: default resolution; card line with the bound dial.
#[test]
fn s5_t1_closeout_resolves_with_defaults() {
    let rg = resolve(&closeout(), &BasketballPack, &ResolveOptions::default());
    let ids: Vec<&str> = rg.rules.iter().map(|r| r.rule.id.as_str()).collect();
    assert_eq!(
        ids,
        [
            "base.two",
            "base.three",
            "kick-out-three",
            "over-limit",
            "corner-catch",
            "off-reb"
        ]
    );
    assert!(render::card_lines(&rg).contains(&"More than 2 dribbles = turnover".to_string()));
}

/// S5-T4: rollback to constraint 1.
#[test]
fn s5_t4_rollback() {
    let opts = ResolveOptions {
        rollback: Some(1),
        ..Default::default()
    };
    let rg = resolve(&closeout(), &BasketballPack, &opts);
    let ids: Vec<&str> = rg.rules.iter().map(|r| r.rule.id.as_str()).collect();
    assert_eq!(ids, ["base.two", "base.three", "kick-out-three"]);
}

/// Suppressing a constraint drops its rules; a dial override changes the card.
#[test]
fn suppress_and_dial_override() {
    let mut g = closeout();
    g.suppressed.push("corner-zones".into());
    let mut opts = ResolveOptions::default();
    opts.dials
        .insert("dribble-limit.limit".into(), ac_domain::Val::Int(1));
    let rg = resolve(&g, &BasketballPack, &opts);
    assert!(rg.rule("corner-catch").is_none());
    assert!(render::card_lines(&rg).contains(&"More than 1 dribble = turnover".to_string()));
}

/// S5-T5: an int dial in a distance slot is a validator error.
#[test]
fn s5_t5_dial_type_mismatch() {
    let mut g = closeout();
    g.constraints[1].zones.push(ac_domain::Zone {
        id: "ring".into(),
        kind: "start".into(),
        shape: ac_domain::Shape {
            circle: Some(ac_domain::Circle {
                center: [0.into(), 0.into()],
                r: "$limit".into(),
            }),
            ..Default::default()
        },
        ..Default::default()
    });
    let rg = resolve(&g, &BasketballPack, &ResolveOptions::default());
    assert!(
        rg.issues
            .iter()
            .any(|i| i.code == "S5 §4" && i.message.contains("int dial")),
        "{:?}",
        rg.issues
    );
}

/// D-029: adding a possession-scoped step to the two-ball game fails validation.
#[test]
fn d029_two_balls_reject_possession_rules() {
    let mut g: Game = ac_domain::from_yaml(
        &std::fs::read_to_string(root().join("games/stress/s11-two-balls.yaml")).unwrap(),
    )
    .unwrap();
    g.constraints[0].rules[0]
        .after
        .push(ac_domain::Trigger::event("pass"));
    let rg = resolve(&g, &BasketballPack, &ResolveOptions::default());
    assert!(
        rg.issues.iter().any(|i| i.code == "D-029"),
        "{:?}",
        rg.issues
    );
}

/// R2/R1/R7 fire in coaching words.
#[test]
fn validator_speaks_to_the_coach() {
    let mut g = closeout();
    g.stage.clear();
    g.constraints[0].invites.clear();
    g.format.roles.retain(|r| r.team != "defense");
    let rg = resolve(&g, &BasketballPack, &ResolveOptions::default());
    let codes: Vec<&str> = rg.issues.iter().map(|i| i.code).collect();
    for c in ["R7", "R2", "R1"] {
        assert!(codes.contains(&c), "{c} missing in {codes:?}");
    }
}

fn load_play(name: &str) -> Play {
    ac_domain::from_yaml(&std::fs::read_to_string(root().join("plays").join(name)).unwrap())
        .unwrap()
}

/// S7-T1, T2: decompose + R9.
#[test]
fn s7_horns_reads_and_untrained() {
    let p = load_play("pl-horns-elbow.yaml");
    assert_eq!(
        play::reads(&p),
        [
            "read.ballscreen.drop",
            "read.ballscreen.hedge",
            "read.ballscreen.switch",
            "read.ballscreen.help-on-roll"
        ]
    );
    assert_eq!(
        play::untrained(&p, &|_| vec![]),
        ["read.ballscreen.help-on-roll"]
    );
    assert!(!play::has_loop(&p));
    // S7-T4: path s0 → t1 → t-drop = 3 actions × 1.2 s
    let path = play::path(&p, &|s| (s == "s1").then(|| "t-drop".to_string()));
    assert_eq!(path, ["t1", "t-drop"]);
    assert!((play::path_seconds(&p, &path) - 3.6).abs() < 1e-9);
}

/// S7-T3: the 5-out concept loops.
#[test]
fn s7_concept_loops() {
    let p = load_play("pl-5out-drive-kick.yaml");
    assert_eq!(p.kind, "concept");
    assert!(play::has_loop(&p));
    assert!(play::lint(&p).is_empty(), "{:?}", play::lint(&p));
    assert!(play::untrained(&p, &|_| vec![]).is_empty());
}

#[test]
fn generate_game_from_read_is_valid() {
    let p = load_play("pl-horns-elbow.yaml");
    let g = ac_sport_basketball::templates::game_from_read(
        &p,
        "t-help",
        "read.ballscreen.help-on-roll",
        "g-gen",
    );
    let rg = resolve(&g, &BasketballPack, &ResolveOptions::default());
    assert!(rg.errors().next().is_none(), "{:?}", rg.issues);
    assert_eq!(g.reads, ["read.ballscreen.help-on-roll"]);
    assert!(rg.roles.iter().any(|r| r.id == "O3") && rg.roles.iter().any(|r| r.id == "D3"));
}

#[test]
fn practice_template_loads() {
    let p: Practice = ac_domain::from_yaml(
        &std::fs::read_to_string(root().join("practices/practice-template.yaml")).unwrap(),
    )
    .unwrap();
    assert_eq!(p.blocks.len(), 3);
    assert_eq!(p.gyms[0].baskets.len(), 4);
}
