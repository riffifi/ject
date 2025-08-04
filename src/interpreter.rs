use crate::ast::{Expr, Stmt, BinaryOp, UnaryOp};
use crate::lexer::InterpolationPart;
use crate::value::{Value, Environment};
use std::fmt;

#[derive(Debug)]
pub struct RuntimeError {
    pub message: String,
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Runtime error: {}", self.message)
    }
}

impl std::error::Error for RuntimeError {}

type RuntimeResult<T> = Result<T, RuntimeError>;

pub struct Interpreter {
    environment: Environment,
}

#[derive(Debug)]
pub enum ControlFlow {
    None,
    Return(Value),
}

impl Interpreter {
    pub fn new() -> Self {
        let mut environment = Environment::new();
        
        // Load standard library
        let stdlib = crate::stdlib::create_stdlib();
        for (name, value) in stdlib {
            environment.define(name, value);
        }
        
        Interpreter {
            environment,
        }
    }
    
    pub fn interpret(&mut self, statements: &[Stmt]) -> RuntimeResult<()> {
        for statement in statements {
            match self.execute_statement(statement)? {
                ControlFlow::Return(_) => break,
                ControlFlow::None => continue,
            }
        }
        Ok(())
    }
    
    fn execute_statement(&mut self, stmt: &Stmt) -> RuntimeResult<ControlFlow> {
        match stmt {
            Stmt::Expression(expr) => {
                self.evaluate_expression(expr)?;
                Ok(ControlFlow::None)
            }
            Stmt::Let { name, value } => {
                let val = self.evaluate_expression(value)?;
                self.environment.define(name.clone(), val);
                Ok(ControlFlow::None)
            }
            Stmt::Assign { name, value } => {
                let val = self.evaluate_expression(value)?;
                if self.environment.set(name, val) {
                    Ok(ControlFlow::None)
                } else {
                    Err(RuntimeError {
                        message: format!("Undefined variable '{}'", name),
                    })
                }
            }
            Stmt::Function { name, params, body } => {
                let func = Value::Function {
                    params: params.clone(),
                    body: body.clone(),
                };
                self.environment.define(name.clone(), func);
                Ok(ControlFlow::None)
            }
            Stmt::If { condition, then_branch, elseif_branches, else_branch } => {
                let cond_value = self.evaluate_expression(condition)?;
                
                if cond_value.is_truthy() {
                    self.execute_block(then_branch)
                } else {
                    // Check elseif conditions
                    for elseif_branch in elseif_branches {
                        let elseif_cond_value = self.evaluate_expression(&elseif_branch.condition)?;
                        if elseif_cond_value.is_truthy() {
                            return self.execute_block(&elseif_branch.body);
                        }
                    }
                    
                    // If no elseif matched, execute else branch if present
                    if let Some(else_stmts) = else_branch {
                        self.execute_block(else_stmts)
                    } else {
                        Ok(ControlFlow::None)
                    }
                }
            }
            Stmt::While { condition, body } => {
                while self.evaluate_expression(condition)?.is_truthy() {
                    match self.execute_block(body)? {
                        ControlFlow::Return(value) => return Ok(ControlFlow::Return(value)),
                        ControlFlow::None => continue,
                    }
                }
                Ok(ControlFlow::None)
            }
            Stmt::For { var, iterable, body } => {
                let iter_value = self.evaluate_expression(iterable)?;
                
                match iter_value {
                    Value::Array(elements) => {
                        for element in elements {
                            self.environment.push_scope();
                            self.environment.define(var.clone(), element);
                            
                            match self.execute_block(body)? {
                                ControlFlow::Return(value) => {
                                    self.environment.pop_scope();
                                    return Ok(ControlFlow::Return(value));
                                }
                                ControlFlow::None => {}
                            }
                            
                            self.environment.pop_scope();
                        }
                    }
                    _ => {
                        return Err(RuntimeError {
                            message: format!("Cannot iterate over {}", iter_value.type_name()),
                        });
                    }
                }
                Ok(ControlFlow::None)
            }
            Stmt::Return(expr) => {
                let value = if let Some(e) = expr {
                    self.evaluate_expression(e)?
                } else {
                    Value::Nil
                };
                Ok(ControlFlow::Return(value))
            }
            Stmt::Import { module_path, items, alias } => {
                // For now, just return Ok - we'll implement actual module loading later
                println!("Import statement: module_path={}, items={:?}, alias={:?}", module_path, items, alias);
                Ok(ControlFlow::None)
            }
            Stmt::Export { name, value } => {
                // For now, just evaluate and store the value like a let statement
                let val = self.evaluate_expression(value)?;
                self.environment.define(name.clone(), val);
                println!("Export statement: name={}", name);
                Ok(ControlFlow::None)
            }
            Stmt::Print(expr) => {
                let value = self.evaluate_expression(expr)?;
                println!("{}", value);
                Ok(ControlFlow::None)
            }
        }
    }
    
