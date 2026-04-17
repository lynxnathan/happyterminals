#!/usr/bin/env bash
# scripts/doc-lint-examples.sh
#
# Enforces the DEMO-05 header contract on every example under
# crates/happyterminals/examples/.
#
# Headline examples MUST contain, in the first 50 lines of main.rs:
#   - at least one `//!` line
#   - the exact phrase "Features exercised:"
#   - the exact phrase "cargo run --example <name> -p happyterminals"
#     (where <name> matches the directory/file stem)
#
# Dev utilities (color-test, static_grid) are exempt from "Features exercised:"
# but MUST contain the marker "Developer utility — not a demo" verbatim.
#
# Exit 0 if every example passes, 1 on first violation (aggregated report).

set -euo pipefail

EXAMPLES_DIR="crates/happyterminals/examples"
DEV_UTILITIES=("color-test" "static_grid")

# Resolve the list of example entry points.
# Subfolder examples: examples/<name>/main.rs
# Single-file examples: examples/<name>.rs
mapfile -t ENTRY_POINTS < <(
    {
        find "$EXAMPLES_DIR" -mindepth 2 -maxdepth 2 -name main.rs -type f
        find "$EXAMPLES_DIR" -mindepth 1 -maxdepth 1 -name '*.rs' -type f
    } | sort
)

if [ "${#ENTRY_POINTS[@]}" -eq 0 ]; then
    echo "::error::No example entry points found under $EXAMPLES_DIR"
    exit 1
fi

is_dev_utility() {
    local stem="$1"
    for u in "${DEV_UTILITIES[@]}"; do
        [ "$stem" = "$u" ] && return 0
    done
    return 1
}

FAIL=0

for entry in "${ENTRY_POINTS[@]}"; do
    # Derive the example name — subfolder dir OR file stem.
    if [[ "$entry" == */main.rs ]]; then
        stem=$(basename "$(dirname "$entry")")
    else
        stem=$(basename "$entry" .rs)
    fi

    head_block=$(head -n 50 "$entry")

    # Every example must have a //! header block.
    if ! echo "$head_block" | grep -qE '^//!'; then
        echo "::error::$entry missing //! header block in first 50 lines"
        FAIL=1
        continue
    fi

    if is_dev_utility "$stem"; then
        # Dev utility marker required.
        if ! echo "$head_block" | grep -qF 'Developer utility — not a demo'; then
            echo "::error::$entry (dev utility) missing 'Developer utility — not a demo' marker"
            FAIL=1
        fi
    else
        # Headline example: Features exercised + Run with lines required.
        if ! echo "$head_block" | grep -qF 'Features exercised:'; then
            echo "::error::$entry missing 'Features exercised:' section in header"
            FAIL=1
        fi
        expected_run="cargo run --example ${stem} -p happyterminals"
        if ! echo "$head_block" | grep -qF "$expected_run"; then
            echo "::error::$entry missing 'Run with' line: $expected_run"
            FAIL=1
        fi
    fi
done

if [ "$FAIL" -eq 1 ]; then
    echo
    echo "doc-lint-examples failed. Fix the headers per Phase 3.4 DEMO-05 contract."
    exit 1
fi

echo "doc-lint-examples: ${#ENTRY_POINTS[@]} example headers clean."
