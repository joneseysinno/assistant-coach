//! Doc type **Play** — plays as decision graphs (03 §4, S7, R8): states (court snapshots)
//! joined by moves, each move optionally guarded by a read. Drag players to set a state;
//! moves fill in from the movement; playback animates one path (derived, never stored);
//! every read shows which games train it (R9) and can generate one.

use std::collections::HashMap;
use std::time::Duration;

use ac_domain::model::{ActionArgs, Target};
use ac_domain::store::{Doc, DocKind};
use ac_domain::{play as graph, Action, Play, PlayState, Point, Transition, PLAY_KINDS};
use ac_sport_basketball::{templates, vocab};
use app_shell::prelude::*;
use dioxus::prelude::*;

use super::{Editing, Map};
use crate::court::{ArrowDraw, CourtView, PlayerDraw};
use crate::intern;
use crate::ui::*;

pub const ID: &str = "play";

pub struct PlayDoc {
    pub ed: Editing<Play>,
    pub state: &'static str,
    pub sel_t: Option<&'static str>,
    pub sel_player: Option<&'static str>,
}

pub static PLAYS: Map<PlayDoc> = Signal::global(HashMap::new);

/// Playback time per play doc (seconds), when playing.
static PLAYBACK: GlobalSignal<HashMap<String, f64>> = Signal::global(HashMap::new);

fn ensure(id: &str) {
    if PLAYS.peek().contains_key(id) {
        return;
    }
    let p = crate::store::load_play(id).unwrap_or_else(|| {
        let p = templates::new_play(id);
        crate::store::save(Doc::Play(p.clone()));
        p
    });
    let first = intern(p.states.first().map(|s| s.id.as_str()).unwrap_or("s0"));
    PLAYS.write().insert(
        id.to_string(),
        PlayDoc {
            ed: Editing::new(p),
            state: first,
            sel_t: None,
            sel_player: None,
        },
    );
}

/// `in_viewport`: keep-alive viewports know their own document (`use_document`); panels
/// and the status bar follow the active document, because the shell reuses their scope
/// and its document context goes stale after a switch.
fn use_play_id_in(in_viewport: bool) -> Option<&'static str> {
    let shell = use_shell();
    let from_ctx = use_document();
    let doc = if in_viewport {
        from_ctx?
    } else {
        shell.active_document()?
    };
    if doc.kind != ID {
        return None;
    }
    if let Some(new) = super::adopt_shell_id(shell, DocKind::Play, &doc.id) {
        ensure(&new);
        if let Some(name) = snapshot(&new).map(|s| s.play.name) {
            super::rename(shell, &new, &name);
        }
        return Some(intern(&new));
    }
    ensure(&doc.id);
    Some(intern(&doc.id))
}

fn use_play_id() -> Option<&'static str> {
    use_play_id_in(false)
}

#[derive(Clone, PartialEq)]
struct Snap {
    play: Play,
    state: &'static str,
    sel_t: Option<&'static str>,
    sel_player: Option<&'static str>,
}

fn snapshot(id: &str) -> Option<Snap> {
    PLAYS.read().get(id).map(|d| Snap {
        play: d.ed.value.clone(),
        state: d.state,
        sel_t: d.sel_t,
        sel_player: d.sel_player,
    })
}

fn edit(id: &str, key: &str, f: impl FnOnce(&mut Play)) {
    let saved = {
        let mut all = PLAYS.write();
        let Some(d) = all.get_mut(id) else { return };
        d.ed.edit(key, f).then(|| d.ed.value.clone())
    };
    if let Some(p) = saved {
        crate::store::save(Doc::Play(p));
    }
}

fn set_view(id: &str, f: impl FnOnce(&mut PlayDoc)) {
    if let Some(d) = PLAYS.write().get_mut(id) {
        f(d)
    }
}

fn is_auto(t: &Transition) -> bool {
    t.extra
        .get("auto")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}

/// Read the moves off two snapshots: ball changes → pass; ball-holder moves → dribble;
/// anyone else who moves → cut.
pub fn detect_actions(from: &PlayState, to: &PlayState) -> Vec<Action> {
    let mut out = Vec::new();
    let moved = |r: &str| -> Option<Point> {
        let a = from.at.get(r)?;
        let b = to.at.get(r)?;
        (((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)).sqrt() > 1.0).then_some(*b)
    };
    let holder = from.ball.clone();
    if let Some(h) = &holder {
        if let Some(p) = moved(h) {
            out.push(Action::new(
                "dribble",
                ActionArgs {
                    by: Some(h.clone()),
                    to: Some(Target::Spot(p)),
                    ..Default::default()
                },
            ));
        }
    }
    for r in to.at.keys() {
        if Some(r) == holder.as_ref() {
            continue;
        }
        if let Some(p) = moved(r) {
            out.push(Action::new(
                "cut",
                ActionArgs {
                    by: Some(r.clone()),
                    to: Some(Target::Spot(p)),
                    ..Default::default()
                },
            ));
        }
    }
    if let (Some(a), Some(b)) = (&from.ball, &to.ball) {
        if a != b {
            out.push(Action::new(
                "pass",
                ActionArgs {
                    from: Some(a.clone()),
                    to: Some(Target::Role(b.clone())),
                    ..Default::default()
                },
            ));
        }
    }
    out
}

