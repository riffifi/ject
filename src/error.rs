use crate::lexer::SourcePosition;
use std::fmt;

#[derive(Debug, Clone)]
pub struct ErrorReport {
    pub message: String,
    pub position: SourcePosition,
    pub source_file: Option<String>,
    pub source_line: Option<String>,
}

impl ErrorReport {
    pub fn new(message: String, position: SourcePosition) -> Self {
        ErrorReport {
            message,
            position,
            source_file: None,
            source_line: None,
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
    
    pub fn format_error(&self) -> String {
        let mut output = String::new();
        
        // Header with file and line info
        if let Some(file) = &self.source_file {
            output.push_str(&format!("Error in {}:{}:{}\n", file, self.position.line, self.position.column));
        } else {
            output.push_str(&format!("Error at line {}:{}\n", self.position.line, self.position.column));
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
            output.push_str(&format!("  {} | {}{}\n", padding, pointer_padding, "^"));
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
