//! The shared court canvas (03 §8 `court`): court lines, zones, players, actions.
//! Basket frame, feet; the basket is at the top (S3 §2). The same drawing is produced as an
//! SVG string for printable exports.

use ac_sport_basketball::court::{self, Court, Geom, Pt};
use dioxus::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub struct ZoneDraw {
    pub id: String,
    pub label: String,
    pub kind: String,
    pub geom: Geom,
    pub selected: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PlayerDraw {
    pub id: String,
    pub at: Pt,
    pub team: String,
    pub ball: bool,
    pub selected: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ArrowDraw {
    /// pass | dribble | cut | screen | handoff | shot | move
    pub kind: String,
    pub from: Pt,
    pub to: Pt,
    pub strong: bool,
}

fn f(x: f64) -> String {
    format!("{x:.2}")
}

pub fn poly_d(p: &[Pt], closed: bool) -> String {
    let mut s = String::new();
    for (i, q) in p.iter().enumerate() {
        s.push_str(if i == 0 { "M" } else { "L" });
        s.push_str(&format!("{} {} ", f(q[0]), f(q[1])));
    }
    if closed {
        s.push('Z');
    }
    s
}

/// `(fill path, clip path)` for a zone outline.
pub fn geom_paths(g: &Geom) -> Vec<(String, Option<String>)> {
    match g {
        Geom::Poly(p) => vec![(poly_d(p, true), None)],
        Geom::Diff { outer, holes } => {
            let mut d = poly_d(outer, true);
            for h in holes {
                d.push(' ');
                d.push_str(&poly_d(h, true));
            }
            vec![(d, Some(poly_d(outer, true)))]
        }
        Geom::Many(v) => v.iter().flat_map(geom_paths).collect(),
    }
}

pub fn zone_color(kind: &str) -> &'static str {
    match kind {
        "bonus" => "#2fbf71",
        "restricted" | "no-dribble" => "#e5484d",
        "safe" => "#3e9be5",
        "start" => "#f5a524",
        "target" => "#a46cf5",
        _ => "#8b8d98",
    }
}

pub fn team_color(team: &str) -> &'static str {
    match team {
        "offense" => "#e5484d",
        "defense" => "#3e63dd",
        _ => "#f5a524",
    }
}

/// Where to draw roles on a game diagram (formats don't store positions).
pub fn formation(roles: &[ac_domain::Role]) -> Vec<(String, String, Pt)> {
    let spots: [&[Pt]; 6] = [
        &[],
        &[[0.0, 22.0]],
        &[[0.0, 22.0], [-15.0, 15.0]],
        &[[0.0, 23.0], [-16.0, 14.0], [16.0, 14.0]],
        &[[-8.0, 22.0], [8.0, 22.0], [-21.0, 2.0], [21.0, 2.0]],
        &[
            [0.0, 24.0],
            [-17.0, 15.0],
            [17.0, 15.0],
            [-21.0, 1.0],
            [21.0, 1.0],
        ],
    ];
    let mut out = Vec::new();
    for team in ["offense", "defense", "neutral"] {
        let ids: Vec<&ac_domain::Role> = roles.iter().filter(|r| r.team == team).collect();
        let n = ids.len();
        for (i, r) in ids.iter().enumerate() {
            let base = if n <= 5 {
                spots[n.clamp(1, 5)].get(i).copied().unwrap_or([0.0, 10.0])
            } else {
                let a = (-80.0 + 160.0 * i as f64 / (n - 1) as f64).to_radians();
                [21.0 * a.sin(), 21.0 * a.cos().max(0.05)]
            };
            let p = match team {
                "defense" => [base[0] * 0.78, base[1] * 0.78],
                "neutral" => [if i == 0 { -6.0 } else { 6.0 }, 13.75],
                _ => base,
            };
            out.push((r.id.clone(), r.team.clone(), p));
        }
    }
    out
}

