//! The data the Studio edits (02 §2, S2–S8). Field names follow the YAML fixture format
//! (S5 §2, D-018). Every struct keeps unknown fields in `extra`, so loading and saving a
//! file never drops something this version doesn't understand yet.
//!
//! Sport words (court names, event ids, base scoring) are *values* here, never type or
//! field names — those live in the sport pack (D-005).

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::val::{OneOrMany, Val};

pub type Extra = IndexMap<String, serde_yaml::Value>;
pub type Point = [f64; 2];

fn is_false(b: &bool) -> bool {
    !*b
}

// ---------------------------------------------------------------------------------------
// Rule (P4) — S2
// ---------------------------------------------------------------------------------------

/// Who judges a rule (D-025).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Judge {
    #[default]
    Coach,
    Players,
    Auto,
}

impl Judge {
    pub const ALL: [Judge; 3] = [Judge::Coach, Judge::Players, Judge::Auto];
    pub fn as_str(self) -> &'static str {
        match self {
            Judge::Coach => "coach",
            Judge::Players => "players",
            Judge::Auto => "auto",
        }
    }
    pub fn parse(s: &str) -> Option<Judge> {
        Judge::ALL.into_iter().find(|j| j.as_str() == s)
    }
}

/// A rule's trigger (`when`) or one `after` step. Exactly one of `event`, `count`, `clock`,
/// `score`, `game`, `possession` is set (S2 §3); the others are filters.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Trigger {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub count: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub by: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub op: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<Val>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub zone: Option<OneOrMany>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub zone_not: Option<OneOrMany>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actor: Option<serde_yaml::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub team: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hand: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clock: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reaches: Option<Val>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub score: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub game: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub possession: Option<String>,
    #[serde(flatten, default, skip_serializing_if = "IndexMap::is_empty")]
    pub extra: Extra,
}

/// The trigger forms of S2 §3.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TriggerKind {
    Event,
    Count,
    Clock,
    Score,
    GameEnd,
    PossessionEnd,
    Unknown,
}

impl Trigger {
    pub fn event(kind: &str) -> Self {
        Trigger {
            event: Some(kind.to_string()),
            ..Default::default()
        }
    }

    pub fn kind(&self) -> TriggerKind {
        if self.count.is_some() {
            TriggerKind::Count
        } else if self.clock.is_some() {
            TriggerKind::Clock
        } else if self.score.is_some() {
            TriggerKind::Score
        } else if self.game.is_some() {
            TriggerKind::GameEnd
        } else if self.possession.is_some() {
            TriggerKind::PossessionEnd
        } else if self.event.is_some() {
            TriggerKind::Event
        } else {
            TriggerKind::Unknown
        }
    }

    /// The event this trigger talks about (`event` or the counted event).
    pub fn event_id(&self) -> Option<&str> {
        self.event.as_deref().or(self.count.as_deref())
    }
}

/// One consequence (`then` item, S2 §5). Struct-of-options so the editor can switch kinds.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Consequence {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub points: Option<Val>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cancel: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub turnover: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub possession: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub restart: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ball: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<serde_yaml::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub constraint: Option<serde_yaml::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub choose: Option<serde_yaml::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dial: Option<serde_yaml::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rotate: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tally: Option<bool>,
    #[serde(flatten, default, skip_serializing_if = "IndexMap::is_empty")]
    pub extra: Extra,
}

