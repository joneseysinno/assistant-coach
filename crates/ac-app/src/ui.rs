//! Small form widgets shared by the document types.

use dioxus::prelude::*;

#[component]
pub fn Field(label: String, #[props(default)] hint: String, children: Element) -> Element {
    rsx! {
        label { class: "app-field",
            span { class: "app-field-label", "{label}" }
            {children}
            if !hint.is_empty() {
                span { class: "app-field-hint", "{hint}" }
            }
        }
    }
}

#[component]
pub fn TextIn(
    value: String,
    on_change: EventHandler<String>,
    #[props(default)] placeholder: String,
    #[props(default)] class: String,
) -> Element {
    rsx! {
        input {
            class: "app-input {class}",
            r#type: "text",
            value: "{value}",
            placeholder: "{placeholder}",
            oninput: move |e| on_change.call(e.value()),
        }
    }
}

#[component]
pub fn TextArea(
    value: String,
    on_change: EventHandler<String>,
    #[props(default)] placeholder: String,
    #[props(default = 3)] rows: u32,
) -> Element {
    rsx! {
        textarea {
            class: "app-input app-textarea",
            rows: "{rows}",
            placeholder: "{placeholder}",
            value: "{value}",
            oninput: move |e| on_change.call(e.value()),
        }
    }
}

#[component]
pub fn NumIn(
    value: f64,
    on_change: EventHandler<f64>,
    #[props(default)] min: Option<f64>,
    #[props(default)] max: Option<f64>,
    #[props(default = 1.0)] step: f64,
    #[props(default)] class: String,
) -> Element {
    let shown = if value.fract() == 0.0 {
        format!("{}", value as i64)
    } else {
        format!("{value}")
    };
    rsx! {
        input {
            class: "app-input app-num {class}",
            r#type: "number",
            value: "{shown}",
            step: "{step}",
            min: min.map(|m| m.to_string()).unwrap_or_default(),
            max: max.map(|m| m.to_string()).unwrap_or_default(),
            oninput: move |e| {
                if let Ok(v) = e.value().parse::<f64>() {
                    on_change.call(v)
                }
            },
        }
    }
}

/// A select over `(value, label)` options.
#[component]
pub fn Pick(
    value: String,
    options: Vec<(String, String)>,
    on_change: EventHandler<String>,
    #[props(default)] class: String,
) -> Element {
    rsx! {
        select {
            class: "app-input app-select {class}",
            onchange: move |e| on_change.call(e.value()),
            for (v, l) in options.iter() {
                option { key: "{v}", value: "{v}", selected: *v == value, "{l}" }
            }
        }
    }
}

pub fn opts(table: &[(&str, &str)]) -> Vec<(String, String)> {
    table
        .iter()
        .map(|(a, b)| (a.to_string(), b.to_string()))
        .collect()
}

pub fn opts_with_none(none: &str, table: &[(&str, &str)]) -> Vec<(String, String)> {
    let mut v = vec![(String::new(), none.to_string())];
    v.extend(opts(table));
    v
}

#[component]
pub fn Check(value: bool, label: String, on_change: EventHandler<bool>) -> Element {
    rsx! {
        label { class: "app-check",
            input {
                r#type: "checkbox",
                checked: value,
                onchange: move |e| on_change.call(e.checked()),
            }
            span { "{label}" }
        }
    }
}

/// Removable chips + an "add" select.
#[component]
pub fn Chips(
    values: Vec<String>,
    options: Vec<(String, String)>,
    on_change: EventHandler<Vec<String>>,
    #[props(default)] add_label: String,
) -> Element {
    let label_of = |v: &str| {
        options
            .iter()
            .find(|o| o.0 == v)
            .map(|o| o.1.clone())
            .unwrap_or_else(|| v.to_string())
    };
    let remaining: Vec<(String, String)> = options
        .iter()
        .filter(|o| !values.contains(&o.0))
        .cloned()
        .collect();
    let add = if add_label.is_empty() {
        "+ add".to_string()
    } else {
        add_label.clone()
    };
    rsx! {
        div { class: "app-chips",
            for v in values.iter() {
                span { key: "{v}", class: "app-chip",
                    "{label_of(v)}"
                    button {
                        class: "app-chip-x",
                        title: "Remove",
                        onclick: {
                            let v = v.clone();
                            let values = values.clone();
                            move |_| on_change.call(values.iter().filter(|x| **x != v).cloned().collect())
                        },
                        "×"
                    }
                }
            }
            if !remaining.is_empty() {
                select {
                    class: "app-input app-chip-add",
                    onchange: {
                        let values = values.clone();
                        move |e: FormEvent| {
                            let v = e.value();
                            if !v.is_empty() {
                                let mut n = values.clone();
                                n.push(v);
                                on_change.call(n);
                            }
                        }
                    },
                    option { value: "", selected: true, "{add}" }
                    for (v, l) in remaining.iter() {
                        option { key: "{v}", value: "{v}", "{l}" }
                    }
                }
            }
        }
    }
}

/// Free-text list edited as comma-separated words.
#[component]
pub fn WordList(
    values: Vec<String>,
    on_change: EventHandler<Vec<String>>,
    #[props(default)] placeholder: String,
) -> Element {
    let text = values.join(", ");
    rsx! {
        input {
            class: "app-input",
            r#type: "text",
            value: "{text}",
            placeholder: "{placeholder}",
            onchange: move |e| {
                let v: Vec<String> = e
                    .value()
                    .split(',')
                    .map(|s| s.trim().replace(' ', "-"))
                    .filter(|s| !s.is_empty())
                    .collect();
                on_change.call(v)
            },
        }
    }
}

#[component]
pub fn Section(
    title: String,
    #[props(default)] actions: Option<Element>,
    children: Element,
) -> Element {
    rsx! {
        section { class: "app-section",
            div { class: "app-section-head",
                h3 { "{title}" }
                if let Some(a) = actions { div { class: "app-section-actions", {a} } }
            }
            {children}
        }
    }
}

#[component]
pub fn Empty(text: String) -> Element {
    rsx! { p { class: "app-empty", "{text}" } }
}
