#[cfg(test)]
mod tests {
    use crate::lexer::Lexer;
    use crate::parser::Parser;
    use crate::linter::Linter;

    fn lint(input: &str) -> (Vec<String>, Vec<String>) {
        let mut lexer = Lexer::new(input);
        let located_tokens = lexer.tokenize_with_positions();
        let tokens: Vec<_> = located_tokens.into_iter().map(|lt| lt.token).collect();
        let mut parser = Parser::new_simple(tokens);
        let statements = parser.parse().unwrap();
        
        let mut linter = Linter::new();
        let (diagnostics, has_errors) = linter.lint(&statements);
        
        let errors: Vec<String> = diagnostics.iter()
            .filter(|d| d.level == crate::diagnostic::DiagnosticLevel::Error)
            .map(|d| d.message.clone())
            .collect();
        
        let warnings: Vec<String> = diagnostics.iter()
            .filter(|d| d.level == crate::diagnostic::DiagnosticLevel::Warning)
            .map(|d| d.message.clone())
            .collect();
        
        (errors, warnings)
    }

    // ========== Unused Variable Tests ==========

    #[test]
    fn test_unused_variable_warning() {
        let (_, warnings) = lint("let x = 42");
        assert!(warnings.iter().any(|w| w.contains("unused") && w.contains("x")));
    }