impl Consequence {
    pub fn points(n: i64) -> Self {
        Consequence {
            points: Some(Val::Int(n)),
            ..Default::default()
        }
    }
    pub fn turnover() -> Self {
        Consequence {
            turnover: Some(true),
            ..Default::default()
        }
    }
    pub fn tally() -> Self {
        Consequence {
            tally: Some(true),
            ..Default::default()
        }
    }
    /// Non-zero points (a literal 0 is "no basket", S2 §5).
    pub fn awards_points(&self) -> bool {
        match &self.points {
            None => false,
            Some(v) => v.as_f64().map(|n| n != 0.0).unwrap_or(true),
        }
    }
    pub fn is_turnover(&self) -> bool {
        self.turnover == Some(true)
    }
    pub fn is_tally(&self) -> bool {
        self.tally == Some(true)
    }
    pub fn changes_possession(&self) -> bool {
        self.is_turnover() || self.possession.as_deref() == Some("switch")
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Rule {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    pub when: Trigger,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub after: Vec<Trigger>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub within: Option<Val>,
    #[serde(default, rename = "if", skip_serializing_if = "Vec::is_empty")]
    pub conditions: Vec<serde_yaml::Value>,
    #[serde(default)]
    pub then: Vec<Consequence>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub overrides: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub judge: Option<Judge>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub side: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub say: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub board: Option<String>,
    #[serde(flatten, default, skip_serializing_if = "IndexMap::is_empty")]
    pub extra: Extra,
}

impl Rule {
    pub fn awards_points(&self) -> bool {
        self.then.iter().any(|c| c.awards_points())
    }
    /// S2 / D-025 default: `coach` if the rule awards points, `players` otherwise.
    pub fn effective_judge(&self) -> Judge {
        self.judge.unwrap_or(if self.awards_points() {
            Judge::Coach
        } else {
            Judge::Players
        })
    }
    pub fn points(&self) -> Option<&Consequence> {
        self.then.iter().find(|c| c.points.is_some())
    }
    pub fn display_label(&self) -> String {
        self.label.clone().unwrap_or_else(|| self.id.clone())
    }
}

// ---------------------------------------------------------------------------------------
// Dial (P6) — S5 §3
// ---------------------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DialKind {
    Int,
    Distance,
    Duration,
    Choice,
    Flag,
}

impl DialKind {
    pub fn as_str(self) -> &'static str {
        match self {
            DialKind::Int => "int",
            DialKind::Distance => "distance",
            DialKind::Duration => "duration",
            DialKind::Choice => "enum",
            DialKind::Flag => "bool",
        }
    }
    pub fn unit(self) -> &'static str {
        match self {
            DialKind::Distance => "ft",
            DialKind::Duration => "s",
            _ => "",
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Dial {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub int: Option<[Val; 2]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub distance: Option<[Val; 2]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration: Option<[Val; 2]>,
    #[serde(default, rename = "enum", skip_serializing_if = "Option::is_none")]
    pub choices: Option<Vec<String>>,
    #[serde(default, rename = "bool", skip_serializing_if = "Option::is_none")]
    pub flag: Option<bool>,
    #[serde(default)]
    pub default: Val,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub step: Option<Val>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stops: Option<IndexMap<String, Val>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub harder: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub levels: Option<Vec<Val>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub live: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visible_to: Option<String>,
    #[serde(flatten, default, skip_serializing_if = "IndexMap::is_empty")]
    pub extra: Extra,
}

impl Dial {
    pub fn int(min: i64, max: i64, default: i64) -> Self {
        Dial {
            int: Some([Val::Int(min), Val::Int(max)]),
            default: Val::Int(default),
            ..Default::default()
        }
    }

    pub fn kind(&self) -> DialKind {
        if self.int.is_some() {
            DialKind::Int
        } else if self.distance.is_some() {
            DialKind::Distance
        } else if self.duration.is_some() {
            DialKind::Duration
        } else if self.choices.is_some() {
            DialKind::Choice
        } else {
            DialKind::Flag
        }
    }

    /// `(min, max)` for numeric dials.
    pub fn bounds(&self) -> Option<(f64, f64)> {
        let b = self
            .int
            .as_ref()
            .or(self.distance.as_ref())
            .or(self.duration.as_ref())?;
        Some((b[0].as_f64()?, b[1].as_f64()?))
    }

    pub fn set_bounds(&mut self, min: f64, max: f64) {
        let pair = [Val::num(min), Val::num(max)];
        match self.kind() {
            DialKind::Int => self.int = Some(pair),
            DialKind::Distance => self.distance = Some(pair),
            DialKind::Duration => self.duration = Some(pair),
            _ => {}
        }
    }

    pub fn set_kind(&mut self, kind: DialKind) {
        let (lo, hi) = self.bounds().unwrap_or((0.0, 10.0));
        self.int = None;
        self.distance = None;
        self.duration = None;
        self.choices = None;
        self.flag = None;
        let pair = [Val::num(lo), Val::num(hi)];
        match kind {
            DialKind::Int => self.int = Some(pair),
            DialKind::Distance => self.distance = Some(pair),
            DialKind::Duration => self.duration = Some(pair),
            DialKind::Choice => {
                self.choices = Some(vec!["a".into(), "b".into()]);
                self.default = Val::Text("a".into());
            }
            DialKind::Flag => {
                self.flag = Some(true);
                self.default = Val::Bool(true);
            }
        }
    }

    pub fn display_name(&self, key: &str) -> String {
        self.label
            .clone()
            .unwrap_or_else(|| key.replace(['_', '-'], " "))
    }
}

// ---------------------------------------------------------------------------------------
// Space (P1) — S3
// ---------------------------------------------------------------------------------------

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Circle {
    pub center: [Val; 2],
    pub r: Val,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Band {
    pub r: [Val; 2],
    pub deg: [Val; 2],
}

/// A zone shape (S3 §6). Exactly one field is set.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Shape {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub named: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rect: Option<[Val; 4]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub circle: Option<Circle>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub band: Option<Band>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub polygon: Option<Vec<[Val; 2]>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub union: Option<Vec<Shape>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub minus: Option<Vec<Shape>>,
    #[serde(flatten, default, skip_serializing_if = "IndexMap::is_empty")]
    pub extra: Extra,
}

impl Shape {
    pub fn named(id: &str) -> Self {
        Shape {
            named: Some(id.to_string()),
            ..Default::default()
        }
    }
    pub fn rect(x0: f64, y0: f64, x1: f64, y1: f64) -> Self {
        Shape {
            rect: Some([Val::num(x0), Val::num(y0), Val::num(x1), Val::num(y1)]),
            ..Default::default()
        }
    }
    pub fn circle(cx: f64, cy: f64, r: f64) -> Self {
        Shape {
            circle: Some(Circle {
                center: [Val::num(cx), Val::num(cy)],
                r: Val::num(r),
            }),
            ..Default::default()
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Zone {
    pub id: String,
    #[serde(default)]
    pub kind: String,
    pub shape: Shape,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(flatten, default, skip_serializing_if = "IndexMap::is_empty")]
    pub extra: Extra,
}

/// The zone kinds of S3 §2 (styling only, D-034).
pub const ZONE_KINDS: [&str; 6] = [
    "bonus",
    "restricted",
    "no-dribble",
    "safe",
    "start",
    "target",
];

// ---------------------------------------------------------------------------------------
// Role (P2) and Format — S4
// ---------------------------------------------------------------------------------------

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Role {
    pub id: String,
    /// `offense` / `defense` / `neutral` — per-possession sides, not team names (S4 §2).
    pub team: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plays_for: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(flatten, default, skip_serializing_if = "IndexMap::is_empty")]
    pub extra: Extra,
}

impl Role {
    pub fn new(id: &str, team: &str) -> Self {
        Role {
            id: id.to_string(),
            team: team.to_string(),
            ..Default::default()
        }
    }
}

/// Role edits inside a constraint (S5 §2.1) — sugar the resolver compiles into rules.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct RoleEdit {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub add: Option<Role>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub restrict: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stay_in: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub penalty: Option<Val>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cannot: Option<String>,
    #[serde(flatten, default, skip_serializing_if = "IndexMap::is_empty")]
    pub extra: Extra,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct SpaceRef {
    #[serde(default)]
    pub court: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub basket: Option<String>,
    #[serde(flatten, default, skip_serializing_if = "IndexMap::is_empty")]
    pub extra: Extra,
}

/// A `type:` plus free fields (start, restart overrides…), kept as written.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Typed {
    #[serde(rename = "type", default)]
    pub kind: String,
    #[serde(flatten, default, skip_serializing_if = "IndexMap::is_empty")]
    pub fields: Extra,
}

impl Typed {
    pub fn new(kind: &str) -> Self {
        Typed {
            kind: kind.to_string(),
            fields: Extra::new(),
        }
    }
    pub fn field(&self, key: &str) -> Option<String> {
        self.fields.get(key).and_then(|v| match v {
            serde_yaml::Value::String(s) => Some(s.clone()),
            serde_yaml::Value::Number(n) => Some(n.to_string()),
            serde_yaml::Value::Bool(b) => Some(b.to_string()),
            _ => None,
        })
    }
    pub fn set_field(&mut self, key: &str, value: Option<&str>) {
        match value {
            Some(v) if !v.is_empty() => {
                self.fields
                    .insert(key.to_string(), serde_yaml::Value::String(v.to_string()));
            }
            _ => {
                self.fields.shift_remove(key);
            }
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Restart {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub on_score: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub on_stop: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub on_violation: Option<String>,
    #[serde(flatten, default, skip_serializing_if = "IndexMap::is_empty")]
    pub extra: Extra,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Rotation {
    #[serde(rename = "type", default)]
    pub kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rotate_on: Option<String>,
    #[serde(flatten, default, skip_serializing_if = "IndexMap::is_empty")]
    pub extra: Extra,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct End {
    #[serde(rename = "type", default)]
    pub kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target: Option<Val>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub win_by: Option<Val>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seconds: Option<Val>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub count: Option<Val>,
    #[serde(flatten, default, skip_serializing_if = "IndexMap::is_empty")]
    pub extra: Extra,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ShotClock {
    pub seconds: Val,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub resets_on: Vec<String>,
    #[serde(flatten, default, skip_serializing_if = "IndexMap::is_empty")]
    pub extra: Extra,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Phase {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attack: Option<String>,
    #[serde(default)]
    pub roles: Vec<Role>,
    #[serde(flatten, default, skip_serializing_if = "IndexMap::is_empty")]
    pub extra: Extra,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Format {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub numbers: Option<String>,
    #[serde(default)]
    pub space: SpaceRef,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub roles: Vec<Role>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub teams: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub balls: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start: Option<Typed>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub restart: Option<Restart>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rotation: Option<Rotation>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end: Option<End>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shot_clock: Option<ShotClock>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub phases: Option<Vec<Phase>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub phase_flow: Option<serde_yaml::Value>,
    #[serde(flatten, default, skip_serializing_if = "IndexMap::is_empty")]
    pub extra: Extra,
}

impl Format {
    /// Every role in the format, across phases.
    pub fn all_roles(&self) -> Vec<Role> {
        match &self.phases {
            Some(phases) if !phases.is_empty() => {
                phases.iter().flat_map(|p| p.roles.clone()).collect()
            }
            _ => self.roles.clone(),
        }
    }

    /// Players on the floor at once (the largest phase for phased formats).
    pub fn on_court(&self) -> usize {
        match &self.phases {
            Some(phases) if !phases.is_empty() => {
                phases.iter().map(|p| p.roles.len()).max().unwrap_or(0)
            }
            _ => self.roles.len(),
        }
    }

    pub fn count_side(&self, side: &str) -> usize {
        self.roles.iter().filter(|r| r.team == side).count()
    }

    /// "3v3", "3v3+1" from the roles (S4 §8.2).
    pub fn derived_numbers(&self) -> String {
        let o = self.count_side("offense");
        let d = self.count_side("defense");
        let n = self.count_side("neutral");
        if n > 0 {
            format!("{o}v{d}+{n}")
        } else {
            format!("{o}v{d}")
        }
    }

    pub fn balls(&self) -> u32 {
        self.balls.unwrap_or(1)
    }
}

// ---------------------------------------------------------------------------------------
// Constraint, Game — S5
// ---------------------------------------------------------------------------------------

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Constraint {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub invites: Vec<String>,
    #[serde(default)]
    pub preserves: Vec<String>,
    #[serde(default)]
    pub targets: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stage: Option<String>,
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub dials: IndexMap<String, Dial>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub zones: Vec<Zone>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub roles: Vec<RoleEdit>,
    #[serde(default)]
    pub rules: Vec<Rule>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cues: Vec<String>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub notes: String,
    #[serde(flatten, default, skip_serializing_if = "IndexMap::is_empty")]
    pub extra: Extra,
}

/// The constraint categories (02 §2; `character` from 01 §12).
pub const CATEGORIES: [&str; 4] = ["task", "environment", "individual", "character"];

/// The PoST stages (R7).
pub const STAGES: [&str; 5] = [
    "coordination",
    "variability",
    "complex",
    "team-based",
    "performance",
];

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Game {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub stage: String,
    #[serde(default)]
    pub format: Format,
    /// Sport-pack base scoring, e.g. `{ two: 1, three: 1 }` (S5 §5).
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub base_rules: IndexMap<String, Val>,
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub dials: IndexMap<String, Dial>,
    #[serde(default)]
    pub constraints: Vec<Constraint>,
    /// Constraints switched off in the stack (feature-tree "suppress"). S14 keeps this on the stack edge.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub suppressed: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub pool: Vec<Constraint>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reads: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub observables: Vec<serde_yaml::Value>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub questions: Vec<String>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub notes: String,
    #[serde(flatten, default, skip_serializing_if = "IndexMap::is_empty")]
    pub extra: Extra,
}

impl Game {
    pub fn is_suppressed(&self, constraint_id: &str) -> bool {
        self.suppressed.iter().any(|s| s == constraint_id)
    }

    /// Every learning target the game is tagged with through its active constraints.
    pub fn targets(&self) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        for c in &self.constraints {
            if self.is_suppressed(&c.id) {
                continue;
            }
            for t in &c.targets {
                if !out.contains(t) {
                    out.push(t.clone());
                }
            }
        }
        out
    }

    pub fn numbers(&self) -> String {
        self.format
            .numbers
            .clone()
            .unwrap_or_else(|| self.format.derived_numbers())
    }

    /// A constraint id not used in this game yet.
    pub fn fresh_constraint_id(&self, base: &str) -> String {
        let taken = |id: &str| {
            self.constraints
                .iter()
                .chain(self.pool.iter())
                .any(|c| c.id == id)
        };
        if !taken(base) {
            return base.to_string();
        }
        (2..)
            .map(|n| format!("{base}-{n}"))
            .find(|id| !taken(id))
            .unwrap()
    }

    /// A rule id not used anywhere in this game yet.
    pub fn fresh_rule_id(&self, base: &str) -> String {
        let taken = |id: &str| {
            self.constraints
                .iter()
                .chain(self.pool.iter())
                .flat_map(|c| c.rules.iter())
                .any(|r| r.id == id)
        };
        if !taken(base) {
            return base.to_string();
        }
        (2..)
            .map(|n| format!("{base}-{n}"))
            .find(|id| !taken(id))
            .unwrap()
    }
}

// ---------------------------------------------------------------------------------------
// Practice, Block, Gym — S8, D-020
// ---------------------------------------------------------------------------------------

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Basket {
    pub id: String,
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub position: Point,
    #[serde(default)]
    pub facing: f64,
    #[serde(flatten, default, skip_serializing_if = "IndexMap::is_empty")]
    pub extra: Extra,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Gym {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub court: String,
    #[serde(default)]
    pub baskets: Vec<Basket>,
    #[serde(flatten, default, skip_serializing_if = "IndexMap::is_empty")]
    pub extra: Extra,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Groups {
    #[serde(default)]
    pub mode: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub team_size: Option<u32>,
    #[serde(flatten, default, skip_serializing_if = "IndexMap::is_empty")]
    pub extra: Extra,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Block {
    pub id: String,
    #[serde(default)]
    pub name: String,
    /// Game (or variant) id.
    #[serde(default)]
    pub game: String,
    #[serde(default)]
    pub baskets: Vec<String>,
    #[serde(default)]
    pub minutes: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub intervals: Option<serde_yaml::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub groups: Option<Groups>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub teams: Option<serde_yaml::Value>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub protected: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_minutes: Option<f64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub modifiers: Vec<serde_yaml::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub notes: String,
    #[serde(flatten, default, skip_serializing_if = "IndexMap::is_empty")]
    pub extra: Extra,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Practice {
    pub id: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub name: String,
    #[serde(default)]
    pub date: String,
    #[serde(default)]
    pub gym: String,
    #[serde(default)]
    pub start: String,
    #[serde(default)]
    pub planned_minutes: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub policy: Option<serde_yaml::Value>,
    #[serde(default)]
    pub transition_s: f64,
    #[serde(default)]
    pub roster_present: Vec<String>,
    /// Players expected, for active-time estimates before a roster exists.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub players: Option<u32>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub theme: String,
    #[serde(default)]
    pub blocks: Vec<Block>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub gyms: Vec<Gym>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub notes: String,
    #[serde(flatten, default, skip_serializing_if = "IndexMap::is_empty")]
    pub extra: Extra,
}

impl Practice {
    pub fn title(&self) -> String {
        if !self.name.is_empty() {
            self.name.clone()
        } else if !self.date.is_empty() {
            format!("Practice {}", self.date)
        } else {
            self.id.clone()
        }
    }

    /// Players expected: the explicit count, else the attendance list.
    pub fn player_count(&self) -> u32 {
        self.players.unwrap_or(self.roster_present.len() as u32)
    }

    pub fn fresh_block_id(&self) -> String {
        (1..)
            .map(|n| format!("b{n}"))
            .find(|id| !self.blocks.iter().any(|b| &b.id == id))
            .unwrap()
    }
}

// ---------------------------------------------------------------------------------------
// Play — S7
// ---------------------------------------------------------------------------------------

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct PlayState {
    pub id: String,
    #[serde(default)]
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ball: Option<String>,
    #[serde(default)]
    pub at: IndexMap<String, Point>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub markers: Option<serde_yaml::Value>,
    #[serde(flatten, default, skip_serializing_if = "IndexMap::is_empty")]
    pub extra: Extra,
}

/// Where an action goes: another role (a pass) or a court point (a cut).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Target {
    Role(String),
    Spot(Point),
}

/// The arguments of one action (S7 §2): `{from, to}`, `{by, to}`, `{by, for}` or `{by}`.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ActionArgs {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub by: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to: Option<Target>,
    #[serde(default, rename = "for", skip_serializing_if = "Option::is_none")]
    pub for_role: Option<String>,
    #[serde(flatten, default, skip_serializing_if = "IndexMap::is_empty")]
    pub extra: Extra,
}

/// One action in a transition, written `{ <kind>: {args} }`. The kinds are sport-pack
/// vocabulary (S1 ball events), so the core keeps them as data (D-005).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Action(pub IndexMap<String, ActionArgs>);

impl Action {
    pub fn new(kind: &str, args: ActionArgs) -> Self {
        let mut m = IndexMap::new();
        m.insert(kind.to_string(), args);
        Action(m)
    }
    pub fn kind(&self) -> &str {
        self.0.keys().next().map(|k| k.as_str()).unwrap_or("?")
    }
    pub fn args(&self) -> ActionArgs {
        self.0.values().next().cloned().unwrap_or_default()
    }
    pub fn args_mut(&mut self) -> Option<&mut ActionArgs> {
        self.0.values_mut().next()
    }
    /// The player who does the action (`by` or `from`).
    pub fn actor(&self) -> Option<String> {
        let a = self.args();
        a.by.or(a.from)
    }
    /// The role an action goes to (pass target), if any.
    pub fn to_role(&self) -> Option<String> {
        match self.args().to {
            Some(Target::Role(r)) => Some(r),
            _ => None,
        }
    }
    /// The court point an action goes to (cut / dribble), if any.
    pub fn to_spot(&self) -> Option<Point> {
        match self.args().to {
            Some(Target::Spot(p)) => Some(p),
            _ => None,
        }
    }
    /// Every role the action involves.
    pub fn roles(&self) -> Vec<String> {
        let a = self.args();
        let mut v: Vec<String> = [a.by, a.from, a.for_role].into_iter().flatten().collect();
        if let Some(Target::Role(r)) = a.to {
            v.push(r);
        }
        v
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Transition {
    pub id: String,
    pub from: String,
    pub to: String,
    #[serde(default)]
    pub actions: Vec<Action>,
    /// A Read id, or none for "always" (S7 §2).
    #[serde(default)]
    pub guard: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timing: Option<String>,
    #[serde(flatten, default, skip_serializing_if = "IndexMap::is_empty")]
    pub extra: Extra,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Play {
    pub id: String,
    #[serde(default)]
    pub name: String,
    /// `linear-set` | `read-set` | `concept`
    #[serde(default)]
    pub kind: String,
    #[serde(default)]
    pub court: String,
    #[serde(default)]
    pub roles: Vec<String>,
    #[serde(default)]
    pub states: Vec<PlayState>,
    #[serde(default)]
    pub transitions: Vec<Transition>,
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub trained_by: IndexMap<String, Vec<String>>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub notes: String,
    #[serde(flatten, default, skip_serializing_if = "IndexMap::is_empty")]
    pub extra: Extra,
}

pub const PLAY_KINDS: [&str; 3] = ["linear-set", "read-set", "concept"];

impl Play {
    pub fn state(&self, id: &str) -> Option<&PlayState> {
        self.states.iter().find(|s| s.id == id)
    }
    pub fn fresh_state_id(&self) -> String {
        (1..)
            .map(|n| format!("s{n}"))
            .find(|id| !self.states.iter().any(|s| &s.id == id))
            .unwrap()
    }
    pub fn fresh_transition_id(&self) -> String {
        (1..)
            .map(|n| format!("t{n}"))
            .find(|id| !self.transitions.iter().any(|t| &t.id == id))
            .unwrap()
    }
}

// ---------------------------------------------------------------------------------------
// YAML in / out (D-018) — strings only; files are the app's business.
// ---------------------------------------------------------------------------------------

pub fn from_yaml<T: serde::de::DeserializeOwned>(text: &str) -> Result<T, String> {
    serde_yaml::from_str(text).map_err(|e| e.to_string())
}

pub fn to_yaml<T: Serialize>(value: &T) -> Result<String, String> {
    serde_yaml::to_string(value).map_err(|e| e.to_string())
}
