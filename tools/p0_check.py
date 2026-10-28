#!/usr/bin/env python3
"""P0 paper-model checker (throwaway prototype, not P1 code).

Loads every game fixture, resolves it the way S5 §5 describes, runs the S2/S3/S4/S5
validator rules, renders card lines (S2 §7) and Remote buttons (S12), and compares
them with each fixture's `expect:` block. The Rust implementation in P1 must produce
the same results; this script is the oracle the golden tests were written against.

usage: python tools/p0_check.py docs/fixtures/games
"""
import sys, glob, os, yaml

EVENTS = set("""possession.start possession.end phase.change pass catch dribble drive paint-touch
screen cut closeout extra-pass assist shot make miss off-reb def-reb loose-ball stop deflection
steal block charge contest turnover violation foul role.enter role.leave zone.enter zone.exit
dial.set constraint.on constraint.off clock.expired clock.warning talk sprint-back capture""".split())
NAMED = set("""half full three-quarter cross-court inside-arc paint restricted block-L block-R
short-corner-L short-corner-R dunker-L dunker-R elbow-L elbow-R corner-L corner-R wing-L wing-R
top above-break mid-range beyond-arc backcourt half-court-line""".split())
TARGETS = set("""off off.create off.create.beat-1v1 off.create.ball-screen off.create.off-ball-screen
off.create.cut off.create.post-seal off.create.transition-push off.extend off.extend.drive-and-kick
off.extend.attack-closeout off.extend.extra-pass off.extend.short-roll off.extend.skip-pass off.finish
off.finish.rim-vs-contest off.finish.shot-selection off.finish.off-reb off.shape off.shape.spacing
off.shape.timing off.shape.relocation off.shape.balance def def.deny def.deny.contain
def.deny.screen-coverage def.deny.gap def.scramble def.scramble.help def.scramble.rotate
def.scramble.closeout def.scramble.x-out def.scramble.talk def.end def.end.contest def.end.box-out
def.end.secure trans.off trans.off.outlet trans.off.fill-lanes trans.off.early-advantage trans.def
trans.def.get-back trans.def.stop-ball trans.def.match-up char.humble char.hungry char.effort
char.attitude""".split())
READS = set("""read.closeout.high-fast read.closeout.short read.help.steps-up read.help.stays
read.ballscreen.drop read.ballscreen.hedge read.ballscreen.switch read.denial.overplay
read.2v1.commit read.2v1.stays-home read.ballscreen.help-on-roll""".split())
STAGES = {"coordination", "variability", "complex", "team-based", "performance"}
EVENT_NOUN = {"make": "Basket", "off-reb": "Offensive rebound", "stop": "Stop", "deflection": "Deflection",
              "charge": "Charge", "paint-touch": "paint touch", "catch": "catch", "drive": "drive",
              "dribble": "dribble", "pass": "pass", "shot": "shot"}

class Report:
    def __init__(s): s.err, s.warn, s.ok = [], [], []
    def e(s, m): s.err.append(m)
    def w(s, m): s.warn.append(m)

def base_rules(spec):
    out = []
    if not spec: return out
    for k, n in spec.items():
        v = {"two": 2, "three": 3}[k]
        out.append({"id": f"base.{k}", "label": "Basket" if v == 2 else "Three", "when": {"event": "make", "value": v},
                    "then": [{"points": n}], "judge": "coach", "_base": True})
    return out

def expand_role_edits(c):
    rules = []
    for r in c.get("roles", []) or []:
        if "restrict" in r:
            role = r["restrict"]
            if "stay_in" in r:
                rules.append({"id": f"{c['id']}.{role}-zone", "label": f"{'Help' if role=='H' else role} left {r['stay_in']}",
                              "when": {"event": "zone.exit", "zone": r["stay_in"], "actor": {"role": role}},
                              "then": [{"points": 1, "to": "offense"}], "judge": "coach",
                              "say": f"The {'help defender' if role=='H' else role} must stay in the {r['stay_in']}"})
            if r.get("cannot") == "shoot":
                rules.append({"id": f"{c['id']}.{role}-shot", "label": "Joker shot" if role == "J" else f"{role} shot",
                              "when": {"event": "shot", "actor": {"role": role}}, "then": [{"turnover": True}],
                              "judge": "players", "say": "The joker can't shoot" if role == "J" else f"{role} can't shoot"})
    return rules

def dial_scope(game, c):
    d = dict(game.get("dials") or {})
    d.update((c or {}).get("dials") or {})
    return d

def dial_type(d):
    for t in ("int", "distance", "duration", "enum", "bool"):
        if t in d: return t

def resolve_ref(v, scope, want, rep, where):
    if isinstance(v, str) and v.startswith("$"):
        name = v[1:]
        if name not in scope:
            rep.e(f"{where}: unresolved dial {v}"); return None
        t = dial_type(scope[name])
        if want and t not in want:
            rep.e(f"{where}: dial {v} is {t}, slot needs {want}")
        return scope[name]["default"]
    return v

