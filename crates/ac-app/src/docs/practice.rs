//! Doc type **Practice** — the practice planner (03 §5, S8): blocks of library games on a
//! timeline with clock times, baskets and groups; time accounting (R10), active time
//! (R11), interleaving (R12), estimated load; a printable plan with every game card.

use std::collections::HashMap;

use ac_domain::practice::{self, BlockGame};
use ac_domain::store::{Doc, DocKind};
use ac_domain::{Block, Game, Groups, Gym, Practice};
use ac_sport_basketball::{court, estimates, render, templates, vocab};
use app_shell::prelude::*;
use dioxus::prelude::*;

use super::{Editing, Map};
use crate::intern;
use crate::ui::*;

pub const ID: &str = "practice";

pub struct PracDoc {
    pub ed: Editing<Practice>,
    pub sel: Option<&'static str>,
}

pub static PRACTICES: Map<PracDoc> = Signal::global(HashMap::new);

fn ensure(id: &str) {
    if PRACTICES.peek().contains_key(id) {
        return;
    }
    let p = crate::store::load_practice(id).unwrap_or_else(|| {
        let p = templates::new_practice(id, &crate::store::today());
        crate::store::save(Doc::Practice(p.clone()));
        p
    });
    PRACTICES.write().insert(
        id.to_string(),
        PracDoc {
            ed: Editing::new(p),
            sel: None,
        },
    );
}

/// `in_viewport`: keep-alive viewports know their own document (`use_document`); panels
/// and the status bar follow the active document, because the shell reuses their scope
/// and its document context goes stale after a switch.
fn use_practice_id_in(in_viewport: bool) -> Option<&'static str> {
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
    if let Some(new) = super::adopt_shell_id(shell, DocKind::Practice, &doc.id) {
        ensure(&new);
        if let Some(name) = snapshot(&new).map(|s| s.0.title()) {
            super::rename(shell, &new, &name);
        }
        return Some(intern(&new));
    }
    ensure(&doc.id);
    Some(intern(&doc.id))
}

fn use_practice_id() -> Option<&'static str> {
    use_practice_id_in(false)
}

fn snapshot(id: &str) -> Option<(Practice, Option<&'static str>)> {
    PRACTICES
        .read()
        .get(id)
        .map(|d| (d.ed.value.clone(), d.sel))
}

fn edit(id: &str, key: &str, f: impl FnOnce(&mut Practice)) {
    let saved = {
        let mut all = PRACTICES.write();
        let Some(d) = all.get_mut(id) else { return };
        d.ed.edit(key, f).then(|| d.ed.value.clone())
    };
    if let Some(p) = saved {
        crate::store::save(Doc::Practice(p));
    }
}

fn with_b(p: &mut Practice, bid: &str, f: impl FnOnce(&mut Block)) {
    if let Some(b) = p.blocks.iter_mut().find(|b| b.id == bid) {
        f(b)
    }
}

fn gym_of(p: &Practice) -> Gym {
    p.gyms
        .iter()
        .find(|g| g.id == p.gym)
        .cloned()
        .unwrap_or_else(templates::default_gym)
}

fn games_by_id() -> HashMap<String, Game> {
    let _ = crate::store::REVISION.read();
    crate::store::all_games()
        .into_iter()
        .map(|g| (g.id.clone(), g))
        .collect()
}

fn block_game(games: &HashMap<String, Game>, id: &str) -> Option<BlockGame> {
    let g = games.get(id)?;
    Some(BlockGame {
        name: g.name.clone(),
        on_court: g.format.on_court(),
        rotation: g
            .format
            .rotation
            .as_ref()
            .map(|r| r.kind.clone())
            .unwrap_or_default(),
        targets: g.targets(),
        stage: g.stage.clone(),
    })
}

fn new_block(p: &Practice, g: &Game) -> Block {
    let gym = gym_of(p);
    let per_side = g.format.count_side("offense").max(1) as u32;
    Block {
        id: p.fresh_block_id(),
        name: g.name.clone(),
        game: g.id.clone(),
        baskets: gym
            .baskets
            .first()
            .map(|b| vec![b.id.clone()])
            .unwrap_or_default(),
        minutes: 10.0,
        groups: Some(Groups {
            mode: "random".into(),
            team_size: Some(per_side),
            ..Default::default()
        }),
        ..Default::default()
    }
}

