//! Court geometry (S3): presets, court regions, named regions, zone shapes.
//!
//! Basket frame, feet: origin at the rim centre, +y toward half court, +x to the right of a
//! player facing the basket. Diagrams draw the basket at the top, so SVG y = basket-frame y.
//!
//! Every named region has an exact `contains` (the spec's definition) and an outline for
//! drawing. Outlines may be "outer minus holes"; the Studio draws those with an even-odd
//! fill clipped to the outer polygon.

use ac_domain::{Shape, Val};

pub type Pt = [f64; 2];

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Court {
    pub id: &'static str,
    pub label: &'static str,
    pub length: f64,
    pub width: f64,
    pub rim_offset: f64,
    pub lane_width: f64,
    /// Free-throw line, measured from the rim centre.
    pub ft_line: f64,
    pub arc_radius: f64,
    /// |x| of the straight corner segments of the 3-point line.
    pub corner_x: f64,
    pub restricted: Option<f64>,
}

/// S3 §3 presets.
pub const PRESETS: &[Court] = &[
    Court {
        id: "hs",
        label: "High school (84 × 50)",
        length: 84.0,
        width: 50.0,
        rim_offset: 5.25,
        lane_width: 12.0,
        ft_line: 13.75,
        arc_radius: 19.75,
        corner_x: 19.75,
        restricted: None,
    },
    Court {
        id: "ms",
        label: "Middle school (74 × 42)",
        length: 74.0,
        width: 42.0,
        rim_offset: 5.25,
        lane_width: 12.0,
        ft_line: 13.75,
        arc_radius: 19.75,
        corner_x: 19.75,
        restricted: None,
    },
    Court {
        id: "ncaa",
        label: "College (94 × 50)",
        length: 94.0,
        width: 50.0,
        rim_offset: 5.25,
        lane_width: 12.0,
        ft_line: 13.75,
        arc_radius: 22.15,
        corner_x: 21.65,
        restricted: Some(4.0),
    },
    Court {
        id: "nba",
        label: "NBA (94 × 50)",
        length: 94.0,
        width: 50.0,
        rim_offset: 5.25,
        lane_width: 16.0,
        ft_line: 13.75,
        arc_radius: 23.75,
        corner_x: 22.0,
        restricted: Some(4.0),
    },
];

pub fn preset(id: &str) -> Court {
    PRESETS
        .iter()
        .copied()
        .find(|c| c.id == id)
        .unwrap_or(PRESETS[0])
}

/// A drawable area.
#[derive(Clone, Debug, PartialEq)]
pub enum Geom {
    Poly(Vec<Pt>),
    /// `outer` minus every hole.
    Diff {
        outer: Vec<Pt>,
        holes: Vec<Vec<Pt>>,
    },
    Many(Vec<Geom>),
}

impl Geom {
    /// Rough centre, for labels.
    pub fn centroid(&self) -> Pt {
        match self {
            Geom::Poly(p) | Geom::Diff { outer: p, .. } => area_centroid(p),
            Geom::Many(v) => v.first().map(|g| g.centroid()).unwrap_or([0.0, 0.0]),
        }
    }
}

pub fn rect_poly(x0: f64, y0: f64, x1: f64, y1: f64) -> Vec<Pt> {
    let (ax, bx) = (x0.min(x1), x0.max(x1));
    let (ay, by) = (y0.min(y1), y0.max(y1));
    vec![[ax, ay], [bx, ay], [bx, by], [ax, by]]
}

pub fn circle_poly(c: Pt, r: f64) -> Vec<Pt> {
    (0..72)
        .map(|i| {
            let a = (i as f64) * 5.0f64.to_radians();
            [c[0] + r * a.cos(), c[1] + r * a.sin()]
        })
        .collect()
}

