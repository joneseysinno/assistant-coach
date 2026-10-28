//! Play graph analysis (S7 §3): decompose into reads, R9 trained/untrained, loops, paths
//! and the derived animation. Plays are graphs; animation is never stored (R8, D-002).

use crate::model::{Play, Point};

/// Every read that guards a transition, in first-seen order (S7 decompose).
pub fn reads(play: &Play) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for t in &play.transitions {
        if let Some(g) = &t.guard {
            if !out.contains(g) {
                out.push(g.clone());
            }
        }
    }
    out
}

/// R9: reads with no game that trains them. `games_training(read)` returns library game
/// ids whose `reads` list it; the play's own `trained_by` also counts.
pub fn untrained(play: &Play, games_training: &dyn Fn(&str) -> Vec<String>) -> Vec<String> {
    reads(play)
        .into_iter()
        .filter(|r| {
            play.trained_by.get(r).is_none_or(|v| v.is_empty()) && games_training(r).is_empty()
        })
        .collect()
}

/// Does the graph contain a cycle? (A `concept` must; a `linear-set` mustn't — S7 §4.3.)
pub fn has_loop(play: &Play) -> bool {
    fn visit(play: &Play, id: &str, stack: &mut Vec<String>, done: &mut Vec<String>) -> bool {
        if stack.iter().any(|s| s == id) {
            return true;
        }
        if done.iter().any(|s| s == id) {
            return false;
        }
        stack.push(id.to_string());
        for t in play.transitions.iter().filter(|t| t.from == id) {
            if visit(play, &t.to, stack, done) {
                return true;
            }
        }
        stack.pop();
        done.push(id.to_string());
        false
    }
    let mut done = Vec::new();
    play.states
        .iter()
        .any(|s| visit(play, &s.id, &mut Vec::new(), &mut done))
}

/// Problems a coach should see (S7 §4).
pub fn lint(play: &Play) -> Vec<String> {
    let mut out = Vec::new();
    for s in &play.states {
        let unguarded = play
            .transitions
            .iter()
            .filter(|t| t.from == s.id && t.guard.is_none())
            .count();
        if unguarded > 1 {
            out.push(format!(
                "'{}' has {unguarded} moves with no read: give each one the read that picks it",
                s.label
            ));
        }
        for r in &play.roles {
            if !s.at.contains_key(r) {
                out.push(format!("'{}' has no spot for {r}", s.label));
            }
        }
    }
    for t in &play.transitions {
        if play.state(&t.from).is_none() || play.state(&t.to).is_none() {
            out.push(format!("Move {} points at a missing state", t.id));
        }
    }
    let looped = has_loop(play);
    if play.kind == "concept" && !looped {
        out.push("A concept should loop back (a rule system, not a sequence)".into());
    }
    if play.kind == "linear-set" && looped {
        out.push("A linear set shouldn't loop: change the kind to concept".into());
    }
    out
}

/// A path from the first state: at each state take the transition `choose` picks
/// (default: the first one), stopping at a dead end or when a transition repeats.
pub fn path(play: &Play, choose: &dyn Fn(&str) -> Option<String>) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let Some(mut at) = play.states.first().map(|s| s.id.clone()) else {
        return out;
    };
    loop {
        let options: Vec<_> = play.transitions.iter().filter(|t| t.from == at).collect();
        let pick = choose(&at)
            .and_then(|id| options.iter().find(|t| t.id == id).copied())
            .or(options.first().copied());
        let Some(t) = pick else { break };
        if out.contains(&t.id) {
            break;
        }
        out.push(t.id.clone());
        at = t.to.clone();
    }
    out
}

/// Seconds per action in the derived animation (S7 §3).
pub const SECONDS_PER_ACTION: f64 = 1.2;

pub fn path_seconds(play: &Play, path: &[String]) -> f64 {
    path.iter()
        .filter_map(|id| play.transitions.iter().find(|t| &t.id == id))
        .map(|t| t.actions.len().max(1) as f64 * SECONDS_PER_ACTION)
        .sum()
}

/// Player positions and the ball holder at one moment.
pub type Frame = (Vec<(String, Point)>, Option<String>);

/// One animation frame at `t` seconds along `path`: positions and the ball holder.
pub fn frame(play: &Play, path: &[String], t: f64) -> Option<Frame> {
    let first = play.states.first()?;
    let mut elapsed = 0.0;
    let mut from = first;
    for id in path {
        let tr = play.transitions.iter().find(|x| &x.id == id)?;
        let to = play.state(&tr.to)?;
        let dur = tr.actions.len().max(1) as f64 * SECONDS_PER_ACTION;
        if t <= elapsed + dur {
            let k = ((t - elapsed) / dur).clamp(0.0, 1.0);
            let k = k * k * (3.0 - 2.0 * k); // ease in-out
            let pos = play
                .roles
                .iter()
                .filter_map(|r| {
                    let a = from.at.get(r)?;
                    let b = to.at.get(r).unwrap_or(a);
                    Some((
                        r.clone(),
                        [a[0] + (b[0] - a[0]) * k, a[1] + (b[1] - a[1]) * k],
                    ))
                })
                .collect();
            let ball = if k < 0.5 {
                from.ball.clone()
            } else {
                to.ball.clone()
            };
            return Some((pos, ball));
        }
        elapsed += dur;
        from = to;
    }
    Some((
        play.roles
            .iter()
            .filter_map(|r| from.at.get(r).map(|p| (r.clone(), *p)))
            .collect(),
        from.ball.clone(),
    ))
}

/// Roles a transition involves (actions + the guard's cue source when known) — S7 §2.
pub fn participants(play: &Play, transition: &str) -> Vec<String> {
    let mut v: Vec<String> = Vec::new();
    if let Some(t) = play.transitions.iter().find(|t| t.id == transition) {
        for a in &t.actions {
            for r in a.roles() {
                if !v.contains(&r) {
                    v.push(r);
                }
            }
        }
    }
    v
}