/// After a state changes, refresh the moves into it that were filled in automatically.
fn refresh_auto(p: &mut Play, state: &str) {
    let states = p.states.clone();
    for t in p
        .transitions
        .iter_mut()
        .filter(|t| t.to == state || t.from == state)
    {
        if !(is_auto(t) || t.actions.is_empty()) {
            continue;
        }
        let (Some(a), Some(b)) = (
            states.iter().find(|s| s.id == t.from),
            states.iter().find(|s| s.id == t.to),
        ) else {
            continue;
        };
        t.actions = detect_actions(a, b);
        t.extra.insert("auto".into(), serde_yaml::Value::Bool(true));
    }
}

fn add_state(pid: &'static str, guarded: bool) {
    let Some(s) = snapshot(pid) else { return };
    let mut new_state = "";
    let mut new_t = "";
    edit(pid, "", |p| {
        let Some(cur) = p.state(s.state).cloned() else {
            return;
        };
        let sid = p.fresh_state_id();
        let tid = p.fresh_transition_id();
        let n = p.states.len() + 1;
        p.states.push(PlayState {
            id: sid.clone(),
            label: if guarded {
                format!("Branch {n}")
            } else {
                format!("Step {n}")
            },
            ball: cur.ball.clone(),
            at: cur.at.clone(),
            ..Default::default()
        });
        let used: Vec<String> = graph::reads(p);
        let guard = guarded.then(|| {
            vocab::READS
                .iter()
                .map(|r| r.id.to_string())
                .find(|r| !used.contains(r))
                .unwrap_or_else(|| "read.closeout.high-fast".into())
        });
        let mut t = Transition {
            id: tid.clone(),
            from: cur.id.clone(),
            to: sid.clone(),
            guard,
            ..Default::default()
        };
        t.extra.insert("auto".into(), serde_yaml::Value::Bool(true));
        p.transitions.push(t);
        new_state = intern(&sid);
        new_t = intern(&tid);
    });
    if !new_state.is_empty() {
        set_view(pid, |d| {
            d.state = new_state;
            d.sel_t = Some(new_t);
        });
    }
}

pub fn doc_type() -> DocType {
    let mut t = DocType::new(ID, "Play", icons::PLAY, Viewport);
    t.toolbar = vec![
        ToolGroup::new(
            "Edit",
            vec![
                Tool::new("pl.undo", "Undo", icons::UNDO).key("Ctrl+Z"),
                Tool::new("pl.redo", "Redo", icons::REDO).key("Ctrl+Y"),
            ],
        ),
        ToolGroup::new(
            "Graph",
            vec![
                Tool::new("pl.next", "Next step (always)", icons::CHEVRON_RIGHT),
                Tool::new("pl.branch", "Branch on a read", icons::LINK),
            ],
        ),
        ToolGroup::new(
            "Players",
            vec![Tool::new("pl.defense", "Show / hide defenders", icons::EYE)],
        ),
        ToolGroup::new(
            "Play",
            vec![
                Tool::new("pl.play", "Play the path", icons::PLAY).primary(),
                Tool::new("pl.yaml", "Export YAML", icons::SHARE),
            ],
        ),
    ];
    t.left_rail = vec![RailItem::panel(
        "pl.graph",
        "States & branches",
        icons::TREE,
        GraphPane,
    )];
    t.right_rail = vec![
        RailItem::panel("pl.props", "Selected", icons::INFO, PropsPane),
        RailItem::panel("pl.reads", "Reads (R9)", icons::TARGET, ReadsPane),
    ];
    t.status = Some(View(Status));
    t.default_left = Some("pl.graph");
    t.default_right = Some("pl.props");
    t
}

