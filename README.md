# Assistant Coach

Constraint-led practice design and live practice scoring for basketball coaches.
Laptop = hub, TV = Board, phone/tablet = Remote. Docs are the source of truth: start at
[`docs/00-README.md`](docs/00-README.md); P1 work follows
[`docs/plans/P1-tasks.md`](docs/plans/P1-tasks.md) and
[`docs/plans/P1-execution.md`](docs/plans/P1-execution.md).

## Layout (D-023)

| Path | What |
|---|---|
| `crates/ac-domain` | Sport-agnostic model, resolver, renderer, `Store` trait. No UI, no IO |
| `crates/ac-sport-basketball` | Court presets, regions, event vocabulary (data) |
| `crates/ac-store-idb` | `Store` on InfiniteDB (the only crate that may use it, D-017) |
| `crates/ac-live` | Hub logic: log, fold, clocks, schedule guard, buttons. Pure |
| `crates/ac-app` | Dioxus desktop: Studio (JoInn app-shell) + embedded hub server |
| `crates/ac-web` | Board + Remote (wasm), served by the hub |
| `spikes/tracer` | T00 tracer bullet — throwaway, deleted at T18 |
| `docs/` | Theory, specs S1–S14, decisions, fixtures (the test corpus), plans |
| `tools/` | `p0_check.py` (fixture oracle), `make_printables.py` |
| `scripts/` | `ci.ps1` / `ci.sh`, `check_boundaries.py` |

## Commands

```powershell
cargo run -p ac-app                                   # Studio
powershell -ExecutionPolicy Bypass -File scripts\ci.ps1   # everything CI checks
cargo test -p ac-store-idb -- --ignored               # InfiniteDB known-bug tests (D-037); pass = bug fixed
```

CI needs Python 3.11+ with `pyyaml` (for the fixture oracle).
