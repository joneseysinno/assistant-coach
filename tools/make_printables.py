#!/usr/bin/env python3
"""Generate the P0 paper-prototype printables (W4.1–W4.3, W4.7) from the game fixtures.

Uses the same resolver/renderer as tools/p0_check.py, so the cards say exactly what the
app will say. Output: docs/printables/paper-prototype.html (print from a browser, US Letter).
usage: python tools/make_printables.py
"""
import glob, os, html, sys
sys.path.insert(0, os.path.dirname(__file__))
import yaml
from p0_check import check_game, render, expand_role_edits, base_rules, dial_scope, dial_type, s12_order

ROOT = os.path.join(os.path.dirname(__file__), "..", "docs")
DECK_GAMES = ["g-closeout-attack-3v3", "s05-shot-clock-scoring", "s10-defense-scores", "s12-zone-restricted-defender", "s03-coach-calls-coverage", "s07-neutral-joker"]
START_TXT = {"closeout": "Defense passes out and closes out", "check-ball": "Check ball at the top", "coach-pass": "Coach passes to a wing",
             "dribble-at": "Attack from half court", "rebound": "Coach shoots; play the rebound", "inbound": "Inbound"}
ROT_TXT = {"none": "Same teams the whole block", "winners-stay": "Winners stay, next team on", "king": "King of the court",
           "three-team": "Offense → defense → off", "continuous": "Keep it going — lines rotate", "round-robin": "Round robin"}
RESTART_TXT = {"switch": "Ball goes to the other team, play on", "keep": "Make it, take it", "rotate": "Next group in", "phase-next": "Go the other way"}
e = html.escape

def load(gid):
    f = [p for p in glob.glob(os.path.join(ROOT, "fixtures", "games", "**", "*.yaml"), recursive=True) if os.path.basename(p).startswith(gid)][0]
    _, g, rep, card = check_game(f)
    return g, card

def level_line(name, d):
    t = dial_type(d)
    lv = d.get("levels")
    unit = {"distance": " ft", "duration": " s"}.get(t, "")
    if not lv: return f"{e(d.get('label', name))}: {d['default']}{unit}"
    return " · ".join(f"<b>L{i+1}</b> {v}{unit}" for i, v in enumerate(lv)) + f" <span class=dim>({e(d.get('label', name))})</span>"

def constraint_cards(g):
    out = []
    fmt = g["format"]
    roles = fmt.get("roles") or [r for p in fmt.get("phases", []) for r in p["roles"]]
    n_o = sum(r["team"] == "offense" for r in roles); n_d = sum(r["team"] == "defense" for r in roles)
    base_lines = [render(r) for r in base_rules(g.get("base_rules"))]
    st = fmt.get("start", {}); rs = fmt.get("restart", {})
    end = fmt.get("end", {})
    end_txt = {"score": f"First to {resolve_val(end.get('target'), g)}", "block": "Play until the clock ends", "time": f"{end.get('seconds')} s games"}.get(end.get("type"), "")
    gd = "".join(f"<li>{level_line(n, d)}</li>" for n, d in (g.get("dials") or {}).items() if not d.get("live"))
    out.append(f"""<div class="card base"><div class="kind">BASE GAME · {e(g['stage'])}</div><h3>{e(g['name'])}</h3>
      <div class="meta">{n_o}v{n_d} · {e(fmt['space']['court'])} court · {e(ROT_TXT.get(fmt.get('rotation',{}).get('type','none'),''))}</div>
      <ul><li>Start: {e(START_TXT.get(st.get('type'), st.get('type','')))}</li><li>After a score: {e(RESTART_TXT.get(rs.get('on_score'), rs.get('on_score','')))}</li>
      <li>{e(end_txt)}</li>{''.join(f'<li>{e(l)}</li>' for l in base_lines)}</ul>{f'<ul class=dials>{gd}</ul>' if gd else ''}</div>""")
    for c in g.get("constraints") or []:
        lines = [render(r) for r in g["_rules"] if (r.get("_c") or {}).get("id") == c["id"]]
        lines = [l for l in lines if l]
        rr = c.get("rules") or []
        for r in rr:
            if "count" in r["when"]:
                pass
        dl = "".join(f"<li>{level_line(n, d)}</li>" for n, d in (c.get("dials") or {}).items())
        out.append(f"""<div class="card {e(c['category'])}"><div class="kind">{e(c['category'].upper())} CONSTRAINT</div><h3>{e(c['name'])}</h3>
          <ul>{''.join(f'<li>{e(fix_dial_text(l, c, g))}</li>' for l in lines)}</ul>{f'<ul class=dials>{dl}</ul>' if dl else ''}
          <div class="foot"><b>Invites:</b> {e(', '.join(c.get('invites', [])))}<br><b>Keeps:</b> {e(', '.join(c.get('preserves', [])))}</div></div>""")
    return out

