use std::io::Read;
use std::process;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let source = if args.len() > 1 {
        std::fs::read_to_string(&args[1]).unwrap_or_else(|e| {
            eprintln!("error: cannot read '{}': {e}", args[1]);
            process::exit(1);
        })
    } else {
        let mut buf = String::new();
        std::io::stdin().read_to_string(&mut buf).unwrap_or_else(|e| {
            eprintln!("error: cannot read stdin: {e}");
            process::exit(1);
        });
        buf
    };

    let result = patchlang::parse(&source);

    if result.is_valid() {
        println!("{}", serde_json::to_string_pretty(&result.program).unwrap_or_else(|e| {
            eprintln!("error: failed to serialize AST: {e}");
            process::exit(2);
        }));
    } else {
        for err in &result.errors {
            let (line, col) = patchlang::error::line_col(&source, err.span.start);
            eprintln!("error[{line}:{col}]: {}", err.message);
            if let Some(hint) = &err.hint {
                eprintln!("  hint: {hint}");
            }
        }
        // Still output the partial AST to stdout
        println!("{}", serde_json::to_string_pretty(&result.program).unwrap_or_else(|e| {
            eprintln!("error: failed to serialize AST: {e}");
            process::exit(2);
        }));
        process::exit(1);
    }
}
