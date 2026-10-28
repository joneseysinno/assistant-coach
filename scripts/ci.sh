#!/usr/bin/env bash
# Assistant Coach CI (bash). Run from the repo root: ./scripts/ci.sh
# Same steps as scripts/ci.ps1 and .github/workflows/ci.yml.
set -euo pipefail
step() { echo "==> $1"; shift; "$@"; }

step "boundaries (review-checklist)" python3 scripts/check_boundaries.py
step "fixtures oracle (p0_check)"    python3 tools/p0_check.py docs/fixtures/games
step "cargo fmt --check"             cargo fmt --all -- --check
step "cargo clippy -D warnings"      cargo clippy --workspace --all-targets -- -D warnings
step "cargo test"                    cargo test --workspace
echo "CI green."
