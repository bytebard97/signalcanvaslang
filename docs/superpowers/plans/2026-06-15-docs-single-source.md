# Docs Single Source of Truth — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make `docs/` the single source of truth for all PatchLang documentation, served via GitHub Pages (Jekyll/Cayman) and rendered to PDF by the existing render script.

**Architecture:** Move the 13 design guide files from `docs/patchlang-design-guide/` up to `docs/`, add Jekyll frontmatter so GitHub Pages renders them as web pages, fix the render script path, update the layout to support mermaid diagrams, and write a clean index page. Trash the stale skeleton docs.

**Tech Stack:** Jekyll (Cayman theme, GitHub Pages legacy mode), mermaid.js CDN, pandoc + WeasyPrint (PDF render script, unchanged logic)

---

## File Map

| File | Action |
|------|--------|
| `docs/index.md`, `docs/cli.md`, `docs/wasm.md`, `docs/python.md`, `docs/quickstart.md`, `docs/examples.md`, `docs/installation.md` | Trash (stale skeleton) |
| `docs/patchlang-design-guide/*.md` (13 files) | Move up to `docs/` |
| `docs/patchlang-design-guide/` (dir) | Remove (empty after move) |
| `docs/_layouts/default.html` | Update nav links, add mermaid.js |
| `docs/_config.yml` | Exclude `frontmatter.md` from Jekyll |
| `docs/render-design-guide.sh` | Update INPUT path from `docs/patchlang-design-guide` to `docs` |
| `docs/index.md` (new) | Create clean nav page |
| `docs/AGENT-CONTEXT.md` | Remove stale `docs/specs/ast-builder-api.md` reference |

---

## Task 1: Trash stale skeleton docs

**Files:**
- Trash: `docs/index.md`, `docs/cli.md`, `docs/wasm.md`, `docs/python.md`, `docs/quickstart.md`, `docs/examples.md`, `docs/installation.md`

- [ ] **Step 1: Trash the files**

```bash
trash \
  docs/index.md \
  docs/cli.md \
  docs/wasm.md \
  docs/python.md \
  docs/quickstart.md \
  docs/examples.md \
  docs/installation.md
```

- [ ] **Step 2: Commit**

```bash
git add -A docs/
git commit -m "chore(docs): trash stale skeleton docs"
```

---

## Task 2: Move design guide files to docs/

**Files:**
- Move: `docs/patchlang-design-guide/*.md` → `docs/`
- Remove: `docs/patchlang-design-guide/` directory

- [ ] **Step 1: Move all 13 files**

```bash
mv docs/patchlang-design-guide/appendix.md docs/
mv docs/patchlang-design-guide/backend.md docs/
mv docs/patchlang-design-guide/changelog.md docs/
mv docs/patchlang-design-guide/compiler.md docs/
mv docs/patchlang-design-guide/debate-context.md docs/
mv docs/patchlang-design-guide/decisions.md docs/
mv docs/patchlang-design-guide/examples.md docs/
mv docs/patchlang-design-guide/frontend-guide.md docs/
mv docs/patchlang-design-guide/frontmatter.md docs/
mv docs/patchlang-design-guide/language-reference.md docs/
mv docs/patchlang-design-guide/overview.md docs/
mv docs/patchlang-design-guide/project-structure.md docs/
mv docs/patchlang-design-guide/reids-questions.md docs/
```

- [ ] **Step 2: Remove empty directory**

```bash
rmdir docs/patchlang-design-guide/
```

- [ ] **Step 3: Verify**

```bash
ls docs/*.md
```

Expected: all 13 files listed at `docs/` root, no `patchlang-design-guide/` directory.

- [ ] **Step 4: Commit**

```bash
git add -A docs/
git commit -m "chore(docs): move design guide to docs/ root"
```

---

## Task 3: Add Jekyll frontmatter to each design guide file

Jekyll needs a YAML front matter block (`--- ... ---`) at the top of each file to render it as a page with a title and clean permalink. `frontmatter.md` is PDF metadata, not a content page — exclude it from Jekyll via `_config.yml` instead.

**Files:**
- Modify: `docs/overview.md`, `docs/language-reference.md`, `docs/examples.md`, `docs/project-structure.md`, `docs/compiler.md`, `docs/backend.md`, `docs/frontend-guide.md`, `docs/decisions.md`, `docs/debate-context.md`, `docs/appendix.md`, `docs/changelog.md`, `docs/reids-questions.md`
- Modify: `docs/_config.yml`