def resolve_val(v, g, c=None):
    if isinstance(v, str) and v.startswith("$"):
        return dial_scope(g, c)[v[1:]]["default"]
    return v

def fix_dial_text(line, c, g):
    return line  # render() already uses dial defaults (the card shows the current level)

def game_card(g, card):
    fmt = g["format"]; st = fmt.get("start", {}); rs = fmt.get("restart", {})
    score = [l for l in card if "=" in l and "turnover" not in l]
    notallowed = [l for l in card if "turnover" in l or "must" in l or "can't" in l or "before you" in l]
    other = [l for l in card if l not in score and l not in notallowed]
    cues = [q for c in g.get("constraints") or [] for q in (c.get("cues") or [])] or ["What did the help defender do?", "Where was the open space?"]
    return f"""<div class="page gamecard"><h1>{e(g['name'])}</h1>
      <div class="court">{court_svg(g)}</div>
      <h2>How to score</h2><ul>{''.join(f'<li>{e(l)}</li>' for l in score)}</ul>
      <h2>Not allowed</h2><ul>{''.join(f'<li>{e(l)}</li>' for l in notallowed) or '<li>—</li>'}</ul>
      <h2>How it runs</h2><ul><li>Start: {e(START_TXT.get(st.get('type'), ''))}</li><li>After a score: {e(RESTART_TXT.get(rs.get('on_score'), ''))}</li>
        <li>{e(ROT_TXT.get(fmt.get('rotation', {}).get('type', 'none'), ''))}</li>{''.join(f'<li>{e(l)}</li>' for l in other)}</ul>
      <h2>Think about</h2><ul>{''.join(f'<li>{e(q)}</li>' for q in cues[:2])}</ul>
      <div class="test">Test (W4.2): hand this to an assistant or a player. Can they run the game with no explanation? Note every question they ask: ______________________________</div></div>"""

def court_svg(g):
    # HS half court, basket frame → SVG (1 ft = 6 px), basket at the top.
    W, H, s = 50, 42, 6
    X = lambda x: (x + 25) * s; Y = lambda y: (y + 5.25) * s
    z = ""
    names = [zz["shape"].get("named") for c in g.get("constraints") or [] for zz in (c.get("zones") or []) if "named" in zz["shape"]]
    if "corner-L" in names: z += f'<rect x="0" y="0" width="{(25-19.75)*s}" height="{8.25*s}" class="bonus"/>'
    if "corner-R" in names: z += f'<rect x="{X(19.75)}" y="0" width="{(25-19.75)*s}" height="{8.25*s}" class="bonus"/>'
    if any(r.get("stay_in") == "paint" for c in g.get("constraints") or [] for r in (c.get("roles") or [])):
        z += f'<rect x="{X(-6)}" y="0" width="{12*s}" height="{19*s}" class="restrict"/>'
    return f"""<svg viewBox="0 0 {W*s} {H*s}" width="100%"><rect x="0" y="0" width="{W*s}" height="{H*s}" class="floor"/>{z}
      <rect x="{X(-6)}" y="0" width="{12*s}" height="{19*s}" class="line"/>
      <circle cx="{X(0)}" cy="{Y(13.75)}" r="{6*s}" class="line"/>
      <path d="M {X(-19.75)} 0 L {X(-19.75)} {Y(0)} A {19.75*s} {19.75*s} 0 0 0 {X(19.75)} {Y(0)} L {X(19.75)} 0" class="line"/>
      <circle cx="{X(0)}" cy="{Y(0)}" r="{0.75*s}" class="rim"/><line x1="{X(-3)}" y1="{Y(-1.25)}" x2="{X(3)}" y2="{Y(-1.25)}" class="board"/></svg>"""

