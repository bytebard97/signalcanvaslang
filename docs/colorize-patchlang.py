#!/usr/bin/env python3
"""Post-process pandoc HTML to add PatchLang syntax highlighting to plain code blocks."""

import re
import sys

KEYWORDS = (
    'template|instance|is|connect|bridge|signal|flag|ports|meta|slot|'
    'config|route|bus|ring|member|use|stream|label|bridge_group|'
    'link_group|for|over|generate|mapping'
)

def colorize_patchlang(code):
    """Apply PatchLang highlighting using a two-pass approach.

    Pass 1: Collect all token spans with their positions.
    Pass 2: Build the output string, wrapping matched regions in <span> tags.
    This avoids regex passes clobbering each other's output.
    """
    tokens = []  # list of (start, end, style, text)

    # 1. Comments
    for m in re.finditer(r'#[^\n]*', code):
        tokens.append((m.start(), m.end(), 'color:#64748b;font-style:italic'))

    # 2. Strings
    for m in re.finditer(r'&quot;.*?&quot;|"[^"]*?"', code):
        tokens.append((m.start(), m.end(), 'color:#a5f3fc'))

    # 3. Keywords
    for m in re.finditer(rf'\b({KEYWORDS})\b', code):
        tokens.append((m.start(), m.end(), 'color:#2dd4bf;font-weight:bold'))

    # 4. Direction keywords
    for m in re.finditer(r'\b(in|out|io)\b(?=[(\s\[:])', code):
        tokens.append((m.start(), m.end(), 'color:#06b6d4;font-weight:bold'))

    # 5. Arrow operator
    for m in re.finditer(r'-&gt;|->', code):
        tokens.append((m.start(), m.end(), 'color:#a78bfa;font-weight:bold'))

    # 6. Numbers (but not inside HTML entities like &#123;)
    for m in re.finditer(r'(?<!&)(?<!#)\b(\d+)\b', code):
        tokens.append((m.start(), m.end(), 'color:#f59e0b'))

    if not tokens:
        return code

    # Sort by start position, longer matches first for ties
    tokens.sort(key=lambda t: (t[0], -(t[1] - t[0])))

    # Remove overlapping tokens (keep the first/longest match)
    filtered = []
    last_end = 0
    for start, end, style in tokens:
        if start >= last_end:
            filtered.append((start, end, style))
            last_end = end

    # Build output
    result = []
    pos = 0
    for start, end, style in filtered:
        result.append(code[pos:start])
        result.append(f'<span style="{style}">{code[start:end]}</span>')
        pos = end
    result.append(code[pos:])

    return ''.join(result)

def process_html(html):
    """Find plain <pre><code> blocks and apply PatchLang highlighting."""
    def replace_block(m):
        return '<pre><code>' + colorize_patchlang(m.group(1)) + '</code></pre>'

    return re.sub(
        r'<pre><code>(.*?)</code></pre>',
        replace_block,
        html,
        flags=re.DOTALL
    )

if __name__ == '__main__':
    filepath = sys.argv[1]
    with open(filepath, 'r') as f:
        html = f.read()
    result = process_html(html)
    with open(filepath, 'w') as f:
        f.write(result)
    print(f"  PatchLang highlighting applied to {filepath}")
