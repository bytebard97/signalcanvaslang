mod error_display;

use clap::{Parser, Subcommand};
use std::collections::{HashMap, HashSet, VecDeque};
use std::io::Read;
use std::process;

/// PatchLang compiler and validation tools.
#[derive(Parser)]
#[command(name = "patchlang", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Parse a .patch file and output the AST as JSON
    Parse {
        /// Path to a .patch file (reads stdin if omitted)
        file: Option<String>,
    },
    /// Parse + run DRC checks, output CheckResult JSON
    Check {
        /// Path to a .patch file (reads stdin if omitted)
        file: Option<String>,
    },
    /// Compile a multi-file project from a project.json manifest
    Compile {
        /// Path to project.json
        manifest: String,
    },
    /// Validate a .layout.json file against the layout schema
    ValidateLayout {
        /// Path to .layout.json
        file: String,
    },
    /// Cross-validate a .patch and .layout.json pair
    ValidateConsistency {
        /// Path to .patch file
        patch: String,
        /// Path to .layout.json file
        layout: String,
    },
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Parse { file } => cmd_parse(file),
        Commands::Check { file } => cmd_check(file),
        Commands::Compile { manifest } => cmd_compile(manifest),
        Commands::ValidateLayout { file } => cmd_validate_layout(file),
        Commands::ValidateConsistency { patch, layout } => {
            cmd_validate_consistency(patch, layout)
        }
    }
}

// ── Helpers ─────────────────────────────────────────────────────────

/// Read source from a file path or stdin (when no path given).
fn read_source(file: Option<String>) -> String {
    match file {
        Some(path) => read_file(&path),
        None => {
            let mut buf = String::new();
            std::io::stdin().read_to_string(&mut buf).unwrap_or_else(|e| {
                eprintln!("error: cannot read stdin: {e}");
                process::exit(2);
            });
            buf
        }
    }
}

/// Read a file by path (required argument, no stdin fallback).
fn read_file(path: &str) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!("error: cannot read '{path}': {e}");
        process::exit(2);
    })
}

/// Serialize a value to pretty JSON, exiting on failure.
fn to_json(value: &impl serde::Serialize) -> String {
    serde_json::to_string_pretty(value).unwrap_or_else(|e| {
        eprintln!("error: serialization failed: {e}");
        process::exit(2);
    })
}

// ── Subcommand implementations ──────────────────────────────────────

fn cmd_parse(file: Option<String>) {
    let source = read_source(file);
    let result = patchlang::parse(&source);

    if !result.is_valid() {
        error_display::print_errors(&source, &result.errors);
    }

    println!("{}", to_json(&result.program));

    if !result.is_valid() {
        process::exit(1);
    }
}

fn cmd_check(file: Option<String>) {
    let source = read_source(file);

    // Parse first for rich error display (raw ParseError has byte spans)
    let parse_result = patchlang::parse(&source);
    if !parse_result.is_valid() {
        error_display::print_errors(&source, &parse_result.errors);
    }

    // Full check for JSON output (includes DRC diagnostics)
    let check_result = patchlang::check(&source);

    for diag in &check_result.diagnostics {
        print_diagnostic(diag);
    }

    println!("{}", to_json(&check_result));

    let has_errors = !check_result.errors.is_empty()
        || check_result
            .diagnostics
            .iter()
            .any(|d| d.severity == patchlang::drc::Severity::Error);
    if has_errors {
        process::exit(1);
    }
}

fn cmd_compile(manifest_path: String) {
    let manifest_json = read_file(&manifest_path);
    let manifest_result = patchlang::parse_manifest(&manifest_json);

    if !manifest_result.errors.is_empty() {
        for err in &manifest_result.errors {
            eprintln!("error: {err}");
        }
        process::exit(1);
    }

    let manifest = manifest_result.manifest.unwrap();
    let project_dir = std::path::Path::new(&manifest_path)
        .parent()
        .unwrap_or(std::path::Path::new("."));

    let files = discover_project_files(&manifest, project_dir);

    let result = patchlang::compile_project(files, &manifest.root);

    for err in &result.errors {
        eprintln!("error: {}", err.message);
    }
    for diag in &result.diagnostics {
        print_diagnostic(diag);
    }

    println!("{}", to_json(&result));

    let has_errors = !result.errors.is_empty()
        || result
            .diagnostics
            .iter()
            .any(|d| d.severity == patchlang::drc::Severity::Error);
    if has_errors {
        process::exit(1);
    }
}

fn cmd_validate_layout(file: String) {
    let json = read_file(&file);
    let result = patchlang::validate_layout(&json);
    println!("{result}");

    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&result) {
        if v["valid"] == false {
            process::exit(1);
        }
    }
}

fn cmd_validate_consistency(patch_path: String, layout_path: String) {
    let patch = read_file(&patch_path);
    let layout = read_file(&layout_path);
    let result = patchlang::validate_project_consistency(&patch, &layout);
    println!("{result}");

    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&result) {
        if v["valid"] == false {
            process::exit(1);
        }
    }
}

// ── Shared formatting ───────────────────────────────────────────────

/// Print a DRC diagnostic to stderr.
fn print_diagnostic(diag: &patchlang::drc::Diagnostic) {
    let prefix = match diag.severity {
        patchlang::drc::Severity::Error => "error",
        patchlang::drc::Severity::Warning => "warning",
        patchlang::drc::Severity::Info => "info",
    };
    eprintln!("{prefix}[{:?}]: {}", diag.layer, diag.message);
    if let Some(fix) = &diag.fix {
        eprintln!("  fix: {fix}");
    }
}

/// BFS-discover all source files reachable from the manifest root.
fn discover_project_files(
    manifest: &patchlang::ProjectManifest,
    project_dir: &std::path::Path,
) -> HashMap<String, String> {
    let mut files = HashMap::new();
    let mut queue = VecDeque::new();
    let mut visited = HashSet::new();

    // Load root file
    let root_path = project_dir.join(&manifest.root);
    let root_source = read_file(root_path.to_str().unwrap_or(&manifest.root));
    files.insert(manifest.root.clone(), root_source);
    queue.push_back(manifest.root.clone());
    visited.insert(manifest.root.clone());

    // Load explicit library files
    for lib_path in &manifest.libraries {
        let full_path = project_dir.join(lib_path);
        let source = read_file(full_path.to_str().unwrap_or(lib_path));
        files.insert(lib_path.clone(), source);
        if visited.insert(lib_path.clone()) {
            queue.push_back(lib_path.clone());
        }
    }

    // BFS through `use` declarations
    while let Some(file_key) = queue.pop_front() {
        let source = match files.get(&file_key) {
            Some(s) => s.clone(),
            None => continue,
        };
        let deps = patchlang::resolve_uses(&source);
        for ns in deps {
            let dep_path = format!("{}.patch", ns.replace('.', "/"));
            if visited.insert(dep_path.clone()) {
                let full_path = project_dir.join(&dep_path);
                match std::fs::read_to_string(&full_path) {
                    Ok(dep_source) => {
                        files.insert(dep_path.clone(), dep_source);
                        queue.push_back(dep_path);
                    }
                    Err(e) => {
                        eprintln!(
                            "warning: cannot read '{}': {e}",
                            full_path.display()
                        );
                    }
                }
            }
        }
    }

    files
}
