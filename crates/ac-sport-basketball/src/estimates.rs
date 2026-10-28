//! Predicted effects (S9, R15, D-007). Every estimate carries its formula and confidence.
//! Coefficients are the spec's starting guesses, to be calibrated from practice logs.

use ac_domain::{Consequence, ResolvedGame};

use crate::court::Court;

#[derive(Clone, Debug, PartialEq)]
pub struct Estimate {
    pub label: &'static str,
    pub value: f64,
    pub unit: &'static str,
    pub formula: String,
    pub confidence: &'static str,
    /// Band / note in coaching words ("typical", "tight"…).
    pub note: String,
}

/// Half-up rounding, as S9 states.
pub fn round_half_up(x: f64) -> f64 {
    (x + 0.5).floor()
}

pub fn per_side(rg: &ResolvedGame) -> usize {
    let o = rg.roles.iter().filter(|r| r.team == "offense").count();
    let d = rg.roles.iter().filter(|r| r.team == "defense").count();
    o.max(d).max(1)
}

pub fn area_per_player(court: &Court, rg: &ResolvedGame) -> Estimate {
    let region = rg.format.space.court.as_str();
    let area = court.region_area(region);
    let n = rg.format.on_court().max(1);
    let v = area / n as f64;
    let note = if v < 250.0 {
        "tight"
    } else if v <= 450.0 {
        "typical"
    } else {
        "open"
    };
    Estimate {
        label: "Space per player",
        value: v.round(),
        unit: "ft²",
        formula: format!("{area:.0} ft² ({region}, {}) ÷ {n} players", court.id),
        confidence: "high",
        note: note.into(),
    }
}

/// Intensity band 1–5 (S9 §3). `work_s` = continuous work bout, if known.
pub fn intensity(court: &Court, rg: &ResolvedGame, work_s: Option<f64>) -> Estimate {
    let n = per_side(rg) as f64;
    let b = (5.0 - 0.5 * n).clamp(2.5, 4.5);
    let app = area_per_player(court, rg).value;
    let a = if app < 250.0 {
        -0.25
    } else if app <= 450.0 {
        0.0
    } else {
        0.5
    };
    let w = match work_s {
        Some(s) if s > 240.0 => 0.5,
        Some(s) if s <= 150.0 => -0.5,
        _ => 0.0,
    };
    let mut c = 0.0;
    let mut cparts = Vec::new();
    if let Some(sc) = rg
        .format
        .shot_clock
        .as_ref()
        .and_then(|s| s.seconds.as_f64())
    {
        if sc <= 8.0 {
            c += 0.25;
            cparts.push("short shot clock +0.25");
        }
    }
    if rg.format.phases.is_some() && rg.format.space.court == "full" {
        c += 0.5;
        cparts.push("full-court phases +0.5");
    }
    if rg.stage == "coordination" && !rg.roles.iter().any(|r| r.team == "defense") {
        c -= 1.0;
        cparts.push("no live defense −1");
    }
    let raw = b + a + w + c;
    let band = round_half_up(raw).clamp(1.0, 5.0);
    let names = ["", "light", "moderate", "hard", "very hard", "max"];
    Estimate {
        label: "Intensity",
        value: band,
        unit: "/5",
        formula: format!(
            "round½↑({b} for {n}v{n} {a:+} space {w:+} work {c:+} rules) = {band}{}",
            if cparts.is_empty() {
                String::new()
            } else {
                format!(" ({})", cparts.join(", "))
            }
        ),
        confidence: "low",
        note: names[band as usize].into(),
    }
}

/// Decisions/touches per player per minute (S9 §4).
pub fn involvement(rg: &ResolvedGame) -> Estimate {
    let n = rg.format.on_court().max(1) as f64;
    let mut m = 1.0;
    let mut parts: Vec<String> = Vec::new();
    let limit = rg
        .rules
        .iter()
        .filter(|r| r.rule.when.count.as_deref() == Some("dribble"))
        .filter_map(|r| r.rule.when.value.as_ref().and_then(|v| v.as_f64()))
        .fold(f64::INFINITY, f64::min);
    if limit <= 1.0 {
        m *= 1.35;
        parts.push("dribble limit ≤1 ×1.35".into());
    } else if limit <= 2.0 {
        m *= 1.2;
        parts.push("dribble limit ≤2 ×1.2".into());
    }
    if rg.rules.iter().any(|r| {
        r.rule
            .after
            .iter()
            .any(|s| s.event.as_deref() == Some("paint-touch"))
    }) {
        m *= 1.1;
        parts.push("paint before three ×1.1".into());
    }
    if rg.roles.iter().any(|r| r.team == "neutral") {
        m *= 1.1;
        parts.push("neutral player ×1.1".into());
    }
    if rg
        .format
        .shot_clock
        .as_ref()
        .and_then(|s| s.seconds.as_f64())
        .is_some_and(|s| s <= 8.0)
    {
        m *= 1.15;
        parts.push("shot clock ≤8 ×1.15".into());
    }
    let v = 12.0 / n * m;
    Estimate {
        label: "Involvement",
        value: (v * 10.0).round() / 10.0,
        unit: "/player/min",
        formula: format!(
            "12 ÷ {n} on court{}",
            if parts.is_empty() {
                String::new()
            } else {
                format!(" × {}", parts.join(" × "))
            }
        ),
        confidence: "low",
        note: String::new(),
    }
}

