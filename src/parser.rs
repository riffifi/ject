use crate::lexer::Token;
use crate::ast::{Expr, Stmt, BinaryOp, UnaryOp};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

#[derive(Debug)]
pub struct ParseError {
    pub message: String,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Parse error: {}", self.message)
    }
}

impl std::error::Error for ParseError {}

type ParseResult<T> = Result<T, ParseError>;

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, current: 0 }
    }
    
    pub fn parse(&mut self) -> ParseResult<Vec<Stmt>> {
        let mut statements = Vec::new();
        
        while !self.is_at_end() {
            // Skip newlines at the top level
            if self.match_token(&Token::Newline) {
                continue;
            }
            
            statements.push(self.statement()?);
        }
        
        Ok(statements)
    }
    
    fn statement(&mut self) -> ParseResult<Stmt> {
        match &self.peek() {
            Token::Let => self.let_statement(),
            Token::Fn => self.function_statement(),
            Token::If => self.if_statement(),
            Token::While => self.while_statement(),
            Token::For => self.for_statement(),
            Token::Return => self.return_statement(),
            Token::Print => self.print_statement(),
            Token::Import => self.import_statement(),
            Token::Export => self.export_statement(),
            Token::Identifier(_) => {
                // Check if this is an assignment
                if self.peek_ahead(1).map(|t| matches!(t, Token::Equal)).unwrap_or(false) {
                    self.assignment_statement()
                } else {
                    Ok(Stmt::Expression(self.expression()?))
                }
            },
            _ => Ok(Stmt::Expression(self.expression()?)),
        }
    }
    
    fn let_statement(&mut self) -> ParseResult<Stmt> {
        self.consume(Token::Let, "Expected 'let'")?;
        
        let name = if let Token::Identifier(name) = self.advance() {
            name
        } else {
            return Err(ParseError {
                message: "Expected identifier after 'let'".to_string(),
            });
        };
        
        self.consume(Token::Equal, "Expected '=' after variable name")?;
        let value = self.expression()?;
        
        Ok(Stmt::Let { name, value })
    }
    
    fn function_statement(&mut self) -> ParseResult<Stmt> {
        self.consume(Token::Fn, "Expected 'fn'")?;
        
        let name = if let Token::Identifier(name) = self.advance() {
            name
        } else {
            return Err(ParseError {
                message: "Expected function name".to_string(),
            });
        };
        
        self.consume(Token::LeftParen, "Expected '(' after function name")?;
        
        let mut params = Vec::new();
        if !self.check(&Token::RightParen) {
            loop {
                if let Token::Identifier(param_name) = self.advance() {
                    // Check for default value
                    let default_value = if self.match_token(&Token::Equal) {
                        Some(self.expression()?)
                    } else {
                        None
                    };
                    
                    params.push(crate::ast::Parameter {
                        name: param_name,
                        default_value,
                    });
                } else {
                    return Err(ParseError {
                        message: "Expected parameter name".to_string(),
                    });
                }
                
                if !self.match_token(&Token::Comma) {
                    break;
                }
            }
        }
        
        self.consume(Token::RightParen, "Expected ')' after parameters")?;
        
        // Skip optional newlines before body
        while self.match_token(&Token::Newline) {}
        
        let body = self.block()?;
        
        Ok(Stmt::Function { name, params, body })
    }
    
    fn if_statement(&mut self) -> ParseResult<Stmt> {
        self.consume(Token::If, "Expected 'if'")?;
        let condition = self.expression()?;
        
        // Optional 'then' keyword
        self.match_token(&Token::Then);
        
        // Skip optional newlines before body
        while self.match_token(&Token::Newline) {}
        
        let then_branch = self.if_block()?;
        
        // Parse elseif branches
        let mut elseif_branches = Vec::new();
        while self.match_token(&Token::ElseIf) {
            let elseif_condition = self.expression()?;
            
            // Optional 'then' keyword
            self.match_token(&Token::Then);
            
            // Skip optional newlines before body
            while self.match_token(&Token::Newline) {}
            
            let elseif_body = self.if_block()?;
            
            elseif_branches.push(crate::ast::ElseIfBranch {
                condition: elseif_condition,
                body: elseif_body,
            });
        }
        
        // Handle traditional "else if" pattern for backward compatibility
        let mut needs_end_token = true;
        if self.match_token(&Token::Else) {
            // Skip optional newlines before else body
            while self.match_token(&Token::Newline) {}
            
            // Check for "else if" pattern
            if self.check(&Token::If) {
                // This is an "else if" - recursively parse remaining elseifs and else
                let remaining_if = self.if_statement()?;
                if let Stmt::If { condition: nested_condition, then_branch: nested_then, elseif_branches: mut nested_elseifs, else_branch: nested_else } = remaining_if {
                    // Convert the nested if into an elseif branch
                    elseif_branches.push(crate::ast::ElseIfBranch {
                        condition: nested_condition,
                        body: nested_then,
                    });
                    
                    // Add all nested elseif branches
                    elseif_branches.append(&mut nested_elseifs);
                    
                    // Set the final else branch
                    return Ok(Stmt::If {
                        condition,
                        then_branch,
                        elseif_branches,
                        else_branch: nested_else,
                    });
                }
                needs_end_token = false; // The recursive call already consumed the end token
            } else {
                // Regular else block
                let else_body = self.if_block()?;
                // Consume the final 'end' token
                self.consume(Token::End, "Expected 'end' after if statement")?;
                return Ok(Stmt::If {
                    condition,
                    then_branch,
                    elseif_branches,
                    else_branch: Some(else_body),
                });
            }
        }
        
        // Consume the final 'end' token if needed
        if needs_end_token {
            // Check if we're at the end of file or encountered a token that shouldn't be here
            if self.is_at_end() {
                return Err(ParseError {
                    message: "Expected 'end' to close if statement but reached end of file".to_string(),
                });
            }
            
            // Provide a better error message based on what we found instead of 'end'
            if let Err(parse_error) = self.consume(Token::End, "Expected 'end' after if statement") {
                let current_token = self.peek();
                let better_message = match current_token {
                    Token::Return => "Expected 'end' to close if statement, but found another 'return'. Did you forget an 'end'?".to_string(),
                    Token::Identifier(_) => "Expected 'end' to close if statement, but found another statement. Did you forget an 'end'?".to_string(),
                    _ => parse_error.message,
                };
                return Err(ParseError { message: better_message });
            }
        }
        
        Ok(Stmt::If {
            condition,
            then_branch,
            elseif_branches,
            else_branch: None,
        })
    }
    
    fn while_statement(&mut self) -> ParseResult<Stmt> {
        self.consume(Token::While, "Expected 'while'")?;
        let condition = self.expression()?;
        
        // Optional 'do' keyword
        self.match_token(&Token::Do);
        
        // Skip optional newlines before body
        while self.match_token(&Token::Newline) {}
        
        let body = self.block()?;
        
        Ok(Stmt::While { condition, body })
    }
    
    fn for_statement(&mut self) -> ParseResult<Stmt> {
        self.consume(Token::For, "Expected 'for'")?;
        
        let var = if let Token::Identifier(name) = self.advance() {
            name
        } else {
            return Err(ParseError {
                message: "Expected variable name in for loop".to_string(),
            });
        };
        
        self.consume(Token::In, "Expected 'in' after for variable")?;
        let iterable = self.expression()?;
        
        // Optional 'do' keyword
        self.match_token(&Token::Do);
        
        // Skip optional newlines before body
        while self.match_token(&Token::Newline) {}
        
        let body = self.block()?;
        
        Ok(Stmt::For { var, iterable, body })
    }
    
    fn return_statement(&mut self) -> ParseResult<Stmt> {
        self.consume(Token::Return, "Expected 'return'")?;
        
        let value = if self.check(&Token::Newline) || self.check(&Token::End) || self.is_at_end() {
            None
        } else {
            Some(self.expression()?)
        };
        
        Ok(Stmt::Return(value))
    }
    
    fn print_statement(&mut self) -> ParseResult<Stmt> {
        self.consume(Token::Print, "Expected 'print'")?;
        let expr = self.expression()?;
        Ok(Stmt::Print(expr))
    }
    
    fn assignment_statement(&mut self) -> ParseResult<Stmt> {
        let name = if let Token::Identifier(name) = self.advance() {
            name
        } else {
            return Err(ParseError {
                message: "Expected identifier in assignment".to_string(),
            });
        };
        
        self.consume(Token::Equal, "Expected '=' in assignment")?;
        let value = self.expression()?;
        
        Ok(Stmt::Assign { name, value })
    }
    
    fn import_statement(&mut self) -> ParseResult<Stmt> {
        self.consume(Token::Import, "Expected 'import'")?;
        
        let (module_path, items, alias) = if self.match_token(&Token::LeftBrace) {
            // import {item1, item2} from "module"
            let mut items = Vec::new();
            
            if !self.check(&Token::RightBrace) {
                loop {
                    if let Token::Identifier(item) = self.advance() {
                        items.push(item);
                    } else {
                        return Err(ParseError {
                            message: "Expected identifier in import list".to_string(),
                        });
                    }
                    
                    if !self.match_token(&Token::Comma) {
                        break;
                    }
                }
            }
            
            self.consume(Token::RightBrace, "Expected '}' after import list")?;
            self.consume(Token::From, "Expected 'from' after import list")?;
            
            let module_path = if let Token::String(path) = self.advance() {
                path
            } else {
                return Err(ParseError {
                    message: "Expected string after 'from'".to_string(),
                });
            };
            
            (module_path, Some(items), None)
        } else {
            // import "module" or import "module" as alias
            let module_path = if let Token::String(path) = self.advance() {
                path
            } else {
                return Err(ParseError {
                    message: "Expected module path string after 'import'".to_string(),
                });
            };
            
            let alias = if self.match_token(&Token::As) {
                if let Token::Identifier(alias_name) = self.advance() {
                    Some(alias_name)
                } else {
                    return Err(ParseError {
                        message: "Expected identifier after 'as'".to_string(),
                    });
                }
            } else {
                None
            };
            
            (module_path, None, alias)
        };
        
        Ok(Stmt::Import { module_path, items, alias })
    }
    
    fn export_statement(&mut self) -> ParseResult<Stmt> {
        self.consume(Token::Export, "Expected 'export'")?;
        
        // Check if this is "export fn"
        if self.match_token(&Token::Fn) {
            let name = if let Token::Identifier(name) = self.advance() {
                name
            } else {
                return Err(ParseError {
                    message: "Expected function name after 'export fn'".to_string(),
                });
            };
            
            self.consume(Token::LeftParen, "Expected '(' after function name")?;
            
            let mut params = Vec::new();
            if !self.check(&Token::RightParen) {
                loop {
                    if let Token::Identifier(param_name) = self.advance() {
                        // Check for default value
                        let default_value = if self.match_token(&Token::Equal) {
                            Some(self.expression()?)
                        } else {
                            None
                        };
                        
                        params.push(crate::ast::Parameter {
                            name: param_name,
                            default_value,
                        });
                    } else {
                        return Err(ParseError {
                            message: "Expected parameter name".to_string(),
                        });
                    }
                    
                    if !self.match_token(&Token::Comma) {
                        break;
                    }
                }
            }
            
            self.consume(Token::RightParen, "Expected ')' after parameters")?;
            
            // Skip optional newlines before body
            while self.match_token(&Token::Newline) {}
            
            let body = self.block()?;
            
            return Ok(Stmt::ExportFunction { name, params, body });
        }
        
        // Regular "export name = value" syntax
        let name = if let Token::Identifier(name) = self.advance() {
            name
        } else {
            return Err(ParseError {
                message: "Expected identifier after 'export'".to_string(),
            });
        };
        
        self.consume(Token::Equal, "Expected '=' after export name")?;
        let value = self.expression()?;
        
        Ok(Stmt::Export { name, value })
    }
    
    fn block(&mut self) -> ParseResult<Vec<Stmt>> {
        let mut statements = Vec::new();
        
        while !self.check(&Token::End) && !self.check(&Token::ElseIf) && !self.check(&Token::Else) && !self.is_at_end() {
            // Skip newlines within blocks
            if self.match_token(&Token::Newline) {
                continue;
            }
            
            statements.push(self.statement()?);
        }
        
        // Only consume 'end' if we stopped because of 'end'
        if self.check(&Token::End) {
            self.consume(Token::End, "Expected 'end'")?;
        }
        
        Ok(statements)
    }
    
    // Special block method for if statements - doesn't consume 'end'
    fn if_block(&mut self) -> ParseResult<Vec<Stmt>> {
        let mut statements = Vec::new();
        
        while !self.check(&Token::End) && !self.check(&Token::ElseIf) && !self.check(&Token::Else) && !self.is_at_end() {
            // Skip newlines within blocks
            if self.match_token(&Token::Newline) {
                continue;
            }
            
            let stmt = self.statement()?;
            statements.push(stmt);
            
            // If we just parsed a return statement and the next token suggests we're outside
            // the if block (like another return statement or function call), this suggests
            // a missing 'end' token
            if let Some(last_stmt) = statements.last() {
                if matches!(last_stmt, crate::ast::Stmt::Return(_)) {
                    // Skip newlines to peek at the next meaningful token
                    let mut peek_pos = self.current;
                    while peek_pos < self.tokens.len() && matches!(self.tokens[peek_pos], Token::Newline) {
                        peek_pos += 1;
                    }
                    
                    if peek_pos < self.tokens.len() {
                        match &self.tokens[peek_pos] {
                            Token::Return | Token::Identifier(_) => {
                                // This suggests we might have gone past the if block
                                // Let's exit the loop and let if_statement handle the error
                                break;
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        
        Ok(statements)
    }
    
    fn expression(&mut self) -> ParseResult<Expr> {
        self.or()
    }
    
    fn or(&mut self) -> ParseResult<Expr> {
        let mut expr = self.and()?;
        
        while self.match_token(&Token::Or) {
            let right = self.and()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: BinaryOp::Or,
                right: Box::new(right),
            };
        }
        
        Ok(expr)
    }
    
    fn and(&mut self) -> ParseResult<Expr> {
        let mut expr = self.range()?;
        
        while self.match_token(&Token::And) {
            let right = self.range()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: BinaryOp::And,
                right: Box::new(right),
            };
        }
        
        Ok(expr)
    }
    
    fn range(&mut self) -> ParseResult<Expr> {
        let mut expr = self.equality()?;
        
        if self.match_token(&Token::DotDot) {
            let end = self.equality()?;
            
            // Check for optional step with colon syntax
            let step = if self.match_token(&Token::Colon) {
                Some(Box::new(self.equality()?))
            } else {
                None
            };
            
            expr = Expr::Range {
                start: Box::new(expr),
                end: Box::new(end),
                step,
            };
        }
        
        Ok(expr)
    }
    
    fn equality(&mut self) -> ParseResult<Expr> {
        let mut expr = self.comparison()?;
        
        while let Some(op) = self.match_equality_op() {
            let right = self.comparison()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: op,
                right: Box::new(right),
            };
        }
        
        Ok(expr)
    }
    
    fn comparison(&mut self) -> ParseResult<Expr> {
        let mut expr = self.term()?;
        
        while let Some(op) = self.match_comparison_op() {
            let right = self.term()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: op,
                right: Box::new(right),
            };
        }
        
        Ok(expr)
    }
    
    fn term(&mut self) -> ParseResult<Expr> {
        let mut expr = self.factor()?;
        
        while let Some(op) = self.match_term_op() {
            // Skip newlines after binary operators
            while self.match_token(&Token::Newline) {}
            let right = self.factor()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: op,
                right: Box::new(right),
            };
        }
        
        Ok(expr)
    }
    
    fn factor(&mut self) -> ParseResult<Expr> {
        let mut expr = self.unary()?;
        
        while let Some(op) = self.match_factor_op() {
            // Skip newlines after binary operators
            while self.match_token(&Token::Newline) {}
            let right = self.unary()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: op,
                right: Box::new(right),
            };
        }
        
        Ok(expr)
    }
    
    fn unary(&mut self) -> ParseResult<Expr> {
        if let Some(op) = self.match_unary_op() {
            let operand = self.unary()?;
            return Ok(Expr::Unary {
                operator: op,
                operand: Box::new(operand),
            });
        }
        
        self.call()
    }
    
    fn call(&mut self) -> ParseResult<Expr> {
        let mut expr = self.primary()?;
        
        loop {
            if self.match_token(&Token::LeftParen) {
                expr = self.finish_call(expr)?;
            } else if self.match_token(&Token::LeftBracket) {
                let index = self.expression()?;
                self.consume(Token::RightBracket, "Expected ']' after array index")?;
                expr = Expr::Index {
                    object: Box::new(expr),
                    index: Box::new(index),
                };
            } else if self.match_token(&Token::Dot) {
                if let Token::Identifier(property) = self.advance() {
                    expr = Expr::Member {
                        object: Box::new(expr),
                        property,
                    };
                } else {
                    return Err(ParseError {
                        message: "Expected property name after '.'".to_string(),
                    });
                }
            } else {
                break;
            }
        }
        
        Ok(expr)
    }
    
    fn finish_call(&mut self, callee: Expr) -> ParseResult<Expr> {
        let mut args = Vec::new();
        
        if !self.check(&Token::RightParen) {
            loop {
                // Check for keyword argument (identifier followed by =)
                if let Token::Identifier(name) = &self.peek() {
                    if self.peek_ahead(1).map(|t| matches!(t, Token::Equal)).unwrap_or(false) {
                        // This is a keyword argument
                        let param_name = if let Token::Identifier(name) = self.advance() {
                            name
                        } else {
                            unreachable!()
                        };
                        
                        self.consume(Token::Equal, "Expected '=' after parameter name")?;
                        let value = self.expression()?;
                        
                        args.push(crate::ast::Argument::Keyword {
                            name: param_name,
                            value,
                        });
                    } else {
                        // This is a positional argument
                        args.push(crate::ast::Argument::Positional(self.expression()?));
                    }
                } else {
                    // This is a positional argument
                    args.push(crate::ast::Argument::Positional(self.expression()?));
                }
                
                if !self.match_token(&Token::Comma) {
                    break;
                }
            }
        }
        
        self.consume(Token::RightParen, "Expected ')' after arguments")?;
        
        Ok(Expr::Call {
            callee: Box::new(callee),
            args,
        })
    }
    
    fn primary(&mut self) -> ParseResult<Expr> {
        match self.advance() {
            Token::Match => self.match_expression(),
            Token::Lambda => self.lambda_expression(),
            Token::True => Ok(Expr::Bool(true)),
            Token::False => Ok(Expr::Bool(false)),
            Token::Nil => Ok(Expr::Nil),
            Token::Integer(n) => Ok(Expr::Integer(n)),
            Token::Float(n) => Ok(Expr::Float(n)),
            Token::String(s) => Ok(Expr::String(s)),
            Token::InterpolatedString(parts) => Ok(Expr::InterpolatedString(parts)),
            Token::Identifier(name) => Ok(Expr::Identifier(name)),
            Token::LeftParen => {
                let expr = self.expression()?;
                self.consume(Token::RightParen, "Expected ')' after expression")?;
                Ok(expr)
            }
            Token::LeftBracket => {
                let mut elements = Vec::new();
                
                if !self.check(&Token::RightBracket) {
                    loop {
                        elements.push(self.expression()?);
                        if !self.match_token(&Token::Comma) {
                            break;
                        }
                    }
                }
                
                self.consume(Token::RightBracket, "Expected ']' after array elements")?;
                Ok(Expr::Array(elements))
            }
            Token::LeftBrace => {
                let mut pairs = Vec::new();
                
                if !self.check(&Token::RightBrace) {
                    loop {
                        // Parse key: value pairs
                        let key = match self.advance() {
                            Token::Identifier(name) => name,
                            Token::String(s) => s,
                            _ => {
                                return Err(ParseError {
                                    message: "Expected string or identifier as dictionary key".to_string(),
                                });
                            }
                        };
                        
                        self.consume(Token::Colon, "Expected ':' after dictionary key")?;
                        let value = self.expression()?;
                        pairs.push((key, value));
                        
                        if !self.match_token(&Token::Comma) {
                            break;
                        }
                    }
                }
                
                self.consume(Token::RightBrace, "Expected '}' after dictionary elements")?;
                Ok(Expr::Dictionary(pairs))
            }
            token => Err(ParseError {
                message: format!("Unexpected token: {:?}", token),
            }),
        }
    }
    
    // Helper methods
    fn lambda_expression(&mut self) -> ParseResult<Expr> {
        self.consume(Token::LeftParen, "Expected '(' after 'fn'")?;
        let mut params = Vec::new();
        if !self.check(&Token::RightParen) {
            loop {
                if let Token::Identifier(name) = self.advance() {
                    params.push(name);
                } else {
                    return Err(ParseError {
                        message: "Expected parameter name".to_string(),
                    });
                }
                if !self.match_token(&Token::Comma) {
                    break;
                }
            }
        }
        self.consume(Token::RightParen, "Expected ')' after parameters")?;
        self.consume(Token::Arrow, "Expected '->' before lambda body")?;

        let body = if self.match_token(&Token::LeftBrace) {
            let mut statements = Vec::new();
            
            while !self.check(&Token::RightBrace) && !self.is_at_end() {
                // Skip newlines within blocks
                if self.match_token(&Token::Newline) {
                    continue;
                }
                statements.push(self.statement()?);
            }
            
            self.consume(Token::RightBrace, "Expected '}' after lambda block")?;
            crate::ast::LambdaBody::Block(statements)
        } else {
            let expr = self.expression()?;
            crate::ast::LambdaBody::Expression(Box::new(expr))
        };

        Ok(Expr::Lambda { params, body })
    }
    
    fn match_expression(&mut self) -> ParseResult<Expr> {
        let expr = self.expression()?;
        
        // Skip optional newlines before match arms
        while self.match_token(&Token::Newline) {}
        
        let mut arms = Vec::new();
        
        while !self.check(&Token::End) && !self.is_at_end() {
            // Skip newlines between arms
            if self.match_token(&Token::Newline) {
                continue;
            }
            
            // Parse pattern
            let pattern = self.parse_pattern()?;
            
            // Expect arrow
            self.consume(Token::Arrow, "Expected '->' after match pattern")?;
            
            // Parse body expression
            let body = self.expression()?;
            
            arms.push(crate::ast::MatchArm { pattern, body });
        }
        
        self.consume(Token::End, "Expected 'end' after match expression")?;
        
        Ok(Expr::Match {
            expr: Box::new(expr),
            arms,
        })
    }
    
    fn parse_pattern(&mut self) -> ParseResult<crate::ast::Pattern> {
        match self.advance() {
            Token::Integer(n) => Ok(crate::ast::Pattern::Literal(Expr::Integer(n))),
            Token::Float(f) => Ok(crate::ast::Pattern::Literal(Expr::Float(f))),
            Token::String(s) => Ok(crate::ast::Pattern::Literal(Expr::String(s))),
            Token::True => Ok(crate::ast::Pattern::Literal(Expr::Bool(true))),
            Token::False => Ok(crate::ast::Pattern::Literal(Expr::Bool(false))),
            Token::Nil => Ok(crate::ast::Pattern::Literal(Expr::Nil)),
            Token::Identifier(name) => {
                if name == "_" {
                    Ok(crate::ast::Pattern::Wildcard)
                } else {
                    Ok(crate::ast::Pattern::Identifier(name))
                }
            }
            token => Err(ParseError {
                message: format!("Unexpected token in pattern: {:?}", token),
            }),
        }
    }
    
    fn match_equality_op(&mut self) -> Option<BinaryOp> {
        if self.match_token(&Token::EqualEqual) {
            Some(BinaryOp::Equal)
        } else if self.match_token(&Token::BangEqual) {
            Some(BinaryOp::NotEqual)
        } else {
            None
        }
    }
    
    fn match_comparison_op(&mut self) -> Option<BinaryOp> {
        if self.match_token(&Token::Greater) {
            Some(BinaryOp::Greater)
        } else if self.match_token(&Token::GreaterEqual) {
            Some(BinaryOp::GreaterEqual)
        } else if self.match_token(&Token::Less) {
            Some(BinaryOp::Less)
        } else if self.match_token(&Token::LessEqual) {
            Some(BinaryOp::LessEqual)
        } else if self.match_token(&Token::In) {
            Some(BinaryOp::In)
        } else {
            None
        }
    }
    
    fn match_term_op(&mut self) -> Option<BinaryOp> {
        if self.match_token(&Token::Minus) {
            Some(BinaryOp::Subtract)
        } else if self.match_token(&Token::Plus) {
            Some(BinaryOp::Add)
        } else {
            None
        }
    }
    
    fn match_factor_op(&mut self) -> Option<BinaryOp> {
        if self.match_token(&Token::Slash) {
            Some(BinaryOp::Divide)
        } else if self.match_token(&Token::Star) {
            Some(BinaryOp::Multiply)
        } else if self.match_token(&Token::Percent) {
            Some(BinaryOp::Modulo)
        } else {
            None
        }
    }
    
    fn match_unary_op(&mut self) -> Option<UnaryOp> {
        if self.match_token(&Token::Bang) {
            Some(UnaryOp::Not)
        } else if self.match_token(&Token::Minus) {
            Some(UnaryOp::Negate)
        } else {
            None
        }
    }
    
    fn match_token(&mut self, token: &Token) -> bool {
        if self.check(token) {
            self.advance();
            true
        } else {
            false
        }
    }
    
    fn check(&self, token: &Token) -> bool {
        if self.is_at_end() {
            false
        } else {
            std::mem::discriminant(&self.peek()) == std::mem::discriminant(token)
        }
    }
    
    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }
    
    fn is_at_end(&self) -> bool {
        matches!(self.peek(), Token::Eof)
    }
    
    fn peek(&self) -> Token {
        self.tokens[self.current].clone()
    }
    
    fn peek_ahead(&self, offset: usize) -> Option<Token> {
        self.tokens.get(self.current + offset).cloned()
    }
    
    fn previous(&self) -> Token {
        self.tokens[self.current - 1].clone()
    }
    
    fn consume(&mut self, token: Token, message: &str) -> ParseResult<Token> {
        if self.check(&token) {
            Ok(self.advance())
        } else {
            Err(ParseError {
                message: format!("{} but got {:?}", message, self.peek()),
            })
        }
    }
}