pub fn doc_type() -> DocType {
    let mut t = DocType::new(ID, "Practice", icons::TABLE, Viewport);
    t.toolbar = vec![
        ToolGroup::new(
            "Edit",
            vec![
                Tool::new("pr.undo", "Undo", icons::UNDO).key("Ctrl+Z"),
                Tool::new("pr.redo", "Redo", icons::REDO).key("Ctrl+Y"),
            ],
        ),
        ToolGroup::new(
            "Blocks",
            vec![
                Tool::new("pr.add", "Add block", icons::PLUS),
                Tool::new("pr.games", "Pick from games", icons::LAYERS),
            ],
        ),
        ToolGroup::new(
            "Practice",
            vec![
                Tool::new("pr.print", "Print plan + game cards", icons::DOWNLOAD).primary(),
                Tool::new("pr.yaml", "Export YAML", icons::SHARE),
            ],
        ),
    ];
    t.left_rail = vec![RailItem::panel(
        "pr.games",
        "Games",
        icons::LAYERS,
        GamesPane,
    )];
    t.right_rail = vec![RailItem::panel(
        "pr.metrics",
        "Time, load & targets",
        icons::SCALE,
        MetricsPane,
    )];
    t.status = Some(View(Status));
    t.default_left = Some("pr.games");
    t.default_right = Some("pr.metrics");
    t
}

pub fn handle(mut shell: Shell, doc: &str, id: &'static str) {
    match id {
        "pr.undo" | "pr.redo" => {
            let saved = {
                let mut all = PRACTICES.write();
                let Some(d) = all.get_mut(doc) else { return };
                let ok = if id == "pr.undo" {
                    d.ed.undo()
                } else {
                    d.ed.redo()
                };
                ok.then(|| d.ed.value.clone())
            };
            if let Some(p) = saved {
                crate::store::save(Doc::Practice(p));
            }
        }
        "pr.add" => edit(doc, "", |p| {
            let id = p.fresh_block_id();
            p.blocks.push(Block {
                id,
                name: "New block".into(),
                minutes: 10.0,
                ..Default::default()
            });
        }),
        "pr.games" => shell.set_panel(Placement::Left, Some("pr.games")),
        "pr.print" => {
            if let Some((p, _)) = snapshot(doc) {
                super::export_html(&p.title(), plan_html(&p));
            }
        }
        "pr.yaml" => {
            if let Some((p, _)) = snapshot(doc) {
                super::export_yaml(DocKind::Practice, &Doc::Practice(p));
            }
        }
        _ => {}
    }
}

// ---------------------------------------------------------------------------------------
// viewport
// ---------------------------------------------------------------------------------------

/// The shell reuses one scope for whatever view sits in a slot (a lone keyed child),
/// so this wrapper has no hooks and renders a distinct component type.
#[allow(non_snake_case)]
fn Viewport() -> Element {
    rsx! { ViewportBody {} }
}

