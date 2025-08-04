use std::io::Write;
use ject::*;

#[test]
fn test_basic_lambda() {
    let input = r#"
        let add = lambda(x, y) -> x + y
        add(5, 3)
    "#;
    
    let mut lexer = lexer::Lexer::new(input);
    let tokens = lexer.tokenize();
    let mut parser = parser::Parser::new(tokens);
    let statements = parser.parse().unwrap();
    let mut interpreter = interpreter::Interpreter::new();
    
    // This should execute without error
    assert!(interpreter.interpret(&statements).is_ok());
}

#[test]
fn test_lambda_with_no_params() {
    let input = r#"
        let get_answer = lambda() -> 42
        get_answer()
    "#;
    
    let mut lexer = lexer::Lexer::new(input);
    let tokens = lexer.tokenize();
    let mut parser = parser::Parser::new(tokens);
    let statements = parser.parse().unwrap();
    let mut interpreter = interpreter::Interpreter::new();
    
    assert!(interpreter.interpret(&statements).is_ok());
}

#[test]
fn test_lambda_with_single_param() {
    let input = r#"
        let square = lambda(n) -> n * n
        square(4)
    "#;
    
    let mut lexer = lexer::Lexer::new(input);
    let tokens = lexer.tokenize();
    let mut parser = parser::Parser::new(tokens);
    let statements = parser.parse().unwrap();
    let mut interpreter = interpreter::Interpreter::new();
    
    assert!(interpreter.interpret(&statements).is_ok());
}

#[test]
fn test_lambda_with_string_operations() {
    let input = r#"
        let greet = lambda(name) -> "Hello, " + name
        greet("World")
    "#;
    
    let mut lexer = lexer::Lexer::new(input);
    let tokens = lexer.tokenize();
    let mut parser = parser::Parser::new(tokens);
    let statements = parser.parse().unwrap();
    let mut interpreter = interpreter::Interpreter::new();
    
    assert!(interpreter.interpret(&statements).is_ok());
}

#[test]
fn test_lambda_error_wrong_arg_count() {
    let input = r#"
        let add = lambda(x, y) -> x + y
        add(5)
    "#;
    
    let mut lexer = lexer::Lexer::new(input);
    let tokens = lexer.tokenize();
    let mut parser = parser::Parser::new(tokens);
    let statements = parser.parse().unwrap();
    let mut interpreter = interpreter::Interpreter::new();
    
    // This should fail with runtime error for wrong argument count
    assert!(interpreter.interpret(&statements).is_err());
}
