//! Plain-English rendering (S2 §7), Board short form, Remote buttons (S12 §2) and the
//! printable game card. A port of `tools/p0_check.py` — that script is the oracle the
//! fixtures' `expect:` blocks were written against, and CI runs both.

use ac_domain::{Consequence, Game, ResolvedGame, ResolvedRule, Rule, TriggerKind};

use crate::vocab::{self, label_of};

fn noun(ev: &str) -> String {
    match ev {
        "make" => "Basket",
        "off-reb" => "Offensive rebound",
        "stop" => "Stop",
        "deflection" => "Deflection",
        "charge" => "Charge",
        "paint-touch" => "paint touch",
        "catch" => "catch",
        "drive" => "drive",
        "dribble" => "dribble",
        "pass" => "pass",
        "shot" => "shot",
        other => return vocab::event_label(other).to_string(),
    }
    .to_string()
}

fn val(v: &Option<ac_domain::Val>) -> String {
    v.as_ref().map(|v| v.to_string()).unwrap_or_default()
}

/// One rule as a sentence for the card, or `None` when it isn't shown (clock rules, tallies).
pub fn sentence(rule: &Rule) -> Option<String> {
    if let Some(s) = &rule.say {
        return Some(s.clone());
    }
    let w = &rule.when;
    let then = &rule.then;
    let turnover = then.iter().any(Consequence::is_turnover);
    let pts = then.iter().find(|t| t.points.is_some());
    match w.kind() {
        TriggerKind::Clock => return None,
        TriggerKind::Count => {
            let n = val(&w.value);
            let ev = w.count.clone().unwrap_or_default();
            return if turnover {
                let plural = if n == "1" { "" } else { "s" };
                Some(if w.op.as_deref() == Some(">") {
                    format!("More than {n} {ev}{plural} = turnover")
                } else {
                    format!("{ev} limit = turnover")
                })
            } else {
                None
            };
        }
        TriggerKind::Event => {}
        _ => return None,
    }
    let ev = w.event.clone().unwrap_or_default();
    let value = w.value.as_ref().and_then(|v| v.as_f64());
    let mut subj = match ev.as_str() {
        "make" => {
            let mut s = if value == Some(3.0) {
                "Three"
            } else {
                "Basket"
            }
            .to_string();
            if let Some(ac_domain::OneOrMany::One(z)) = &w.zone {
                if z == "paint" && value == Some(2.0) {
                    s = "Layup in the paint".into();
                }
            }
            s
        }
        "catch" if w.zone.is_some() => {
            let zones = w.zone.as_ref().unwrap().items();
            if zones.iter().all(|z| z.starts_with("corner")) {
                "Catch in a corner zone".to_string()
            } else {
                format!(
                    "Catch in the {}",
                    vocab::region_label(&zones[0]).to_lowercase()
                )
            }
        }
        other => noun(other),
    };
    if let Some(step) = rule.after.first() {
        let step_noun = noun(step.event.as_deref().unwrap_or(""));
        if rule.within.is_some() {
            subj = format!(
                "Score within {} seconds of the {step_noun}",
                val(&rule.within)
            );
        } else {
            let art = if step_noun
                .chars()
                .next()
                .is_some_and(|c| "aeiou".contains(c))
            {
                "an"
            } else {
                "a"
            };
            subj = format!("{subj} after {art} {step_noun}");
        }
    }
    if let Some(p) = pts {
        let n = val(&p.points);
        if p.to.as_deref() == Some("defense") {
            return Some(format!("{subj} = +{n} for the defense"));
        }
        if subj == "Layup in the paint" {
            return Some(format!("{subj} = +{n}"));
        }
        return Some(format!("{subj} = {n}"));
    }
    if turnover {
        return Some(format!("{subj} = turnover"));
    }
    None
}

