mod lexer;
mod parser;
mod ast;
mod value;
mod interpreter;

use lexer::Lexer;
use parser::Parser;
use interpreter::Interpreter;
use std::{env, fs};
use std::io::{self, Write};

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
            eprintln!("❌ Error reading file '{}': {}", filename, error);
            std::process::exit(1);
        }
    }
}

fn run_repl() {
    println!("🎨 Welcome to Ject - The Elegant Scripting Language");
    println!("Type 'exit' to quit, or start writing beautiful code!");
    println!("💡 Tip: You can also run files with 'ject filename.ject'\n");
    
    let mut interpreter = Interpreter::new();
    
    loop {
        print!("ject› ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        if input.trim() == "exit" {
            println!("✨ Thanks for using Ject! Have a beautiful day!");
            break;
        }
        
        if input.trim().is_empty() {
            continue;
        }

        execute_source(&input, &mut interpreter);
    }
}

fn execute_source(source: &str, interpreter: &mut Interpreter) {
    let tokens = Lexer::new(source).tokenize();
    let mut parser = Parser::new(tokens);

    match parser.parse() {
        Ok(statements) => {
            match interpreter.interpret(&statements) {
                Ok(_) => {}
                Err(error) => println!("❌ Runtime Error: {}", error),
            }
        }
        Err(error) => println!("❌ Parse Error: {}", error),
    }
}
