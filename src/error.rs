use crate::lexer::SourcePosition;
use std::fmt;

/// Provides helpful suggestions for common errors
pub fn get_error_suggestion(error_message: &str) -> Option<String> {
    let msg = error_message.to_lowercase();
    
    if msg.contains("undefined variable") {
        return Some("Tip: Check for typos, or declare the variable with 'let' first.\n   Example: let my_var = 10".to_string());
    }
    if msg.contains("unexpected token") || msg.contains("expected") {
        return Some("Tip: Check your syntax. Did you forget an 'end' keyword?\n   Ject uses 'end' to close blocks (if, fn, while, for).".to_string());
    }
    if msg.contains("cannot assign") {
        return Some("Tip: Make sure the variable exists before assigning to it.\n   Use 'let var = value' for first assignment.".to_string());
    }
    if msg.contains("array index") {
        return Some("Tip: Array indices must be integers. Negative indices count from the end.\n   Example: arr[-1] gets the last element.".to_string());
    }
    if msg.contains("break") || msg.contains("continue") {
        return Some("Tip: 'break' and 'continue' can only be used inside loops (for/while).".to_string());
    }
    if msg.contains("increment") || msg.contains("decrement") || msg.contains("++") || msg.contains("--") {
        return Some("Tip: ++ and -- can only be used with numeric variables.\n   Example: counter++ or ++counter".to_string());
    }
    if msg.contains("division") || msg.contains("sqrt") || msg.contains("pow") {
        return Some("Tip: Check for division by zero or invalid math operations.\n   sqrt() requires non-negative numbers.".to_string());
    }
    if msg.contains("function") || msg.contains("call") {
        return Some("Tip: Check function name spelling and argument count.\n   Example: my_func(arg1, arg2)".to_string());
    }
    if msg.contains("missing") && msg.contains("end") {
        return Some("Tip: Every block needs an 'end' keyword.\n   - if/elseif/else blocks need 'end'\n   - fn blocks need 'end'\n   - for/while loops need 'end'".to_string());
    }
    
    None
}

#[derive(Debug, Clone)]
pub struct ErrorReport {
    pub message: String,
    pub position: SourcePosition,
    pub source_file: Option<String>,
    pub source_line: Option<String>,
    pub suggestion: Option<String>,
}

impl ErrorReport {
    pub fn new(message: String, position: SourcePosition) -> Self {
        let suggestion = get_error_suggestion(&message);
        ErrorReport {
            message,
            position,
            source_file: None,
            source_line: None,
            suggestion,
        }
    }

    pub fn with_source(mut self, file: String, source: &str) -> Self {
        self.source_file = Some(file);

        // Extract the source line
        let lines: Vec<&str> = source.lines().collect();
        if self.position.line > 0 && self.position.line <= lines.len() {
            self.source_line = Some(lines[self.position.line - 1].to_string());
        }

        self
    }
    
    pub fn with_suggestion(mut self, suggestion: String) -> Self {
        self.suggestion = Some(suggestion);
        self
    }

    pub fn format_error(&self) -> String {
        let mut output = String::new();

        // Header with file and line info
        if let Some(file) = &self.source_file {
            output.push_str(&format!("error in {}:{}:{}\n",
                file,
                self.position.line,
                self.position.column));
        } else {
            output.push_str(&format!("error at line {}:{}\n",
                self.position.line,
                self.position.column));
        }

        // Error message
        output.push_str(&format!("  {}\n", self.message));
        output.push('\n');

        // Source line with pointer
        if let Some(line) = &self.source_line {
            // Line number with padding
            let line_num_str = self.position.line.to_string();
            let padding = " ".repeat(line_num_str.len());

            output.push_str(&format!("  {} |\n", padding));
            output.push_str(&format!("  {} | {}\n", line_num_str, line));

            // Pointer to the error position
            let pointer_padding = " ".repeat(self.position.column.saturating_sub(1));
            output.push_str(&format!("  {} | {}^\n", padding, pointer_padding));
        }

        // Add suggestion if available
        if let Some(suggestion) = &self.suggestion {
            output.push_str(&format!("\n  Tip: {}\n", suggestion));
        }

        output
    }
}

impl fmt::Display for ErrorReport {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.format_error())
    }
}

#[derive(Debug)]
pub struct ParseError {
    pub report: ErrorReport,
}

impl ParseError {
    pub fn new(message: String, position: SourcePosition) -> Self {
        ParseError {
            report: ErrorReport::new(message, position),
        }
    }
    
    pub fn with_source(mut self, file: String, source: &str) -> Self {
        self.report = self.report.with_source(file, source);
        self
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.report)
    }
}

impl std::error::Error for ParseError {}

#[derive(Debug)]
pub struct RuntimeError {
    pub report: ErrorReport,
}

impl RuntimeError {
    pub fn new(message: String, position: SourcePosition) -> Self {
        RuntimeError {
            report: ErrorReport::new(message, position),
        }
    }
    
    pub fn with_source(mut self, file: String, source: &str) -> Self {
        self.report = self.report.with_source(file, source);
        self
    }
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.report)
    }
}

impl std::error::Error for RuntimeError {}

#[derive(Debug)]
pub struct LexError {
    pub report: ErrorReport,
}

impl LexError {
    pub fn new(message: String, position: SourcePosition) -> Self {
        LexError {
            report: ErrorReport::new(message, position),
        }
    }
    
    pub fn with_source(mut self, file: String, source: &str) -> Self {
        self.report = self.report.with_source(file, source);
        self
    }
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.report)
    }
}

impl std::error::Error for LexError {}
