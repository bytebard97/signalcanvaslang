//! Human-friendly error display with source line snippets and carets.
//!
//! Converts parse errors into annotated source excerpts on stderr:
//!
//! ```text
//! error[2:14]: expected port direction: in, out, or io
//!   |
//! 2 |   ports { X: sideways }
//!   |              ^^^^^^^^
//! ```

use std::fmt::Write as _;

use patchlang::error::{line_col, ParseError};

/// Format a single parse error with the source line and a caret span.
/// Returns the formatted string (does not print directly).
pub fn format_parse_error(source: &str, err: &ParseError) -> String {
    let mut buf = String::new();

    let (line, col) = line_col(source, err.span.start);
    let end_col = if err.span.end > err.span.start {
        let (end_line, ec) = line_col(source, err.span.end);
        // If span crosses lines, only underline to end of start line
        if end_line != line {
            source
                .lines()
                .nth(line - 1)
                .map(|l| l.len() + 1)
                .unwrap_or(col + 1)
        } else {
            ec
        }
    } else {
        col + 1
    };

    let source_line = source.lines().nth(line - 1).unwrap_or("");
    let line_num_width = line.to_string().len();

    let _ = writeln!(buf, "error[{line}:{col}]: {}", err.message);
    let _ = writeln!(buf, "{:width$} |", "", width = line_num_width);
    let _ = writeln!(buf, "{line} | {source_line}");

    // Build caret line: spaces up to the error start, then ^ characters
    let caret_offset = col.saturating_sub(1);
    let caret_len = end_col.saturating_sub(col).max(1);
    let _ = writeln!(
        buf,
        "{:width$} | {:>padding$}{}",
        "",
        "",
        "^".repeat(caret_len),
        width = line_num_width,
        padding = caret_offset,
    );

    if let Some(hint) = &err.hint {
        let _ = writeln!(buf, "{:width$} = hint: {hint}", "", width = line_num_width);
    }

    buf
}

/// Print a single parse error with the source line and a caret span.
pub fn print_parse_error(source: &str, err: &ParseError) {
    eprint!("{}", format_parse_error(source, err));
    eprintln!();
}

/// Print all parse errors with source snippets.
pub fn print_errors(source: &str, errors: &[ParseError]) {
    for err in errors {
        print_parse_error(source, err);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use patchlang::error::{ParseError, Span};

    #[test]
    fn caret_points_at_correct_column() {
        let source = "template Bad {\n  ports { X: sideways }\n}";
        let err = ParseError {
            message: "expected port direction".to_string(),
            span: Span {
                start: 29,
                end: 37,
                file: None,
            },
            hint: None,
        };
        let output = format_parse_error(source, &err);
        assert!(output.contains("expected port direction"), "output should contain the error message");
        assert!(output.contains("ports { X: sideways }"), "output should contain the source line");
        assert!(output.contains("^^^^^^^^"), "output should contain caret characters");
    }

    #[test]
    fn single_char_span_shows_one_caret() {
        let source = "x";
        let err = ParseError {
            message: "unexpected".to_string(),
            span: Span {
                start: 0,
                end: 0,
                file: None,
            },
            hint: Some("try something else".to_string()),
        };
        let output = format_parse_error(source, &err);
        assert!(output.contains("unexpected"), "output should contain the error message");
        assert!(output.contains("^"), "output should contain at least one caret");
        assert!(output.contains("hint: try something else"), "output should contain the hint");
    }

    #[test]
    fn empty_source_does_not_panic() {
        let source = "";
        let err = ParseError {
            message: "unexpected EOF".to_string(),
            span: Span {
                start: 0,
                end: 0,
                file: None,
            },
            hint: None,
        };
        let output = format_parse_error(source, &err);
        assert!(output.contains("unexpected EOF"), "output should contain the error message");
        assert!(output.contains("^"), "output should contain at least one caret");
    }
}