def remote_sheet(g):
    allr = [r for r in g["_rules"] if not r.get("_mod")]
    btns = s12_order(allr)
    pts = {r.get("label") or r["id"]: next((t["points"] for t in r["then"] if "points" in t), 0) for r in allr}
    calls = [r.get("label") or r["id"] for r in allr if r["judge"] == "coach" and not any("points" in t and t["points"] for t in r["then"])]
    row = lambda b: f"<tr><td class=lbl>{e(b)} <b>+{pts.get(b, '')}</b></td><td class=box></td><td class=lbl>{e(b)} <b>+{pts.get(b, '')}</b></td><td class=box></td></tr>"
    return f"""<div class="page"><h1>Remote button sheet — {e(g['name'])}</h1>
      <p class=dim>W4.7 live dry-run. Tick marks instead of taps. Circle a mark to "undo". Basket: ____ · Block: ____ · Scorer: ________</p>
      <table class="remote"><tr><th>RED</th><th>ticks</th><th>BLUE</th><th>ticks</th></tr>{''.join(row(b) for b in btns)}</table>
      <h2>Calls (no points)</h2><table class="remote">{''.join(f'<tr><td class=lbl>{e(c)}</td><td class=box colspan=3></td></tr>' for c in calls + ['Stop ⇄ (possession)'])}</table>
      <h2>Clock and schedule</h2><table class="remote sched"><tr><th>Planned</th><th>Start</th><th>Warn 1:00</th><th>0:00</th><th>Decision at 0:00 (next / +1 from __ / finish poss / keep going)</th><th>End</th></tr>
      <tr><td>&nbsp;</td><td></td><td></td><td></td><td></td><td></td></tr></table>
      <h2>Notes</h2><p class=dim>Buttons you wished you had · buttons you never used · moments you couldn't tell which team: </p><div class=lines></div></div>"""

def tally_sheet():
    rows = "".join(f"<tr><td>{i}</td><td></td><td></td><td></td><td></td><td></td><td></td></tr>" for i in range(1, 9))
    return f"""<div class="page"><h1>Tally sheet (W4.3)</h1>
      <p class=dim>One block. Track 1–2 pinpoints live while someone else keeps score. Each box = one opportunity: ✔ correct, ✘ incorrect. Note the clock when you start and stop.</p>
      <table class="tally"><tr><th>#</th><th>Pinpoint 1: ____________ ✔/✘ (one box per opportunity)</th><th>count ✔</th><th>count opp</th><th>Pinpoint 2: ____________</th><th>count ✔</th><th>count opp</th></tr>{rows}</table>
      <p>Start ______  Stop ______  Active minutes ______ · Rate ✔/min ______ · Accuracy ______ · 7/8 met? ______</p>
      <h2>Dial turn (W4.4)</h2><table class="tally"><tr><th>Time</th><th>Dial</th><th>From → to</th><th>Why (success rate)</th><th>What changed in the next 2 minutes</th></tr>{''.join('<tr><td>&nbsp;</td><td></td><td></td><td></td><td></td></tr>' for _ in range(4))}</table>
      <h2>Capture (W4.5)</h2><p class=dim>When players invent something good, sketch it in 30 s:</p>
      <div class="sketches">{''.join(f'<div class=sk>{court_svg({"constraints": []})}<div class=dim>time ____ note __________</div></div>' for _ in range(2))}</div></div>"""

