//! Doc type **Game** — the small-sided-game designer (03 §3): base format + constraint
//! stack (suppress, reorder, rollback), structured rules over events, dials, zones, a live
//! game card, predicted effects and the coaching-language validator.

use std::collections::HashMap;

use ac_domain::store::{Doc, DocKind};
use ac_domain::{
    resolve, Consequence, Constraint, Dial, DialKind, Game, Issue, Judge, OneOrMany,
    ResolveOptions, ResolvedGame, RoleEdit, Rule, Severity, Shape, Trigger, TriggerKind, Val, Zone,
    CATEGORIES, STAGES, ZONE_KINDS,
};
use ac_sport_basketball::{court, estimates, render, templates, vocab, BasketballPack};
use app_shell::prelude::*;
use dioxus::prelude::*;

use super::{Editing, Map};
use crate::court::{formation, CourtView, PlayerDraw, ZoneDraw};
use crate::intern;
use crate::ui::*;

pub const ID: &str = "game";

#[derive(Clone, Debug, PartialEq)]
pub enum Sel {
    Format,
    Constraint(&'static str),
    Rule(&'static str, &'static str),
}

pub struct GameDoc {
    pub ed: Editing<Game>,
    pub sel: Sel,
    pub rollback: Option<usize>,
}

pub static GAMES: Map<GameDoc> = Signal::global(HashMap::new);

// ---------------------------------------------------------------------------------------
// state helpers
// ---------------------------------------------------------------------------------------

fn ensure(id: &str) {
    if GAMES.peek().contains_key(id) {
        return;
    }
    let g = crate::store::load_game(id).unwrap_or_else(|| {
        let g = templates::new_game(id);
        crate::store::save(Doc::Game(g.clone()));
        g
    });
    GAMES.write().insert(
        id.to_string(),
        GameDoc {
            ed: Editing::new(g),
            sel: Sel::Format,
            rollback: None,
        },
    );
}

/// The document id for this view (adopting shell-made ids), loaded and ready.
/// `in_viewport`: keep-alive viewports know their own document (`use_document`); panels
/// and the status bar follow the active document, because the shell reuses their scope
/// and its document context goes stale after a switch.
fn use_game_id_in(in_viewport: bool) -> Option<&'static str> {
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
    if let Some(new) = super::adopt_shell_id(shell, DocKind::Game, &doc.id) {
        ensure(&new);
        if let Some(name) = snapshot(&new).map(|s| s.0.name) {
            super::rename(shell, &new, &name);
        }
        return Some(intern(&new));
    }
    ensure(&doc.id);
    Some(intern(&doc.id))
}

fn use_game_id() -> Option<&'static str> {
    use_game_id_in(false)
}

pub fn snapshot(id: &str) -> Option<(Game, Sel, Option<usize>)> {
    GAMES
        .read()
        .get(id)
        .map(|d| (d.ed.value.clone(), d.sel.clone(), d.rollback))
}

pub fn edit(id: &str, key: &str, f: impl FnOnce(&mut Game)) {
    let saved = {
        let mut all = GAMES.write();
        let Some(d) = all.get_mut(id) else { return };
        if d.ed.edit(key, f) {
            Some(d.ed.value.clone())
        } else {
            None
        }
    };
    if let Some(g) = saved {
        crate::store::save(Doc::Game(g));
    }
}

fn select(id: &str, sel: Sel) {
    if let Some(d) = GAMES.write().get_mut(id) {
        d.sel = sel;
    }
}

fn set_rollback(id: &str, r: Option<usize>) {
    if let Some(d) = GAMES.write().get_mut(id) {
        d.rollback = r;
    }
}

fn with_c(g: &mut Game, cid: &str, f: impl FnOnce(&mut Constraint)) {
    if let Some(c) = g
        .constraints
        .iter_mut()
        .chain(g.pool.iter_mut())
        .find(|c| c.id == cid)
    {
        f(c)
    }
}

fn with_r(g: &mut Game, cid: &str, rid: &str, f: impl FnOnce(&mut Rule)) {
    with_c(g, cid, |c| {
        if let Some(r) = c.rules.iter_mut().find(|r| r.id == rid) {
            f(r)
        }
    })
}

pub fn resolved(g: &Game, rollback: Option<usize>) -> ResolvedGame {
    resolve(
        g,
        &BasketballPack,
        &ResolveOptions {
            rollback,
            ..Default::default()
        },
    )
}

/// A rule's card sentence with its dials bound (from the resolved game when it's active).
fn sentence_in(rg: &ResolvedGame, g: &Game, cid: &str, r: &Rule) -> String {
    if let Some(rr) = rg.rule(&r.id) {
        return render::sentence(&rr.rule)
            .unwrap_or_else(|| format!("{} (not on the card)", rr.rule.display_label()));
    }
    // suppressed / rolled back / pool: bind it on its own
    let c = g
        .constraints
        .iter()
        .chain(g.pool.iter())
        .find(|c| c.id == cid)
        .cloned();
    match c {
        Some(c) => preview_sentences(g, &c)
            .into_iter()
            .find(|(id, _)| *id == r.id)
            .map(|(_, s)| s)
            .unwrap_or_else(|| r.display_label()),
        None => r.display_label(),
    }
}

/// Every rule of a constraint rendered with its dial defaults, as if it were alone in `g`.
pub fn preview_sentences(g: &Game, c: &Constraint) -> Vec<(String, String)> {
    let mut tmp = g.clone();
    tmp.constraints = vec![c.clone()];
    tmp.suppressed.clear();
    tmp.pool.clear();
    let rg = resolved(&tmp, None);
    rg.rules
        .iter()
        .filter(|r| !r.is_base())
        .map(|r| {
            (
                r.rule.id.clone(),
                render::sentence(&r.rule)
                    .unwrap_or_else(|| format!("{} (not on the card)", r.rule.display_label())),
            )
        })
        .collect()
}

fn court_preset() -> String {
    crate::settings::SETTINGS.read().court.clone()
}

// ---------------------------------------------------------------------------------------
// doc type + commands
// ---------------------------------------------------------------------------------------

pub fn doc_type() -> DocType {
    let mut t = DocType::new(ID, "Game", icons::TARGET, Viewport);
    t.toolbar = vec![
        ToolGroup::new(
            "Edit",
            vec![
                Tool::new("gm.undo", "Undo", icons::UNDO).key("Ctrl+Z"),
                Tool::new("gm.redo", "Redo", icons::REDO).key("Ctrl+Y"),
            ],
        ),
        ToolGroup::new(
            "Constraints",
            vec![
                Tool::new("gm.add", "New constraint", icons::PLUS),
                Tool::new("gm.library", "Constraint library", icons::LAYERS),
            ],
        ),
        ToolGroup::new(
            "Game",
            vec![
                Tool::new("gm.duplicate", "Duplicate as a new game", icons::FILES),
                Tool::new("gm.print", "Print game card", icons::DOWNLOAD).primary(),
                Tool::new("gm.yaml", "Export YAML", icons::SHARE),
            ],
        ),
    ];
    t.left_rail = vec![
        RailItem::panel("gm.stack", "Constraint stack", icons::TREE, StackPane),
        RailItem::panel("gm.lib", "Constraint library", icons::LAYERS, LibraryPane),
    ];
    t.right_rail = vec![
        RailItem::panel("gm.props", "Properties", icons::INFO, PropertiesPane),
        RailItem::panel("gm.checks", "Checks & effects", icons::SCALE, ChecksPane),
    ];
    t.status = Some(View(Status));
    t.default_left = Some("gm.stack");
    t.default_right = Some("gm.props");
    t
}

pub fn handle(mut shell: Shell, doc: &str, id: &'static str) {
    match id {
        "gm.undo" | "gm.redo" => {
            let saved = {
                let mut all = GAMES.write();
                let Some(d) = all.get_mut(doc) else { return };
                let ok = if id == "gm.undo" {
                    d.ed.undo()
                } else {
                    d.ed.redo()
                };
                ok.then(|| d.ed.value.clone())
            };
            if let Some(g) = saved {
                super::rename(shell, doc, &g.name);
                crate::store::save(Doc::Game(g));
            }
        }
        "gm.add" => {
            let mut new_id = String::new();
            edit(doc, "", |g| {
                let c = templates::blank_constraint(g);
                new_id = c.id.clone();
                g.constraints.push(c);
            });
            select(doc, Sel::Constraint(intern(&new_id)));
            shell.set_panel(Placement::Right, Some("gm.props"));
        }
        "gm.library" => shell.set_panel(Placement::Left, Some("gm.lib")),
        "gm.duplicate" => {
            let Some((g, _, _)) = snapshot(doc) else {
                return;
            };
            let nid = crate::store::new_id(DocKind::Game);
            let mut copy = g.clone();
            copy.id = nid.clone();
            copy.name = format!("{} (copy)", g.name);
            crate::store::save(Doc::Game(copy.clone()));
            super::open(shell, DocKind::Game, &nid, &copy.name);
        }
        "gm.print" => {
            if let Some((g, _, rb)) = snapshot(doc) {
                let html = card_page(&[(g.clone(), rb)]);
                super::export_html(&g.name, html);
            }
        }
        "gm.yaml" => {
            if let Some((g, _, _)) = snapshot(doc) {
                super::export_yaml(DocKind::Game, &Doc::Game(g));
            }
        }
        _ => {}
    }
}

