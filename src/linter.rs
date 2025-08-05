use std::collections::{HashMap, HashSet};
use crate::ast::{Stmt, Expr, Parameter, Argument};
use crate::diagnostic::Diagnostic;

#[derive(Debug, Clone)]
struct Variable {
    name: String,
    used: bool,
    declared_in_scope: usize,
}

#[derive(Debug, Clone)]
struct FunctionSignature {
    name: String,
    parameters: Vec<Parameter>,
}

#[derive(Debug, Clone)]
struct LintError {
    message: String,
    position: Option<crate::lexer::SourcePosition>,
}

#[derive(Debug, Clone)]
struct LintWarning {
    message: String,
    position: Option<crate::lexer::SourcePosition>,
}

pub struct Linter {
    scopes: Vec<HashMap<String, Variable>>, // Stack of scopes with variable info
    warnings: Vec<LintWarning>,
    errors: Vec<LintError>,
    current_scope_id: usize,
    functions: HashSet<String>,
    function_signatures: HashMap<String, FunctionSignature>, // Track function signatures
    in_function: bool,
    // Store positioned tokens to find locations of identifiers
    positioned_tokens: Vec<(crate::lexer::Token, crate::lexer::SourcePosition)>,
    source: String,
}

#[derive(Debug, Clone, PartialEq)]
enum ScopeKind {
    Global,
    Function,
    Block,
    MatchArm,
}

impl Linter {
    pub fn new() -> Self {
        let mut linter = Linter {
            scopes: vec![HashMap::new()], // Global scope
            warnings: Vec::new(),
            errors: Vec::new(),
            current_scope_id: 0,
            functions: HashSet::new(),
            function_signatures: HashMap::new(),
            in_function: false,
            positioned_tokens: Vec::new(),
            source: String::new(),
        };
        
        // Add built-in functions to the functions set
        linter.add_builtin_functions();
        linter
    }
    
    fn add_builtin_functions(&mut self) {
        // Mathematical functions
        self.functions.insert("abs".to_string());
        self.functions.insert("sqrt".to_string());
        self.functions.insert("pow".to_string());
        self.functions.insert("sin".to_string());
        self.functions.insert("cos".to_string());
        self.functions.insert("tan".to_string());
        self.functions.insert("floor".to_string());
        self.functions.insert("ceil".to_string());
        self.functions.insert("round".to_string());
        self.functions.insert("min".to_string());
        self.functions.insert("max".to_string());
        
        // Array functions
        self.functions.insert("len".to_string());
        self.functions.insert("push".to_string());
        self.functions.insert("pop".to_string());
        self.functions.insert("map".to_string());
        self.functions.insert("filter".to_string());
        self.functions.insert("reduce".to_string());
        self.functions.insert("sum".to_string());
        
        // String functions
        self.functions.insert("upper".to_string());
        self.functions.insert("lower".to_string());
        self.functions.insert("trim".to_string());
        self.functions.insert("split".to_string());
        self.functions.insert("join".to_string());
        self.functions.insert("replace".to_string());
        
        // Utility functions
        self.functions.insert("type_of".to_string());
        self.functions.insert("range".to_string());
        self.functions.insert("random".to_string());
        
        // File I/O functions
        self.functions.insert("read_file".to_string());
        self.functions.insert("write_file".to_string());
        
        // JSON functions
        self.functions.insert("parse_json".to_string());
        self.functions.insert("to_json".to_string());
        
        // String indexing/slicing functions
        self.functions.insert("char_at".to_string());
        self.functions.insert("substring".to_string());
    }
    
    pub fn with_tokens_and_source(mut self, positioned_tokens: Vec<(crate::lexer::Token, crate::lexer::SourcePosition)>, source: String) -> Self {
        self.positioned_tokens = positioned_tokens;
        self.source = source;
        self
    }