/// Annular sector about the rim: radii `r`, angles from +y (deg; positive toward +x).
pub fn band_poly(r: [f64; 2], deg: [f64; 2]) -> Vec<Pt> {
    let steps = (((deg[1] - deg[0]).abs() / 2.0).ceil() as usize).max(1);
    let at = |rad: f64, d: f64| {
        let a = d.to_radians();
        [rad * a.sin(), rad * a.cos()]
    };
    let mut pts: Vec<Pt> = (0..=steps)
        .map(|i| at(r[1], deg[0] + (deg[1] - deg[0]) * i as f64 / steps as f64))
        .collect();
    if r[0] > 0.0 {
        pts.extend(
            (0..=steps)
                .rev()
                .map(|i| at(r[0], deg[0] + (deg[1] - deg[0]) * i as f64 / steps as f64)),
        );
    } else {
        pts.push([0.0, 0.0]);
    }
    pts
}

pub fn poly_area(p: &[Pt]) -> f64 {
    let n = p.len();
    if n < 3 {
        return 0.0;
    }
    let mut s = 0.0;
    for i in 0..n {
        let a = p[i];
        let b = p[(i + 1) % n];
        s += a[0] * b[1] - b[0] * a[1];
    }
    s.abs() / 2.0
}

/// Area-weighted centroid (falls back to the vertex mean for degenerate polygons).
pub fn area_centroid(p: &[Pt]) -> Pt {
    let n = p.len();
    let (mut a, mut cx, mut cy) = (0.0, 0.0, 0.0);
    for i in 0..n {
        let (p0, p1) = (p[i], p[(i + 1) % n]);
        let cross = p0[0] * p1[1] - p1[0] * p0[1];
        a += cross;
        cx += (p0[0] + p1[0]) * cross;
        cy += (p0[1] + p1[1]) * cross;
    }
    if a.abs() < 1e-9 {
        let m = n.max(1) as f64;
        return [
            p.iter().map(|q| q[0]).sum::<f64>() / m,
            p.iter().map(|q| q[1]).sum::<f64>() / m,
        ];
    }
    [cx / (3.0 * a), cy / (3.0 * a)]
}

pub fn point_in_poly(pt: Pt, poly: &[Pt]) -> bool {
    let mut inside = false;
    let n = poly.len();
    let mut j = n.wrapping_sub(1);
    for i in 0..n {
        let (a, b) = (poly[i], poly[j]);
        if ((a[1] > pt[1]) != (b[1] > pt[1]))
            && (pt[0] < (b[0] - a[0]) * (pt[1] - a[1]) / (b[1] - a[1]) + a[0])
        {
            inside = !inside;
        }
        j = i;
    }
    inside
}

/// Sutherland–Hodgman: clip `subject` to the convex polygon `clip` (counter-clockwise).
fn clip_convex(subject: &[Pt], clip: &[Pt]) -> Vec<Pt> {
    let mut out = subject.to_vec();
    let n = clip.len();
    for i in 0..n {
        let (a, b) = (clip[i], clip[(i + 1) % n]);
        let inside = |p: Pt| (b[0] - a[0]) * (p[1] - a[1]) - (b[1] - a[1]) * (p[0] - a[0]) >= 0.0;
        let hit = |p: Pt, q: Pt| {
            let (x1, y1, x2, y2) = (p[0], p[1], q[0], q[1]);
            let (x3, y3, x4, y4) = (a[0], a[1], b[0], b[1]);
            let den = (x1 - x2) * (y3 - y4) - (y1 - y2) * (x3 - x4);
            if den.abs() < 1e-12 {
                return p;
            }
            let t = ((x1 - x3) * (y3 - y4) - (y1 - y3) * (x3 - x4)) / den;
            [x1 + t * (x2 - x1), y1 + t * (y2 - y1)]
        };
        let input = std::mem::take(&mut out);
        if input.is_empty() {
            break;
        }
        let m = input.len();
        for k in 0..m {
            let cur = input[k];
            let prev = input[(k + m - 1) % m];
            match (inside(cur), inside(prev)) {
                (true, true) => out.push(cur),
                (true, false) => {
                    out.push(hit(prev, cur));
                    out.push(cur);
                }
                (false, true) => out.push(hit(prev, cur)),
                (false, false) => {}
            }
        }
    }
    out
}

impl Court {
    /// Half-court line (y) in basket frame.
    pub fn half_line(&self) -> f64 {
        self.length / 2.0 - self.rim_offset
    }
    pub fn baseline(&self) -> f64 {
        -self.rim_offset
    }
    /// y where the 3-point arc meets the straight corner segments.
    pub fn arc_break(&self) -> f64 {
        (self.arc_radius.powi(2) - self.corner_x.powi(2))
            .max(0.0)
            .sqrt()
    }

