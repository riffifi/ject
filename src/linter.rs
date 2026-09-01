use crate::ast::{Argument, Expr, Parameter, Stmt};
use crate::diagnostic::Diagnostic;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
struct Variable {
    name: String,
    used: bool,
}

#[derive(Debug, Clone)]
struct FunctionSignature {
    parameters: Vec<Parameter>,
}

#[derive(Debug, Clone)]
struct ModuleExport {
    name: String,
    parameters: Option<Vec<Parameter>>,
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

#[derive(Clone)]
pub struct Linter {
    scopes: Vec<HashMap<String, Variable>>, // Stack of scopes with variable info
    warnings: Vec<LintWarning>,
    errors: Vec<LintError>,
    functions: HashSet<String>,
    function_signatures: HashMap<String, FunctionSignature>, // Track function signatures
    module_signatures: HashMap<String, HashMap<String, FunctionSignature>>,
    in_function: bool,
    // Store positioned tokens to find locations of identifiers
    positioned_tokens: Vec<(crate::lexer::Token, crate::lexer::SourcePosition)>,
    source: String,
    source_path: std::path::PathBuf,
}

impl Linter {
    fn error_code(message: &str) -> &'static str {
        if message.contains("module '")
            || message.contains("cannot inspect module")
            || message.contains("does not export")
        {
            "E3101"
        } else {
            "E2001"
        }
    }

    fn should_skip_unused_warning(name: &str) -> bool {
        if name.starts_with('_') {
            return true;
        }
        matches!(
            name,
            "PI" | "E"
                | "type_of"
                | "to_int"
                | "to_float"
                | "to_string"
                | "to_bool"
                | "len"
                | "range"
                | "push"
                | "pop"
                | "sum"
                | "contains"
                | "index_of"
                | "first"
                | "last"
                | "sort"
                | "reverse"
                | "unique"
                | "map"
                | "filter"
                | "reduce"
                | "abs"
                | "sqrt"
                | "pow"
                | "sin"
                | "cos"
                | "tan"
                | "floor"
                | "ceil"
                | "round"
                | "min"
                | "max"
                | "random"
                | "upper"
                | "lower"
                | "trim"
                | "split"
                | "join"
                | "replace"
                | "char_at"
                | "substring"
                | "input"
                | "print"
                | "read_file"
                | "write_file"
                | "assert"
                | "log"
                | "log10"
                | "exp"
                | "log2"
                | "ln"
                | "degrees"
                | "radians"
                | "deg_to_rad"
                | "rad_to_deg"
                | "clamp"
                | "sign"
                | "gcd"
                | "lcm"
                | "asin"
                | "acos"
                | "atan"
                | "atan2"
                | "sinh"
                | "cosh"
                | "tanh"
                | "round_to"
                | "capitalize"
                | "title_case"
                | "trim_left"
                | "trim_right"
                | "pad_left"
                | "pad_right"
                | "pad_center"
                | "starts_with"
                | "ends_with"
                | "contains_str"
                | "count"
                | "find"
                | "replace_all"
                | "replace_first"
                | "remove"
                | "repeat"
                | "reverse_str"
                | "left"
                | "right"
                | "truncate"
                | "is_empty"
                | "is_numeric"
                | "is_alpha"
                | "is_alphanumeric"
                | "word_count"
                | "sentence_count"
                | "paragraph_count"
                | "lines"
                | "extract_numbers"
                | "to_char_codes"
                | "from_char_codes"
                | "format"
                | "escape"
                | "unescape"
                | "wrap_text"
                | "any"
                | "all"
                | "average"
                | "median"
                | "slice"
                | "take"
                | "drop"
                | "initial"
                | "rest"
                | "concat"
                | "zip"
                | "union"
                | "intersection"
                | "difference"
                | "flatten"
                | "chunk"
                | "group_by"
                | "partition"
                | "shuffle"
                | "rotate_left"
                | "rotate_right"
                | "insert_at"
                | "remove_at"
                | "without"
                | "compact"
                | "compact_unique"
                | "enumerate"
                | "fill"
                | "range_arr"
                | "sample"
                | "sort_by"
                | "to_uarray"
                | "to_array"
                | "parse_json"
                | "to_json"
                | "env"
                | "exit"
                | "now"
                | "timestamp"
                | "sleep"
                | "to_binary"
                | "to_octal"
                | "to_hex"
                | "from_binary"
                | "from_octal"
                | "from_hex"
                | "base_repr"
                | "from_base"
                | "random_int"
                | "random_float"
                | "println"
                | "exec"
                | "file_exists"
                | "is_file"
                | "is_dir"
                | "list_dir"
                | "mkdir"
                | "remove_file"
                | "collection"
                | "add_to"
                | "remove_from"
                | "has"
                | "intersect"
                | "size"
                | "is_subset"
                | "is_superset"
                | "clear_collection"
        )
    }

