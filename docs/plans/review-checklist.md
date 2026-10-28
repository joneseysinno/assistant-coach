# Review Checklist — for every PR Cursor lands (W5.5)

Paste this into the review. A PR passes when every applicable box is ticked or the exception is written down.

## Boundaries
- [ ] `ac-domain` and `ac-live` have no Dioxus, tokio, file or network IO.
- [ ] Only `ac-store-idb` depends on `infinite-db` (**D-017**); UI never calls the store directly except through the `Store` trait.
- [ ] No basketball words in `ac-domain` type or field names — they live in `ac-sport-basketball` data (**D-005**).
- [ ] No new primitive or "kind of thing" without a decision in `decisions.md` (**D-003**).
- [ ] Board/Remote (`ac-web`) never open the DB (**D-019**).

## Model rules (02 §5)
- [ ] **R4/R6** — difficulty is in dials; variants store deltas, never copied stacks.
- [ ] **R5 / D-004** — every point source is a Rule; no free-text scoring.
- [ ] **D-034** — precedence only through `overrides`; zone `kind` has no behaviour.
- [ ] **D-025** — buttons only for `judge: coach` rules; points rules are never `players`.
- [ ] **D-033** — no trigger forms beyond S2 §3 (no general sequences, no negation).
- [ ] **R13 / D-008** — no technique/form fields anywhere.
- [ ] **R15 / D-007** — every estimate returns formula text + confidence.

## Live (S11–S13)
- [ ] **D-012** — clocks stored as start/paused/duration; nothing counts down by decrementing.
- [ ] **D-013** — every state change is a log entry; undo is an entry, not a delete.
- [ ] **D-036** — tap acked only after the log append is durable; InfiniteDB called only on the store thread.
- [ ] Idempotency by request id; per-device undo (S11 §9).
- [ ] Clients never decide clock expiry; the hub emits it.

## Tests
- [ ] The task's acceptance tests from `P1-tasks.md`/`P2-tasks.md` exist and pass.
- [ ] Fixtures were not edited to make a test pass (if a fixture changed, the PR says why and `tools/p0_check.py` still passes).
- [ ] Rendered card text for every fixture is unchanged, or the change is intended and the fixture `expect:` updated in the same PR.

## Docs
- [ ] If behaviour differs from a spec, the spec was updated (or a decision added) in the same PR — **the docs stay the source of truth**.
