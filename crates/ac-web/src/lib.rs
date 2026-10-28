//! # ac-web
//!
//! Board (TV, `/board`) and Remote (phone/tablet, `/remote`) — Dioxus web clients built to
//! wasm and served by the hub in `ac-app` (D-010, D-011). They talk to the hub over the
//! `/ws` JSON protocol (S11) and never open the database (D-019).
//!
//! Filled in by P1 tasks T19 (Board) and T20 (Remote). The T00 tracer bullet in
//! `spikes/tracer` proves the build-and-embed pipeline first.

#[cfg(test)]
mod tests {
    #[test]
    fn crate_builds() {}
}