pub fn handle(_shell: Shell, doc: &str, id: &'static str) {
    let pid = intern(doc);
    match id {
        "pl.undo" | "pl.redo" => {
            let saved = {
                let mut all = PLAYS.write();
                let Some(d) = all.get_mut(doc) else { return };
                let ok = if id == "pl.undo" {
                    d.ed.undo()
                } else {
                    d.ed.redo()
                };
                if ok && d.ed.value.state(d.state).is_none() {
                    d.state = intern(
                        d.ed.value
                            .states
                            .first()
                            .map(|s| s.id.as_str())
                            .unwrap_or("s0"),
                    );
                    d.sel_t = None;
                }
                ok.then(|| d.ed.value.clone())
            };
            if let Some(p) = saved {
                crate::store::save(Doc::Play(p));
            }
        }
        "pl.next" => add_state(pid, false),
        "pl.branch" => add_state(pid, true),
        "pl.defense" => edit(pid, "", |p| {
            let has_d = p.roles.iter().any(|r| r.starts_with('D'));
            if has_d {
                p.roles.retain(|r| !r.starts_with('D'));
                for s in &mut p.states {
                    s.at.retain(|k, _| !k.starts_with('D'));
                }
            } else {
                let offense: Vec<String> = p.roles.clone();
                for o in &offense {
                    let d = o.replacen('O', "D", 1);
                    p.roles.push(d.clone());
                    for s in &mut p.states {
                        if let Some(at) = s.at.get(o).copied() {
                            // between the player and the rim
                            s.at.insert(d.clone(), [at[0] * 0.82, at[1] * 0.82 - 0.5]);
                        }
                    }
                }
            }
        }),
        "pl.play" => {
            let playing = PLAYBACK.read().contains_key(doc);
            if playing {
                PLAYBACK.write().remove(doc);
            } else {
                PLAYBACK.write().insert(doc.to_string(), 0.0);
            }
        }
        "pl.yaml" => {
            if let Some(s) = snapshot(doc) {
                super::export_yaml(DocKind::Play, &Doc::Play(s.play));
            }
        }
        _ => {}
    }
}

/// Path for playback: from the first state, preferring the selected move where it leaves.
fn chosen_path(p: &Play, sel_t: Option<&str>) -> Vec<String> {
    let pick = sel_t
        .and_then(|t| p.transitions.iter().find(|x| x.id == t))
        .map(|t| (t.from.clone(), t.id.clone()));
    // also prefer the branch that leads toward the selected move
    let toward: Option<String> = pick.as_ref().map(|(from, _)| from.clone());
    graph::path(p, &|state: &str| {
        if let Some((from, id)) = &pick {
            if from == state {
                return Some(id.clone());
            }
        }
        if let Some(goal) = &toward {
            // a move out of `state` that reaches the goal state
            return p
                .transitions
                .iter()
                .find(|t| t.from == state && (&t.to == goal || reaches(p, &t.to, goal, 0)))
                .map(|t| t.id.clone());
        }
        None
    })
}

fn reaches(p: &Play, from: &str, goal: &str, depth: usize) -> bool {
    if depth > 12 {
        return false;
    }
    p.transitions
        .iter()
        .filter(|t| t.from == from)
        .any(|t| t.to == goal || reaches(p, &t.to, goal, depth + 1))
}

fn arrows_for(p: &Play, state: &PlayState, t: &Transition, strong: bool) -> Vec<ArrowDraw> {
    let to_state = p.state(&t.to);
    let pos = |s: &PlayState, r: &str| s.at.get(r).copied();
    let mut out = Vec::new();
    for a in &t.actions {
        let args = a.args();
        match a.kind() {
            "pass" | "handoff" => {
                if let (Some(f), Some(r)) = (
                    args.from.as_deref().and_then(|x| pos(state, x)),
                    a.to_role(),
                ) {
                    let target = to_state.and_then(|s| pos(s, &r)).or_else(|| pos(state, &r));
                    if let Some(q) = target {
                        out.push(ArrowDraw {
                            kind: a.kind().to_string(),
                            from: f,
                            to: q,
                            strong,
                        });
                    }
                }
            }
            "screen" => {
                // a short stem from the screener toward the player it frees, ending in a bar
                if let Some(f) = args.by.as_deref().and_then(|b| pos(state, b)) {
                    let toward = args
                        .for_role
                        .as_deref()
                        .and_then(|r| pos(state, r))
                        .unwrap_or([0.0, f[1] + 5.0]);
                    let (dx, dy) = (toward[0] - f[0], toward[1] - f[1]);
                    let len = (dx * dx + dy * dy).sqrt().max(0.01);
                    let q = [f[0] + dx / len * 2.6, f[1] + dy / len * 2.6];
                    out.push(ArrowDraw {
                        kind: "screen".into(),
                        from: f,
                        to: q,
                        strong,
                    });
                }
            }
            "shot" => {
                if let Some(f) = args.by.as_deref().and_then(|b| pos(state, b)) {
                    out.push(ArrowDraw {
                        kind: "shot".into(),
                        from: f,
                        to: [0.0, 0.0],
                        strong,
                    });
                }
            }
            _ => {
                if let Some(by) = args.by.as_deref() {
                    let dest = a.to_spot().or_else(|| to_state.and_then(|s| pos(s, by)));
                    if let (Some(f), Some(q)) = (pos(state, by), dest) {
                        out.push(ArrowDraw {
                            kind: a.kind().to_string(),
                            from: f,
                            to: q,
                            strong,
                        });
                    }
                }
            }
        }
    }
    out
}

fn role_team(r: &str) -> &'static str {
    if r.starts_with('D') {
        "defense"
    } else {
        "offense"
    }
}