// ---------------------------------------------------------------------------------------
// viewport: court + live card
// ---------------------------------------------------------------------------------------

pub fn zone_draws(rg: &ResolvedGame, preset: &str, selected: Option<&str>) -> Vec<ZoneDraw> {
    let c = court::preset(preset);
    rg.zones
        .iter()
        .filter_map(|z| {
            let geom = court::shape_geom(&c, &z.zone.shape, &rg.format.space.court)?;
            Some(ZoneDraw {
                id: z.constraint.clone(),
                label: z
                    .zone
                    .label
                    .clone()
                    .unwrap_or_else(|| z.zone.kind.replace('-', " ")),
                kind: z.zone.kind.clone(),
                geom,
                selected: selected == Some(z.constraint.as_str()),
            })
        })
        .collect()
}

pub fn player_draws(rg: &ResolvedGame) -> Vec<PlayerDraw> {
    formation(&rg.roles)
        .into_iter()
        .map(|(id, team, at)| PlayerDraw {
            id,
            at,
            team,
            ball: false,
            selected: false,
        })
        .collect()
}

#[component]
pub fn CardView(card: render::GameCard, buttons: Vec<String>, calls: Vec<String>) -> Element {
    rsx! {
        div { class: "app-card",
            h2 { class: "app-card-title", "{card.title}" }
            p { class: "app-muted", "{card.subtitle}" }
            if !card.how_to_score.is_empty() {
                h4 { "How to score" }
                ul { for (i, l) in card.how_to_score.iter().enumerate() { li { key: "{i}", "{l}" } } }
            }
            if !card.rules.is_empty() {
                h4 { "Rules" }
                ul { for (i, l) in card.rules.iter().enumerate() { li { key: "{i}", "{l}" } } }
            }
            if !card.how_it_runs.is_empty() {
                h4 { "How it runs" }
                ul { for (i, l) in card.how_it_runs.iter().enumerate() { li { key: "{i}", "{l}" } } }
            }
            if !card.questions.is_empty() {
                h4 { "Ask" }
                ul { class: "app-questions", for (i, l) in card.questions.iter().enumerate() { li { key: "{i}", "{l}" } } }
            }
            div { class: "app-card-foot",
                if !card.board.is_empty() {
                    div { span { class: "app-tag", "TV" } span { class: "app-mono", " {card.board}" } }
                }
                if !buttons.is_empty() {
                    div {
                        span { class: "app-tag", "Phone" }
                        for (i, b) in buttons.iter().enumerate() { span { key: "{i}", class: "app-btn-preview", "{b}" } }
                        for (i, b) in calls.iter().enumerate() { span { key: "c{i}", class: "app-btn-preview app-btn-call", "{b}" } }
                    }
                }
            }
        }
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
    let Some(gid) = use_game_id_in(true) else {
        return rsx! {};
    };
    let Some((g, sel, rollback)) = snapshot(gid) else {
        return rsx! {};
    };
    let preset = court_preset();
    let rg = resolved(&g, rollback);
    let selected_c = match &sel {
        Sel::Constraint(c) | Sel::Rule(c, _) => Some(*c),
        Sel::Format => None,
    };
    let zones = zone_draws(&rg, &preset, selected_c);
    let players = player_draws(&rg);
    let card = render::game_card(&g, &rg);
    let buttons = render::buttons(&rg);
    let calls = render::calls(&rg);
    let n_active = g
        .constraints
        .iter()
        .filter(|c| !g.is_suppressed(&c.id))
        .count();
    let caption = render::start_text(&rg).unwrap_or_default();
    rsx! {
        div { class: "app-game",
            if let Some(n) = rollback {
                div { class: "app-banner",
                    "Rolled back: showing the game with the first {n} of {n_active} constraints. "
                    button { class: "app-link", onclick: move |_| set_rollback(gid, None), "Show all" }
                }
            }
            div { class: "app-game-grid",
                CourtView {
                    preset: preset.clone(),
                    region: rg.format.space.court.clone(),
                    zones,
                    players,
                    arrows: vec![],
                    caption,
                    on_zone: move |cid: String| select(gid, Sel::Constraint(intern(&cid))),
                }
                CardView { card, buttons, calls }
            }
        }
    }
}

// ---------------------------------------------------------------------------------------
// left: constraint stack
// ---------------------------------------------------------------------------------------

fn cat_class(cat: &str) -> String {
    format!(
        "app-cat app-cat-{}",
        if cat.is_empty() { "task" } else { cat }
    )
}

/// The shell reuses one scope for whatever view sits in a slot (a lone keyed child),
/// so this wrapper has no hooks and renders a distinct component type.
#[allow(non_snake_case)]
fn StackPane() -> Element {
    rsx! { StackPaneBody {} }
}

#[component]
fn StackPaneBody() -> Element {
    let Some(gid) = use_game_id() else {
        return rsx! {};
    };
    let Some((g, sel, rollback)) = snapshot(gid) else {
        return rsx! {};
    };
    let rg = resolved(&g, rollback);
    let errors_for = |cid: &str| {
        rg.issues
            .iter()
            .filter(|i| i.constraint.as_deref() == Some(cid) && i.severity == Severity::Error)
            .count()
    };
    let n = g
        .constraints
        .iter()
        .filter(|c| !g.is_suppressed(&c.id))
        .count();
    let rb_value = rollback.unwrap_or(n);
    let targets = g.targets();
    let format_sel = sel == Sel::Format;
    rsx! {
        div { class: "app-pane",
            div {
                class: if format_sel { "app-row app-row-sel" } else { "app-row" },
                onclick: move |_| select(gid, Sel::Format),
                span { class: "app-cat app-cat-base", "■" }
                div { class: "app-row-main",
                    div { class: "app-row-title", "Base game" }
                    div { class: "app-row-sub", "{g.numbers()} · {vocab::region_label(&g.format.space.court)} · {g.stage}" }
                }
            }
            Section { title: "Constraint stack".to_string(),
                actions: rsx! {
                    button { class: "app-icon-btn", title: "New constraint", onclick: move |_| {
                        let mut new_id = String::new();
                        edit(gid, "", |g| { let c = templates::blank_constraint(g); new_id = c.id.clone(); g.constraints.push(c); });
                        select(gid, Sel::Constraint(intern(&new_id)));
                    }, "+" }
                },
                if g.constraints.is_empty() {
                    Empty { text: "No constraints yet. Add one with +, or from the constraint library." }
                }
                for (i, c) in g.constraints.iter().enumerate() {
                    {
                        let cid = intern(&c.id);
                        let on = !g.is_suppressed(&c.id);
                        let is_sel = matches!(&sel, Sel::Constraint(x) | Sel::Rule(x, _) if *x == cid);
                        let errs = errors_for(&c.id);
                        let last = g.constraints.len() - 1;
                        let rolled = rollback.is_some_and(|r| {
                            let pos = g.constraints.iter().take(i + 1).filter(|c| !g.is_suppressed(&c.id)).count();
                            on && pos > r
                        });
                        rsx! {
                            div { key: "{cid}",
                                div {
                                    class: if is_sel { "app-row app-row-sel" } else if !on || rolled { "app-row app-row-off" } else { "app-row" },
                                    onclick: move |_| select(gid, Sel::Constraint(cid)),
                                    input {
                                        r#type: "checkbox",
                                        title: "Use this constraint",
                                        checked: on,
                                        onclick: move |e| e.stop_propagation(),
                                        onchange: move |e| {
                                            let v = e.checked();
                                            edit(gid, "", |g| {
                                                g.suppressed.retain(|s| s != cid);
                                                if !v { g.suppressed.push(cid.to_string()); }
                                            });
                                        },
                                    }
                                    span { class: cat_class(&c.category), title: "{c.category}", "●" }
                                    div { class: "app-row-main",
                                        div { class: "app-row-title", "{c.name}" }
                                        div { class: "app-row-sub",
                                            {
                                                let n = c.rules.len() + c.roles.iter().filter(|e| e.restrict.is_some()).count();
                                                if n == 1 { "1 rule".to_string() } else { format!("{n} rules") }
                                            }
                                            if errs > 0 { span { class: "app-err-count", " · {errs} to fix" } }
                                        }
                                    }
                                    div { class: "app-row-tools",
                                        button { class: "app-icon-btn app-sm", title: "Move up", disabled: i == 0,
                                            onclick: move |e| { e.stop_propagation(); edit(gid, "", |g| { if i > 0 { g.constraints.swap(i, i - 1) } }) }, "↑" }
                                        button { class: "app-icon-btn app-sm", title: "Move down", disabled: i == last,
                                            onclick: move |e| { e.stop_propagation(); edit(gid, "", |g| { if i + 1 < g.constraints.len() { g.constraints.swap(i, i + 1) } }) }, "↓" }
                                        button { class: "app-icon-btn app-sm", title: "Delete constraint",
                                            onclick: move |e| { e.stop_propagation(); edit(gid, "", |g| { g.constraints.retain(|c| c.id != cid); g.suppressed.retain(|s| s != cid); }); select(gid, Sel::Format); }, "×" }
                                    }
                                }
                                if is_sel {
                                    div { class: "app-subrows",
                                        for r in c.rules.iter() {
                                            {
                                                let rid = intern(&r.id);
                                                let rsel = matches!(&sel, Sel::Rule(_, x) if *x == rid);
                                                let text = sentence_in(&rg, &g, cid, r);
                                                rsx! {
                                                    div { key: "{rid}",
                                                        class: if rsel { "app-subrow app-row-sel" } else { "app-subrow" },
                                                        onclick: move |_| select(gid, Sel::Rule(cid, rid)),
                                                        "{text}"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            if n > 0 {
                div { class: "app-rollback",
                    span { class: "app-field-label", "Rollback: show the game with the first {rb_value} of {n} constraints" }
                    input {
                        r#type: "range", min: "0", max: "{n}", value: "{rb_value}",
                        oninput: move |e| {
                            let v: usize = e.value().parse().unwrap_or(n);
                            set_rollback(gid, if v >= n { None } else { Some(v) });
                        },
                    }
                }
            }
            if !g.pool.is_empty() {
                Section { title: "Pool (picked during play)".to_string(),
                    for c in g.pool.iter() {
                        {
                            let cid = intern(&c.id);
                            let is_sel = matches!(&sel, Sel::Constraint(x) | Sel::Rule(x, _) if *x == cid);
                            rsx! {
                                div { key: "{cid}", class: if is_sel { "app-row app-row-sel" } else { "app-row" },
                                    onclick: move |_| select(gid, Sel::Constraint(cid)),
                                    span { class: cat_class(&c.category), "●" }
                                    div { class: "app-row-main", div { class: "app-row-title", "{c.name}" } }
                                }
                            }
                        }
                    }
                }
            }
            Section { title: "Trains".to_string(),
                if targets.is_empty() { Empty { text: "Tag constraints with learning targets." } }
                ul { class: "app-plain",
                    for t in targets.iter() { li { key: "{t}", "{vocab::target_path_label(t)}" } }
                }
                if !g.reads.is_empty() {
                    p { class: "app-field-label", "Reads it invites" }
                    ul { class: "app-plain",
                        for r in g.reads.iter() { li { key: "{r}", "{vocab::read_label(r)}" } }
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------------------
// left: constraint library
// ---------------------------------------------------------------------------------------

/// The shell reuses one scope for whatever view sits in a slot (a lone keyed child),
/// so this wrapper has no hooks and renders a distinct component type.
#[allow(non_snake_case)]
fn LibraryPane() -> Element {
    rsx! { LibraryPaneBody {} }
}

#[component]
fn LibraryPaneBody() -> Element {
    let Some(gid) = use_game_id() else {
        return rsx! {};
    };
    let mut filter = use_signal(String::new);
    let _rev = crate::store::REVISION.read();
    let mut seen: Vec<String> = Vec::new();
    let mut items: Vec<(String, Constraint)> = Vec::new();
    for c in templates::starter_constraints() {
        seen.push(c.name.to_lowercase());
        items.push(("Starter".into(), c));
    }
    for other in crate::store::all_games() {
        if other.id == gid {
            continue;
        }
        for c in other.constraints.iter().chain(other.pool.iter()) {
            let key = c.name.to_lowercase();
            if !seen.contains(&key) && !c.name.is_empty() {
                seen.push(key);
                items.push((other.name.clone(), c.clone()));
            }
        }
    }
    let base = snapshot(gid).map(|s| s.0).unwrap_or_default();
    let f = filter.read().to_lowercase();
    let shown: Vec<(String, Constraint)> = items
        .into_iter()
        .filter(|(from, c)| {
            f.is_empty()
                || c.name.to_lowercase().contains(&f)
                || from.to_lowercase().contains(&f)
                || c.targets
                    .iter()
                    .any(|t| vocab::target_label(t).to_lowercase().contains(&f))
                || c.invites.iter().any(|t| t.contains(&f))
        })
        .collect();
    rsx! {
        div { class: "app-pane",
            p { class: "app-muted", "Constraints from the starter set and from your other games. Adding one copies it into this game." }
            input { class: "app-input", r#type: "search", placeholder: "Filter: name, target, invites…", value: "{filter}", oninput: move |e| filter.set(e.value()) }
            for (i, (from, c)) in shown.iter().enumerate() {
                {
                    let c2 = c.clone();
                    let sentences: Vec<String> = preview_sentences(&base, c).into_iter().map(|(_, s)| s).collect();
                    rsx! {
                        div { key: "{i}", class: "app-lib-item",
                            div { class: "app-lib-head",
                                span { class: cat_class(&c.category), "●" }
                                span { class: "app-row-title", "{c.name}" }
                                button { class: "app-btn app-sm", onclick: move |_| {
                                    let mut new_id = String::new();
                                    edit(gid, "", |g| { let a = templates::adopt(g, &c2); new_id = a.id.clone(); g.constraints.push(a); });
                                    select(gid, Sel::Constraint(intern(&new_id)));
                                }, "Add" }
                            }
                            div { class: "app-row-sub", "{from}" }
                            ul { class: "app-plain app-small", for (k, s) in sentences.iter().enumerate() { li { key: "{k}", "{s}" } } }
                            if !c.targets.is_empty() {
                                div { class: "app-row-sub", "Trains: {c.targets.iter().map(|t| vocab::target_label(t)).collect::<Vec<_>>().join(\", \")}" }
                            }
                        }
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------------------
// right: properties
// ---------------------------------------------------------------------------------------

/// The shell reuses one scope for whatever view sits in a slot (a lone keyed child),
/// so this wrapper has no hooks and renders a distinct component type.
#[allow(non_snake_case)]
fn PropertiesPane() -> Element {
    rsx! { PropertiesPaneBody {} }
}

#[component]
fn PropertiesPaneBody() -> Element {
    let Some(gid) = use_game_id() else {
        return rsx! {};
    };
    let Some((g, sel, _)) = snapshot(gid) else {
        return rsx! {};
    };
    match sel {
        Sel::Format => rsx! { FormatEditor { gid, game: g } },
        Sel::Constraint(cid) => {
            if g.constraints
                .iter()
                .chain(g.pool.iter())
                .any(|c| c.id == cid)
            {
                rsx! { ConstraintEditor { gid, cid, game: g } }
            } else {
                rsx! { FormatEditor { gid, game: g } }
            }
        }
        Sel::Rule(cid, rid) => rsx! { RuleEditor { gid, cid, rid, game: g } },
    }
}

fn set_counts(g: &mut Game, o: usize, d: usize, neutral: bool) {
    let keep =
        |team: &str, n: usize, roles: &[ac_domain::Role], prefix: &str| -> Vec<ac_domain::Role> {
            let mut v: Vec<ac_domain::Role> = roles
                .iter()
                .filter(|r| r.team == team)
                .take(n)
                .cloned()
                .collect();
            let mut k = 1;
            while v.len() < n {
                let id = format!("{prefix}{k}");
                if !roles.iter().any(|r| r.id == id) && !v.iter().any(|r| r.id == id) {
                    v.push(ac_domain::Role::new(&id, team));
                }
                k += 1;
            }
            v
        };
    let roles = g.format.roles.clone();
    let mut out = keep("offense", o, &roles, "O");
    out.extend(keep("defense", d, &roles, "D"));
    if neutral {
        let j = roles
            .iter()
            .find(|r| r.team == "neutral")
            .cloned()
            .unwrap_or_else(|| {
                let mut r = ac_domain::Role::new("J", "neutral");
                r.plays_for = Some("offense".into());
                r
            });
        out.push(j);
    }
    g.format.roles = out;
    g.format.numbers = None;
}

/// A number-or-dial value editor (S5 §4).
#[component]
fn ValEdit(
    value: Option<Val>,
    dials: Vec<(String, String)>,
    on_change: EventHandler<Option<Val>>,
    #[props(default)] allow_none: bool,
) -> Element {
    let is_ref = value
        .as_ref()
        .and_then(|v| v.dial_ref())
        .map(str::to_string);
    let mut options: Vec<(String, String)> = Vec::new();
    if allow_none {
        options.push(("".into(), "—".into()));
    }
    options.push(("#".into(), "number".into()));
    for (k, l) in &dials {
        options.push((format!("${k}"), format!("dial: {l}")));
    }
    let mode = match (&value, &is_ref) {
        (None, _) => "".to_string(),
        (_, Some(r)) => format!("${r}"),
        _ => "#".to_string(),
    };
    let num = value.as_ref().and_then(|v| v.as_f64()).unwrap_or(1.0);
    rsx! {
        div { class: "app-inline",
            Pick { value: mode.clone(), options, class: "app-w-auto", on_change: move |m: String| {
                if m.is_empty() { on_change.call(None) }
                else if m == "#" { on_change.call(Some(Val::Int(1))) }
                else { on_change.call(Some(Val::Text(m))) }
            } }
            if mode == "#" {
                NumIn { value: num, class: "app-w-num", on_change: move |v: f64| on_change.call(Some(Val::num(v))) }
            }
        }
    }
}

#[component]
fn DialRow(
    label: String,
    dial: Dial,
    on_change: EventHandler<Dial>,
    on_delete: Option<EventHandler<()>>,
) -> Element {
    let kind = dial.kind();
    let (lo, hi) = dial.bounds().unwrap_or((0.0, 10.0));
    let cur = dial.default.as_f64().unwrap_or(lo);
    let unit = kind.unit();
    let d1 = dial.clone();
    let d2 = dial.clone();
    let d3 = dial.clone();
    let d4 = dial.clone();
    let d5 = dial.clone();
    rsx! {
        div { class: "app-dial",
            div { class: "app-dial-head",
                input { class: "app-input app-dial-name", value: "{label}", oninput: move |e| { let mut d = d1.clone(); d.label = Some(e.value()); on_change.call(d) } }
                Pick { value: kind.as_str().to_string(), class: "app-w-auto",
                    options: vec![("int".into(), "count".into()), ("distance".into(), "distance (ft)".into()), ("duration".into(), "time (s)".into()), ("enum".into(), "choice".into())],
                    on_change: move |k: String| {
                        let mut d = d2.clone();
                        d.set_kind(match k.as_str() { "distance" => DialKind::Distance, "duration" => DialKind::Duration, "enum" => DialKind::Choice, _ => DialKind::Int });
                        on_change.call(d)
                    } }
                if let Some(h) = on_delete {
                    button { class: "app-icon-btn app-sm", title: "Remove dial", onclick: move |_| h.call(()), "×" }
                }
            }
            match kind {
                DialKind::Choice => {
                    let choices = dial.choices.clone().unwrap_or_default();
                    let def = dial.default.as_str().unwrap_or("").to_string();
                    rsx! {
                        Pick { value: def, options: choices.iter().map(|c| (c.clone(), c.clone())).collect(), on_change: move |v: String| { let mut d = d3.clone(); d.default = Val::Text(v); on_change.call(d) } }
                        WordList { values: choices.clone(), placeholder: "options, comma separated", on_change: move |v: Vec<String>| { let mut d = d4.clone(); if !v.is_empty() { if !v.contains(&d.default.to_string()) { d.default = Val::Text(v[0].clone()); } d.choices = Some(v); } on_change.call(d) } }
                    }
                }
                DialKind::Flag => rsx! {},
                _ => rsx! {
                    div { class: "app-dial-slider",
                        input { r#type: "range", min: "{lo}", max: "{hi}", step: if kind == DialKind::Int { "1" } else { "0.5" }, value: "{cur}",
                            oninput: move |e| { if let Ok(v) = e.value().parse::<f64>() { let mut d = d3.clone(); d.default = Val::num(v); on_change.call(d) } } }
                        span { class: "app-dial-value", "{Val::num(cur)}{unit}" }
                    }
                    div { class: "app-inline app-small",
                        span { "from" }
                        NumIn { value: lo, class: "app-w-num", on_change: move |v: f64| { let mut d = d4.clone(); let (_, h) = d.bounds().unwrap_or((0.0, 10.0)); d.set_bounds(v, h.max(v)); on_change.call(d) } }
                        span { "to" }
                        NumIn { value: hi, class: "app-w-num", on_change: move |v: f64| { let mut d = d5.clone(); let (l, _) = d.bounds().unwrap_or((0.0, 10.0)); d.set_bounds(l.min(v), v); on_change.call(d) } }
                        Pick { value: dial.harder.clone().unwrap_or_default(), class: "app-w-auto",
                            options: vec![("".into(), "harder: ?".into()), ("up".into(), "harder ↑".into()), ("down".into(), "harder ↓".into())],
                            on_change: { let dd = dial.clone(); move |v: String| { let mut d = dd.clone(); d.harder = if v.is_empty() { None } else { Some(v) }; on_change.call(d) } } }
                    }
                },
            }
        }
    }
}

fn game_int_dials(g: &Game, c: Option<&Constraint>) -> Vec<(String, String)> {
    let mut v: Vec<(String, String)> = Vec::new();
    if let Some(c) = c {
        for (k, d) in &c.dials {
            v.push((k.clone(), d.display_name(k)));
        }
    }
    for (k, d) in &g.dials {
        if !v.iter().any(|x| &x.0 == k) {
            v.push((k.clone(), d.display_name(k)));
        }
    }
    v
}

#[component]
fn FormatEditor(gid: &'static str, game: Game) -> Element {
    let shell = use_shell();
    let g = game;
    let f = &g.format;
    let o = f.count_side("offense");
    let d = f.count_side("defense");
    let joker = f.count_side("neutral") > 0;
    let start = f.start.clone().unwrap_or_default();
    let restart = f.restart.clone().unwrap_or_default();
    let rotation = f.rotation.clone().unwrap_or_default();
    let end = f.end.clone().unwrap_or_default();
    let two = g.base_rules.get("two").and_then(|v| v.as_f64());
    let three = g.base_rules.get("three").and_then(|v| v.as_f64());
    let shot = f.shot_clock.as_ref().map(|s| s.seconds.clone());
    let dials = game_int_dials(&g, None);
    let spots: Vec<(String, String)> = [
        "top",
        "wing-L",
        "wing-R",
        "corner-L",
        "corner-R",
        "half-court-line",
        "paint",
    ]
    .iter()
    .map(|s| (s.to_string(), vocab::region_label(s).replace('-', " ")))
    .collect();
    let phased = f.phases.is_some();
    let reads_opts: Vec<(String, String)> = vocab::READS
        .iter()
        .map(|r| (r.id.to_string(), r.cue.to_string()))
        .collect();
    rsx! {
        div { class: "app-pane app-form",
            h3 { class: "app-pane-title", "Base game" }
            Field { label: "Name",
                TextIn { value: g.name.clone(), on_change: move |v: String| { super::rename(shell, gid, &v); edit(gid, "name", move |g| g.name = v) } }
            }
            Field { label: "Stage (R7)", hint: "Where this game sits on the PoST ladder",
                Pick { value: g.stage.clone(), options: STAGES.iter().map(|s| (s.to_string(), s.to_string())).collect(), on_change: move |v: String| edit(gid, "", move |g| g.stage = v) }
            }
            if phased {
                p { class: "app-muted", "This game has phases ({f.numbers.clone().unwrap_or_default()}); edit its roles in YAML for now." }
            } else {
                div { class: "app-grid3",
                    Field { label: "Offense", NumIn { value: o as f64, min: 1.0, max: 5.0, on_change: move |v: f64| edit(gid, "", move |g| { let d = g.format.count_side("defense"); let n = g.format.count_side("neutral") > 0; set_counts(g, v.clamp(1.0, 5.0) as usize, d, n) }) } }
                    Field { label: "Defense", NumIn { value: d as f64, min: 0.0, max: 5.0, on_change: move |v: f64| edit(gid, "", move |g| { let o = g.format.count_side("offense"); let n = g.format.count_side("neutral") > 0; set_counts(g, o, v.clamp(0.0, 5.0) as usize, n) }) } }
                    Field { label: "Joker", Check { value: joker, label: "neutral".to_string(), on_change: move |v: bool| edit(gid, "", move |g| { let o = g.format.count_side("offense"); let d = g.format.count_side("defense"); set_counts(g, o, d, v) }) } }
                }
            }
            Field { label: "Court",
                Pick { value: f.space.court.clone(), options: opts(vocab::COURT_REGIONS), on_change: move |v: String| edit(gid, "", move |g| g.format.space.court = v) }
            }
            Field { label: "Start",
                Pick { value: start.kind.clone(), options: opts(vocab::START_TYPES), on_change: move |v: String| edit(gid, "", move |g| { let mut s = g.format.start.clone().unwrap_or_default(); s.kind = v; g.format.start = Some(s) }) }
            }
            if matches!(start.kind.as_str(), "check-ball" | "dribble-at" | "inbound" | "coach-pass") {
                {
                    let key = match start.kind.as_str() { "dribble-at" => "from", "coach-pass" => "from", _ => "spot" };
                    rsx! {
                        Field { label: "Start spot",
                            Pick { value: start.field(key).unwrap_or_default(), options: { let mut v = vec![("".to_string(), "—".to_string())]; v.extend(spots.clone()); v },
                                on_change: move |v: String| edit(gid, "", move |g| { let mut s = g.format.start.clone().unwrap_or_default(); s.set_field(key, Some(&v)); g.format.start = Some(s) }) }
                        }
                    }
                }
            }
            div { class: "app-grid2",
                Field { label: "After a score",
                    Pick { value: restart.on_score.clone().unwrap_or_default(), options: opts_with_none("—", vocab::RESTARTS), on_change: move |v: String| edit(gid, "", move |g| { let mut r = g.format.restart.clone().unwrap_or_default(); r.on_score = (!v.is_empty()).then_some(v); g.format.restart = Some(r) }) }
                }
                Field { label: "After a stop",
                    Pick { value: restart.on_stop.clone().unwrap_or_default(), options: opts_with_none("—", vocab::RESTARTS), on_change: move |v: String| edit(gid, "", move |g| { let mut r = g.format.restart.clone().unwrap_or_default(); r.on_stop = (!v.is_empty()).then_some(v); g.format.restart = Some(r) }) }
                }
            }
            div { class: "app-grid2",
                Field { label: "Rotation",
                    Pick { value: rotation.kind.clone(), options: opts(vocab::ROTATIONS), on_change: move |v: String| edit(gid, "", move |g| { let mut r = g.format.rotation.clone().unwrap_or_default(); r.kind = v; g.format.rotation = Some(r) }) }
                }
                Field { label: "Teams at this basket",
                    NumIn { value: f.teams.unwrap_or(2) as f64, min: 2.0, max: 8.0, on_change: move |v: f64| edit(gid, "teams", move |g| g.format.teams = Some(v.clamp(2.0, 8.0) as u32)) }
                }
            }
            div { class: "app-grid2",
                Field { label: "Ends",
                    Pick { value: end.kind.clone(), options: opts(vocab::END_TYPES), on_change: move |v: String| edit(gid, "", move |g| { let mut e = g.format.end.clone().unwrap_or_default(); if v == "score" && e.target.is_none() { e.target = Some(Val::Int(7)); } e.kind = v; g.format.end = Some(e) }) }
                }
                match end.kind.as_str() {
                    "score" => rsx! { Field { label: "Game to", ValEdit { value: end.target.clone(), dials: dials.clone(), on_change: move |v: Option<Val>| edit(gid, "target", move |g| { let mut e = g.format.end.clone().unwrap_or_default(); e.target = v; g.format.end = Some(e) }) } } },
                    "time" => rsx! { Field { label: "Seconds", NumIn { value: end.seconds.as_ref().and_then(|v| v.as_f64()).unwrap_or(60.0), on_change: move |v: f64| edit(gid, "secs", move |g| { let mut e = g.format.end.clone().unwrap_or_default(); e.seconds = Some(Val::num(v)); g.format.end = Some(e) }) } } },
                    "possessions" => rsx! { Field { label: "Possessions", NumIn { value: end.count.as_ref().and_then(|v| v.as_f64()).unwrap_or(5.0), on_change: move |v: f64| edit(gid, "poss", move |g| { let mut e = g.format.end.clone().unwrap_or_default(); e.count = Some(Val::num(v)); g.format.end = Some(e) }) } } },
                    _ => rsx! {},
                }
            }
            h4 { class: "app-sub", "Base scoring" }
            div { class: "app-grid2",
                Field { label: "Basket worth",
                    NumIn { value: two.unwrap_or(0.0), min: 0.0, on_change: move |v: f64| edit(gid, "two", move |g| { if v == 0.0 { g.base_rules.shift_remove("two"); } else { g.base_rules.insert("two".into(), Val::num(v)); } }) }
                }
                Field { label: "Three worth",
                    NumIn { value: three.unwrap_or(0.0), min: 0.0, on_change: move |v: f64| edit(gid, "three", move |g| { if v == 0.0 { g.base_rules.shift_remove("three"); } else { g.base_rules.insert("three".into(), Val::num(v)); } }) }
                }
            }
            Field { label: "Shot clock (D-021)",
                div { class: "app-inline",
                    Check { value: shot.is_some(), label: "on".to_string(), on_change: move |v: bool| edit(gid, "", move |g| {
                        g.format.shot_clock = v.then(|| ac_domain::ShotClock { seconds: Val::Int(12), resets_on: vec!["possession.change".into()], ..Default::default() })
                    }) }
                    if shot.is_some() {
                        ValEdit { value: shot.clone(), dials: dials.clone(), on_change: move |v: Option<Val>| edit(gid, "shot", move |g| { if let (Some(sc), Some(v)) = (g.format.shot_clock.as_mut(), v) { sc.seconds = v } }) }
                    }
                }
            }
            h4 { class: "app-sub", "Game dials" }
            for (k, dial) in g.dials.iter() {
                {
                    let key = intern(k);
                    rsx! {
                        DialRow { key: "{key}", label: dial.display_name(k), dial: dial.clone(),
                            on_change: move |d: Dial| edit(gid, key, move |g| { g.dials.insert(key.to_string(), d); }),
                            on_delete: move |_| edit(gid, "", move |g| { g.dials.shift_remove(key); }),
                        }
                    }
                }
            }
            button { class: "app-btn app-sm", onclick: move |_| edit(gid, "", |g| {
                let k = (1..).map(|n| format!("dial{n}")).find(|k| !g.dials.contains_key(k)).unwrap();
                let mut d = Dial::int(1, 10, 5); d.label = Some("New dial".into());
                g.dials.insert(k, d);
            }), "+ dial" }
            Field { label: "Reads this game invites (S6)",
                Chips { values: g.reads.clone(), options: reads_opts, add_label: "+ read".to_string(), on_change: move |v: Vec<String>| edit(gid, "", move |g| g.reads = v) }
            }
            Field { label: "Questions to ask (one per line)", hint: "External focus or questions (R14)",
                TextArea { value: g.questions.join("\n"), on_change: move |v: String| edit(gid, "questions", move |g| g.questions = v.lines().map(str::to_string).filter(|s| !s.trim().is_empty()).collect()) }
            }
            Field { label: "Notes",
                TextArea { value: g.notes.clone(), on_change: move |v: String| edit(gid, "notes", move |g| g.notes = v) }
            }
        }
    }
}

#[component]
fn ConstraintEditor(gid: &'static str, cid: &'static str, game: Game) -> Element {
    let g = game;
    let Some(c) = g
        .constraints
        .iter()
        .chain(g.pool.iter())
        .find(|c| c.id == cid)
        .cloned()
    else {
        return rsx! {};
    };
    let in_pool = g.pool.iter().any(|x| x.id == cid);
    let targets_opts: Vec<(String, String)> = vocab::TARGETS
        .iter()
        .filter(|t| t.0.split('.').count() >= 2)
        .map(|t| (t.0.to_string(), vocab::target_path_label(t.0)))
        .collect();
    let zone_opts = opts(vocab::NAMED_REGIONS);
    let role_ids: Vec<(String, String)> = g
        .format
        .all_roles()
        .iter()
        .map(|r| (r.id.clone(), r.id.clone()))
        .collect();
    let body_words: Vec<&str> = c.cues.iter().flat_map(|q| vocab::body_words(q)).collect();
    let rg = resolved(&g, None);
    rsx! {
        div { class: "app-pane app-form",
            h3 { class: "app-pane-title", if in_pool { "Pool constraint" } else { "Constraint" } }
            Field { label: "Name",
                TextIn { value: c.name.clone(), on_change: move |v: String| edit(gid, "cname", move |g| with_c(g, cid, |c| c.name = v)) }
            }
            Field { label: "Category",
                Pick { value: c.category.clone(), options: CATEGORIES.iter().map(|s| (s.to_string(), s.to_string())).collect(), on_change: move |v: String| edit(gid, "", move |g| with_c(g, cid, |c| c.category = v)) }
            }
            Field { label: "Invites (R2)", hint: "Behaviours it should bring out, comma separated",
                WordList { values: c.invites.clone(), placeholder: "drive-and-kick, extra-pass", on_change: move |v: Vec<String>| edit(gid, "", move |g| with_c(g, cid, |c| c.invites = v)) }
            }
            Field { label: "Preserves (R3)", hint: "Information it keeps for the players",
                WordList { values: c.preserves.clone(), placeholder: "help-defense, closeout", on_change: move |v: Vec<String>| edit(gid, "", move |g| with_c(g, cid, |c| c.preserves = v)) }
            }
            Field { label: "Learning targets (R2)",
                Chips { values: c.targets.clone(), options: targets_opts, add_label: "+ target".to_string(), on_change: move |v: Vec<String>| edit(gid, "", move |g| with_c(g, cid, |c| c.targets = v)) }
            }
            h4 { class: "app-sub", "Rules" }
            for r in c.rules.iter() {
                {
                    let rid = intern(&r.id);
                    let text = sentence_in(&rg, &g, cid, r);
                    rsx! {
                        div { key: "{rid}", class: "app-rule-row", onclick: move |_| select(gid, Sel::Rule(cid, rid)),
                            span { class: "app-judge app-judge-{r.effective_judge().as_str()}", "{r.effective_judge().as_str()}" }
                            span { "{text}" }
                        }
                    }
                }
            }
            div { class: "app-inline",
                button { class: "app-btn app-sm", onclick: move |_| add_rule(gid, cid, "score"), "+ scoring rule" }
                button { class: "app-btn app-sm", onclick: move |_| add_rule(gid, cid, "limit"), "+ limit" }
                button { class: "app-btn app-sm", onclick: move |_| add_rule(gid, cid, "tally"), "+ tally" }
            }
            h4 { class: "app-sub", "Dials (difficulty, R4)" }
            for (k, dial) in c.dials.iter() {
                {
                    let key = intern(k);
                    rsx! {
                        DialRow { key: "{key}", label: dial.display_name(k), dial: dial.clone(),
                            on_change: move |d: Dial| edit(gid, key, move |g| with_c(g, cid, |c| { c.dials.insert(key.to_string(), d); })),
                            on_delete: move |_| edit(gid, "", move |g| with_c(g, cid, |c| { c.dials.shift_remove(key); })),
                        }
                    }
                }
            }
            button { class: "app-btn app-sm", onclick: move |_| edit(gid, "", move |g| with_c(g, cid, |c| {
                let k = (1..).map(|n| if n == 1 { "limit".to_string() } else { format!("dial{n}") }).find(|k| !c.dials.contains_key(k)).unwrap();
                let mut d = Dial::int(0, 4, 2); d.label = Some("Limit".into()); d.harder = Some("down".into());
                c.dials.insert(k, d);
            })), "+ dial" }
            h4 { class: "app-sub", "Zones (space)" }
            for (zi, z) in c.zones.iter().enumerate() {
                div { key: "{zi}", class: "app-zone-row",
                    TextIn { value: z.id.clone(), class: "app-w-id", on_change: move |v: String| edit(gid, "zid", move |g| with_c(g, cid, |c| { if let Some(z) = c.zones.get_mut(zi) { let old = z.id.clone(); z.id = v.clone(); for r in c.rules.iter_mut() { for t in std::iter::once(&mut r.when).chain(r.after.iter_mut()) { if let Some(zz) = &mut t.zone { *zz = match zz.clone() { OneOrMany::One(s) if s == old => OneOrMany::One(v.clone()), OneOrMany::Many(m) => OneOrMany::Many(m.into_iter().map(|s| if s == old { v.clone() } else { s }).collect()), o => o }; } } } } })) }
                    Pick { value: z.kind.clone(), class: "app-w-auto", options: ZONE_KINDS.iter().map(|s| (s.to_string(), s.to_string())).collect(), on_change: move |v: String| edit(gid, "", move |g| with_c(g, cid, |c| if let Some(z) = c.zones.get_mut(zi) { z.kind = v })) }
                    if let Some(n) = z.shape.named.clone() {
                        Pick { value: n, class: "app-w-auto", options: zone_opts.clone(), on_change: move |v: String| edit(gid, "", move |g| with_c(g, cid, |c| if let Some(z) = c.zones.get_mut(zi) { z.shape = Shape::named(&v) })) }
                    } else {
                        span { class: "app-muted app-small", "custom shape" }
                    }
                    button { class: "app-icon-btn app-sm", title: "Remove zone", onclick: move |_| edit(gid, "", move |g| with_c(g, cid, |c| { if zi < c.zones.len() { c.zones.remove(zi); } })), "×" }
                }
            }
            button { class: "app-btn app-sm", onclick: move |_| edit(gid, "", move |g| with_c(g, cid, |c| {
                let id = (1..).map(|n| format!("zone-{n}")).find(|id| !c.zones.iter().any(|z| &z.id == id)).unwrap();
                c.zones.push(Zone { id, kind: "bonus".into(), shape: Shape::named("corner-L"), ..Default::default() });
            })), "+ zone" }
            h4 { class: "app-sub", "Role limits" }
            for (ri, e) in c.roles.iter().enumerate() {
                div { key: "{ri}", class: "app-zone-row",
                    span { class: "app-small",
                        {
                            match (&e.restrict, &e.stay_in, &e.cannot, &e.add) {
                                (Some(r), Some(z), _, _) => format!("{r} must stay in the {}", vocab::region_label(z).to_lowercase()),
                                (Some(r), _, Some(w), _) => format!("{r} can't {w}"),
                                (_, _, _, Some(a)) => format!("Adds {} ({})", a.id, a.team),
                                _ => "role edit".to_string(),
                            }
                        }
                    }
                    button { class: "app-icon-btn app-sm", title: "Remove", onclick: move |_| edit(gid, "", move |g| with_c(g, cid, |c| { if ri < c.roles.len() { c.roles.remove(ri); } })), "×" }
                }
            }
            {
                let role_ids2 = role_ids.clone();
                rsx! {
                    div { class: "app-inline app-small",
                        span { "Add:" }
                        Pick { value: "".to_string(), class: "app-w-auto", options: { let mut v = vec![("".to_string(), "role…".to_string())]; v.extend(role_ids.clone()); v },
                            on_change: move |role: String| if !role.is_empty() { edit(gid, "", move |g| with_c(g, cid, |c| c.roles.push(RoleEdit { restrict: Some(role), stay_in: Some("paint".into()), ..Default::default() }))) } }
                        span { "stays in a zone ·" }
                        Pick { value: "".to_string(), class: "app-w-auto", options: { let mut v = vec![("".to_string(), "role…".to_string())]; v.extend(role_ids2.clone()); v },
                            on_change: move |role: String| if !role.is_empty() { edit(gid, "", move |g| with_c(g, cid, |c| c.roles.push(RoleEdit { restrict: Some(role), cannot: Some("shoot".into()), ..Default::default() }))) } }
                        span { "can't shoot" }
                    }
                }
            }
            if let Some(e) = c.roles.iter().position(|e| e.stay_in.is_some()) {
                Field { label: "Zone for that role",
                    Pick { value: c.roles[e].stay_in.clone().unwrap_or_default(), options: opts(vocab::NAMED_REGIONS), on_change: move |v: String| edit(gid, "", move |g| with_c(g, cid, |c| if let Some(x) = c.roles.get_mut(e) { x.stay_in = Some(v) })) }
                }
            }
            Field { label: "Cues / questions (one per line)", hint: "R14: point at the defense, the space or the ball, not the body",
                TextArea { value: c.cues.join("\n"), on_change: move |v: String| edit(gid, "cues", move |g| with_c(g, cid, |c| c.cues = v.lines().map(str::to_string).filter(|s| !s.trim().is_empty()).collect())) }
            }
            if !body_words.is_empty() {
                p { class: "app-warn", "R14: '{body_words.join(\"', '\")}' points at the body. Try describing what the player sees or where the ball goes." }
            }
            Field { label: "Notes",
                TextArea { value: c.notes.clone(), on_change: move |v: String| edit(gid, "cnotes", move |g| with_c(g, cid, |c| c.notes = v)) }
            }
            div { class: "app-inline",
                if in_pool {
                    button { class: "app-btn app-sm", onclick: move |_| edit(gid, "", move |g| { if let Some(i) = g.pool.iter().position(|c| c.id == cid) { let c = g.pool.remove(i); g.constraints.push(c); } }), "Move into the stack" }
                } else {
                    button { class: "app-btn app-sm", onclick: move |_| edit(gid, "", move |g| { if let Some(i) = g.constraints.iter().position(|c| c.id == cid) { let c = g.constraints.remove(i); g.pool.push(c); } }), "Move to the pool" }
                }
                button { class: "app-btn app-sm app-danger", onclick: move |_| { edit(gid, "", move |g| { g.constraints.retain(|c| c.id != cid); g.pool.retain(|c| c.id != cid); }); select(gid, Sel::Format); }, "Delete constraint" }
            }
        }
    }
}

fn add_rule(gid: &'static str, cid: &'static str, kind: &str) {
    let mut new_id = String::new();
    let kind = kind.to_string();
    edit(gid, "", |g| {
        let rid = g.fresh_rule_id(&format!("{cid}-{kind}"));
        new_id = rid.clone();
        let rule = match kind.as_str() {
            "limit" => Rule {
                id: rid,
                label: Some("Too many dribbles".into()),
                when: Trigger {
                    count: Some("dribble".into()),
                    by: Some("player".into()),
                    op: Some(">".into()),
                    value: Some(Val::Int(2)),
                    ..Default::default()
                },
                then: vec![Consequence::turnover()],
                judge: Some(Judge::Coach),
                ..Default::default()
            },
            "tally" => Rule {
                id: rid,
                label: Some("Deflection".into()),
                when: Trigger::event("deflection"),
                then: vec![Consequence::tally()],
                judge: Some(Judge::Coach),
                ..Default::default()
            },
            _ => Rule {
                id: rid,
                label: Some("Bonus".into()),
                when: Trigger::event("make"),
                then: vec![Consequence::points(2)],
                judge: Some(Judge::Coach),
                ..Default::default()
            },
        };
        with_c(g, cid, |c| c.rules.push(rule));
    });
    select(gid, Sel::Rule(cid, intern(&new_id)));
}

fn trigger_mode(t: &Trigger) -> &'static str {
    match t.kind() {
        TriggerKind::Count => "count",
        TriggerKind::Clock => "clock",
        TriggerKind::GameEnd => "game-end",
        TriggerKind::Score | TriggerKind::PossessionEnd => "other",
        _ => "event",
    }
}

fn consequence_mode(c: &Consequence) -> &'static str {
    if c.points.is_some() {
        "points"
    } else if c.is_turnover() {
        "turnover"
    } else if c.possession.is_some() {
        "possession"
    } else if c.is_tally() {
        "tally"
    } else if c.restart.is_some() {
        "restart"
    } else if c.role.is_some() {
        "role"
    } else if c.choose.is_some() {
        "choose"
    } else if c.end.is_some() {
        "end"
    } else {
        "other"
    }
}

fn new_consequence(mode: &str) -> Consequence {
    match mode {
        "points" => Consequence::points(1),
        "turnover" => Consequence::turnover(),
        "possession" => Consequence {
            possession: Some("switch".into()),
            ..Default::default()
        },
        "restart" => Consequence {
            restart: Some("check-ball".into()),
            ..Default::default()
        },
        "end" => Consequence {
            end: Some("game".into()),
            ..Default::default()
        },
        _ => Consequence::tally(),
    }
}

#[component]
fn RuleEditor(gid: &'static str, cid: &'static str, rid: &'static str, game: Game) -> Element {
    let g = game;
    let Some(c) = g
        .constraints
        .iter()
        .chain(g.pool.iter())
        .find(|c| c.id == cid)
        .cloned()
    else {
        return rsx! { Empty { text: "Pick a rule in the stack." } };
    };
    let Some(r) = c.rules.iter().find(|r| r.id == rid).cloned() else {
        return rsx! { Empty { text: "Pick a rule in the stack." } };
    };
    let rg = resolved(&g, None);
    let sentence = rg
        .rule(rid)
        .map(|rr| render::sentence(&rr.rule))
        .unwrap_or_else(|| Some(sentence_in(&rg, &g, cid, &r)));
    let mode = trigger_mode(&r.when);
    let events: Vec<(String, String)> = vocab::EVENTS
        .iter()
        .filter(|e| !e.0.contains('.'))
        .map(|e| (e.0.to_string(), e.1.to_string()))
        .collect();
    let mut zone_opts: Vec<(String, String)> = vec![("".into(), "anywhere".into())];
    for z in &c.zones {
        zone_opts.push((z.id.clone(), format!("{} (this constraint)", z.id)));
    }
    zone_opts.extend(opts(vocab::NAMED_REGIONS));
    let zone_value = r
        .when
        .zone
        .as_ref()
        .and_then(|z| z.first().map(str::to_string))
        .unwrap_or_default();
    let zone_list = r.when.zone.as_ref().is_some_and(|z| z.is_list());
    let after_ev = r
        .after
        .first()
        .and_then(|s| s.event.clone())
        .unwrap_or_default();
    let dials = game_int_dials(&g, Some(&c));
    let others: Vec<(String, String)> = rg
        .rules
        .iter()
        .filter(|x| x.rule.id != rid)
        .map(|x| (x.rule.id.clone(), x.rule.display_label()))
        .collect();
    let issues: Vec<Issue> = rg
        .issues
        .iter()
        .filter(|i| i.message.contains(&format!("'{}'", r.display_label())))
        .cloned()
        .collect();
    let make_value = r
        .when
        .value
        .as_ref()
        .map(|v| v.to_string())
        .unwrap_or_default();
    rsx! {
        div { class: "app-pane app-form",
            button { class: "app-link", onclick: move |_| select(gid, Sel::Constraint(cid)), "← {c.name}" }
            h3 { class: "app-pane-title", "Rule" }
            div { class: "app-sentence",
                match &sentence {
                    Some(s) => rsx! { "“{s}”" },
                    None => rsx! { span { class: "app-muted", "Not shown on the card (clock or tally rule)" } },
                }
            }
            Field { label: "Button label", hint: "What the scorer's phone button says (16 characters or fewer)",
                TextIn { value: r.label.clone().unwrap_or_default(), on_change: move |v: String| edit(gid, "rlabel", move |g| with_r(g, cid, rid, |r| r.label = (!v.is_empty()).then_some(v))) }
            }
            h4 { class: "app-sub", "When" }
            Field { label: "Trigger",
                Pick { value: mode.to_string(), options: vec![("event".into(), "Something happens".into()), ("count".into(), "Too many of something (limit)".into()), ("clock".into(), "A clock runs out".into()), ("game-end".into(), "The game ends".into())],
                    on_change: move |m: String| edit(gid, "", move |g| with_r(g, cid, rid, |r| {
                        r.when = match m.as_str() {
                            "count" => Trigger { count: Some("dribble".into()), by: Some("player".into()), op: Some(">".into()), value: Some(Val::Int(2)), ..Default::default() },
                            "clock" => Trigger { clock: Some("shot".into()), reaches: Some(Val::Int(0)), ..Default::default() },
                            "game-end" => Trigger { game: Some("end".into()), ..Default::default() },
                            _ => Trigger::event("make"),
                        };
                        if m != "event" { r.after.clear(); r.within = None; }
                    })) }
            }
            match mode {
                "event" => rsx! {
                    Field { label: "Event",
                        Pick { value: r.when.event.clone().unwrap_or_default(), options: events.clone(), on_change: move |v: String| edit(gid, "", move |g| with_r(g, cid, rid, |r| { if v != "make" { r.when.value = None; } r.when.event = Some(v) })) }
                    }
                    if r.when.event.as_deref() == Some("make") {
                        Field { label: "Shot",
                            Pick { value: make_value.clone(), options: vec![("".into(), "any basket".into()), ("2".into(), "two".into()), ("3".into(), "three".into()), ("ft".into(), "free throw".into())],
                                on_change: move |v: String| edit(gid, "", move |g| with_r(g, cid, rid, |r| r.when.value = match v.as_str() { "" => None, "2" => Some(Val::Int(2)), "3" => Some(Val::Int(3)), o => Some(Val::Text(o.into())) })) }
                        }
                    }
                    Field { label: "Where", hint: if zone_list { "This rule lists several zones; picking one replaces the list" } else { "" },
                        Pick { value: zone_value.clone(), options: zone_opts.clone(), on_change: move |v: String| edit(gid, "", move |g| with_r(g, cid, rid, |r| r.when.zone = (!v.is_empty()).then_some(OneOrMany::One(v)))) }
                    }
                    Field { label: "Only after (same possession)",
                        Pick { value: after_ev.clone(), options: { let mut v = vec![("".to_string(), "—".to_string())]; v.extend(events.clone()); v },
                            on_change: move |v: String| edit(gid, "", move |g| with_r(g, cid, rid, |r| { if v.is_empty() { r.after.clear(); r.within = None; } else { r.after = vec![Trigger::event(&v)]; } })) }
                    }
                    if !after_ev.is_empty() {
                        Field { label: "Within (seconds of that)",
                            ValEdit { value: r.within.clone(), dials: dials.clone(), allow_none: true, on_change: move |v: Option<Val>| edit(gid, "within", move |g| with_r(g, cid, rid, |r| r.within = v)) }
                        }
                    }
                },
                "count" => rsx! {
                    div { class: "app-grid2",
                        Field { label: "Counting",
                            Pick { value: r.when.count.clone().unwrap_or_default(), options: events.clone(), on_change: move |v: String| edit(gid, "", move |g| with_r(g, cid, rid, |r| r.when.count = Some(v))) }
                        }
                        Field { label: "By",
                            Pick { value: r.when.by.clone().unwrap_or_else(|| "player".into()), options: vec![("player".into(), "one player".into()), ("team".into(), "the team".into())], on_change: move |v: String| edit(gid, "", move |g| with_r(g, cid, rid, |r| r.when.by = Some(v))) }
                        }
                    }
                    div { class: "app-grid2",
                        Field { label: "Is",
                            Pick { value: r.when.op.clone().unwrap_or_else(|| ">".into()), options: vec![(">".into(), "more than".into()), (">=".into(), "at least".into()), ("==".into(), "exactly".into()), ("<".into(), "fewer than".into())], on_change: move |v: String| edit(gid, "", move |g| with_r(g, cid, rid, |r| r.when.op = Some(v))) }
                        }
                        Field { label: "Number",
                            ValEdit { value: r.when.value.clone(), dials: dials.clone(), on_change: move |v: Option<Val>| edit(gid, "cval", move |g| with_r(g, cid, rid, |r| r.when.value = v)) }
                        }
                    }
                    p { class: "app-muted app-small", "Counts reset every possession. Things nobody tallies live (dribbles, passes) become a violation button for the coach (S2 §3)." }
                },
                "clock" => rsx! {
                    Field { label: "Clock",
                        Pick { value: r.when.clock.clone().unwrap_or_default(), options: vec![("shot".into(), "shot clock".into()), ("block".into(), "block clock".into()), ("game".into(), "game clock".into())], on_change: move |v: String| edit(gid, "", move |g| with_r(g, cid, rid, |r| r.when.clock = Some(v))) }
                    }
                },
                _ => rsx! {},
            }
            h4 { class: "app-sub", "Then" }
            for (ti, t) in r.then.iter().enumerate() {
                {
                    let cm = consequence_mode(t);
                    rsx! {
                        div { key: "{ti}", class: "app-then",
                            Pick { value: cm.to_string(), class: "app-w-auto",
                                options: vec![("points".into(), "Points".into()), ("turnover".into(), "Turnover".into()), ("possession".into(), "Change possession".into()), ("restart".into(), "Restart".into()), ("end".into(), "End the game".into()), ("tally".into(), "Just count it (tally)".into()), ("role".into(), "Role change".into()), ("choose".into(), "Pick a rule from the pool".into()), ("other".into(), "Other".into())],
                                on_change: move |m: String| edit(gid, "", move |g| with_r(g, cid, rid, |r| if let Some(t) = r.then.get_mut(ti) { *t = new_consequence(&m) })) }
                            if cm == "points" {
                                ValEdit { value: t.points.clone(), dials: dials.clone(), on_change: move |v: Option<Val>| edit(gid, "pts", move |g| with_r(g, cid, rid, |r| if let Some(t) = r.then.get_mut(ti) { t.points = v.or(Some(Val::Int(1))) })) }
                                Pick { value: t.to.clone().unwrap_or_default(), class: "app-w-auto", options: vec![("".into(), "to the team it's credited to".into()), ("offense".into(), "to the offense".into()), ("defense".into(), "to the defense".into())],
                                    on_change: move |v: String| edit(gid, "", move |g| with_r(g, cid, rid, |r| if let Some(t) = r.then.get_mut(ti) { t.to = (!v.is_empty()).then_some(v) })) }
                            }
                            if cm == "restart" {
                                Pick { value: t.restart.clone().unwrap_or_default(), class: "app-w-auto", options: opts(vocab::START_TYPES), on_change: move |v: String| edit(gid, "", move |g| with_r(g, cid, rid, |r| if let Some(t) = r.then.get_mut(ti) { t.restart = Some(v) })) }
                            }
                            button { class: "app-icon-btn app-sm", title: "Remove", onclick: move |_| edit(gid, "", move |g| with_r(g, cid, rid, |r| { if ti < r.then.len() { r.then.remove(ti); } })), "×" }
                        }
                    }
                }
            }
            button { class: "app-btn app-sm", onclick: move |_| edit(gid, "", move |g| with_r(g, cid, rid, |r| r.then.push(Consequence::turnover()))), "+ then" }
            Field { label: "Replaces (D-034)", hint: "Rules this one overrides for the same play, e.g. a bonus three replaces the normal three",
                Chips { values: r.overrides.clone(), options: others, add_label: "+ replaces".to_string(), on_change: move |v: Vec<String>| edit(gid, "", move |g| with_r(g, cid, rid, |r| r.overrides = v)) }
            }
            Field { label: "Who judges it (D-025)",
                Pick { value: r.effective_judge().as_str().to_string(), options: vec![("coach".into(), "Coach taps it".into()), ("players".into(), "Players follow it".into()), ("auto".into(), "App works it out".into())],
                    on_change: move |v: String| edit(gid, "", move |g| with_r(g, cid, rid, |r| r.judge = Judge::parse(&v))) }
            }
            Field { label: "Card sentence (optional)", hint: "Leave empty to use the generated sentence",
                TextIn { value: r.say.clone().unwrap_or_default(), placeholder: sentence.clone().unwrap_or_default(), on_change: move |v: String| edit(gid, "say", move |g| with_r(g, cid, rid, |r| r.say = (!v.is_empty()).then_some(v))) }
            }
            for (i, iss) in issues.iter().enumerate() {
                p { key: "{i}", class: if iss.severity == Severity::Error { "app-err" } else { "app-warn" }, "{iss.code}: {iss.message}" }
            }
            button { class: "app-btn app-sm app-danger", onclick: move |_| { edit(gid, "", move |g| with_c(g, cid, |c| c.rules.retain(|r| r.id != rid))); select(gid, Sel::Constraint(cid)); }, "Delete rule" }
        }
    }
}

// ---------------------------------------------------------------------------------------
// right: checks & predicted effects
// ---------------------------------------------------------------------------------------

/// The shell reuses one scope for whatever view sits in a slot (a lone keyed child),
/// so this wrapper has no hooks and renders a distinct component type.
#[allow(non_snake_case)]
fn ChecksPane() -> Element {
    rsx! { ChecksPaneBody {} }
}

#[component]
fn ChecksPaneBody() -> Element {
    let Some(gid) = use_game_id() else {
        return rsx! {};
    };
    let Some((g, _, rollback)) = snapshot(gid) else {
        return rsx! {};
    };
    let rg = resolved(&g, rollback);
    let c = court::preset(&court_preset());
    let ests = [
        estimates::area_per_player(&c, &rg),
        estimates::intensity(&c, &rg, None),
        estimates::involvement(&rg),
    ];
    let repr = estimates::representativeness(&rg);
    let errors: Vec<Issue> = rg
        .issues
        .iter()
        .filter(|i| i.severity == Severity::Error)
        .cloned()
        .collect();
    let warnings: Vec<Issue> = rg
        .issues
        .iter()
        .filter(|i| i.severity == Severity::Warning)
        .cloned()
        .collect();
    let buttons = render::buttons(&rg);
    let calls = render::calls(&rg);
    let board = render::board_line(&rg);
    rsx! {
        div { class: "app-pane",
            Section { title: "Checks".to_string(),
                if errors.is_empty() && warnings.is_empty() {
                    p { class: "app-ok", "✓ Nothing to fix." }
                }
                for (i, iss) in errors.iter().chain(warnings.iter()).enumerate() {
                    {
                        let target = iss.constraint.clone().map(|c| intern(&c));
                        rsx! {
                            div { key: "{i}",
                                class: if iss.severity == Severity::Error { "app-issue app-err" } else { "app-issue app-warn" },
                                onclick: move |_| if let Some(c) = target { select(gid, Sel::Constraint(c)) },
                                span { class: "app-code", "{iss.code}" }
                                span { "{iss.message}" }
                            }
                        }
                    }
                }
            }
            Section { title: "Predicted effects (estimates, R15)".to_string(),
                for (i, e) in ests.iter().enumerate() {
                    div { key: "{i}", class: "app-est",
                        div { class: "app-est-head",
                            span { class: "app-est-label", "{e.label}" }
                            span { class: "app-est-value", "{e.value}{e.unit}" }
                            if !e.note.is_empty() { span { class: "app-tag", "{e.note}" } }
                            span { class: "app-conf app-conf-{e.confidence}", "{e.confidence}" }
                        }
                        div { class: "app-est-formula", "{e.formula}" }
                    }
                }
                p { class: "app-muted app-small", "Starting guesses from S9; they get calibrated from what you log at practice." }
            }
            Section { title: "Game-like? (representativeness)".to_string(),
                ul { class: "app-plain",
                    for (i, (label, ok)) in repr.iter().enumerate() {
                        li { key: "{i}", class: if *ok { "app-ok" } else { "app-muted" }, if *ok { "✓ " } else { "○ " } "{label}" }
                    }
                }
            }
            Section { title: "In the gym".to_string(),
                p { class: "app-field-label", "TV line" }
                p { class: "app-mono", if board.is_empty() { "(base scoring only)" } else { "{board}" } }
                p { class: "app-field-label", "Scorer's buttons (per team)" }
                div { class: "app-chips",
                    for (i, b) in buttons.iter().enumerate() { span { key: "{i}", class: "app-btn-preview", "{b}" } }
                    for (i, b) in calls.iter().enumerate() { span { key: "c{i}", class: "app-btn-preview app-btn-call", "{b}" } }
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
    let Some(gid) = use_game_id() else {
        return rsx! {};
    };
    let Some((g, _, rollback)) = snapshot(gid) else {
        return rsx! {};
    };
    let rg = resolved(&g, rollback);
    let c = court::preset(&court_preset());
    let app = estimates::area_per_player(&c, &rg).value;
    let n = rg.active.len();
    let errs = rg.errors().count();
    rsx! {
        span { "{g.numbers()} · {vocab::region_label(&g.format.space.court)} · {n} constraints · {g.stage} · ~{app} ft²/player" }
        if errs > 0 { span { class: "app-err-count", " · {errs} to fix" } }
    }
}

// ---------------------------------------------------------------------------------------
// printable card(s)
// ---------------------------------------------------------------------------------------

pub fn card_html(g: &Game, rollback: Option<usize>, preset: &str) -> String {
    let rg = resolved(g, rollback);
    let card = render::game_card(g, &rg);
    let svg = crate::court::svg_string(
        preset,
        &rg.format.space.court,
        &zone_draws(&rg, preset, None),
        &player_draws(&rg),
        300,
    );
    let list = |title: &str, items: &[String]| {
        if items.is_empty() {
            String::new()
        } else {
            format!(
                "<h3>{title}</h3><ul>{}</ul>",
                items
                    .iter()
                    .map(|i| format!("<li>{}</li>", super::esc(i)))
                    .collect::<String>()
            )
        }
    };
    format!(
        r#"<div class="card"><div>{svg}</div><div><h2>{}</h2><p class="sub">{}</p>{}{}{}{}{}</div></div>"#,
        super::esc(&card.title),
        super::esc(&card.subtitle),
        list("How to score", &card.how_to_score),
        list("Rules", &card.rules),
        list("How it runs", &card.how_it_runs),
        list("Ask", &card.questions),
        if card.board.is_empty() {
            String::new()
        } else {
            format!(
                r#"<p><span class="board">TV: {}</span></p>"#,
                super::esc(&card.board)
            )
        },
    )
}

pub fn card_page(games: &[(Game, Option<usize>)]) -> String {
    let preset = court_preset();
    let body: String = games
        .iter()
        .map(|(g, r)| card_html(g, *r, &preset))
        .collect();
    format!(
        "<!doctype html><html><head><meta charset=\"utf-8\"><title>Game cards</title><style>{}</style></head><body>{body}<p class=\"noprint muted\">Print with Ctrl+P.</p></body></html>",
        super::PRINT_CSS
    )
}
