#!/usr/bin/env bash
# Convert an asciinema .cast into an animated .webp (+ .gif fallback).
#
# Pipeline:
#   1. Trim cast to TRIM_SECONDS (default 25s) — caps the asset size
#   2. agg       cast -> gif (animated, themed)
#   3. gifsicle  lossless optimize + color quantization (256 -> 64)
#   4. ffmpeg    gif -> animated webp, downscaled for README width
#
# Usage:
#   bash scripts/convert-recording.sh                               # defaults (25s, auto width)
#   bash scripts/convert-recording.sh recordings/showcase.cast      # specify input
#   TRIM_SECONDS=15 bash scripts/convert-recording.sh               # shorter clip
#   SCALE_WIDTH=600 bash scripts/convert-recording.sh               # narrower output
#
# Output:
#   recordings/<name>.gif    — fallback
#   recordings/<name>.webp   — primary, embedded in README

set -euo pipefail

INPUT="${1:-recordings/showcase.cast}"
TRIM_SECONDS="${TRIM_SECONDS:-25}"
SCALE_WIDTH="${SCALE_WIDTH:-720}"
WEBP_QUALITY="${WEBP_QUALITY:-55}"

BASENAME="$(basename "$INPUT" .cast)"
DIR="$(dirname "$INPUT")"
TRIMMED="$DIR/$BASENAME.trimmed.cast"
GIF="$DIR/$BASENAME.gif"
WEBP="$DIR/$BASENAME.webp"

if [ ! -f "$INPUT" ]; then
  echo "No recording found at $INPUT — run scripts/record-showcase.sh first"
  exit 1
fi

for bin in agg gifsicle ffmpeg python3; do
  if ! command -v "$bin" >/dev/null 2>&1; then
    echo "$bin not found in PATH — install it first"
    exit 1
  fi
done

echo "→ Trimming cast to first ${TRIM_SECONDS}s -> $TRIMMED"
python3 - "$INPUT" "$TRIMMED" "$TRIM_SECONDS" <<'PY'
import json, sys
src, dst, max_secs = sys.argv[1], sys.argv[2], float(sys.argv[3])
with open(src) as f_in, open(dst, 'w') as f_out:
    f_out.write(f_in.readline())  # header
    for line in f_in:
        line = line.rstrip('\n')
        if not line:
            continue
        try:
            ev = json.loads(line)
        except Exception:
            continue
        if not isinstance(ev, list) or len(ev) < 1:
            continue
        if ev[0] > max_secs:
            break
        f_out.write(line + '\n')
PY

echo "→ agg: trimmed cast -> $GIF"
agg --theme monokai --font-size 14 "$TRIMMED" "$GIF"

echo "→ gifsicle: lossless optimize + quantize to 64 colors"
gifsicle -O3 --colors 64 --batch "$GIF"

echo "→ ffmpeg libwebp: $GIF -> $WEBP (scale to ${SCALE_WIDTH}w, q=${WEBP_QUALITY})"
ffmpeg -y -hide_banner -loglevel warning \
  -i "$GIF" \
  -vf "scale=${SCALE_WIDTH}:-2:flags=lanczos,fps=20" \
  -c:v libwebp_anim \
  -lossless 0 \
  -compression_level 6 \
  -q:v "$WEBP_QUALITY" \
  -loop 0 \
  -preset default \
  -an \
  "$WEBP"

echo ""
echo "→ Sizes:"
ls -lh "$GIF" "$WEBP" | awk '{ printf "   %-6s  %s\n", $5, $NF }'
echo ""
echo "README embed:"
echo "   ![happyterminals showcase](${WEBP#recordings/})"
echo ""
echo "If the webp is still too large, try:"
echo "   TRIM_SECONDS=15 SCALE_WIDTH=640 WEBP_QUALITY=45 bash scripts/convert-recording.sh"
