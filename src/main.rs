mod lexer;
mod parser;
mod ast;
mod value;
mod interpreter;
mod stdlib;
mod error;
mod linter;
mod diagnostic;

use lexer::Lexer;
use parser::Parser;
use interpreter::Interpreter;
use diagnostic::DiagnosticRenderer;
use std::{env, fs};
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() > 1 {
        // File execution mode
        let filename = &args[1];
        run_file(filename);
    } else {
        // Interactive REPL mode
        run_repl();
    }
}

fn run_file(filename: &str) {
    match fs::read_to_string(filename) {
        Ok(source) => {
            let mut interpreter = Interpreter::new();
            execute_source(&source, &mut interpreter, Some(filename.to_string()));
        }
        Err(error) => {
            eprintln!("Error reading file '{}': {}", filename, error);
            std::process::exit(1);
        }
    }
}

fn run_repl() {
    println!("Ject REPL - v0.1.0");
    println!("Use arrow keys to access history");
    println!("'exit' to, well, exit\n");
    
    let mut interpreter = Interpreter::new();
    let mut rl = DefaultEditor::new().expect("Failed to create readline editor");

    // Try to load history from file
    let _ = rl.load_history(".ject_history");

    loop {
        match rl.readline("ject› ") {
            Ok(line) => {
                let input = line.trim();
                if input == "exit" {
                    println!("Goodbye!");
                    break;
                }
                if !input.is_empty() {
                    let _ = rl.add_history_entry(input);
                    execute_source(input, &mut interpreter, None);
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

fn execute_source(source: &str, interpreter: &mut Interpreter, filename: Option<String>) {
    let mut lexer = Lexer::new(source);
    let located_tokens = lexer.tokenize_with_positions();
    let tokens: Vec<lexer::Token> = located_tokens.into_iter().map(|lt| lt.token).collect();
    let mut parser = Parser::new(tokens);

    match parser.parse() {
        Ok(statements) => {
            // Run linter to detect errors and warnings
            let mut linter = linter::Linter::new();
            let (diagnostics, has_errors) = linter.lint(&statements);
            
            // Create diagnostic renderer for beautiful output
            let renderer = DiagnosticRenderer::new();
            
            // Display all diagnostics with colorful formatting
            for diagnostic in &diagnostics {
                renderer.render(diagnostic, filename.as_deref(), Some(source));
            }
            
            // Only run interpreter if no errors were found
            if !has_errors {
                match interpreter.interpret(&statements) {
                    Ok(_) => {}
                    Err(error) => {
                        println!("Runtime Error: {}", error);
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
            let parse_diagnostic = crate::diagnostic::Diagnostic::error(error.message.clone())
                .with_code("E0002".to_string());
            renderer.render(&parse_diagnostic, filename.as_deref(), Some(source));
            
            if filename.is_some() {
                std::process::exit(1);
            }
        },
    }
}