/// The shell reuses one scope for whatever view sits in a slot (a lone keyed child),
/// so this wrapper has no hooks and renders a distinct component type.
#[allow(non_snake_case)]
fn Viewport() -> Element {
    rsx! { ViewportBody {} }
}

#[component]
fn ViewportBody() -> Element {
    super::use_undo_keys();
    let Some(pid) = use_play_id_in(true) else {
        return rsx! {};
    };
    let Some(s) = snapshot(pid) else {
        return rsx! {};
    };
    let p = &s.play;
    let preset = crate::settings::SETTINGS.read().court.clone();
    let playing_t = PLAYBACK.read().get(pid).copied();

    // playback clock
    use_future(move || async move {
        loop {
            tokio::time::sleep(Duration::from_millis(33)).await;
            let t = PLAYBACK.peek().get(pid).copied();
            if let Some(t) = t {
                let total = snapshot(pid)
                    .map(|s| graph::path_seconds(&s.play, &chosen_path(&s.play, s.sel_t)))
                    .unwrap_or(0.0);
                if t > total + 0.6 {
                    PLAYBACK.write().remove(pid);
                } else {
                    PLAYBACK.write().insert(pid.to_string(), t + 0.033);
                }
            }
        }
    });

    let Some(state) = p
        .state(s.state)
        .cloned()
        .or_else(|| p.states.first().cloned())
    else {
        return rsx! { Empty { text: "This play has no states." } };
    };
    let (players, arrows, caption) = if let Some(t) = playing_t {
        let path = chosen_path(p, s.sel_t);
        let (pos, ball) = graph::frame(p, &path, t).unwrap_or_default();
        let players: Vec<PlayerDraw> = pos
            .into_iter()
            .map(|(r, at)| PlayerDraw {
                team: role_team(&r).into(),
                ball: ball.as_deref() == Some(r.as_str()),
                selected: false,
                id: r,
                at,
            })
            .collect();
        (
            players,
            vec![],
            format!(
                "Playing {:.1} s of {:.1} s",
                t,
                graph::path_seconds(p, &path)
            ),
        )
    } else {
        let players: Vec<PlayerDraw> = p
            .roles
            .iter()
            .filter_map(|r| {
                state.at.get(r).map(|at| PlayerDraw {
                    id: r.clone(),
                    at: *at,
                    team: role_team(r).into(),
                    ball: state.ball.as_deref() == Some(r.as_str()),
                    selected: s.sel_player == Some(r.as_str()),
                })
            })
            .collect();
        let mut arrows = Vec::new();
        // how we got here, faded
        for t in p.transitions.iter().filter(|t| t.to == state.id) {
            if let Some(prev) = p.state(&t.from) {
                for mut a in arrows_for(p, prev, t, false) {
                    a.kind = format!("{} app-arrow-in", a.kind);
                    arrows.push(a);
                }
            }
        }
        for t in p.transitions.iter().filter(|t| t.from == state.id) {
            arrows.extend(arrows_for(p, &state, t, s.sel_t == Some(t.id.as_str())));
        }
        (
            players,
            arrows,
            "Drag players to set this state. Click a player, then 'Give ball'.".to_string(),
        )
    };
    let sid = intern(&state.id);
    let roles: Vec<(String, String)> = p
        .roles
        .iter()
        .filter(|r| !r.starts_with('D'))
        .map(|r| (r.clone(), r.clone()))
        .collect();
    let idx = p.states.iter().position(|x| x.id == state.id).unwrap_or(0);
    let prev = (idx > 0).then(|| intern(&p.states[idx - 1].id));
    let next = p.states.get(idx + 1).map(|x| intern(&x.id));
    let sel_player = s.sel_player;
    rsx! {
        div { class: "app-play",
            div { class: "app-play-head",
                button { class: "app-icon-btn", disabled: prev.is_none(), title: "Previous state", onclick: move |_| if let Some(x) = prev { set_view(pid, |d| { d.state = x; d.sel_t = None; }) }, "‹" }
                TextIn { value: state.label.clone(), class: "app-state-name", on_change: move |v: String| edit(pid, "slabel", move |p| if let Some(st) = p.states.iter_mut().find(|x| x.id == sid) { st.label = v }) }
                button { class: "app-icon-btn", disabled: next.is_none(), title: "Next state", onclick: move |_| if let Some(x) = next { set_view(pid, |d| { d.state = x; d.sel_t = None; }) }, "›" }
                span { class: "app-field-label", "Ball" }
                Pick { value: state.ball.clone().unwrap_or_default(), class: "app-w-auto", options: { let mut v = vec![("".to_string(), "—".to_string())]; v.extend(roles.clone()); v },
                    on_change: move |v: String| edit(pid, "", move |p| { if let Some(st) = p.states.iter_mut().find(|x| x.id == sid) { st.ball = (!v.is_empty()).then_some(v) } refresh_auto(p, sid); }) }
                if let Some(pl) = sel_player {
                    button { class: "app-btn app-sm", onclick: move |_| edit(pid, "", move |p| { if let Some(st) = p.states.iter_mut().find(|x| x.id == sid) { st.ball = Some(pl.to_string()) } refresh_auto(p, sid); }), "Give ball to {pl}" }
                }
                div { class: "app-spacer" }
                button { class: "app-btn app-sm", onclick: move |_| add_state(pid, false), "+ Next step" }
                button { class: "app-btn app-sm", onclick: move |_| add_state(pid, true), "+ Branch on a read" }
                button { class: "app-btn app-sm app-primary", onclick: move |_| {
                    if PLAYBACK.read().contains_key(pid) { PLAYBACK.write().remove(pid); } else { PLAYBACK.write().insert(pid.to_string(), 0.0); }
                }, if playing_t.is_some() { "■ Stop" } else { "▶ Play" } }
            }
            CourtView {
                preset,
                region: if p.court.is_empty() { "half".to_string() } else { p.court.clone() },
                zones: vec![],
                players,
                arrows,
                caption,
                on_player: move |r: String| set_view(pid, |d| d.sel_player = Some(intern(&r))),
                on_move: move |(r, at): (String, Point)| {
                    if PLAYBACK.read().contains_key(pid) { return; }
                    edit(pid, &format!("drag-{r}"), move |p| {
                        if let Some(st) = p.states.iter_mut().find(|x| x.id == sid) { st.at.insert(r.clone(), at); }
                        refresh_auto(p, sid);
                    })
                },
            }
        }
    }
}