CSS = """
@page { size: letter; margin: 0.4in; }
* { box-sizing: border-box; }
body { font-family: "Segoe UI", Arial, sans-serif; color: #111; margin: 0; background: #fff; }
.page { page-break-after: always; break-after: page; padding: 0.1in 0; }
h1 { font-size: 20pt; margin: 0 0 6pt; } h2 { font-size: 12pt; margin: 10pt 0 4pt; border-bottom: 1px solid #999; }
.dim { color: #666; font-size: 9pt; }
.deck { display: grid; grid-template-columns: 5in; gap: 0.2in; justify-content: center; }
.card { width: 5in; height: 3in; border: 1.5px dashed #888; border-radius: 6px; padding: 10px 12px; overflow: hidden; position: relative; font-size: 10.5pt; break-inside: avoid; }
.card h3 { margin: 2px 0 4px; font-size: 15pt; } .card ul { margin: 2px 0; padding-left: 16px; } .card li { margin: 1px 0; }
.card .kind { font-size: 7.5pt; letter-spacing: .08em; color: #555; font-weight: 700; }
.card .meta { font-size: 9pt; color: #333; } .card .dials { list-style: none; padding: 0; margin-top: 6px; border-top: 1px solid #ddd; font-size: 9.5pt; }
.card .foot { position: absolute; bottom: 8px; left: 12px; right: 12px; font-size: 8pt; color: #444; }
.card.base { border-style: solid; border-width: 2.5px; } .card.task { border-left: 8px solid #1e88e5; } .card.environment { border-left: 8px solid #43a047; }
.card.character { border-left: 8px solid #fb8c00; } .card.individual { border-left: 8px solid #8e24aa; }
.gamecard ul { font-size: 14pt; } .gamecard h1 { font-size: 26pt; } .gamecard .court { width: 3.4in; float: right; margin: 0 0 8pt 12pt; }
.test { clear: both; margin-top: 14pt; font-size: 9pt; color: #555; border-top: 1px dotted #999; padding-top: 6pt; }
svg .floor { fill: #faf6ee; stroke: #333; stroke-width: 2; } svg .line { fill: none; stroke: #333; stroke-width: 1.5; }
svg .rim { fill: none; stroke: #d84315; stroke-width: 2; } svg .board { stroke: #333; stroke-width: 3; }
svg .bonus { fill: #ffe082; } svg .restrict { fill: #ef9a9a; opacity: .6; }
table { border-collapse: collapse; width: 100%; } th, td { border: 1px solid #555; padding: 4px 6px; font-size: 10pt; vertical-align: top; }
.remote td.lbl { width: 24%; font-size: 12pt; } .remote td.box { width: 26%; height: 0.55in; } .sched td { height: 0.5in; }
.tally td { height: 0.42in; } .lines { height: 1.4in; background: repeating-linear-gradient(#fff 0 26px, #bbb 26px 27px); }
.sketches { display: grid; grid-template-columns: 1fr 1fr; gap: 12px; } .sk svg { width: 100%; }
.cover h1 { font-size: 26pt; } .cover li { margin: 4pt 0; font-size: 11pt; }
@media screen { body { background: #e9e9e9; } .page { background: #fff; width: 8.5in; margin: 16px auto; padding: 0.4in; box-shadow: 0 1px 6px rgba(0,0,0,.2); } }
"""

def main():
    cards, gamecards, remotes = [], [], []
    for gid in DECK_GAMES:
        g, card = load(gid)
        cards += constraint_cards(g)
        remotes.append(remote_sheet(g))
    g, card = load("g-closeout-attack-3v3")
    gamecards.append(game_card(g, card))
    g2, card2 = load("s12-zone-restricted-defender")
    gamecards.append(game_card(g2, card2))
    deck_pages = "".join(f'<div class="page"><p class=dim><b>Constraint deck {i//3+1}</b> · W4.1 — cut on the dashed lines; stack a base card + constraint cards to build a game; dials print as Level 1/2/3.</p><div class="deck">{"".join(cards[i:i+3])}</div></div>' for i in range(0, len(cards), 3))
    cover = """<div class="page cover"><h1>Assistant Coach — P0 paper prototype</h1>
      <p>Generated from the fixtures by <code>tools/make_printables.py</code>; the wording is exactly what the app will render (S2 §7).</p>
      <ol><li><b>Constraint deck</b> (W4.1) — are the constraints separable? does order matter? do the sentences read clearly?</li>
      <li><b>Game cards</b> (W4.2) — can an assistant or a player run the game with no explanation?</li>
      <li><b>Remote button sheets</b> (W4.7) — tick marks instead of taps; do the buttons match what a scorer needs?</li>
      <li><b>Tally sheet</b> (W4.3–4.5) — how many pinpoints can you track live? dial turns; capture sketches.</li></ol>
      <p>Log findings in <code>docs/validation/paper-prototype.md</code> and <code>live-dry-run.md</code>.</p></div>"""
    doc = f"<!doctype html><html><head><meta charset=utf-8><title>Paper prototype</title><style>{CSS}</style></head><body>{cover}{deck_pages}{''.join(gamecards)}{''.join(remotes)}{tally_sheet()}</body></html>"
    out = os.path.join(ROOT, "printables", "paper-prototype.html")
    open(out, "w").write(doc)
    print("wrote", out, f"({len(cards)} cards, {len(gamecards)} game cards, {len(remotes)} remote sheets)")

if __name__ == "__main__":
    main()