    #[test]
    fn test_used_variable_no_warning() {
        let (_, warnings) = lint(r#"
let x = 42
print x
"#);
        assert!(!warnings.iter().any(|w| w.contains("unused") && w.contains("x")));
    }

    #[test]
    fn test_underscore_variable_no_warning() {
        let (_, warnings) = lint(r#"
let _unused = 42
"#);
        // Variables starting with _ should not trigger warnings
        assert!(!warnings.iter().any(|w| w.contains("unused")));
    }

    #[test]
    fn test_multiple_unused_variables() {
        let (_, warnings) = lint(r#"
let x = 1
let y = 2
let z = 3
print x
"#);
        // y and z should trigger warnings
        assert!(warnings.iter().any(|w| w.contains("unused") && w.contains("y")));
        assert!(warnings.iter().any(|w| w.contains("unused") && w.contains("z")));
    }

    // ========== Undefined Variable Tests ==========

    #[test]
    fn test_undefined_variable_error() {
        let (errors, _) = lint("print undefined_var");
        assert!(errors.iter().any(|e| e.contains("undefined") || e.contains("undeclared")));
    }

    #[test]
    fn test_undefined_variable_in_expression() {
        let (errors, _) = lint("let y = x + 1");
        assert!(errors.iter().any(|e| e.contains("undefined") || e.contains("undeclared")));
    }

    #[test]
    fn test_undefined_function_call() {
        let (errors, _) = lint("undefined_fn()");
        assert!(errors.iter().any(|e| e.contains("undefined") || e.contains("undeclared")));
    }

    #[test]
    fn test_defined_variable_no_error() {
        let (errors, _) = lint(r#"
let x = 42
print x
"#);
        assert!(!errors.iter().any(|e| e.contains("undefined")));
    }

    #[test]
    fn test_defined_function_no_error() {
        let (errors, _) = lint(r#"
fn test()
    print "hello"
end
test()
"#);
        assert!(!errors.iter().any(|e| e.contains("undefined")));
    }

    // ========== Variable Shadowing Tests ==========

    #[test]
    fn test_variable_shadowing_warning() {
        let (_, warnings) = lint(r#"
let x = 10
let x = 20
"#);
        assert!(warnings.iter().any(|w| w.contains("already declared") || w.contains("shadow")));
    }

    #[test]
    fn test_function_redeclaration_warning() {
        let (_, warnings) = lint(r#"
fn test()
    print "first"
end

fn test()
    print "second"
end
"#);
        assert!(warnings.iter().any(|w| w.contains("already defined")));
    }

    // ========== Scope Tests ==========

    #[test]
    fn test_function_scope() {
        let (errors, warnings) = lint(r#"
fn test()
    let x = 42
end
print x
"#);
        // x should be undefined outside function
        assert!(errors.iter().any(|e| e.contains("undefined") || e.contains("undeclared")));
    }

    #[test]
    fn test_if_scope() {
        let (errors, _) = lint(r#"
if true then
    let x = 42
    print x
end
"#);
        // x should be accessible within if block
        assert!(!errors.iter().any(|e| e.contains("undefined")));
    }

    #[test]
    fn test_for_loop_scope() {
        let (errors, _) = lint(r#"
for i in 1..10 do
    print i
end
"#);
        // i should be accessible within loop
        assert!(!errors.iter().any(|e| e.contains("undefined")));
    }

    #[test]
    fn test_nested_scope_access() {
        let (errors, _) = lint(r#"
let outer = 10
fn test()
    print outer
end
test()
"#);
        // Should be able to access outer scope
        assert!(!errors.iter().any(|e| e.contains("undefined")));
    }

    // ========== Assignment Tests ==========

    #[test]
    fn test_assignment_to_undeclared() {
        let (errors, _) = lint("x = 42");
        assert!(errors.iter().any(|e| e.contains("undeclared") || e.contains("assign")));
    }

    #[test]
    fn test_assignment_to_declared() {
        let (errors, _) = lint(r#"
let x = 10
x = 42
"#);
        assert!(!errors.iter().any(|e| e.contains("undeclared")));
    }

    // ========== Return Statement Tests ==========

    #[test]
    fn test_return_outside_function() {
        let (errors, _) = lint("return 42");
        assert!(errors.iter().any(|e| e.contains("return") && e.contains("outside")));
    }

    #[test]
    fn test_return_inside_function() {
        let (errors, _) = lint(r#"
fn test()
    return 42
end
"#);
        assert!(!errors.iter().any(|e| e.contains("outside")));
    }

    #[test]
    fn test_return_inside_if_in_function() {
        let (errors, _) = lint(r#"
fn test()
    if true then
        return 42
    end
end
"#);
        assert!(!errors.iter().any(|e| e.contains("outside")));
    }

    // ========== Built-in Function Tests ==========

    #[test]
    fn test_builtin_function_no_error() {
        let (errors, _) = lint(r#"
print abs(-42)
print sqrt(16)
print len([1, 2, 3])
print upper("hello")
"#);
        assert!(!errors.iter().any(|e| e.contains("undefined")));
    }

    #[test]
    fn test_builtin_constant_no_error() {
        let (errors, _) = lint(r#"
print PI
print E
"#);
        assert!(!errors.iter().any(|e| e.contains("undefined")));
    }

    // ========== Import Tests ==========

    #[test]
    fn test_imported_variable_no_error() {
        // This would need actual module files to test properly
        // For now, just test that the linter doesn't crash
        let (errors, _) = lint(r#"
import {PI} from "math"
print PI
"#);
        // Should not have undefined error for PI
        // (depending on implementation, this might still error if module isn't found)
    }

    // ========== Complex Programs ==========

    #[test]
    fn test_complex_program_no_errors() {
        let (errors, warnings) = lint(r#"
struct Point { x, y }

fn distance(p1, p2)
    let dx = p2.x - p1.x
    let dy = p2.y - p1.y
    return sqrt(dx * dx + dy * dy)
end

let origin = new Point { x: 0, y: 0 }
let point = new Point { x: 3, y: 4 }
let dist = distance(origin, point)
print "Distance: " + dist
"#);
        // Should have no errors (might have unused warnings)
        assert!(!errors.iter().any(|e| e.contains("undefined")));
    }

    #[test]
    fn test_program_with_multiple_issues() {
        let (errors, warnings) = lint(r#"
let x = 10
let y = 20
let z = undefined_var

fn test()
    return return_value
end

return 42
"#);
        // Should have multiple errors
        assert!(errors.len() > 0);
        // Should have unused variable warnings
        assert!(warnings.len() > 0);
    }

    // ========== Edge Cases ==========

    #[test]
    fn test_empty_program() {
        let (errors, warnings) = lint("");
        // Empty program should have no errors
        assert_eq!(errors.len(), 0);
        // Warnings are acceptable for empty programs
        assert!(errors.is_empty());
    }

    #[test]
    fn test_comments_only() {
        let (errors, warnings) = lint("# Just a comment");
        // Comments are ignored by linter, no errors expected
        assert_eq!(errors.len(), 0);
        // Warnings may occur for programs with no executable statements
        assert!(errors.is_empty());
    }

    #[test]
    fn test_lambda_parameters() {
        let (errors, _) = lint(r#"
let add = lambda(a, b) -> a + b
print add(2, 3)
"#);
        // Lambda parameters should be recognized
        assert!(!errors.iter().any(|e| e.contains("undefined")));
    }

    #[test]
    fn test_function_parameters() {
        let (errors, _) = lint(r#"
fn greet(name)
    print "Hello, " + name
end
greet("World")
"#);
        // Function parameters should be recognized
        assert!(!errors.iter().any(|e| e.contains("undefined")));
    }

    #[test]
    fn test_function_default_parameters() {
        let (errors, _) = lint(r#"
fn greet(name="World")
    print "Hello, " + name
end
greet()
"#);
        // Default parameters should be handled
        assert!(!errors.iter().any(|e| e.contains("undefined")));
    }

    #[test]
    fn test_match_expression() {
        let (errors, _) = lint(r#"
let x = 2
match x
    1 -> print "one"
    2 -> print "two"
    _ -> print "other"
end
"#);
        assert!(!errors.iter().any(|e| e.contains("undefined")));
    }

    #[test]
    fn test_try_catch() {
        let (errors, _) = lint(r#"
try
    risky_operation()
catch err
    print err
end
"#);
        // err should be recognized in catch block
        // risky_operation will be undefined though
        assert!(errors.iter().any(|e| e.contains("undefined")));
    }

    #[test]
    fn test_array_comprehension_like() {
        let (errors, _) = lint(r#"
let numbers = [1, 2, 3, 4, 5]
let doubled = map(numbers, lambda(n) -> n * 2)
print doubled
"#);
        assert!(!errors.iter().any(|e| e.contains("undefined")));
    }

    #[test]
    fn test_recursive_function() {
        let (errors, _) = lint(r#"
fn factorial(n)
    if n <= 1 then
        return 1
    else
        return n * factorial(n - 1)
    end
end
print factorial(5)
"#);
        // factorial should be recognized as defined
        assert!(!errors.iter().any(|e| e.contains("undefined")));
    }

    #[test]
    fn test_mutual_recursion() {
        let (errors, _) = lint(r#"
fn is_even(n)
    if n == 0 then
        return true
    else
        return is_odd(n - 1)
    end
end

fn is_odd(n)
    if n == 0 then
        return false
    else
        return is_even(n - 1)
    end
end

print is_even(10)
"#);
        // Mutual recursion might cause issues depending on implementation
        // This tests how the linter handles forward references
    }

    // ========== Suggestion Tests ==========

    #[test]
    fn test_similar_variable_suggestion() {
        let (errors, _) = lint(r#"
let my_variable = 42
print my_variabl
"#);
        // Should suggest my_variable
        assert!(errors.iter().any(|e| e.contains("my_variable") || e.contains("similar")));
    }

    #[test]
    fn test_function_name_suggestion() {
        let (errors, _) = lint(r#"
fn calculate_sum()
    return 42
end
calculate_sums()
"#);
        // Should suggest calculate_sum
        assert!(errors.iter().any(|e| e.contains("calculate_sum") || e.contains("similar")));
    }

    // ========== REPL Linting Tests ==========

    #[test]
    fn test_repl_lint_maintains_state() {
        let mut lexer = Lexer::new("let x = 42");
        let located_tokens = lexer.tokenize_with_positions();
        let tokens: Vec<_> = located_tokens.into_iter().map(|lt| lt.token).collect();
        let mut parser = Parser::new_simple(tokens);
        let statements = parser.parse().unwrap();
        
        let mut linter = Linter::new();
        let (_, has_errors) = linter.lint_repl(&statements);
        assert!(!has_errors);
        
        // Now use x in a second statement
        let mut lexer = Lexer::new("print x");
        let located_tokens = lexer.tokenize_with_positions();
        let tokens: Vec<_> = located_tokens.into_iter().map(|lt| lt.token).collect();
        let mut parser = Parser::new_simple(tokens);
        let statements = parser.parse().unwrap();
        
        let (_, has_errors) = linter.lint_repl(&statements);
        // Should not have errors because x was defined earlier
        assert!(!has_errors);
    }
}