def render(rule):
    if rule.get("say"): return rule["say"]
    w, then = rule["when"], rule["then"]
    pts = next((t for t in then if "points" in t), None)
    if "clock" in w:
        return None  # rendered by the clock line
    if "count" in w:
        n = w["_value"]; ev = w["count"]
        if any(t.get("turnover") for t in then):
            return f"More than {n} {ev}s = turnover" if w["op"] == ">" else f"{ev} limit = turnover"
    ev = w.get("event")
    if ev == "make":
        subj = "Three" if w.get("value") == 3 else "Basket"
        if "zone" in w and not isinstance(w["zone"], list) and w["zone"] == "paint" and w.get("value") == 2:
            subj = "Layup in the paint"
    elif ev == "catch" and "zone" in w:
        subj = "Catch in a corner zone"
    else:
        subj = EVENT_NOUN.get(ev, ev)
    after = rule.get("after") or []
    if after:
        step = after[0]["event"]
        if rule.get("within") is not None:
            subj = f"Score within {rule['_within']} seconds of the {EVENT_NOUN.get(step, step)}"
        else:
            art = "an" if EVENT_NOUN.get(step, step)[0] in "aeiou" else "a"
            subj = f"{subj} after {art} {EVENT_NOUN.get(step, step)}"
    if pts is not None:
        n = pts["points"]
        if pts.get("to") == "defense": return f"{subj} = +{n} for the defense"
        if subj == "Layup in the paint": return f"{subj} = +{n}"
        return f"{subj} = {n}"
    if any(t.get("turnover") for t in then): return f"{subj} = turnover"
    return None

def format_lines(fmt, game):
    lines = []
    for r in fmt.get("roles", []) or []:
        if r.get("team") == "neutral":
            lines.append(f"The joker plays for the {r.get('plays_for','offense')}")
    if (fmt.get("restart") or {}).get("on_score") == "keep":
        lines.append("Make it, take it")
    return lines