    fn execute_block(&mut self, statements: &[Stmt]) -> RuntimeResult<ControlFlow> {
        for statement in statements {
            match self.execute_statement(statement)? {
                ControlFlow::Return(value) => return Ok(ControlFlow::Return(value)),
                ControlFlow::None => continue,
            }
        }
        Ok(ControlFlow::None)
    }
    
    fn evaluate_expression(&mut self, expr: &Expr) -> RuntimeResult<Value> {
        match expr {
            Expr::Integer(n) => Ok(Value::Integer(*n)),
            Expr::Float(f) => Ok(Value::Float(*f)),
            Expr::String(s) => Ok(Value::String(s.clone())),
            Expr::InterpolatedString(parts) => {
                let mut result = String::new();
                for part in parts {
                    match part {
                        InterpolationPart::Text(text) => {
                            result.push_str(text);
                        }
                        InterpolationPart::Expression(expr_str) => {
                            // Parse and evaluate the expression
                            let mut lexer = crate::lexer::Lexer::new(expr_str);
                            let tokens = lexer.tokenize();
                            let mut parser = crate::parser::Parser::new(tokens);
                            
                            match parser.parse() {
                                Ok(statements) => {
                                    if let Some(stmt) = statements.first() {
                                        if let crate::ast::Stmt::Expression(expr) = stmt {
                                            let value = self.evaluate_expression(expr)?;
                                            result.push_str(&value.to_string());
                                        } else {
                                            return Err(RuntimeError {
                                                message: "Invalid expression in string interpolation".to_string(),
                                            });
                                        }
                                    }
                                }
                                Err(e) => {
                                    return Err(RuntimeError {
                                        message: format!("Parse error in string interpolation: {}", e),
                                    });
                                }
                            }
                        }
                    }
                }
                Ok(Value::String(result))
            }
            Expr::Bool(b) => Ok(Value::Bool(*b)),
            Expr::Nil => Ok(Value::Nil),
            Expr::Identifier(name) => {
                self.environment.get(name).ok_or_else(|| RuntimeError {
                    message: format!("Undefined variable '{}'.", name),
                })
            }
            Expr::Binary { left, operator, right } => {
                let left_val = self.evaluate_expression(left)?;
                let right_val = self.evaluate_expression(right)?;
                self.evaluate_binary_op(&left_val, operator, &right_val)
            }
            Expr::Unary { operator, operand } => {
                let operand_val = self.evaluate_expression(operand)?;
                self.evaluate_unary_op(operator, &operand_val)
            }
            Expr::Call { callee, args } => {
                let func = self.evaluate_expression(callee)?;
                let mut arg_values = Vec::new();
                
                for arg in args {
                    arg_values.push(self.evaluate_expression(arg)?);
                }
                
                self.call_function(func, arg_values)
            }
            Expr::Array(elements) => {
                let mut values = Vec::new();
                for element in elements {
                    values.push(self.evaluate_expression(element)?);
                }
                Ok(Value::Array(values))
            }
            Expr::Index { object, index } => {
                let obj = self.evaluate_expression(object)?;
                let idx = self.evaluate_expression(index)?;
                
                match (obj, idx) {
                    (Value::Array(arr), Value::Integer(i)) => {
                        let index = i as usize;
                        if index < arr.len() {
                            Ok(arr[index].clone())
                        } else {
                            Err(RuntimeError {
                                message: "Array index out of bounds".to_string(),
                            })
                        }
                    }
                    (obj, idx) => Err(RuntimeError {
                        message: format!("Cannot index {} with {}", obj.type_name(), idx.type_name()),
                    }),
                }
            }
            Expr::Lambda { params, body } => {
                Ok(Value::Lambda {
                    params: params.clone(),
                    body: body.clone(),
                })
            }
            Expr::Range { start, end, step } => {
                let start_val = self.evaluate_expression(start)?;
                let end_val = self.evaluate_expression(end)?;
                
                let step_val = if let Some(step_expr) = step {
                    self.evaluate_expression(step_expr)?
                } else {
                    Value::Integer(1)  // Default step is 1
                };
                
                match (start_val, end_val, step_val) {
                    (Value::Integer(start_i), Value::Integer(end_i), Value::Integer(step_i)) => {
                        if step_i == 0 {
                            return Err(RuntimeError {
                                message: "Range step cannot be zero".to_string(),
                            });
                        }
                        
                        let mut result = Vec::new();
                        let mut current = start_i;
                        
                        if step_i > 0 {
                            while current < end_i {
                                result.push(Value::Integer(current));
                                current += step_i;
                            }
                        } else {
                            while current > end_i {
                                result.push(Value::Integer(current));
                                current += step_i;
                            }
                        }
                        
                        Ok(Value::Array(result))
                    }
                    _ => Err(RuntimeError {
                        message: "Range start, end, and step must all be integers".to_string(),
                    })
                }
            }
        }
    }
    
