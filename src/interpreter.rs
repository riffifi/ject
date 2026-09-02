use crate::ast::{Argument, BinaryOp, Expr, Stmt, UnaryOp};
use crate::lexer::InterpolationPart;
use crate::value::{Environment, Value};
use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::path::Path;

/// Get a helpful suggestion for runtime errors
pub fn get_runtime_suggestion(message: &str) -> String {
    let msg = message.to_lowercase();

    if msg.contains("undefined variable") {
        return "Check for typos, or declare the variable with 'let' first.".to_string();
    }
    if msg.contains("array index") {
        return "Use negative indices to count from the end (arr[-1] = last element).".to_string();
    }
    if msg.contains("break/continue") {
        return "break/continue can only affect the loop they're directly inside -- they can't jump out of a function, lambda, or match arm.".to_string();
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
    if msg.contains("division by zero") {
        return "Check the denominator isn't zero before dividing.".to_string();
    }
    if msg.contains("expected") && msg.contains("argument") {
        return "Check the number of arguments passed matches the function/lambda's parameters."
            .to_string();
    }
    if msg.contains("circular import") {
        return "One of these modules needs to stop importing the other, or move the shared code into a third module both of them import instead.".to_string();
    }
    if msg.contains("does not export") || msg.contains("not found in module") {
        return "Check the export name for typos, and that it's declared with 'export' in the module.".to_string();
    }
    if msg.contains("cannot access field") {
        return "This value doesn't have that field/member, and there's no function by that name to fall back to as arr.method(x) sugar for method(arr, x).".to_string();
    }
    if msg.contains("cannot call") {
        return "Only functions, lambdas, and builtins can be called -- check this isn't a plain value.".to_string();
    }
    if msg.contains("module") && msg.contains("not found") {
        return "Check the import path -- relative imports ('./x') resolve against the importing file's own directory.".to_string();
    }
    String::new()
}

#[derive(Debug)]
pub struct RuntimeError {
    pub message: String,
}

impl RuntimeError {
    fn with_frame(mut self, name: &str) -> Self {
        self.message.push_str("\n  at ");
        self.message.push_str(name);
        self
    }

    fn with_location(mut self, span: crate::diagnostic::SourceSpan) -> Self {
        if !self
            .message
            .lines()
            .any(|line| line.trim_start().starts_with("-->"))
        {
            self.message
                .push_str(&format!("\n  --> {}:{}", span.line, span.column));
        }
        self
    }

    fn with_source(mut self, source: Option<&str>) -> Self {
        let Some(source) = source.filter(|source| !source.starts_with("<embedded:")) else {
            return self;
        };
        let marker = "\n  --> ";
        if let Some(start) = self.message.find(marker) {
            let value_start = start + marker.len();
            let value_end = self.message[value_start..]
                .find('\n')
                .map(|offset| value_start + offset)
                .unwrap_or(self.message.len());
            let location = &self.message[value_start..value_end];
            if location.split(':').count() == 2 {
                self.message
                    .replace_range(value_start..value_end, &format!("{source}:{location}"));
            }
        }
        self
    }
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Runtime error: {}{}",
            self.message,
            get_runtime_suggestion(&self.message)
        )
    }
}

impl std::error::Error for RuntimeError {}

type RuntimeResult<T> = Result<T, RuntimeError>;

