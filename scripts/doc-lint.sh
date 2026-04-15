#!/usr/bin/env bash
# scripts/doc-lint.sh
#
# Fails if any of the forbidden strings appear outside vendor/_reference/ or
# the single ADR that documents the rationale for avoiding them.
#
# Forbidden strings — see PROJECT.md and REQUIREMENTS.md HYG-09:
#   - "tui-vfx"         : effects layer is tachyonfx
#   - "Haskell bindings": descoped; Python is the only binding
#   - "pyo3-asyncio"    : abandoned upstream; we use pyo3-async-runtimes
#   - "cgmath"          : unmaintained; we use glam
#   - "tui-rs"          : deprecated crate name; we use ratatui
#
# Allowlist (rationale only, prose not code):
#   - docs/decisions/stack-rationale.md
#   - vendor/_reference/** (upstream copies are exempt)
#   - .eclusa/** (planning artifacts; discussed explicitly below)
#
# Exit 0 if clean, 1 with a human-readable error if not.

set -euo pipefail

FORBIDDEN=(
    "tui-vfx"
    "Haskell bindings"
    "pyo3-asyncio"
    "cgmath"
    "tui-rs"
)

# Paths allowed to contain the forbidden strings.
# Use `rg --glob '!...'` negations to exclude.
EXCLUDES=(
    '!vendor/_reference/**'
    '!docs/decisions/stack-rationale.md'
    '!.eclusa/**'                           # planning artifacts discuss the pivots explicitly
    '!scripts/doc-lint.sh'                  # this file literally contains the strings
    '!.github/workflows/ci.yml'             # if we ever name the step after them
    '!CHANGELOG.md'                         # migration notes can cite old names
    '!CONTRIBUTING.md'                      # enumerates the forbidden list as contributor rules
    '!.git/**'                              # git internal files (COMMIT_EDITMSG, hooks, etc.)
    '!target/**'
    '!Cargo.lock'
)

FAIL=0

for word in "${FORBIDDEN[@]}"; do
    # Build rg command: case-insensitive, show line numbers, excludes above.
    rg_cmd=(rg --hidden --line-number --color=never --no-heading --fixed-strings "$word")
    for exc in "${EXCLUDES[@]}"; do
        rg_cmd+=(--glob "$exc")
    done

    if OUTPUT=$("${rg_cmd[@]}" 2>/dev/null); then
        echo "::error::Forbidden string '$word' found outside allowlist:"
        echo "$OUTPUT" | sed 's/^/  /'
        FAIL=1
    fi
done

if [ "$FAIL" -eq 1 ]; then
    echo
    echo "Doc-lint failed. If a reference to a forbidden term is genuinely needed for"
    echo "rationale, add it ONLY to docs/decisions/stack-rationale.md. Otherwise, fix"
    echo "the file to use the current name (tachyonfx, pyo3-async-runtimes, glam, ratatui)."
    exit 1
fi

echo "Doc-lint: clean."