    pub fn lint(&mut self, statements: &[Stmt]) -> (Vec<Diagnostic>, bool) {
        self.scopes.clear();
        self.scopes.push(HashMap::new()); // Global scope
        self.warnings.clear();
        self.errors.clear();
        self.current_scope_id = 0;
        
        // Re-add built-in functions (don't clear them)
        self.functions.clear();
        self.add_builtin_functions();
        
        self.function_signatures.clear();
        self.in_function = false;

        // Single pass: analyze all statements
        for stmt in statements {
            self.analyze_statement(stmt);
        }

        // Check for unused variables in all scopes
        for scope in &self.scopes {
            for var in scope.values() {
                if !var.used && !var.name.starts_with('_') {
                    self.warnings.push(LintWarning {
                        message: format!("unused variable `{}`", var.name),
                        position: None, // TODO: We could track declaration positions
                    });
                }
            }
        }

        // Convert to diagnostics
        let mut diagnostics = Vec::new();
        let mut has_errors = false;
        
        for error in &self.errors {
            has_errors = true;
            let mut diagnostic = Diagnostic::error(error.message.clone()).with_code("E0001".to_string());
            if let Some(pos) = &error.position {
                diagnostic = diagnostic.with_location(pos.line, pos.column);
            }
            diagnostics.push(diagnostic);
        }
        
        for warning in &self.warnings {
            let mut diagnostic = Diagnostic::warning(warning.message.clone()).with_code("W0001".to_string());
            if let Some(pos) = &warning.position {
                diagnostic = diagnostic.with_location(pos.line, pos.column);
            }
            diagnostics.push(diagnostic);
        }
        
        (diagnostics, has_errors)
    }

    fn push_scope(&mut self) {
        self.current_scope_id += 1;
        self.scopes.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        if self.scopes.len() > 1 {
            // Before popping, check for unused variables in this scope
            if let Some(scope) = self.scopes.last() {
                for var in scope.values() {
                    if !var.used && !var.name.starts_with('_') {
                        self.warnings.push(LintWarning {
                            message: format!("unused variable `{}`", var.name),
                            position: None, // TODO: We could track declaration positions
                        });
                    }
                }
            }
            self.scopes.pop();
        }
    }

    fn declare_variable(&mut self, name: String) {
        if let Some(current_scope) = self.scopes.last_mut() {
            if current_scope.contains_key(&name) {
                let position = self.find_identifier_position(&name);
                self.warnings.push(LintWarning {
                    message: format!("warning: variable `{}` is already declared in this scope", name),
                    position,
                });
            } else {
                current_scope.insert(name.clone(), Variable {
                    name,
                    used: false,
                    declared_in_scope: self.current_scope_id,
                });
            }
        }
    }

    fn use_variable(&mut self, name: &str) -> bool {
        // Look for variable in scopes from innermost to outermost
        for scope in self.scopes.iter_mut().rev() {
            if let Some(var) = scope.get_mut(name) {
                var.used = true;
                return true;
            }
        }
        false
    }

    fn find_variable(&self, name: &str) -> bool {
        for scope in self.scopes.iter().rev() {
            if scope.contains_key(name) {
                return true;
            }
        }
        false
    }

