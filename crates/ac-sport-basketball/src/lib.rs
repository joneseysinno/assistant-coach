//! # ac-sport-basketball
//!
//! The basketball sport pack (D-005): court geometry (S3), vocabulary (S1, S6), the
//! plain-English card renderer (S2 §7), predicted effects (S9) and starter templates.
//! `ac-domain` stays sport-agnostic; everything that says "three" or "paint" lives here.

pub mod court;
pub mod estimates;
pub mod render;
pub mod templates;
pub mod vocab;

use ac_domain::{Consequence, Judge, Pack, RoleEdit, Rule, Trigger, Val};
use indexmap::IndexMap;

/// The basketball pack.
#[derive(Clone, Copy, Debug, Default)]
pub struct BasketballPack;

impl Pack for BasketballPack {
    fn is_event(&self, id: &str) -> bool {
        vocab::EVENTS.iter().any(|e| e.0 == id)
    }
    fn is_named_region(&self, id: &str) -> bool {
        vocab::NAMED_REGIONS.iter().any(|r| r.0 == id)
            || vocab::COURT_REGIONS.iter().any(|r| r.0 == id)
            || vocab::PLACES.contains(&id)
    }
    fn is_target(&self, id: &str) -> bool {
        vocab::TARGETS.iter().any(|t| t.0 == id)
    }
    fn is_read(&self, id: &str) -> bool {
        vocab::read(id).is_some()
    }

    /// `{ two: n, three: n }` → `base.two` ("Basket") and `base.three` ("Three").
    fn base_rules(&self, spec: &IndexMap<String, Val>) -> Vec<Rule> {
        spec.iter()
            .filter_map(|(k, n)| {
                let (value, label) = match k.as_str() {
                    "two" => (2, "Basket"),
                    "three" => (3, "Three"),
                    _ => return None,
                };
                Some(Rule {
                    id: format!("base.{k}"),
                    label: Some(label.into()),
                    when: Trigger {
                        event: Some("make".into()),
                        value: Some(Val::Int(value)),
                        ..Default::default()
                    },
                    then: vec![Consequence {
                        points: Some(n.clone()),
                        ..Default::default()
                    }],
                    judge: Some(Judge::Coach),
                    ..Default::default()
                })
            })
            .collect()
    }

    /// S5 §2.1 role-edit sugar, as `p0_check.py` compiles it.
    fn role_edit_rules(&self, cid: &str, edit: &RoleEdit) -> Vec<Rule> {
        let mut out = Vec::new();
        let Some(role) = edit.restrict.as_deref() else {
            return out;
        };
        let (short, long) = match role {
            "H" => ("Help".to_string(), "help defender".to_string()),
            "J" => ("Joker".to_string(), "joker".to_string()),
            r => (r.to_string(), r.to_string()),
        };
        let actor = |r: &str| {
            let mut m = serde_yaml::Mapping::new();
            m.insert("role".into(), r.into());
            Some(serde_yaml::Value::Mapping(m))
        };
        if let Some(zone) = &edit.stay_in {
            out.push(Rule {
                id: format!("{cid}.{role}-zone"),
                label: Some(format!("{short} left {zone}")),
                when: Trigger {
                    event: Some("zone.exit".into()),
                    zone: Some(ac_domain::OneOrMany::One(zone.clone())),
                    actor: actor(role),
                    ..Default::default()
                },
                then: vec![Consequence {
                    points: Some(edit.penalty.clone().unwrap_or(Val::Int(1))),
                    to: Some("offense".into()),
                    ..Default::default()
                }],
                judge: Some(Judge::Coach),
                say: Some(format!("The {long} must stay in the {zone}")),
                ..Default::default()
            });
        }
        if let Some(what) = edit.cannot.as_deref() {
            let (ev, verb) = match what {
                "shoot" => ("shot", "shoot"),
                "dribble" => ("dribble", "dribble"),
                other => (other, other),
            };
            let label = if role == "J" && ev == "shot" {
                "Joker shot".to_string()
            } else {
                format!("{short} {ev}")
            };
            let say = if role == "J" {
                format!("The joker can't {verb}")
            } else {
                format!("{role} can't {verb}")
            };
            out.push(Rule {
                id: format!("{cid}.{role}-{ev}"),
                label: Some(label),
                when: Trigger {
                    event: Some(ev.into()),
                    actor: actor(role),
                    ..Default::default()
                },
                then: vec![Consequence::turnover()],
                judge: Some(Judge::Players),
                say: Some(say),
                ..Default::default()
            });
        }
        out
    }
}
