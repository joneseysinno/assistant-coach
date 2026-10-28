//! # ac-domain
//!
//! The sport-agnostic core of Assistant Coach (D-005): Space, Role, Event, Rule, Read, Dial
//! (the closed primitive set, D-003) and their compositions — Format, Constraint, Game,
//! Variant, Practice — plus the resolver, the plain-English renderer and the `Store` trait.
//!
//! Rules for this crate (checked by `scripts/check_boundaries.py`):
//! - no Dioxus, no async runtime, no file or network IO;
//! - no `infinite-db` (storage goes through the `Store` trait, D-017);
//! - no basketball words in type or field names — those live in `ac-sport-basketball` data.
//!
//! Filled in by P1 tasks T02 (geometry), T04–T09 (events, rules, format, loader, resolver,
//! renderer) and T10 (Store trait + MemStore).

#[cfg(test)]
mod tests {
    #[test]
    fn crate_builds() {}
}
