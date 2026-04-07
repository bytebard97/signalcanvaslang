//! Convert PatchLang parse errors and DRC diagnostics to LSP Diagnostic format.

use patchlang::drc::{self, Severity as DrcSeverity};
use patchlang::error::{ParseError, ParseResult};
use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, Range};

use crate::span_utils::offsets_to_range;

/// Source identifier for parse-error diagnostics.
const SOURCE_PARSER: &str = "patchlang";
/// Source identifier for DRC diagnostics.
const SOURCE_DRC: &str = "patchlang-drc";

/// Map DRC severity to LSP DiagnosticSeverity.
fn map_severity(severity: &DrcSeverity) -> DiagnosticSeverity {
    match severity {
        DrcSeverity::Error => DiagnosticSeverity::ERROR,
        DrcSeverity::Warning => DiagnosticSeverity::WARNING,
        DrcSeverity::Info => DiagnosticSeverity::INFORMATION,
    }
}

/// Convert a single parse error to an LSP Diagnostic.
fn parse_error_to_diagnostic(source: &str, error: &ParseError) -> Diagnostic {
    let range = offsets_to_range(source, error.span.start, error.span.end);
    let message = match &error.hint {
        Some(hint) => format!("{} (hint: {})", error.message, hint),
        None => error.message.clone(),
    };
    Diagnostic {
        range,
        severity: Some(DiagnosticSeverity::ERROR),
        source: Some(SOURCE_PARSER.to_string()),
        message,
        ..Default::default()
    }
}

/// Convert a single DRC diagnostic to an LSP Diagnostic.
fn drc_to_diagnostic(source: &str, diag: &drc::Diagnostic) -> Diagnostic {
    let range = match &diag.span {
        Some(span) => offsets_to_range(source, span.start, span.end),
        None => Range::default(),
    };
    let mut message = format!("[{:?}] {}", diag.layer, diag.message);
    if let Some(fix) = &diag.fix {
        message.push_str(&format!(" (fix: {})", fix));
    }
    Diagnostic {
        range,
        severity: Some(map_severity(&diag.severity)),
        source: Some(SOURCE_DRC.to_string()),
        message,
        ..Default::default()
    }
}