- [ ] **Step 1: Add frontmatter to docs/overview.md**

Prepend to the top of the file (before the existing `# PatchLang Specification` heading):

```markdown
---
layout: default
title: Overview
permalink: /overview/
---
```

- [ ] **Step 2: Add frontmatter to docs/language-reference.md**

```markdown
---
layout: default
title: Language Reference
permalink: /language-reference/
---
```

- [ ] **Step 3: Add frontmatter to docs/examples.md**

```markdown
---
layout: default
title: Examples
permalink: /examples/
---
```

- [ ] **Step 4: Add frontmatter to docs/project-structure.md**

```markdown
---
layout: default
title: Project Structure
permalink: /project-structure/
---
```

- [ ] **Step 5: Add frontmatter to docs/compiler.md**

```markdown
---
layout: default
title: Compiler
permalink: /compiler/
---
```

- [ ] **Step 6: Add frontmatter to docs/backend.md**

```markdown
---
layout: default
title: Backend
permalink: /backend/
---
```

- [ ] **Step 7: Add frontmatter to docs/frontend-guide.md**

```markdown
---
layout: default
title: Frontend Guide
permalink: /frontend-guide/
---
```

- [ ] **Step 8: Add frontmatter to docs/decisions.md**

```markdown
---
layout: default
title: Design Decisions
permalink: /decisions/
---
```

- [ ] **Step 9: Add frontmatter to docs/debate-context.md**

```markdown
---
layout: default
title: Debate Context
permalink: /debate-context/
---
```

- [ ] **Step 10: Add frontmatter to docs/appendix.md**

```markdown
---
layout: default
title: Appendix
permalink: /appendix/
---
```

- [ ] **Step 11: Add frontmatter to docs/changelog.md**

```markdown
---
layout: default
title: Changelog
permalink: /changelog/
---
```

- [ ] **Step 12: Add frontmatter to docs/reids-questions.md**

```markdown
---
layout: default
title: Reid's Questions
permalink: /reids-questions/
---
```

- [ ] **Step 13: Exclude frontmatter.md from Jekyll in _config.yml**

`docs/frontmatter.md` contains PDF title-page metadata (logo path, subtitle, date) — not a web page. Add it to the Jekyll exclude list.

Replace the existing `docs/_config.yml`:

```yaml
title: PatchLang
description: "A domain-specific language for broadcast and live production signal flow"
theme: jekyll-theme-cayman
exclude:
  - frontmatter.md
  - render-design-guide.sh
  - render-fitted.sh
  - colorize-patchlang.py
  - mermaid-config.json
  - trim-content-page.py
```

- [ ] **Step 14: Commit**

```bash
git add docs/
git commit -m "chore(docs): add Jekyll frontmatter and exclude PDF-only files"
```

---

## Task 4: Update render scripts and fix PDF frontmatter corruption

Both render scripts (`render-design-guide.sh` and `render-fitted.sh`) hardcode `docs/patchlang-design-guide` as their default input and need updating. Additionally, after Task 3 adds Jekyll frontmatter (`--- layout: default ---`) to each content file, those blocks will corrupt the PDF unless we strip them per-file before concatenation. The global awk strip at line 122 of the script only handles the *first* frontmatter block (the PDF metadata from `frontmatter.md`) — all subsequent `---` delimiters cause YAML content to leak into the body.

**Files:**
- Modify: `docs/render-design-guide.sh`
- Modify: `docs/render-fitted.sh`

- [ ] **Step 1: Update the default INPUT path in render-design-guide.sh**

In `docs/render-design-guide.sh`, line 32, find:

```bash
INPUT="${1:-$SCRIPT_DIR/patchlang-design-guide}"
```

Replace with:

```bash
INPUT="${1:-$SCRIPT_DIR}"
```

- [ ] **Step 2: Add per-file frontmatter stripping in the concatenation loop**

In `docs/render-design-guide.sh`, find the concatenation loop (lines 54–59):

```bash
    for part in "${FILE_ORDER[@]}"; do
        if [[ -f "$INPUT/$part" ]]; then
            cat "$INPUT/$part" >> "$COMBINED"
            echo -e "\n\n" >> "$COMBINED"
        fi
    done
```

Replace with:

```bash
    for part in "${FILE_ORDER[@]}"; do
        if [[ -f "$INPUT/$part" ]]; then
            if [[ "$part" == "frontmatter.md" ]]; then
                cat "$INPUT/$part" >> "$COMBINED"
            else
                awk 'NR==1&&/^---$/{f=1;next} f&&/^---$/{f=0;next} f{next} {print}' "$INPUT/$part" >> "$COMBINED"
            fi
            echo -e "\n\n" >> "$COMBINED"
        fi
    done
```