/// "How the game runs" lines the format adds to the card (neutral roles, make-it-take-it).
pub fn format_lines(rg: &ResolvedGame) -> Vec<String> {
    let mut lines = Vec::new();
    for r in &rg.format.roles {
        if r.team == "neutral" {
            lines.push(format!(
                "The joker plays for the {}",
                r.plays_for.as_deref().unwrap_or("offense")
            ));
        }
    }
    if rg
        .format
        .restart
        .as_ref()
        .and_then(|r| r.on_score.as_deref())
        == Some("keep")
    {
        lines.push("Make it, take it".into());
    }
    lines
}

/// The card lines (S2 §7), in the order `p0_check.py` produces them.
pub fn card_lines(rg: &ResolvedGame) -> Vec<String> {
    let mut card: Vec<String> = rg.rules.iter().filter_map(|r| sentence(&r.rule)).collect();
    if let Some(sc) = &rg.format.shot_clock {
        card.push(format!("Shot clock: {} seconds", sc.seconds));
    }
    let nb = rg
        .rules
        .iter()
        .filter(|r| r.is_base())
        .count()
        .min(card.len());
    let fl = format_lines(rg);
    card.splice(nb..nb, fl);
    card
}

fn scores(r: &ResolvedRule) -> bool {
    r.rule.awards_points()
}

/// S12 §2: one button per coach-judged scoring rule; base rules first, then stack order.
pub fn buttons(rg: &ResolvedGame) -> Vec<String> {
    let pts: Vec<&ResolvedRule> = rg
        .rules
        .iter()
        .filter(|r| r.judge == ac_domain::Judge::Coach && scores(r))
        .collect();
    pts.iter()
        .filter(|r| r.is_base())
        .chain(pts.iter().filter(|r| !r.is_base()))
        .map(|r| r.rule.display_label())
        .collect()
}

/// Calls tray: rules that don't score but end a possession or adjust points.
pub fn calls(rg: &ResolvedGame) -> Vec<String> {
    rg.rules
        .iter()
        .filter(|r| {
            !scores(r)
                && matches!(r.judge, ac_domain::Judge::Coach | ac_domain::Judge::Players)
                && r.rule
                    .then
                    .iter()
                    .any(|t| t.is_turnover() || t.points.is_some())
        })
        .map(|r| r.rule.display_label())
        .collect()
}

/// Board short form: the first three constraint rules in stack order (S2 §7).
pub fn board_line(rg: &ResolvedGame) -> String {
    let mut out = Vec::new();
    for r in rg.rules.iter().filter(|r| !r.is_base() && !r.is_modifier()) {
        let then = &r.rule.then;
        if !then.is_empty() && then.iter().all(Consequence::is_tally) {
            continue;
        }
        if let Some(b) = &r.rule.board {
            out.push(b.clone());
        } else if let Some(p) = then.iter().find(|t| t.awards_points()) {
            out.push(format!("{} = {}", r.rule.display_label(), val(&p.points)));
        } else if r.rule.when.count.is_some() {
            let n = val(&r.rule.when.value);
            out.push(format!(
                "{n} {}{} max",
                r.rule.when.count.clone().unwrap_or_default(),
                if n == "1" { "" } else { "s" }
            ));
        } else {
            out.push(r.rule.display_label());
        }
        if out.len() == 3 {
            break;
        }
    }
    out.join(" · ")
}