    pub fn new() -> Self {
        let mut linter = Linter {
            scopes: vec![HashMap::new()], // Global scope
            warnings: Vec::new(),
            errors: Vec::new(),
            functions: HashSet::new(),
            function_signatures: HashMap::new(),
            module_signatures: HashMap::new(),
            in_function: false,
            positioned_tokens: Vec::new(),
            source: String::new(),
            source_path: std::env::current_dir().unwrap_or_default(),
        };

        // Add built-in functions to the functions set
        linter.add_builtin_functions();
        linter
    }

    fn add_builtin_functions(&mut self) {
        // ========== CorLib Functions (always available) ==========

        // Type inspection & conversion
        self.functions.insert("type_of".to_string());
        self.functions.insert("to_int".to_string());
        self.functions.insert("to_float".to_string());
        self.functions.insert("to_string".to_string());
        self.functions.insert("to_bool".to_string());

        // Collection primitives
        self.functions.insert("len".to_string());
        self.functions.insert("range".to_string());
        self.functions.insert("push".to_string());
        self.functions.insert("pop".to_string());

        // Array primitives
        self.functions.insert("sum".to_string());
        self.functions.insert("contains".to_string());
        self.functions.insert("index_of".to_string());
        self.functions.insert("first".to_string());
        self.functions.insert("last".to_string());
        self.functions.insert("sort".to_string());
        self.functions.insert("reverse".to_string());
        self.functions.insert("unique".to_string());

        // Higher-order functions
        self.functions.insert("map".to_string());
        self.functions.insert("filter".to_string());
        self.functions.insert("reduce".to_string());

        // Math primitives
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
        self.functions.insert("random".to_string());

        // String primitives
        self.functions.insert("upper".to_string());
        self.functions.insert("lower".to_string());
        self.functions.insert("trim".to_string());
        self.functions.insert("split".to_string());
        self.functions.insert("join".to_string());
        self.functions.insert("replace".to_string());
        self.functions.insert("char_at".to_string());
        self.functions.insert("substring".to_string());
        self.functions.insert("repeat".to_string());

        // I/O primitives
        self.functions.insert("input".to_string());
        self.functions.insert("print".to_string());
        self.functions.insert("read_file".to_string());
        self.functions.insert("write_file".to_string());
        self.functions.insert("append_file".to_string());
        self.functions.insert("read_lines".to_string());

        // Testing
        self.functions.insert("assert".to_string());
        self.functions.insert("has_key".to_string());
        self.functions.insert("delete".to_string());
        self.functions.insert("keys".to_string());
        self.functions.insert("values".to_string());

        // Constants (variables, not functions)
        self.declare_variable("PI".to_string());
        self.declare_variable("E".to_string());

        // ========== Module Functions (available via import) ==========

        // Math module (import "math")
        self.functions.insert("log".to_string());
        self.functions.insert("log10".to_string());
        self.functions.insert("exp".to_string());
        self.functions.insert("log2".to_string());
        self.functions.insert("ln".to_string());
        self.functions.insert("degrees".to_string());
        self.functions.insert("radians".to_string());
        self.functions.insert("deg_to_rad".to_string());
        self.functions.insert("rad_to_deg".to_string());
        self.functions.insert("clamp".to_string());
        self.functions.insert("sign".to_string());
        self.functions.insert("gcd".to_string());
        self.functions.insert("lcm".to_string());
        self.functions.insert("asin".to_string());
        self.functions.insert("acos".to_string());
        self.functions.insert("atan".to_string());
        self.functions.insert("atan2".to_string());
        self.functions.insert("sinh".to_string());
        self.functions.insert("cosh".to_string());
        self.functions.insert("tanh".to_string());
        self.functions.insert("round_to".to_string());

        // String module (import "string")
        self.functions.insert("capitalize".to_string());
        self.functions.insert("title_case".to_string());
        self.functions.insert("trim_left".to_string());
        self.functions.insert("trim_right".to_string());
        self.functions.insert("pad_left".to_string());
        self.functions.insert("pad_right".to_string());
        self.functions.insert("pad_center".to_string());
        self.functions.insert("starts_with".to_string());
        self.functions.insert("ends_with".to_string());
        self.functions.insert("contains_str".to_string());
        self.functions.insert("count".to_string());
        self.functions.insert("find".to_string());
        self.functions.insert("replace_all".to_string());
        self.functions.insert("replace_first".to_string());
        self.functions.insert("remove".to_string());
        self.functions.insert("repeat".to_string());
        self.functions.insert("reverse_str".to_string());
        self.functions.insert("left".to_string());
        self.functions.insert("right".to_string());
        self.functions.insert("truncate".to_string());
        self.functions.insert("is_empty".to_string());
        self.functions.insert("is_numeric".to_string());
        self.functions.insert("is_alpha".to_string());
        self.functions.insert("is_alphanumeric".to_string());
        self.functions.insert("word_count".to_string());
        self.functions.insert("sentence_count".to_string());
        self.functions.insert("paragraph_count".to_string());
        self.functions.insert("lines".to_string());
        self.functions.insert("extract_numbers".to_string());
        self.functions.insert("to_char_codes".to_string());
        self.functions.insert("from_char_codes".to_string());
        self.functions.insert("format".to_string());
        self.functions.insert("escape".to_string());
        self.functions.insert("unescape".to_string());
        self.functions.insert("wrap_text".to_string());

        // Array module (import "array")
        self.functions.insert("any".to_string());
        self.functions.insert("all".to_string());
        self.functions.insert("average".to_string());
        self.functions.insert("median".to_string());
        self.functions.insert("slice".to_string());
        self.functions.insert("take".to_string());
        self.functions.insert("drop".to_string());
        self.functions.insert("initial".to_string());
        self.functions.insert("rest".to_string());
        self.functions.insert("concat".to_string());
        self.functions.insert("zip".to_string());
        self.functions.insert("union".to_string());
        self.functions.insert("intersection".to_string());
        self.functions.insert("difference".to_string());
        self.functions.insert("flatten".to_string());
        self.functions.insert("chunk".to_string());
        self.functions.insert("group_by".to_string());
        self.functions.insert("partition".to_string());
        self.functions.insert("shuffle".to_string());
        self.functions.insert("rotate_left".to_string());
        self.functions.insert("rotate_right".to_string());
        self.functions.insert("insert_at".to_string());
        self.functions.insert("remove_at".to_string());
        self.functions.insert("without".to_string());
        self.functions.insert("compact".to_string());
        self.functions.insert("compact_unique".to_string());
        self.functions.insert("enumerate".to_string());
        self.functions.insert("fill".to_string());
        self.functions.insert("range_arr".to_string());
        self.functions.insert("sample".to_string());
        self.functions.insert("sort_by".to_string());
        self.functions.insert("to_uarray".to_string());
        self.functions.insert("to_array".to_string());

        // IO module (import "io")
        // read_file, write_file already in CorLib

        // JSON module (import "json")
        self.functions.insert("parse_json".to_string());
        self.functions.insert("to_json".to_string());

        // System module (import "system")
        self.functions.insert("env".to_string());
        self.functions.insert("exit".to_string());
        self.functions.insert("args".to_string());
        self.functions.insert("cwd".to_string());
        self.functions.insert("now".to_string());
        self.functions.insert("timestamp".to_string());
        self.functions.insert("sleep".to_string());

        // Base module (import "base")
        self.functions.insert("to_binary".to_string());
        self.functions.insert("to_octal".to_string());
        self.functions.insert("to_hex".to_string());
        self.functions.insert("from_binary".to_string());
        self.functions.insert("from_octal".to_string());
        self.functions.insert("from_hex".to_string());
        self.functions.insert("base_repr".to_string());
        self.functions.insert("from_base".to_string());

        // Additional utility functions
        self.functions.insert("random_int".to_string());
        self.functions.insert("random_float".to_string());
        self.functions.insert("println".to_string());

        // System utilities
        self.functions.insert("exec".to_string());
        self.functions.insert("file_exists".to_string());
        self.functions.insert("is_file".to_string());
        self.functions.insert("is_dir".to_string());
        self.functions.insert("list_dir".to_string());
        self.functions.insert("mkdir".to_string());
        self.functions.insert("remove_file".to_string());

        // Collection functions
        self.functions.insert("collection".to_string());
        self.functions.insert("add_to".to_string());
        self.functions.insert("remove_from".to_string());
        self.functions.insert("has".to_string());
        self.functions.insert("intersect".to_string());
        self.functions.insert("size".to_string());
        self.functions.insert("is_subset".to_string());
        self.functions.insert("is_superset".to_string());
        self.functions.insert("clear_collection".to_string());
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

    pub fn with_source_path(mut self, path: impl Into<std::path::PathBuf>) -> Self {
        self.source_path = path.into();
        self
    }

    pub fn lint(&mut self, statements: &[Stmt]) -> (Vec<Diagnostic>, bool) {
        self.scopes.clear();
        self.scopes.push(HashMap::new()); // Global scope
        self.warnings.clear();
        self.errors.clear();

        // Re-add built-in functions (don't clear them)
        self.functions.clear();
        self.add_builtin_functions();

        self.function_signatures.clear();
        self.module_signatures.clear();
        self.in_function = false;

        // Single pass: analyze all statements
        for stmt in statements {
            self.analyze_statement(stmt);
        }

        // Check for unused variables in all scopes
        for scope in &self.scopes {
            for var in scope.values() {
                // Skip warnings for stdlib constants and underscore-prefixed variables
                if !var.used && !Self::should_skip_unused_warning(&var.name) {
                    self.warnings.push(LintWarning {
                        message: format!("unused variable `{}`", var.name),
                        position: self.find_identifier_position(&var.name),
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
            }
            if let Some(pos) = &error.position {
                diagnostic = diagnostic.with_location(pos.line, pos.column);
                // Add source line context
                if let Some(source_line) = self.get_source_line(pos.line) {
                    diagnostic = diagnostic.with_source_line(source_line);
                }
            }
            diagnostics.push(diagnostic);
        }

        for warning in &self.warnings {
            let mut diagnostic =
                Diagnostic::warning(warning.message.clone()).with_code("W2001".to_string());
            if let Some(pos) = &warning.position {
                diagnostic = diagnostic.with_location(pos.line, pos.column);
                // Add source line context
                if let Some(source_line) = self.get_source_line(pos.line) {
                    diagnostic = diagnostic.with_source_line(source_line);
                }
            }
            diagnostics.push(diagnostic);
        }

        (diagnostics, has_errors)
    }

    // REPL-specific linting that maintains state between statements
    pub fn lint_repl(&mut self, statements: &[Stmt]) -> (Vec<Diagnostic>, bool) {
        // Only clear warnings and errors, keep global scope variables and functions
        self.warnings.clear();
        self.errors.clear();

        // Ensure we have at least the global scope
        if self.scopes.is_empty() {
            self.scopes.push(HashMap::new());
        }

        // Make sure built-in functions are always available
        self.add_builtin_functions();

        self.in_function = false;

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
                diagnostic = diagnostic.with_location(pos.line, pos.column);
                // Add source line context
                if let Some(source_line) = self.get_source_line(pos.line) {
                    diagnostic = diagnostic.with_source_line(source_line);
                }
            }
            diagnostics.push(diagnostic);
        }

        for warning in &self.warnings {
            let mut diagnostic =
                Diagnostic::warning(warning.message.clone()).with_code("W2001".to_string());
            if let Some(pos) = &warning.position {
                diagnostic = diagnostic.with_location(pos.line, pos.column);
                // Add source line context
                if let Some(source_line) = self.get_source_line(pos.line) {
                    diagnostic = diagnostic.with_source_line(source_line);
                }
            }
            diagnostics.push(diagnostic);
        }

        (diagnostics, has_errors)
    }

    fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        if self.scopes.len() > 1 {
            // Before popping, check for unused variables in this scope
            if let Some(scope) = self.scopes.last() {
                for var in scope.values() {
                    if !var.used && !Self::should_skip_unused_warning(&var.name) {
                        self.warnings.push(LintWarning {
                            message: format!("unused variable `{}`", var.name),
                            position: self.find_identifier_position(&var.name),
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
                // Don't warn about redeclaration of builtin constants in REPL mode
                // They may be re-initialized by the interpreter
                if name == "PI" || name == "E" {
                    return; // Silently ignore PI/E redeclaration
                }

                let position = self.find_identifier_position(&name);
                self.warnings.push(LintWarning {
                    message: format!(
                        "warning: variable `{}` is already declared in this scope",
                        name
                    ),
                    position,
                });
            } else {
                current_scope.insert(name.clone(), Variable { name, used: false });
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
        self.get_module_api(module_path)
            .map(|exports| exports.into_iter().map(|export| export.name).collect())
    }

    fn get_module_api(&self, module_path: &str) -> Result<Vec<ModuleExport>, String> {
        let module_content = crate::module_resolver::ModuleResolver::for_path(&self.source_path)
            .resolve(module_path)
            .map_err(|error| error.to_string())?
            .source;

        let mut lexer = crate::lexer::Lexer::new(&module_content);
        let located_tokens = lexer.tokenize_with_positions();
        let tokens: Vec<crate::lexer::Token> =
            located_tokens.into_iter().map(|lt| lt.token).collect();
        let mut parser = crate::parser::Parser::new_simple(tokens);

        let statements = parser
            .parse()
            .map_err(|error| format!("cannot inspect module '{module_path}': {error}"))?;

        // Extract export names
        let mut exports = Vec::new();
        for statement in &statements {
            match statement {
                crate::ast::Stmt::Export { name, .. } => {
                    exports.push(ModuleExport {
                        name: name.clone(),
                        parameters: None,
                    });
                }
                crate::ast::Stmt::ExportFunction { name, params, .. } => {
                    exports.push(ModuleExport {
                        name: name.clone(),
                        parameters: Some(params.clone()),
                    });
                }
                _ => {}
            }
        }

        Ok(exports)
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
                if var_lower.len() == name_lower.len() {
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
            Stmt::Assign { target, value } => {
                // Analyze the value expression first
                self.analyze_expr(value);
                // Check if variable exists based on target type
                match target {
                    crate::ast::AssignTarget::Identifier(name) => {
                        if !self.use_variable(&name) {
                            let position = self.find_identifier_position(&name);
                            self.errors.push(LintError {
                                message: format!("cannot assign to undeclared variable `{}`", name),
                                position,
                            });
                        }
                    }
                    crate::ast::AssignTarget::Index { object, index } => {
                        // Check if the array variable exists
                        if !self.use_variable(&object) {
                            let position = self.find_identifier_position(&object);
                            self.errors.push(LintError {
                                message: format!(
                                    "cannot index into undeclared variable `{}`",
                                    object
                                ),
                                position,
                            });
                        }
                        // Analyze the index expression
                        self.analyze_expr(&index);
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
                        if !self.use_variable(&object) {
                            let position = self.find_identifier_position(&object);
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
                self.function_signatures.insert(
                    name.clone(),
                    FunctionSignature {
                        parameters: params.clone(),
                    },
                );

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
                for stmt in body {
                    self.analyze_statement(stmt);
                }
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
                    match self.get_module_api(module_path) {
                        Ok(exports) => Some(exports),
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
                        self.declare_variable(item.clone());
                        let mut valid_export = false;
                        if let Some(parameters) = module_api
                            .as_ref()
                            .and_then(|exports| exports.iter().find(|export| export.name == *item))
                        {
                            valid_export = true;
                            if let Some(parameters) = &parameters.parameters {
                                self.function_signatures.insert(
                                    item.clone(),
                                    FunctionSignature {
                                        parameters: parameters.clone(),
                                    },
                                );
                            }
                        } else if let Some(exports) = &module_api {
                            if !exports.iter().any(|export| export.name == *item) {
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
                    if let Some(exports) = &module_api {
                        for export in exports {
                            self.declare_variable(export.name.clone());
                            if let Some(parameters) = &export.parameters {
                                self.function_signatures.insert(
                                    export.name.clone(),
                                    FunctionSignature {
                                        parameters: parameters.clone(),
                                    },
                                );
                            }
                        }
                    }
                }

                if let Some(alias_name) = alias {
                    self.declare_variable(alias_name.clone());
                    if module_api.is_none() {
                        self.use_variable(alias_name);
                    }
                    if let Some(exports) = &module_api {
                        let signatures = exports
                            .iter()
                            .filter_map(|export| {
                                export.parameters.clone().map(|parameters| {
                                    (export.name.clone(), FunctionSignature { parameters })
                                })
                            })
                            .collect();
                        self.module_signatures
                            .insert(alias_name.clone(), signatures);
                    }
                }
            }
            Stmt::Export { name, value } => {
                self.analyze_expr(value);
                self.declare_variable(name.clone());
                // An export is consumed by other modules, outside this file's
                // analysis boundary. Treat publication as a use.
                self.use_variable(name);
            }
            Stmt::ExportFunction { name, params, body } => {
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
                self.function_signatures.insert(
                    name.clone(),
                    FunctionSignature {
                        parameters: params.clone(),
                    },
                );

                self.declare_variable(name.clone());
                // Public functions are API surface even when this module does
                // not call them internally.
                self.use_variable(name);
                self.push_scope();

                let was_in_function = self.in_function;
                self.in_function = true;

                for param in params {
                    // Analyze default value first
                    if let Some(default_expr) = &param.default_value {
                        self.analyze_expr(default_expr);
                    }
                    self.declare_variable(param.name.clone());
                }

                for stmt in body {
                    self.analyze_statement(stmt);
                }

                self.in_function = was_in_function;
                self.pop_scope();
            }
            Stmt::Struct { name, fields: _ } => {
                // Struct definitions are type definitions, not variables
                // They should be tracked separately, but for now just declare them
                // so they don't trigger "undeclared" errors
                self.declare_variable(name.clone());
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
            Stmt::Break => {
                // break is valid inside loops
            }
            Stmt::Continue => {
                // continue is valid inside loops
            }
        }
    }

    fn analyze_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Identifier(name) => {
                // Check if it's a function first, then check if it's a variable
                // Prefer lexical variable bindings over builtins (e.g. param `min` vs `min()`)
                if !self.use_variable(name) && !self.functions.contains(name) {
                    let position = self.find_identifier_position(name);

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
                        if let Some(signature) = self
                            .module_signatures
                            .get(alias)
                            .and_then(|functions| functions.get(field))
                            .cloned()
                        {
                            self.validate_call_against_signature(
                                &format!("{alias}.{field}"),
                                &signature,
                                args,
                            );
                        }
                    }
                }
            }
            Expr::Index { object, index } => {
                self.analyze_expr(object);
                self.analyze_expr(index);
            }
            Expr::Member { object, .. } => {
                self.analyze_expr(object);
            }
            Expr::StructAccess { object, .. } => {
                self.analyze_expr(object);
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
            Expr::Array(elements) => {
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
            Expr::Generator {
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
                    for pattern in &arm.patterns {
                        self.analyze_pattern(pattern);
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

    fn validate_function_call(&mut self, func_name: &str, args: &[Argument]) {
        if let Some(signature) = self.function_signatures.get(func_name).cloned() {
            self.validate_call_against_signature(func_name, &signature, args);
        }
        // If function signature not found, we already reported "undeclared variable" error
    }

    fn validate_call_against_signature(
        &mut self,
        func_name: &str,
        signature: &FunctionSignature,
        args: &[Argument],
    ) {
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
                        let position = self.find_identifier_position(func_name);
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
                let position = self.find_identifier_position(func_name);
                self.errors.push(LintError {
                    message: format!(
                        "missing required argument `{}` for function `{}`",
                        param.name, func_name
                    ),
                    position,
                });
            }
        }
    }
}
