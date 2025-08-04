mod lexer;
mod parser;
mod ast;
mod value;
mod interpreter;
mod stdlib;

use lexer::Lexer;
use parser::Parser;
use interpreter::Interpreter;
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
            execute_source(&source, &mut interpreter);
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
                    execute_source(input, &mut interpreter);
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

fn execute_source(source: &str, interpreter: &mut Interpreter) {
    let tokens = Lexer::new(source).tokenize();
    let mut parser = Parser::new(tokens);

    match parser.parse() {
        Ok(statements) => {
            match interpreter.interpret(&statements) {
                Ok(_) => {}
                Err(error) => println!("Runtime Error: {}", error),
            }
        }
        Err(error) => println!("Parse Error: {}", error),
    }
}