    /// Playable rectangle of a court region `[x0, y0, x1, y1]` (S3 §5).
    pub fn region_rect(&self, region: &str) -> [f64; 4] {
        let w = self.width / 2.0;
        let b = self.baseline();
        match region {
            "full" => [-w, b, w, self.length - self.rim_offset],
            "three-quarter" => [-w, b, w, self.length * 0.75 - self.rim_offset],
            "cross-court" => {
                let hx = self.length / 4.0;
                [-hx, b, hx, self.width - self.rim_offset]
            }
            _ => [-w, b, w, self.half_line()],
        }
    }

    pub fn region_area(&self, region: &str) -> f64 {
        if region == "inside-arc" {
            return poly_area(&self.inside_arc_poly());
        }
        let r = self.region_rect(region);
        (r[2] - r[0]) * (r[3] - r[1])
    }

    /// The 3-point line from the left baseline to the right baseline.
    pub fn arc_line(&self) -> Vec<Pt> {
        let b = self.baseline();
        let yb = self.arc_break();
        let start = (self.corner_x / self.arc_radius)
            .clamp(-1.0, 1.0)
            .asin()
            .to_degrees();
        let mut pts = vec![[-self.corner_x, b], [-self.corner_x, yb]];
        let steps = 90;
        for i in 0..=steps {
            let d = -start + (2.0 * start) * i as f64 / steps as f64;
            let a = d.to_radians();
            pts.push([self.arc_radius * a.sin(), self.arc_radius * a.cos()]);
        }
        pts.push([self.corner_x, yb]);
        pts.push([self.corner_x, b]);
        pts
    }

    pub fn inside_arc_poly(&self) -> Vec<Pt> {
        self.arc_line()
    }

    fn inside_arc(&self, p: Pt) -> bool {
        if p[1] < self.baseline() {
            return false;
        }
        if p[1] <= self.arc_break() {
            p[0].abs() <= self.corner_x
        } else {
            (p[0] * p[0] + p[1] * p[1]).sqrt() <= self.arc_radius
        }
    }

    /// Angle from +y in degrees (0 = straight out from the rim, + toward +x).
    fn angle(p: Pt) -> f64 {
        p[0].atan2(p[1]).to_degrees()
    }

    fn lane(&self) -> f64 {
        self.lane_width / 2.0
    }

    /// Exact membership for a named region (S3 §7). `None` if the id isn't a region.
    pub fn contains_named(&self, id: &str, p: Pt) -> Option<bool> {
        let b = self.baseline();
        let l = self.lane();
        let outside = !self.inside_arc(p);
        let in_court = {
            let r = self.region_rect("half");
            p[0] >= r[0] && p[0] <= r[2] && p[1] >= r[1]
        };
        let circ = |c: Pt, r: f64| ((p[0] - c[0]).powi(2) + (p[1] - c[1]).powi(2)).sqrt() <= r;
        let rect = |x0: f64, x1: f64, y0: f64, y1: f64| {
            p[0] >= x0.min(x1) && p[0] <= x0.max(x1) && p[1] >= y0 && p[1] <= y1
        };
        Some(match id {
            "paint" => rect(-l, l, b, self.ft_line),
            "restricted" => self
                .restricted
                .is_some_and(|r| p[1] >= 0.0 && circ([0.0, 0.0], r)),
            "block-L" => circ([-(l + 1.0), 1.75], 2.0),
            "block-R" => circ([l + 1.0, 1.75], 2.0),
            "short-corner-L" => rect(-14.0, -l, b, 4.0),
            "short-corner-R" => rect(l, 14.0, b, 4.0),
            "dunker-L" => rect(-(l + 4.0), -l, b, 2.0),
            "dunker-R" => rect(l, l + 4.0, b, 2.0),
            "elbow-L" => circ([-l, self.ft_line], 2.5),
            "elbow-R" => circ([l, self.ft_line], 2.5),
            "corner-L" => in_court && outside && p[1] < 3.0 && p[0] < -self.corner_x,
            "corner-R" => in_court && outside && p[1] < 3.0 && p[0] > self.corner_x,
            "wing-L" | "wing-R" => {
                let a = Self::angle(p);
                let side = if id == "wing-L" { a < 0.0 } else { a > 0.0 };
                in_court && outside && p[1] >= 3.0 && side && (30.0..=75.0).contains(&a.abs())
            }
            "top" => in_court && outside && p[1] > 0.0 && Self::angle(p).abs() < 30.0,
            "above-break" => in_court && outside && p[1] >= 3.0,
            "mid-range" => self.inside_arc(p) && !rect(-l, l, b, self.ft_line),
            "inside-arc" => self.inside_arc(p),
            "beyond-arc" => in_court && outside,
            "backcourt" => p[1] > self.half_line(),
            _ => return None,
        })
    }