#[component]
fn ViewportBody() -> Element {
    super::use_undo_keys();
    let shell = use_shell();
    let Some(pid) = use_practice_id_in(true) else {
        return rsx! {};
    };
    let Some((p, sel)) = snapshot(pid) else {
        return rsx! {};
    };
    let games = games_by_id();
    let lookup = |id: &str| block_game(&games, id);
    let m = practice::metrics(&p, &lookup);
    let gym = gym_of(&p);
    let mut game_opts: Vec<(String, String)> = vec![("".into(), "— pick a game —".into())];
    let mut sorted: Vec<&Game> = games.values().collect();
    sorted.sort_by_key(|g| g.name.to_lowercase());
    game_opts.extend(
        sorted
            .iter()
            .map(|g| (g.id.clone(), format!("{} ({})", g.name, g.numbers()))),
    );
    let gyms: Vec<Gym> = if p.gyms.is_empty() {
        vec![templates::default_gym()]
    } else {
        p.gyms.clone()
    };
    let unassigned_class = if m.unassigned < -0.01 {
        "app-err-text"
    } else if m.unassigned > 0.01 {
        "app-warn-text"
    } else {
        "app-ok"
    };
    let last = p.blocks.len().saturating_sub(1);
    rsx! {
        div { class: "app-practice",
            div { class: "app-practice-head",
                div { class: "app-grid-head",
                    Field { label: "Practice",
                        TextIn { value: p.name.clone(), placeholder: p.title(), on_change: move |v: String| { super::rename(shell, pid, &if v.is_empty() { "Practice".into() } else { v.clone() }); edit(pid, "name", move |p| p.name = v) } }
                    }
                    Field { label: "Date", input { class: "app-input", r#type: "date", value: "{p.date}", oninput: move |e| edit(pid, "date", move |p| p.date = e.value()) } }
                    Field { label: "Start", input { class: "app-input", r#type: "time", value: "{p.start}", oninput: move |e| edit(pid, "start", move |p| p.start = e.value()) } }
                    Field { label: "Minutes", NumIn { value: p.planned_minutes, min: 10.0, on_change: move |v: f64| edit(pid, "mins", move |p| p.planned_minutes = v) } }
                    Field { label: "Players", NumIn { value: p.player_count() as f64, min: 0.0, on_change: move |v: f64| edit(pid, "players", move |p| p.players = Some(v.max(0.0) as u32)) } }
                    Field { label: "Changeover (s)", NumIn { value: p.transition_s, min: 0.0, step: 15.0, on_change: move |v: f64| edit(pid, "trans", move |p| p.transition_s = v.max(0.0)) } }
                    Field { label: "Gym",
                        Pick { value: gym.id.clone(), options: gyms.iter().map(|g| (g.id.clone(), format!("{} ({})", g.name, g.court))).collect(), on_change: move |v: String| edit(pid, "", move |p| p.gym = v) }
                    }
                }
                Field { label: "Theme",
                    TextIn { value: p.theme.clone(), placeholder: "What this practice is about".to_string(), on_change: move |v: String| edit(pid, "theme", move |p| p.theme = v) }
                }
                div { class: "app-timebar",
                    span { "Blocks {m.block_minutes} min + changeovers {m.transition_minutes:.0} min = {m.planned_total:.0} of {p.planned_minutes} min · " }
                    span { class: unassigned_class,
                        if m.unassigned < -0.01 { "{-m.unassigned:.0} min over" } else if m.unassigned > 0.01 { "{m.unassigned:.0} min not planned" } else { "every minute accounted for" }
                    }
                }
            }
            div { class: "app-blocks",
                if p.blocks.is_empty() {
                    Empty { text: "No blocks yet. Add games from the Games panel on the left, or use Add block." }
                }
                for (i, b) in p.blocks.iter().enumerate() {
                    {
                        let bid = intern(&b.id);
                        let bm = &m.blocks[i];
                        let t0 = practice::clock_time(&p.start, bm.starts_at);
                        let t1 = practice::clock_time(&p.start, bm.starts_at + b.minutes);
                        let g = games.get(&b.game).cloned();
                        let summary = g.as_ref().map(|g| {
                            let rg = super::game::resolved(g, None);
                            let line = render::board_line(&rg);
                            if line.is_empty() { render::card_lines(&rg).join(" · ") } else { line }
                        });
                        let ratio = bm.active_ratio.map(|r| format!("{:.0}% active", r * 100.0)).unwrap_or_default();
                        let low = bm.active_ratio.is_some_and(|r| r < practice::TARGET_ACTIVE_RATIO);
                        let groups = b.groups.clone().unwrap_or_default();
                        let is_sel = sel == Some(bid);
                        let gopts = {
                            let mut v = game_opts.clone();
                            if !b.game.is_empty() && !games.contains_key(&b.game) {
                                v.push((b.game.clone(), format!("missing: {}", b.game)));
                            }
                            v
                        };
                        let game_open = g.clone();
                        rsx! {
                            div { key: "{bid}",
                                class: if is_sel { "app-block app-block-sel" } else if b.protected { "app-block app-block-protected" } else { "app-block" },
                                onclick: move |_| { if let Some(d) = PRACTICES.write().get_mut(pid) { d.sel = Some(bid) } },
                                div { class: "app-block-time",
                                    div { class: "app-block-clock", "{t0}" }
                                    div { class: "app-muted app-small", "– {t1}" }
                                    div { class: "app-block-min",
                                        NumIn { value: b.minutes, min: 1.0, class: "app-w-num", on_change: move |v: f64| edit(pid, "min", move |p| with_b(p, bid, |b| b.minutes = v.max(1.0))) }
                                        span { class: "app-small", "min" }
                                    }
                                }
                                div { class: "app-block-body",
                                    div { class: "app-block-line",
                                        TextIn { value: b.name.clone(), class: "app-block-name", on_change: move |v: String| edit(pid, "bname", move |p| with_b(p, bid, |b| b.name = v)) }
                                        Pick { value: b.game.clone(), options: gopts, class: "app-block-game", on_change: move |v: String| {
                                            let gname = games_by_id().get(&v).map(|g| g.name.clone());
                                            edit(pid, "", move |p| with_b(p, bid, |b| { if b.name.is_empty() || b.name == "New block" { if let Some(n) = gname { b.name = n; } } b.game = v; }))
                                        } }
                                    }
                                    if let Some(s) = summary {
                                        div { class: "app-block-summary",
                                            span { class: "app-mono app-small", "{s}" }
                                            if let Some(gg) = game_open {
                                                button { class: "app-link app-small", onclick: move |e| { e.stop_propagation(); super::open(shell, DocKind::Game, &gg.id, &gg.name) }, "open game" }
                                            }
                                        }
                                    } else if !b.game.is_empty() {
                                        div { class: "app-err-text app-small", "Game '{b.game}' isn't in the library" }
                                    }
                                    div { class: "app-block-line",
                                        span { class: "app-field-label", "Baskets" }
                                        for bk in gym.baskets.iter() {
                                            {
                                                let on = b.baskets.contains(&bk.id);
                                                let kid = bk.id.clone();
                                                rsx! {
                                                    button { key: "{kid}", class: if on { "app-toggle app-toggle-on" } else { "app-toggle" }, title: "{bk.label}",
                                                        onclick: move |e| { e.stop_propagation(); let k = kid.clone(); edit(pid, "", move |p| with_b(p, bid, |b| { if b.baskets.contains(&k) { b.baskets.retain(|x| *x != k) } else { b.baskets.push(k) } })) },
                                                        "{bk.id}"
                                                    }
                                                }
                                            }
                                        }
                                        span { class: "app-field-label", "Groups" }
                                        Pick { value: groups.mode.clone(), class: "app-w-auto", options: opts_with_none("—", vocab::GROUP_MODES), on_change: move |v: String| edit(pid, "", move |p| with_b(p, bid, |b| { let mut gr = b.groups.clone().unwrap_or_default(); gr.mode = v; b.groups = Some(gr) })) }
                                        span { class: "app-small", "of" }
                                        NumIn { value: groups.team_size.unwrap_or(3) as f64, min: 1.0, max: 5.0, class: "app-w-num", on_change: move |v: f64| edit(pid, "tsize", move |p| with_b(p, bid, |b| { let mut gr = b.groups.clone().unwrap_or_default(); gr.team_size = Some(v.clamp(1.0, 5.0) as u32); b.groups = Some(gr) })) }
                                        Check { value: b.protected, label: "protected".to_string(), on_change: move |v: bool| edit(pid, "", move |p| with_b(p, bid, |b| { b.protected = v; b.min_minutes = v.then_some(b.minutes) })) }
                                        span { class: if low { "app-tag app-tag-warn" } else { "app-tag" }, "{ratio}" }
                                    }
                                    TextIn { value: b.prompt.clone().unwrap_or_default(), placeholder: "Question for the TV during this block (optional)".to_string(), class: "app-small",
                                        on_change: move |v: String| edit(pid, "prompt", move |p| with_b(p, bid, |b| b.prompt = (!v.is_empty()).then_some(v))) }
                                }
                                div { class: "app-row-tools app-col",
                                    button { class: "app-icon-btn app-sm", title: "Earlier", disabled: i == 0, onclick: move |e| { e.stop_propagation(); edit(pid, "", |p| if i > 0 { p.blocks.swap(i, i - 1) }) }, "↑" }
                                    button { class: "app-icon-btn app-sm", title: "Later", disabled: i == last, onclick: move |e| { e.stop_propagation(); edit(pid, "", |p| if i + 1 < p.blocks.len() { p.blocks.swap(i, i + 1) }) }, "↓" }
                                    button { class: "app-icon-btn app-sm", title: "Duplicate block", onclick: move |e| { e.stop_propagation(); edit(pid, "", |p| { if let Some(b) = p.blocks.get(i).cloned() { let mut c = b; c.id = p.fresh_block_id(); p.blocks.insert(i + 1, c); } }) }, "⧉" }
                                    button { class: "app-icon-btn app-sm", title: "Remove block", onclick: move |e| { e.stop_propagation(); edit(pid, "", |p| { if i < p.blocks.len() { p.blocks.remove(i); } }) }, "×" }
                                }
                            }
                        }
                    }
                }
                button { class: "app-btn", onclick: move |_| edit(pid, "", |p| { let id = p.fresh_block_id(); p.blocks.push(Block { id, name: "New block".into(), minutes: 10.0, ..Default::default() }); }), "+ Add block" }
            }
        }
    }
}

// ---------------------------------------------------------------------------------------
// left: games to add
// ---------------------------------------------------------------------------------------

/// The shell reuses one scope for whatever view sits in a slot (a lone keyed child),
/// so this wrapper has no hooks and renders a distinct component type.
#[allow(non_snake_case)]
fn GamesPane() -> Element {
    rsx! { GamesPaneBody {} }
}

#[component]
fn GamesPaneBody() -> Element {
    let Some(pid) = use_practice_id() else {
        return rsx! {};
    };
    let mut filter = use_signal(String::new);
    let games = games_by_id();
    let mut list: Vec<Game> = games.into_values().collect();
    list.sort_by_key(|g| g.name.to_lowercase());
    let f = filter.read().to_lowercase();
    rsx! {
        div { class: "app-pane",
            input { class: "app-input", r#type: "search", placeholder: "Find a game or a target", value: "{filter}", oninput: move |e| filter.set(e.value()) }
            for g in list.into_iter().filter(|g| f.is_empty() || g.name.to_lowercase().contains(&f) || g.targets().iter().any(|t| vocab::target_label(t).to_lowercase().contains(&f))) {
                {
                    let g2 = g.clone();
                    let targets: Vec<String> = g.targets().iter().take(2).map(|t| vocab::target_label(t).to_string()).collect();
                    rsx! {
                        div { key: "{g.id}", class: "app-lib-item",
                            div { class: "app-lib-head",
                                span { class: "app-row-title", "{g.name}" }
                                button { class: "app-btn app-sm", onclick: move |_| edit(pid, "", |p| { let b = new_block(p, &g2); p.blocks.push(b) }), "Add" }
                            }
                            div { class: "app-row-sub", "{g.numbers()} · {vocab::region_label(&g.format.space.court)} · {g.stage}" }
                            if !targets.is_empty() { div { class: "app-row-sub", "{targets.join(\" · \")}" } }
                        }
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------------------
// right: metrics
// ---------------------------------------------------------------------------------------

struct BlockLoad {
    band: f64,
    load: f64,
}

fn block_loads(
    p: &Practice,
    games: &HashMap<String, Game>,
    m: &practice::PracticeMetrics,
) -> Vec<Option<BlockLoad>> {
    let c = court::preset(&crate::settings::SETTINGS.read().court);
    p.blocks
        .iter()
        .zip(m.blocks.iter())
        .map(|(b, bm)| {
            let g = games.get(&b.game)?;
            let rg = super::game::resolved(g, None);
            let band = estimates::intensity(&c, &rg, Some(b.minutes * 60.0)).value;
            Some(BlockLoad {
                band,
                load: estimates::load_units(bm.active_minutes, band),
            })
        })
        .collect()
}

/// The shell reuses one scope for whatever view sits in a slot (a lone keyed child),
/// so this wrapper has no hooks and renders a distinct component type.
#[allow(non_snake_case)]
fn MetricsPane() -> Element {
    rsx! { MetricsPaneBody {} }
}

#[component]
fn MetricsPaneBody() -> Element {
    let Some(pid) = use_practice_id() else {
        return rsx! {};
    };
    let Some((p, _)) = snapshot(pid) else {
        return rsx! {};
    };
    let games = games_by_id();
    let lookup = |id: &str| block_game(&games, id);
    let m = practice::metrics(&p, &lookup);
    let loads = block_loads(&p, &games, &m);
    let total_load: f64 = loads.iter().flatten().map(|l| l.load).sum::<f64>() + 0.0;
    let runs = practice::interleaving_runs(&p, &lookup, 3);
    let performance = p
        .blocks
        .iter()
        .filter_map(|b| games.get(&b.game))
        .any(|g| g.stage == "performance");
    // target mix by principle, minutes
    let mut mix: Vec<(String, f64)> = Vec::new();
    for b in &p.blocks {
        if let Some(g) = games.get(&b.game) {
            let mut ps: Vec<String> = g.targets().iter().map(|t| practice::principle(t)).collect();
            ps.dedup();
            for pr in ps {
                match mix.iter_mut().find(|x| x.0 == pr) {
                    Some(x) => x.1 += b.minutes,
                    None => mix.push((pr, b.minutes)),
                }
            }
        }
    }
    mix.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    let mix_max = mix.first().map(|x| x.1).unwrap_or(1.0).max(1.0);
    let players = p.player_count();
    let band_names = ["", "light", "moderate", "hard", "very hard", "max"];
    rsx! {
        div { class: "app-pane",
            Section { title: "Time (R10)".to_string(),
                table { class: "app-table",
                    tr { td { "Blocks" } td { class: "app-num-cell", "{m.block_minutes} min" } }
                    tr { td { "Changeovers" } td { class: "app-num-cell", "{m.transition_minutes:.0} min" } }
                    tr { td { "Planned" } td { class: "app-num-cell", "{m.planned_total:.0} / {p.planned_minutes} min" } }
                    tr { td { "Left over" } td { class: if m.unassigned < -0.01 { "app-num-cell app-err-text" } else { "app-num-cell" }, "{m.unassigned:.0} min" } }
                }
            }
            Section { title: "Active time per player (R11)".to_string(),
                if players == 0 { p { class: "app-muted", "Set the number of players to see active time." } }
                for (i, b) in p.blocks.iter().enumerate() {
                    {
                        let bm = &m.blocks[i];
                        let pct = bm.active_ratio.unwrap_or(0.0) * 100.0;
                        let low = bm.active_ratio.is_some_and(|r| r < practice::TARGET_ACTIVE_RATIO);
                        rsx! {
                            div { key: "{b.id}", class: "app-bar-row",
                                span { class: "app-bar-label", "{b.name}" }
                                div { class: "app-bar", div { class: if low { "app-bar-fill app-bar-low" } else { "app-bar-fill" }, style: "width: {pct:.0}%" } }
                                span { class: "app-bar-val", if bm.active_ratio.is_some() { "{pct:.0}%" } else { "–" } }
                            }
                        }
                    }
                }
                p { class: "app-small", "≈ {m.active_minutes_per_player:.0} active minutes per player out of {m.block_minutes} min of blocks." }
                if !m.low_active.is_empty() {
                    p { class: "app-warn", "Under 60% active: add a basket or a rotation for those blocks." }
                }
            }
            Section { title: "Estimated load (S9 §6)".to_string(),
                for (i, b) in p.blocks.iter().enumerate() {
                    if let Some(l) = &loads[i] {
                        div { key: "{b.id}", class: "app-bar-row",
                            span { class: "app-bar-label", "{b.name}" }
                            span { class: "app-tag", "{band_names[l.band as usize]}" }
                            span { class: "app-bar-val", "{l.load:.0} units" }
                        }
                    }
                }
                p { class: "app-small", "Session ≈ {total_load:.0} load units per player (active minutes × RPE guess). Low confidence until you log session RPE." }
            }
            Section { title: "What it trains (R12)".to_string(),
                if mix.is_empty() { p { class: "app-muted", "Tag the games' constraints with learning targets." } }
                for (pr, mins) in mix.iter() {
                    div { key: "{pr}", class: "app-bar-row",
                        span { class: "app-bar-label", "{vocab::target_path_label(pr)}" }
                        div { class: "app-bar", div { class: "app-bar-fill app-bar-target", style: "width: {mins / mix_max * 100.0:.0}%" } }
                        span { class: "app-bar-val", "{mins} min" }
                    }
                }
                if !performance {
                    for (pr, from, len) in runs.iter() {
                        p { key: "{pr}", class: "app-warn", "{vocab::target_path_label(pr)} fills {len} blocks in a row from {from}: mix in something else (interleave)." }
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
    let Some(pid) = use_practice_id() else {
        return rsx! {};
    };
    let Some((p, _)) = snapshot(pid) else {
        return rsx! {};
    };
    let games = games_by_id();
    let lookup = |id: &str| block_game(&games, id);
    let m = practice::metrics(&p, &lookup);
    rsx! { span { "{p.planned_minutes} min · {p.blocks.len()} blocks · {m.unassigned:.0} min unassigned · ≈{m.active_minutes_per_player:.0} active min/player" } }
}

// ---------------------------------------------------------------------------------------
// printable plan
// ---------------------------------------------------------------------------------------

pub fn plan_html(p: &Practice) -> String {
    let games = games_by_id();
    let lookup = |id: &str| block_game(&games, id);
    let m = practice::metrics(p, &lookup);
    let esc = super::esc;
    let mut rows = String::new();
    for (b, bm) in p.blocks.iter().zip(m.blocks.iter()) {
        let g = games.get(&b.game);
        let groups = b
            .groups
            .as_ref()
            .map(|g| format!("{} of {}", g.mode, g.team_size.unwrap_or(0)))
            .unwrap_or_default();
        rows.push_str(&format!(
            "<tr><td>{}–{}</td><td><b>{}</b>{}<br><span class=\"muted\">{}</span></td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
            practice::clock_time(&p.start, bm.starts_at),
            practice::clock_time(&p.start, bm.starts_at + b.minutes),
            esc(&b.name),
            if b.protected { " 🔒" } else { "" },
            esc(&g.map(|g| format!("{} · {}", g.name, g.numbers())).unwrap_or_default()),
            b.minutes,
            esc(&b.baskets.join(", ")),
            esc(&groups),
            esc(b.prompt.as_deref().unwrap_or("")),
        ));
    }
    let mut seen = Vec::new();
    let mut cards = Vec::new();
    for b in &p.blocks {
        if let Some(g) = games.get(&b.game) {
            if !seen.contains(&g.id) {
                seen.push(g.id.clone());
                cards.push((g.clone(), None));
            }
        }
    }
    let preset = crate::settings::SETTINGS.read().court.clone();
    let card_html: String = cards
        .iter()
        .map(|(g, r)| super::game::card_html(g, *r, &preset))
        .collect();
    format!(
        "<!doctype html><html><head><meta charset=\"utf-8\"><title>{title}</title><style>{css}</style></head><body>\
<h1>{title}</h1><p class=\"sub\">{date} · start {start} · {mins} min · {players} players{theme}</p>\
<table><tr><th>Time</th><th>Block</th><th>Min</th><th>Baskets</th><th>Groups</th><th>Ask</th></tr>{rows}</table>\
<p class=\"muted\">Blocks {bmin} min + changeovers {tmin:.0} min · {un:.0} min unassigned · ≈{act:.0} active min per player</p>\
<h2>Game cards</h2>{card_html}<p class=\"noprint muted\">Print with Ctrl+P.</p></body></html>",
        title = esc(&p.title()),
        css = super::PRINT_CSS,
        date = esc(&p.date),
        start = esc(&p.start),
        mins = p.planned_minutes,
        players = p.player_count(),
        theme = if p.theme.is_empty() { String::new() } else { format!(" · {}", esc(&p.theme)) },
        bmin = m.block_minutes,
        tmin = m.transition_minutes,
        un = m.unassigned,
        act = m.active_minutes_per_player,
    )
}
