#!/usr/bin/env python3
"""Automates the "Boundaries" section of docs/plans/review-checklist.md.

Fails (exit 1) when a crate depends on something its layer forbids:
  - ac-domain, ac-live, ac-sport-basketball: no dioxus, tokio, axum, infinite-db, or any IO crate
  - only ac-store-idb may depend on infinite-db                         (D-017)
  - ac-web never depends on ac-store-idb or infinite-db                 (D-019)
  - ac-domain names contain no basketball words                         (D-005)

Run from the repo root:  python scripts/check_boundaries.py
"""
from __future__ import annotations

import re
import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CRATES = ROOT / "crates"

PURE = {"ac-domain", "ac-live", "ac-sport-basketball"}
FORBIDDEN_IN_PURE = {
    "dioxus", "tokio", "axum", "hyper", "reqwest", "infinite-db", "ac-store-idb",
    "async-std", "smol", "tungstenite", "tokio-tungstenite", "rusqlite", "notify",
}
ONLY_STORE = {"infinite-db"}
WEB_FORBIDDEN = {"infinite-db", "ac-store-idb"}

# D-005: identifiers in ac-domain must be sport-agnostic. Matched as whole words in
# type/fn/field/variant names (not in comments or string literals).
BASKETBALL_WORDS = {
    "basketball", "dribble", "dribbles", "rebound", "layup", "dunk", "paint",
    "freethrow", "free_throw", "three_pointer", "screen", "pick_and_roll",
    "backboard", "rim", "hoop", "baseline", "sideline", "nba", "ncaa",
}
IDENT = re.compile(r"\b(?:struct|enum|fn|type|trait|const|static|mod)\s+([A-Za-z_][A-Za-z0-9_]*)"
                   r"|^\s*(?:pub(?:\([^)]*\))?\s+)?([a-z_][a-z0-9_]*)\s*:", re.M)


def deps(manifest: dict) -> set[str]:
    out: set[str] = set()
    for key in ("dependencies", "build-dependencies"):
        out |= set(manifest.get(key, {}).keys())
    for target in manifest.get("target", {}).values():
        for key in ("dependencies", "build-dependencies"):
            out |= set(target.get(key, {}).keys())
    return out


def strip_comments_and_strings(src: str) -> str:
    src = re.sub(r"//[^\n]*", "", src)
    src = re.sub(r"/\*.*?\*/", "", src, flags=re.S)
    src = re.sub(r'"(?:\\.|[^"\\])*"', '""', src)
    return src


def snake_words(name: str) -> set[str]:
    snake = re.sub(r"(?<!^)(?=[A-Z])", "_", name).lower()
    parts = snake.split("_")
    return set(parts) | {snake}


def main() -> int:
    errors: list[str] = []
    for manifest_path in sorted(CRATES.glob("*/Cargo.toml")):
        name = manifest_path.parent.name
        manifest = tomllib.loads(manifest_path.read_text(encoding="utf-8"))
        d = deps(manifest)
        if name in PURE:
            for bad in sorted(d & FORBIDDEN_IN_PURE):
                errors.append(f"{name}: depends on `{bad}` (pure crate: no UI/async/IO/DB)")
        if name != "ac-store-idb":
            for bad in sorted(d & ONLY_STORE):
                errors.append(f"{name}: depends on `{bad}` (only ac-store-idb may, D-017)")
        if name == "ac-web":
            for bad in sorted(d & WEB_FORBIDDEN):
                errors.append(f"{name}: depends on `{bad}` (Board/Remote never open the DB, D-019)")

    for rs in sorted((CRATES / "ac-domain" / "src").rglob("*.rs")):
        src = strip_comments_and_strings(rs.read_text(encoding="utf-8"))
        for m in IDENT.finditer(src):
            ident = m.group(1) or m.group(2)
            hit = snake_words(ident) & BASKETBALL_WORDS
            if hit:
                line = src.count("\n", 0, m.start()) + 1
                errors.append(f"{rs.relative_to(ROOT)}:{line}: `{ident}` uses basketball word(s) "
                              f"{sorted(hit)} in ac-domain (D-005 — move it to ac-sport-basketball)")

    if errors:
        print("Boundary check FAILED:")
        for e in errors:
            print("  -", e)
        return 1
    print("Boundary check passed.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
