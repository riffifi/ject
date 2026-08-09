mod lexer;
mod parser;
mod ast;
mod value;
mod interpreter;
mod stdlib;
mod numpy;
mod gui;
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

/// Heuristically decides whether `source` looks like an incomplete statement/block
/// that should keep accumulating more input lines, rather than a genuine syntax
/// error to show immediately. Used by the REPL to support multi-line input the way
/// Python's REPL does -- typing `fn foo(x)` and pressing enter waits for a matching
/// `end` instead of erroring immediately.
fn input_seems_incomplete(source: &str) -> bool {
    let mut lexer = Lexer::new(source);
    let tokens: Vec<lexer::Token> = lexer
        .tokenize_with_positions()
        .into_iter()
        .map(|lt| lt.token)
        .collect();

    let mut paren_depth: i32 = 0;
    let mut bracket_depth: i32 = 0;
    let mut brace_depth: i32 = 0;
    let mut block_depth: i32 = 0;

    let mut i = 0;
    while i < tokens.len() {
        match &tokens[i] {
            lexer::Token::LeftParen => paren_depth += 1,
            lexer::Token::RightParen => paren_depth -= 1,
            lexer::Token::LeftBracket => bracket_depth += 1,
            lexer::Token::RightBracket => bracket_depth -= 1,
            lexer::Token::LeftBrace | lexer::Token::LeftBracePipe => brace_depth += 1,
            lexer::Token::RightBrace | lexer::Token::RightPipeBrace => brace_depth -= 1,
            lexer::Token::If
            | lexer::Token::While
            | lexer::Token::For
            | lexer::Token::Try
            | lexer::Token::Match => {
                block_depth += 1;
            }
            lexer::Token::Fn => {
                // `fn(...) -> expr` / `fn name(...) -> expr` need no matching `end`;
                // `fn(...) ... end` / `fn name(...) ... end` do. Scan ahead past this
                // header (skipping the optional name and the parameter list) to see
                // which form it is.
                let mut j = i + 1;
                let mut header_paren_depth = 0i32;
                let mut seen_paren = false;
                let mut incomplete_header = false;
                loop {
                    if j >= tokens.len() {
                        incomplete_header = true;
                        break;
                    }
                    match &tokens[j] {
                        lexer::Token::LeftParen => {
                            header_paren_depth += 1;
                            seen_paren = true;
                        }
                        lexer::Token::RightParen => {
                            header_paren_depth -= 1;
                            if seen_paren && header_paren_depth <= 0 {
                                j += 1;
                                break;
                            }
                        }
                        _ => {}
                    }
                    j += 1;
                }
                if incomplete_header {
                    return true;
                }
                if tokens.get(j) != Some(&lexer::Token::Arrow) {
                    block_depth += 1;
                }
                i = j;
                continue;
            }
            lexer::Token::End => {
                block_depth -= 1;
            }
            _ => {}
        }
        i += 1;
    }

    if paren_depth > 0 || bracket_depth > 0 || brace_depth > 0 || block_depth > 0 {
        return true;
    }

    // A trailing token that clearly expects something to follow it (e.g. `let x =`,
    // `1 +`, a dangling `,`) means there's more coming even though nothing's unbalanced.
    let last_meaningful = tokens
        .iter()
        .rev()
        .find(|t| !matches!(t, lexer::Token::Newline | lexer::Token::Eof));
    if let Some(t) = last_meaningful {
        use lexer::Token::*;
        if matches!(
            t,
            Plus | Minus | Star | Slash | Percent
                | Equal | EqualEqual | BangEqual
                | Less | Greater | LessEqual | GreaterEqual
                | And | Or | Comma | Arrow | Dot | DotDot
                | Let | Import | From | As | Colon
        ) {
            return true;
        }
    }

    false
}