fn arrow_path(a: &ArrowDraw) -> String {
    let (p, q) = (a.from, a.to);
    if a.kind.starts_with("dribble") {
        // zig-zag
        let (dx, dy) = (q[0] - p[0], q[1] - p[1]);
        let len = (dx * dx + dy * dy).sqrt().max(0.01);
        let (ux, uy) = (dx / len, dy / len);
        let (nx, ny) = (-uy, ux);
        let n = ((len / 1.6) as usize).max(2);
        let mut s = format!("M{} {} ", f(p[0]), f(p[1]));
        for i in 1..n {
            let t = i as f64 / n as f64;
            let side = if i % 2 == 0 { 0.6 } else { -0.6 };
            s.push_str(&format!(
                "L{} {} ",
                f(p[0] + dx * t + nx * side),
                f(p[1] + dy * t + ny * side)
            ));
        }
        s.push_str(&format!("L{} {}", f(q[0] - ux * 0.8), f(q[1] - uy * 0.8)));
        s
    } else {
        format!("M{} {} L{} {}", f(p[0]), f(p[1]), f(q[0]), f(q[1]))
    }
}

/// Court → client mapping for dragging (viewBox, preserveAspectRatio meet).
#[derive(Clone, Copy, Debug, PartialEq, Default)]
struct Mapping {
    left: f64,
    top: f64,
    width: f64,
    height: f64,
}

fn to_court(m: Mapping, vb: [f64; 4], cx: f64, cy: f64) -> Pt {
    if m.width <= 0.0 || m.height <= 0.0 {
        return [0.0, 0.0];
    }
    let s = (m.width / vb[2]).min(m.height / vb[3]);
    let ox = m.left + (m.width - vb[2] * s) / 2.0;
    let oy = m.top + (m.height - vb[3] * s) / 2.0;
    [vb[0] + (cx - ox) / s, vb[1] + (cy - oy) / s]
}