This preserves `frontmatter.md`'s YAML (the PDF title-page metadata the render script parses) while stripping the Jekyll frontmatter added in Task 3 from every other file.

- [ ] **Step 3: Update the default INPUT path in render-fitted.sh**

In `docs/render-fitted.sh`, line 18, find:

```bash
INPUT="${1:-$SCRIPT_DIR/patchlang-design-guide}"
```

Replace with:

```bash
INPUT="${1:-$SCRIPT_DIR}"
```

- [ ] **Step 4: Smoke-test the render script (MANDATORY)**

```bash
cd docs && ./render-design-guide.sh
```

Expected: PDF generated at `docs/patchlang-v026-spec.pdf` with no errors, no raw YAML lines (`layout:`, `permalink:`, `title:`) visible in the body text. (Requires pandoc and weasyprint installed; skip only if neither is installed.)

- [ ] **Step 5: Commit**

```bash
git add docs/render-design-guide.sh docs/render-fitted.sh
git commit -m "chore(docs): update render scripts to use docs/ root and strip per-file Jekyll frontmatter"
```

---

## Task 5: Update layout — navigation and mermaid.js

The design guide files contain mermaid diagram blocks (` ```mermaid ... ``` `). Jekyll/kramdown renders these as `<pre><code class="language-mermaid">` — browsers won't render them as diagrams unless mermaid.js is loaded. Fix by adding a JS snippet that converts the code blocks to `<div class="mermaid">` before mermaid initializes.

Also update the header nav to link to the real pages.

**Files:**
- Modify: `docs/_layouts/default.html`

- [ ] **Step 1: Replace default.html with updated version**

```html
<!DOCTYPE html>
<html lang="{{ site.lang | default: 'en-US' }}">
<head>
  <meta charset="UTF-8">
  <meta http-equiv="X-UA-Compatible" content="IE=edge">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  {% seo %}
  <link rel="stylesheet" href="{{ '/assets/css/style.css?v=' | append: site.github.build_revision | relative_url }}">
</head>
<body>
  <header class="page-header" role="banner">
    <h1 class="project-name">{{ page.title | default: site.title }}</h1>
    <h2 class="project-tagline">{{ page.description | default: site.description }}</h2>
    <a href="{{ site.github.repository_url }}" class="btn">GitHub</a>
    <a href="{{ '/' | relative_url }}" class="btn">Home</a>
    <a href="{{ '/overview/' | relative_url }}" class="btn">Overview</a>
    <a href="{{ '/language-reference/' | relative_url }}" class="btn">Language Reference</a>
    <a href="{{ '/examples/' | relative_url }}" class="btn">Examples</a>
    <a href="{{ '/compiler/' | relative_url }}" class="btn">Compiler</a>
    <a href="{{ '/changelog/' | relative_url }}" class="btn">Changelog</a>
  </header>

  <main id="content" class="main-content" role="main">
    {{ content }}
    <footer class="site-footer">
      <span class="site-footer-owner">
        <a href="{{ site.github.repository_url }}">PatchLang</a> is maintained by
        <a href="https://github.com/SignalCanvas">SignalCanvas</a>.
      </span>
    </footer>
  </main>

  <script src="https://cdn.jsdelivr.net/npm/mermaid@10/dist/mermaid.min.js"></script>
  <script>
    document.addEventListener('DOMContentLoaded', function () {
      document.querySelectorAll('.language-mermaid').forEach(function (el) {
        var div = document.createElement('div');
        div.className = 'mermaid';
        div.textContent = el.textContent.trim();
        el.replaceWith(div);
      });
      mermaid.initialize({ startOnLoad: true, theme: 'dark' });
    });
  </script>
</body>
</html>
```

- [ ] **Step 2: Commit**

```bash
git add docs/_layouts/default.html
git commit -m "chore(docs): update nav and add mermaid.js rendering"
```

---

## Task 6: Write the index page

`docs/index.md` is the GitHub Pages homepage. Replace it with a clean nav page that links to all sections and serves as an LLM-friendly entry point.

**Files:**
- Create: `docs/index.md`

- [ ] **Step 1: Write docs/index.md**