    fn evaluate_binary_op(&self, left: &Value, op: &BinaryOp, right: &Value) -> RuntimeResult<Value> {
        match (left, op, right) {
            // Arithmetic
            (Value::Integer(a), BinaryOp::Add, Value::Integer(b)) => Ok(Value::Integer(a + b)),
            (Value::Float(a), BinaryOp::Add, Value::Float(b)) => Ok(Value::Float(a + b)),
            (Value::Integer(a), BinaryOp::Add, Value::Float(b)) => Ok(Value::Float(*a as f64 + b)),
            (Value::Float(a), BinaryOp::Add, Value::Integer(b)) => Ok(Value::Float(a + *b as f64)),
            (Value::String(a), BinaryOp::Add, Value::String(b)) => Ok(Value::String(format!("{}{}", a, b))),
            (Value::String(a), BinaryOp::Add, b) => Ok(Value::String(format!("{}{}", a, b))),
            (a, BinaryOp::Add, Value::String(b)) => Ok(Value::String(format!("{}{}", a, b))),
            
            (Value::Integer(a), BinaryOp::Subtract, Value::Integer(b)) => Ok(Value::Integer(a - b)),
            (Value::Float(a), BinaryOp::Subtract, Value::Float(b)) => Ok(Value::Float(a - b)),
            (Value::Integer(a), BinaryOp::Subtract, Value::Float(b)) => Ok(Value::Float(*a as f64 - b)),
            (Value::Float(a), BinaryOp::Subtract, Value::Integer(b)) => Ok(Value::Float(a - *b as f64)),
            
            (Value::Integer(a), BinaryOp::Multiply, Value::Integer(b)) => Ok(Value::Integer(a * b)),
            (Value::Float(a), BinaryOp::Multiply, Value::Float(b)) => Ok(Value::Float(a * b)),
            (Value::Integer(a), BinaryOp::Multiply, Value::Float(b)) => Ok(Value::Float(*a as f64 * b)),
            (Value::Float(a), BinaryOp::Multiply, Value::Integer(b)) => Ok(Value::Float(a * *b as f64)),
            
            (Value::Integer(a), BinaryOp::Divide, Value::Integer(b)) => {
                if *b == 0 {
                    Err(RuntimeError { message: "Division by zero".to_string() })
                } else {
                    Ok(Value::Float(*a as f64 / *b as f64))
                }
            }
            (Value::Float(a), BinaryOp::Divide, Value::Float(b)) => {
                if *b == 0.0 {
                    Err(RuntimeError { message: "Division by zero".to_string() })
                } else {
                    Ok(Value::Float(a / b))
                }
            }
            (Value::Integer(a), BinaryOp::Divide, Value::Float(b)) => {
                if *b == 0.0 {
                    Err(RuntimeError { message: "Division by zero".to_string() })
                } else {
                    Ok(Value::Float(*a as f64 / b))
                }
            }
            (Value::Float(a), BinaryOp::Divide, Value::Integer(b)) => {
                if *b == 0 {
                    Err(RuntimeError { message: "Division by zero".to_string() })
                } else {
                    Ok(Value::Float(a / *b as f64))
                }
            }
            
            (Value::Integer(a), BinaryOp::Modulo, Value::Integer(b)) => {
                if *b == 0 {
                    Err(RuntimeError { message: "Modulo by zero".to_string() })
                } else {
                    Ok(Value::Integer(a % b))
                }
            }
            
            // Comparison
            (Value::Integer(a), BinaryOp::Equal, Value::Integer(b)) => Ok(Value::Bool(a == b)),
            (Value::Float(a), BinaryOp::Equal, Value::Float(b)) => Ok(Value::Bool(a == b)),
            (Value::String(a), BinaryOp::Equal, Value::String(b)) => Ok(Value::Bool(a == b)),
            (Value::Bool(a), BinaryOp::Equal, Value::Bool(b)) => Ok(Value::Bool(a == b)),
            (Value::Nil, BinaryOp::Equal, Value::Nil) => Ok(Value::Bool(true)),
            (_, BinaryOp::Equal, _) => Ok(Value::Bool(false)),
            
            (a, BinaryOp::NotEqual, b) => {
                let equal = self.evaluate_binary_op(a, &BinaryOp::Equal, b)?;
                if let Value::Bool(is_equal) = equal {
                    Ok(Value::Bool(!is_equal))
                } else {
                    Ok(Value::Bool(true))
                }
            }
            
            (Value::Integer(a), BinaryOp::Less, Value::Integer(b)) => Ok(Value::Bool(a < b)),
            (Value::Float(a), BinaryOp::Less, Value::Float(b)) => Ok(Value::Bool(a < b)),
            (Value::Integer(a), BinaryOp::Less, Value::Float(b)) => Ok(Value::Bool((*a as f64) < *b)),
            (Value::Float(a), BinaryOp::Less, Value::Integer(b)) => Ok(Value::Bool(*a < (*b as f64))),
            
            (Value::Integer(a), BinaryOp::Greater, Value::Integer(b)) => Ok(Value::Bool(a > b)),
            (Value::Float(a), BinaryOp::Greater, Value::Float(b)) => Ok(Value::Bool(a > b)),
            (Value::Integer(a), BinaryOp::Greater, Value::Float(b)) => Ok(Value::Bool((*a as f64) > *b)),
            (Value::Float(a), BinaryOp::Greater, Value::Integer(b)) => Ok(Value::Bool(*a > (*b as f64))),
            
            (Value::Integer(a), BinaryOp::LessEqual, Value::Integer(b)) => Ok(Value::Bool(a <= b)),
            (Value::Float(a), BinaryOp::LessEqual, Value::Float(b)) => Ok(Value::Bool(a <= b)),
            (Value::Integer(a), BinaryOp::LessEqual, Value::Float(b)) => Ok(Value::Bool((*a as f64) <= *b)),
            (Value::Float(a), BinaryOp::LessEqual, Value::Integer(b)) => Ok(Value::Bool(*a <= (*b as f64))),
            
            (Value::Integer(a), BinaryOp::GreaterEqual, Value::Integer(b)) => Ok(Value::Bool(a >= b)),
            (Value::Float(a), BinaryOp::GreaterEqual, Value::Float(b)) => Ok(Value::Bool(a >= b)),
            (Value::Integer(a), BinaryOp::GreaterEqual, Value::Float(b)) => Ok(Value::Bool((*a as f64) >= *b)),
            (Value::Float(a), BinaryOp::GreaterEqual, Value::Integer(b)) => Ok(Value::Bool(*a >= (*b as f64))),
            
            // Logical
            (a, BinaryOp::And, b) => {
                if a.is_truthy() {
                    Ok(b.clone())
                } else {
                    Ok(a.clone())
                }
            }
            (a, BinaryOp::Or, b) => {
                if a.is_truthy() {
                    Ok(a.clone())
                } else {
                    Ok(b.clone())
                }
            }
            
            (left, op, right) => Err(RuntimeError {
                message: format!("Unsupported operation: {} {} {}", left.type_name(), op, right.type_name()),
            }),
        }
    }
    