pub struct Interpreter {
    environment: Environment,
    // Canonical file paths of modules currently being loaded (innermost last), used to
    // detect circular imports with a clear error instead of infinite recursion.
    import_stack: Vec<String>,
    // Cache of already-loaded modules (canonical file path -> its exports), so importing
    // the same module twice doesn't re-parse and re-execute the file, and so module-level
    // state is a singleton across importers, like Python/JS modules.
    module_cache: HashMap<String, HashMap<String, Value>>,
    // Directory of the module file currently being loaded (innermost last). A relative
    // import (`./x`, `../x`) inside that module resolves against THIS, not the process's
    // current working directory -- otherwise a module's own internal imports break the
    // moment it's used from a different directory than the one it happened to be
    // authored/tested in, or is nested inside a subdirectory of a larger project.
    module_dir_stack: Vec<std::path::PathBuf>,
    // Package import name -> dependency entry file, prepared from Ject.toml.
    package_modules: HashMap<String, std::path::PathBuf>,
    current_package: Option<String>,
    // Checked periodically inside loop bodies. A host (e.g. the REPL) can share this
    // with a Ctrl+C handler to interrupt a runaway script -- e.g. an infinite `while
    // true do ... end` -- without killing the whole process. Not set by anything
    // unless a host explicitly wires it up via `set_interrupt_flag`; defaults to an
    // always-false flag that's never touched.
    interrupt_flag: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
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
            import_stack: Vec::new(),
            module_cache: HashMap::new(),
            module_dir_stack: vec![std::env::current_dir().unwrap_or_default()],
            package_modules: HashMap::new(),
            current_package: None,
            interrupt_flag: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }

    /// Shares an interrupt flag with this interpreter -- a host can set this (e.g.
    /// from a Ctrl+C handler) to make a running loop stop cleanly with a runtime
    /// error instead of hanging forever, without killing the process. The host is
    /// responsible for resetting the flag back to `false` after handling the
    /// resulting error, or every subsequent run would be interrupted immediately too.
    pub fn set_interrupt_flag(&mut self, flag: std::sync::Arc<std::sync::atomic::AtomicBool>) {
        self.interrupt_flag = flag;
    }

    fn check_interrupted(&self) -> RuntimeResult<()> {
        if self
            .interrupt_flag
            .load(std::sync::atomic::Ordering::Relaxed)
        {
            Err(RuntimeError {
                message: "Interrupted".to_string(),
            })
        } else {
            Ok(())
        }
    }

    /// Sets the base directory that top-level relative imports (in the entry script
    /// itself) resolve against. Call this with the entry script's own directory right
    /// after `new()`, before `interpret`, so `import "./x"` in the entry file resolves
    /// relative to wherever that file lives rather than the process's CWD.
    pub fn set_script_dir(&mut self, dir: std::path::PathBuf) {
        self.module_dir_stack[0] = dir.clone();
        if let Ok(project) = crate::package::discover(&dir) {
            self.current_package = Some(project.name.clone());
            for (alias, root) in &project.dependencies {
                if let Ok(dependency) = crate::package::load(root) {
                    self.package_modules
                        .insert(alias.clone(), dependency.entry.clone());
                }
            }
            if let Ok(dependencies) = crate::package::dependency_projects(&project) {
                for dependency in dependencies {
                    self.package_modules
                        .entry(dependency.name.clone())
                        .or_insert(dependency.entry);
                }
            }
        }
    }

    fn current_import_base(&self) -> std::path::PathBuf {
        self.module_dir_stack.last().cloned().unwrap_or_default()
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

    /// Same as `interpret`, but if the final statement is a bare expression (not an
    /// assignment, a `print`, etc.), its value is returned instead of being silently
    /// discarded -- this is what lets a REPL echo `2 + 2` -> `4` the way Python's does,
    /// without every function body needing to change how it behaves when run as a
    /// script.
    pub fn interpret_repl(&mut self, statements: &[Stmt]) -> RuntimeResult<Option<Value>> {
        let Some((last, rest)) = statements.split_last() else {
            return Ok(None);
        };

        for statement in rest {
            match self.execute_statement(statement)? {
                ControlFlow::Return(_) => return Ok(None),
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
                ControlFlow::None => {}
            }
        }

        if let Stmt::Expression(expr) = last {
            Ok(Some(self.evaluate_expression(expr)?))
        } else {
            match self.execute_statement(last)? {
                ControlFlow::Return(_) => Ok(None),
                ControlFlow::Throw(error) => Err(RuntimeError {
                    message: format!("Uncaught error: {}", error),
                }),
                ControlFlow::Break | ControlFlow::Continue => Err(RuntimeError {
                    message: "break/continue outside of loop".to_string(),
                }),
                ControlFlow::None => Ok(None),
            }
        }
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
                        if self.environment.set(name, val) {
                            Ok(ControlFlow::None)
                        } else {
                            Err(RuntimeError {
                                message: format!("Undefined variable '{}'", name),
                            })
                        }
                    }
                    crate::ast::AssignTarget::Index { object, index } => {
                        let obj = self.environment.get(object).ok_or_else(|| RuntimeError {
                            message: format!("Undefined variable '{}'", object),
                        })?;

                        let idx = self.evaluate_expression(index)?;

                        if let Value::Array(arr) = &obj {
                            if let Value::Integer(i) = idx {
                                let len = arr.borrow().len() as i64;
                                let actual_index = if i < 0 { len + i } else { i };
                                if actual_index < 0 || actual_index >= len {
                                    return Err(RuntimeError {
                                        message: format!("Array index out of bounds: {}", i),
                                    });
                                }
                                arr.borrow_mut()[actual_index as usize] = val;
                                Ok(ControlFlow::None)
                            } else {
                                Err(RuntimeError {
                                    message: "Array index must be integer".to_string(),
                                })
                            }
                        } else if let Value::Dictionary(dict) = obj {
                            if let Value::String(key) = idx {
                                dict.borrow_mut().insert(key, val);
                                Ok(ControlFlow::None)
                            } else {
                                Err(RuntimeError {
                                    message: "Dictionary key must be string".to_string(),
                                })
                            }
                        } else {
                            Err(RuntimeError {
                                message: format!("Cannot index into {}", obj.type_name()),
                            })
                        }
                    }
                    crate::ast::AssignTarget::IndexChain { object, indices } => {
                        self.assign_index_chain(object, indices.as_slice(), val)?;
                        Ok(ControlFlow::None)
                    }
                    crate::ast::AssignTarget::Field { object, field } => {
                        let obj = self.environment.get(object).ok_or_else(|| RuntimeError {
                            message: format!("Undefined variable '{}'", object),
                        })?;

                        if let Value::Dictionary(dict) = obj {
                            dict.borrow_mut().insert(field.clone(), val);
                            Ok(ControlFlow::None)
                        } else if let Value::StructInstance {
                            struct_name,
                            mut fields,
                        } = obj
                        {
                            fields.insert(field.clone(), val);
                            self.environment.set(
                                object,
                                Value::StructInstance {
                                    struct_name,
                                    fields,
                                },
                            );
                            Ok(ControlFlow::None)
                        } else {
                            Err(RuntimeError {
                                message: format!("Cannot assign field on {}", obj.type_name()),
                            })
                        }
                    }
                }
            }
            Stmt::Function { name, params, body } => {
                let func = Value::Function {
                    name: name.clone(),
                    source: self.import_stack.last().cloned(),
                    params: params.clone(),
                    body: body.clone(),
                    closure_env: self.environment.clone(),
                };
                self.environment.define(name.clone(), func);
                Ok(ControlFlow::None)
            }
            Stmt::If {
                condition,
                then_branch,
                elseif_branches,
                else_branch,
            } => {
                let cond_value = self.evaluate_expression(condition)?;

                if cond_value.is_truthy() {
                    self.execute_block(then_branch)
                } else {
                    // Check elseif conditions
                    for elseif_branch in elseif_branches {
                        let elseif_cond_value =
                            self.evaluate_expression(&elseif_branch.condition)?;
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
                    self.check_interrupted()?;
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
            Stmt::For {
                var,
                iterable,
                body,
            } => {
                let iter_value = self.evaluate_expression(iterable)?;

                match iter_value {
                    Value::Array(elements) => {
                        let elements = elements.borrow().clone();
                        for element in elements {
                            self.check_interrupted()?;
                            self.environment.push_scope();
                            self.environment.define(var.clone(), element);

                            let block_result = self.execute_block(body);
                            self.environment.pop_scope();

                            match block_result? {
                                ControlFlow::Return(value) => {
                                    return Ok(ControlFlow::Return(value));
                                }
                                ControlFlow::Throw(error) => {
                                    return Ok(ControlFlow::Throw(error));
                                }
                                ControlFlow::Break => {
                                    break;
                                }
                                ControlFlow::Continue => {
                                    continue;
                                }
                                ControlFlow::None => {}
                            }
                        }
                    }
                    Value::String(s) => {
                        // Iterate over characters in string
                        for ch in s.chars() {
                            self.check_interrupted()?;
                            self.environment.push_scope();
                            self.environment
                                .define(var.clone(), Value::String(ch.to_string()));

                            let block_result = self.execute_block(body);
                            self.environment.pop_scope();

                            match block_result? {
                                ControlFlow::Return(value) => {
                                    return Ok(ControlFlow::Return(value));
                                }
                                ControlFlow::Throw(error) => {
                                    return Ok(ControlFlow::Throw(error));
                                }
                                ControlFlow::Break => {
                                    break;
                                }
                                ControlFlow::Continue => {
                                    continue;
                                }
                                ControlFlow::None => {}
                            }
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
            Stmt::Import {
                module_path,
                items,
                alias,
            } => {
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
                    name: name.clone(),
                    source: self.import_stack.last().cloned(),
                    params: params.clone(),
                    body: body.clone(),
                    closure_env: self.environment.clone(),
                };
                self.environment.define(name.clone(), func);
                Ok(ControlFlow::None)
            }
            Stmt::Print { values, sep, end } => {
                // Evaluate all values
                let mut output = Vec::new();
                for value_expr in values {
                    let value = self.evaluate_expression(value_expr)?;
                    output.push(value.display()); // Use display() for print (no quotes on strings)
                }

                // Determine separator (default: space)
                let separator = if let Some(sep_expr) = sep {
                    self.evaluate_expression(sep_expr)?.display()
                } else {
                    " ".to_string()
                };

                // Determine end (default: newline)
                let ending = if let Some(end_expr) = end {
                    self.evaluate_expression(end_expr)?.display()
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
            Stmt::Try {
                body,
                catch_var,
                catch_body,
            } => {
                let try_result = self.execute_block(body);
                match try_result {
                    Ok(ControlFlow::Throw(error_value)) => {
                        // Catch the error
                        self.environment.push_scope();
                        if let Some(var_name) = catch_var {
                            // Store the error value directly (it can be any value)
                            self.environment
                                .define(var_name.clone(), error_value.clone());
                        }
                        let result = self.execute_block(catch_body);
                        self.environment.pop_scope();
                        Ok(result?)
                    }
                    Ok(other) => Ok(other),
                    Err(e) => {
                        // Runtime error from function call - convert to throw
                        self.environment.push_scope();
                        if let Some(var_name) = catch_var {
                            self.environment
                                .define(var_name.clone(), Value::String(e.message.clone()));
                        }
                        let result = self.execute_block(catch_body);
                        self.environment.pop_scope();
                        Ok(result?)
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
            Expr::Located { expression, span } => self
                .evaluate_expression(expression)
                .map_err(|error| error.with_location(*span)),
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
                            let tokens: Vec<crate::lexer::Token> =
                                located_tokens.into_iter().map(|lt| lt.token).collect();
                            let mut parser = crate::parser::Parser::new_simple(tokens);

                            match parser.parse() {
                                Ok(statements) => {
                                    if let Some(stmt) = statements.first() {
                                        if let crate::ast::Stmt::Expression(expr) = stmt {
                                            let value = self.evaluate_expression(expr)?;
                                            result.push_str(&value.display());
                                        } else {
                                            return Err(RuntimeError {
                                                message:
                                                    "Invalid expression in string interpolation"
                                                        .to_string(),
                                            });
                                        }
                                    }
                                }
                                Err(e) => {
                                    return Err(RuntimeError {
                                        message: format!(
                                            "Parse error in string interpolation: {}",
                                            e
                                        ),
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
            Expr::Identifier(name) => self.environment.get(name).ok_or_else(|| RuntimeError {
                message: format!("Undefined variable '{}'.", name),
            }),
            Expr::Binary {
                left,
                operator,
                right,
            } => {
                let left_val = self.evaluate_expression(left)?;

                // Short-circuit: don't evaluate (or execute the side effects of) the
                // right operand when the left operand alone already determines the result.
                match operator {
                    BinaryOp::And if !left_val.is_truthy() => return Ok(left_val),
                    BinaryOp::Or if left_val.is_truthy() => return Ok(left_val),
                    _ => {}
                }

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
                // Parser compatibility: some nested-call parses are re-associated so that
                // the inner call appears under `callee` and the original callee identifier
                // becomes the single positional argument.
                //
                // Reconstruct the canonical AST shape to preserve runtime semantics.
                if let (
                    Expr::Call { .. },
                    [crate::ast::Argument::Positional(Expr::Identifier(name))],
                ) = (callee.as_ref(), args.as_slice())
                {
                    let rebuilt = Expr::Call {
                        callee: Box::new(Expr::Identifier(name.clone())),
                        args: vec![crate::ast::Argument::Positional((**callee).clone())],
                    };
                    return self.evaluate_expression(&rebuilt);
                }

                // Check for higher-order functions that need special handling
                if let Expr::Identifier(func_name) = &**callee {
                    if func_name == "map"
                        || func_name == "filter"
                        || func_name == "reduce"
                        || func_name == "any"
                        || func_name == "all"
                    {
                        return self.call_higher_order_function(func_name, args);
                    }
                }

                // `obj.field(args)`: if `field` is a genuine member of `obj` (a struct
                // field, dictionary key, or module export), call that -- this is what
                // makes documented patterns like `import "math" as m; m.log(x, base)`
                // work. Otherwise (obj is a primitive with no member concept at all --
                // an array, string, number, etc. -- or a struct/dict without that
                // field), `field` is treated as method-call sugar for a free function:
                // `arr.map(f)` is exactly `map(arr, f)`. Member access always wins over
                // sugar, so a real member is never shadowed by a same-named function.
                if let Expr::StructAccess { object, field } = &**callee {
                    let obj_val = self.evaluate_expression(object)?;
                    let member = match &obj_val {
                        Value::StructInstance { fields, .. } => fields.get(field).cloned(),
                        Value::Dictionary(dict) => dict.borrow().get(field).cloned(),
                        Value::ModuleObject(exports) => exports.get(field).cloned(),
                        _ => None,
                    };
                    if let Some(member_val) = member {
                        return self.call_function(member_val, args);
                    }

                    // Evaluate the remaining (non-receiver) arguments once; obj_val is
                    // already evaluated, so it's used directly rather than re-evaluating
                    // the receiver expression a second time.
                    let mut rest_values = Vec::with_capacity(args.len());
                    for arg in args {
                        match arg {
                            crate::ast::Argument::Positional(expr) => {
                                rest_values.push(self.evaluate_expression(expr)?)
                            }
                            crate::ast::Argument::Keyword { .. } => {
                                return Err(RuntimeError {
                                    message: format!(
                                        "{}() does not support keyword arguments",
                                        field
                                    ),
                                });
                            }
                        }
                    }

                    if field == "map"
                        || field == "filter"
                        || field == "reduce"
                        || field == "any"
                        || field == "all"
                    {
                        let mut values = vec![obj_val];
                        values.extend(rest_values);
                        return self.call_higher_order_with_values(field, values);
                    }
                    if let Some(
                        free_fn @ (Value::BuiltinFunction(_)
                        | Value::Function { .. }
                        | Value::Lambda { .. }
                        | Value::ModuleFunction { .. }),
                    ) = self.environment.get(field)
                    {
                        let mut values = vec![obj_val];
                        values.extend(rest_values);
                        return self.invoke_callable(&free_fn, values);
                    }

                    return Err(RuntimeError {
                        message: format!(
                            "Cannot access field '{}' on {}",
                            field,
                            obj_val.type_name()
                        ),
                    });
                }

                let func = self.evaluate_expression(callee)?;

                self.call_function(func, args)
            }
            Expr::Array(elements) => {
                let mut values = Vec::new();
                for element in elements {
                    values.push(self.evaluate_expression(element)?);
                }
                Ok(Value::array(values))
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
            Expr::ListComprehension {
                expr,
                var,
                iterable,
                condition,
            } => {
                // Evaluate the iterable
                let iter_value = self.evaluate_expression(iterable)?;

                // Get elements from iterable (array, unique array, string chars, or range)
                let elements = match iter_value {
                    Value::Array(arr) => arr.borrow().clone(),
                    Value::UniqueArray(arr) => arr,
                    Value::String(s) => s.chars().map(|c| Value::String(c.to_string())).collect(),
                    _ => {
                        return Err(RuntimeError {
                            message: "Can only iterate over arrays, strings, or ranges".to_string(),
                        })
                    }
                };

                let mut result = Vec::new();

                for item in &elements {
                    // Create new scope for loop variable
                    self.environment.push_scope();
                    self.environment.define(var.clone(), item.clone());

                    let outcome: RuntimeResult<Option<Value>> = (|| {
                        let include = if let Some(cond) = condition {
                            self.evaluate_expression(cond)?.is_truthy()
                        } else {
                            true
                        };
                        if include {
                            Ok(Some(self.evaluate_expression(expr)?))
                        } else {
                            Ok(None)
                        }
                    })();

                    self.environment.pop_scope();

                    if let Some(value) = outcome? {
                        result.push(value);
                    }
                }

                Ok(Value::array(result))
            }
            Expr::Generator {
                expr,
                var,
                iterable,
                condition,
            } => {
                // For now, generators evaluate eagerly like list comprehensions
                // In the future, this could return a lazy iterator
                let iter_value = self.evaluate_expression(iterable)?;

                let elements = match iter_value {
                    Value::Array(arr) => arr.borrow().clone(),
                    Value::UniqueArray(arr) => arr,
                    Value::String(s) => s.chars().map(|c| Value::String(c.to_string())).collect(),
                    _ => {
                        return Err(RuntimeError {
                            message: "Can only iterate over arrays, strings, or ranges".to_string(),
                        })
                    }
                };

                let mut result = Vec::new();

                for item in &elements {
                    self.environment.push_scope();
                    self.environment.define(var.clone(), item.clone());

                    let outcome: RuntimeResult<Option<Value>> = (|| {
                        let include = if let Some(cond) = condition {
                            self.evaluate_expression(cond)?.is_truthy()
                        } else {
                            true
                        };
                        if include {
                            Ok(Some(self.evaluate_expression(expr)?))
                        } else {
                            Ok(None)
                        }
                    })();

                    self.environment.pop_scope();

                    if let Some(value) = outcome? {
                        result.push(value);
                    }
                }

                // Return as array for now (could be made lazy in future)
                Ok(Value::array(result))
            }
            Expr::Dictionary(pairs) => {
                let mut map = std::collections::HashMap::new();
                for (key, value_expr) in pairs {
                    let value = self.evaluate_expression(value_expr)?;
                    map.insert(key.clone(), value);
                }
                Ok(Value::dictionary(map))
            }
            Expr::Index { object, index } => {
                // Flatten the canonical left-associated AST, then apply indices
                // in the same left-to-right order as the source.
                let mut indices: Vec<Expr> = Vec::new();
                let mut base: &Expr = object.as_ref();
                indices.push((**index).clone());

                while let Expr::Index {
                    object: inner_obj,
                    index: inner_idx,
                } = base
                {
                    indices.push((**inner_idx).clone());
                    base = inner_obj.as_ref();
                }
                indices.reverse();

                let mut current = self.evaluate_expression(base)?;
                for idx_expr in indices {
                    let idx = self.evaluate_expression(&idx_expr)?;
                    current = match (current, idx) {
                        (Value::Array(arr), Value::Integer(i)) => {
                            let arr = arr.borrow();
                            // Handle negative indices
                            let actual_index = if i < 0 {
                                (arr.len() as i64 + i) as usize
                            } else {
                                i as usize
                            };

                            if actual_index < arr.len() {
                                arr[actual_index].clone()
                            } else {
                                return Err(RuntimeError {
                                    message: format!("Array index out of bounds: {}", i),
                                });
                            }
                        }
                        (Value::Dictionary(dict), Value::String(key)) => {
                            dict.borrow().get(&key).cloned().unwrap_or(Value::Nil)
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
                                Value::String(chars[actual_index].to_string())
                            } else {
                                return Err(RuntimeError {
                                    message: format!("String index out of bounds: {}", i),
                                });
                            }
                        }
                        (obj, idx) => {
                            return Err(RuntimeError {
                                message: format!(
                                    "Cannot index {} with {}",
                                    obj.type_name(),
                                    idx.type_name()
                                ),
                            });
                        }
                    };
                }

                Ok(current)
            }
            Expr::Slice {
                object,
                from,
                to,
                step,
            } => {
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
                    Value::Integer(1) // Default step is 1
                };

                // Convert step to integer
                let step_i = match step_val {
                    Value::Integer(n) => n,
                    Value::Float(f) => f.floor() as i64,
                    _ => {
                        return Err(RuntimeError {
                            message: "Slice step must be a number".to_string(),
                        })
                    }
                };

                if step_i == 0 {
                    return Err(RuntimeError {
                        message: "Slice step cannot be zero".to_string(),
                    });
                }

                // Perform slicing based on object type
                match obj {
                    Value::Array(arr) => {
                        let arr = arr.borrow();
                        let len = arr.len() as i64;

                        // Convert from/to to actual indices
                        let from_i = match from_val {
                            Some(Value::Integer(n)) => {
                                if n < 0 {
                                    len + n
                                } else {
                                    n
                                }
                            }
                            None => {
                                if step_i > 0 {
                                    0
                                } else {
                                    len - 1
                                }
                            }
                            _ => {
                                return Err(RuntimeError {
                                    message: "Slice 'from' must be an integer".to_string(),
                                })
                            }
                        };

                        let to_i = match to_val {
                            Some(Value::Integer(n)) => {
                                if n < 0 {
                                    len + n
                                } else {
                                    n
                                }
                            }
                            None => {
                                if step_i > 0 {
                                    len
                                } else {
                                    -1
                                }
                            }
                            _ => {
                                return Err(RuntimeError {
                                    message: "Slice 'to' must be an integer".to_string(),
                                })
                            }
                        };

                        // Normalize indices (but preserve -1 for reverse slice end)
                        let from_i = if from_i < 0 {
                            0
                        } else if from_i > len {
                            len
                        } else {
                            from_i
                        };
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
                                if current == 0 {
                                    break;
                                }
                                current += step_i;
                            }
                        }

                        Ok(Value::array(result))
                    }
                    Value::String(s) => {
                        let chars: Vec<char> = s.chars().collect();
                        let len = chars.len() as i64;

                        // Convert from/to to actual indices
                        let from_i = match from_val {
                            Some(Value::Integer(n)) => {
                                if n < 0 {
                                    len + n
                                } else {
                                    n
                                }
                            }
                            None => {
                                if step_i > 0 {
                                    0
                                } else {
                                    len - 1
                                }
                            }
                            _ => {
                                return Err(RuntimeError {
                                    message: "Slice 'from' must be an integer".to_string(),
                                })
                            }
                        };

                        let to_i = match to_val {
                            Some(Value::Integer(n)) => {
                                if n < 0 {
                                    len + n
                                } else {
                                    n
                                }
                            }
                            None => {
                                if step_i > 0 {
                                    len
                                } else {
                                    -1
                                }
                            }
                            _ => {
                                return Err(RuntimeError {
                                    message: "Slice 'to' must be an integer".to_string(),
                                })
                            }
                        };

                        // Normalize indices (but preserve -1 for reverse slice end)
                        let from_i = if from_i < 0 {
                            0
                        } else if from_i > len {
                            len
                        } else {
                            from_i
                        };
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
                                if current == 0 {
                                    break;
                                }
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
                        message: format!(
                            "Cannot access property '{}' on {}",
                            property,
                            obj.type_name()
                        ),
                    }),
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
                    Value::Dictionary(dict) => {
                        // Convenience: allow `dict.key` as sugar for `dict["key"]`
                        Ok(dict.borrow().get(field).cloned().unwrap_or(Value::Nil))
                    }
                    Value::ModuleObject(exports) => {
                        // Also support module member access via dot notation
                        exports.get(field).cloned().ok_or_else(|| RuntimeError {
                            message: format!("Property '{}' not found in module", field),
                        })
                    }
                    _ => Err(RuntimeError {
                        message: format!("Cannot access field '{}' on {}", field, obj.type_name()),
                    }),
                }
            }
            Expr::StructInit {
                struct_name,
                fields,
            } => {
                // Get struct definition
                let struct_def = self
                    .environment
                    .get(struct_name)
                    .ok_or_else(|| RuntimeError {
                        message: format!("Struct '{}' not defined", struct_name),
                    })?;

                if let Value::StructDefinition {
                    fields: def_fields, ..
                } = struct_def
                {
                    // Create struct instance
                    let mut instance_fields = HashMap::new();

                    // Initialize fields from the struct init
                    for (field_name, field_value_expr) in fields {
                        if !def_fields.contains(field_name) {
                            return Err(RuntimeError {
                                message: format!(
                                    "Field '{}' not found in struct '{}'",
                                    field_name, struct_name
                                ),
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
                    Value::Integer(1) // Default step is 1
                };

                // Convert to integers (floor floats)
                let start_i = match start_val {
                    Value::Integer(n) => n,
                    Value::Float(f) => f.floor() as i64,
                    _ => {
                        return Err(RuntimeError {
                            message: "Range start must be a number".to_string(),
                        })
                    }
                };
                let end_i = match end_val {
                    Value::Integer(n) => n,
                    Value::Float(f) => f.floor() as i64,
                    _ => {
                        return Err(RuntimeError {
                            message: "Range end must be a number".to_string(),
                        })
                    }
                };
                let step_i = match step_val {
                    Value::Integer(n) => n,
                    Value::Float(f) => f.floor() as i64,
                    _ => {
                        return Err(RuntimeError {
                            message: "Range step must be a number".to_string(),
                        })
                    }
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

                Ok(Value::array(result))
            }
            Expr::ConditionalExpr {
                condition,
                then_expr,
                elseif_branches,
                else_expr,
            } => {
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
                    let mut matched = false;
                    let mut bound_name: Option<&String> = None;
                    for pattern in &arm.patterns {
                        if self.pattern_matches(pattern, &match_value)? {
                            matched = true;
                            if let crate::ast::Pattern::Identifier(name) = pattern {
                                bound_name = Some(name);
                            }
                            break;
                        }
                    }
                    if matched {
                        self.environment.push_scope();
                        if let Some(name) = bound_name {
                            self.environment.define(name.clone(), match_value.clone());
                        }
                        let result = self.evaluate_match_arm_body(&arm.body);
                        self.environment.pop_scope();
                        return result;
                    }
                }

                Err(RuntimeError {
                    message: "No matching pattern found in match expression".to_string(),
                })
            }
            Expr::Print { values, sep, end } => {
                // Expression form of print: perform side effects, return nil.
                let mut output = Vec::new();
                for v in values {
                    let value = self.evaluate_expression(v)?;
                    output.push(value.display());
                }

                let separator = if let Some(sep) = sep {
                    self.evaluate_expression(sep)?.display()
                } else {
                    " ".to_string()
                };

                let ending = if let Some(end) = end {
                    self.evaluate_expression(end)?.display()
                } else {
                    "\n".to_string()
                };

                print!("{}", output.join(&separator));
                print!("{}", ending);
                Ok(Value::Nil)
            }
        }
    }

    fn evaluate_binary_op(
        &self,
        left: &Value,
        op: &BinaryOp,
        right: &Value,
    ) -> RuntimeResult<Value> {
        match (left, op, right) {
            // Arithmetic
            (Value::Integer(a), BinaryOp::Add, Value::Integer(b)) => Ok(Value::Integer(a + b)),
            (Value::Float(a), BinaryOp::Add, Value::Float(b)) => Ok(Value::Float(a + b)),
            (Value::Integer(a), BinaryOp::Add, Value::Float(b)) => Ok(Value::Float(*a as f64 + b)),
            (Value::Float(a), BinaryOp::Add, Value::Integer(b)) => Ok(Value::Float(a + *b as f64)),
            (Value::String(a), BinaryOp::Add, Value::String(b)) => {
                Ok(Value::String(format!("{}{}", a, b)))
            }
            (Value::String(a), BinaryOp::Add, b) => Ok(Value::String(format!("{}{}", a, b))),
            (a, BinaryOp::Add, Value::String(b)) => Ok(Value::String(format!("{}{}", a, b))),
            (Value::Array(a), BinaryOp::Add, Value::Array(b)) => {
                let mut result = a.borrow().clone();
                result.extend(b.borrow().iter().cloned());
                Ok(Value::array(result))
            }

            (Value::Integer(a), BinaryOp::Subtract, Value::Integer(b)) => Ok(Value::Integer(a - b)),
            (Value::Float(a), BinaryOp::Subtract, Value::Float(b)) => Ok(Value::Float(a - b)),
            (Value::Integer(a), BinaryOp::Subtract, Value::Float(b)) => {
                Ok(Value::Float(*a as f64 - b))
            }
            (Value::Float(a), BinaryOp::Subtract, Value::Integer(b)) => {
                Ok(Value::Float(a - *b as f64))
            }

            (Value::Integer(a), BinaryOp::Multiply, Value::Integer(b)) => Ok(Value::Integer(a * b)),
            (Value::Float(a), BinaryOp::Multiply, Value::Float(b)) => Ok(Value::Float(a * b)),
            (Value::Integer(a), BinaryOp::Multiply, Value::Float(b)) => {
                Ok(Value::Float(*a as f64 * b))
            }
            (Value::Float(a), BinaryOp::Multiply, Value::Integer(b)) => {
                Ok(Value::Float(a * *b as f64))
            }

            (Value::Integer(a), BinaryOp::Divide, Value::Integer(b)) => {
                if *b == 0 {
                    Err(RuntimeError {
                        message: "Division by zero".to_string(),
                    })
                } else {
                    Ok(Value::Float(*a as f64 / *b as f64))
                }
            }
            (Value::Float(a), BinaryOp::Divide, Value::Float(b)) => {
                if *b == 0.0 {
                    Err(RuntimeError {
                        message: "Division by zero".to_string(),
                    })
                } else {
                    Ok(Value::Float(a / b))
                }
            }
            (Value::Integer(a), BinaryOp::Divide, Value::Float(b)) => {
                if *b == 0.0 {
                    Err(RuntimeError {
                        message: "Division by zero".to_string(),
                    })
                } else {
                    Ok(Value::Float(*a as f64 / b))
                }
            }
            (Value::Float(a), BinaryOp::Divide, Value::Integer(b)) => {
                if *b == 0 {
                    Err(RuntimeError {
                        message: "Division by zero".to_string(),
                    })
                } else {
                    Ok(Value::Float(a / *b as f64))
                }
            }

            (Value::Integer(a), BinaryOp::Modulo, Value::Integer(b)) => {
                if *b == 0 {
                    Err(RuntimeError {
                        message: "Modulo by zero".to_string(),
                    })
                } else {
                    Ok(Value::Integer(a % b))
                }
            }
            (Value::Float(a), BinaryOp::Modulo, Value::Float(b)) => {
                if *b == 0.0 {
                    Err(RuntimeError {
                        message: "Modulo by zero".to_string(),
                    })
                } else {
                    Ok(Value::Float(a % b))
                }
            }
            (Value::Integer(a), BinaryOp::Modulo, Value::Float(b)) => {
                if *b == 0.0 {
                    Err(RuntimeError {
                        message: "Modulo by zero".to_string(),
                    })
                } else {
                    Ok(Value::Float((*a as f64) % b))
                }
            }
            (Value::Float(a), BinaryOp::Modulo, Value::Integer(b)) => {
                if *b == 0 {
                    Err(RuntimeError {
                        message: "Modulo by zero".to_string(),
                    })
                } else {
                    Ok(Value::Float(a % (*b as f64)))
                }
            }

            // Comparison
            (Value::Integer(a), BinaryOp::Equal, Value::Integer(b)) => Ok(Value::Bool(a == b)),
            (Value::Float(a), BinaryOp::Equal, Value::Float(b)) => Ok(Value::Bool(a == b)),
            (Value::Integer(a), BinaryOp::Equal, Value::Float(b)) => {
                Ok(Value::Bool(*a as f64 == *b))
            }
            (Value::Float(a), BinaryOp::Equal, Value::Integer(b)) => {
                Ok(Value::Bool(*a == *b as f64))
            }
            (Value::String(a), BinaryOp::Equal, Value::String(b)) => Ok(Value::Bool(a == b)),
            (Value::Bool(a), BinaryOp::Equal, Value::Bool(b)) => Ok(Value::Bool(a == b)),
            (Value::Array(a), BinaryOp::Equal, Value::Array(b)) => Ok(Value::Bool(a == b)),
            (Value::Collection(a), BinaryOp::Equal, Value::Collection(b)) => {
                Ok(Value::Bool(a == b))
            }
            (Value::Nil, BinaryOp::Equal, Value::Nil) => Ok(Value::Bool(true)),
            (Value::Dictionary(_), BinaryOp::Equal, Value::Dictionary(_)) => {
                Ok(Value::Bool(left == right))
            }
            (Value::UniqueArray(_), BinaryOp::Equal, Value::UniqueArray(_)) => {
                Ok(Value::Bool(left == right))
            }
            (Value::StructInstance { .. }, BinaryOp::Equal, Value::StructInstance { .. }) => {
                Ok(Value::Bool(left == right))
            }
            (Value::Native(_), BinaryOp::Equal, Value::Native(_)) => Ok(Value::Bool(left == right)),
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
            (Value::Integer(a), BinaryOp::Less, Value::Float(b)) => {
                Ok(Value::Bool((*a as f64) < *b))
            }
            (Value::Float(a), BinaryOp::Less, Value::Integer(b)) => {
                Ok(Value::Bool(*a < (*b as f64)))
            }

            (Value::Integer(a), BinaryOp::Greater, Value::Integer(b)) => Ok(Value::Bool(a > b)),
            (Value::Float(a), BinaryOp::Greater, Value::Float(b)) => Ok(Value::Bool(a > b)),
            (Value::Integer(a), BinaryOp::Greater, Value::Float(b)) => {
                Ok(Value::Bool((*a as f64) > *b))
            }
            (Value::Float(a), BinaryOp::Greater, Value::Integer(b)) => {
                Ok(Value::Bool(*a > (*b as f64)))
            }

            (Value::Integer(a), BinaryOp::LessEqual, Value::Integer(b)) => Ok(Value::Bool(a <= b)),
            (Value::Float(a), BinaryOp::LessEqual, Value::Float(b)) => Ok(Value::Bool(a <= b)),
            (Value::Integer(a), BinaryOp::LessEqual, Value::Float(b)) => {
                Ok(Value::Bool((*a as f64) <= *b))
            }
            (Value::Float(a), BinaryOp::LessEqual, Value::Integer(b)) => {
                Ok(Value::Bool(*a <= (*b as f64)))
            }

            (Value::Integer(a), BinaryOp::GreaterEqual, Value::Integer(b)) => {
                Ok(Value::Bool(a >= b))
            }
            (Value::Float(a), BinaryOp::GreaterEqual, Value::Float(b)) => Ok(Value::Bool(a >= b)),
            (Value::Integer(a), BinaryOp::GreaterEqual, Value::Float(b)) => {
                Ok(Value::Bool((*a as f64) >= *b))
            }
            (Value::Float(a), BinaryOp::GreaterEqual, Value::Integer(b)) => {
                Ok(Value::Bool(*a >= (*b as f64)))
            }

            // In operator - check if left value is contained in right value
            (left_val, BinaryOp::In, Value::Array(arr)) => {
                for item in arr.borrow().iter() {
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
                Ok(Value::Bool(dict.borrow().contains_key(key)))
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

            (left, op, right) => {
                let mut msg = format!(
                    "Unsupported operation: {} {} {}",
                    left.type_name(),
                    op,
                    right.type_name()
                );
                if matches!(op, BinaryOp::Modulo) {
                    msg.push_str(" (tip: `%` supports integer and float numbers only)");
                }
                Err(RuntimeError { message: msg })
            }
        }
    }

    fn evaluate_unary_op(&self, op: &UnaryOp, operand: &Value) -> RuntimeResult<Value> {
        match (op, operand) {
            (UnaryOp::Negate, Value::Integer(n)) => Ok(Value::Integer(-n)),
            (UnaryOp::Negate, Value::Float(f)) => Ok(Value::Float(-f)),
            (UnaryOp::Not, val) => Ok(Value::Bool(!val.is_truthy())),
            (op, operand) => Err(RuntimeError {
                message: format!(
                    "Unsupported unary operation: {} {}",
                    op,
                    operand.type_name()
                ),
            }),
        }
    }

    fn load_module(
        &mut self,
        module_path: &str,
        items: &Option<Vec<String>>,
        alias: &Option<String>,
    ) -> RuntimeResult<()> {
        if let Some(backend) = module_path.strip_prefix("@native/") {
            let own_backend = self.current_package.as_deref() == Some(backend);
            let embedded_facade = self
                .import_stack
                .last()
                .is_some_and(|path| path == &format!("<embedded:{backend}>"));
            let bundled_facade = self.import_stack.last().is_some_and(|path| {
                Path::new(path).file_stem().and_then(|stem| stem.to_str()) == Some(backend)
                    && Path::new(path)
                        .parent()
                        .and_then(|parent| parent.file_name())
                        .and_then(|name| name.to_str())
                        == Some("stdlib")
            });
            let dependency_facade = self.package_modules.get(backend).is_some_and(|entry| {
                let entry = entry
                    .canonicalize()
                    .unwrap_or_else(|_| entry.clone())
                    .to_string_lossy()
                    .to_string();
                self.import_stack.last() == Some(&entry)
            });
            if !own_backend && !embedded_facade && !bundled_facade && !dependency_facade {
                return Err(RuntimeError {
                    message: format!(
                        "native backend '{backend}' is private; import its public package instead"
                    ),
                });
            }
        }

        // Private native backends and the public `base` primitive module.
        if crate::stdlib::is_native_only_module(module_path) {
            let Some(module_functions) = crate::stdlib::get_module(module_path) else {
                return Err(RuntimeError {
                    message: format!(
                        "Internal error: native module '{}' missing exports",
                        module_path
                    ),
                });
            };
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

        // Standard-library, package, and path imports share the same canonical
        // resolver used by diagnostics and editor tooling.
        let resolved =
            crate::module_resolver::ModuleResolver::for_path(&self.current_import_base())
                .resolve(module_path)
                .map_err(|error| RuntimeError {
                    message: error.to_string(),
                })?;
        let module_file_path = resolved.identity.cache_key();
        let module_dir = resolved.directory;
        let module_content = resolved.source;

        // Already loaded this exact module before: reuse its exports instead of
        // re-parsing and re-executing the file. This also makes a module's top-level
        // state a singleton across every place that imports it (as in Python/JS),
        // rather than each importer getting a fresh, independent copy.
        if let Some(cached_exports) = self.module_cache.get(&module_file_path) {
            let exports = cached_exports.clone();
            return self.apply_module_exports(module_path, exports, items, alias);
        }

        // Currently in the middle of loading this same module further up the call
        // stack: importing it again here would recurse forever instead of erroring.
        if self.import_stack.iter().any(|p| p == &module_file_path) {
            let mut chain = self.import_stack.clone();
            chain.push(module_file_path.clone());
            return Err(RuntimeError {
                message: format!("Circular import detected: {}", chain.join(" -> ")),
            });
        }

        self.import_stack.push(module_file_path.clone());
        self.module_dir_stack.push(module_dir);
        let body_result =
            self.execute_module_file(module_path, &module_file_path, Some(module_content));
        self.module_dir_stack.pop();
        self.import_stack.pop();
        let exports = body_result?;

        self.module_cache
            .insert(module_file_path.clone(), exports.clone());
        self.apply_module_exports(module_path, exports, items, alias)
    }

    /// Parses and executes a module file's body, producing its exports. Does not apply
    /// any caching/cycle-detection itself (see the caller, `load_module`) — this just
    /// does the actual work of running the file, restoring `self.environment` to the
    /// caller's environment before returning, whether it succeeds or fails.
    fn execute_module_file(
        &mut self,
        module_path: &str,
        module_file_path: &str,
        embedded_module_content: Option<String>,
    ) -> RuntimeResult<HashMap<String, Value>> {
        // Read and parse the module file
        let module_content = if let Some(content) = embedded_module_content {
            content
        } else {
            fs::read_to_string(module_file_path).map_err(|e| RuntimeError {
                message: format!("Failed to read module '{}': {}", module_path, e),
            })?
        };

        let mut lexer = crate::lexer::Lexer::new(&module_content);
        let located_tokens = lexer.tokenize_with_positions();
        let tokens = located_tokens
            .into_iter()
            .map(|token| (token.token, token.position))
            .collect();
        let mut parser = crate::parser::Parser::new(tokens);
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

        let module_file_stem = if module_file_path.starts_with("<embedded:") {
            module_path.trim_end_matches(".ject").to_string()
        } else {
            Path::new(module_file_path)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string()
        };
        for (name, value) in crate::stdlib::inject_module_file_builtins(&module_file_stem) {
            module_env.define(name, value);
        }

        // Save current environment and switch to module environment
        let saved_env = std::mem::replace(&mut self.environment, module_env);

        // Run the module body, whatever it returns (Ok or Err) — the environment is
        // restored below either way, so a failing module (e.g. a bad `export` value)
        // can't leave the interpreter stuck in its module-local environment.
        let outcome = self
            .run_module_body(&statements, &module_file_stem, module_file_path)
            .map_err(|error| error.with_source(Some(module_file_path)));

        self.environment = saved_env;
        outcome
    }

    fn run_module_body(
        &mut self,
        statements: &[Stmt],
        module_file_stem: &str,
        module_file_path: &str,
    ) -> RuntimeResult<HashMap<String, Value>> {
        // First, execute all non-export statements to build up the module environment
        for statement in statements {
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
        for statement in statements {
            if let Stmt::ExportFunction { name, params, body } = statement {
                let func = Value::Function {
                    name: name.clone(),
                    source: Some(module_file_path.to_string()),
                    params: params.clone(),
                    body: body.clone(),
                    closure_env: self.environment.clone(),
                };
                self.environment.define(name.clone(), func);
            }
        }

        // Now process export statements and create module functions with proper closure
        let mut exports = HashMap::new();
        for statement in statements {
            match statement {
                Stmt::Export { name, value } => {
                    let val = self.evaluate_expression(value)?;
                    exports.insert(name.clone(), val.clone());
                }
                Stmt::ExportFunction { name, params, body } => {
                    // Create ModuleFunction with the current module environment as closure
                    // This captures all the module's variables and functions
                    let func = Value::ModuleFunction {
                        name: name.clone(),
                        source: Some(module_file_path.to_string()),
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

        for (k, v) in crate::stdlib::inject_module_file_builtins(module_file_stem) {
            exports.entry(k).or_insert(v);
        }

        Ok(exports)
    }

    fn apply_module_exports(
        &mut self,
        module_path: &str,
        exports: HashMap<String, Value>,
        items: &Option<Vec<String>>,
        alias: &Option<String>,
    ) -> RuntimeResult<()> {
        // Import the exported values based on import type
        match (items, alias) {
            (Some(item_list), None) => {
                // import {item1, item2} from "module"
                for item_name in item_list {
                    if let Some(value) = exports.get(item_name) {
                        self.environment.define(item_name.clone(), value.clone());
                    } else {
                        return Err(RuntimeError {
                            message: format!(
                                "Module '{}' does not export '{}'",
                                module_path, item_name
                            ),
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
                    message:
                        "Cannot use both specific imports and alias in the same import statement"
                            .to_string(),
                });
            }
        }

        Ok(())
    }

    fn call_function(&mut self, func: Value, args: &[Argument]) -> RuntimeResult<Value> {
        match &func {
            Value::Function { params, .. } | Value::ModuleFunction { params, .. } => {
                // Function supports keyword arguments and defaults, so argument
                // resolution stays here; invoke_callable just does the actual call.
                let resolved_args = self.resolve_arguments(params, args)?;
                self.invoke_callable(&func, resolved_args)
            }
            Value::Lambda { .. } => {
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
                self.invoke_callable(&func, arg_values)
            }
            Value::BuiltinFunction(_) | Value::NativeFunction { .. } => {
                let mut arg_values = Vec::new();
                for arg in args {
                    match arg {
                        Argument::Positional(expr) => {
                            arg_values.push(self.evaluate_expression(expr)?);
                        }
                        Argument::Keyword { .. } => {
                            return Err(RuntimeError {
                                message: "Builtin functions do not support keyword arguments"
                                    .to_string(),
                            });
                        }
                    }
                }
                self.invoke_callable(&func, arg_values)
            }
            _ => Err(RuntimeError {
                message: format!("Cannot call {}", func.type_name()),
            }),
        }
    }

    /// Calls any callable `Value` with already-evaluated positional argument values.
    /// This is the single place that swaps in a closure's captured environment and
    /// guarantees it is restored before returning -- on success, on a runtime error,
    /// and on an escaping `throw`/`break`/`continue` -- so a failure inside a
    /// function/lambda/module-function body can never leave the interpreter stuck
    /// running in that callee's environment. Every call site (direct calls, method
    /// dispatch, and the map/filter/reduce/any/all callbacks) should go through this
    /// rather than hand-rolling the swap/restore dance, since that's exactly what led
    /// to the environment-leak class of bug here before.
    /// Runs a lambda's block body. Unlike a named function's block body (which always
    /// requires an explicit `return`, matching Python-style predictability),
    /// an anonymous lambda's block implicitly returns the value of a trailing bare
    /// expression statement, the way an OCaml/Ruby/Rust block does -- so
    /// `fn(x) x * x end` doesn't need `return x * x`.
    fn execute_lambda_block(&mut self, statements: &[Stmt]) -> RuntimeResult<Value> {
        if statements.is_empty() {
            return Ok(Value::Nil);
        }

        let (last, rest) = statements.split_last().unwrap();

        for statement in rest {
            match self.execute_statement(statement)? {
                ControlFlow::Return(value) => return Ok(value),
                ControlFlow::Throw(error) => {
                    return Err(RuntimeError {
                        message: error.display(),
                    });
                }
                ControlFlow::Break | ControlFlow::Continue => {
                    return Err(RuntimeError {
                        message: "break/continue in lambda".to_string(),
                    });
                }
                ControlFlow::None => {}
            }
        }

        if let Stmt::Expression(expr) = last {
            // Trailing bare expression with no earlier explicit return: its value IS
            // the lambda's result.
            self.evaluate_expression(expr)
        } else {
            match self.execute_statement(last)? {
                ControlFlow::Return(value) => Ok(value),
                ControlFlow::Throw(error) => Err(RuntimeError {
                    message: error.display(),
                }),
                ControlFlow::Break | ControlFlow::Continue => Err(RuntimeError {
                    message: "break/continue in lambda".to_string(),
                }),
                ControlFlow::None => Ok(Value::Nil),
            }
        }
    }

    pub(crate) fn invoke_callable(
        &mut self,
        callable: &Value,
        arg_values: Vec<Value>,
    ) -> RuntimeResult<Value> {
        match callable {
            Value::Lambda {
                params,
                body,
                closure_env,
            } => {
                if arg_values.len() != params.len() {
                    return Err(RuntimeError {
                        message: format!(
                            "Expected {} argument(s) but got {}",
                            params.len(),
                            arg_values.len()
                        ),
                    });
                }

                let saved_env = std::mem::replace(&mut self.environment, closure_env.clone());
                self.environment.push_scope();
                for (param, arg) in params.iter().zip(arg_values.iter()) {
                    self.environment.define(param.clone(), arg.clone());
                }

                let result = match &body {
                    crate::ast::LambdaBody::Expression(expr) => self.evaluate_expression(expr),
                    crate::ast::LambdaBody::Block(statements) => {
                        self.execute_lambda_block(statements)
                    }
                };

                self.environment.pop_scope();
                self.environment = saved_env;
                result
            }
            Value::Function {
                name,
                source,
                params,
                body,
                closure_env,
            } => {
                if arg_values.len() != params.len() {
                    return Err(RuntimeError {
                        message: format!(
                            "Expected {} argument(s) but got {}",
                            params.len(),
                            arg_values.len()
                        ),
                    });
                }

                let saved_env = std::mem::replace(&mut self.environment, closure_env.clone());
                self.environment.push_scope();
                for (param, arg) in params.iter().zip(arg_values.iter()) {
                    self.environment.define(param.name.clone(), arg.clone());
                }

                let result = match self.execute_block(body) {
                    Ok(ControlFlow::Return(value)) => Ok(value),
                    Ok(ControlFlow::Throw(error)) => Err(RuntimeError {
                        message: error.display(),
                    }),
                    Ok(ControlFlow::Break) | Ok(ControlFlow::Continue) => Err(RuntimeError {
                        message: "break/continue in function".to_string(),
                    }),
                    Ok(ControlFlow::None) => Ok(Value::Nil),
                    Err(e) => Err(e),
                };

                self.environment.pop_scope();
                self.environment = saved_env;
                result.map_err(|error| error.with_source(source.as_deref()).with_frame(name))
            }
            Value::ModuleFunction {
                name,
                source,
                params,
                body,
                closure_env,
            } => {
                if arg_values.len() != params.len() {
                    return Err(RuntimeError {
                        message: format!(
                            "Expected {} argument(s) but got {}",
                            params.len(),
                            arg_values.len()
                        ),
                    });
                }

                let saved_env = std::mem::replace(&mut self.environment, closure_env.clone());
                self.environment.push_scope();
                for (param, arg) in params.iter().zip(arg_values.iter()) {
                    self.environment.define(param.name.clone(), arg.clone());
                }

                let result = match self.execute_block(body) {
                    Ok(ControlFlow::Return(value)) => Ok(value),
                    Ok(ControlFlow::Throw(error)) => Err(RuntimeError {
                        message: error.display(),
                    }),
                    Ok(ControlFlow::Break) | Ok(ControlFlow::Continue) => Err(RuntimeError {
                        message: "break/continue in function".to_string(),
                    }),
                    Ok(ControlFlow::None) => Ok(Value::Nil),
                    Err(e) => Err(e),
                };

                self.environment.pop_scope();
                self.environment = saved_env;
                result.map_err(|error| error.with_source(source.as_deref()).with_frame(name))
            }
            Value::BuiltinFunction(name) => crate::stdlib::call_builtin_function(name, arg_values),
            Value::NativeFunction { module, name } => {
                crate::native::call_module_with_interpreter(module, name, arg_values, self)
            }
            _ => Err(RuntimeError {
                message: format!("Cannot call {}", callable.type_name()),
            }),
        }
    }

    fn resolve_arguments(
        &mut self,
        params: &[crate::ast::Parameter],
        args: &[Argument],
    ) -> RuntimeResult<Vec<Value>> {
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
                        message: format!("Missing required argument '{}'", param.name),
                    });
                }
            }
        }

        // Convert Vec<Option<Value>> to Vec<Value>
        Ok(resolved_args.into_iter().map(|arg| arg.unwrap()).collect())
    }

    fn pattern_matches(
        &mut self,
        pattern: &crate::ast::Pattern,
        value: &Value,
    ) -> RuntimeResult<bool> {
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
            crate::ast::Pattern::Relational(op, expr) => {
                let pattern_value = self.evaluate_expression(expr)?;
                let result = self.evaluate_binary_op(value, op, &pattern_value)?;
                if let Value::Bool(b) = result {
                    Ok(b)
                } else {
                    Ok(false)
                }
            }
            crate::ast::Pattern::Range(start_expr, end_expr) => {
                let start_value = self.evaluate_expression(start_expr)?;
                let end_value = self.evaluate_expression(end_expr)?;
                let above_start =
                    self.evaluate_binary_op(value, &BinaryOp::GreaterEqual, &start_value)?;
                let below_end = self.evaluate_binary_op(value, &BinaryOp::LessEqual, &end_value)?;
                match (above_start, below_end) {
                    (Value::Bool(a), Value::Bool(b)) => Ok(a && b),
                    _ => Ok(false),
                }
            }
        }
    }

    fn evaluate_match_arm_body(&mut self, body: &crate::ast::MatchArmBody) -> RuntimeResult<Value> {
        match body {
            crate::ast::MatchArmBody::Expression(expr) => self.evaluate_expression(expr),
            crate::ast::MatchArmBody::Block(statements) => match self.execute_block(statements)? {
                ControlFlow::Return(value) => Ok(value),
                ControlFlow::Throw(error) => Err(RuntimeError {
                    message: format!("Error in match arm: {}", error),
                }),
                ControlFlow::Break | ControlFlow::Continue => Err(RuntimeError {
                    message: "break/continue not allowed directly in a match arm".to_string(),
                }),
                ControlFlow::None => Ok(Value::Nil),
            },
        }
    }

    fn normalize_array_index_cell(i: i64, len: usize) -> Result<usize, RuntimeError> {
        let actual = if i < 0 { len as i64 + i } else { i };
        if actual < 0 || actual >= len as i64 {
            Err(RuntimeError {
                message: format!("Array index out of bounds: {}", i),
            })
        } else {
            Ok(actual as usize)
        }
    }

    fn set_array_at_int_path(
        mut arr: Vec<Value>,
        path: &[i64],
        val: Value,
    ) -> Result<Vec<Value>, RuntimeError> {
        if path.is_empty() {
            return Err(RuntimeError {
                message: "Empty index chain".to_string(),
            });
        }
        if path.len() == 1 {
            let idx = Self::normalize_array_index_cell(path[0], arr.len())?;
            arr[idx] = val;
            return Ok(arr);
        }
        let idx = Self::normalize_array_index_cell(path[0], arr.len())?;
        let inner = match arr.get(idx) {
            Some(Value::Array(a)) => a.borrow().clone(),
            Some(other) => {
                return Err(RuntimeError {
                    message: format!(
                        "Cannot assign through {}; expected nested array",
                        other.type_name()
                    ),
                })
            }
            None => {
                return Err(RuntimeError {
                    message: "Array index out of bounds".to_string(),
                })
            }
        };
        let new_inner = Self::set_array_at_int_path(inner, &path[1..], val)?;
        arr[idx] = Value::array(new_inner);
        Ok(arr)
    }

    fn assign_index_chain(
        &mut self,
        root_name: &str,
        idx_exprs: &[Expr],
        val: Value,
    ) -> RuntimeResult<()> {
        let mut path_i64 = Vec::with_capacity(idx_exprs.len());
        for e in idx_exprs {
            let v = self.evaluate_expression(e)?;
            match v {
                Value::Integer(i) => path_i64.push(i),
                _ => {
                    return Err(RuntimeError {
                        message: "Nested index assignment requires integer indices".to_string(),
                    })
                }
            }
        }

        let obj = self
            .environment
            .get(root_name)
            .ok_or_else(|| RuntimeError {
                message: format!("Undefined variable '{}'", root_name),
            })?;

        let Value::Array(root_arr) = obj else {
            return Err(RuntimeError {
                message: format!("Cannot chain-index assign into {}", obj.type_name()),
            });
        };

        let snapshot = root_arr.borrow().clone();
        let new_root = Self::set_array_at_int_path(snapshot, &path_i64, val)?;
        *root_arr.borrow_mut() = new_root;
        Ok(())
    }

    fn evaluate_increment_decrement(
        &mut self,
        target: &Expr,
        prefix: bool,
        is_increment: bool,
    ) -> RuntimeResult<Value> {
        match target {
            Expr::Identifier(name) => {
                let current = self.environment.get(name).ok_or_else(|| RuntimeError {
                    message: format!("Undefined variable '{}'", name),
                })?;
                let new_value = match current {
                    Value::Integer(n) => Value::Integer(if is_increment { n + 1 } else { n - 1 }),
                    Value::Float(f) => Value::Float(if is_increment { f + 1.0 } else { f - 1.0 }),
                    _ => {
                        return Err(RuntimeError {
                            message: "Can only increment/decrement numbers".to_string(),
                        })
                    }
                };
                self.environment.set(name, new_value.clone());
                Ok(if prefix { new_value } else { current })
            }
            _ => Err(RuntimeError {
                message: "Invalid increment/decrement target".to_string(),
            }),
        }
    }

    /// Extracts (array_or_unique_array, is_unique) from an evaluated Value, erroring
    /// with a consistent message if it's neither.
    fn expect_array_like(
        &self,
        value: Value,
        func_name: &str,
    ) -> RuntimeResult<(Vec<Value>, bool)> {
        match value {
            Value::Array(arr) => Ok((arr.borrow().clone(), false)),
            Value::UniqueArray(arr) => Ok((arr, true)),
            other => Err(RuntimeError {
                message: format!(
                    "{}() requires an array or unique array, got {}",
                    func_name,
                    other.type_name()
                ),
            }),
        }
    }

    fn call_higher_order_function(
        &mut self,
        func_name: &str,
        args: &[crate::ast::Argument],
    ) -> RuntimeResult<Value> {
        // Evaluate every argument expression exactly once, here, then hand off to the
        // Value-based core. This matters most for the `obj.map(f)` method-sugar call
        // site, which already has `obj` evaluated -- routing it through here again
        // would evaluate the receiver expression a second time (and re-run any side
        // effects in it) just to reach the same array.
        let mut values = Vec::with_capacity(args.len());
        for arg in args {
            match arg {
                crate::ast::Argument::Positional(expr) => {
                    values.push(self.evaluate_expression(expr)?)
                }
                crate::ast::Argument::Keyword { .. } => {
                    return Err(RuntimeError {
                        message: format!("{}() does not support keyword arguments", func_name),
                    });
                }
            }
        }
        self.call_higher_order_with_values(func_name, values)
    }

    /// Same as `call_higher_order_function`, but takes already-evaluated argument
    /// values instead of AST expressions -- for call sites (like method-sugar
    /// dispatch) that already have the receiver evaluated and shouldn't evaluate it
    /// twice.
    fn call_higher_order_with_values(
        &mut self,
        func_name: &str,
        values: Vec<Value>,
    ) -> RuntimeResult<Value> {
        match func_name {
            "map" => {
                if values.len() != 2 {
                    return Err(RuntimeError {
                        message: "map() takes 2 arguments (array, function)".to_string(),
                    });
                }
                let mut values = values;
                let func_val = values.pop().unwrap();
                let array_val = values.pop().unwrap();
                let (arr, is_unique) = self.expect_array_like(array_val, "map")?;

                let mut result = Vec::new();
                let mut seen = std::collections::HashSet::new();
                for item in &arr {
                    let mapped = self.invoke_callable(&func_val, vec![item.clone()])?;
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

                if is_unique {
                    Ok(Value::UniqueArray(result))
                } else {
                    Ok(Value::array(result))
                }
            }
            "filter" => {
                if values.len() != 2 {
                    return Err(RuntimeError {
                        message: "filter() takes 2 arguments (array, function)".to_string(),
                    });
                }
                let mut values = values;
                let func_val = values.pop().unwrap();
                let array_val = values.pop().unwrap();
                let (arr, is_unique) = self.expect_array_like(array_val, "filter")?;

                let mut result = Vec::new();
                let mut seen = std::collections::HashSet::new();
                for item in &arr {
                    let keep = self
                        .invoke_callable(&func_val, vec![item.clone()])?
                        .is_truthy();
                    if keep {
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

                if is_unique {
                    Ok(Value::UniqueArray(result))
                } else {
                    Ok(Value::array(result))
                }
            }
            "reduce" => {
                if values.len() < 2 || values.len() > 3 {
                    return Err(RuntimeError {
                        message: "reduce() takes 2 or 3 arguments (array, function, [initial])"
                            .to_string(),
                    });
                }
                let mut values = values;
                let explicit_initial = if values.len() > 2 { values.pop() } else { None };
                let func_val = values.pop().unwrap();
                let array_val = values.pop().unwrap();
                let (arr, _is_unique) = self.expect_array_like(array_val, "reduce")?;

                // With no explicit initial value, the first element seeds the
                // accumulator and iteration starts from the second element -- not
                // Value::Nil, which would make the very first call `nil OP first_item`
                // and fail for any operator that doesn't accept nil.
                let (mut accumulator, start_index) = if let Some(initial) = explicit_initial {
                    (initial, 0)
                } else if let Some(first) = arr.first() {
                    (first.clone(), 1)
                } else {
                    return Err(RuntimeError {
                        message: "reduce() of an empty array requires an initial value".to_string(),
                    });
                };

                for item in arr.iter().skip(start_index) {
                    accumulator =
                        self.invoke_callable(&func_val, vec![accumulator, item.clone()])?;
                }
                Ok(accumulator)
            }
            "any" => {
                if values.len() != 2 {
                    return Err(RuntimeError {
                        message: "any() takes 2 arguments (array, function)".to_string(),
                    });
                }
                let mut values = values;
                let func_val = values.pop().unwrap();
                let array_val = values.pop().unwrap();
                let (arr, _is_unique) = self.expect_array_like(array_val, "any")?;

                for item in &arr {
                    if self
                        .invoke_callable(&func_val, vec![item.clone()])?
                        .is_truthy()
                    {
                        return Ok(Value::Bool(true));
                    }
                }
                Ok(Value::Bool(false))
            }
            "all" => {
                if values.len() != 2 {
                    return Err(RuntimeError {
                        message: "all() takes 2 arguments (array, function)".to_string(),
                    });
                }
                let mut values = values;
                let func_val = values.pop().unwrap();
                let array_val = values.pop().unwrap();
                let (arr, _is_unique) = self.expect_array_like(array_val, "all")?;

                for item in &arr {
                    if !self
                        .invoke_callable(&func_val, vec![item.clone()])?
                        .is_truthy()
                    {
                        return Ok(Value::Bool(false));
                    }
                }
                Ok(Value::Bool(true))
            }
            _ => Err(RuntimeError {
                message: format!("Unknown function: {}", func_name),
            }),
        }
    }
}