    /// A ray from the rim at `deg` hits the 3-point line at this distance.
    fn arc_hit(&self, deg: f64) -> f64 {
        let a = deg.to_radians();
        let (s, c) = (a.sin().abs(), a.cos());
        let r = self.arc_radius;
        if r * c >= self.arc_break() || s < 1e-9 {
            r
        } else {
            self.corner_x / s
        }
    }

    fn beyond_sector(&self, d0: f64, d1: f64, ymin: f64, region: &str) -> Vec<Pt> {
        let mut pts = Vec::new();
        let steps = 40;
        for i in 0..=steps {
            let d = d0 + (d1 - d0) * i as f64 / steps as f64;
            let r = self.arc_hit(d);
            let a = d.to_radians();
            pts.push([r * a.sin(), r * a.cos()]);
        }
        for d in [d1, d0] {
            let a = d.to_radians();
            pts.push([200.0 * a.sin(), 200.0 * a.cos()]);
        }
        let r = self.region_rect(region);
        clip_convex(&pts, &rect_poly(r[0], ymin.max(r[1]), r[2], r[3]))
    }

    /// Outline of a named region, clipped to the court region.
    pub fn named_geom(&self, id: &str, region: &str) -> Option<Geom> {
        let b = self.baseline();
        let l = self.lane();
        let r = self.region_rect(region);
        let top_y = r[3];
        Some(match id {
            "half" | "full" | "three-quarter" | "cross-court" => {
                let q = self.region_rect(id);
                Geom::Poly(rect_poly(q[0], q[1], q[2], q[3]))
            }
            "paint" => Geom::Poly(rect_poly(-l, b, l, self.ft_line)),
            "restricted" => Geom::Poly(band_poly([0.0, self.restricted?], [-90.0, 90.0])),
            "block-L" => Geom::Poly(circle_poly([-(l + 1.0), 1.75], 2.0)),
            "block-R" => Geom::Poly(circle_poly([l + 1.0, 1.75], 2.0)),
            "short-corner-L" => Geom::Poly(rect_poly(-14.0, b, -l, 4.0)),
            "short-corner-R" => Geom::Poly(rect_poly(l, b, 14.0, 4.0)),
            "dunker-L" => Geom::Poly(rect_poly(-(l + 4.0), b, -l, 2.0)),
            "dunker-R" => Geom::Poly(rect_poly(l, b, l + 4.0, 2.0)),
            "elbow-L" => Geom::Poly(circle_poly([-l, self.ft_line], 2.5)),
            "elbow-R" => Geom::Poly(circle_poly([l, self.ft_line], 2.5)),
            "corner-L" => Geom::Poly(rect_poly(r[0], b, -self.corner_x, 3.0)),
            "corner-R" => Geom::Poly(rect_poly(self.corner_x, b, r[2], 3.0)),
            "wing-L" => Geom::Poly(self.beyond_sector(-75.0, -30.0, 3.0, region)),
            "wing-R" => Geom::Poly(self.beyond_sector(30.0, 75.0, 3.0, region)),
            "top" => Geom::Poly(self.beyond_sector(-30.0, 30.0, 0.0, region)),
            "above-break" => Geom::Diff {
                outer: rect_poly(r[0], 3.0, r[2], top_y),
                holes: vec![self.inside_arc_poly()],
            },
            "mid-range" => Geom::Diff {
                outer: self.inside_arc_poly(),
                holes: vec![rect_poly(-l, b, l, self.ft_line)],
            },
            "inside-arc" => Geom::Poly(self.inside_arc_poly()),
            "beyond-arc" => Geom::Diff {
                outer: rect_poly(r[0], r[1], r[2], top_y),
                holes: vec![self.inside_arc_poly()],
            },
            "backcourt" => Geom::Poly(rect_poly(
                r[0],
                self.half_line(),
                r[2],
                top_y.max(self.half_line()),
            )),
            _ => return None,
        })
    }

