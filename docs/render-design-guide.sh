#!/bin/bash
# Render PatchLang design guide to a styled PDF via HTML+CSS+WeasyPrint.
#
# Usage: ./render-design-guide.sh [input.md | input-dir/] [output.pdf]
#
# If input is a directory, concatenates all .md files in order:
#   frontmatter.md, overview.md, examples.md, project-structure.md,
#   frontend-guide.md, compiler.md, backend.md, language-reference.md, appendix.md
#
# Requirements: pandoc, weasyprint
#   brew install pandoc weasyprint

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
CONTENT_HEIGHT="500in"

# Parse optional flags
while [[ "$1" == --* ]]; do
    case "$1" in
        --content-height)
            CONTENT_HEIGHT="$2"
            shift 2
            ;;
        *)
            echo "Unknown option: $1" >&2
            exit 1
            ;;
    esac
done

INPUT="${1:-$SCRIPT_DIR/patchlang-design-guide}"
OUTPUT="${2:-$SCRIPT_DIR/patchlang-v026-spec.pdf}"
TMPDIR="$(mktemp -d)"

trap 'rm -rf "$TMPDIR"' EXIT

# If input is a directory, concatenate the parts in order
if [[ -d "$INPUT" ]]; then
    COMBINED="$TMPDIR/combined.md"
    FILE_ORDER=(
        frontmatter.md
        changelog.md
        reids-questions.md
        overview.md
        examples.md
        project-structure.md
        frontend-guide.md
        compiler.md
        backend.md
        language-reference.md
        appendix.md
    )
    for part in "${FILE_ORDER[@]}"; do
        if [[ -f "$INPUT/$part" ]]; then
            cat "$INPUT/$part" >> "$COMBINED"
            echo -e "\n\n" >> "$COMBINED"
        fi
    done
    INPUT="$COMBINED"
    echo "Rendering: directory -> $OUTPUT (${#FILE_ORDER[@]} parts)"
else
    echo "Rendering: $INPUT -> $OUTPUT"
fi

# --- Step 1: Extract and render Mermaid diagrams to PNG ---
COUNTER=0
PROCESSED="$TMPDIR/processed.md"
IN_MERMAID=false
MERMAID_BUF=""

