use crate::ast::{Argument, Expr, Parameter, Stmt};
use crate::diagnostic::Diagnostic;
use crate::module_interface::{ExportKind, ModuleExport, ModuleInterface};
use crate::semantic::SymbolKind;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Debug, Clone)]
struct Variable {
    name: String,
    used: bool,
    mutable: bool,
    warn_if_unused: bool,
    declaration: Option<crate::lexer::SourcePosition>,
    module_interface: Option<ModuleInterface>,
    function_signature: Option<FunctionSignature>,
}

#[derive(Debug, Clone)]
struct FunctionSignature {
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
    previous: Option<crate::lexer::SourcePosition>,
}

#[derive(Clone)]
pub struct Linter {
    scopes: Vec<HashMap<String, Variable>>, // Stack of scopes with variable info
    warnings: Vec<LintWarning>,
    errors: Vec<LintError>,
    functions: HashSet<String>,
    function_signatures: HashMap<String, FunctionSignature>, // Track function signatures
    in_function: bool,
    in_loop: bool,
    expression_start: Option<(usize, usize)>,
    // Store positioned tokens to find locations of identifiers
    positioned_tokens: Vec<(crate::lexer::Token, crate::lexer::SourcePosition)>,
    next_return_token: usize,
    next_loop_control_token: usize,
    source: String,
    source_path: std::path::PathBuf,
    module_interfaces: RefCell<HashMap<String, Result<ModuleInterface, String>>>,
    source_overrides: HashMap<std::path::PathBuf, String>,
    declaration_positions: HashMap<(String, SymbolKind), VecDeque<crate::lexer::SourcePosition>>,
    module_function_declarations: HashMap<String, Option<crate::lexer::SourcePosition>>,
}

impl Default for Linter {
    fn default() -> Self {
        Self::new()
    }
}

impl Linter {
    fn predeclare_functions(&mut self, statements: &[Stmt]) {
        for statement in statements {
            let (name, params) = match statement {
                Stmt::Function { name, params, .. } | Stmt::ExportFunction { name, params, .. } => {
                    (name, params)
                }
                _ => continue,
            };
            self.functions.insert(name.clone());
            // If CorLib already contributed a binding, calls now use the
            // predeclared source function at that name.
            self.set_binding_signature(name, params);
            self.function_signatures.insert(
                name.clone(),
                FunctionSignature {
                    parameters: params.clone(),
                },
            );
        }
    }

