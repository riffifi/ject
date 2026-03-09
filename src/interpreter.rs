use crate::ast::{Expr, Stmt, BinaryOp, UnaryOp, Argument};
use crate::lexer::InterpolationPart;
use crate::value::{Value, Environment};
use std::fmt;
use std::fs;
use std::path::Path;
use std::collections::HashMap;

/// Get a helpful suggestion for runtime errors
pub fn get_runtime_suggestion(message: &str) -> String {
    let msg = message.to_lowercase();

    if msg.contains("undefined variable") {
        return "Check for typos, or declare the variable with 'let' first.".to_string();
    }
    if msg.contains("array index") {
        return "Use negative indices to count from the end (arr[-1] = last element).".to_string();
    }
    if msg.contains("break") {
        return "'break' can only be used inside for or while loops.".to_string();
    }
    if msg.contains("continue") {
        return "'continue' can only be used inside for or while loops.".to_string();
    }
    if msg.contains("sqrt") || msg.contains("pow") {
        return "sqrt() requires non-negative numbers. pow(0,0) is undefined.".to_string();
    }
    String::new()
}

#[derive(Debug)]
pub struct RuntimeError {
    pub message: String,
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Runtime error: {}{}", self.message, get_runtime_suggestion(&self.message))
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
    Throw(Value),
    Break,
    Continue,
}