// ---------------------------------------------------------------------------------------
// left: states & branches
// ---------------------------------------------------------------------------------------

/// The shell reuses one scope for whatever view sits in a slot (a lone keyed child),
/// so this wrapper has no hooks and renders a distinct component type.
#[allow(non_snake_case)]
fn GraphPane() -> Element {
    rsx! { GraphPaneBody {} }
}

#[component]
fn GraphPaneBody() -> Element {
    let shell = use_shell();
    let Some(pid) = use_play_id() else {
        return rsx! {};
    };
    let Some(s) = snapshot(pid) else {
        return rsx! {};
    };
    let p = s.play.clone();
    let looped = graph::has_loop(&p);
    rsx! {
        div { class: "app-pane app-form",
            Field { label: "Play",
                TextIn { value: p.name.clone(), on_change: move |v: String| { super::rename(shell, pid, &v); edit(pid, "name", move |p| p.name = v) } }
            }
            Field { label: "Kind", hint: if looped { "This graph loops back on itself" } else { "" },
                Pick { value: p.kind.clone(), options: PLAY_KINDS.iter().map(|k| (k.to_string(), match *k { "linear-set" => "Set (one way through)", "read-set" => "Set with reads", _ => "Concept (loops)" }.to_string())).collect(),
                    on_change: move |v: String| edit(pid, "", move |p| p.kind = v) }
            }
            Field { label: "Court",
                Pick { value: if p.court.is_empty() { "half".to_string() } else { p.court.clone() }, options: opts(&vocab::COURT_REGIONS[..4]), on_change: move |v: String| edit(pid, "", move |p| p.court = v) }
            }
            h4 { class: "app-sub", "States & moves" }
            for st in p.states.iter() {
                {
                    let sid = intern(&st.id);
                    let cur = s.state == sid;
                    let outs: Vec<Transition> = p.transitions.iter().filter(|t| t.from == st.id).cloned().collect();
                    let can_delete = p.states.len() > 1;
                    rsx! {
                        div { key: "{sid}",
                            div { class: if cur { "app-row app-row-sel" } else { "app-row" },
                                onclick: move |_| set_view(pid, |d| { d.state = sid; d.sel_t = None; }),
                                span { class: "app-cat app-cat-base", "◆" }
                                div { class: "app-row-main",
                                    div { class: "app-row-title", "{st.label}" }
                                    div { class: "app-row-sub", "ball: {st.ball.clone().unwrap_or_else(|| \"—\".into())}" }
                                }
                                if can_delete {
                                    div { class: "app-row-tools",
                                        button { class: "app-icon-btn app-sm", title: "Delete state", onclick: move |e| {
                                            e.stop_propagation();
                                            edit(pid, "", move |p| { p.states.retain(|x| x.id != sid); p.transitions.retain(|t| t.from != sid && t.to != sid); });
                                            set_view(pid, |d| { let f = d.ed.value.states.first().map(|x| intern(&x.id)).unwrap_or("s0"); d.state = f; d.sel_t = None; });
                                        }, "×" }
                                    }
                                }
                            }
                            div { class: "app-subrows",
                                for t in outs.iter() {
                                    {
                                        let tid = intern(&t.id);
                                        let to_label = p.state(&t.to).map(|x| x.label.clone()).unwrap_or_else(|| t.to.clone());
                                        let guard = t.guard.as_deref().map(vocab::read_label).unwrap_or_else(|| "always".into());
                                        let sel = s.sel_t == Some(tid);
                                        let moves: Vec<String> = t.actions.iter().map(|a| a.kind().to_string()).collect();
                                        rsx! {
                                            div { key: "{tid}", class: if sel { "app-subrow app-row-sel" } else { "app-subrow" },
                                                onclick: move |_| set_view(pid, |d| { d.sel_t = Some(tid); d.state = sid; }),
                                                span { class: if t.guard.is_some() { "app-guard" } else { "app-guard app-guard-always" }, "{guard}" }
                                                " → {to_label}"
                                                if !moves.is_empty() { span { class: "app-muted", " ({moves.join(\", \")})" } }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            div { class: "app-inline",
                button { class: "app-btn app-sm", onclick: move |_| add_state(pid, false), "+ Next step" }
                button { class: "app-btn app-sm", onclick: move |_| add_state(pid, true), "+ Branch on a read" }
            }
            if p.states.len() > 1 {
                Field { label: "Loop back", hint: "Concepts loop: connect this state back to an earlier one",
                    Pick { value: "".to_string(), options: { let mut v = vec![("".to_string(), "connect current state to…".to_string())]; v.extend(p.states.iter().filter(|x| x.id != s.state).map(|x| (x.id.clone(), x.label.clone()))); v },
                        on_change: { let from = s.state; move |to: String| if !to.is_empty() { edit(pid, "", move |p| { let id = p.fresh_transition_id(); p.transitions.push(Transition { id, from: from.to_string(), to, ..Default::default() }); }) } } }
                }
            }
            Field { label: "Notes",
                TextArea { value: p.notes.clone(), on_change: move |v: String| edit(pid, "notes", move |p| p.notes = v) }
            }
        }
    }
}

// ---------------------------------------------------------------------------------------
// right: selected move / state
// ---------------------------------------------------------------------------------------

/// The shell reuses one scope for whatever view sits in a slot (a lone keyed child),
/// so this wrapper has no hooks and renders a distinct component type.
#[allow(non_snake_case)]
fn PropsPane() -> Element {
    rsx! { PropsPaneBody {} }
}

#[component]
fn PropsPaneBody() -> Element {
    let Some(pid) = use_play_id() else {
        return rsx! {};
    };
    let Some(s) = snapshot(pid) else {
        return rsx! {};
    };
    let p = s.play.clone();
    let Some(tid) = s.sel_t else {
        let lint = graph::lint(&p);
        return rsx! {
            div { class: "app-pane",
                h3 { class: "app-pane-title", "Building a play" }
                ol { class: "app-steps",
                    li { "Drag players into the starting alignment." }
                    li { "Press “+ Next step” for a move that always happens, or “+ Branch on a read” for a choice." }
                    li { "In the new state, drag the players to where they go and give the ball to whoever has it. The moves (pass, dribble, cut) fill in from that." }
                    li { "Pick the read that decides each branch, then check the Reads panel: every read should have a game that trains it (R9)." }
                }
                if !lint.is_empty() {
                    Section { title: "Checks".to_string(),
                        for (i, l) in lint.iter().enumerate() { p { key: "{i}", class: "app-warn", "{l}" } }
                    }
                }
            }
        };
    };
    let Some(t) = p.transitions.iter().find(|t| t.id == tid).cloned() else {
        return rsx! {};
    };
    let roles: Vec<(String, String)> = p.roles.iter().map(|r| (r.clone(), r.clone())).collect();
    let reads: Vec<(String, String)> = {
        let mut v = vec![("".to_string(), "always (no read)".to_string())];
        v.extend(
            vocab::READS
                .iter()
                .map(|r| (r.id.to_string(), r.cue.to_string())),
        );
        v
    };
    let from_label = p
        .state(&t.from)
        .map(|x| x.label.clone())
        .unwrap_or_default();
    let to_label = p.state(&t.to).map(|x| x.label.clone()).unwrap_or_default();
    let auto = is_auto(&t);
    rsx! {
        div { class: "app-pane app-form",
            h3 { class: "app-pane-title", "Move: {from_label} → {to_label}" }
            Field { label: "Read that picks this move", hint: "What the player sees. Leave as 'always' for a move that always happens.",
                Pick { value: t.guard.clone().unwrap_or_default(), options: reads, on_change: move |v: String| edit(pid, "", move |p| if let Some(t) = p.transitions.iter_mut().find(|t| t.id == tid) { t.guard = (!v.is_empty()).then_some(v) }) }
            }
            Field { label: "Timing cue",
                TextIn { value: t.timing.clone().unwrap_or_default(), placeholder: "e.g. when the ball crosses the free-throw line".to_string(), on_change: move |v: String| edit(pid, "timing", move |p| if let Some(t) = p.transitions.iter_mut().find(|t| t.id == tid) { t.timing = (!v.is_empty()).then_some(v) }) }
            }
            h4 { class: "app-sub", if auto { "Actions (filled from the movement)" } else { "Actions" } }
            for (ai, a) in t.actions.iter().enumerate() {
                {
                    let args = a.args();
                    let actor = a.actor().unwrap_or_default();
                    let target = match a.kind() { "pass" | "handoff" => a.to_role().unwrap_or_default(), "screen" => args.for_role.clone().unwrap_or_default(), _ => String::new() };
                    let kind = a.kind().to_string();
                    rsx! {
                        div { key: "{ai}", class: "app-then",
                            Pick { value: kind.clone(), class: "app-w-auto", options: opts(vocab::ACTIONS), on_change: move |k: String| edit(pid, "", move |p| if let Some(t) = p.transitions.iter_mut().find(|t| t.id == tid) {
                                if let Some(a) = t.actions.get_mut(ai) {
                                    let old = a.args();
                                    let who = old.by.clone().or(old.from.clone());
                                    let args = match k.as_str() {
                                        "pass" | "handoff" => ActionArgs { from: who, to: old.to.clone().filter(|x| matches!(x, Target::Role(_))), ..Default::default() },
                                        "screen" => ActionArgs { by: who, for_role: None, ..Default::default() },
                                        "shot" => ActionArgs { by: who, ..Default::default() },
                                        _ => ActionArgs { by: who, to: old.to.clone().filter(|x| matches!(x, Target::Spot(_))), ..Default::default() },
                                    };
                                    *a = Action::new(&k, args);
                                }
                                t.extra.shift_remove("auto");
                            }) }
                            Pick { value: actor.clone(), class: "app-w-auto", options: roles.clone(), on_change: move |r: String| edit(pid, "", move |p| if let Some(t) = p.transitions.iter_mut().find(|t| t.id == tid) {
                                if let Some(args) = t.actions.get_mut(ai).and_then(|a| a.args_mut()) { if args.from.is_some() { args.from = Some(r) } else { args.by = Some(r) } }
                                t.extra.shift_remove("auto");
                            }) }
                            if matches!(kind.as_str(), "pass" | "handoff" | "screen") {
                                span { class: "app-small", if kind == "screen" { "for" } else { "to" } }
                                Pick { value: target.clone(), class: "app-w-auto", options: { let mut v = vec![("".to_string(), "—".to_string())]; v.extend(roles.clone()); v },
                                    on_change: { let kind = kind.clone(); move |r: String| { let kind = kind.clone(); edit(pid, "", move |p| if let Some(t) = p.transitions.iter_mut().find(|t| t.id == tid) {
                                        if let Some(args) = t.actions.get_mut(ai).and_then(|a| a.args_mut()) { if kind == "screen" { args.for_role = Some(r) } else { args.to = Some(Target::Role(r)) } }
                                        t.extra.shift_remove("auto");
                                    }) } } }
                            }
                            button { class: "app-icon-btn app-sm", title: "Remove", onclick: move |_| edit(pid, "", move |p| if let Some(t) = p.transitions.iter_mut().find(|t| t.id == tid) { if ai < t.actions.len() { t.actions.remove(ai); } t.extra.shift_remove("auto"); }), "×" }
                        }
                    }
                }
            }
            div { class: "app-inline",
                button { class: "app-btn app-sm", onclick: move |_| edit(pid, "", move |p| {
                    let first = p.roles.first().cloned();
                    if let Some(t) = p.transitions.iter_mut().find(|t| t.id == tid) { t.actions.push(Action::new("screen", ActionArgs { by: first, ..Default::default() })); t.extra.shift_remove("auto"); }
                }), "+ action" }
                button { class: "app-btn app-sm", onclick: move |_| edit(pid, "", move |p| {
                    let states = p.states.clone();
                    if let Some(t) = p.transitions.iter_mut().find(|t| t.id == tid) {
                        if let (Some(a), Some(b)) = (states.iter().find(|s| s.id == t.from), states.iter().find(|s| s.id == t.to)) {
                            t.actions = detect_actions(a, b);
                            t.extra.insert("auto".into(), serde_yaml::Value::Bool(true));
                        }
                    }
                }), "Fill from movement" }
            }
            button { class: "app-btn app-sm app-danger", onclick: move |_| { edit(pid, "", move |p| p.transitions.retain(|t| t.id != tid)); set_view(pid, |d| d.sel_t = None); }, "Delete move" }
        }
    }
}

// ---------------------------------------------------------------------------------------
// right: reads (R9) + generate game from read
// ---------------------------------------------------------------------------------------

/// The shell reuses one scope for whatever view sits in a slot (a lone keyed child),
/// so this wrapper has no hooks and renders a distinct component type.
#[allow(non_snake_case)]
fn ReadsPane() -> Element {
    rsx! { ReadsPaneBody {} }
}

#[component]
fn ReadsPaneBody() -> Element {
    let shell = use_shell();
    let Some(pid) = use_play_id() else {
        return rsx! {};
    };
    let Some(s) = snapshot(pid) else {
        return rsx! {};
    };
    let p = s.play.clone();
    let lib = crate::store::LIBRARY.read().clone();
    let games: Vec<crate::store::Entry> = lib
        .iter()
        .filter(|e| e.kind == DocKind::Game)
        .cloned()
        .collect();
    let reads = graph::reads(&p);
    let trains = |r: &str| -> Vec<(String, String)> {
        let mut v: Vec<(String, String)> = games
            .iter()
            .filter(|g| g.reads.iter().any(|x| x == r))
            .map(|g| (g.id.clone(), g.name.clone()))
            .collect();
        for id in p.trained_by.get(r).cloned().unwrap_or_default() {
            if !v.iter().any(|x| x.0 == id) {
                let name = games
                    .iter()
                    .find(|g| g.id == id)
                    .map(|g| g.name.clone())
                    .unwrap_or(id.clone());
                v.push((id, name));
            }
        }
        v
    };
    let untrained = reads.iter().filter(|r| trains(r).is_empty()).count();
    let game_opts: Vec<(String, String)> = games
        .iter()
        .map(|g| (g.id.clone(), g.name.clone()))
        .collect();
    rsx! {
        div { class: "app-pane",
            h3 { class: "app-pane-title", "Reads in this play" }
            if reads.is_empty() {
                Empty { text: "No reads yet. Add a branch on a read to make the play a decision graph." }
            } else if untrained == 0 {
                p { class: "app-ok", "✓ Every read has a game that trains it." }
            } else {
                p { class: "app-warn", "{untrained} read(s) with no game that trains them (R9)." }
            }
            for r in reads.iter() {
                {
                    let rid = intern(r);
                    let def = vocab::read(r);
                    let tr = trains(r);
                    let t_id = p.transitions.iter().find(|t| t.guard.as_deref() == Some(r.as_str())).map(|t| intern(&t.id)).unwrap_or("");
                    let play_for_gen = p.clone();
                    rsx! {
                        div { key: "{rid}", class: if tr.is_empty() { "app-read app-read-untrained" } else { "app-read" },
                            div { class: "app-row-title", "{vocab::read_label(r)}" }
                            if let Some(d) = def {
                                div { class: "app-row-sub", "Cue from the {d.source} · {vocab::target_label(d.target)}" }
                                ul { class: "app-plain app-small",
                                    for (i, (o, ok)) in d.options.iter().enumerate() { li { key: "{i}", if *ok { "✓ {o}" } else { "✗ {o}" } } }
                                }
                            }
                            div { class: "app-row-sub",
                                if tr.is_empty() { span { class: "app-warn-text", "Not trained by any game" } } else { "Trained by:" }
                            }
                            for (gid, gname) in tr.iter() {
                                {
                                    let (g1, n1) = (gid.clone(), gname.clone());
                                    rsx! { button { key: "{gid}", class: "app-link app-small", onclick: move |_| super::open(shell, DocKind::Game, &g1, &n1), "{gname}" } }
                                }
                            }
                            div { class: "app-inline",
                                button { class: "app-btn app-sm", onclick: move |_| {
                                    let gid = crate::store::new_id(DocKind::Game);
                                    let g = templates::game_from_read(&play_for_gen, t_id, rid, &gid);
                                    let name = g.name.clone();
                                    crate::store::save(Doc::Game(g));
                                    let link = gid.clone();
                                    edit(pid, "", move |p| p.trained_by.entry(rid.to_string()).or_default().push(link));
                                    super::open(shell, DocKind::Game, &gid, &name);
                                }, "Generate a game" }
                                Pick { value: "".to_string(), class: "app-w-auto", options: { let mut v = vec![("".to_string(), "link a game…".to_string())]; v.extend(game_opts.clone()); v },
                                    on_change: move |g: String| if !g.is_empty() { edit(pid, "", move |p| { let e = p.trained_by.entry(rid.to_string()).or_default(); if !e.contains(&g) { e.push(g) } }) } }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// The shell reuses one scope for whatever view sits in a slot (a lone keyed child),
/// so this wrapper has no hooks and renders a distinct component type.
#[allow(non_snake_case)]
fn Status() -> Element {
    rsx! { StatusBody {} }
}

#[component]
fn StatusBody() -> Element {
    let Some(pid) = use_play_id() else {
        return rsx! {};
    };
    let Some(s) = snapshot(pid) else {
        return rsx! {};
    };
    let p = &s.play;
    let reads = graph::reads(p).len();
    let secs = graph::path_seconds(p, &chosen_path(p, s.sel_t));
    let plural = |n: usize, w: &str| {
        if n == 1 {
            format!("1 {w}")
        } else {
            format!("{n} {w}s")
        }
    };
    rsx! { span { "{plural(p.states.len(), \"state\")} · {plural(p.transitions.len(), \"move\")} · {plural(reads, \"read\")} · path {secs.max(0.0):.1} s" } }
}