    fn analyze_statement(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Let { name, value } => {
                // Analyze the value expression first
                self.analyze_expr(value);
                // Then declare the variable (Rust-like: can't use variable before declaration)
                self.declare_variable(name.clone());
            }
            Stmt::Assign { name, value } => {
                // Analyze the value expression first
                self.analyze_expr(value);
                // Check if variable exists
                if !self.use_variable(name) {
                    let position = self.find_identifier_position(name);
                    self.errors.push(LintError {
                        message: format!("cannot assign to undeclared variable `{}`", name),
                        position,
                    });
                }
            }
            Stmt::Function { name, params, body } => {
                // Check for function redeclaration
                if self.functions.contains(name) {
                    let position = self.find_identifier_position(name);
                    self.warnings.push(LintWarning {
                        message: format!("warning: function `{}` is already defined", name),
                        position,
                    });
                }
                self.functions.insert(name.clone());
                
                // Store function signature for validation
                self.function_signatures.insert(name.clone(), FunctionSignature {
                    name: name.clone(),
                    parameters: params.clone(),
                });
                
                // Declare the function as a function, not as a variable
                
                // Create new scope for function
                self.push_scope();
                let was_in_function = self.in_function;
                self.in_function = true;
                
                // Add parameters to function scope and analyze default values
                for param in params {
                    // Analyze default value first (before declaring the parameter)
                    if let Some(default_expr) = &param.default_value {
                        self.analyze_expr(default_expr);
                    }
                    self.declare_variable(param.name.clone());
                }
                
                // Analyze function body
                for stmt in body {
                    self.analyze_statement(stmt);
                }
                
                self.in_function = was_in_function;
                self.pop_scope();
            }
            Stmt::If { condition, then_branch, elseif_branches, else_branch } => {
                self.analyze_expr(condition);
                
                // Each branch gets its own scope
                self.push_scope();
                for stmt in then_branch {
                    self.analyze_statement(stmt);
                }
                self.pop_scope();
                
                for branch in elseif_branches {
                    self.analyze_expr(&branch.condition);
                    self.push_scope();
                    for stmt in &branch.body {
                        self.analyze_statement(stmt);
                    }
                    self.pop_scope();
                }
                
                if let Some(else_body) = else_branch {
                    self.push_scope();
                    for stmt in else_body {
                        self.analyze_statement(stmt);
                    }
                    self.pop_scope();
                }
            }
            Stmt::While { condition, body } => {
                self.analyze_expr(condition);
                self.push_scope();
                for stmt in body {
                    self.analyze_statement(stmt);
                }
                self.pop_scope();
            }
            Stmt::For { var, iterable, body } => {
                self.analyze_expr(iterable);
                // For loop creates its own scope with the loop variable
                self.push_scope();
                self.declare_variable(var.clone());
                for stmt in body {
                    self.analyze_statement(stmt);
                }
                self.pop_scope();
            }
            Stmt::Return(Some(expr)) => {
                self.analyze_expr(expr);
                if !self.in_function {
                    self.errors.push(LintError {
                        message: "`return` outside of function".to_string(),
                        position: None, // TODO: Could find 'return' keyword position
                    });
                }
            }
            Stmt::Return(None) => {
                if !self.in_function {
                    self.errors.push(LintError {
                        message: "`return` outside of function".to_string(),
                        position: None, // TODO: Could find 'return' keyword position
                    });
                }
            }
            Stmt::Print(expr) => {
                self.analyze_expr(expr);
            }
            Stmt::Expression(expr) => {
                self.analyze_expr(expr);
            }
            Stmt::Import { .. } | Stmt::Export { .. } | Stmt::ExportFunction { .. } => {
                // TODO: Handle imports/exports if needed
            }
        }
    }

    fn analyze_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Identifier(name) => {
                // Check if it's a function first, then check if it's a variable
                if !self.functions.contains(name) && !self.use_variable(name) {
                    let position = self.find_identifier_position(name);
                    self.errors.push(LintError {
                        message: format!("use of undeclared variable `{}`", name),
                        position,
                    });
                }
            }
            Expr::Binary { left, right, .. } => {
                self.analyze_expr(left);
                self.analyze_expr(right);
            }
            Expr::Unary { operand, .. } => {
                self.analyze_expr(operand);
            }
            Expr::Call { callee, args } => {
                self.analyze_expr(callee);
                for arg in args {
                    match arg {
                        crate::ast::Argument::Positional(expr) => self.analyze_expr(expr),
                        crate::ast::Argument::Keyword { value, .. } => self.analyze_expr(value),
                    }
                }
                
                // Validate function call if it's a direct function call
                if let Expr::Identifier(func_name) = callee.as_ref() {
                    self.validate_function_call(func_name, args);
                }
            }
            Expr::Index { object, index } => {
                self.analyze_expr(object);
                self.analyze_expr(index);
            }
            Expr::Member { object, .. } => {
                self.analyze_expr(object);
            }
            Expr::Array(elements) => {
                for elem in elements {
                    self.analyze_expr(elem);
                }
            }
            Expr::Dictionary(pairs) => {
                for (_, value) in pairs {
                    self.analyze_expr(value);
                }
            }
            Expr::Range { start, end, step } => {
                self.analyze_expr(start);
                self.analyze_expr(end);
                if let Some(step_expr) = step {
                    self.analyze_expr(step_expr);
                }
            }
            Expr::Lambda { params, body } => {
                // Lambda creates its own scope
                self.push_scope();
                // Declare parameters
                for param in params {
                    self.declare_variable(param.clone());
                }
                
                match body {
                    crate::ast::LambdaBody::Block(stmts) => {
                        for stmt in stmts {
                            self.analyze_statement(stmt);
                        }
                    }
                    crate::ast::LambdaBody::Expression(expr) => {
                        self.analyze_expr(expr);
                    }
                }
                self.pop_scope();
            }
            Expr::Match { expr, arms } => {
                self.analyze_expr(expr);
                for arm in arms {
                    // Each match arm gets its own scope for pattern bindings
                    self.push_scope();
                    self.analyze_pattern(&arm.pattern);
                    self.analyze_expr(&arm.body);
                    self.pop_scope();
                }
            }
            // Literals don't need analysis
            Expr::Integer(_) | Expr::Float(_) | Expr::String(_) | 
            Expr::InterpolatedString(_) | Expr::Bool(_) | Expr::Nil => {}
            
            _ => {}
        }
    }

    fn analyze_pattern(&mut self, pattern: &crate::ast::Pattern) {
        match pattern {
            crate::ast::Pattern::Literal(expr) => {
                self.analyze_expr(expr);
            }
            crate::ast::Pattern::Identifier(name) => {
                // Pattern identifiers create new bindings in the current scope
                self.declare_variable(name.clone());
            }
            crate::ast::Pattern::Wildcard => {
                // Wildcard doesn't create any bindings
            }
        }
    }
    
    // Helper method to find the position of an identifier in the positioned tokens
    fn find_identifier_position(&self, identifier: &str) -> Option<crate::lexer::SourcePosition> {
        for (token, position) in &self.positioned_tokens {
            if let crate::lexer::Token::Identifier(name) = token {
                if name == identifier {
                    return Some(position.clone());
                }
            }
        }
        None
    }
    
    fn validate_function_call(&mut self, func_name: &str, args: &[Argument]) {
        if let Some(signature) = self.function_signatures.get(func_name).cloned() {
            // Simulate the argument resolution logic from the interpreter
            let mut resolved_args = vec![false; signature.parameters.len()]; // track which args are provided
            let mut positional_count = 0;
            
            // First pass: handle positional arguments
            for arg in args {
                match arg {
                    Argument::Positional(_) => {
                        if positional_count >= signature.parameters.len() {
                            let position = self.find_identifier_position(func_name);
                            self.errors.push(LintError {
                                message: format!("too many arguments for function `{}`", func_name),
                                position,
                            });
                            return;
                        }
                        resolved_args[positional_count] = true;
                        positional_count += 1;
                    }
                    Argument::Keyword { .. } => {
                        // We'll handle keyword arguments in the second pass
                    }
                }
            }
            
            // Second pass: handle keyword arguments
            for arg in args {
                if let Argument::Keyword { name, .. } = arg {
                    // Find the parameter with this name
                    let param_index = signature.parameters.iter().position(|p| p.name == *name);
                    
                    match param_index {
                        Some(index) => {
                            if resolved_args[index] {
                                let position = self.find_identifier_position(func_name);
                                self.errors.push(LintError {
                                    message: format!("argument `{}` specified multiple times in call to `{}`", name, func_name),
                                    position,
                                });
                                return;
                            }
                            resolved_args[index] = true;
                        }
                        None => {
                            let position = self.find_identifier_position(func_name);
                            self.errors.push(LintError {
                                message: format!("unknown parameter `{}` for function `{}`", name, func_name),
                                position,
                            });
                            return;
                        }
                    }
                }
            }
            
            // Third pass: check for missing required arguments
            for (i, param) in signature.parameters.iter().enumerate() {
                if !resolved_args[i] && param.default_value.is_none() {
                    let position = self.find_identifier_position(func_name);
                    self.errors.push(LintError {
                        message: format!("missing required argument `{}` for function `{}`", param.name, func_name),
                        position,
                    });
                }
            }
        }
        // If function signature not found, we already reported "undeclared variable" error
    }

}