/// RPE guess for an intensity band (S9 §6).
pub fn rpe(band: f64) -> f64 {
    match band as i64 {
        1 => 3.0,
        2 => 4.0,
        3 => 5.0,
        4 => 6.5,
        _ => 8.0,
    }
}

pub fn load_units(active_minutes: f64, band: f64) -> f64 {
    active_minutes * rpe(band)
}

/// Representativeness checklist (03 §3): live defense, real basket, time pressure, help.
pub fn representativeness(rg: &ResolvedGame) -> Vec<(&'static str, bool)> {
    let d = rg.roles.iter().filter(|r| r.team == "defense").count();
    vec![
        ("Live defense", d > 0),
        ("Help defense present", d >= 2),
        (
            "Time pressure",
            rg.format.shot_clock.is_some()
                || rg.rules.iter().any(|r| r.rule.within.is_some())
                || rg.rules.iter().any(|r| r.rule.when.clock.is_some()),
        ),
        (
            "Score is kept",
            rg.rules
                .iter()
                .any(|r| r.rule.then.iter().any(Consequence::awards_points)),
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::court::preset;
    use crate::BasketballPack;
    use ac_domain::{resolve, Game, ResolveOptions};

    fn game(yaml: &str) -> ResolvedGame {
        let g: Game = ac_domain::from_yaml(yaml).unwrap();
        resolve(&g, &BasketballPack, &ResolveOptions::default())
    }

    const THREES: &str = "id: x\nstage: team-based\nformat:\n  space: { court: half }\n  roles: [ {id: O1, team: offense}, {id: O2, team: offense}, {id: O3, team: offense}, {id: D1, team: defense}, {id: D2, team: defense}, {id: D3, team: defense} ]\n";

    #[test]
    fn s9_t1_three_on_three_half() {
        let rg = game(THREES);
        assert_eq!(area_per_player(&preset("hs"), &rg).value, 350.0);
        assert_eq!(intensity(&preset("hs"), &rg, None).value, 4.0);
    }

    #[test]
    fn s9_t2_five_on_five_full_continuous() {
        let rg = game("id: x\nstage: team-based\nformat:\n  space: { court: full }\n  roles: [ {id: O1, team: offense}, {id: O2, team: offense}, {id: O3, team: offense}, {id: O4, team: offense}, {id: O5, team: offense}, {id: D1, team: defense}, {id: D2, team: defense}, {id: D3, team: defense}, {id: D4, team: defense}, {id: D5, team: defense} ]\n");
        assert_eq!(area_per_player(&preset("hs"), &rg).value, 420.0);
        assert_eq!(intensity(&preset("hs"), &rg, Some(480.0)).value, 3.0);
    }

    #[test]
    fn s9_t3_two_on_two_cross_court() {
        let rg = game("id: x\nstage: team-based\nformat:\n  space: { court: cross-court }\n  roles: [ {id: O1, team: offense}, {id: O2, team: offense}, {id: D1, team: defense}, {id: D2, team: defense} ]\n");
        assert_eq!(area_per_player(&preset("hs"), &rg).value, 525.0);
        assert_eq!(intensity(&preset("hs"), &rg, None).value, 5.0);
    }

    #[test]
    fn s9_t4_involvement_with_dribble_limit() {
        let y = format!("{THREES}dials: {{}}\nconstraints:\n  - id: c\n    rules:\n      - {{ id: r, when: {{ count: dribble, by: player, op: '>', value: 2 }}, then: [ {{ turnover: true }} ], judge: coach }}\n");
        assert_eq!(involvement(&game(&y)).value, 2.4);
    }

    #[test]
    fn s9_t5_load() {
        assert_eq!(load_units(20.0, 4.0), 130.0);
    }
}
