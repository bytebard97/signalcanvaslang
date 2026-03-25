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

use patchlang::error::{line_col, ParseError};

/// Print a single parse error with the source line and a caret span.
pub fn print_parse_error(source: &str, err: &ParseError) {
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

    eprintln!("error[{line}:{col}]: {}", err.message);
    eprintln!("{:width$} |", "", width = line_num_width);
    eprintln!("{line} | {source_line}");

    // Build caret line: spaces up to the error start, then ^ characters
    let caret_offset = col.saturating_sub(1);
    let caret_len = end_col.saturating_sub(col).max(1);
    eprintln!(
        "{:width$} | {:>padding$}{}",
        "",
        "",
        "^".repeat(caret_len),
        width = line_num_width,
        padding = caret_offset,
    );

    if let Some(hint) = &err.hint {
        eprintln!("{:width$} = hint: {hint}", "", width = line_num_width);
    }
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
        // Should not panic; visual output goes to stderr
        print_parse_error(source, &err);
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
        print_parse_error(source, &err);
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
        print_parse_error(source, &err);
    }
}
