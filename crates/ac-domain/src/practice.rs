//! Practice time accounting (R10) and active ratio (R11) — S8 §3–4.

use serde::Serialize;

use crate::model::Practice;

/// What the planner needs to know about the game a block runs.
#[derive(Clone, Debug, PartialEq)]
pub struct BlockGame {
    pub name: String,
    /// Players on the floor for one instance of the game.
    pub on_court: usize,
    /// Rotation type (S4 §5).
    pub rotation: String,
    /// Learning targets of the active constraints.
    pub targets: Vec<String>,
    pub stage: String,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct BlockMetrics {
    pub id: String,
    /// Clock offset from the practice start, minutes.
    pub starts_at: f64,
    pub minutes: f64,
    /// Players on the floor across the block's baskets.
    pub on_court: usize,
    /// on_court / players (≤ 1). `None` when the player count isn't known.
    pub active_ratio: Option<f64>,
    /// Active minutes for an average player.
    pub active_minutes: f64,
    pub game_missing: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PracticeMetrics {
    pub block_minutes: f64,
    pub transition_minutes: f64,
    pub planned_total: f64,
    /// planned_minutes − planned_total (negative = over).
    pub unassigned: f64,
    pub blocks: Vec<BlockMetrics>,
    /// Average active minutes per player across the practice.
    pub active_minutes_per_player: f64,
    /// Blocks whose game is below the target active ratio (default 0.6, S8 §4).
    pub low_active: Vec<String>,
}

pub const TARGET_ACTIVE_RATIO: f64 = 0.6;

pub fn metrics(p: &Practice, game: &dyn Fn(&str) -> Option<BlockGame>) -> PracticeMetrics {
    let n = p.blocks.len();
    let transition_minutes = if n > 1 {
        (n - 1) as f64 * p.transition_s / 60.0
    } else {
        0.0
    };
    // `+ 0.0` turns the empty sum's -0.0 into 0.0 (it would print as "-0")
    let block_minutes: f64 = p.blocks.iter().map(|b| b.minutes).sum::<f64>() + 0.0;
    let planned_total = block_minutes + transition_minutes;
    let players = p.player_count() as usize;
    let mut t = 0.0;
    let mut blocks = Vec::new();
    let mut active_sum = 0.0;
    let mut low = Vec::new();
    for b in &p.blocks {
        let g = game(&b.game);
        let instances = b.baskets.len().max(1);
        let on_court = g.as_ref().map(|g| g.on_court * instances).unwrap_or(0);
        let ratio = if players > 0 && on_court > 0 {
            Some((on_court as f64 / players as f64).min(1.0))
        } else {
            None
        };
        let active = b.minutes * ratio.unwrap_or(1.0);
        active_sum += active;
        if ratio.is_some_and(|r| r < TARGET_ACTIVE_RATIO) {
            low.push(b.id.clone());
        }
        blocks.push(BlockMetrics {
            id: b.id.clone(),
            starts_at: t,
            minutes: b.minutes,
            on_court,
            active_ratio: ratio,
            active_minutes: active,
            game_missing: g.is_none(),
        });
        t += b.minutes + p.transition_s / 60.0;
    }
    PracticeMetrics {
        block_minutes,
        transition_minutes,
        planned_total,
        unassigned: p.planned_minutes - planned_total,
        blocks,
        active_minutes_per_player: active_sum,
        low_active: low,
    }
}

/// Roll a target id up to its principle (two levels: `off.extend.drive-and-kick` → `off.extend`).
pub fn principle(target: &str) -> String {
    target.split('.').take(2).collect::<Vec<_>>().join(".")
}

/// R12: warn when the same principle fills `run` or more consecutive blocks (unless the
/// practice is performance stage). Returns (principle, first block id, length).
pub fn interleaving_runs(
    p: &Practice,
    game: &dyn Fn(&str) -> Option<BlockGame>,
    run: usize,
) -> Vec<(String, String, usize)> {
    let principles: Vec<Vec<String>> = p
        .blocks
        .iter()
        .map(|b| {
            let mut v: Vec<String> = game(&b.game)
                .map(|g| g.targets.iter().map(|t| principle(t)).collect())
                .unwrap_or_default();
            v.dedup();
            v
        })
        .collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < principles.len() {
        for pr in &principles[i] {
            let len = principles[i..]
                .iter()
                .take_while(|ps| ps.contains(pr))
                .count();
            if len >= run
                && !out
                    .iter()
                    .any(|(x, _, _): &(String, String, usize)| x == pr)
            {
                out.push((pr.clone(), p.blocks[i].id.clone(), len));
            }
        }
        i += 1;
    }
    out
}

/// "15:30" + minutes → "15:42".
pub fn clock_time(start: &str, offset_min: f64) -> String {
    let mut parts = start.split(':');
    let (Some(h), Some(m)) = (parts.next(), parts.next()) else {
        return format!("+{}", offset_min.round() as i64);
    };
    let (Ok(h), Ok(m)) = (h.trim().parse::<i64>(), m.trim().parse::<i64>()) else {
        return format!("+{}", offset_min.round() as i64);
    };
    let total = h * 60 + m + offset_min.round() as i64;
    format!("{}:{:02}", (total / 60) % 24, total % 60)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Block;

    fn three_v_three(_: &str) -> Option<BlockGame> {
        Some(BlockGame {
            name: "3v3".into(),
            on_court: 6,
            rotation: "none".into(),
            targets: vec!["off.extend.drive-and-kick".into()],
            stage: "team-based".into(),
        })
    }

    fn practice(blocks: Vec<(f64, usize)>) -> Practice {
        Practice {
            id: "p".into(),
            planned_minutes: 90.0,
            transition_s: 60.0,
            players: Some(12),
            start: "15:30".into(),
            blocks: blocks
                .into_iter()
                .enumerate()
                .map(|(i, (m, baskets))| Block {
                    id: format!("b{i}"),
                    game: "g".into(),
                    minutes: m,
                    baskets: (0..baskets).map(|k| format!("B{k}")).collect(),
                    ..Default::default()
                })
                .collect(),
            ..Default::default()
        }
    }

    /// S8 §3 worked example: 9 blocks = 82 min + 8 transitions of 60 s = 90 → 0 unassigned.
    #[test]
    fn s8_time_accounting() {
        let mut blocks = vec![(10.0, 1); 8];
        blocks.push((2.0, 1));
        let m = metrics(&practice(blocks), &three_v_three);
        assert_eq!(m.block_minutes, 82.0);
        assert_eq!(m.planned_total, 90.0);
        assert_eq!(m.unassigned, 0.0);
    }

    /// S8 §4 / S4-T1: 12 players, 3v3 on one basket → 0.5; on two baskets → 1.0.
    #[test]
    fn s8_active_ratio() {
        let one = metrics(&practice(vec![(10.0, 1)]), &three_v_three);
        assert_eq!(one.blocks[0].active_ratio, Some(0.5));
        assert_eq!(one.blocks[0].active_minutes, 5.0);
        assert_eq!(one.low_active, vec!["b0".to_string()]);
        let two = metrics(&practice(vec![(10.0, 2)]), &three_v_three);
        assert_eq!(two.blocks[0].active_ratio, Some(1.0));
    }

    /// S6-T1 + R12.
    #[test]
    fn interleaving_flags_three_in_a_row() {
        assert_eq!(principle("off.extend.attack-closeout"), "off.extend");
        let p = practice(vec![(5.0, 1), (5.0, 1), (5.0, 1)]);
        let runs = interleaving_runs(&p, &three_v_three, 3);
        assert_eq!(runs, vec![("off.extend".to_string(), "b0".to_string(), 3)]);
    }

    #[test]
    fn clock_times() {
        assert_eq!(clock_time("15:30", 12.0), "15:42");
        assert_eq!(clock_time("15:50", 15.0), "16:05");
        assert_eq!(clock_time("", 5.0), "+5");
    }
}