while IFS= read -r line || [[ -n "$line" ]]; do
    if [[ "$line" =~ ^\`\`\`mermaid ]]; then
        IN_MERMAID=true
        MERMAID_BUF=""
        continue
    fi

    if $IN_MERMAID; then
        if [[ "$line" =~ ^\`\`\`$ ]]; then
            IN_MERMAID=false
            COUNTER=$((COUNTER + 1))
            PNG_FILE="$TMPDIR/diagram-${COUNTER}.png"

            echo "$MERMAID_BUF" > "$TMPDIR/diagram-${COUNTER}.mmd"
            echo "  Rendering diagram $COUNTER..."
            mmdc -i "$TMPDIR/diagram-${COUNTER}.mmd" -o "$PNG_FILE" \
                -c "$SCRIPT_DIR/mermaid-config.json" \
                -b "#0a1628" --width 900 --scale 3 2>/dev/null

            if command -v identify &>/dev/null; then
                IMG_DIMS=$(identify -format "%w %h" "$PNG_FILE")
                echo "    -> ${IMG_DIMS}"
            fi

            echo "" >> "$PROCESSED"
            echo "![](${PNG_FILE})" >> "$PROCESSED"
            echo "" >> "$PROCESSED"
        else
            MERMAID_BUF+="$line"$'\n'
        fi
        continue
    fi

    echo "$line" >> "$PROCESSED"
done < "$INPUT"

echo "  Rendered $COUNTER diagram(s)"

# --- Step 2: Extract metadata from YAML front matter ---
TITLE="$(grep '^title:' "$INPUT" | head -1 | sed 's/^title: *"//' | sed 's/"$//')"
SUBTITLE="$(grep 'title-subtitle:' "$INPUT" | head -1 | sed 's/.*title-subtitle: *"//' | sed 's/"$//')"
DATE="$(grep '^date:' "$INPUT" | head -1 | sed 's/^date: *//')"
LOGO_FILE="$(grep 'title-logo:' "$INPUT" | head -1 | sed 's/.*title-logo: *//')"
if [[ -n "$LOGO_FILE" && ! "$LOGO_FILE" = /* ]]; then
    LOGO_FILE="$(cd "$(dirname "$INPUT")" && pwd)/$LOGO_FILE"
fi

# --- Step 3: Convert markdown body to HTML fragment ---
BODY_MD="$TMPDIR/body.md"
# Strip YAML front matter (BSD sed compatible)
awk 'BEGIN{skip=0} /^---$/{skip++; next} skip>=2{print}' "$PROCESSED" > "$BODY_MD"

BODY_HTML="$TMPDIR/body.html"
pandoc "$BODY_MD" -t html5 -s --toc --toc-depth=2 --highlight-style=breezedark -o "$TMPDIR/body_full.html"
# Extract just the body content (between <body> and </body>) since we build our own HTML shell
sed -n '/<body>/,/<\/body>/p' "$TMPDIR/body_full.html" | sed '1s/.*<body>//' | sed '$s/<\/body>.*//' > "$BODY_HTML"

# --- Step 3b: Apply PatchLang syntax highlighting ---
python3 "$SCRIPT_DIR/colorize-patchlang.py" "$BODY_HTML" 2>&1 || echo "  (PatchLang highlighting skipped)"

# --- Step 4: Build standalone HTML with SignalCanvas-branded CSS ---
HTML_FILE="$TMPDIR/design-guide.html"

cat > "$HTML_FILE" << 'HTMLHEAD'
<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<link href="https://fonts.googleapis.com/css2?family=Inter:wght@300;400;500;600;700;800&family=JetBrains+Mono:wght@400;500&display=swap" rel="stylesheet">
<style>
@page {
  size: letter;
  margin: 0.6in;
  background: var(--bg);
}
@page title {
  size: letter;
  margin: 0;
  background: linear-gradient(160deg, #0a1628 0%, #0f2847 40%, #0d3b4f 70%, #0a1628 100%);
}
@page toc {
  size: letter;
  margin: 0.6in;
  background: var(--bg);
}
@page content {
  size: 8.5in CONTENT_HEIGHT_PLACEHOLDER;
  margin: 0.6in;
  background: var(--bg);
}

:root {
  /* SignalCanvas brand palette — teal to navy gradient */
  --sc-teal: #14b8a6;
  --sc-teal-light: #2dd4bf;
  --sc-teal-dark: #0d9488;
  --sc-navy: #0f2847;
  --sc-navy-light: #1a3a5c;
  --sc-navy-dark: #0a1628;
  --sc-cyan: #06b6d4;
  --sc-green: #10b981;
  --sc-amber: #f59e0b;
  --sc-rose: #f43f5e;

  --bg: #0a1628;
  --bg-card: #0f2040;
  --bg-section: #112240;
  --text: #e2e8f0;
  --text-secondary: #94a3b8;
  --text-heading: #f1f5f9;
  --text-accent: #2dd4bf;
  --border: #1e3a5f;
  --shadow-sm: 0 1px 2px rgba(0,0,0,0.2);
  --shadow-md: 0 4px 12px rgba(0,0,0,0.3);
  --radius: 10px;
}

* { margin: 0; padding: 0; box-sizing: border-box; }

body {
  font-family: 'Inter', system-ui, -apple-system, sans-serif;
  font-size: 10pt;
  line-height: 1.65;
  color: var(--text);
  background: var(--bg);
  -webkit-font-smoothing: antialiased;
}

/* ============================================
   TITLE PAGE — SignalCanvas branded
   ============================================ */
.title-page {
  page: title;
  page-break-after: always;
  height: 11in;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  text-align: center;
  padding: 2in 1.5in;
  position: relative;
  overflow: hidden;
}

.title-page::before {
  content: '';
  position: absolute;
  width: 700px;
  height: 700px;
  border-radius: 50%;
  background: radial-gradient(circle, rgba(20,184,166,0.12) 0%, transparent 70%);
  top: -250px;
  right: -200px;
}

.title-page::after {
  content: '';
  position: absolute;
  width: 500px;
  height: 500px;
  border-radius: 50%;
  background: radial-gradient(circle, rgba(6,182,212,0.08) 0%, transparent 70%);
  bottom: -150px;
  left: -150px;
}

.title-page img.logo {
  width: 420px;
  margin-bottom: 48px;
  opacity: 0.95;
  position: relative;
  z-index: 1;
}

.title-page h1 {
  font-size: 34pt;
  font-weight: 800;
  color: #ffffff;
  margin-bottom: 12px;
  letter-spacing: -1px;
  line-height: 1.1;
  position: relative;
  z-index: 1;
}

.title-page .subtitle {
  font-size: 13pt;
  color: var(--sc-teal-light);
  font-weight: 400;
  margin-bottom: 48px;
  position: relative;
  z-index: 1;
}

.title-page .divider {
  width: 80px;
  height: 3px;
  background: linear-gradient(90deg, var(--sc-teal), var(--sc-cyan));
  border-radius: 2px;
  margin-bottom: 48px;
  position: relative;
  z-index: 1;
}

.title-page .date {
  font-size: 10pt;
  color: #cbd5e1;
  font-weight: 300;
  letter-spacing: 2px;
  text-transform: uppercase;
  position: relative;
  z-index: 1;
}

.title-page .tagline {
  margin-top: 64px;
  font-size: 9pt;
  color: #64748b;
  font-style: italic;
  max-width: 460px;
  line-height: 1.6;
  position: relative;
  z-index: 1;
}

/* ============================================
   CONTENT AREA
   ============================================ */
.content {
  page: content;
  padding: 0;
  max-width: 100%;
}

h2 {
  font-size: 13pt;
  font-weight: 700;
  color: var(--text-heading);
  margin-top: 28px;
  margin-bottom: 12px;
  padding: 8px 16px;
  background: linear-gradient(135deg, var(--bg-section) 0%, #0f2a45 100%);
  border-left: 4px solid var(--sc-teal);
  border-radius: 0 var(--radius) var(--radius) 0;
  page-break-after: avoid;
  page-break-before: auto;
  letter-spacing: -0.3px;
}

h2:first-child {
  page-break-before: auto;
  padding-top: 12px;
  margin-top: 0;
}

h3 {
  font-size: 10pt;
  font-weight: 600;
  color: var(--sc-teal-light);
  margin-top: 16px;
  margin-bottom: 8px;
  page-break-after: avoid;
}

p {
  margin-bottom: 12px;
  color: #cbd5e1;
}

strong {
  color: #e2e8f0;
}

em {
  color: #94a3b8;
}

/* ============================================
   LISTS
   ============================================ */
ul, ol {
  margin-bottom: 12px;
  padding-left: 24px;
  color: #cbd5e1;
}

li {
  margin-bottom: 4px;
}

/* ============================================
   DIAGRAM CARDS
   ============================================ */
img:not(.logo) {
  display: block;
  margin: 16px auto;
  max-width: 100%;
  max-height: 7in;
  object-fit: contain;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  padding: 12px;
  background: var(--bg-card);
  page-break-inside: avoid;
  box-shadow: var(--shadow-md);
}

h2, h3 {
  page-break-after: avoid;
}

img:not(.logo), table {
  page-break-before: avoid;
}

/* ============================================
   TABLES — teal-branded
   ============================================ */
table {
  width: 100%;
  border-collapse: separate;
  border-spacing: 0;
  margin: 20px 0;
  font-size: 9pt;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  overflow: hidden;
  box-shadow: var(--shadow-sm);
  page-break-inside: avoid;
}

thead {
  background: linear-gradient(135deg, var(--sc-navy-light) 0%, var(--sc-teal-dark) 100%);
  color: white;
}

th {
  padding: 10px 16px;
  text-align: left;
  font-weight: 600;
  font-size: 8.5pt;
  text-transform: uppercase;
  letter-spacing: 0.8px;
}

td {
  padding: 10px 16px;
  border-bottom: 1px solid #0f2040;
  color: #cbd5e1;
}

tbody tr:last-child td {
  border-bottom: none;
}

tbody tr:nth-child(odd) {
  background: #0d1f38;
}

tbody tr:nth-child(even) {
  background: var(--bg);
}

td strong {
  color: var(--sc-teal-light);
  font-weight: 600;
}

td code {
  color: var(--sc-teal-light);
}

/* ============================================
   CODE — dark themed with teal accents
   ============================================ */
code {
  font-family: 'JetBrains Mono', 'Fira Code', monospace;
  font-size: 7.5pt;
  background: #0d1f38;
  color: #e2e8f0;
  border-radius: 4px;
  padding: 2px 6px;
}

pre {
  background: #060e1a;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  padding: 10px 14px;
  overflow-x: auto;
  margin: 16px 0;
  box-shadow: var(--shadow-md);
  break-inside: avoid;
  orphans: 3;
  widows: 3;
}

/* Syntax highlighting colors (breezedark overrides for our theme) */
code span.kw { color: #2dd4bf; font-weight: 600; } /* keywords */
code span.dt { color: #06b6d4; }                    /* data types */
code span.st { color: #a5f3fc; }                    /* strings */
code span.dv { color: #f59e0b; }                    /* decimal values / numbers */
code span.fu { color: #67e8f9; }                    /* functions */
code span.co { color: #64748b; font-style: italic; } /* comments */
code span.ot { color: #2dd4bf; }                    /* other tokens */
code span.at { color: #a78bfa; }                    /* attributes */
code span.va { color: #e2e8f0; }                    /* variables */
code span.cf { color: #2dd4bf; }                    /* control flow */
code span.op { color: #94a3b8; }                    /* operators */
code span.bu { color: #67e8f9; }                    /* builtins */
code span.er { color: #f43f5e; font-weight: bold; } /* errors */

pre code {
  background: none;
  border: none;
  padding: 0;
  color: #e2e8f0;
  white-space: pre-wrap;
  word-wrap: break-word;
}

/* ============================================
   TABLE OF CONTENTS
   ============================================ */
nav#TOC {
  page: toc;
  page-break-after: always;
  padding: 48px 20px;
}

nav#TOC > ul {
  list-style: none;
  padding-left: 0;
}

nav#TOC ul ul {
  list-style: none;
  padding-left: 24px;
}

nav#TOC li {
  margin-bottom: 6px;
  line-height: 1.8;
}

nav#TOC a {
  color: var(--sc-teal-light);
  text-decoration: none;
  font-size: 10pt;
}

nav#TOC > ul > li > a {
  font-size: 12pt;
  font-weight: 600;
  color: var(--text-heading);
}

nav#TOC::before {
  content: "Contents";
  display: block;
  font-size: 18pt;
  font-weight: 700;
  color: var(--text-heading);
  margin-bottom: 24px;
  padding-bottom: 12px;
  border-bottom: 2px solid var(--sc-teal-dark);
}

/* ============================================
   HORIZONTAL RULES
   ============================================ */
hr {
  border: none;
  height: 2px;
  background: linear-gradient(90deg, transparent, var(--sc-teal-dark), transparent);
  margin: 32px 0;
}

</style>
</head>
<body>
HTMLHEAD

# Replace content height placeholder with actual value
sed -i '' "s/CONTENT_HEIGHT_PLACEHOLDER/${CONTENT_HEIGHT}/" "$HTML_FILE"

# Write title page
cat >> "$HTML_FILE" << TITLEPAGE
<div class="title-page">
  <img class="logo" src="file://${LOGO_FILE}" alt="SignalCanvas">
  <h1>${TITLE}</h1>
  <div class="subtitle">${SUBTITLE}</div>
  <div class="divider"></div>
  <div class="date">${DATE}</div>
  <div class="tagline">Design decisions ratified through structured Socratic debate and cross-agent consensus. This document is the authoritative reference for all PatchLang v0.2.0 changes.</div>
</div>
TITLEPAGE

# Write body
echo '<div class="content">' >> "$HTML_FILE"
cat "$BODY_HTML" >> "$HTML_FILE"
echo '</div></body></html>' >> "$HTML_FILE"

# Remove any auto-generated notes from body
sed -i '' '/<em>This document was generated/d' "$HTML_FILE" 2>/dev/null || true

echo "  Generated HTML ($(du -h "$HTML_FILE" | cut -f1))"

# --- Step 5: Render to PDF via WeasyPrint ---
echo "  Rendering PDF via WeasyPrint..."

if ! command -v weasyprint &>/dev/null; then
    echo "ERROR: weasyprint not found. Install with: brew install weasyprint"
    echo "HTML file saved at: $HTML_FILE"
    exit 1
fi

weasyprint "file://$HTML_FILE" "$OUTPUT"

echo "Done: $OUTPUT ($(du -h "$OUTPUT" | cut -f1))"
