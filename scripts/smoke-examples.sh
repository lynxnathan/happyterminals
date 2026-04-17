#!/usr/bin/env bash
# scripts/smoke-examples.sh
#
# Compile-only smoke gate for the 5 headline happyterminals examples.
# Runs `cargo check --example <name> -p happyterminals` for each, fail-fast.
#
# Why compile-only: CI runners (GitHub Actions ubuntu-latest) are TTY-less,
# and crossterm's raw-mode acquire errors on non-TTY stdin — so `cargo run
# --example` crashes before user code executes. Compile-only catches type
# drift, missing asset paths referenced via include_str!, feature gating,
# and prelude gaps. Runtime smoke is deferred to a future phase that adds
# HT_SMOKE_EXIT_AFTER_MS or a PTY wrapper (see Phase 3.4 RESEARCH §Finding 4).
#
# Exit 0 if every headline compiles, 1 on first failure.

set -euo pipefail

HEADLINE=(
    "model-viewer"
    "particles"
    "transitions"
    "json-loader"
    "text-reveal"
)

echo "Smoke: cargo check --example <name> -p happyterminals"
echo "       (${#HEADLINE[@]} headline examples)"
echo

for name in "${HEADLINE[@]}"; do
    echo "→ $name"
    cargo check --example "$name" -p happyterminals
done

echo
echo "Smoke: all ${#HEADLINE[@]} headline examples compile."
