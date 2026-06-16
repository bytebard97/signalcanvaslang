#!/bin/bash
# Two-pass PDF renderer that auto-sizes the content page.
#
# Pass 1: Render with oversized content page (500in)
# Pass 2: Measure actual content height with trim-content-page.py
# Pass 3: Re-render with the measured height
#
# Usage: ./render-fitted.sh [input.md | input-dir/] [output.pdf]
#
# Same interface as render-design-guide.sh but the content page is auto-sized.

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
RENDER_SCRIPT="$SCRIPT_DIR/render-design-guide.sh"
TRIM_SCRIPT="$SCRIPT_DIR/trim-content-page.py"

INPUT="${1:-$SCRIPT_DIR}"
OUTPUT="${2:-$SCRIPT_DIR/patchlang-v026-spec.pdf}"

CONTENT_PAGE=3
BOTTOM_MARGIN=1.0

# --- Pass 1: Render with oversized content page ---
echo "=== Pass 1: Oversized render (500in content page) ==="
TEMP_PDF="$(mktemp /tmp/render-fitted-XXXXXX.pdf)"
"$RENDER_SCRIPT" "$INPUT" "$TEMP_PDF"

# --- Pass 2: Measure actual content height ---
echo ""
echo "=== Pass 2: Measuring content height ==="
IDEAL_HEIGHT=$(python3 "$TRIM_SCRIPT" "$TEMP_PDF" --page "$CONTENT_PAGE" --margin "$BOTTOM_MARGIN")
echo "  Using content height: ${IDEAL_HEIGHT}in"

# --- Pass 3: Re-render with fitted height ---
echo ""
echo "=== Pass 3: Fitted render (${IDEAL_HEIGHT}in content page) ==="
"$RENDER_SCRIPT" --content-height "${IDEAL_HEIGHT}in" "$INPUT" "$OUTPUT"

# Clean up temp file
trash "$TEMP_PDF" 2>/dev/null || true

echo ""
echo "=== Done: $OUTPUT ==="