    pub(crate) fn error_code(message: &str) -> &'static str {
        if message.starts_with("package graph: ") {
            "E4102"
        } else if message.starts_with("circular module import: ") {
            "E3102"
        } else if message.starts_with("module graph ")
            || message.contains("module '")
            || message.contains("cannot inspect module")
            || message.contains("does not export")
            || (message.starts_with("native backend '")
                && message.ends_with(" is private; import its public package instead"))
        {
            "E3101"
        } else {
            "E2001"
        }
    }

    fn should_skip_unused_warning(variable: &Variable) -> bool {
        !variable.warn_if_unused || variable.name.starts_with('_')
    }

    pub fn new() -> Self {
        Linter {
            scopes: vec![HashMap::new()], // Global scope
            warnings: Vec::new(),
            errors: Vec::new(),
            functions: HashSet::new(),
            function_signatures: HashMap::new(),
            in_function: false,
            in_loop: false,
            expression_start: None,
            positioned_tokens: Vec::new(),
            next_return_token: 0,
            next_loop_control_token: 0,
            source: String::new(),
            source_path: std::env::current_dir().unwrap_or_default(),
            module_interfaces: RefCell::new(HashMap::new()),
            source_overrides: HashMap::new(),
            declaration_positions: HashMap::new(),
            module_function_declarations: HashMap::new(),
        }
    }

    fn add_builtin_functions(&mut self) {
        for name in crate::stdlib::corlib_function_names() {
            self.declare_external_constant(name.clone());
            self.functions.insert(name);
        }

        if let Some(module_stem) = self.source_path.file_stem().and_then(|stem| stem.to_str()) {
            let injected = crate::stdlib::inject_module_file_builtins(module_stem)
                .into_iter()
                .filter_map(|(name, value)| {
                    matches!(value, crate::value::Value::BuiltinFunction(_)).then_some(name)
                })
                .collect::<Vec<_>>();
            for name in injected {
                self.declare_external_constant(name.clone());
                self.functions.insert(name);
            }
        }

        for name in crate::stdlib::corlib_constant_names() {
            self.declare_external_constant(name);
        }
    }

    pub fn with_tokens_and_source(
        mut self,
        positioned_tokens: Vec<(crate::lexer::Token, crate::lexer::SourcePosition)>,
        source: String,
    ) -> Self {
        self.positioned_tokens = positioned_tokens;
        self.source = source;
        self
    }

    fn reset_declaration_positions(&mut self) {
        self.declaration_positions.clear();
        self.module_function_declarations.clear();
        for symbol in crate::semantic::analyze(&self.source).symbols {
            let position =
                crate::lexer::SourcePosition::new(symbol.span.line, symbol.span.column, 0);
            self.declaration_positions
                .entry((symbol.name, symbol.kind))
                .or_default()
                .push_back(position);
        }
    }

    fn record_module_function(
        &mut self,
        name: &str,
        position: Option<crate::lexer::SourcePosition>,
    ) -> bool {
        if let Some(previous) = self
            .module_function_declarations
            .insert(name.to_owned(), position.clone())
        {
            self.warnings.push(LintWarning {
                message: format!("function `{name}` is already defined"),
                position,
                previous,
            });
            return true;
        }
        false
    }

    pub fn with_source_path(mut self, path: impl Into<std::path::PathBuf>) -> Self {
        self.source_path = path.into();
        self
    }

    pub fn with_source_overrides(mut self, sources: HashMap<std::path::PathBuf, String>) -> Self {
        self.source_overrides = sources;
        self
    }

    pub fn lint(&mut self, statements: &[Stmt]) -> (Vec<Diagnostic>, bool) {
        self.next_return_token = 0;
        self.next_loop_control_token = 0;
        self.scopes.clear();
        self.scopes.push(HashMap::new()); // Global scope
        self.warnings.clear();
        self.reset_declaration_positions();
        self.errors.clear();
        self.module_interfaces.borrow_mut().clear();

        // Re-add built-in functions (don't clear them)
        self.functions.clear();
        self.add_builtin_functions();

        self.function_signatures.clear();
        self.in_function = false;
        self.in_loop = false;
        self.predeclare_functions(statements);

        // Single pass: analyze all statements
        for stmt in statements {
            self.analyze_statement(stmt);
        }

        // Check for unused variables in all scopes
        for scope in &self.scopes {
            for var in scope.values() {
                // Skip warnings for stdlib constants and underscore-prefixed variables
                if !var.used && !Self::should_skip_unused_warning(var) {
                    self.warnings.push(LintWarning {
                        message: format!("unused variable `{}`", var.name),
                        position: var.declaration.clone(),
                        previous: None,
                    });
                }
            }
        }

        // Convert to diagnostics
        let mut diagnostics = Vec::new();
        let mut has_errors = false;

        for error in &self.errors {
            has_errors = true;
            let mut diagnostic = Diagnostic::error(error.message.clone())
                .with_code(Self::error_code(&error.message).to_string());
            if Self::error_code(&error.message) == "E3101" {
                diagnostic = diagnostic.with_help(
                    "check the import path, package dependency, and exported name".into(),
                );
            } else if error.message.contains("immutable binding") {
                diagnostic = diagnostic.with_help(
                    "declare a mutable binding with `let` if this value must be reassigned".into(),
                );
            }
            if let Some(pos) = &error.position {
                diagnostic = diagnostic.with_span(pos.line, pos.column, self.identifier_width(pos));
                // Add source line context
                if let Some(source_line) = self.get_source_line(pos.line) {
                    diagnostic = diagnostic.with_source_line(source_line);
                }
            }
            diagnostics.push(diagnostic);
        }

        diagnostics.extend(self.warning_diagnostics());

        (diagnostics, has_errors)
    }

    // REPL-specific linting that maintains state between statements
    pub fn lint_repl(&mut self, statements: &[Stmt]) -> (Vec<Diagnostic>, bool) {
        self.next_return_token = 0;
        self.next_loop_control_token = 0;
        // Only clear warnings and errors, keep global scope variables and functions
        self.warnings.clear();
        self.reset_declaration_positions();
        self.errors.clear();
        self.module_interfaces.borrow_mut().clear();

        // Ensure we have at least the global scope
        if self.scopes.is_empty() {
            self.scopes.push(HashMap::new());
        }

        // Make sure built-in functions are always available
        self.add_builtin_functions();

        self.in_function = false;
        self.in_loop = false;
        self.predeclare_functions(statements);

        // Single pass: analyze all statements
        for stmt in statements {
            self.analyze_statement(stmt);
        }

        // Don't check for unused variables in REPL mode - they might be used later

        // Convert to diagnostics
        let mut diagnostics = Vec::new();
        let mut has_errors = false;

        for error in &self.errors {
            has_errors = true;
            let mut diagnostic = Diagnostic::error(error.message.clone())
                .with_code(Self::error_code(&error.message).to_string());
            if Self::error_code(&error.message) == "E3101" {
                diagnostic = diagnostic.with_help(
                    "check the import path, package dependency, and exported name".into(),
                );
            }
            if let Some(pos) = &error.position {
                diagnostic = diagnostic.with_span(pos.line, pos.column, self.identifier_width(pos));
                // Add source line context
                if let Some(source_line) = self.get_source_line(pos.line) {
                    diagnostic = diagnostic.with_source_line(source_line);
                }
            }
            diagnostics.push(diagnostic);
        }

        diagnostics.extend(self.warning_diagnostics());

        (diagnostics, has_errors)
    }

    fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn warning_diagnostic(&self, warning: &LintWarning) -> Diagnostic {
        let mut diagnostic = Diagnostic::warning(warning.message.clone()).with_code("W2001".into());
        if let Some(pos) = &warning.position {
            diagnostic = diagnostic.with_span(pos.line, pos.column, self.identifier_width(pos));
            if let Some(source_line) = self.get_source_line(pos.line) {
                diagnostic = diagnostic.with_source_line(source_line);
            }
        }
        if let Some(pos) = &warning.previous {
            diagnostic = diagnostic.with_secondary_label(
                crate::diagnostic::SourceSpan::new(
                    pos.line,
                    pos.column,
                    self.identifier_width(pos),
                ),
                "previous declaration here",
            );
        }
        diagnostic
    }

    fn warning_diagnostics(&self) -> Vec<Diagnostic> {
        let mut warnings: Vec<_> = self.warnings.iter().collect();
        warnings.sort_by_key(|warning| {
            (
                warning
                    .position
                    .as_ref()
                    .map(|position| (position.line, position.column)),
                warning.message.as_str(),
            )
        });
        warnings
            .into_iter()
            .map(|warning| self.warning_diagnostic(warning))
            .collect()
    }

    fn identifier_width(&self, pos: &crate::lexer::SourcePosition) -> usize {
        self.positioned_tokens
            .binary_search_by_key(&(pos.line, pos.column), |(_, token)| {
                (token.line, token.column)
            })
            .ok()
            .and_then(|index| match &self.positioned_tokens[index].0 {
                crate::lexer::Token::Identifier(name) => Some(name.chars().count()),
                _ => None,
            })
            .unwrap_or(1)
    }

    fn pop_scope(&mut self) {
        if self.scopes.len() > 1 {
            // Before popping, check for unused variables in this scope
            if let Some(scope) = self.scopes.last() {
                for var in scope.values() {
                    if !var.used && !Self::should_skip_unused_warning(var) {
                        self.warnings.push(LintWarning {
                            message: format!("unused variable `{}`", var.name),
                            position: var.declaration.clone(),
                            previous: None,
                        });
                    }
                }
            }
            self.scopes.pop();
        }
    }

    fn declare_variable(&mut self, name: String) {
        self.declare_binding(name, true, Some(SymbolKind::Variable));
    }

    fn declare_constant(&mut self, name: String) {
        self.declare_binding(name, false, Some(SymbolKind::Constant));
    }

    fn declare_parameter(&mut self, name: String) {
        self.declare_binding(name, true, Some(SymbolKind::Parameter));
    }

    fn declare_import(&mut self, name: String, kind: SymbolKind) {
        self.declare_binding(name, false, Some(kind));
    }

    fn declare_external_constant(&mut self, name: String) {
        self.declare_binding_with_unused_warning(name, false, None, false);
    }

    fn declare_binding(&mut self, name: String, mutable: bool, kind: Option<SymbolKind>) {
        self.declare_binding_with_unused_warning(name, mutable, kind, true);
    }

    fn declare_binding_with_unused_warning(
        &mut self,
        name: String,
        mutable: bool,
        kind: Option<SymbolKind>,
        warn_if_unused: bool,
    ) {
        let is_function = kind == Some(SymbolKind::Function);
        let declaration = kind.and_then(|kind| {
            self.declaration_positions
                .get_mut(&(name.clone(), kind))
                .and_then(VecDeque::pop_front)
        });
        let reported_duplicate = is_function
            && self.scopes.len() == 1
            && self.record_module_function(&name, declaration.clone());
        let previous_binding = self.scopes.last().and_then(|scope| scope.get(&name));
        if let Some(previous) = previous_binding.filter(|binding| {
            !reported_duplicate
                && (!is_function || binding.warn_if_unused || binding.declaration.is_some())
        }) {
            self.warnings.push(LintWarning {
                message: format!("variable `{}` is already declared in this scope", name),
                position: declaration.clone(),
                previous: previous.declaration.clone(),
            });
        }
        if let Some(current_scope) = self.scopes.last_mut() {
            current_scope.insert(
                name.clone(),
                Variable {
                    name,
                    used: false,
                    mutable,
                    warn_if_unused,
                    declaration,
                    module_interface: None,
                    function_signature: None,
                },
            );
        }
    }

    fn binding_is_mutable(&self, name: &str) -> Option<bool> {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(name).map(|variable| variable.mutable))
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

    fn get_source_line(&self, line_num: usize) -> Option<String> {
        if line_num == 0 {
            return None;
        }
        let lines: Vec<&str> = self.source.lines().collect();
        if line_num <= lines.len() {
            Some(lines[line_num - 1].to_string())
        } else {
            None
        }
    }

    fn get_module_exports(&self, module_path: &str) -> Result<Vec<String>, String> {
        self.get_module_interface(module_path).map(|interface| {
            interface
                .exports
                .into_iter()
                .map(|export| export.name)
                .collect()
        })
    }

    fn get_module_interface(&self, module_path: &str) -> Result<ModuleInterface, String> {
        if module_path == "base" {
            return Ok(ModuleInterface {
                exports: crate::stdlib::get_base_module()
                    .into_keys()
                    .map(|name| ModuleExport {
                        name,
                        kind: ExportKind::Value,
                    })
                    .collect(),
            });
        }
        if let Some(interface) = self.module_interfaces.borrow().get(module_path) {
            return interface.clone();
        }
        let result = crate::module_resolver::ModuleResolver::for_path(&self.source_path)
            .resolve_with_sources(module_path, &self.source_overrides)
            .map_err(|error| error.to_string())
            .and_then(|module| {
                ModuleInterface::parse(&module.source)
                    .map_err(|error| format!("cannot inspect module '{module_path}': {error}"))
            });
        self.module_interfaces
            .borrow_mut()
            .insert(module_path.to_string(), result.clone());
        result
    }

    fn find_variable(&self, name: &str) -> bool {
        for scope in self.scopes.iter().rev() {
            if scope.contains_key(name) {
                return true;
            }
        }
        false
    }

    fn find_similar_variables(&self, name: &str) -> Vec<String> {
        let mut suggestions = Vec::new();
        let name_lower = name.to_lowercase();

        // Check all scopes for similar variable names
        for scope in &self.scopes {
            for var_name in scope.keys() {
                let var_lower = var_name.to_lowercase();
                // Simple similarity check - same length and similar characters
                if var_lower.chars().count() == name_lower.chars().count() {
                    let mut diff = 0;
                    for (c1, c2) in name_lower.chars().zip(var_lower.chars()) {
                        if c1 != c2 {
                            diff += 1;
                        }
                    }
                    // If only 1-2 characters differ, suggest it
                    if diff <= 2 && diff > 0 {
                        suggestions.push(var_name.clone());
                    }
                } else if var_lower.contains(&name_lower) || name_lower.contains(&var_lower) {
                    // Partial match
                    suggestions.push(var_name.clone());
                }
            }
        }

        // Also check function names
        for func_name in &self.functions {
            let func_lower = func_name.to_lowercase();
            if func_lower == name_lower {
                suggestions.push(func_name.clone());
            }
        }

        suggestions.sort();
        suggestions.dedup();
        suggestions
    }

    fn analyze_statement(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Let { name, value } => {
                // Analyze the value expression first
                self.analyze_expr(value);
                // Then declare the variable (Rust-like: can't use variable before declaration)
                self.declare_variable(name.clone());
            }
            Stmt::Const { name, value } => {
                self.analyze_expr(value);
                self.declare_constant(name.clone());
            }
            Stmt::Assign { target, value } => {
                // Analyze the value expression first
                self.analyze_expr(value);
                let root = match target {
                    crate::ast::AssignTarget::Identifier(name)
                    | crate::ast::AssignTarget::Index { object: name, .. }
                    | crate::ast::AssignTarget::IndexChain { object: name, .. }
                    | crate::ast::AssignTarget::Field { object: name, .. } => name,
                };
                if self.binding_is_mutable(root) == Some(false) {
                    self.errors.push(LintError {
                        message: format!("cannot assign through immutable binding `{root}`"),
                        position: self.find_identifier_position(root),
                    });
                }
                // Check if variable exists based on target type
                match target {
                    crate::ast::AssignTarget::Identifier(name) => {
                        if !self.use_variable(name) {
                            let position = self.find_identifier_position(name);
                            self.errors.push(LintError {
                                message: format!("cannot assign to undeclared variable `{}`", name),
                                position,
                            });
                        }
                    }
                    crate::ast::AssignTarget::Index { object, index } => {
                        // Check if the array variable exists
                        if !self.use_variable(object) {
                            let position = self.find_identifier_position(object);
                            self.errors.push(LintError {
                                message: format!(
                                    "cannot index into undeclared variable `{}`",
                                    object
                                ),
                                position,
                            });
                        }
                        // Analyze the index expression
                        self.analyze_expr(index);
                    }
                    crate::ast::AssignTarget::IndexChain { object, indices } => {
                        if !self.use_variable(object) {
                            let position = self.find_identifier_position(object);
                            self.errors.push(LintError {
                                message: format!(
                                    "cannot chain-index assign on undeclared variable `{}`",
                                    object
                                ),
                                position,
                            });
                        }
                        for ix in indices {
                            self.analyze_expr(ix);
                        }
                    }
                    crate::ast::AssignTarget::Field { object, field: _ } => {
                        // Check if the object variable exists
                        if !self.use_variable(object) {
                            let position = self.find_identifier_position(object);
                            self.errors.push(LintError {
                                message: format!(
                                    "cannot assign field on undeclared variable `{}`",
                                    object
                                ),
                                position,
                            });
                        }
                    }
                }
            }
            Stmt::Function { name, params, body } => {
                if self.scopes.len() > 1 {
                    self.declare_binding_with_unused_warning(
                        name.clone(),
                        true,
                        Some(SymbolKind::Function),
                        false,
                    );
                    self.set_binding_signature(name, params);
                } else {
                    let declaration = self
                        .declaration_positions
                        .get_mut(&(name.clone(), SymbolKind::Function))
                        .and_then(VecDeque::pop_front);
                    self.record_module_function(name, declaration);
                }

                // Create new scope for function
                self.push_scope();
                let was_in_function = self.in_function;
                self.in_function = true;
                let was_in_loop = std::mem::replace(&mut self.in_loop, false);

                // Add parameters to function scope and analyze default values
                for param in params {
                    // Analyze default value first (before declaring the parameter)
                    if let Some(default_expr) = &param.default_value {
                        self.analyze_expr(default_expr);
                    }
                    self.declare_parameter(param.name.clone());
                }

                // Analyze function body
                for stmt in body {
                    self.analyze_statement(stmt);
                }

                self.in_function = was_in_function;
                self.in_loop = was_in_loop;
                self.pop_scope();
            }
            Stmt::If {
                condition,
                then_branch,
                elseif_branches,
                else_branch,
            } => {
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
                let was_in_loop = std::mem::replace(&mut self.in_loop, true);
                for stmt in body {
                    self.analyze_statement(stmt);
                }
                self.in_loop = was_in_loop;
                self.pop_scope();
            }
            Stmt::For {
                var,
                iterable,
                body,
            } => {
                self.analyze_expr(iterable);
                // For loop creates its own scope with the loop variable
                self.push_scope();
                self.declare_variable(var.clone());
                let was_in_loop = std::mem::replace(&mut self.in_loop, true);
                for stmt in body {
                    self.analyze_statement(stmt);
                }
                self.in_loop = was_in_loop;
                self.pop_scope();
            }
            Stmt::Return(Some(expr)) => {
                let expression_start = match expr {
                    Expr::Located { span, .. } => Some((span.line, span.column)),
                    _ => None,
                };
                let position = self.take_return_position(expression_start);
                self.analyze_expr(expr);
                if !self.in_function {
                    self.errors.push(LintError {
                        message: "`return` outside of function".to_string(),
                        position,
                    });
                }
            }
            Stmt::Return(None) => {
                let position = self.take_return_position(None);
                if !self.in_function {
                    self.errors.push(LintError {
                        message: "`return` outside of function".to_string(),
                        position,
                    });
                }
            }
            Stmt::Print { values, sep, end } => {
                for value in values {
                    self.analyze_expr(value);
                }
                if let Some(s) = sep {
                    self.analyze_expr(s);
                }
                if let Some(e) = end {
                    self.analyze_expr(e);
                }
            }
            Stmt::Expression(expr) => {
                self.analyze_expr(expr);
            }
            Stmt::Import {
                module_path,
                items,
                alias,
            } => {
                // Private native backends are supplied by the package ABI loader,
                // not by a .ject source module. Their public facade is still
                // checked normally; this boundary only skips source lookup.
                let module_api = if module_path.starts_with("@native/") {
                    None
                } else {
                    match self.get_module_interface(module_path) {
                        Ok(interface) => Some(interface),
                        Err(message) => {
                            self.errors.push(LintError {
                                message,
                                position: self.find_module_position(module_path),
                            });
                            None
                        }
                    }
                };
                // Handle selective imports
                if let Some(item_list) = items {
                    for item in item_list {
                        self.declare_import(item.clone(), SymbolKind::Import);
                        let mut valid_export = false;
                        if let Some(export) = module_api
                            .as_ref()
                            .and_then(|interface| interface.export(item))
                        {
                            valid_export = true;
                            if let ExportKind::Function { parameters } = &export.kind {
                                self.set_binding_signature(item, parameters);
                            }
                        } else if let Some(interface) = &module_api {
                            if interface.export(item).is_none() {
                                self.errors.push(LintError {
                                    message: format!(
                                        "module '{module_path}' does not export '{item}'"
                                    ),
                                    position: self.find_identifier_position(item),
                                });
                            }
                        }
                        if module_api.is_none() || !valid_export {
                            // Keep recovery symbols from producing a secondary
                            // unused warning after the actionable import error.
                            self.use_variable(item);
                        }
                    }
                } else if alias.is_none() {
                    if let Some(interface) = &module_api {
                        for export in &interface.exports {
                            self.declare_external_constant(export.name.clone());
                            if let ExportKind::Function { parameters } = &export.kind {
                                self.set_binding_signature(&export.name, parameters);
                            }
                        }
                    }
                }

                if let Some(alias_name) = alias {
                    self.declare_import(alias_name.clone(), SymbolKind::Module);
                    if module_api.is_none() {
                        self.use_variable(alias_name);
                    }
                    if let (Some(interface), Some(binding)) = (
                        &module_api,
                        self.scopes
                            .last_mut()
                            .and_then(|scope| scope.get_mut(alias_name)),
                    ) {
                        binding.module_interface = Some(interface.clone());
                    }
                }
            }
            Stmt::Export {
                name,
                value,
                mutable,
            } => {
                self.analyze_expr(value);
                if *mutable {
                    self.declare_variable(name.clone());
                } else {
                    self.declare_constant(name.clone());
                }
                // An export is consumed by other modules, outside this file's
                // analysis boundary. Treat publication as a use.
                self.use_variable(name);
            }
            Stmt::ExportFunction { name, params, body } => {
                self.declare_binding(name.clone(), true, Some(SymbolKind::Function));
                self.set_binding_signature(name, params);
                // Public functions are API surface even when this module does
                // not call them internally.
                self.use_variable(name);
                self.push_scope();

                let was_in_function = self.in_function;
                self.in_function = true;
                let was_in_loop = std::mem::replace(&mut self.in_loop, false);

                for param in params {
                    // Analyze default value first
                    if let Some(default_expr) = &param.default_value {
                        self.analyze_expr(default_expr);
                    }
                    self.declare_parameter(param.name.clone());
                }

                for stmt in body {
                    self.analyze_statement(stmt);
                }

                self.in_function = was_in_function;
                self.in_loop = was_in_loop;
                self.pop_scope();
            }
            Stmt::Struct { name, fields: _ } => {
                // Struct definitions are type definitions, not variables
                // They should be tracked separately, but for now just declare them
                // so they don't trigger "undeclared" errors
                self.declare_binding(name.clone(), true, Some(SymbolKind::Struct));
                // Mark as used immediately so it doesn't show as unused
                if let Some(scope) = self.scopes.last_mut() {
                    if let Some(var) = scope.get_mut(name) {
                        var.used = true; // Struct definitions are always "used"
                    }
                }
            }
            Stmt::Try {
                body,
                catch_var,
                catch_body,
            } => {
                self.push_scope();
                for stmt in body {
                    self.analyze_statement(stmt);
                }
                self.pop_scope();

                self.push_scope();
                if let Some(var_name) = catch_var {
                    self.declare_variable(var_name.clone());
                }
                for stmt in catch_body {
                    self.analyze_statement(stmt);
                }
                self.pop_scope();
            }
            Stmt::Throw(expr) => {
                self.analyze_expr(expr);
            }
            Stmt::Break | Stmt::Continue => {
                let is_break = matches!(stmt, Stmt::Break);
                let position = self.take_loop_control_position(is_break);
                if !self.in_loop {
                    let keyword = if is_break { "break" } else { "continue" };
                    self.errors.push(LintError {
                        message: format!("`{keyword}` outside of loop"),
                        position,
                    });
                }
            }
        }
    }

    fn analyze_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Located { expression, span } => {
                let previous = self.expression_start.replace((span.line, span.column));
                self.analyze_expr(expression);
                self.expression_start = previous;
            }
            Expr::Identifier(name) => {
                // Check if it's a function first, then check if it's a variable
                // Prefer lexical variable bindings over builtins (e.g. param `min` vs `min()`)
                if !self.use_variable(name) && !self.functions.contains(name) {
                    let position = self
                        .positioned_tokens
                        .iter()
                        .find_map(|(token, position)| {
                            (matches!(token, crate::lexer::Token::Identifier(identifier) if identifier == name)
                                && self.expression_start.is_none_or(|start| {
                                    (position.line, position.column) >= start
                                }))
                            .then(|| position.clone())
                        })
                        .or_else(|| self.find_identifier_position(name));

                    // Try to find similar variable names for suggestions
                    let suggestions = self.find_similar_variables(name);
                    let mut message = format!("undefined variable `{}`", name);

                    if !suggestions.is_empty() {
                        if suggestions.len() == 1 {
                            message
                                .push_str(&format!("\n  help: did you mean `{}`?", suggestions[0]));
                        } else {
                            message.push_str("\n  help: did you mean one of these?");
                            for sug in &suggestions[..suggestions.len().min(3)] {
                                message.push_str(&format!("\n    - `{}`", sug));
                            }
                        }
                    } else {
                        if let Some(module_name) = self.find_module_exporting_symbol(name) {
                            message.push_str(&format!(
                                "\n  help: import it first: import {{{}}} from \"{}\"",
                                name, module_name
                            ));
                        } else {
                            message.push_str(
                                "\n  help: variables must be declared with `let` before use",
                            );
                        }
                    }

                    self.errors.push(LintError { message, position });
                }
            }
            Expr::Binary { left, right, .. } => {
                self.analyze_expr(left);
                self.analyze_expr(right);
            }
            Expr::Unary { operand, .. } => {
                self.analyze_expr(operand);
            }
            Expr::Increment { target, .. } | Expr::Decrement { target, .. } => {
                self.analyze_expr(target);
                if let Expr::Identifier(name) = target.as_ref() {
                    if self.binding_is_mutable(name) == Some(false) {
                        self.errors.push(LintError {
                            message: format!("cannot assign through immutable binding `{name}`"),
                            position: self.find_identifier_position(name),
                        });
                    }
                }
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
                } else if let Expr::StructAccess { object, field } = callee.as_ref() {
                    if let Expr::Identifier(alias) = object.as_ref() {
                        if let Some((signature, position)) = self
                            .scopes
                            .iter()
                            .rev()
                            .find_map(|scope| scope.get(alias))
                            .and_then(|binding| {
                                let export = binding.module_interface.as_ref()?.export(field)?;
                                let ExportKind::Function { parameters } = &export.kind else {
                                    return None;
                                };
                                Some((
                                    FunctionSignature {
                                        parameters: parameters.clone(),
                                    },
                                    binding.declaration.clone(),
                                ))
                            })
                        {
                            self.validate_call_against_signature(
                                &format!("{alias}.{field}"),
                                &signature,
                                args,
                                position,
                            );
                        }
                    }
                }
            }
            Expr::Index { object, index } => {
                self.analyze_expr(object);
                self.analyze_expr(index);
            }
            Expr::Slice {
                object,
                from,
                to,
                step,
            } => {
                self.analyze_expr(object);
                for bound in [from.as_deref(), to.as_deref(), step.as_deref()]
                    .into_iter()
                    .flatten()
                {
                    self.analyze_expr(bound);
                }
            }
            Expr::StructAccess { object, field } => {
                self.analyze_expr(object);
                if let Expr::Identifier(alias) = object.as_ref() {
                    if let Some(binding) =
                        self.scopes.iter().rev().find_map(|scope| scope.get(alias))
                    {
                        if binding
                            .module_interface
                            .as_ref()
                            .is_some_and(|api| api.export(field).is_none())
                        {
                            self.errors.push(LintError {
                                message: format!(
                                    "module alias '{alias}' does not export '{field}'"
                                ),
                                position: binding.declaration.clone(),
                            });
                        }
                    }
                }
            }
            Expr::StructInit {
                struct_name,
                fields,
            } => {
                // Check if struct is defined
                if !self.find_variable(struct_name) {
                    let position = self.find_identifier_position(struct_name);
                    self.errors.push(LintError {
                        message: format!("use of undeclared struct `{}`", struct_name),
                        position,
                    });
                }
                // Analyze field values
                for (_, field_value) in fields {
                    self.analyze_expr(field_value);
                }
            }
            Expr::Array(elements) | Expr::UniqueArray(elements) => {
                for elem in elements {
                    self.analyze_expr(elem);
                }
            }
            Expr::ListComprehension {
                expr,
                var,
                iterable,
                condition,
            } => {
                self.analyze_expr(iterable);
                self.push_scope();
                self.declare_variable(var.clone());
                self.analyze_expr(expr);
                if let Some(cond) = condition {
                    self.analyze_expr(cond);
                }
                self.pop_scope();
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
                let was_in_function = self.in_function;
                self.in_function = true;
                let was_in_loop = std::mem::replace(&mut self.in_loop, false);
                // Declare parameters
                for param in params {
                    self.declare_parameter(param.clone());
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
                self.in_function = was_in_function;
                self.in_loop = was_in_loop;
                self.pop_scope();
            }
            Expr::Match { expr, arms } => {
                self.analyze_expr(expr);
                for arm in arms {
                    // Each match arm gets its own scope for pattern bindings
                    self.push_scope();
                    for pattern in &arm.patterns {
                        self.analyze_pattern(pattern);
                    }
                    if let Some(guard) = &arm.guard {
                        self.analyze_expr(guard);
                    }
                    match &arm.body {
                        crate::ast::MatchArmBody::Expression(expr) => {
                            self.analyze_expr(expr);
                        }
                        crate::ast::MatchArmBody::Block(stmts) => {
                            for stmt in stmts {
                                self.analyze_statement(stmt);
                            }
                        }
                    }
                    self.pop_scope();
                }
            }
            Expr::ConditionalExpr {
                condition,
                then_expr,
                elseif_branches,
                else_expr,
            } => {
                self.analyze_expr(condition);
                self.analyze_expr(then_expr);
                for branch in elseif_branches {
                    self.analyze_expr(&branch.condition);
                    self.analyze_expr(&branch.then_expr);
                }
                if let Some(expression) = else_expr {
                    self.analyze_expr(expression);
                }
            }
            Expr::Print { values, sep, end } => {
                for v in values {
                    self.analyze_expr(v);
                }
                if let Some(sep) = sep {
                    self.analyze_expr(sep);
                }
                if let Some(end) = end {
                    self.analyze_expr(end);
                }
            }
            Expr::CompiledInterpolation(parts) => {
                for part in parts {
                    self.analyze_expr(part);
                }
            }
            Expr::InterpolatedString(parts) => {
                for part in parts {
                    if let crate::lexer::InterpolationPart::Expression(expr_str) = part {
                        // Interpolated expressions are stored as raw source text and only
                        // parsed at evaluation time, so parse them here too, purely to
                        // discover which variables they reference (marking them used).
                        let mut lexer = crate::lexer::Lexer::new(expr_str);
                        let located_tokens = lexer.tokenize_with_positions();
                        let tokens: Vec<crate::lexer::Token> =
                            located_tokens.into_iter().map(|lt| lt.token).collect();
                        let mut parser = crate::parser::Parser::new_simple(tokens);
                        if let Ok(statements) = parser.parse() {
                            for stmt in &statements {
                                if let Stmt::Expression(inner_expr) = stmt {
                                    self.analyze_expr(inner_expr);
                                }
                            }
                        }
                    }
                }
            }

            // Literals don't need analysis
            Expr::Integer(_) | Expr::Float(_) | Expr::String(_) | Expr::Bool(_) | Expr::Nil => {}
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
            crate::ast::Pattern::Relational(_, expr) => {
                self.analyze_expr(expr);
            }
            crate::ast::Pattern::Range(start, end) => {
                self.analyze_expr(start);
                self.analyze_expr(end);
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

    fn take_return_position(
        &mut self,
        before: Option<(usize, usize)>,
    ) -> Option<crate::lexer::SourcePosition> {
        let (index, (_, position)) = self
            .positioned_tokens
            .iter()
            .enumerate()
            .skip(self.next_return_token)
            .find(|(_, (token, _))| matches!(token, crate::lexer::Token::Return))?;
        // Arrow bodies lower to a return without a corresponding source token.
        if before.is_some_and(|start| (position.line, position.column) >= start) {
            return None;
        }
        self.next_return_token = index + 1;
        Some(position.clone())
    }

    fn take_loop_control_position(
        &mut self,
        is_break: bool,
    ) -> Option<crate::lexer::SourcePosition> {
        use crate::lexer::Token;

        let (index, (_, position)) = self
            .positioned_tokens
            .iter()
            .enumerate()
            .skip(self.next_loop_control_token)
            .find(|(_, (token, _))| {
                matches!(
                    (token, is_break),
                    (Token::Break, true) | (Token::Continue, false)
                )
            })?;
        self.next_loop_control_token = index + 1;
        Some(position.clone())
    }

    fn find_module_position(&self, module: &str) -> Option<crate::lexer::SourcePosition> {
        self.positioned_tokens.iter().find_map(|(token, position)| {
            matches!(token, crate::lexer::Token::String(value) if value == module)
                .then(|| position.clone())
        })
    }

    fn find_module_exporting_symbol(&self, symbol: &str) -> Option<String> {
        let common_modules = [
            "math",
            "string",
            "array",
            "io",
            "json",
            "system",
            "datetime",
            "util",
            "collections",
        ];
        for module in common_modules {
            if let Ok(exports) = self.get_module_exports(module) {
                if exports.iter().any(|e| e == symbol) {
                    return Some(module.to_string());
                }
            }
        }
        None
    }

    fn set_binding_signature(&mut self, name: &str, parameters: &[Parameter]) {
        if let Some(binding) = self.scopes.last_mut().and_then(|scope| scope.get_mut(name)) {
            binding.function_signature = Some(FunctionSignature {
                parameters: parameters.to_vec(),
            });
        }
    }

    fn validate_function_call(&mut self, func_name: &str, args: &[Argument]) {
        let binding = self
            .scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(func_name));
        let signature = match binding {
            Some(binding) => binding.function_signature.as_ref(),
            None => self.function_signatures.get(func_name),
        };
        if let Some(signature) = signature.cloned() {
            let position = binding
                .and_then(|binding| binding.declaration.clone())
                .or_else(|| self.find_identifier_position(func_name));
            self.validate_call_against_signature(func_name, &signature, args, position);
        }
        // If function signature not found, we already reported "undeclared variable" error
    }

    fn validate_call_against_signature(
        &mut self,
        func_name: &str,
        signature: &FunctionSignature,
        args: &[Argument],
        position: Option<crate::lexer::SourcePosition>,
    ) {
        // Simulate the argument resolution logic from the interpreter
        let mut resolved_args = vec![false; signature.parameters.len()]; // track which args are provided
        let mut positional_count = 0;

        // First pass: handle positional arguments
        for arg in args {
            match arg {
                Argument::Positional(_) => {
                    if positional_count >= signature.parameters.len() {
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
                            self.errors.push(LintError {
                                message: format!(
                                    "argument `{}` specified multiple times in call to `{}`",
                                    name, func_name
                                ),
                                position,
                            });
                            return;
                        }
                        resolved_args[index] = true;
                    }
                    None => {
                        self.errors.push(LintError {
                            message: format!(
                                "unknown parameter `{}` for function `{}`",
                                name, func_name
                            ),
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
                self.errors.push(LintError {
                    message: format!(
                        "missing required argument `{}` for function `{}`",
                        param.name, func_name
                    ),
                    position: position.clone(),
                });
            }
        }
    }
}
