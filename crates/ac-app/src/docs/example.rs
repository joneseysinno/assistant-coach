//! Example document type. Rename it, or copy it as the starting point for
//! each new kind of document.

use std::collections::HashMap;

use app_shell::prelude::*;
use dioxus::prelude::*;

pub const ID: &str = "example";

/// Per-document state, keyed by document id, so the viewport, panels and
/// status readout of one document all see the same data.
#[derive(Clone, Default, PartialEq, Debug)]
pub struct ExampleState {
    pub items: Vec<String>,
}

pub static STATE: GlobalSignal<HashMap<String, ExampleState>> = Signal::global(HashMap::new);

fn state(doc: &str) -> ExampleState {
    STATE.read().get(doc).cloned().unwrap_or_default()
}

pub fn doc_type() -> DocType {
    let mut t = DocType::new(ID, "Example", icons::FILES, Viewport);
    t.toolbar = vec![ToolGroup::new(
        "Edit",
        vec![
            Tool::new("ex.add", "Add item", icons::PLUS).key("Shift+A"),
            Tool::new("ex.clear", "Clear items", icons::CLOSE),
        ],
    )];
    t.left_rail = vec![RailItem::panel(
        "ex.outline",
        "Outline",
        icons::TREE,
        Outline,
    )];
    t.right_rail = vec![RailItem::panel(
        "ex.props",
        "Properties",
        icons::INFO,
        Properties,
    )];
    t.status = Some(View(Status));
    t.default_left = Some("ex.outline");
    t
}

/// Command handler for this document type.
pub fn handle(doc: &str, id: &'static str) {
    let mut all = STATE.write();
    let st = all.entry(doc.to_string()).or_default();
    match id {
        "ex.add" => {
            let n = st.items.len() + 1;
            st.items.push(format!("Item {n}"));
        }
        "ex.clear" => st.items.clear(),
        _ => {}
    }
}

#[allow(non_snake_case)]
fn Viewport() -> Element {
    let doc = use_document();
    let id = doc.as_ref().map(|d| d.id.clone()).unwrap_or_default();
    let name = doc.map(|d| d.name).unwrap_or_default();
    let st = state(&id);
    rsx! {
        div { class: "app-viewport",
            h2 { "{name}" }
            p { class: "app-muted", "Replace this viewport with your document's main view." }
            ul { class: "app-list",
                for (i, item) in st.items.iter().enumerate() {
                    li { key: "{i}", "{item}" }
                }
            }
        }
    }
}

#[allow(non_snake_case)]
fn Outline() -> Element {
    let id = use_document().map(|d| d.id).unwrap_or_default();
    let st = state(&id);
    rsx! {
        ul { class: "app-list app-pad",
            for (i, item) in st.items.iter().enumerate() {
                li { key: "{i}", "{item}" }
            }
            if st.items.is_empty() {
                li { class: "app-muted", "Use the toolbar (or Shift+A) to add items." }
            }
        }
    }
}

#[allow(non_snake_case)]
fn Properties() -> Element {
    let doc = use_document();
    let id = doc.as_ref().map(|d| d.id.clone()).unwrap_or_default();
    let name = doc.map(|d| d.name).unwrap_or_default();
    let st = state(&id);
    rsx! {
        dl { class: "app-props",
            dt { "Name" } dd { "{name}" }
            dt { "Items" } dd { "{st.items.len()}" }
        }
    }
}

#[allow(non_snake_case)]
fn Status() -> Element {
    let id = use_document().map(|d| d.id).unwrap_or_default();
    let st = state(&id);
    rsx! {
        span { "items: {st.items.len()}" }
    }
}