```markdown
---
layout: default
title: PatchLang
---

<p align="center">
  <img src="images/logo.png" alt="SignalCanvas" width="400">
</p>

# PatchLang

A domain-specific language for describing signal flow in broadcast and live production environments. Human-readable, git-diffable, LLM-friendly.

```patch
template Rio3224 {
  meta { manufacturer: "Yamaha"  model: "Rio3224"  category: "Stagebox" }
  ports {
    Dante_Pri: io(etherCON) [Dante, primary]
    Mic_In[1..32]: in(XLR)
    Line_Out[1..16]: out(XLR)
  }
  bridge Mic_In -> Dante_Pri
}

instance Stage_Left is Rio3224 { location: "Stage Left Wing" }
connect Stage_Left.Dante_Pri -> FOH.Dante_Pri { cable: "Cat6a"  length: "30m" }
bridge Stage_Left.Mic_In[1..32] -> FOH.Dante_Ch[1..32]
```

---

## Language

| | |
|---|---|
| [Overview](overview/) | What PatchLang is and why it exists |
| [Language Reference](language-reference/) | Full grammar and syntax reference |
| [Examples](examples/) | Real-world signal flow examples |
| [Changelog](changelog/) | Version history |

## Architecture

| | |
|---|---|
| [Project Structure](project-structure/) | How PatchLang files and projects are organized |
| [Compiler](compiler/) | Compiler internals, DRC rules, WASM/Python bindings |
| [Backend](backend/) | Django API integration |
| [Frontend Guide](frontend-guide/) | How the Vue frontend consumes PatchLang |

## Design

| | |
|---|---|
| [Design Decisions](decisions/) | Recorded architectural decisions (D001–D018) |
| [Debate Context](debate-context/) | Structured debates behind key design choices |
| [Appendix](appendix/) | Reference tables and supplementary material |
| [Reid's Questions](reids-questions/) | Open spec questions and answers |
```

- [ ] **Step 2: Commit**

```bash
git add docs/index.md
git commit -m "chore(docs): add clean index nav page"
```

---

## Task 7: Fix stale path reference in AGENT-CONTEXT.md

`AGENT-CONTEXT.md` references `docs/specs/ast-builder-api.md` — a file we trashed. Remove that reference.

**Files:**
- Modify: `docs/AGENT-CONTEXT.md`

- [ ] **Step 1: Find and remove the reference**

Search for `ast-builder-api`:

```bash
grep -n "ast-builder-api" docs/AGENT-CONTEXT.md
```

- [ ] **Step 2: Remove the stale line**

In `docs/AGENT-CONTEXT.md`, find the line that says something like:

```
Full specification: `docs/specs/ast-builder-api.md`
```

Delete that line (or replace with `Full specification: see the Builder API section in docs/compiler.md`).

- [ ] **Step 3: Commit**

```bash
git add docs/AGENT-CONTEXT.md
git commit -m "chore(docs): remove stale ast-builder-api.md reference"
```

---

## Task 8: Preview locally, then push to master

Preview locally with Jekyll before pushing so you're not shipping broken docs to the live site.

**Note:** Pushing to master requires explicit authorization from Geoff at execution time — do not push without asking first.

- [ ] **Step 1: Install Jekyll dependencies (if not already done)**

```bash
cd /Users/ceres/Desktop/SignalCanvas/SignalCanvasLang/docs
bundle install
```

If no Gemfile exists, create one:
```ruby
source "https://rubygems.org"
gem "github-pages", group: :jekyll_plugins
```
Then re-run `bundle install`.

- [ ] **Step 2: Serve locally and verify**

```bash
bundle exec jekyll serve --source /Users/ceres/Desktop/SignalCanvas/SignalCanvasLang/docs --destination /tmp/jekyll-out --baseurl ""
```

Open `http://localhost:4000` in the browser. Check:
- Home page loads with nav buttons
- `/overview/`, `/language-reference/`, `/examples/`, `/compiler/`, `/changelog/` all load with content
- Mermaid diagrams render as charts (not raw code blocks) on `/compiler/`
- No stale links to `patchlang-design-guide/` paths

- [ ] **Step 3: Get authorization and push to master**

Ask Geoff: "Local preview looks good — OK to push to master?"

```bash
git push origin master
```

- [ ] **Step 4: Wait for Pages build (~1 min) and verify live site**

```
https://signalcanvas.github.io/SignalCanvasLang/
```

Check the same pages load correctly on the live site.

- [ ] **Step 5: Check Pages build status if anything looks wrong**

```bash
gh api repos/SignalCanvas/SignalCanvasLang/pages/builds --jq '.[0] | {status, error}'
```
