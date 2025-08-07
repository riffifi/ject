use crate::ast::{Expr, Stmt, BinaryOp, UnaryOp, Argument};
use crate::lexer::InterpolationPart;
use crate::value::{Value, Environment};
use std::fmt;
use std::fs;
use std::path::Path;
use std::collections::HashMap;

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
                self.load_module(module_path, items, alias)?;
                Ok(ControlFlow::None)
            }
            Stmt::Export { name, value } => {
                // For now, just evaluate and store the value like a let statement
                let val = self.evaluate_expression(value)?;
                self.environment.define(name.clone(), val);
                Ok(ControlFlow::None)
            }
            Stmt::ExportFunction { name, params, body } => {
                let func = Value::Function {
                    params: params.clone(),
                    body: body.clone(),
                };
                self.environment.define(name.clone(), func);
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
                            let located_tokens = lexer.tokenize_with_positions();
                            let tokens: Vec<crate::lexer::Token> = located_tokens.into_iter().map(|lt| lt.token).collect();
let mut parser = crate::parser::Parser::new_simple(tokens);
                            
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
                
                self.call_function(func, args)
            }
            Expr::Array(elements) => {
                let mut values = Vec::new();
                for element in elements {
                    values.push(self.evaluate_expression(element)?);
                }
                Ok(Value::Array(values))
            }
            Expr::Dictionary(pairs) => {
                let mut map = std::collections::HashMap::new();
                for (key, value_expr) in pairs {
                    let value = self.evaluate_expression(value_expr)?;
                    map.insert(key.clone(), value);
                }
                Ok(Value::Dictionary(map))
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
                    (Value::Dictionary(dict), Value::String(key)) => {
                        Ok(dict.get(&key).cloned().unwrap_or(Value::Nil))
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
            Expr::Member { object, property } => {
                let obj = self.evaluate_expression(object)?;
                
                match obj {
                    Value::ModuleObject(exports) => {
                        exports.get(property).cloned().ok_or_else(|| RuntimeError {
                            message: format!("Property '{}' not found in module", property),
                        })
                    }
                    _ => Err(RuntimeError {
                        message: format!("Cannot access property '{}' on {}", property, obj.type_name()),
                    })
                }
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
            Expr::ConditionalExpr { condition, then_expr, elseif_branches, else_expr } => {
                let cond_value = self.evaluate_expression(condition)?;
                
                if cond_value.is_truthy() {
                    return self.evaluate_expression(then_expr);
                } 
                
                for elseif in elseif_branches {
                    let elseif_cond_value = self.evaluate_expression(&elseif.condition)?;
                    if elseif_cond_value.is_truthy() {
                        return self.evaluate_expression(&elseif.then_expr);
                    }
                }
                
                if let Some(else_expr) = else_expr {
                    return self.evaluate_expression(else_expr);
                }
                
                Ok(Value::Nil)
            }
            Expr::Match { expr, arms } => {
                let match_value = self.evaluate_expression(expr)?;
                
                for arm in arms {
                    if self.pattern_matches(&arm.pattern, &match_value)? {
                        // If pattern is an identifier, bind it to the value
                        if let crate::ast::Pattern::Identifier(name) = &arm.pattern {
                            self.environment.push_scope();
                            self.environment.define(name.clone(), match_value.clone());
                            let result = self.evaluate_expression(&arm.body)?;
                            self.environment.pop_scope();
                            return Ok(result);
                        } else {
                            return Ok(self.evaluate_expression(&arm.body)?);
                        }
                    }
                }
                
                Err(RuntimeError {
                    message: "No matching pattern found in match expression".to_string(),
                })
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
            (Value::Array(a), BinaryOp::Add, Value::Array(b)) => {
                let mut result = a.clone();
                result.extend(b.clone());
                Ok(Value::Array(result))
            },
            
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
            (Value::Integer(a), BinaryOp::Equal, Value::Float(b)) => Ok(Value::Bool(*a as f64 == *b)),
            (Value::Float(a), BinaryOp::Equal, Value::Integer(b)) => Ok(Value::Bool(*a == *b as f64)),
            (Value::String(a), BinaryOp::Equal, Value::String(b)) => Ok(Value::Bool(a == b)),
            (Value::Bool(a), BinaryOp::Equal, Value::Bool(b)) => Ok(Value::Bool(a == b)),
            (Value::Array(a), BinaryOp::Equal, Value::Array(b)) => Ok(Value::Bool(a == b)),
            (Value::Collection(a), BinaryOp::Equal, Value::Collection(b)) => Ok(Value::Bool(a == b)),
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
            
            // In operator - check if left value is contained in right value
            (left_val, BinaryOp::In, Value::Array(arr)) => {
                for item in arr {
                    let equal = self.evaluate_binary_op(left_val, &BinaryOp::Equal, item)?;
                    if let Value::Bool(true) = equal {
                        return Ok(Value::Bool(true));
                    }
                }
                Ok(Value::Bool(false))
            }
            (Value::String(substr), BinaryOp::In, Value::String(s)) => {
                Ok(Value::Bool(s.contains(substr)))
            }
            (left_val, BinaryOp::In, Value::String(s)) => {
                // Convert left value to string and check if it's in the string
                let left_str = left_val.to_string();
                Ok(Value::Bool(s.contains(&left_str)))
            }
            (Value::String(key), BinaryOp::In, Value::Dictionary(dict)) => {
                Ok(Value::Bool(dict.contains_key(key)))
            }
            (Value::String(key), BinaryOp::In, Value::Collection(set)) => {
                Ok(Value::Bool(set.contains(key)))
            }
            
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
    
    fn load_module(&mut self, module_path: &str, items: &Option<Vec<String>>, alias: &Option<String>) -> RuntimeResult<()> {
        // Look for module in examples/modules/ directory for now
        let module_file_path = format!("examples/modules/{}.ject", module_path);
        
        if !Path::new(&module_file_path).exists() {
            return Err(RuntimeError {
                message: format!("Module '{}' not found at {}", module_path, module_file_path),
            });
        }
        
        // Read and parse the module file
        let module_content = fs::read_to_string(&module_file_path)
            .map_err(|e| RuntimeError {
                message: format!("Failed to read module '{}': {}", module_path, e),
            })?;
            
        let mut lexer = crate::lexer::Lexer::new(&module_content);
        let located_tokens = lexer.tokenize_with_positions();
        let tokens: Vec<crate::lexer::Token> = located_tokens.into_iter().map(|lt| lt.token).collect();
let mut parser = crate::parser::Parser::new_simple(tokens);
        let statements = parser.parse().map_err(|e| RuntimeError {
            message: format!("Parse error in module '{}': {}", module_path, e),
        })?;
        
        // Create a new environment for the module
        let mut module_env = Environment::new();
        
        // Load standard library into module environment 
        let stdlib = crate::stdlib::create_stdlib();
        for (name, value) in stdlib {
            module_env.define(name, value);
        }
        
        // Save current environment and switch to module environment
        let saved_env = std::mem::replace(&mut self.environment, module_env);
        
        // First, execute all non-export statements to build up the module environment
        for statement in &statements {
            match statement {
                Stmt::Export { .. } | Stmt::ExportFunction { .. } => {
                    // Skip export statements for now
                }
                _ => {
                    self.execute_statement(statement)?;
                }
            }
        }
        
        // First, process export functions and define them in the module environment
        for statement in &statements {
            if let Stmt::ExportFunction { name, params, body } = statement {
                let func = Value::Function {
                    params: params.clone(),
                    body: body.clone(),
                };
                self.environment.define(name.clone(), func);
            }
        }
        
        // Now process export statements and create module functions with proper closure
        let mut exports = HashMap::new();
        for statement in &statements {
            match statement {
                Stmt::Export { name, value } => {
                    let val = self.evaluate_expression(value)?;
                    exports.insert(name.clone(), val.clone());
                }
                Stmt::ExportFunction { name, params, body } => {
                    let func = Value::ModuleFunction {
                        params: params.clone(),
                        body: body.clone(),
                        closure_env: self.environment.clone(),
                    };
                    exports.insert(name.clone(), func.clone());
                }
                _ => {
                    // Already processed
                }
            }
        }
        
        // Restore original environment
        self.environment = saved_env;
        
        // Import the exported values based on import type
        match (items, alias) {
            (Some(item_list), None) => {
                // import {item1, item2} from "module"
                for item_name in item_list {
                    if let Some(value) = exports.get(item_name) {
                        self.environment.define(item_name.clone(), value.clone());
                    } else {
                        return Err(RuntimeError {
                            message: format!("Module '{}' does not export '{}'", module_path, item_name),
                        });
                    }
                }
            }
            (None, Some(alias_name)) => {
                // import "module" as alias
                // Create a module object with all exports
                let module_obj = Value::ModuleObject(exports);
                self.environment.define(alias_name.clone(), module_obj);
            }
            (None, None) => {
                // import "module" - import all exports directly
                for (name, value) in exports {
                    self.environment.define(name, value);
                }
            }
            (Some(_), Some(_)) => {
                return Err(RuntimeError {
                    message: "Cannot use both specific imports and alias in the same import statement".to_string(),
                });
            }
        }
        
        Ok(())
    }
    
    fn call_function(&mut self, func: Value, args: &[Argument]) -> RuntimeResult<Value> {
        match func {
            Value::Function { params, body } => {
                let resolved_args = self.resolve_arguments(&params, args)?;
                
                self.environment.push_scope();
                
                for (param, arg) in params.iter().zip(resolved_args.iter()) {
                    self.environment.define(param.name.clone(), arg.clone());
                }
                
                let result = match self.execute_block(&body)? {
                    ControlFlow::Return(value) => value,
                    ControlFlow::None => Value::Nil,
                };
                
                self.environment.pop_scope();
                Ok(result)
            }
            Value::ModuleFunction { params, body, closure_env } => {
                let resolved_args = self.resolve_arguments(&params, args)?;
                
                // Save current environment and use the closure environment
                let saved_env = std::mem::replace(&mut self.environment, closure_env);
                
                self.environment.push_scope();
                
                for (param, arg) in params.iter().zip(resolved_args.iter()) {
                    self.environment.define(param.name.clone(), arg.clone());
                }
                
                let result = match self.execute_block(&body)? {
                    ControlFlow::Return(value) => value,
                    ControlFlow::None => Value::Nil,
                };
                
                self.environment.pop_scope();
                
                // Restore original environment
                self.environment = saved_env;
                Ok(result)
            }
            Value::Lambda { params, body } => {
                // Convert arguments to old format for lambdas (they don't support defaults yet)
                let mut arg_values = Vec::new();
                for arg in args {
                    match arg {
                        Argument::Positional(expr) => {
                            arg_values.push(self.evaluate_expression(expr)?);
                        }
                        Argument::Keyword { .. } => {
                            return Err(RuntimeError {
                                message: "Lambdas do not support keyword arguments".to_string(),
                            });
                        }
                    }
                }
                
                if arg_values.len() != params.len() {
                    return Err(RuntimeError {
                        message: format!("Expected {} arguments but got {}", params.len(), arg_values.len()),
                    });
                }
                
                self.environment.push_scope();
                
                for (param, arg) in params.iter().zip(arg_values.iter()) {
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
                // Convert arguments to old format for builtin functions
                let mut arg_values = Vec::new();
                for arg in args {
                    match arg {
                        Argument::Positional(expr) => {
                            arg_values.push(self.evaluate_expression(expr)?);
                        }
                        Argument::Keyword { .. } => {
                            return Err(RuntimeError {
                                message: "Builtin functions do not support keyword arguments".to_string(),
                            });
                        }
                    }
                }
                
                crate::stdlib::call_builtin_function(&name, arg_values)
            }
            _ => Err(RuntimeError {
                message: format!("Cannot call {}", func.type_name()),
            }),
        }
    }
    
    fn resolve_arguments(&mut self, params: &[crate::ast::Parameter], args: &[Argument]) -> RuntimeResult<Vec<Value>> {
        let mut resolved_args = vec![None; params.len()];
        let mut positional_count = 0;
        
        // First pass: handle positional arguments
        for arg in args {
            match arg {
                Argument::Positional(expr) => {
                    if positional_count >= params.len() {
                        return Err(RuntimeError {
                            message: "Too many positional arguments".to_string(),
                        });
                    }
                    resolved_args[positional_count] = Some(self.evaluate_expression(expr)?);
                    positional_count += 1;
                }
                Argument::Keyword { .. } => {
                    // We'll handle keyword arguments in the second pass
                }
            }
        }
        
        // Second pass: handle keyword arguments
        for arg in args {
            if let Argument::Keyword { name, value } = arg {
                // Find the parameter with this name
                let param_index = params.iter().position(|p| p.name == *name);
                
                match param_index {
                    Some(index) => {
                        if resolved_args[index].is_some() {
                            return Err(RuntimeError {
                                message: format!("Argument '{}' specified multiple times", name),
                            });
                        }
                        resolved_args[index] = Some(self.evaluate_expression(value)?);
                    }
                    None => {
                        return Err(RuntimeError {
                            message: format!("Unknown parameter '{}'", name),
                        });
                    }
                }
            }
        }
        
        // Third pass: fill in default values and check for missing arguments
        for (i, param) in params.iter().enumerate() {
            if resolved_args[i].is_none() {
                if let Some(default_expr) = &param.default_value {
                    resolved_args[i] = Some(self.evaluate_expression(default_expr)?);
                } else {
                    return Err(RuntimeError {
                        message: format!("Missing required argument '{}'"  , param.name),
                    });
                }
            }
        }
        
        // Convert Vec<Option<Value>> to Vec<Value>
        Ok(resolved_args.into_iter().map(|arg| arg.unwrap()).collect())
    }
    
    fn pattern_matches(&mut self, pattern: &crate::ast::Pattern, value: &Value) -> RuntimeResult<bool> {
        match pattern {
            crate::ast::Pattern::Wildcard => Ok(true),
            crate::ast::Pattern::Identifier(_) => Ok(true), // Identifiers always match and bind
            crate::ast::Pattern::Literal(expr) => {
                let pattern_value = self.evaluate_expression(expr)?;
                let equal = self.evaluate_binary_op(value, &BinaryOp::Equal, &pattern_value)?;
                if let Value::Bool(is_equal) = equal {
                    Ok(is_equal)
                } else {
                    Ok(false)
                }
            }
        }
    }
}
