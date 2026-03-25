//! Shared span/position conversion utilities for the LSP server.

use patchlang::error::{line_col, Span};
use tower_lsp::lsp_types::{Position, Range};

/// Convert a PatchLang Span to an LSP Range.
pub fn span_to_range(source: &str, span: &Span) -> Range {
    let (start_line, start_col) = line_col(source, span.start);
    let (end_line, end_col) = line_col(source, span.end);
    Range {
        start: Position {
            line: start_line.saturating_sub(1) as u32,
            character: start_col.saturating_sub(1) as u32,
        },
        end: Position {
            line: end_line.saturating_sub(1) as u32,
            character: end_col.saturating_sub(1) as u32,
        },
    }
}

/// Convert a byte offset pair to an LSP Range.
pub fn offsets_to_range(source: &str, start: usize, end: usize) -> Range {
    let (start_line, start_col) = line_col(source, start);
    let (end_line, end_col) = line_col(source, end);
    Range {
        start: Position {
            line: start_line.saturating_sub(1) as u32,
            character: start_col.saturating_sub(1) as u32,
        },
        end: Position {
            line: end_line.saturating_sub(1) as u32,
            character: end_col.saturating_sub(1) as u32,
        },
    }
}

/// Convert an LSP Position to a byte offset in the source string.
pub fn position_to_offset(source: &str, position: Position) -> Option<usize> {
    let mut current_line: u32 = 0;
    let mut current_col: u32 = 0;
    for (i, ch) in source.char_indices() {
        if current_line == position.line && current_col == position.character {
            return Some(i);
        }
        if ch == '\n' {
            if current_line == position.line {
                return Some(i);
            }
            current_line += 1;
            current_col = 0;
        } else {
            current_col += 1;
        }
    }
    if current_line == position.line && current_col == position.character {
        return Some(source.len());
    }
    if current_line == position.line {
        return Some(source.len());
    }
    None
}