    /// Court markings for drawing a region: each item is a polyline (`closed` = polygon).
    pub fn lines(&self, region: &str) -> Vec<(Vec<Pt>, bool)> {
        let r = self.region_rect(region);
        let mut out = vec![(rect_poly(r[0], r[1], r[2], r[3]), true)];
        // backboard + rim
        out.push((vec![[-3.0, -1.25], [3.0, -1.25]], false));
        out.push((circle_poly([0.0, 0.0], 0.75), true));
        if region == "cross-court" {
            return out;
        }
        let b = self.baseline();
        let l = self.lane();
        out.push((rect_poly(-l, b, l, self.ft_line), true));
        out.push((
            band_poly([6.0, 6.0], [-90.0, 90.0])
                .into_iter()
                .take(91)
                .map(|p| [p[0], p[1] + self.ft_line])
                .collect(),
            false,
        ));
        out.push((self.arc_line(), false));
        if let Some(rr) = self.restricted {
            out.push((
                band_poly([rr, rr], [-90.0, 90.0])
                    .into_iter()
                    .take(91)
                    .collect(),
                false,
            ));
        }
        let half = self.half_line();
        if r[3] >= half - 0.01 {
            out.push((vec![[r[0], half], [r[2], half]], false));
            let ring = circle_poly([0.0, half], 6.0);
            let inside: Vec<Pt> = ring.into_iter().filter(|p| p[1] <= r[3] + 0.01).collect();
            if !inside.is_empty() {
                out.push((inside, false));
            }
        }
        if region == "full" {
            // the other end, mirrored about the half-court line
            let far = 2.0 * half;
            let m = |p: Pt| [p[0], far - p[1]];
            out.push((vec![m([-3.0, -1.25]), m([3.0, -1.25])], false));
            out.push((circle_poly([0.0, far], 0.75), true));
            out.push((rect_poly(-l, far - b, l, far - self.ft_line), true));
            out.push((self.arc_line().into_iter().map(m).collect(), false));
        }
        out
    }
}

fn num(v: &Val) -> f64 {
    v.as_f64().unwrap_or(0.0)
}

/// Outline of a (dial-bound) zone shape (S3 §6).
pub fn shape_geom(court: &Court, shape: &Shape, region: &str) -> Option<Geom> {
    if let Some(n) = &shape.named {
        return court.named_geom(n, region);
    }
    if let Some(r) = &shape.rect {
        return Some(Geom::Poly(rect_poly(
            num(&r[0]),
            num(&r[1]),
            num(&r[2]),
            num(&r[3]),
        )));
    }
    if let Some(c) = &shape.circle {
        return Some(Geom::Poly(circle_poly(
            [num(&c.center[0]), num(&c.center[1])],
            num(&c.r),
        )));
    }
    if let Some(b) = &shape.band {
        return Some(Geom::Poly(band_poly(
            [num(&b.r[0]), num(&b.r[1])],
            [num(&b.deg[0]), num(&b.deg[1])],
        )));
    }
    if let Some(p) = &shape.polygon {
        return Some(Geom::Poly(
            p.iter().map(|q| [num(&q[0]), num(&q[1])]).collect(),
        ));
    }
    if let Some(u) = &shape.union {
        return Some(Geom::Many(
            u.iter()
                .filter_map(|s| shape_geom(court, s, region))
                .collect(),
        ));
    }
    if let Some(m) = &shape.minus {
        let mut it = m.iter().filter_map(|s| shape_geom(court, s, region));
        let first = it.next()?;
        let outer = match first {
            Geom::Poly(p) | Geom::Diff { outer: p, .. } => p,
            Geom::Many(_) => return Some(first),
        };
        let holes = it
            .filter_map(|g| match g {
                Geom::Poly(p) | Geom::Diff { outer: p, .. } => Some(p),
                Geom::Many(_) => None,
            })
            .collect();
        return Some(Geom::Diff { outer, holes });
    }
    None
}

