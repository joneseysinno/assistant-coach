//! Scalar values as they appear in game files: numbers, booleans, words, and dial
//! references written as `"$name"` (S5 §4).

use serde::{Deserialize, Serialize};

/// A scalar from a game file. Integers stay integers on save, so a round trip doesn't turn
/// `7` into `7.0`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Val {
    Bool(bool),
    Int(i64),
    Float(f64),
    Text(String),
}

impl Default for Val {
    fn default() -> Self {
        Val::Int(0)
    }
}

impl Val {
    pub fn num(v: f64) -> Self {
        if v.fract() == 0.0 && v.abs() < 1e15 {
            Val::Int(v as i64)
        } else {
            Val::Float(v)
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Val::Int(i) => Some(*i as f64),
            Val::Float(f) => Some(*f),
            Val::Text(t) => t.parse().ok(),
            Val::Bool(_) => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Val::Text(t) => Some(t),
            _ => None,
        }
    }

    /// `Some("limit")` for `"$limit"`.
    pub fn dial_ref(&self) -> Option<&str> {
        self.as_str().and_then(|s| s.strip_prefix('$'))
    }

    pub fn is_ref(&self) -> bool {
        self.dial_ref().is_some()
    }
}

impl std::fmt::Display for Val {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Val::Bool(b) => write!(f, "{b}"),
            Val::Int(i) => write!(f, "{i}"),
            Val::Float(x) => {
                if x.fract() == 0.0 {
                    write!(f, "{}", *x as i64)
                } else {
                    write!(f, "{x}")
                }
            }
            Val::Text(t) => write!(f, "{t}"),
        }
    }
}

impl From<i64> for Val {
    fn from(v: i64) -> Self {
        Val::Int(v)
    }
}

impl From<&str> for Val {
    fn from(v: &str) -> Self {
        Val::Text(v.to_string())
    }
}

/// One zone id or a list of them (`zone: corner-L` or `zone: [corner-L, corner-R]`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OneOrMany {
    One(String),
    Many(Vec<String>),
}

impl OneOrMany {
    pub fn items(&self) -> Vec<String> {
        match self {
            OneOrMany::One(s) => vec![s.clone()],
            OneOrMany::Many(v) => v.clone(),
        }
    }
    pub fn first(&self) -> Option<&str> {
        match self {
            OneOrMany::One(s) => Some(s),
            OneOrMany::Many(v) => v.first().map(|s| s.as_str()),
        }
    }
    pub fn is_list(&self) -> bool {
        matches!(self, OneOrMany::Many(_))
    }
}
