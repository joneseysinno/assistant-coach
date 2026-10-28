//! `resolve(game) → ResolvedGame` (S5 §5) plus the validator (S2 §8, S3 §8, S4 §8, S5 §8,
//! R1–R7). The card renderer, the Remote button generator and the hub only ever see a
//! `ResolvedGame`.

use indexmap::IndexMap;
use serde::Serialize;

use crate::model::{Constraint, Dial, DialKind, Format, Game, Role, Rule, Zone, STAGES};
use crate::pack::Pack;
use crate::val::Val;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum Severity {
    Error,
    Warning,
}

/// One validator or linter finding, in coaching language where possible.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Issue {
    pub severity: Severity,
    /// The rule/decision it comes from: "R2", "S2 §8.5", "D-029"…
    pub code: &'static str,
    pub message: String,
    /// The constraint it belongs to, if any (so the Studio can point at it).
    pub constraint: Option<String>,
}

impl Issue {
    fn err(code: &'static str, msg: impl Into<String>, c: Option<&str>) -> Self {
        Issue {
            severity: Severity::Error,
            code,
            message: msg.into(),
            constraint: c.map(str::to_string),
        }
    }
    fn warn(code: &'static str, msg: impl Into<String>, c: Option<&str>) -> Self {
        Issue {
            severity: Severity::Warning,
            code,
            message: msg.into(),
            constraint: c.map(str::to_string),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub enum RuleSource {
    /// From the format's base scoring (`base.*`).
    Base,
    /// From a constraint in the stack.
    Constraint(String),
    /// A block-level player modifier (D-031).
    Modifier,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ResolvedRule {
    /// The rule with every `$dial` replaced by its value.
    pub rule: Rule,
    pub source: RuleSource,
    pub judge: crate::model::Judge,
}

impl ResolvedRule {
    pub fn is_base(&self) -> bool {
        self.source == RuleSource::Base
    }
    pub fn is_modifier(&self) -> bool {
        self.source == RuleSource::Modifier
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ResolvedZone {
    pub zone: Zone,
    pub constraint: String,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct BoundDial {
    /// `name` for game dials, `constraint.name` for constraint dials (S5 §4).
    pub key: String,
    pub owner: Option<String>,
    pub name: String,
    pub dial: Dial,
    pub value: Val,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ResolvedGame {
    pub id: String,
    pub name: String,
    pub stage: String,
    /// The format with dials bound.
    pub format: Format,
    pub roles: Vec<Role>,
    pub zones: Vec<ResolvedZone>,
    /// Base rules first, then constraint rules in stack order, then modifiers.
    pub rules: Vec<ResolvedRule>,
    pub dials: Vec<BoundDial>,
    /// Ids of the constraints that contributed (stack order, after suppression and rollback).
    pub active: Vec<String>,
    pub issues: Vec<Issue>,
}

impl ResolvedGame {
    pub fn errors(&self) -> impl Iterator<Item = &Issue> {
        self.issues.iter().filter(|i| i.severity == Severity::Error)
    }
    pub fn rule(&self, id: &str) -> Option<&ResolvedRule> {
        self.rules.iter().find(|r| r.rule.id == id)
    }
}

#[derive(Clone, Debug, Default)]
pub struct ResolveOptions {
    /// Keep only the first N *active* constraints (the Studio rollback bar).
    pub rollback: Option<usize>,
    /// Dial values by key (`name` or `constraint.name`); wins over defaults.
    pub dials: IndexMap<String, Val>,
    /// Block-level player modifiers (D-031), appended after the stack.
    pub modifiers: Vec<Rule>,
}

/// Dial scope for one constraint: its own dials first, then the game's (S5 §4).
struct Scope<'a> {
    game: &'a Game,
    constraint: Option<&'a Constraint>,
    overrides: &'a IndexMap<String, Val>,
}

impl Scope<'_> {
    fn lookup(&self, name: &str) -> Option<(String, &Dial)> {
        if let Some(c) = self.constraint {
            if let Some(d) = c.dials.get(name) {
                return Some((format!("{}.{name}", c.id), d));
            }
        }
        self.game.dials.get(name).map(|d| (name.to_string(), d))
    }

    fn value(&self, name: &str) -> Option<(Val, DialKind)> {
        let (key, dial) = self.lookup(name)?;
        let v = self
            .overrides
            .get(&key)
            .cloned()
            .unwrap_or_else(|| dial.default.clone());
        Some((v, dial.kind()))
    }
}

/// Replace every `"$name"` string inside any serializable value.
fn bind<T>(value: &T, scope: &Scope, issues: &mut Vec<Issue>, where_: &str) -> T
where
    T: Serialize + serde::de::DeserializeOwned + Clone,
{
    let Ok(mut y) = serde_yaml::to_value(value) else {
        return value.clone();
    };
    walk(&mut y, scope, issues, where_);
    serde_yaml::from_value(y).unwrap_or_else(|_| value.clone())
}

fn walk(v: &mut serde_yaml::Value, scope: &Scope, issues: &mut Vec<Issue>, where_: &str) {
    match v {
        serde_yaml::Value::String(s) if s.starts_with('$') => {
            let name = s[1..].to_string();
            match scope.value(&name) {
                Some((val, _)) => {
                    *v = serde_yaml::to_value(val).unwrap_or(serde_yaml::Value::Null);
                }
                None => issues.push(Issue::err(
                    "S5 §4",
                    format!("{where_}: dial ${name} is not defined"),
                    scope.constraint.map(|c| c.id.as_str()),
                )),
            }
        }
        serde_yaml::Value::Sequence(seq) => {
            for x in seq {
                walk(x, scope, issues, where_);
            }
        }
        serde_yaml::Value::Mapping(m) => {
            for (_, x) in m.iter_mut() {
                walk(x, scope, issues, where_);
            }
        }
        _ => {}
    }
}

fn collect_refs(v: &serde_yaml::Value, out: &mut Vec<String>) {
    match v {
        serde_yaml::Value::String(s) if s.starts_with('$') => out.push(s.clone()),
        serde_yaml::Value::Sequence(seq) => seq.iter().for_each(|x| collect_refs(x, out)),
        serde_yaml::Value::Mapping(m) => m.iter().for_each(|(_, x)| collect_refs(x, out)),
        _ => {}
    }
}

/// Type check one dial slot (S5 §4): `want` lists the dial kinds that fit.
fn check_slot(
    v: Option<&Val>,
    want: &[DialKind],
    scope: &Scope,
    issues: &mut Vec<Issue>,
    where_: &str,
) {
    let Some(name) = v.and_then(|v| v.dial_ref()) else {
        return;
    };
    if let Some((_, kind)) = scope.value(name) {
        if !want.contains(&kind) {
            let names: Vec<&str> = want.iter().map(|k| k.as_str()).collect();
            issues.push(Issue::err(
                "S5 §4",
                format!(
                    "{where_}: dial ${name} is a {} dial, but this slot needs {}",
                    kind.as_str(),
                    names.join(" or ")
                ),
                scope.constraint.map(|c| c.id.as_str()),
            ));
        }
    }
}

fn check_dials(owner: Option<&str>, dials: &IndexMap<String, Dial>, issues: &mut Vec<Issue>) {
    for (name, d) in dials {
        if let Some((lo, hi)) = d.bounds() {
            if lo > hi {
                issues.push(Issue::err(
                    "S5 §8.5",
                    format!("dial {name}: min is above max"),
                    owner,
                ));
            }
            if let Some(v) = d.default.as_f64() {
                if v < lo || v > hi {
                    issues.push(Issue::err(
                        "S5 §8.5",
                        format!("dial {name}: default {v} is outside {lo}–{hi}"),
                        owner,
                    ));
                }
            }
            for lv in d.levels.iter().flatten() {
                if let Some(x) = lv.as_f64() {
                    if x < lo || x > hi {
                        issues.push(Issue::err(
                            "S5 §8.5",
                            format!("dial {name}: level {x} is outside {lo}–{hi}"),
                            owner,
                        ));
                    }
                }
            }
        }
        if let (Some(choices), Some(def)) = (&d.choices, d.default.as_str()) {
            if !choices.iter().any(|c| c == def) {
                issues.push(Issue::err(
                    "S5 §8.5",
                    format!("dial {name}: default '{def}' is not one of the options"),
                    owner,
                ));
            }
        }
    }
}

/// Resolve a game into the flat, fully-bound form (S5 §5) and validate it.
pub fn resolve(game: &Game, pack: &dyn Pack, opts: &ResolveOptions) -> ResolvedGame {
    let mut issues = Vec::new();
    let game_scope = Scope {
        game,
        constraint: None,
        overrides: &opts.dials,
    };

    // 1. format + base rules
    let format: Format = bind(&game.format, &game_scope, &mut issues, "format");
    check_slot(
        game.format.end.as_ref().and_then(|e| e.target.as_ref()),
        &[DialKind::Int],
        &game_scope,
        &mut issues,
        "end target",
    );
    check_slot(
        game.format.shot_clock.as_ref().map(|s| &s.seconds),
        &[DialKind::Duration, DialKind::Int],
        &game_scope,
        &mut issues,
        "shot clock",
    );
    let mut roles = format.all_roles();
    let mut rules: Vec<ResolvedRule> = pack
        .base_rules(&game.base_rules)
        .into_iter()
        .map(|r| ResolvedRule {
            judge: r.effective_judge(),
            rule: r,
            source: RuleSource::Base,
        })
        .collect();
    let mut zones = Vec::new();
    let mut dials = Vec::new();
    for (name, d) in &game.dials {
        dials.push(BoundDial {
            key: name.clone(),
            owner: None,
            name: name.clone(),
            dial: d.clone(),
            value: opts.dials.get(name).cloned().unwrap_or(d.default.clone()),
        });
    }
    check_dials(None, &game.dials, &mut issues);

    // 2. walk the stack
    let mut active = Vec::new();
    for c in &game.constraints {
        if game.is_suppressed(&c.id) {
            continue;
        }
        if let Some(n) = opts.rollback {
            if active.len() >= n {
                break;
            }
        }
        active.push(c.id.clone());
        let scope = Scope {
            game,
            constraint: Some(c),
            overrides: &opts.dials,
        };
        for (name, d) in &c.dials {
            let key = format!("{}.{name}", c.id);
            dials.push(BoundDial {
                value: opts.dials.get(&key).cloned().unwrap_or(d.default.clone()),
                key,
                owner: Some(c.id.clone()),
                name: name.clone(),
                dial: d.clone(),
            });
        }
        for z in &c.zones {
            // S5 §4: dials inside geometry must be distance dials
            if let Ok(y) = serde_yaml::to_value(&z.shape) {
                let mut refs = Vec::new();
                collect_refs(&y, &mut refs);
                for r in refs {
                    check_slot(
                        Some(&Val::Text(r)),
                        &[DialKind::Distance],
                        &scope,
                        &mut issues,
                        &format!("zone {}", z.id),
                    );
                }
            }
            zones.push(ResolvedZone {
                zone: bind(z, &scope, &mut issues, &format!("zone {}", z.id)),
                constraint: c.id.clone(),
            });
        }
        for edit in &c.roles {
            if let Some(add) = &edit.add {
                roles.push(add.clone());
            }
        }
        let compiled = c.roles.iter().flat_map(|e| pack.role_edit_rules(&c.id, e));
        for r in c.rules.iter().cloned().chain(compiled) {
            let where_ = format!("rule {}", r.id);
            check_slot(
                r.when.value.as_ref().filter(|_| r.when.count.is_some()),
                &[DialKind::Int],
                &scope,
                &mut issues,
                &where_,
            );
            check_slot(
                r.within.as_ref(),
                &[DialKind::Duration, DialKind::Int],
                &scope,
                &mut issues,
                &where_,
            );
            let bound = bind(&r, &scope, &mut issues, &where_);
            rules.push(ResolvedRule {
                judge: bound.effective_judge(),
                rule: bound,
                source: RuleSource::Constraint(c.id.clone()),
            });
        }
    }
    for m in &opts.modifiers {
        let bound = bind(m, &game_scope, &mut issues, "modifier");
        rules.push(ResolvedRule {
            judge: bound.effective_judge(),
            rule: bound,
            source: RuleSource::Modifier,
        });
    }

    let mut rg = ResolvedGame {
        id: game.id.clone(),
        name: game.name.clone(),
        stage: game.stage.clone(),
        format,
        roles,
        zones,
        rules,
        dials,
        active,
        issues,
    };
    validate(game, pack, &mut rg);
    rg
}

fn validate(game: &Game, pack: &dyn Pack, rg: &mut ResolvedGame) {
    let mut out = Vec::new();

    // R7 stage
    if !STAGES.contains(&game.stage.as_str()) {
        out.push(Issue::err(
            "R7",
            "Pick a stage for this game (coordination, variability, complex, team-based or performance)",
            None,
        ));
    }
    // S4 §8.1 / R1
    let teams: Vec<&str> = rg.roles.iter().map(|r| r.team.as_str()).collect();
    if !teams.contains(&"offense") {
        out.push(Issue::err("S4 §8.1", "The game has no offense role", None));
    }
    if !teams.contains(&"defense") {
        if game.stage == "coordination" {
            if game.notes.trim().is_empty() {
                out.push(Issue::warn(
                    "R1",
                    "No live defense: say why in the notes (coordination stage)",
                    None,
                ));
            }
        } else {
            out.push(Issue::err(
                "R1",
                "No live defense. Add a defender, or set the stage to coordination and say why",
                None,
            ));
        }
    }
    let mut seen_roles = Vec::new();
    for r in &game.format.roles {
        if seen_roles.contains(&r.id) {
            out.push(Issue::err(
                "S4 §8.1",
                format!("Role {} appears twice", r.id),
                None,
            ));
        }
        seen_roles.push(r.id.clone());
    }

    // R2/R3 + S6 on constraints (stack + pool)
    for c in game.constraints.iter().chain(game.pool.iter()) {
        let cid = Some(c.id.as_str());
        if c.invites.is_empty() {
            out.push(Issue::err(
                "R2",
                format!("'{}' should say what it invites", c.name),
                cid,
            ));
        }
        if c.targets.is_empty() {
            out.push(Issue::err(
                "R2",
                format!("'{}' needs a learning target", c.name),
                cid,
            ));
        }
        if c.preserves.is_empty() {
            out.push(Issue::err(
                "R3",
                format!("'{}' should say what information it keeps", c.name),
                cid,
            ));
        }
        for t in &c.targets {
            if !pack.is_target(t) {
                out.push(Issue::err(
                    "S6",
                    format!("'{}': unknown learning target {t}", c.name),
                    cid,
                ));
            }
        }
        check_dials(cid, &c.dials, &mut out);
    }

    // S2 §8 on resolved rules (+ pool rules, which may be chosen in play)
    let zone_ids: Vec<String> = game
        .constraints
        .iter()
        .chain(game.pool.iter())
        .flat_map(|c| c.zones.iter().map(|z| z.id.clone()))
        .collect();
    let mut all_ids: Vec<String> = rg.rules.iter().map(|r| r.rule.id.clone()).collect();
    for c in &game.pool {
        all_ids.extend(c.rules.iter().map(|r| r.id.clone()));
    }
    let mut dup = Vec::new();
    for (i, id) in all_ids.iter().enumerate() {
        if all_ids[..i].contains(id) && !dup.contains(id) {
            dup.push(id.clone());
            out.push(Issue::err(
                "S2 §2",
                format!("Two rules share the id '{id}'"),
                None,
            ));
        }
    }
    let balls = rg.format.balls();
    let pool_rules: Vec<(Rule, Option<String>)> = game
        .pool
        .iter()
        .flat_map(|c| c.rules.iter().map(move |r| (r.clone(), Some(c.id.clone()))))
        .collect();
    let checked: Vec<(Rule, Option<String>)> = rg
        .rules
        .iter()
        .map(|r| {
            (
                r.rule.clone(),
                match &r.source {
                    RuleSource::Constraint(c) => Some(c.clone()),
                    _ => None,
                },
            )
        })
        .chain(pool_rules)
        .collect();
    for (r, cid) in &checked {
        let c = cid.as_deref();
        let label = r.display_label();
        for ev in [r.when.event.as_deref(), r.when.count.as_deref()]
            .into_iter()
            .flatten()
        {
            if !pack.is_event(ev) {
                out.push(Issue::err(
                    "S2 §8.1",
                    format!("Rule '{label}': unknown event '{ev}'"),
                    c,
                ));
            }
        }
        for z in [&r.when.zone, &r.when.zone_not]
            .into_iter()
            .flatten()
            .flat_map(|z| z.items())
        {
            if !zone_ids.contains(&z) && !pack.is_named_region(&z) {
                out.push(Issue::err(
                    "S2 §8.1",
                    format!("Rule '{label}': zone '{z}' doesn't exist"),
                    c,
                ));
            }
        }
        if r.when.kind() == crate::model::TriggerKind::Unknown {
            out.push(Issue::err(
                "S2 §3",
                format!("Rule '{label}': pick what triggers it"),
                c,
            ));
        }
        if r.after.len() > 2 {
            out.push(Issue::err(
                "D-033",
                format!("Rule '{label}': at most two 'after' steps"),
                c,
            ));
        }
        for s in &r.after {
            if let Some(ev) = &s.event {
                if !pack.is_event(ev) {
                    out.push(Issue::err(
                        "S2 §8.1",
                        format!("Rule '{label}': unknown event '{ev}'"),
                        c,
                    ));
                }
            }
        }
        if r.within.is_some() && r.after.is_empty() {
            out.push(Issue::err(
                "S2 §8.3",
                format!("Rule '{label}': 'within' needs an 'after' step"),
                c,
            ));
        }
        if balls > 1 && (r.when.count.is_some() || !r.after.is_empty()) {
            out.push(Issue::err(
                "D-029",
                format!("Rule '{label}': possession rules don't work with {balls} balls"),
                c,
            ));
        }
        if r.awards_points() && r.effective_judge() == crate::model::Judge::Players {
            out.push(Issue::err(
                "S2 §8.5",
                format!("Rule '{label}' gives points, so the coach must judge it"),
                c,
            ));
        }
        for o in &r.overrides {
            if !all_ids.contains(o) {
                out.push(Issue::err(
                    "S2 §8.6",
                    format!("Rule '{label}' overrides '{o}', which isn't in this game"),
                    c,
                ));
            }
        }
        let changes = r.then.iter().filter(|t| t.changes_possession()).count();
        if changes > 1 {
            out.push(Issue::warn(
                "S2 §8.7",
                format!("Rule '{label}' changes possession twice; only the first counts"),
                c,
            ));
        }
        if r.then.is_empty() {
            out.push(Issue::err(
                "S2 §5",
                format!("Rule '{label}': pick what happens"),
                c,
            ));
        }
    }

    // S3 §8.3 dead zones (warning)
    for z in &rg.zones {
        if z.zone.kind == "start" {
            continue;
        }
        let used = rg.rules.iter().any(|r| {
            [&r.rule.when.zone, &r.rule.when.zone_not]
                .into_iter()
                .flatten()
                .any(|zz| zz.items().contains(&z.zone.id))
                || r.rule.after.iter().any(|s| {
                    s.zone
                        .as_ref()
                        .is_some_and(|zz| zz.items().contains(&z.zone.id))
                })
        });
        if !used {
            out.push(Issue::warn(
                "S3 §8.3",
                format!("Zone '{}' isn't used by any rule", z.zone.id),
                Some(&z.constraint),
            ));
        }
    }

    // S6 reads
    for rd in &game.reads {
        if !pack.is_read(rd) {
            out.push(Issue::warn("S6", format!("Unknown read {rd}"), None));
        }
    }

    // R4 lint: two constraints that differ only in a number → make it a dial
    for (i, a) in game.constraints.iter().enumerate() {
        for b in &game.constraints[i + 1..] {
            if a.rules.len() == 1
                && b.rules.len() == 1
                && a.rules[0].when.event_id() == b.rules[0].when.event_id()
                && a.rules[0].when.count.is_some()
                && b.rules[0].when.count.is_some()
            {
                out.push(Issue::warn(
                    "R4",
                    format!(
                        "'{}' and '{}' look like the same limit: make it one constraint with a dial",
                        a.name, b.name
                    ),
                    Some(&b.id),
                ));
            }
        }
    }

    rg.issues.extend(out);
}