/// Point-in-zone for a (dial-bound) shape.
pub fn shape_contains(court: &Court, shape: &Shape, region: &str, p: Pt) -> bool {
    if let Some(n) = &shape.named {
        if let Some(b) = court.contains_named(n, p) {
            return b;
        }
        let r = court.region_rect(n);
        return p[0] >= r[0] && p[0] <= r[2] && p[1] >= r[1] && p[1] <= r[3];
    }
    if let Some(c) = &shape.circle {
        let (cx, cy, r) = (num(&c.center[0]), num(&c.center[1]), num(&c.r));
        return ((p[0] - cx).powi(2) + (p[1] - cy).powi(2)).sqrt() <= r;
    }
    if let Some(u) = &shape.union {
        return u.iter().any(|s| shape_contains(court, s, region, p));
    }
    if let Some(m) = &shape.minus {
        let mut it = m.iter();
        return it
            .next()
            .is_some_and(|f| shape_contains(court, f, region, p))
            && !it.any(|s| shape_contains(court, s, region, p));
    }
    match shape_geom(court, shape, region) {
        Some(Geom::Poly(poly)) => point_in_poly(p, &poly),
        _ => false,
    }
}

/// S3 §4: basket frame → gym frame for a basket at `position` facing `facing` degrees.
pub fn to_gym(local: Pt, position: Pt, facing: f64) -> Pt {
    let t = facing.to_radians();
    let f = [t.sin(), t.cos()];
    let r = [-f[1], f[0]];
    [
        position[0] + local[1] * f[0] + local[0] * r[0],
        position[1] + local[1] * f[1] + local[0] * r[1],
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-6
    }

    #[test]
    fn s3_t1_t2_half_court_areas() {
        assert!(close(preset("hs").region_area("half"), 2100.0));
        assert!(close(preset("ms").region_area("half"), 42.0 * 37.0));
    }

    #[test]
    fn s3_t3_to_t6_named_regions() {
        let hs = preset("hs");
        assert_eq!(hs.contains_named("corner-L", [-22.0, 1.0]), Some(true));
        assert_eq!(hs.contains_named("corner-L", [-18.0, 1.0]), Some(false));
        assert_eq!(hs.contains_named("top", [0.0, 20.0]), Some(true));
        assert_eq!(hs.contains_named("paint", [5.0, 10.0]), Some(true));
        assert_eq!(hs.contains_named("wing-R", [15.0, 15.0]), Some(true));
        assert_eq!(hs.contains_named("wing-R", [-15.0, 15.0]), Some(false));
    }

    #[test]
    fn s3_t7_area_per_player() {
        assert!(close(preset("hs").region_area("half") / 6.0, 350.0));
    }

    #[test]
    fn s3_t8_basket_to_gym() {
        let p = to_gym([0.0, 10.0], [-19.75, 18.0], 90.0);
        assert!(close(p[0], -9.75) && close(p[1], 18.0), "{p:?}");
        let q = to_gym([5.0, 0.0], [0.0, 36.75], 180.0);
        assert!(close(q[0], 5.0) && close(q[1], 36.75), "{q:?}");
    }

    #[test]
    fn s3_t9_dial_bound_circle() {
        let s = Shape::circle(0.0, 0.0, 12.0);
        assert!(shape_contains(&preset("hs"), &s, "half", [0.0, 11.9]));
    }

    #[test]
    fn drawn_outlines_agree_with_membership() {
        let hs = preset("hs");
        for id in [
            "corner-L", "corner-R", "wing-L", "wing-R", "top", "paint", "elbow-R",
        ] {
            let g = hs.named_geom(id, "half").unwrap();
            let c = g.centroid();
            assert_eq!(hs.contains_named(id, c), Some(true), "{id} centroid {c:?}");
        }
        assert!(poly_area(&hs.inside_arc_poly()) > 600.0);
    }
}