#[component]
pub fn CourtView(
    preset: String,
    region: String,
    zones: Vec<ZoneDraw>,
    players: Vec<PlayerDraw>,
    arrows: Vec<ArrowDraw>,
    #[props(default)] caption: String,
    #[props(default)] on_zone: Option<EventHandler<String>>,
    #[props(default)] on_player: Option<EventHandler<String>>,
    #[props(default)] on_move: Option<EventHandler<(String, Pt)>>,
) -> Element {
    let c: Court = court::preset(&preset);
    let r = c.region_rect(&region);
    let vb = [r[0] - 2.5, r[1] - 2.5, r[2] - r[0] + 5.0, r[3] - r[1] + 5.0];
    let lines = c.lines(&region);
    let mut dragging = use_signal(|| None::<String>);
    let mut mounted = use_signal(|| None::<std::rc::Rc<MountedData>>);
    let mut mapping = use_signal(Mapping::default);
    let draggable = on_move.is_some();
    let uid = use_hook(|| {
        static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(1);
        crate::intern(&format!(
            "c{}",
            NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ))
    });

    let refresh = move || async move {
        let js = format!(
            "const r = document.getElementById('{uid}').getBoundingClientRect(); return [r.left, r.top, r.width, r.height];"
        );
        if let Ok(v) = document::eval(&js).join::<Vec<f64>>().await {
            if v.len() == 4 {
                mapping.set(Mapping {
                    left: v[0],
                    top: v[1],
                    width: v[2],
                    height: v[3],
                });
            }
        }
    };
    let _ = &mut mounted;

    rsx! {
        div { class: "app-court-wrap",
            svg {
                id: "{uid}",
                class: if draggable { "app-court app-court-edit" } else { "app-court" },
                view_box: "{vb[0]} {vb[1]} {vb[2]} {vb[3]}",
                onmounted: move |e| {
                    mounted.set(Some(e.data()));
                    spawn(refresh());
                },
                onmousemove: move |e| {
                    let Some(id) = dragging.read().clone() else { return };
                    if mapping.read().width <= 0.0 { return; }
                    let p = e.client_coordinates();
                    let at = to_court(*mapping.read(), vb, p.x, p.y);
                    let at = [
                        (at[0].clamp(r[0], r[2]) * 2.0).round() / 2.0,
                        (at[1].clamp(r[1], r[3]) * 2.0).round() / 2.0,
                    ];
                    if let Some(h) = &on_move {
                        h.call((id, at));
                    }
                },
                onmouseup: move |_| dragging.set(None),
                onmouseleave: move |_| dragging.set(None),
                defs {
                    marker {
                        id: "{uid}-head",
                        view_box: "0 0 10 10",
                        ref_x: "8",
                        ref_y: "5",
                        marker_width: "5",
                        marker_height: "5",
                        orient: "auto-start-reverse",
                        path { d: "M0 0 L10 5 L0 10 z", fill: "currentColor" }
                    }
                }
                rect {
                    x: "{r[0]}", y: "{r[1]}",
                    width: "{r[2] - r[0]}", height: "{r[3] - r[1]}",
                    class: "app-court-floor",
                }
                for (i, z) in zones.iter().enumerate() {
                    g {
                        key: "z{i}",
                        class: if z.selected { "app-zone app-zone-sel" } else { "app-zone" },
                        onclick: {
                            let id = z.id.clone();
                            let h = on_zone;
                            move |_| if let Some(h) = &h { h.call(id.clone()) }
                        },
                        for (k, (d, clip)) in geom_paths(&z.geom).into_iter().enumerate() {
                            if let Some(cp) = clip {
                                clipPath { id: "{uid}-z{i}-{k}", path { d: "{cp}" } }
                                path {
                                    d: "{d}",
                                    fill: zone_color(&z.kind),
                                    fill_rule: "evenodd",
                                    clip_path: "url(#{uid}-z{i}-{k})",
                                }
                            } else {
                                path { d: "{d}", fill: zone_color(&z.kind) }
                            }
                        }
                        {
                            let c = z.geom.centroid();
                            rsx! {
                                text {
                                    x: "{c[0]:.1}", y: "{c[1]:.1}",
                                    class: "app-zone-label",
                                    text_anchor: "middle",
                                    dominant_baseline: "middle",
                                    "{z.label}"
                                }
                            }
                        }
                    }
                }
                for (i, (pts, closed)) in lines.iter().enumerate() {
                    path { key: "l{i}", d: poly_d(pts, *closed), class: "app-court-line" }
                }
                for (i, a) in arrows.iter().enumerate() {
                    g { key: "a{i}",
                        class: if a.strong { "app-arrow app-arrow-strong app-arrow-{a.kind}" } else { "app-arrow app-arrow-{a.kind}" },
                        if a.kind.starts_with("screen") {
                            {
                                let (p, q) = (a.from, a.to);
                                let (dx, dy) = (q[0] - p[0], q[1] - p[1]);
                                let len = (dx * dx + dy * dy).sqrt().max(0.01);
                                let (nx, ny) = (-dy / len * 1.4, dx / len * 1.4);
                                rsx! {
                                    path { d: "M{p[0]} {p[1]} L{q[0]} {q[1]}" }
                                    path { d: "M{q[0] - nx} {q[1] - ny} L{q[0] + nx} {q[1] + ny}", class: "app-arrow-bar" }
                                }
                            }
                        } else {
                            path {
                                d: arrow_path(a),
                                marker_end: if !a.kind.starts_with("shot") { "url(#{uid}-head)" } else { "" },
                            }
                        }
                    }
                }
                for p in players.iter() {
                    g {
                        key: "{p.id}",
                        class: if p.selected { "app-player app-player-sel" } else { "app-player" },
                        transform: "translate({p.at[0]:.2} {p.at[1]:.2})",
                        onmousedown: {
                            let id = p.id.clone();
                            let h = on_player;
                            move |e: MouseEvent| {
                                e.stop_propagation();
                                if let Some(h) = &h { h.call(id.clone()) }
                                if draggable {
                                    dragging.set(Some(id.clone()));
                                    spawn(refresh());
                                }
                            }
                        },
                        if p.team == "defense" {
                            path { d: "M-1.5 -0.4 L0 1.3 L1.5 -0.4", class: "app-player-d", stroke: team_color(&p.team) }
                            circle { r: "1.6", class: "app-player-hit" }
                            text { y: "-1.3", class: "app-player-label app-player-label-d", text_anchor: "middle", fill: team_color(&p.team), "{p.id}" }
                        } else {
                            circle { r: "1.5", fill: team_color(&p.team), class: "app-player-dot" }
                            text { class: "app-player-label", text_anchor: "middle", dominant_baseline: "central", "{p.id}" }
                        }
                        if p.ball {
                            circle { cx: "1.4", cy: "-1.3", r: "0.75", class: "app-ball" }
                        }
                    }
                }
            }
            if !caption.is_empty() {
                p { class: "app-court-caption", "{caption}" }
            }
        }
    }
}

