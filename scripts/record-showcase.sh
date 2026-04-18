#!/usr/bin/env bash
# Record the showcase example with asciinema.
#
# Usage: bash scripts/record-showcase.sh
# Output: recordings/showcase.cast
#
# During recording:
#   1. Wait for the showcase to start (~1s)
#   2. Navigate menu with ↑/↓ arrows, press Enter to swap model
#   3. Type a short message (e.g. "terminal is 3d now")
#   4. Press Tab to cycle reveal effects
#   5. Drag with left mouse to orbit, scroll to zoom (if your terminal mouse works)
#   6. Press F5 to replay reveal
#   7. Press Ctrl-C or q to quit — this also ends the recording
#
# Keep it under ~30 seconds for a tight demo gif.

set -euo pipefail

mkdir -p recordings
TARGET="recordings/showcase.cast"

echo "→ Recording to $TARGET"
echo "→ Exit the demo (q or Ctrl-C) to end the recording."
echo ""

# --overwrite so reruns don't append; --quiet to avoid asciinema's preamble
# leaking into the cast content.
exec asciinema rec \
  -t "happyterminals — showcase" \
  -c "cargo run --quiet --example showcase -p happyterminals" \
  --overwrite \
  --quiet \
  "$TARGET"