/// Build all LSP diagnostics from a parse result and DRC run.
///
/// Runs DRC only when parsing succeeds (no parse errors), matching the
/// behavior of `patchlang::check()` — DRC on a partial AST produces
/// misleading false positives.
pub fn build_diagnostics(source: &str) -> (ParseResult, Vec<Diagnostic>) {
    let parse_result = patchlang::parse(source);

    let mut lsp_diagnostics: Vec<Diagnostic> = parse_result
        .errors
        .iter()
        .map(|e| parse_error_to_diagnostic(source, e))
        .collect();

    if parse_result.errors.is_empty() {
        let drc_diags = drc::run_all(&parse_result.program, &patchlang::LibraryContext::empty());
        lsp_diagnostics.extend(drc_diags.iter().map(|d| drc_to_diagnostic(source, d)));
    }

    (parse_result, lsp_diagnostics)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn span_to_range_first_line() {
        let source = "template Foo {\n}";
        // byte 9 = 'F' in "Foo", line 1 col 10 -> LSP (0, 9)
        let range = offsets_to_range(source, 9, 12);
        assert_eq!(range.start.line, 0);
        assert_eq!(range.start.character, 9);
        assert_eq!(range.end.line, 0);
        assert_eq!(range.end.character, 12);
    }

    #[test]
    fn span_to_range_second_line() {
        let source = "line one\nline two";
        // "two" starts at byte 14, line 2 col 6 -> LSP (1, 5)
        let range = offsets_to_range(source, 14, 17);
        assert_eq!(range.start.line, 1);
        assert_eq!(range.start.character, 5);
    }

    #[test]
    fn map_severity_error() {
        assert_eq!(map_severity(&DrcSeverity::Error), DiagnosticSeverity::ERROR);
    }

    #[test]
    fn map_severity_warning() {
        assert_eq!(
            map_severity(&DrcSeverity::Warning),
            DiagnosticSeverity::WARNING
        );
    }

    #[test]
    fn map_severity_info() {
        assert_eq!(
            map_severity(&DrcSeverity::Info),
            DiagnosticSeverity::INFORMATION
        );
    }

    #[test]
    fn convert_parse_error_to_lsp_diagnostic() {
        let source = "template { }";
        let error = ParseError {
            message: "expected template name".to_string(),
            span: patchlang::Span {
                start: 9,
                end: 10,
                file: None,
            },
            hint: Some("add a name after 'template'".to_string()),
        };
        let diag = parse_error_to_diagnostic(source, &error);
        assert_eq!(diag.severity, Some(DiagnosticSeverity::ERROR));
        assert_eq!(diag.source, Some("patchlang".to_string()));
        assert!(diag.message.contains("expected template name"));
        assert!(diag.message.contains("hint:"));
    }

    #[test]
    fn convert_parse_error_without_hint() {
        let source = "template Foo";
        let error = ParseError {
            message: "unexpected end of input".to_string(),
            span: patchlang::Span {
                start: 12,
                end: 12,
                file: None,
            },
            hint: None,
        };
        let diag = parse_error_to_diagnostic(source, &error);
        assert_eq!(diag.message, "unexpected end of input");
    }

    #[test]
    fn convert_drc_warning_to_lsp_diagnostic() {
        let source = "template Foo {\n  ports { X: out }\n}";
        let drc_diag = drc::Diagnostic {
            severity: DrcSeverity::Warning,
            layer: drc::DRCLayer::Structural,
            message: "unused port".to_string(),
            span: Some(patchlang::Span {
                start: 25,
                end: 26,
                file: None,
            }),
            source: None,
            target: None,
            fix: Some("connect or remove X".to_string()),
        };
        let diag = drc_to_diagnostic(source, &drc_diag);
        assert_eq!(diag.severity, Some(DiagnosticSeverity::WARNING));
        assert_eq!(diag.source, Some("patchlang-drc".to_string()));
        assert!(diag.message.contains("unused port"));
        assert!(diag.message.contains("fix:"));
    }

    #[test]
    fn convert_drc_without_span() {
        let source = "template Foo { ports { X: out } }";
        let drc_diag = drc::Diagnostic {
            severity: DrcSeverity::Info,
            layer: drc::DRCLayer::Logical,
            message: "general info".to_string(),
            span: None,
            source: None,
            target: None,
            fix: None,
        };
        let diag = drc_to_diagnostic(source, &drc_diag);
        assert_eq!(diag.range, Range::default());
        assert_eq!(diag.severity, Some(DiagnosticSeverity::INFORMATION));
    }

    #[test]
    fn build_diagnostics_with_parse_errors() {
        let source = "template {";
        let (result, diags) = build_diagnostics(source);
        assert!(!result.errors.is_empty());
        assert!(!diags.is_empty());
        // All should be parser errors, no DRC
        for d in &diags {
            assert_eq!(d.source, Some("patchlang".to_string()));
        }
    }

    #[test]
    fn build_diagnostics_clean_source() {
        let source = "template Foo {\n  ports {\n    X: out\n  }\n}";
        let (result, _diags) = build_diagnostics(source);
        assert!(result.errors.is_empty());
        // DRC may or may not produce diagnostics for this simple template,
        // but no parse errors should appear
    }

    #[test]
    fn build_diagnostics_invalid_source_has_error() {
        let source = "template { ports { X: sideways } }";
        let (_result, diags) = build_diagnostics(source);
        assert!(!diags.is_empty(), "invalid source should produce diagnostics");
        assert_eq!(diags[0].severity, Some(DiagnosticSeverity::ERROR));
        assert!(
            !diags[0].message.is_empty(),
            "diagnostic message should be meaningful: {}",
            diags[0].message
        );
    }

    #[test]
    fn build_diagnostics_drc_error_has_warning_severity() {
        let source = "template D { ports { Out: out } }\n\
                       instance A is D\n\
                       instance B is D\n\
                       connect A.Out -> B.Out";
        let (_result, diags) = build_diagnostics(source);
        assert!(
            diags.iter().any(|d| d.severity == Some(DiagnosticSeverity::ERROR)),
            "direction violation should produce an error diagnostic, got: {:?}",
            diags.iter().map(|d| (&d.message, &d.severity)).collect::<Vec<_>>()
        );
    }
}
