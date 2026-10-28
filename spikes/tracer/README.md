# T00 — Tracer bullet (throwaway)

> Deleted when T18 lands. See `docs/plans/P1-execution.md` §2 P-1. Guardrail: **no domain types, no rules, no store.** If you want to add a feature, it belongs in a real task.

One exe on the laptop. Phones open `/remote` and tap +1/+2/+3, the TV shows `/board`, and the hub keeps the only copy of the score and clock. It answers four questions before P1 builds on them:

| # | Question | How T00 answers it |
|---|---|---|
| 1 | Does the phone reach the laptop over your router, past Windows Firewall? | Scan the QR, tap. `gym-network.md` §2 |
| 2 | Does a Dioxus **wasm** build embed in the exe and talk to `/ws`? | `build.ps1` → open `/wasm` on the phone |
| 3 | Is the Board readable from the far baseline? | `/board` full screen on the TV. `gym-network.md` §3 |
| 4 | How fast is a tap, and does a Wi-Fi drop lose or double taps? | The stats line on the Remote; the airplane-mode test below |

## Build (on the laptop, once)

```powershell
cd D:\Rust\Dioxus\assistant-coach\spikes\tracer
rustup target add wasm32-unknown-unknown    # once
dx --version                                # needs 0.7.x
powershell -ExecutionPolicy Bypass -File build.ps1
```

This produces `target\release\tracer-hub.exe`, a single file with every page embedded.
If `dx build` fails, write the error down (it's finding #2) and run the hub without the wasm page: `cargo run --release -p tracer-hub`.

## Run

```powershell
.\target\release\tracer-hub.exe            # window with QR + URLs
.\target\release\tracer-hub.exe --headless # console only
```

- **Firewall:** the first run asks to allow `tracer-hub.exe`. Allow it on **Private** networks, and make sure the router's Wi-Fi is set to Private in Windows. If the phone still can't connect, add a port rule (it survives rebuilds, unlike a per-exe rule):
  `New-NetFirewallRule -DisplayName "Assistant Coach 7878" -Direction Inbound -Protocol TCP -LocalPort 7878 -Profile Private -Action Allow` (admin PowerShell).
- **TV:** drag a browser window to the TV, open `http://localhost:7878/board`, press **F**, click once (unlocks sound).
- **Phone:** scan the QR in the hub window. Keep the screen awake: Settings → Display → Screen timeout → 10 min, for the test.

## The gym script (about 15 minutes; results go in `docs/validation/gym-network.md`)

1. **Reach:** phone scans the QR → Remote loads. Note the load time and whether the firewall prompt appeared.
2. **Tap latency:** tap 20 times at center court. Read the stats line: `rtt p50 / p95 / max`.
3. **Range:** walk to each corner and the far baseline, tapping. Note where the dot turns red.
4. **Wi-Fi drop:** at center court, turn on airplane mode, tap +1 five times (they show as *pending*), wait 10 s, turn airplane mode off. Within a few seconds pending → 0, and the TV adds exactly 5. Note `reconnects` and `dup-acks` (dup-acks > 0 is fine: the hub received taps whose acks were lost and ignored the re-sends).
5. **Two scorers:** a second phone or tablet scores at the same time; the totals must match what was tapped.
6. **Clock:** set 1:00 → Start. Watch the TV clock against the phone's clock for a minute: amber chirp at 1:00, horn at 0:00, red overtime after. Is the horn audible over HDMI during play?
7. **Readability:** stand at the far baseline and the far corner. Scores (30 vh), clock (22 vh), rule line (6 vh): readable? `mock-board.html` key 7 has the ladder.
8. **wasm:** open `/wasm` on the phone. Does it track the score as you tap? (Finding #2.)

## What's in here

```
hub/src/lib.rs        scoreboard state (clock = duration + banked + started_at), /ws handler, routes
hub/src/main.rs       hub window (Dioxus desktop) + server thread; --headless
hub/assets/           remote.html, board.html, index.html (plain JS on purpose: testable without wasm)
hub/assets/wasm/      placeholder; build.ps1 replaces it with the dx output
hub/tests/            two clients scoring at once with re-sends -> each tap counted once
wasm-board/           the Dioxus wasm proof page
build.ps1             dx build -> copy -> cargo build --release
```

## Verified in the cloud session (2026-10-28)

- `cargo test`: 4 unit + 2 integration tests (200 + 150 taps from two clients, one re-sending every tap ×3 → exact score). Clippy `-D warnings` clean.
- Real Chromium, 2 phone-sized Remotes + 1 Board: phone A offline for 6 s mid-taps → shows "reconnecting…" within 4 s, 7 taps pending → reconnect → board exactly +7, no duplicates; per-device undo; Board clock 0:57 three seconds after a 1:00 start, amber.
- Hub window under a virtual display: server reachable while the Dioxus window runs.
- **Not verified here:** the wasm build (the cloud session couldn't download the wasm std library). That is exactly finding #2, and it's tested on your laptop with `build.ps1`.
- Found and fixed: Chrome kept a dead socket "open" for more than 5 s with Wi-Fi off, so the Remote kept saying *connected*. Both pages now drop the socket on a 4 s heartbeat timeout and reconnect. **Carry this into T20** (S11 §10).
