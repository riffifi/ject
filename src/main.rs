mod lexer;
mod parser;
mod ast;
mod value;
mod interpreter;
mod stdlib;
mod numpy;
mod gui;
mod error;
mod linter;
mod diagnostic;

use lexer::Lexer;
use parser::Parser;
use interpreter::{Interpreter, get_runtime_suggestion};
use diagnostic::DiagnosticRenderer;
use std::{env, fs};
use std::path::Path;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

#[derive(Clone, Copy)]
enum ExecutionMode {
    /// Parse, lint, and run the interpreter.
    Run,
    /// Parse and lint only (for editors / CI — never executes user code).
    CheckOnly,
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() {
        run_repl();
        return;
    }

    match args[0].as_str() {
        "--help" | "-h" => {
            print_help();
        }
        "--version" | "-V" => {
            println!("ject {}", env!("CARGO_PKG_VERSION"));
        }
        "--introspect" => {
            println!("{}", stdlib::introspect_native_kernel_json());
        }
        "--check" => {
            let files: Vec<&String> = args.iter().skip(1).collect();
            if files.is_empty() {
                eprintln!("error: --check requires at least one .ject file");
                std::process::exit(2);
            }
            for path in files {
                check_file(path);
            }
        }
        "--test" => {
            let files: Vec<&String> = args.iter().skip(1).collect();
            if files.is_empty() {
                eprintln!("Usage: ject --test <file.ject> [file2.ject ...]");
                std::process::exit(2);
            }
            for path in files {
                run_file(path);
            }
        }
        flag if flag.starts_with('-') => {
            eprintln!("Unknown option: {}. Try --help.", flag);
            std::process::exit(2);
        }
        _ => {
            // Legacy: `ject script.ject`
            run_file(&args[0]);
        }
    }
}

fn print_help() {
    println!(
        "\
Ject language interpreter

USAGE:
    ject                      Start REPL
    ject <file.ject>          Run a script
    ject --check <file> [...] Parse + lint only (no execution)
    ject --test <file> [...]  Run script(s); exit non-zero on failure
    ject --introspect         Print native kernel metadata (JSON)
    ject --help               Show this message"
    );
}

fn run_file(filename: &str) {
    match fs::read_to_string(filename) {
        Ok(source) => {
            let mut interpreter = Interpreter::new();
            if let Some(dir) = Path::new(filename).parent() {
                if !dir.as_os_str().is_empty() {
                    interpreter.set_script_dir(dir.to_path_buf());
                }
            }
            execute_source(
                &source,
                &mut interpreter,
                Some(filename.to_string()),
                ExecutionMode::Run,
            );
        }
        Err(error) => {
            eprintln!("Error reading file '{}': {}", filename, error);
            std::process::exit(1);
        }
    }
}

fn check_file(filename: &str) {
    match fs::read_to_string(filename) {
        Ok(source) => {
            let mut interpreter = Interpreter::new();
            if let Some(dir) = Path::new(filename).parent() {
                if !dir.as_os_str().is_empty() {
                    interpreter.set_script_dir(dir.to_path_buf());
                }
            }
            execute_source(
                &source,
                &mut interpreter,
                Some(filename.to_string()),
                ExecutionMode::CheckOnly,
            );
        }
        Err(error) => {
            eprintln!("Error reading file '{}': {}", filename, error);
            std::process::exit(1);
        }
    }
}

