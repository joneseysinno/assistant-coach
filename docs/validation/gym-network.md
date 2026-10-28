# Gym Network Test (W4.6) — exit criterion E9

> **Updated 2026-10-28 (P1-execution P-1):** run this with the **T00 tracer** instead of `python -m http.server`. Same checks, real stack, plus tap latency, a Wi-Fi-drop test and the wasm pipeline. Build and gym script: `spikes/tracer/README.md`. Fill in the tables below; §5 is new.
>
> Original note: environment only, no app code. Do this first: it's the biggest technical unknown and it decides S12 type sizes.
> Bring: laptop + charger, router, Android phone, HDMI cable, a tape measure (or count floor boards), this sheet.

## 1. Setup (5 min)

1. Plug in the router. Connect the **laptop** and the **phone** to its Wi-Fi (not the school network).
2. Laptop: Windows Settings → Network → set this Wi-Fi to **Private**.
3. Laptop power: set *lid close = do nothing* and *sleep = never* while plugged in.
4. Find the laptop's IP: `ipconfig` → "IPv4 Address" under the Wi-Fi adapter: `192.168.___.___`
5. Copy `docs/printables/` to the laptop. In that folder run: `python -m http.server 7878 --bind 0.0.0.0`
   - If Windows Firewall asks, allow on **Private networks**. Note whether it asked: ☐ yes ☐ no

## 2. Phone → laptop

| Check | Result |
|---|---|
| Phone browser opens `http://<laptop-ip>:7878/mock-board.html` | ☐ works ☐ fails — error: |
| If it failed: firewall rule added for port 7878 (inbound, private) then retried | ☐ works |
| Load time on the phone | ____ s |
| Reload 10× standing at center court — any failures? | ____ / 10 |
| Walk away from the router until the page fails to reload. Distance: | ____ ft (where: __________) |
| Router placement that worked best | __________ |

## 3. TV Board readability

Laptop → TV by HDMI, extend the display, open `http://localhost:7878/mock-board.html` on the TV, press **F** for full screen. Keys: `1` running · `2` last minute · `3` overtime · `4` two baskets · `5` rest · `6` between blocks · `7` readability ladder.

TV size: ____″ · Where the TV sits: __________

| Element (key 1 / 3 / 4) | Readable at the **far baseline**? | Far **corner**? | Half court? |
|---|---|---|---|
| Scores (30 vh) | ☐ | ☐ | ☐ |
| Block clock (22 vh) | ☐ | ☐ | ☐ |
| Shot clock (14 vh) | ☐ | ☐ | ☐ |
| Game name (7 vh) | ☐ | ☐ | ☐ |
| Rule line (6 vh) | ☐ | ☐ | ☐ |
| Next / prompt (4.5 vh) | ☐ | ☐ | ☐ |
| Amber (key 2) and red overtime (key 3) obvious from the far end? | ☐ | ☐ | ☐ |
| Two-basket view (key 4) readable from each side basket? | ☐ | ☐ | — |

Ladder (key 7): smallest size readable at a glance from — far baseline: ____ vh · far corner: ____ vh · half court: ____ vh

Sound: does the TV play audio from the laptop over HDMI? ☐ yes ☐ no · loud enough over a live game? ☐

## 3b. Tracer results (T00 Remote stats line)

| Check | Result |
|---|---|
| Firewall prompt for `tracer-hub.exe`? Port rule needed? | |
| Tap RTT at center court: p50 / p95 / max (ms) | |
| Tap RTT at far corner / far baseline (ms) | |
| Airplane mode 10 s mid-taps: pending → 0 after reconnect? TV added exactly the taps? `reconnects` / `dup-acks` | |
| Two scorers at once: totals match? | |
| TV clock vs. phone clock over 1 min: visibly out of step? | |
| Horn audible over HDMI during play? | |
| `/wasm` page tracks the score on the phone? (`build.ps1` succeeded?) | |

## 4. Results → decisions

| Finding | Changes |
|---|---|
| Firewall prompt / rule needed | First-run setup step in P1 (**T25**): a port rule for 7878 on Private networks |
| Max usable phone distance | Router placement note in the setup checklist |
| Readability sizes | S12 §5 type sizes (replace the provisional vh values) |
| Sound | S12 §7 |

Date: ________ · Gym: ________ · Notes:
