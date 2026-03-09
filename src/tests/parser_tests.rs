#[cfg(test)]
mod tests {
    use crate::lexer::Lexer;
    use crate::parser::Parser;
    use crate::ast::{Expr, Stmt, BinaryOp, UnaryOp, Pattern};

    fn parse(input: &str) -> Result<Vec<Stmt>, String> {
        let mut lexer = Lexer::new(input);
        let located_tokens = lexer.tokenize_with_positions();
        let tokens: Vec<_> = located_tokens.into_iter().map(|lt| lt.token).collect();
        let mut parser = Parser::new_simple(tokens);
        parser.parse().map_err(|e| e.message)
    }

    // ========== Basic Statement Tests ==========

    #[test]
    fn test_let_statement() {
        let stmts = parse("let x = 42").unwrap();
        assert_eq!(stmts.len(), 1);
        if let Stmt::Let { name, value } = &stmts[0] {
            assert_eq!(name, "x");
            if let Expr::Integer(n) = value {
                assert_eq!(n, &42);
            } else {
                panic!("Expected Integer");
            }
        } else {
            panic!("Expected Let statement");
        }
    }

    #[test]
    fn test_let_statement_with_string() {
        let stmts = parse("let name = \"Alice\"").unwrap();
        assert_eq!(stmts.len(), 1);
        if let Stmt::Let { name, value } = &stmts[0] {
            assert_eq!(name, "name");
            if let Expr::String(s) = value {
                assert_eq!(s, "Alice");
            } else {
                panic!("Expected String");
            }
        } else {
            panic!("Expected Let statement");
        }
    }

    #[test]
    fn test_let_statement_with_bool() {
        let stmts = parse("let is_true = true").unwrap();
        assert_eq!(stmts.len(), 1);
        if let Stmt::Let { value, .. } = &stmts[0] {
            if let Expr::Bool(b) = value {
                assert_eq!(b, &true);
            } else {
                panic!("Expected Bool");
            }
        } else {
            panic!("Expected Let statement");
        }

        let stmts = parse("let is_false = false").unwrap();
        assert_eq!(stmts.len(), 1);
        if let Stmt::Let { value, .. } = &stmts[0] {
            if let Expr::Bool(b) = value {
                assert_eq!(b, &false);
            } else {
                panic!("Expected Bool");
            }
        }
    }

    #[test]
    fn test_let_statement_with_nil() {
        let stmts = parse("let empty = nil").unwrap();
        assert_eq!(stmts.len(), 1);
        if let Stmt::Let { value, .. } = &stmts[0] {
            assert!(matches!(value, Expr::Nil));
        } else {
            panic!("Expected Let statement");
        }
    }

    #[test]
    fn test_assignment_statement() {
        let stmts = parse("x = 10").unwrap();
        assert_eq!(stmts.len(), 1);
        if let Stmt::Assign { target, value } = &stmts[0] {
            if let crate::ast::AssignTarget::Identifier(name) = target {
                assert_eq!(name, "x");
            } else {
                panic!("Expected Identifier target");
            }
            if let Expr::Integer(n) = value {
                assert_eq!(n, &10);
            } else {
                panic!("Expected Integer");
            }
        } else {
            panic!("Expected Assign statement");
        }
    }

    #[test]
    fn test_expression_statement() {
        let stmts = parse("42").unwrap();
        assert_eq!(stmts.len(), 1);
        if let Stmt::Expression(expr) = &stmts[0] {
            if let Expr::Integer(n) = expr {
                assert_eq!(n, &42);
            } else {
                panic!("Expected Integer");
            }
        } else {
            panic!("Expected Expression statement");
        }
    }

    #[test]
    fn test_print_statement() {
        let stmts = parse("print \"Hello\"").unwrap();
        assert_eq!(stmts.len(), 1);
        if let Stmt::Print(expr) = &stmts[0] {
            if let Expr::String(s) = expr {
                assert_eq!(s, "Hello");
            } else {
                panic!("Expected String");
            }
        } else {
            panic!("Expected Print statement");
        }
    }

    // ========== Arithmetic Expression Tests ==========

    #[test]
    fn test_addition() {
        let stmts = parse("1 + 2").unwrap();
        assert_eq!(stmts.len(), 1);
        if let Stmt::Expression(Expr::Binary { operator, .. }) = &stmts[0] {
            assert_eq!(operator, &BinaryOp::Add);
        } else {
            panic!("Expected Binary expression with Add");
        }
    }

    #[test]
    fn test_subtraction() {
        let stmts = parse("10 - 5").unwrap();
        assert_eq!(stmts.len(), 1);
        if let Stmt::Expression(Expr::Binary { operator, .. }) = &stmts[0] {
            assert_eq!(operator, &BinaryOp::Subtract);
        } else {
            panic!("Expected Binary expression with Subtract");
        }
    }

    #[test]
    fn test_multiplication() {
        let stmts = parse("3 * 4").unwrap();
        assert_eq!(stmts.len(), 1);
        if let Stmt::Expression(Expr::Binary { operator, .. }) = &stmts[0] {
            assert_eq!(operator, &BinaryOp::Multiply);
        } else {
            panic!("Expected Binary expression with Multiply");
        }
    }

    #[test]
    fn test_division() {
        let stmts = parse("20 / 4").unwrap();
        assert_eq!(stmts.len(), 1);
        if let Stmt::Expression(Expr::Binary { operator, .. }) = &stmts[0] {
            assert_eq!(operator, &BinaryOp::Divide);
        } else {
            panic!("Expected Binary expression with Divide");
        }
    }

    #[test]
    fn test_modulo() {
        let stmts = parse("17 % 5").unwrap();
        assert_eq!(stmts.len(), 1);
        if let Stmt::Expression(Expr::Binary { operator, .. }) = &stmts[0] {
            assert_eq!(operator, &BinaryOp::Modulo);
        } else {
            panic!("Expected Binary expression with Modulo");
        }
    }

    #[test]
    fn test_operator_precedence() {
        // 1 + 2 * 3 should be parsed as 1 + (2 * 3)
        let stmts = parse("1 + 2 * 3").unwrap();
        assert_eq!(stmts.len(), 1);
        if let Stmt::Expression(Expr::Binary { left, operator, right }) = &stmts[0] {
            assert_eq!(operator, &BinaryOp::Add);
            // Left should be 1
            if let Expr::Integer(n) = **left {
                assert_eq!(n, 1);
            } else {
                panic!("Expected left to be 1");
            }
            // Right should be 2 * 3
            if let Expr::Binary { left: inner_left, operator: inner_op, right: inner_right } = &**right {
                assert_eq!(inner_op, &BinaryOp::Multiply);
                if let Expr::Integer(n) = **inner_left {
                    assert_eq!(n, 2);
                }
                if let Expr::Integer(n) = **inner_right {
                    assert_eq!(n, 3);
                }
            } else {
                panic!("Expected right to be multiplication");
            }
        } else {
            panic!("Expected Binary expression");
        }
    }

    #[test]
    fn test_parentheses() {
        // (1 + 2) * 3
        let stmts = parse("(1 + 2) * 3").unwrap();
        assert_eq!(stmts.len(), 1);
        if let Stmt::Expression(Expr::Binary { left, operator, right }) = &stmts[0] {
            assert_eq!(operator, &BinaryOp::Multiply);
            // Left should be (1 + 2)
            if let Expr::Binary { operator: inner_op, .. } = &**left {
                assert_eq!(inner_op, &BinaryOp::Add);
            } else {
                panic!("Expected left to be addition");
            }
            // Right should be 3
            if let Expr::Integer(n) = **right {
                assert_eq!(n, 3);
            }
        } else {
            panic!("Expected Binary expression");
        }
    }

    #[test]
    fn test_unary_negation() {
        let stmts = parse("-42").unwrap();
        assert_eq!(stmts.len(), 1);
        if let Stmt::Expression(Expr::Unary { operator, .. }) = &stmts[0] {
            assert_eq!(operator, &UnaryOp::Negate);
        } else {
            panic!("Expected Unary expression");
        }
    }

    #[test]
    fn test_unary_not() {
        let stmts = parse("!true").unwrap();
        assert_eq!(stmts.len(), 1);
        if let Stmt::Expression(Expr::Unary { operator, .. }) = &stmts[0] {
            assert_eq!(operator, &UnaryOp::Not);
        } else {
            panic!("Expected Unary expression");
        }
    }

    #[test]
    fn test_double_negation() {
        let stmts = parse("- -42").unwrap();
        assert_eq!(stmts.len(), 1);
        if let Stmt::Expression(Expr::Unary { operator, operand }) = &stmts[0] {
            assert_eq!(operator, &UnaryOp::Negate);
            if let Expr::Unary { operator: inner_op, .. } = &**operand {
                assert_eq!(inner_op, &UnaryOp::Negate);
            } else {
                panic!("Expected nested Unary");
            }
        } else {
            panic!("Expected Unary expression");
        }
    }

    // ========== Comparison and Logical Tests ==========

    #[test]
    fn test_equality() {
        let stmts = parse("x == 5").unwrap();
        assert_eq!(stmts.len(), 1);
        if let Stmt::Expression(Expr::Binary { operator, .. }) = &stmts[0] {
            assert_eq!(operator, &BinaryOp::Equal);
        } else {
            panic!("Expected Binary expression with Equal");
        }
    }

    #[test]
    fn test_inequality() {
        let stmts = parse("x != 5").unwrap();
        assert_eq!(stmts.len(), 1);
        if let Stmt::Expression(Expr::Binary { operator, .. }) = &stmts[0] {
            assert_eq!(operator, &BinaryOp::NotEqual);
        } else {
            panic!("Expected Binary expression with NotEqual");
        }
    }

    #[test]
    fn test_less_than() {
        let stmts = parse("x < 5").unwrap();
        assert_eq!(stmts.len(), 1);
        if let Stmt::Expression(Expr::Binary { operator, .. }) = &stmts[0] {
            assert_eq!(operator, &BinaryOp::Less);
        } else {
            panic!("Expected Binary expression with Less");
        }
    }

    #[test]
    fn test_greater_than() {
        let stmts = parse("x > 5").unwrap();
        assert_eq!(stmts.len(), 1);
        if let Stmt::Expression(Expr::Binary { operator, .. }) = &stmts[0] {
            assert_eq!(operator, &BinaryOp::Greater);
        } else {
            panic!("Expected Binary expression with Greater");
        }
    }

    #[test]
    fn test_less_equal() {
        let stmts = parse("x <= 5").unwrap();
        assert_eq!(stmts.len(), 1);
        if let Stmt::Expression(Expr::Binary { operator, .. }) = &stmts[0] {
            assert_eq!(operator, &BinaryOp::LessEqual);
        } else {
            panic!("Expected Binary expression with LessEqual");
        }
    }

    #[test]
    fn test_greater_equal() {
        let stmts = parse("x >= 5").unwrap();
        assert_eq!(stmts.len(), 1);
        if let Stmt::Expression(Expr::Binary { operator, .. }) = &stmts[0] {
            assert_eq!(operator, &BinaryOp::GreaterEqual);
        } else {
            panic!("Expected Binary expression with GreaterEqual");
        }
    }

    #[test]
    fn test_and_operator() {
        let stmts = parse("true and false").unwrap();
        assert_eq!(stmts.len(), 1);
        if let Stmt::Expression(Expr::Binary { operator, .. }) = &stmts[0] {
            assert_eq!(operator, &BinaryOp::And);
        } else {
            panic!("Expected Binary expression with And");
        }
    }

    #[test]
    fn test_or_operator() {
        let stmts = parse("true or false").unwrap();
        assert_eq!(stmts.len(), 1);
        if let Stmt::Expression(Expr::Binary { operator, .. }) = &stmts[0] {
            assert_eq!(operator, &BinaryOp::Or);
        } else {
            panic!("Expected Binary expression with Or");
        }
    }

    #[test]
    fn test_in_operator() {
        let stmts = parse("x in arr").unwrap();
        assert_eq!(stmts.len(), 1);
        if let Stmt::Expression(Expr::Binary { operator, .. }) = &stmts[0] {
            assert_eq!(operator, &BinaryOp::In);
        } else {
            panic!("Expected Binary expression with In");
        }
    }

    // ========== Control Flow Tests ==========

    #[test]
    fn test_if_statement() {
        let stmts = parse("if x > 0 then\n    print \"positive\"\nend").unwrap();
        assert_eq!(stmts.len(), 1);
        if let Stmt::If { condition, then_branch, elseif_branches, else_branch } = &stmts[0] {
            assert!(matches!(condition, Expr::Binary { operator: BinaryOp::Greater, .. }));
            assert_eq!(then_branch.len(), 1);
            assert!(matches!(then_branch[0], Stmt::Print(_)));
            assert!(elseif_branches.is_empty());
            assert!(else_branch.is_none());
        } else {
            panic!("Expected If statement");
        }
    }

    #[test]
    fn test_if_else_statement() {
        let stmts = parse("if x > 0 then\n    print \"positive\"\nelse\n    print \"negative\"\nend").unwrap();
        assert_eq!(stmts.len(), 1);
        if let Stmt::If { condition, then_branch, elseif_branches, else_branch } = &stmts[0] {
            assert!(elseif_branches.is_empty());
            assert!(else_branch.is_some());
            let else_body = else_branch.as_ref().unwrap();
            assert_eq!(else_body.len(), 1);
            assert!(matches!(else_body[0], Stmt::Print(_)));
        } else {
            panic!("Expected If statement");
        }
    }

    #[test]
    fn test_if_elseif_else_statement() {
        let stmts = parse("if x > 0 then\n    print \"positive\"\nelseif x < 0 then\n    print \"negative\"\nelse\n    print \"zero\"\nend").unwrap();
        assert_eq!(stmts.len(), 1);
        if let Stmt::If { condition, then_branch, elseif_branches, else_branch } = &stmts[0] {
            assert_eq!(elseif_branches.len(), 1);
            assert!(else_branch.is_some());
        } else {
            panic!("Expected If statement");
        }
    }

    #[test]
    fn test_while_statement() {
        let stmts = parse("while x < 10 do\n    x = x + 1\nend").unwrap();
        assert_eq!(stmts.len(), 1);
        if let Stmt::While { condition, body } = &stmts[0] {
            assert!(matches!(condition, Expr::Binary { operator: BinaryOp::Less, .. }));
            assert_eq!(body.len(), 1);
            assert!(matches!(body[0], Stmt::Assign { .. }));
        } else {
            panic!("Expected While statement");
        }
    }

    #[test]
    fn test_for_statement() {
        let stmts = parse("for i in 1..10 do\n    print i\nend").unwrap();
        assert_eq!(stmts.len(), 1);
        if let Stmt::For { var, iterable, body } = &stmts[0] {
            assert_eq!(var, "i");
            assert!(matches!(iterable, Expr::Range { .. }));
            assert_eq!(body.len(), 1);
            assert!(matches!(body[0], Stmt::Print(_)));
        } else {
            panic!("Expected For statement");
        }
    }

    #[test]
    fn test_for_in_array() {
        let stmts = parse("for item in [1, 2, 3] do\n    print item\nend").unwrap();
        assert_eq!(stmts.len(), 1);
        if let Stmt::For { var, iterable, body } = &stmts[0] {
            assert_eq!(var, "item");
            assert!(matches!(iterable, Expr::Array(_)));
            assert_eq!(body.len(), 1);
        } else {
            panic!("Expected For statement");
        }
    }

    #[test]
    fn test_return_statement() {
        let stmts = parse("return 42").unwrap();
        assert_eq!(stmts.len(), 1);
        if let Stmt::Return(Some(expr)) = &stmts[0] {
            if let Expr::Integer(n) = expr {
                assert_eq!(n, &42);
            } else {
                panic!("Expected Integer");
            }
        } else {
            panic!("Expected Return statement");
        }
    }

    #[test]
    fn test_return_without_value() {
        let stmts = parse("return").unwrap();
        assert_eq!(stmts.len(), 1);
        if let Stmt::Return(None) = &stmts[0] {
            // Success
        } else {
            panic!("Expected Return statement without value");
        }
    }

    // ========== Function Tests ==========

    #[test]
    fn test_function_definition() {
        let stmts = parse("fn add(a, b)\n    return a + b\nend").unwrap();
        assert_eq!(stmts.len(), 1);
        if let Stmt::Function { name, params, body } = &stmts[0] {
            assert_eq!(name, "add");
            assert_eq!(params.len(), 2);
            assert_eq!(params[0].name, "a");
            assert_eq!(params[1].name, "b");
            assert_eq!(body.len(), 1);
            assert!(matches!(body[0], Stmt::Return(_)));
        } else {
            panic!("Expected Function statement");
        }
    }

    #[test]
    fn test_function_with_default_params() {
        let stmts = parse("fn greet(name=\"World\")\n    print \"Hello, \" + name\nend").unwrap();
        assert_eq!(stmts.len(), 1);
        if let Stmt::Function { params, .. } = &stmts[0] {
            assert_eq!(params.len(), 1);
            assert_eq!(params[0].name, "name");
            assert!(params[0].default_value.is_some());
        } else {
            panic!("Expected Function statement");
        }
    }

    #[test]
    fn test_function_call() {
        let stmts = parse("add(1, 2)").unwrap();
        assert_eq!(stmts.len(), 1);
        if let Stmt::Expression(Expr::Call { callee, args }) = &stmts[0] {
            if let Expr::Identifier(name) = &**callee {
                assert_eq!(name, "add");
            }
            assert_eq!(args.len(), 2);
        } else {
            panic!("Expected Call expression");
        }
    }

    #[test]
    fn test_function_call_with_keyword_args() {
        let stmts = parse("greet(name=\"Alice\")").unwrap();
        assert_eq!(stmts.len(), 1);
        if let Stmt::Expression(Expr::Call { args, .. }) = &stmts[0] {
            assert_eq!(args.len(), 1);
            if let crate::ast::Argument::Keyword { name, .. } = &args[0] {
                assert_eq!(name, "name");
            } else {
                panic!("Expected Keyword argument");
            }
        } else {
            panic!("Expected Call expression");
        }
    }

    // ========== Lambda Tests ==========

    #[test]
    fn test_lambda_expression() {
        let stmts = parse("lambda(x) -> x * x").unwrap();
        assert_eq!(stmts.len(), 1);
        if let Stmt::Expression(Expr::Lambda { params, body }) = &stmts[0] {
            assert_eq!(params.len(), 1);
            assert_eq!(params[0], "x");
            if let crate::ast::LambdaBody::Expression(expr) = body {
                assert!(matches!(&**expr, Expr::Binary { operator: BinaryOp::Multiply, .. }));
            } else {
                panic!("Expected Lambda expression body");
            }
        } else {
            panic!("Expected Lambda expression");
        }
    }

    #[test]
    fn test_lambda_with_block() {
        let stmts = parse("lambda(x) -> { print x }").unwrap();
        assert_eq!(stmts.len(), 1);
        if let Stmt::Expression(Expr::Lambda { params, body }) = &stmts[0] {
            assert_eq!(params.len(), 1);
            if let crate::ast::LambdaBody::Block(stmts) = body {
                assert_eq!(stmts.len(), 1);
                assert!(matches!(stmts[0], Stmt::Print(_)));
            } else {
                panic!("Expected Lambda block body");
            }
        } else {
            panic!("Expected Lambda expression");
        }
    }

    // ========== Array Tests ==========

    #[test]
    fn test_array_literal() {
        let stmts = parse("[1, 2, 3]").unwrap();
        assert_eq!(stmts.len(), 1);
        if let Stmt::Expression(Expr::Array(elements)) = &stmts[0] {
            assert_eq!(elements.len(), 3);
            if let Expr::Integer(n) = &elements[0] {
                assert_eq!(n, &1);
            }
            if let Expr::Integer(n) = &elements[1] {
                assert_eq!(n, &2);
            }
            if let Expr::Integer(n) = &elements[2] {
                assert_eq!(n, &3);
            }
        } else {
            panic!("Expected Array expression");
        }
    }

    #[test]
    fn test_empty_array() {
        let stmts = parse("[]").unwrap();
        assert_eq!(stmts.len(), 1);
        if let Stmt::Expression(Expr::Array(elements)) = &stmts[0] {
            assert!(elements.is_empty());
        } else {
            panic!("Expected Array expression");
        }
    }

    #[test]
    fn test_array_indexing() {
        let stmts = parse("arr[0]").unwrap();
        assert_eq!(stmts.len(), 1);
        if let Stmt::Expression(Expr::Index { object, index }) = &stmts[0] {
            if let Expr::Identifier(name) = &**object {
                assert_eq!(name, "arr");
            }
            if let Expr::Integer(n) = &**index {
                assert_eq!(n, &0);
            }
        } else {
            panic!("Expected Index expression");
        }
    }

    #[test]
    fn test_nested_array_indexing() {
        let stmts = parse("matrix[0][1]").unwrap();
        assert_eq!(stmts.len(), 1);
        if let Stmt::Expression(Expr::Index { object, index }) = &stmts[0] {
            // object should be another Index expression
            if let Expr::Index { object: inner_obj, index: inner_idx } = &**object {
                if let Expr::Integer(n) = &**inner_idx {
                    assert_eq!(n, &1);
                }
            } else {
                panic!("Expected nested Index");
            }
        } else {
            panic!("Expected Index expression");
        }
    }

    // ========== Dictionary Tests ==========

    #[test]
    fn test_dictionary_literal() {
        let stmts = parse("{name: \"Alice\", age: 30}").unwrap();
        assert_eq!(stmts.len(), 1);
        if let Stmt::Expression(Expr::Dictionary(pairs)) = &stmts[0] {
            assert_eq!(pairs.len(), 2);
            assert_eq!(pairs[0].0, "name");
            assert_eq!(pairs[1].0, "age");
        } else {
            panic!("Expected Dictionary expression");
        }
    }

    #[test]
    fn test_empty_dictionary() {
        let stmts = parse("{}").unwrap();
        assert_eq!(stmts.len(), 1);
        if let Stmt::Expression(Expr::Dictionary(pairs)) = &stmts[0] {
            assert!(pairs.is_empty());
        } else {
            panic!("Expected Dictionary expression");
        }
    }

    // ========== Range Tests ==========

    #[test]
    fn test_range_expression() {
        let stmts = parse("1..10").unwrap();
        assert_eq!(stmts.len(), 1);
        if let Stmt::Expression(Expr::Range { start, end, step }) = &stmts[0] {
            if let Expr::Integer(n) = &**start {
                assert_eq!(n, &1);
            }
            if let Expr::Integer(n) = &**end {
                assert_eq!(n, &10);
            }
            assert!(step.is_none());
        } else {
            panic!("Expected Range expression");
        }
    }

    #[test]
    fn test_range_with_step() {
        let stmts = parse("1..10:2").unwrap();
        assert_eq!(stmts.len(), 1);
        if let Stmt::Expression(Expr::Range { step, .. }) = &stmts[0] {
            assert!(step.is_some());
            if let Expr::Integer(n) = &**step.as_ref().unwrap() {
                assert_eq!(n, &2);
            }
        } else {
            panic!("Expected Range expression");
        }
    }

    #[test]
    fn test_reverse_range() {
        let stmts = parse("10..0:-1").unwrap();
        assert_eq!(stmts.len(), 1);
        if let Stmt::Expression(Expr::Range { start, end, step }) = &stmts[0] {
            if let Expr::Integer(n) = &**start {
                assert_eq!(n, &10);
            }
            if let Expr::Integer(n) = &**end {
                assert_eq!(n, &0);
            }
            if let Expr::Integer(n) = &**step.as_ref().unwrap() {
                assert_eq!(n, &-1);
            }
        } else {
            panic!("Expected Range expression");
        }
    }

    // ========== Struct Tests ==========

    #[test]
    fn test_struct_definition() {
        let stmts = parse("struct Point { x, y }").unwrap();
        assert_eq!(stmts.len(), 1);
        if let Stmt::Struct { name, fields } = &stmts[0] {
            assert_eq!(name, "Point");
            assert_eq!(fields.len(), 2);
            assert_eq!(fields[0], "x");
            assert_eq!(fields[1], "y");
        } else {
            panic!("Expected Struct statement");
        }
    }

    #[test]
    fn test_struct_init() {
        let stmts = parse("new Point { x: 10, y: 20 }").unwrap();
        assert_eq!(stmts.len(), 1);
        if let Stmt::Expression(Expr::StructInit { struct_name, fields }) = &stmts[0] {
            assert_eq!(struct_name, "Point");
            assert_eq!(fields.len(), 2);
            assert_eq!(fields[0].0, "x");
            assert_eq!(fields[1].0, "y");
        } else {
            panic!("Expected StructInit expression");
        }
    }

    // ========== Import/Export Tests ==========

    #[test]
    fn test_import_statement() {
        let stmts = parse("import \"math\"").unwrap();
        assert_eq!(stmts.len(), 1);
        if let Stmt::Import { module_path, items, alias } = &stmts[0] {
            assert_eq!(module_path, "math");
            assert!(items.is_none());
            assert!(alias.is_none());
        } else {
            panic!("Expected Import statement");
        }
    }

    #[test]
    fn test_import_with_alias() {
        let stmts = parse("import \"math\" as m").unwrap();
        assert_eq!(stmts.len(), 1);
        if let Stmt::Import { module_path, alias, .. } = &stmts[0] {
            assert_eq!(module_path, "math");
            assert_eq!(alias, &Some("m".to_string()));
        } else {
            panic!("Expected Import statement");
        }
    }

    #[test]
    fn test_selective_import() {
        let stmts = parse("import {PI, sqrt} from \"math\"").unwrap();
        assert_eq!(stmts.len(), 1);
        if let Stmt::Import { module_path, items, .. } = &stmts[0] {
            assert_eq!(module_path, "math");
            assert!(items.is_some());
            let items = items.as_ref().unwrap();
            assert_eq!(items.len(), 2);
            assert_eq!(items[0], "PI");
            assert_eq!(items[1], "sqrt");
        } else {
            panic!("Expected Import statement");
        }
    }

    #[test]
    fn test_export_statement() {
        let stmts = parse("export PI = 3.14159").unwrap();
        assert_eq!(stmts.len(), 1);
        if let Stmt::Export { name, .. } = &stmts[0] {
            assert_eq!(name, "PI");
        } else {
            panic!("Expected Export statement");
        }
    }

    #[test]
    fn test_export_function() {
        let stmts = parse("export fn add(a, b)\n    return a + b\nend").unwrap();
        assert_eq!(stmts.len(), 1);
        if let Stmt::ExportFunction { name, params, .. } = &stmts[0] {
            assert_eq!(name, "add");
            assert_eq!(params.len(), 2);
        } else {
            panic!("Expected ExportFunction statement");
        }
    }

    // ========== Match Expression Tests ==========

    #[test]
    fn test_match_expression() {
        let stmts = parse("match x\n    1 -> \"one\"\n    2 -> \"two\"\n    _ -> \"other\"\nend").unwrap();
        assert_eq!(stmts.len(), 1);
        if let Stmt::Expression(Expr::Match { expr, arms }) = &stmts[0] {
            assert!(matches!(&**expr, Expr::Identifier(_)));
            assert_eq!(arms.len(), 3);
            
            // First arm: literal pattern
            if let Pattern::Literal(Expr::Integer(n)) = &arms[0].pattern {
                assert_eq!(n, &1);
            } else {
                panic!("Expected literal pattern");
            }
            
            // Last arm: wildcard pattern
            if let Pattern::Wildcard = &arms[2].pattern {
                // Success
            } else {
                panic!("Expected wildcard pattern");
            }
        } else {
            panic!("Expected Match expression");
        }
    }

    // ========== Conditional Expression Tests ==========

    #[test]
    fn test_conditional_expression() {
        let stmts = parse("if x > 0 then \"positive\" else \"negative\" end").unwrap();
        assert_eq!(stmts.len(), 1);
        if let Stmt::Expression(Expr::ConditionalExpr { condition, then_expr, else_expr, .. }) = &stmts[0] {
            assert!(matches!(&**condition, Expr::Binary { operator: BinaryOp::Greater, .. }));
            if let Expr::String(s) = &**then_expr {
                assert_eq!(s, "positive");
            }
            if let Some(else_e) = else_expr {
                if let Expr::String(s) = &**else_e {
                    assert_eq!(s, "negative");
                }
            } else {
                panic!("Expected else expression");
            }
        } else {
            panic!("Expected ConditionalExpr");
        }
    }

    // ========== Error Handling Tests ==========

    #[test]
    fn test_try_catch() {
        let stmts = parse("try\n    risky_operation()\ncatch err\n    print err\nend").unwrap();
        assert_eq!(stmts.len(), 1);
        if let Stmt::Try { body, catch_var, catch_body } = &stmts[0] {
            assert_eq!(body.len(), 1);
            assert_eq!(catch_var, &Some("err".to_string()));
            assert_eq!(catch_body.len(), 1);
        } else {
            panic!("Expected Try statement");
        }
    }

    #[test]
    fn test_throw_statement() {
        let stmts = parse("throw \"error message\"").unwrap();
        assert_eq!(stmts.len(), 1);
        if let Stmt::Throw(expr) = &stmts[0] {
            if let Expr::String(s) = expr {
                assert_eq!(s, "error message");
            }
        } else {
            panic!("Expected Throw statement");
        }
    }

    // ========== Parse Error Tests ==========

    #[test]
    fn test_missing_end_in_if() {
        let result = parse("if x > 0 then\n    print \"positive\"");
        assert!(result.is_err());
    }

    #[test]
    fn test_unbalanced_parentheses() {
        let result = parse("add(1, 2");
        assert!(result.is_err());
    }

    #[test]
    fn test_unbalanced_brackets() {
        let result = parse("[1, 2, 3");
        assert!(result.is_err());
    }

    #[test]
    fn test_unbalanced_braces() {
        let result = parse("{name: \"Alice\"");
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_token() {
        // Parser should receive an error or unexpected token
        let result = parse("@invalid");
        // This might fail at lexing or parsing stage
        assert!(result.is_err() || result.unwrap().is_empty());
    }

    // ========== Complex Expression Tests ==========

    #[test]
    fn test_chained_function_calls() {
        let stmts = parse("foo(bar(baz(42)))").unwrap();
        assert_eq!(stmts.len(), 1);
        if let Stmt::Expression(Expr::Call { callee, .. }) = &stmts[0] {
            // callee should be a call to bar
            if let Expr::Call { callee: inner_callee, .. } = &**callee {
                // inner_callee should be a call to baz
                if let Expr::Call { .. } = &**inner_callee {
                    // Success - nested calls
                } else {
                    panic!("Expected nested Call");
                }
            } else {
                panic!("Expected nested Call");
            }
        } else {
            panic!("Expected Call expression");
        }
    }

    #[test]
    fn test_member_access() {
        let stmts = parse("obj.property").unwrap();
        assert_eq!(stmts.len(), 1);
        if let Stmt::Expression(Expr::StructAccess { object, field }) = &stmts[0] {
            if let Expr::Identifier(name) = &**object {
                assert_eq!(name, "obj");
            }
            assert_eq!(field, "property");
        } else {
            panic!("Expected StructAccess expression");
        }
    }

    #[test]
    fn test_chained_member_access() {
        let stmts = parse("obj.prop1.prop2").unwrap();
        assert_eq!(stmts.len(), 1);
        if let Stmt::Expression(Expr::StructAccess { object, field }) = &stmts[0] {
            assert_eq!(field, "prop2");
            // object should be another StructAccess
            if let Expr::StructAccess { field: inner_field, .. } = &**object {
                assert_eq!(inner_field, "prop1");
            }
        } else {
            panic!("Expected StructAccess expression");
        }
    }

    #[test]
    fn test_complex_math_expression() {
        let stmts = parse("(a + b) * (c - d) / (e % f)").unwrap();
        assert_eq!(stmts.len(), 1);
        // Should parse correctly with proper precedence
    }

    #[test]
    fn test_string_concatenation() {
        let stmts = parse("\"Hello, \" + name + \"!\"").unwrap();
        assert_eq!(stmts.len(), 1);
        // Should parse as binary expressions
    }

    #[test]
    fn test_mixed_type_array() {
        let stmts = parse("[1, \"two\", true, nil]").unwrap();
        assert_eq!(stmts.len(), 1);
        if let Stmt::Expression(Expr::Array(elements)) = &stmts[0] {
            assert_eq!(elements.len(), 4);
            assert!(matches!(&elements[0], Expr::Integer(_)));
            assert!(matches!(&elements[1], Expr::String(_)));
            assert!(matches!(&elements[2], Expr::Bool(_)));
            assert!(matches!(&elements[3], Expr::Nil));
        } else {
            panic!("Expected Array expression");
        }
    }
}