fn run_repl() {
    println!("Ject REPL - version {}", env!("CARGO_PKG_VERSION"));
    println!("Use arrow keys to access history");
    println!("Ctrl+C cancels the current line, or interrupts a running script; Ctrl+D exits");
    println!("'exit' to, well, exit\n");

    let mut interpreter = Interpreter::new();
    let mut linter = linter::Linter::new(); // Persistent linter for REPL
    let mut rl = DefaultEditor::new().expect("Failed to create readline editor");

    // Try to load history from file
    let _ = rl.load_history(".ject_history");

    // Shared with a Ctrl+C handler below: lets a running script (e.g. an infinite
    // `while true do ... end`) be interrupted cleanly instead of only being able to
    // cancel line input that hasn't been submitted yet. If installing the handler
    // fails for any reason, the REPL still works exactly as before -- Ctrl+C just
    // won't be able to interrupt a script already running.
    let interrupt_flag = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    interpreter.set_interrupt_flag(interrupt_flag.clone());
    {
        let flag = interrupt_flag.clone();
        let _ = ctrlc::set_handler(move || {
            flag.store(true, std::sync::atomic::Ordering::Relaxed);
        });
    }

    // Accumulates lines while input looks incomplete (an unclosed `fn ... end`, an
    // open paren, a trailing `+`, etc.) -- mirrors Python's REPL waiting for more
    // input instead of erroring on the first line of a multi-line statement.
    let mut buffer = String::new();

    loop {
        let prompt = if buffer.is_empty() { ">> " } else { ".. " };
        match rl.readline(prompt) {
            Ok(line) => {
                if buffer.is_empty() {
                    let trimmed = line.trim();
                    if trimmed == "exit" {
                        println!("Goodbye!");
                        break;
                    }
                    if trimmed.is_empty() {
                        continue;
                    }
                }

                if !buffer.is_empty() {
                    buffer.push('\n');
                }
                buffer.push_str(&line);

                // An empty line while continuing a multi-line block means "run what
                // I've got" -- try executing even if the heuristic below would
                // otherwise still want more (it can't always tell an
                // intentionally-abandoned block from a genuinely unfinished one).
                if !line.trim().is_empty() && input_seems_incomplete(&buffer) {
                    continue;
                }

                let _ = rl.add_history_entry(buffer.trim());
                interrupt_flag.store(false, std::sync::atomic::Ordering::Relaxed);
                execute_source_repl(&buffer, &mut interpreter, &mut linter);
                interrupt_flag.store(false, std::sync::atomic::Ordering::Relaxed);
                buffer.clear();
            }
            Err(ReadlineError::Interrupted) => {
                println!("\n^C");
                buffer.clear(); // Cancel whatever multi-line input was being composed too.
                continue;
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
                            let mut runtime_diagnostic =
                                crate::diagnostic::Diagnostic::error(error.message.clone())
                                    .with_code("E0003".to_string());
                            let suggestion = get_runtime_suggestion(&error.message);
                            if !suggestion.is_empty() {
                                runtime_diagnostic = runtime_diagnostic.with_help(suggestion.trim().to_string());
                            }
                            renderer.render(&runtime_diagnostic, filename.as_deref(), Some(source));
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
                match interpreter.interpret_repl(&statements) {
                    Ok(Some(result)) if !matches!(result, value::Value::Nil) => {
                        println!("{}", result.display());
                    }
                    Ok(_) => {}
                    Err(error) if error.message == "Interrupted" => {
                        println!("\n^C");
                    }
                    Err(error) => {
                        let mut runtime_diagnostic =
                            crate::diagnostic::Diagnostic::error(error.message.clone())
                                .with_code("E0003".to_string());
                        let suggestion = get_runtime_suggestion(&error.message);
                        if !suggestion.is_empty() {
                            runtime_diagnostic = runtime_diagnostic.with_help(suggestion.trim().to_string());
                        }
                        let renderer = DiagnosticRenderer::new();
                        renderer.render(&runtime_diagnostic, None, Some(source));
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