/// How a possession starts (S4 §3 card text).
pub fn start_text(rg: &ResolvedGame) -> Option<String> {
    let s = rg.format.start.as_ref()?;
    let f = |k: &str| s.field(k);
    Some(match s.kind.as_str() {
        "check-ball" => format!(
            "Check ball at the {}",
            f("spot").unwrap_or_else(|| "top".into())
        ),
        "coach-pass" => match f("to") {
            Some(to) => format!("Coach passes to {to}"),
            None => "Coach passes in".into(),
        },
        "closeout" => "Defense passes out and closes out".into(),
        "rebound" => "Coach shoots; play the rebound".into(),
        "transition" => "Outlet and go".into(),
        "inbound" => format!(
            "{} out of bounds",
            f("spot")
                .map(|s| {
                    let mut c = s.chars();
                    c.next()
                        .map(|x| x.to_uppercase().collect::<String>() + c.as_str())
                        .unwrap_or_default()
                })
                .unwrap_or_else(|| "Baseline".into())
        ),
        "dribble-at" => format!(
            "Attack from the {}",
            f("from")
                .map(|x| vocab::region_label(&x).replace('-', " ").to_lowercase())
                .unwrap_or_else(|| "top".into())
        ),
        other => label_of(vocab::START_TYPES, other).to_string(),
    })
}

pub fn end_text(rg: &ResolvedGame) -> Option<String> {
    let e = rg.format.end.as_ref()?;
    Some(match e.kind.as_str() {
        "score" => {
            let mut s = format!("First to {}", val(&e.target));
            if e.win_by.as_ref().and_then(|v| v.as_f64()).unwrap_or(1.0) > 1.0 {
                s.push_str(&format!(", win by {}", val(&e.win_by)));
            }
            s
        }
        "time" => format!("{}-second games", val(&e.seconds)),
        "possessions" => format!("{} possessions each", val(&e.count)),
        _ => "Play until the horn".into(),
    })
}

pub fn restart_text(rg: &ResolvedGame) -> Option<String> {
    let r = rg.format.restart.as_ref()?;
    let on = |x: &Option<String>| {
        x.as_deref()
            .map(|s| label_of(vocab::RESTARTS, s).to_lowercase())
    };
    match (on(&r.on_score), on(&r.on_stop)) {
        (Some(a), Some(b)) if a == b => Some(format!("After a score or a stop: {a}")),
        (Some(a), Some(b)) => Some(format!("After a score: {a}. After a stop: {b}")),
        (Some(a), None) => Some(format!("After a score: {a}")),
        (None, Some(b)) => Some(format!("After a stop: {b}")),
        _ => None,
    }
}

/// The one-page game card (03 §3): written for players, generated from the rules (R5).
#[derive(Clone, Debug, PartialEq)]
pub struct GameCard {
    pub title: String,
    pub subtitle: String,
    pub how_to_score: Vec<String>,
    pub rules: Vec<String>,
    pub how_it_runs: Vec<String>,
    pub questions: Vec<String>,
    pub board: String,
}

pub fn game_card(game: &Game, rg: &ResolvedGame) -> GameCard {
    let mut how_to_score = Vec::new();
    let mut rules = Vec::new();
    for r in &rg.rules {
        let Some(line) = sentence(&r.rule) else {
            continue;
        };
        if r.rule.awards_points() {
            how_to_score.push(line);
        } else {
            rules.push(line);
        }
    }
    if let Some(sc) = &rg.format.shot_clock {
        rules.push(format!("Shot clock: {} seconds", sc.seconds));
    }
    rules.extend(format_lines(rg));
    let mut how_it_runs = Vec::new();
    how_it_runs.extend(start_text(rg));
    how_it_runs.extend(restart_text(rg));
    if let Some(rot) = &rg.format.rotation {
        if rot.kind != "none" && !rot.kind.is_empty() {
            how_it_runs.push(label_of(vocab::ROTATIONS, &rot.kind).to_string());
        }
    }
    how_it_runs.extend(end_text(rg));
    let mut questions: Vec<String> = game.questions.clone();
    for c in &game.constraints {
        if rg.active.contains(&c.id) {
            questions.extend(c.cues.iter().cloned());
        }
    }
    questions.truncate(3);
    let court = vocab::region_label(&rg.format.space.court).to_string();
    GameCard {
        title: game.name.clone(),
        subtitle: format!("{} · {}", game.numbers(), court),
        how_to_score,
        rules,
        how_it_runs,
        questions,
        board: board_line(rg),
    }
}
