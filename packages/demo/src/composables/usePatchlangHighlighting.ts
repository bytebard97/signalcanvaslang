// packages/demo/src/composables/usePatchlangHighlighting.ts
import { StreamLanguage, LanguageSupport, HighlightStyle, syntaxHighlighting } from '@codemirror/language'
import { EditorView } from '@codemirror/view'
import { tags as t } from '@lezer/highlight'

// ─── Token colors ──────────────────────────────────────────────────────────────

const COLOR_KEYWORD = '#2DD4BF'
const COLOR_COMMENT = '#6B7280'
const COLOR_STRING = '#FFAC5A'
const COLOR_NUMBER = '#E0E7FF'
const COLOR_OPERATOR = '#60A5FA'
const COLOR_DEFAULT = '#e1e2ea'

// ─── Editor chrome colors ──────────────────────────────────────────────────────

const BG_EDITOR = '#0B0E13'
const BG_GUTTER = '#0B0E13'
const COLOR_GUTTER_TEXT = '#6B7280'
const COLOR_SELECTION = 'rgba(87, 241, 219, 0.1)'
const COLOR_CURSOR = '#57f1db'
const COLOR_ACTIVE_LINE = 'rgba(255, 255, 255, 0.03)'

// ─── PatchLang keywords ────────────────────────────────────────────────────────

const PATCHLANG_KEYWORDS = new Set([
  'template', 'instance', 'is', 'connect', 'bridge', 'ports', 'meta',
  'in', 'out', 'io', 'slot', 'use', 'ring', 'member', 'signal', 'flag',
  'stream', 'config', 'label', 'bus', 'route', 'for', 'over', 'generate',
  'bridge_group', 'link_group',
])

// ─── Stream tokenizer ──────────────────────────────────────────────────────────

interface TokenizerState { inString: boolean }

const patchlangStream = StreamLanguage.define<TokenizerState>({
  name: 'patchlang',

  startState(): TokenizerState {
    return { inString: false }
  },

  token(stream, state): string | null {
    if (stream.eatSpace()) return null

    if (stream.peek() === '#') {
      stream.skipToEnd()
      return 'comment'
    }

    if (stream.peek() === '"') {
      stream.next()
      state.inString = true
      while (!stream.eol()) {
        const ch = stream.next()
        if (ch === '"') { state.inString = false; break }
      }
      return 'string'
    }

    if (stream.match('->')) return 'operator'

    if (stream.match(/^[0-9]+(\.\.[0-9]+)?/)) return 'number'

    if (stream.match(/^[a-zA-Z_][a-zA-Z0-9_]*/)) {
      return PATCHLANG_KEYWORDS.has(stream.current()) ? 'keyword' : null
    }

    const ch = stream.next()
    if (ch && /^[{}[\]():,]$/.test(ch)) return 'punctuation'

    return null
  },

  copyState(state: TokenizerState): TokenizerState {
    return { inString: state.inString }
  },
})

// ─── Highlight style ───────────────────────────────────────────────────────────

const patchlangHighlightStyle = HighlightStyle.define([
  { tag: t.keyword, color: COLOR_KEYWORD },
  { tag: t.comment, color: COLOR_COMMENT, fontStyle: 'italic' },
  { tag: t.string, color: COLOR_STRING },
  { tag: t.number, color: COLOR_NUMBER },
  { tag: t.operator, color: COLOR_OPERATOR },
  { tag: t.punctuation, color: COLOR_DEFAULT },
])

// ─── Dark editor theme ─────────────────────────────────────────────────────────

export const patchlangDarkTheme = EditorView.theme(
  {
    '&': {
      backgroundColor: BG_EDITOR,
      color: COLOR_DEFAULT,
      height: '100%',
      fontFamily: 'ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace',
      fontSize: '12px',
    },
    '.cm-content': { caretColor: COLOR_CURSOR, padding: '12px 0' },
    '.cm-cursor': { borderLeftColor: COLOR_CURSOR },
    '.cm-gutters': { backgroundColor: BG_GUTTER, borderRight: 'none' },
    '.cm-lineNumbers .cm-gutterElement': {
      color: COLOR_GUTTER_TEXT,
      opacity: '0.5',
      minWidth: '2.5em',
      paddingRight: '12px',
    },
    '.cm-selectionBackground': { backgroundColor: COLOR_SELECTION },
    '&.cm-focused .cm-selectionBackground': { backgroundColor: COLOR_SELECTION },
    '.cm-activeLine': { backgroundColor: COLOR_ACTIVE_LINE },
    '.cm-scroller': { overflow: 'auto', lineHeight: '1.6' },
  },
  { dark: true },
)

// ─── Public API ────────────────────────────────────────────────────────────────

export function patchlangLanguage(): LanguageSupport {
  return new LanguageSupport(patchlangStream, [syntaxHighlighting(patchlangHighlightStyle)])
}
