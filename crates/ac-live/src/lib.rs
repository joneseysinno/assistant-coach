//! # ac-live
//!
//! Hub logic for live practice, as pure functions over the append-only log (D-013):
//! `fold(practice, entries) -> LiveState`, clocks stored as start + paused + duration
//! (D-012), the schedule guard (D-015, S13), Remote buttons generated from rules (D-014)
//! and request handling with idempotency.
//!
//! Rules: no Dioxus, no tokio, no IO; time comes in through an injected clock.
//!
//! Filled in by P1 tasks T13–T17.

#[cfg(test)]
mod tests {
    #[test]
    fn crate_builds() {}
}