def check_game(path):
    rep = Report()
    g = yaml.safe_load(open(path))
    gid = g.get("id", os.path.basename(path))
    if g.get("stage") not in STAGES: rep.e("R7: missing/invalid stage")
    fmt = g.get("format", {})
    phases = fmt.get("phases")
    roles = [r for p in phases for r in p["roles"]] if phases else fmt.get("roles", [])
    teams = {r.get("team") for r in roles}
    if "offense" not in teams: rep.e("S4: no offense role")
    if "defense" not in teams and g.get("stage") != "coordination": rep.e("R1: no defense role")
    balls = fmt.get("balls", 1)
    zones = set()
    rules = base_rules(g.get("base_rules"))
    stack = list(g.get("constraints") or [])
    for c in stack + list(g.get("pool") or []):
        for k in ("invites", "preserves", "targets"):
            if not c.get(k): rep.e(f"R2/R3: constraint {c['id']} missing {k}")
        for t in c.get("targets") or []:
            if t not in TARGETS: rep.e(f"S6: constraint {c['id']} target {t} not in taxonomy")
        for z in c.get("zones") or []:
            zones.add(z["id"])
            def walk(v):
                if isinstance(v, dict): [walk(x) for x in v.values()]
                elif isinstance(v, list): [walk(x) for x in v]
                elif isinstance(v, str) and v.startswith("$"):
                    resolve_ref(v, dial_scope(g, c), {"distance"}, rep, f"zone {z['id']}")
            walk(z["shape"])
    active = []
    for c in stack:
        for r in (c.get("rules") or []) + expand_role_edits(c):
            r = dict(r); r["_c"] = c; active.append(r)
    pool_rules = []
    for c in g.get("pool") or []:
        for r in c.get("rules") or []:
            r = dict(r); r["_c"] = c; pool_rules.append(r)
    mods = ((g.get("block_example") or {}).get("modifiers") or [])
    for m in mods:
        r = dict(m["rule"]); r["_c"] = None; r["_mod"] = True; active.append(r)
    allrules = rules + active
    ids = [r["id"] for r in allrules + pool_rules]
    if len(ids) != len(set(ids)): rep.e("S2: duplicate rule ids")
    for r in allrules + pool_rules:
        where = f"rule {r['id']}"
        scope = dial_scope(g, r.get("_c"))
        w = r["when"]
        ev = w.get("event") or w.get("count")
        if ev and ev not in EVENTS: rep.e(f"{where}: unknown event {ev}")
        if "count" in w:
            w["_value"] = resolve_ref(w["value"], scope, {"int"}, rep, where)
            if balls > 1: rep.e(f"{where}: count pattern with balls={balls} (D-029)")
        z = w.get("zone")
        for zz in (z if isinstance(z, list) else [z] if z else []):
            if zz not in zones and zz not in NAMED: rep.e(f"{where}: zone {zz} not found")
        if r.get("after"):
            if len(r["after"]) > 2: rep.e(f"{where}: more than 2 after-steps (D-033)")
            if balls > 1: rep.e(f"{where}: possession-scoped sequence with balls={balls} (D-029)")
            for s in r["after"]:
                if s["event"] not in EVENTS: rep.e(f"{where}: unknown event {s['event']}")
        if r.get("within") is not None:
            if not r.get("after"): rep.e(f"{where}: within without after")
            r["_within"] = resolve_ref(r["within"], scope, {"duration"}, rep, where)
        for cnd in r.get("if") or []:
            if "dial" in cnd and cnd["dial"] not in scope: rep.e(f"{where}: condition dial {cnd['dial']} unresolved")
            if "count" in cnd and balls > 1: rep.e(f"{where}: count condition with balls={balls}")
        has_pts = any("points" in t and t["points"] for t in r["then"])
        judge = r.get("judge", "coach" if has_pts else "players")
        r["judge"] = judge
        if has_pts and judge == "players" : rep.e(f"{where}: points rule judged by players (S2 §8.5)")
        for o in r.get("overrides") or []:
            if o not in ids: rep.e(f"{where}: overrides unknown {o}")
    for k, v in (g.get("format") or {}).items():
        pass
    # dial bounds
    for scope_owner in [g] + stack:
        for name, d in (scope_owner.get("dials") or {}).items():
            t = dial_type(d)
            if t in ("int", "distance", "duration"):
                lo, hi = d[t]
                if not (lo <= d["default"] <= hi): rep.e(f"dial {name}: default out of bounds")
                for lv in d.get("levels") or []:
                    if not (lo <= lv <= hi): rep.e(f"dial {name}: level {lv} out of bounds")
    # render
    card = []
    for r in allrules:
        line = render(r)
        if line: card.append(line)
    sc = fmt.get("shot_clock")
    if sc:
        secs = resolve_ref(sc["seconds"], dial_scope(g, None), {"duration"}, rep, "shot_clock")
        card.append(f"Shot clock: {secs} seconds")
    fl = format_lines(fmt, g)
    # format lines go after base rules, before constraint rules (neutral/restart are 'how it runs')
    nb = len([r for r in allrules if r.get("_base")])
    card = card[:nb] + fl + card[nb:]
    buttons = [r.get("label") or r["id"] for r in allrules if r["judge"] == "coach" and any("points" in t and t["points"] for t in r["then"])]
    buttons.sort(key=lambda lbl: 0)  # stack order kept (S12 sorts by constraint first, see S12 §2)
    calls = [r.get("label") or r["id"] for r in allrules if not any("points" in t and t["points"] for t in r["then"]) and r["judge"] in ("coach", "players") and any(t.get("turnover") or ("points" in t) for t in r["then"])]
    exp = g.get("expect") or {}
    want = exp.get("card") or exp.get("card_block")
    if want is not None and want != card:
        rep.e(f"card mismatch:\n      got  {card}\n      want {want}")
    if "buttons_per_team" in exp:
        s12 = s12_order(allrules)
        if s12 != exp["buttons_per_team"]:
            rep.e(f"buttons mismatch: got {s12} want {exp['buttons_per_team']}")
    if "calls" in exp and calls != exp["calls"]:
        rep.e(f"calls mismatch: got {calls} want {exp['calls']}")
    if "board" in exp:
        b = board_line(active, g)
        if b != exp["board"]: rep.e(f"board mismatch: got {b!r} want {exp['board']!r}")
    for rd in g.get("reads") or []:
        if rd not in READS: rep.e(f"read {rd} not in pack")
    g["_rules"] = allrules
    return gid, g, rep, card

def s12_order(allrules):
    """S12 §2: base scoring rules first (same place in every game), then constraint rules in stack order."""
    pts = [r for r in allrules if r["judge"] == "coach" and any("points" in t and t["points"] for t in r["then"])]
    order = [r for r in pts if r.get("_base")] + [r for r in pts if not r.get("_base")]
    return [r.get("label") or r["id"] for r in order]

def board_line(active, g):
    out = []
    for r in active:
        if r.get("_mod"): continue
        then = r["then"]
        if all(t.get("tally") for t in then): continue
        if r.get("board"): out.append(r["board"]); continue
        pts = next((t for t in then if "points" in t), None)
        if pts and pts["points"]:
            out.append(f"{r.get('label')} = {pts['points']}")
        elif "count" in r["when"]:
            out.append(f"{r['when']['_value']} {r['when']['count']}s max")
        else:
            out.append(r.get("label") or r["id"])
        if len(out) == 3: break
    return " · ".join(out)

def main(root):
    files = sorted(glob.glob(os.path.join(root, "**", "*.yaml"), recursive=True))
    bad = 0
    for f in files:
        gid, g, rep, card = check_game(f)
        status = "FAIL" if rep.err else "ok"
        st = (g.get("stress") or {}).get("result", "")
        print(f"[{status:4}] {gid}  {st}")
        for m in rep.err: print(f"    ✘ {m}")
        for m in rep.warn: print(f"    ! {m}")
        bad += bool(rep.err)
    print(f"\n{len(files)} fixtures, {bad} failing")
    return 1 if bad else 0

if __name__ == "__main__":
    sys.exit(main(sys.argv[1] if len(sys.argv) > 1 else "docs/fixtures/games"))