    fn evaluate_unary_op(&self, op: &UnaryOp, operand: &Value) -> RuntimeResult<Value> {
        match (op, operand) {
            (UnaryOp::Negate, Value::Integer(n)) => Ok(Value::Integer(-n)),
            (UnaryOp::Negate, Value::Float(f)) => Ok(Value::Float(-f)),
            (UnaryOp::Not, val) => Ok(Value::Bool(!val.is_truthy())),
            (op, operand) => Err(RuntimeError {
                message: format!("Unsupported unary operation: {} {}", op, operand.type_name()),
            }),
        }
    }
    
    fn call_function(&mut self, func: Value, args: Vec<Value>) -> RuntimeResult<Value> {
        match func {
            Value::Function { params, body } => {
                if args.len() != params.len() {
                    return Err(RuntimeError {
                        message: format!("Expected {} arguments but got {}", params.len(), args.len()),
                    });
                }
                
                self.environment.push_scope();
                
                for (param, arg) in params.iter().zip(args.iter()) {
                    self.environment.define(param.clone(), arg.clone());
                }
                
                let result = match self.execute_block(&body)? {
                    ControlFlow::Return(value) => value,
                    ControlFlow::None => Value::Nil,
                };
                
                self.environment.pop_scope();
                Ok(result)
            }
            Value::Lambda { params, body } => {
                if args.len() != params.len() {
                    return Err(RuntimeError {
                        message: format!("Expected {} arguments but got {}", params.len(), args.len()),
                    });
                }
                
                self.environment.push_scope();
                
                for (param, arg) in params.iter().zip(args.iter()) {
                    self.environment.define(param.clone(), arg.clone());
                }
                
                let result = match body {
                    crate::ast::LambdaBody::Expression(expr) => {
                        self.evaluate_expression(&expr)?
                    }
                    crate::ast::LambdaBody::Block(statements) => {
                        match self.execute_block(&statements)? {
                            ControlFlow::Return(value) => value,
                            ControlFlow::None => Value::Nil,
                        }
                    }
                };
                
                self.environment.pop_scope();
                Ok(result)
            }
            Value::BuiltinFunction(name) => {
                crate::stdlib::call_builtin_function(&name, args)
            }
            _ => Err(RuntimeError {
                message: format!("Cannot call {}", func.type_name()),
            }),
        }
    }
}