/// The same drawing as a standalone SVG string (for printed cards and plans).
pub fn svg_string(
    preset: &str,
    region: &str,
    zones: &[ZoneDraw],
    players: &[PlayerDraw],
    width_px: u32,
) -> String {
    let c = court::preset(preset);
    let r = c.region_rect(region);
    let vb = [r[0] - 2.5, r[1] - 2.5, r[2] - r[0] + 5.0, r[3] - r[1] + 5.0];
    let mut s = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="{} {} {} {}" width="{width_px}" style="font-family:system-ui,sans-serif">"#,
        vb[0], vb[1], vb[2], vb[3]
    );
    s.push_str(&format!(
        r##"<rect x="{}" y="{}" width="{}" height="{}" fill="#f4ead7" stroke="none"/>"##,
        r[0],
        r[1],
        r[2] - r[0],
        r[3] - r[1]
    ));
    for (i, z) in zones.iter().enumerate() {
        for (k, (d, clip)) in geom_paths(&z.geom).into_iter().enumerate() {
            match clip {
                Some(cp) => s.push_str(&format!(
                    r#"<clipPath id="p{i}{k}"><path d="{cp}"/></clipPath><path d="{d}" fill="{}" fill-opacity="0.35" fill-rule="evenodd" clip-path="url(#p{i}{k})"/>"#,
                    zone_color(&z.kind)
                )),
                None => s.push_str(&format!(
                    r#"<path d="{d}" fill="{}" fill-opacity="0.35"/>"#,
                    zone_color(&z.kind)
                )),
            }
        }
    }
    for (pts, closed) in c.lines(region) {
        s.push_str(&format!(
            r##"<path d="{}" fill="none" stroke="#5b4a32" stroke-width="0.25"/>"##,
            poly_d(&pts, closed)
        ));
    }
    for p in players {
        let col = team_color(&p.team);
        if p.team == "defense" {
            s.push_str(&format!(
                r#"<g transform="translate({:.2} {:.2})"><path d="M-1.5 -0.4 L0 1.3 L1.5 -0.4" fill="none" stroke="{col}" stroke-width="0.45"/><text y="-1.3" text-anchor="middle" font-size="1.6" fill="{col}">{}</text></g>"#,
                p.at[0], p.at[1], p.id
            ));
        } else {
            s.push_str(&format!(
                r##"<g transform="translate({:.2} {:.2})"><circle r="1.5" fill="{col}"/><text text-anchor="middle" dominant-baseline="central" font-size="1.3" font-weight="700" fill="#fff">{}</text></g>"##,
                p.at[0], p.at[1], p.id
            ));
        }
    }
    s.push_str("</svg>");
    s
}