impl Interpreter {
    pub fn new() -> Self {
        let mut environment = Environment::new();

        // Load CorLib (core library - always available)
        let corlib = crate::stdlib::create_corlib();
        for (name, value) in corlib {
            environment.define(name, value);
        }

        // Note: Standard library modules are now loaded via import statements
        // CorLib (core library) is loaded above
        // stdlib/index.ject is just documentation, not executable code

        Interpreter {
            environment,
        }
    }
    
//     fn load_stdlib_from_ject(environment: &mut Environment) -> RuntimeResult<()> {
//         // Load stdlib/index.ject
//         let stdlib_path = "stdlib/index.ject";
//         
//         if !Path::new(stdlib_path).exists() {
//             // If stdlib doesn't exist in Ject files, that's okay - use Rust stdlib
//             return Ok(());
//         }
//         
//         // Read and parse the stdlib index
//         let stdlib_content = fs::read_to_string(stdlib_path)
//             .map_err(|e| RuntimeError {
//                 message: format!("Failed to read stdlib: {}", e),
//             })?;
//             
//         let mut lexer = crate::lexer::Lexer::new(&stdlib_content);
//         let located_tokens = lexer.tokenize_with_positions();
//         let tokens: Vec<crate::lexer::Token> = located_tokens.into_iter().map(|lt| lt.token).collect();
//         let mut parser = crate::parser::Parser::new_simple(tokens);
//         let statements = parser.parse().map_err(|e| RuntimeError {
//             message: format!("Parse error in stdlib: {}", e),
//         })?;
//         
//         // Create a temporary interpreter to execute stdlib
//         let mut stdlib_interpreter = Interpreter {
//             environment: environment.clone(),
//         };
//         
//         // Execute stdlib statements
//         for statement in &statements {
//             match stdlib_interpreter.execute_statement(statement)? {
//                 ControlFlow::Return(_) => break,
//                 ControlFlow::Throw(_) => break, // Errors in stdlib are ignored
//                 ControlFlow::Break | ControlFlow::Continue => continue,
//                 ControlFlow::None => continue,
//             }
//         }
//         
//         // Merge stdlib environment back
//         *environment = stdlib_interpreter.environment;
//         
//         Ok(())
//     }
//     
    pub fn interpret(&mut self, statements: &[Stmt]) -> RuntimeResult<()> {
        for statement in statements {
            match self.execute_statement(statement)? {
                ControlFlow::Return(_) => break,
                ControlFlow::Throw(error) => {
                    return Err(RuntimeError {
                        message: format!("Uncaught error: {}", error),
                    });
                }
                ControlFlow::Break | ControlFlow::Continue => {
                    return Err(RuntimeError {
                        message: "break/continue outside of loop".to_string(),
                    });
                }
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
            Stmt::Assign { target, value } => {
                let val = self.evaluate_expression(value)?;
                
                match target {
                    crate::ast::AssignTarget::Identifier(name) => {
                        if self.environment.set(&name, val) {
                            Ok(ControlFlow::None)
                        } else {
                            Err(RuntimeError {
                                message: format!("Undefined variable '{}'", name),
                            })
                        }
                    }
                    crate::ast::AssignTarget::Index { object, index } => {
                        let obj = self.environment.get(&object)
                            .ok_or_else(|| RuntimeError {
                                message: format!("Undefined variable '{}'", object),
                            })?;
                        
                        let idx = self.evaluate_expression(&index)?;
                        
                        if let Value::Array(mut arr) = obj {
                            if let Value::Integer(i) = idx {
                                let actual_index = if i < 0 { arr.len() as i64 + i } else { i };
                                if actual_index < 0 || actual_index >= arr.len() as i64 {
                                    return Err(RuntimeError {
                                        message: format!("Array index out of bounds: {}", i),
                                    });
                                }
                                arr[actual_index as usize] = val;
                                self.environment.set(&object, Value::Array(arr));
                                Ok(ControlFlow::None)
                            } else {
                                Err(RuntimeError { message: "Array index must be integer".to_string() })
                            }
                        } else if let Value::Dictionary(mut dict) = obj {
                            if let Value::String(key) = idx {
                                dict.insert(key, val);
                                self.environment.set(&object, Value::Dictionary(dict));
                                Ok(ControlFlow::None)
                            } else {
                                Err(RuntimeError { message: "Dictionary key must be string".to_string() })
                            }
                        } else {
                            Err(RuntimeError { message: format!("Cannot index into {}", obj.type_name()) })
                        }
                    }
                    crate::ast::AssignTarget::Field { object, field } => {
                        let obj = self.environment.get(&object)
                            .ok_or_else(|| RuntimeError {
                                message: format!("Undefined variable '{}'", object),
                            })?;
                        
                        if let Value::Dictionary(mut dict) = obj {
                            dict.insert(field.clone(), val);
                            self.environment.set(&object, Value::Dictionary(dict));
                            Ok(ControlFlow::None)
                        } else if let Value::StructInstance { struct_name, mut fields } = obj {
                            fields.insert(field.clone(), val);
                            self.environment.set(&object, Value::StructInstance { struct_name, fields });
                            Ok(ControlFlow::None)
                        } else {
                            Err(RuntimeError { message: format!("Cannot assign field on {}", obj.type_name()) })
                        }
                    }
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
                        ControlFlow::Throw(error) => return Ok(ControlFlow::Throw(error)),
                        ControlFlow::Break => break,
                        ControlFlow::Continue => continue,
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
                                ControlFlow::Throw(error) => {
                                    self.environment.pop_scope();
                                    return Ok(ControlFlow::Throw(error));
                                }
                                ControlFlow::Break => {
                                    self.environment.pop_scope();
                                    break;
                                }
                                ControlFlow::Continue => {
                                    self.environment.pop_scope();
                                    continue;
                                }
                                ControlFlow::None => {}
                            }

                            self.environment.pop_scope();
                        }
                    }
                    Value::String(s) => {
                        // Iterate over characters in string
                        for ch in s.chars() {
                            self.environment.push_scope();
                            self.environment.define(var.clone(), Value::String(ch.to_string()));

                            match self.execute_block(body)? {
                                ControlFlow::Return(value) => {
                                    self.environment.pop_scope();
                                    return Ok(ControlFlow::Return(value));
                                }
                                ControlFlow::Throw(error) => {
                                    self.environment.pop_scope();
                                    return Ok(ControlFlow::Throw(error));
                                }
                                ControlFlow::Break => {
                                    self.environment.pop_scope();
                                    break;
                                }
                                ControlFlow::Continue => {
                                    self.environment.pop_scope();
                                    continue;
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
            Stmt::Print { values, sep, end } => {
                // Evaluate all values
                let mut output = Vec::new();
                for value_expr in values {
                    let value = self.evaluate_expression(value_expr)?;
                    output.push(value.display());  // Use display() for print (no quotes on strings)
                }

                // Determine separator (default: space)
                let separator = if let Some(sep_expr) = sep {
                    self.evaluate_expression(sep_expr)?.to_string()
                } else {
                    " ".to_string()
                };

                // Determine end (default: newline)
                let ending = if let Some(end_expr) = end {
                    self.evaluate_expression(end_expr)?.to_string()
                } else {
                    "\n".to_string()
                };

                // Print with separator
                print!("{}", output.join(&separator));
                print!("{}", ending);

                Ok(ControlFlow::None)
            }
            Stmt::Struct { name, fields } => {
                let struct_def = Value::StructDefinition {
                    name: name.clone(),
                    fields: fields.clone(),
                };
                self.environment.define(name.clone(), struct_def);
                Ok(ControlFlow::None)
            }
            Stmt::Try { body, catch_var, catch_body } => {
                let try_result = self.execute_block(body);
                match try_result {
                    Ok(ControlFlow::Throw(error_value)) => {
                        // Catch the error
                        self.environment.push_scope();
                        if let Some(var_name) = catch_var {
                            // Store the error value directly (it can be any value)
                            self.environment.define(var_name.clone(), error_value.clone());
                        }
                        let result = self.execute_block(catch_body)?;
                        self.environment.pop_scope();
                        Ok(result)
                    }
                    Ok(other) => Ok(other),
                    Err(e) => {
                        // Runtime error from function call - convert to throw
                        self.environment.push_scope();
                        if let Some(var_name) = catch_var {
                            self.environment.define(var_name.clone(), Value::String(e.message.clone()));
                        }
                        let result = self.execute_block(catch_body)?;
                        self.environment.pop_scope();
                        Ok(result)
                    }
                }
            }
            Stmt::Throw(expr) => {
                let error_value = self.evaluate_expression(expr)?;
                Ok(ControlFlow::Throw(error_value))
            }
            Stmt::Break => Ok(ControlFlow::Break),
            Stmt::Continue => Ok(ControlFlow::Continue),
        }
    }
    
    fn execute_block(&mut self, statements: &[Stmt]) -> RuntimeResult<ControlFlow> {
        for statement in statements {
            match self.execute_statement(statement)? {
                ControlFlow::Return(value) => return Ok(ControlFlow::Return(value)),
                ControlFlow::Throw(error) => return Ok(ControlFlow::Throw(error)),
                ControlFlow::Break => return Ok(ControlFlow::Break),
                ControlFlow::Continue => return Ok(ControlFlow::Continue),
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
            Expr::Increment { target, prefix } => {
                self.evaluate_increment_decrement(target, *prefix, true)
            }
            Expr::Decrement { target, prefix } => {
                self.evaluate_increment_decrement(target, *prefix, false)
            }
            Expr::Call { callee, args } => {
                // Check for higher-order functions that need special handling
                if let Expr::Identifier(func_name) = &**callee {
                    if func_name == "map" || func_name == "filter" || func_name == "reduce" {
                        return self.call_higher_order_function(func_name, args);
                    }
                }
                
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
            Expr::UniqueArray(elements) => {
                // Evaluate all elements and deduplicate
                let mut seen = std::collections::HashSet::new();
                let mut values = Vec::new();
                for element in elements {
                    let value = self.evaluate_expression(element)?;
                    // Create a string representation for comparison
                    let key = value.to_string();
                    if !seen.contains(&key) {
                        seen.insert(key);
                        values.push(value);
                    }
                }
                Ok(Value::UniqueArray(values))
            }
            Expr::ListComprehension { expr, var, iterable, condition } => {
                // Evaluate the iterable
                let iter_value = self.evaluate_expression(iterable)?;
                
                // Get elements from iterable (array, unique array, or range)
                let elements = match iter_value {
                    Value::Array(arr) => arr,
                    Value::UniqueArray(arr) => arr,
                    _ => return Err(RuntimeError { message: "Can only iterate over arrays or ranges".to_string() }),
                };
                
                let mut result = Vec::new();
                
                for item in &elements {
                    // Create new scope for loop variable
                    self.environment.push_scope();
                    self.environment.define(var.clone(), item.clone());
                    
                    // Check condition if present
                    let include = if let Some(cond) = condition {
                        let cond_value = self.evaluate_expression(cond)?;
                        cond_value.is_truthy()
                    } else {
                        true
                    };
                    
                    if include {
                        // Evaluate the expression
                        let value = self.evaluate_expression(expr)?;
                        result.push(value);
                    }
                    
                    self.environment.pop_scope();
                }
                
                Ok(Value::Array(result))
            }
            Expr::Generator { expr, var, iterable, condition } => {
                // For now, generators evaluate eagerly like list comprehensions
                // In the future, this could return a lazy iterator
                let iter_value = self.evaluate_expression(iterable)?;
                
                let elements = match iter_value {
                    Value::Array(arr) => arr,
                    Value::UniqueArray(arr) => arr,
                    _ => return Err(RuntimeError { message: "Can only iterate over arrays or ranges".to_string() }),
                };
                
                let mut result = Vec::new();
                
                for item in &elements {
                    self.environment.push_scope();
                    self.environment.define(var.clone(), item.clone());
                    
                    let include = if let Some(cond) = condition {
                        let cond_value = self.evaluate_expression(cond)?;
                        cond_value.is_truthy()
                    } else {
                        true
                    };
                    
                    if include {
                        let value = self.evaluate_expression(expr)?;
                        result.push(value);
                    }
                    
                    self.environment.pop_scope();
                }
                
                // Return as array for now (could be made lazy in future)
                Ok(Value::Array(result))
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
                        // Handle negative indices
                        let actual_index = if i < 0 {
                            (arr.len() as i64 + i) as usize
                        } else {
                            i as usize
                        };
                        
                        if actual_index < arr.len() {
                            Ok(arr[actual_index].clone())
                        } else {
                            Err(RuntimeError {
                                message: format!("Array index out of bounds: {}", i),
                            })
                        }
                    }
                    (Value::Dictionary(dict), Value::String(key)) => {
                        Ok(dict.get(&key).cloned().unwrap_or(Value::Nil))
                    }
                    (Value::String(s), Value::Integer(i)) => {
                        // Handle negative indices for strings
                        let chars: Vec<char> = s.chars().collect();
                        let len = chars.len() as i64;
                        let actual_index = if i < 0 {
                            (len + i) as usize
                        } else {
                            i as usize
                        };
                        
                        if actual_index < chars.len() {
                            Ok(Value::String(chars[actual_index].to_string()))
                        } else {
                            Err(RuntimeError {
                                message: format!("String index out of bounds: {}", i),
                            })
                        }
                    }
                    (obj, idx) => Err(RuntimeError {
                        message: format!("Cannot index {} with {}", obj.type_name(), idx.type_name()),
                    }),
                }
            }
            Expr::Slice { object, from, to, step } => {
                let obj = self.evaluate_expression(object)?;
                
                // Evaluate slice parameters
                let from_val = if let Some(from_expr) = from {
                    Some(self.evaluate_expression(from_expr)?)
                } else {
                    None
                };
                
                let to_val = if let Some(to_expr) = to {
                    Some(self.evaluate_expression(to_expr)?)
                } else {
                    None
                };
                
                let step_val = if let Some(step_expr) = step {
                    self.evaluate_expression(step_expr)?
                } else {
                    Value::Integer(1)  // Default step is 1
                };
                
                // Convert step to integer
                let step_i = match step_val {
                    Value::Integer(n) => n,
                    Value::Float(f) => f.floor() as i64,
                    _ => return Err(RuntimeError { message: "Slice step must be a number".to_string() }),
                };
                
                if step_i == 0 {
                    return Err(RuntimeError {
                        message: "Slice step cannot be zero".to_string(),
                    });
                }
                
                // Perform slicing based on object type
                match obj {
                    Value::Array(arr) => {
                        let len = arr.len() as i64;

                        // Convert from/to to actual indices
                        let from_i = match from_val {
                            Some(Value::Integer(n)) => {
                                if n < 0 { len + n } else { n }
                            }
                            None => {
                                if step_i > 0 { 0 } else { len - 1 }
                            }
                            _ => return Err(RuntimeError { message: "Slice 'from' must be an integer".to_string() }),
                        };

                        let to_i = match to_val {
                            Some(Value::Integer(n)) => {
                                if n < 0 { len + n } else { n }
                            }
                            None => {
                                if step_i > 0 { len } else { -1 }
                            }
                            _ => return Err(RuntimeError { message: "Slice 'to' must be an integer".to_string() }),
                        };

                        // Normalize indices (but preserve -1 for reverse slice end)
                        let from_i = if from_i < 0 { 0 } else if from_i > len { len } else { from_i };
                        // For to_i, only clamp upper bound, preserve negative for reverse slice
                        let to_i = if to_i > len { len } else { to_i };

                        let mut result = Vec::new();
                        let mut current = from_i;

                        if step_i > 0 {
                            while current < to_i {
                                result.push(arr[current as usize].clone());
                                current += step_i;
                            }
                        } else {
                            // For reverse slice, go down to and including to_i (if to_i >= 0)
                            // or down to and including 0 (if to_i < 0)
                            while current >= 0 && current < len && (to_i < 0 || current > to_i) {
                                result.push(arr[current as usize].clone());
                                if current == 0 { break; }
                                current += step_i;
                            }
                        }

                        Ok(Value::Array(result))
                    }
                    Value::String(s) => {
                        let chars: Vec<char> = s.chars().collect();
                        let len = chars.len() as i64;

                        // Convert from/to to actual indices
                        let from_i = match from_val {
                            Some(Value::Integer(n)) => {
                                if n < 0 { len + n } else { n }
                            }
                            None => {
                                if step_i > 0 { 0 } else { len - 1 }
                            }
                            _ => return Err(RuntimeError { message: "Slice 'from' must be an integer".to_string() }),
                        };

                        let to_i = match to_val {
                            Some(Value::Integer(n)) => {
                                if n < 0 { len + n } else { n }
                            }
                            None => {
                                if step_i > 0 { len } else { -1 }
                            }
                            _ => return Err(RuntimeError { message: "Slice 'to' must be an integer".to_string() }),
                        };

                        // Normalize indices (but preserve -1 for reverse slice end)
                        let from_i = if from_i < 0 { 0 } else if from_i > len { len } else { from_i };
                        // For to_i, only clamp upper bound, preserve negative for reverse slice
                        let to_i = if to_i > len { len } else { to_i };

                        let mut result = String::new();
                        let mut current = from_i;

                        if step_i > 0 {
                            while current < to_i {
                                result.push(chars[current as usize]);
                                current += step_i;
                            }
                        } else {
                            // For reverse slice, go down to and including to_i (if to_i >= 0)
                            // or down to and including 0 (if to_i < 0)
                            while current >= 0 && current < len && (to_i < 0 || current > to_i) {
                                result.push(chars[current as usize]);
                                if current == 0 { break; }
                                current += step_i;
                            }
                        }

                        Ok(Value::String(result))
                    }
                    _ => Err(RuntimeError {
                        message: format!("Cannot slice {}", obj.type_name()),
                    }),
                }
            }
            Expr::Lambda { params, body } => {
                // Capture current environment for closure support
                Ok(Value::Lambda {
                    params: params.clone(),
                    body: body.clone(),
                    closure_env: self.environment.clone(),
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
            Expr::StructAccess { object, field } => {
                let obj = self.evaluate_expression(object)?;
                
                match obj {
                    Value::StructInstance { fields, .. } => {
                        fields.get(field).cloned().ok_or_else(|| RuntimeError {
                            message: format!("Field '{}' not found in struct instance", field),
                        })
                    }
                    Value::ModuleObject(exports) => {
                        // Also support module member access via dot notation
                        exports.get(field).cloned().ok_or_else(|| RuntimeError {
                            message: format!("Property '{}' not found in module", field),
                        })
                    }
                    _ => Err(RuntimeError {
                        message: format!("Cannot access field '{}' on {}", field, obj.type_name()),
                    })
                }
            }
            Expr::StructInit { struct_name, fields } => {
                // Get struct definition
                let struct_def = self.environment.get(struct_name)
                    .ok_or_else(|| RuntimeError {
                        message: format!("Struct '{}' not defined", struct_name),
                    })?;
                
                if let Value::StructDefinition { fields: def_fields, .. } = struct_def {
                    // Create struct instance
                    let mut instance_fields = HashMap::new();
                    
                    // Initialize fields from the struct init
                    for (field_name, field_value_expr) in fields {
                        if !def_fields.contains(field_name) {
                            return Err(RuntimeError {
                                message: format!("Field '{}' not found in struct '{}'", field_name, struct_name),
                            });
                        }
                        let field_value = self.evaluate_expression(field_value_expr)?;
                        instance_fields.insert(field_name.clone(), field_value);
                    }
                    
                    // Initialize missing fields to nil
                    for field_name in &def_fields {
                        if !instance_fields.contains_key(field_name) {
                            instance_fields.insert(field_name.clone(), Value::Nil);
                        }
                    }
                    
                    Ok(Value::StructInstance {
                        struct_name: struct_name.clone(),
                        fields: instance_fields,
                    })
                } else {
                    Err(RuntimeError {
                        message: format!("'{}' is not a struct", struct_name),
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

                // Convert to integers (floor floats)
                let start_i = match start_val {
                    Value::Integer(n) => n,
                    Value::Float(f) => f.floor() as i64,
                    _ => return Err(RuntimeError { message: "Range start must be a number".to_string() }),
                };
                let end_i = match end_val {
                    Value::Integer(n) => n,
                    Value::Float(f) => f.floor() as i64,
                    _ => return Err(RuntimeError { message: "Range end must be a number".to_string() }),
                };
                let step_i = match step_val {
                    Value::Integer(n) => n,
                    Value::Float(f) => f.floor() as i64,
                    _ => return Err(RuntimeError { message: "Range step must be a number".to_string() }),
                };

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
        // First check if this is a builtin module (base conversion, etc.)
        if let Some(module_functions) = crate::stdlib::get_module(module_path) {
            // It's a builtin module - load the requested functions
            let module_env = if let Some(items) = items {
                // Selective import: import {item1, item2} from "module"
                let mut selected = HashMap::new();
                for item in items {
                    if let Some(value) = module_functions.get(item) {
                        selected.insert(item.clone(), value.clone());
                    } else {
                        return Err(RuntimeError {
                            message: format!("'{}' not found in module '{}'", item, module_path),
                        });
                    }
                }
                selected
            } else if let Some(alias_name) = alias {
                // Import with alias: import "module" as m
                let mut aliased = HashMap::new();
                aliased.insert(alias_name.clone(), Value::ModuleObject(module_functions));
                aliased
            } else {
                // Full import: import "module" - load all functions directly
                module_functions
            };

            // Add module functions to current environment
            for (name, value) in module_env {
                self.environment.define(name, value);
            }

            return Ok(());
        }

        // Not a builtin module - load from file
        // Get the directory where the executable is located for resolving stdlib paths
        let exe_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

        // Try to find project root (go up from target/release or target/debug)
        let project_root = exe_dir.parent()
            .and_then(|p| p.parent())
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| exe_dir.clone());

        // Get home directory for ~ expansion
        let home_dir = std::env::var("HOME")
            .ok()
            .map(|h| std::path::PathBuf::from(h));

        // Get current working directory
        let cwd = std::env::current_dir().unwrap_or_default();

        // Determine the module file path based on import style
        let module_file_path = if module_path.starts_with("~/") {
            // Home directory path: import "~/Documents/MyModule"
            if let Some(ref home) = home_dir {
                let path_without_tilde = module_path.trim_start_matches("~/");
                let path_with_ext = if path_without_tilde.ends_with(".ject") {
                    path_without_tilde.to_string()
                } else {
                    format!("{}.ject", path_without_tilde)
                };
                let full_path = home.join(&path_with_ext);
                if full_path.exists() {
                    full_path.to_string_lossy().to_string()
                } else {
                    return Err(RuntimeError {
                        message: format!("Module '{}' not found at {}", module_path, full_path.display()),
                    });
                }
            } else {
                return Err(RuntimeError {
                    message: "Could not determine home directory for '~' path".to_string(),
                });
            }
        } else if module_path.starts_with("/") {
            // Absolute path: import "/home/user/mymodule"
            let path_with_ext = if module_path.ends_with(".ject") {
                module_path.to_string()
            } else {
                format!("{}.ject", module_path)
            };
            let full_path = std::path::PathBuf::from(&path_with_ext);
            if full_path.exists() {
                full_path.to_string_lossy().to_string()
            } else {
                return Err(RuntimeError {
                    message: format!("Module '{}' not found at {}", module_path, full_path.display()),
                });
            }
        } else if module_path.starts_with("./") || module_path.starts_with("../") {
            // Relative path from current directory
            let path_with_ext = if module_path.ends_with(".ject") {
                module_path.to_string()
            } else {
                format!("{}.ject", module_path)
            };
            let full_path = cwd.join(&path_with_ext);
            if full_path.exists() {
                full_path.to_string_lossy().to_string()
            } else {
                return Err(RuntimeError {
                    message: format!("Module '{}' not found at {}", module_path, full_path.display()),
                });
            }
        } else if module_path.contains("/") {
            // Path relative to project root (e.g., "modules/math" or "lib/utils")
            let path_with_ext = if module_path.ends_with(".ject") {
                module_path.to_string()
            } else {
                format!("{}.ject", module_path)
            };
            let full_path = project_root.join(&path_with_ext);
            if full_path.exists() {
                full_path.to_string_lossy().to_string()
            } else {
                // Try cwd as fallback
                let cwd_path = cwd.join(&path_with_ext);
                if cwd_path.exists() {
                    cwd_path.to_string_lossy().to_string()
                } else {
                    return Err(RuntimeError {
                        message: format!("Module '{}' not found at {}", module_path, full_path.display()),
                    });
                }
            }
        } else {
            // Simple module name - check stdlib directory (relative to project root first, then cwd)
            let module_name = if module_path.ends_with(".ject") {
                module_path.trim_end_matches(".ject").to_string()
            } else {
                module_path.to_string()
            };
            
            // Try project root stdlib first
            let stdlib_path = project_root.join("stdlib").join(format!("{}.ject", module_name));
            if stdlib_path.exists() {
                stdlib_path.to_string_lossy().to_string()
            } else {
                // Try cwd stdlib
                let cwd_stdlib = cwd.join("stdlib").join(format!("{}.ject", module_name));
                if cwd_stdlib.exists() {
                    cwd_stdlib.to_string_lossy().to_string()
                } else {
                    return Err(RuntimeError {
                        message: format!("Module '{}' not found. Expected at 'stdlib/{}.ject' or provide a path.", module_path, module_name),
                    });
                }
            }
        };

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
        
        // Process export functions and define them in the module environment first
        // This ensures they're available in the module scope for potential self-references
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
                    // Create ModuleFunction with the current module environment as closure
                    // This captures all the module's variables and functions
                    let func = Value::ModuleFunction {
                        params: params.clone(),
                        body: body.clone(),
                        closure_env: self.environment.clone(),
                    };
                    exports.insert(name.clone(), func);
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
                    ControlFlow::Throw(error) => return Err(RuntimeError {
                        message: format!("Error in function: {}", error),
                    }),
                    ControlFlow::Break | ControlFlow::Continue => {
                        return Err(RuntimeError {
                            message: "break/continue in function".to_string(),
                        });
                    }
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
                    ControlFlow::Throw(error) => return Err(RuntimeError {
                        message: format!("Error in function: {}", error),
                    }),
                    ControlFlow::Break | ControlFlow::Continue => {
                        return Err(RuntimeError {
                            message: "break/continue in function".to_string(),
                        });
                    }
                    ControlFlow::None => Value::Nil,
                };
                
                self.environment.pop_scope();
                
                // Restore original environment
                self.environment = saved_env;
                Ok(result)
            }
            Value::Lambda { params, body, closure_env } => {
                // Save current environment and use closure environment
                let saved_env = std::mem::replace(&mut self.environment, closure_env.clone());
                
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
                
                let result = match &body {
                    crate::ast::LambdaBody::Expression(ref expr) => {
                        self.evaluate_expression(&expr)?
                    }
                    crate::ast::LambdaBody::Block(statements) => {
                        match self.execute_block(&statements)? {
                            ControlFlow::Return(value) => value,
                            ControlFlow::Throw(error) => return Err(RuntimeError {
                                message: format!("Error in lambda: {}", error),
                            }),
                            ControlFlow::Break | ControlFlow::Continue => {
                                return Err(RuntimeError {
                                    message: "break/continue in lambda".to_string(),
                                });
                            }
                            ControlFlow::None => Value::Nil,
                        }
                    }
                };

                self.environment.pop_scope();
                // Restore original environment
                self.environment = saved_env;
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

                // Try numpy functions first (they have np_ prefix)
                if name.starts_with("np_") {
                    crate::numpy::call_numpy_function(&name, arg_values)
                } else {
                    crate::stdlib::call_builtin_function(&name, arg_values)
                }
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

    fn evaluate_increment_decrement(&mut self, target: &Expr, prefix: bool, is_increment: bool) -> RuntimeResult<Value> {
        match target {
            Expr::Identifier(name) => {
                let current = self.environment.get(name)
                    .ok_or_else(|| RuntimeError { message: format!("Undefined variable '{}'", name) })?;
                let new_value = match current {
                    Value::Integer(n) => Value::Integer(if is_increment { n + 1 } else { n - 1 }),
                    Value::Float(f) => Value::Float(if is_increment { f + 1.0 } else { f - 1.0 }),
                    _ => return Err(RuntimeError { message: "Can only increment/decrement numbers".to_string() }),
                };
                self.environment.set(name, new_value.clone());
                Ok(if prefix { new_value } else { current })
            }
            _ => Err(RuntimeError { message: "Invalid increment/decrement target".to_string() }),
        }
    }

    fn call_higher_order_function(&mut self, func_name: &str, args: &[crate::ast::Argument]) -> RuntimeResult<Value> {
        match func_name {
            "map" => {
                if args.len() != 2 {
                    return Err(RuntimeError { message: "map() takes 2 arguments (array, function)".to_string() });
                }
                let array_val = match &args[0] {
                    crate::ast::Argument::Positional(expr) => self.evaluate_expression(expr)?,
                    _ => return Err(RuntimeError { message: "First argument must be positional".to_string() }),
                };
                let func_val = match &args[1] {
                    crate::ast::Argument::Positional(expr) => self.evaluate_expression(expr)?,
                    _ => return Err(RuntimeError { message: "Second argument must be positional".to_string() }),
                };

                // Handle both Array and UniqueArray
                let (arr, is_unique) = match array_val {
                    Value::Array(arr) => (arr, false),
                    Value::UniqueArray(arr) => (arr, true),
                    _ => return Err(RuntimeError { message: "map() requires an array or unique array".to_string() }),
                };

                if let Value::Lambda { params, body, closure_env } = func_val {
                    let mut result = Vec::new();
                    let mut seen = std::collections::HashSet::new();
                    
                    for item in &arr {
                        let saved_env = std::mem::replace(&mut self.environment, closure_env.clone());
                        self.environment.push_scope();
                        if !params.is_empty() {
                            self.environment.define(params[0].clone(), item.clone());
                        }
                        let mapped = match &body {
                            crate::ast::LambdaBody::Expression(ref expr) => self.evaluate_expression(&expr)?,
                            crate::ast::LambdaBody::Block(ref stmts) => {
                                match self.execute_block(&stmts)? {
                                    ControlFlow::Return(v) => v,
                                    ControlFlow::None => Value::Nil,
                                    _ => Value::Nil,
                                }
                            }
                        };
                        self.environment.pop_scope();
                        self.environment = saved_env;
                        
                        // For unique arrays, deduplicate results
                        if is_unique {
                            let key = mapped.to_string();
                            if !seen.contains(&key) {
                                seen.insert(key);
                                result.push(mapped);
                            }
                        } else {
                            result.push(mapped);
                        }
                    }
                    
                    // Return same type as input
                    if is_unique {
                        return Ok(Value::UniqueArray(result));
                    }
                    return Ok(Value::Array(result));
                }
                Err(RuntimeError { message: "map() requires array/unique_array and lambda".to_string() })
            }
            "filter" => {
                if args.len() != 2 {
                    return Err(RuntimeError { message: "filter() takes 2 arguments (array, function)".to_string() });
                }
                let array_val = match &args[0] {
                    crate::ast::Argument::Positional(expr) => self.evaluate_expression(expr)?,
                    _ => return Err(RuntimeError { message: "First argument must be positional".to_string() }),
                };
                let func_val = match &args[1] {
                    crate::ast::Argument::Positional(expr) => self.evaluate_expression(expr)?,
                    _ => return Err(RuntimeError { message: "Second argument must be positional".to_string() }),
                };

                // Handle both Array and UniqueArray
                let (arr, is_unique) = match array_val {
                    Value::Array(arr) => (arr, false),
                    Value::UniqueArray(arr) => (arr, true),
                    _ => return Err(RuntimeError { message: "filter() requires an array or unique array".to_string() }),
                };

                if let Value::Lambda { params, body, closure_env } = func_val {
                    let mut result = Vec::new();
                    let mut seen = std::collections::HashSet::new();
                    
                    for item in &arr {
                        let saved_env = std::mem::replace(&mut self.environment, closure_env.clone());
                        self.environment.push_scope();
                        if !params.is_empty() {
                            self.environment.define(params[0].clone(), item.clone());
                        }
                        let keep = match &body {
                            crate::ast::LambdaBody::Expression(ref expr) => self.evaluate_expression(&expr)?.is_truthy(),
                            crate::ast::LambdaBody::Block(ref stmts) => {
                                match self.execute_block(&stmts)? {
                                    ControlFlow::Return(v) => v.is_truthy(),
                                    ControlFlow::None => false,
                                    _ => false,
                                }
                            }
                        };
                        self.environment.pop_scope();
                        self.environment = saved_env;
                        
                        if keep {
                            // For unique arrays, deduplicate results
                            if is_unique {
                                let key = item.to_string();
                                if !seen.contains(&key) {
                                    seen.insert(key);
                                    result.push(item.clone());
                                }
                            } else {
                                result.push(item.clone());
                            }
                        }
                    }
                    
                    // Return same type as input
                    if is_unique {
                        return Ok(Value::UniqueArray(result));
                    }
                    return Ok(Value::Array(result));
                }
                Err(RuntimeError { message: "filter() requires array/unique_array and lambda".to_string() })
            }
            "reduce" => {
                if args.len() < 2 {
                    return Err(RuntimeError { message: "reduce() needs at least 2 arguments (array, function, [initial])".to_string() });
                }
                let array_val = match &args[0] {
                    crate::ast::Argument::Positional(expr) => self.evaluate_expression(expr)?,
                    _ => return Err(RuntimeError { message: "First argument must be positional".to_string() }),
                };
                let func_val = match &args[1] {
                    crate::ast::Argument::Positional(expr) => self.evaluate_expression(expr)?,
                    _ => return Err(RuntimeError { message: "Second argument must be positional".to_string() }),
                };
                let mut accumulator = if args.len() > 2 {
                    match &args[2] {
                        crate::ast::Argument::Positional(expr) => self.evaluate_expression(expr)?,
                        _ => Value::Nil,
                    }
                } else {
                    Value::Nil
                };
                
                if let Value::Array(arr) = array_val {
                    if let Value::Lambda { params, body, closure_env } = func_val {
                        for item in &arr {
                            let saved_env = std::mem::replace(&mut self.environment, closure_env.clone());
                            self.environment.push_scope();
                            if params.len() >= 2 {
                                self.environment.define(params[0].clone(), accumulator.clone());
                                self.environment.define(params[1].clone(), item.clone());
                            }
                            let new_acc = match &body {
                                crate::ast::LambdaBody::Expression(ref expr) => self.evaluate_expression(&expr)?,
                                crate::ast::LambdaBody::Block(ref stmts) => {
                                    match self.execute_block(&stmts)? {
                                        ControlFlow::Return(v) => v,
                                        ControlFlow::None => Value::Nil,
                                        _ => Value::Nil,
                                    }
                                }
                            };
                            self.environment.pop_scope();
                            self.environment = saved_env;
                            accumulator = new_acc;
                        }
                        return Ok(accumulator);
                    }
                }
                Err(RuntimeError { message: "reduce() requires array and lambda".to_string() })
            }
            _ => Err(RuntimeError { message: format!("Unknown function: {}", func_name) })
        }
    }
}