fn run_repl() {
    println!("Ject REPL - version {}", env!("CARGO_PKG_VERSION"));
    println!("Use arrow keys to access history");
    println!("'exit' to, well, exit\n");

    let mut interpreter = Interpreter::new();
    let mut linter = linter::Linter::new(); // Persistent linter for REPL
    let mut rl = DefaultEditor::new().expect("Failed to create readline editor");

    // Try to load history from file
    let _ = rl.load_history(".ject_history");

    loop {
        match rl.readline(">> ") {
            Ok(line) => {
                let input = line.trim();
                if input == "exit" {
                    println!("Goodbye!");
                    break;
                }
                if !input.is_empty() {
                    let _ = rl.add_history_entry(input);
                    execute_source_repl(input, &mut interpreter, &mut linter);
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("\n^C");
                continue; // Continue instead of break to allow Ctrl+C to cancel current line
            }
            Err(ReadlineError::Eof) => {
                println!("\n^D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }

    // Save history to file
    let _ = rl.save_history(".ject_history");
}

fn execute_source(
    source: &str,
    interpreter: &mut Interpreter,
    filename: Option<String>,
    mode: ExecutionMode,
) {
    let mut lexer = Lexer::new(source);
    let located_tokens = lexer.tokenize_with_positions();
    let positioned_tokens: Vec<(lexer::Token, lexer::SourcePosition)> =
        located_tokens.into_iter().map(|lt| (lt.token, lt.position)).collect();

    // Clone positioned tokens for linter before parser consumes them
    let positioned_tokens_for_linter = positioned_tokens.clone();
    let mut parser = Parser::new(positioned_tokens);

    match parser.parse() {
        Ok(statements) => {
            // Run linter to detect errors and warnings
            let mut linter = linter::Linter::new()
                .with_tokens_and_source(positioned_tokens_for_linter, source.to_string());
            let (diagnostics, has_errors) = linter.lint(&statements);

            // Create diagnostic renderer for beautiful output
            let renderer = DiagnosticRenderer::new();

            // Display all diagnostics with colorful formatting
            for diagnostic in &diagnostics {
                renderer.render(diagnostic, filename.as_deref(), Some(source));
            }

            // Only run interpreter if no errors were found
            if !has_errors {
                match mode {
                    ExecutionMode::CheckOnly => {}
                    ExecutionMode::Run => match interpreter.interpret(&statements) {
                        Ok(_) => {}
                        Err(error) => {
                            // Display runtime error with colors
                            use colored::*;
                            eprintln!(
                                "{}: {}",
                                "Runtime Error".red().bold(),
                                error.message.bold()
                            );

                            // Display suggestion if available
                            let suggestion = get_runtime_suggestion(&error.message);
                            if !suggestion.is_empty() {
                                eprintln!("{} {}", "Tip:".blue().bold(), suggestion.trim().bold());
                            }
                            if filename.is_some() {
                                std::process::exit(1);
                            }
                        }
                    },
                }
            } else {
                // Exit with error code when running files with linter errors
                if filename.is_some() {
                    std::process::exit(1);
                }
            }
        }
        Err(error) => {
            // Create diagnostic renderer for parse errors
            let renderer = DiagnosticRenderer::new();
            let mut parse_diagnostic =
                crate::diagnostic::Diagnostic::error(error.message.clone()).with_code("E0002".to_string());

            // Use position information if available
            if let (Some(line), Some(column)) = (error.line, error.column) {
                parse_diagnostic = parse_diagnostic.with_location(line, column);
            }

            renderer.render(&parse_diagnostic, filename.as_deref(), Some(source));

            if filename.is_some() {
                std::process::exit(1);
            }
        }
    }
}

fn execute_source_repl(source: &str, interpreter: &mut Interpreter, linter: &mut linter::Linter) {
    let mut lexer = Lexer::new(source);
    let located_tokens = lexer.tokenize_with_positions();
    let positioned_tokens: Vec<(lexer::Token, lexer::SourcePosition)> =
        located_tokens.into_iter().map(|lt| (lt.token, lt.position)).collect();

    // Clone positioned tokens for linter before parser consumes them
    let positioned_tokens_for_linter = positioned_tokens.clone();
    let mut parser = Parser::new(positioned_tokens);

    match parser.parse() {
        Ok(statements) => {
            // Use REPL-aware linter that maintains state
            *linter = linter
                .clone()
                .with_tokens_and_source(positioned_tokens_for_linter, source.to_string());
            let (diagnostics, has_errors) = linter.lint_repl(&statements);

            // Create diagnostic renderer for beautiful output
            let renderer = DiagnosticRenderer::new();

            // In REPL mode, only show errors (not warnings) to reduce noise
            // Warnings are still collected but not displayed unless verbose mode
            for diagnostic in &diagnostics {
                if diagnostic.level == crate::diagnostic::DiagnosticLevel::Error {
                    renderer.render(diagnostic, None, Some(source));
                }
            }

            // Only run interpreter if no errors were found
            if !has_errors {
                match interpreter.interpret(&statements) {
                    Ok(_) => {}
                    Err(error) => {
                        // Display runtime error with colors
                        use colored::*;
                        eprintln!(
                            "{}: {}",
                            "Runtime Error".red().bold(),
                            error.message.bold()
                        );

                        // Display suggestion if available
                        let suggestion = get_runtime_suggestion(&error.message);
                        if !suggestion.is_empty() {
                            eprintln!("{} {}", "Tip:".blue().bold(), suggestion.trim().bold());
                        }
                    }
                }
            }
        }
        Err(error) => {
            // Create diagnostic renderer for parse errors
            let renderer = DiagnosticRenderer::new();
            let mut parse_diagnostic =
                crate::diagnostic::Diagnostic::error(error.message.clone()).with_code("E0002".to_string());

            // Use position information if available
            if let (Some(line), Some(column)) = (error.line, error.column) {
                parse_diagnostic = parse_diagnostic.with_location(line, column);
            }

            renderer.render(&parse_diagnostic, None, Some(source));
        }
    }
}
