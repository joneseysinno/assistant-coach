//! # ac-domain
//!
//! The sport-agnostic core of Assistant Coach (D-005): Space, Role, Event, Rule, Read, Dial
//! (the closed primitive set, D-003) and their compositions — Format, Constraint, Game,
//! Practice, Play — plus the resolver, the validator, practice arithmetic, play-graph
//! analysis and the `Store` trait.
//!
//! Rules for this crate (checked by `scripts/check_boundaries.py`):
//! - no Dioxus, no async runtime, no file or network IO;
//! - no `infinite-db` (storage goes through the `Store` trait, D-017);
//! - no basketball words in type or field names — those live in `ac-sport-basketball`.

pub mod model;
pub mod pack;
pub mod play;
pub mod practice;
pub mod resolve;
pub mod store;
pub mod val;

pub use model::*;
pub use pack::Pack;
pub use resolve::{
    resolve, Issue, ResolveOptions, ResolvedGame, ResolvedRule, RuleSource, Severity,
};
pub use store::{Doc, DocKind, MemStore, Store, StoreError};
pub use val::{OneOrMany, Val};
