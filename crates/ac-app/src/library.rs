//! Library nav pane (03 §2): every game, practice and play, searchable and filterable by
//! learning target. Click to open; duplicate or delete from the row.

use ac_domain::store::{Doc, DocKind};
use ac_sport_basketball::vocab;
use app_shell::prelude::*;
use dioxus::prelude::*;

use crate::store::{Entry, LIBRARY};

fn kind_label(k: DocKind) -> &'static str {
    match k {
        DocKind::Game => "Games",
        DocKind::Practice => "Practices",
        DocKind::Play => "Plays",
    }
}

fn duplicate(shell: Shell, e: &Entry) {
    let Ok(Some(doc)) = crate::store::store().load(e.kind, &e.id) else {
        return;
    };
    let id = crate::store::new_id(e.kind);
    let copy = match doc {
        Doc::Game(mut g) => {
            g.id = id.clone();
            g.name = format!("{} (copy)", g.name);
            Doc::Game(g)
        }
        Doc::Practice(mut p) => {
            p.id = id.clone();
            p.name = format!("{} (copy)", p.title());
            Doc::Practice(p)
        }
        Doc::Play(mut p) => {
            p.id = id.clone();
            p.name = format!("{} (copy)", p.name);
            Doc::Play(p)
        }
    };
    let name = copy.name();
    crate::store::save(copy);
    crate::docs::open(shell, e.kind, &id, &name);
}

/// The shell reuses one scope for whatever view sits in a slot (a lone keyed child),
/// so this wrapper has no hooks and renders a distinct component type.
#[allow(non_snake_case)]
pub fn LibraryPane() -> Element {
    rsx! { LibraryPaneBody {} }
}

#[component]
fn LibraryPaneBody() -> Element {
    let shell = use_shell();
    let mut filter = use_signal(String::new);
    let mut kind = use_signal(|| None::<DocKind>);
    let mut target = use_signal(String::new);
    let mut confirm = use_signal(|| None::<(DocKind, String)>);
    let lib = LIBRARY.read().clone();
    let f = filter.read().to_lowercase();
    let tf = target.read().clone();
    let trained: Vec<String> = lib
        .iter()
        .filter(|e| e.kind == DocKind::Game)
        .flat_map(|e| e.reads.clone())
        .collect();
    let principles: Vec<(String, String)> = {
        let mut v = vec![("".to_string(), "Any learning target".to_string())];
        v.extend(
            vocab::TARGETS
                .iter()
                .filter(|t| t.0.split('.').count() == 2)
                .map(|t| (t.0.to_string(), vocab::target_path_label(t.0))),
        );
        v
    };
    let visible = |e: &Entry| {
        (kind.read().is_none() || *kind.read() == Some(e.kind))
            && (f.is_empty()
                || e.name.to_lowercase().contains(&f)
                || e.detail.to_lowercase().contains(&f)
                || e.targets
                    .iter()
                    .any(|t| vocab::target_label(t).to_lowercase().contains(&f)))
            && (tf.is_empty() || e.targets.iter().any(|t| t.starts_with(&tf)))
    };
    rsx! {
        div { class: "app-pane app-library",
            div { class: "app-inline",
                button { class: "app-btn app-sm", onclick: move |_| { crate::docs::create(shell, DocKind::Game); }, "+ Game" }
                button { class: "app-btn app-sm", onclick: move |_| { crate::docs::create(shell, DocKind::Practice); }, "+ Practice" }
                button { class: "app-btn app-sm", onclick: move |_| { crate::docs::create(shell, DocKind::Play); }, "+ Play" }
            }
            input { class: "app-input", r#type: "search", placeholder: "Search", value: "{filter}", oninput: move |e| filter.set(e.value()) }
            div { class: "app-seg",
                for (k, l) in [(None, "All"), (Some(DocKind::Game), "Games"), (Some(DocKind::Practice), "Practices"), (Some(DocKind::Play), "Plays")] {
                    button {
                        key: "{l}",
                        class: if *kind.read() == k { "app-seg-btn app-seg-on" } else { "app-seg-btn" },
                        onclick: move |_| kind.set(k),
                        "{l}"
                    }
                }
            }
            select { class: "app-input", onchange: move |e| target.set(e.value()),
                for (v, l) in principles.iter() { option { key: "{v}", value: "{v}", selected: *v == tf, "{l}" } }
            }
            for k in DocKind::ALL {
                {
                    let rows: Vec<Entry> = lib.iter().filter(|e| e.kind == k && visible(e)).cloned().collect();
                    rsx! {
                        if !rows.is_empty() {
                            h4 { key: "{k:?}", class: "app-lib-kind", "{kind_label(k)} ({rows.len()})" }
                            for e in rows.into_iter() {
                                {
                                    let e2 = e.clone();
                                    let e3 = e.clone();
                                    let untrained = if e.kind == DocKind::Play { e.reads.iter().filter(|r| !trained.contains(r)).count() } else { 0 };
                                    let confirming = confirm.read().as_ref() == Some(&(e.kind, e.id.clone()));
                                    let (ek, eid) = (e.kind, e.id.clone());
                                    rsx! {
                                        div { key: "{e.id}", class: "app-lib-row",
                                            div { class: "app-lib-main",
                                                onclick: move |_| crate::docs::open(shell, e2.kind, &e2.id, &e2.name),
                                                div { class: "app-row-title", "{e.name}" }
                                                div { class: "app-row-sub", "{e.detail}" }
                                                if !e.targets.is_empty() {
                                                    div { class: "app-row-sub app-tags",
                                                        for t in e.targets.iter().take(3) { span { key: "{t}", class: "app-tag", "{vocab::target_label(t)}" } }
                                                    }
                                                }
                                                if untrained > 0 {
                                                    div { class: "app-row-sub app-warn-text", "{untrained} read(s) no game trains yet (R9)" }
                                                }
                                            }
                                            div { class: "app-row-tools",
                                                button { class: "app-icon-btn app-sm", title: "Duplicate", onclick: move |_| duplicate(shell, &e3), "⧉" }
                                                if confirming {
                                                    button { class: "app-btn app-sm app-danger", onclick: move |_| {
                                                        let mut sh = shell;
                                                        sh.close_document(&eid);
                                                        crate::store::delete(ek, &eid);
                                                        confirm.set(None);
                                                    }, "Delete?" }
                                                } else {
                                                    button { class: "app-icon-btn app-sm", title: "Delete", onclick: {
                                                        let id = e.id.clone();
                                                        move |_| confirm.set(Some((ek, id.clone())))
                                                    }, "×" }
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
    }
}
