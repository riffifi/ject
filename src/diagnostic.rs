use colored::*;

#[derive(Debug, Clone, PartialEq)]
pub enum DiagnosticLevel {
    Error,
    Warning,
    Note,
    Help,
}

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub level: DiagnosticLevel,
    pub message: String,
    pub code: Option<String>,
    pub line: Option<usize>,
    pub column: Option<usize>,
    pub source_line: Option<String>,
    pub filename: Option<String>,
    pub help: Option<String>,
}

impl Diagnostic {
    pub fn error(message: String) -> Self {
        Diagnostic {
            level: DiagnosticLevel::Error,
            message,
            code: None,
            line: None,
            column: None,
            source_line: None,
            filename: None,
            help: None,
        }
    }

    pub fn warning(message: String) -> Self {
        Diagnostic {
            level: DiagnosticLevel::Warning,
            message,
            code: None,
            line: None,
            column: None,
            source_line: None,
            filename: None,
            help: None,
        }
    }

    pub fn with_location(mut self, line: usize, column: usize) -> Self {
        self.line = Some(line);
        self.column = Some(column);
        self
    }

    pub fn with_source_line(mut self, source_line: String) -> Self {
        self.source_line = Some(source_line);
        self
    }

    pub fn with_filename(mut self, filename: String) -> Self {
        self.filename = Some(filename);
        self
    }

    pub fn with_code(mut self, code: String) -> Self {
        self.code = Some(code);
        self
    }

    pub fn with_help(mut self, help: String) -> Self {
        self.help = Some(help);
        self
    }
}

pub struct DiagnosticRenderer;

impl DiagnosticRenderer {
    pub fn new() -> Self {
        DiagnosticRenderer
    }
    
    pub fn render(&self, diagnostic: &Diagnostic, filename: Option<&str>, source_code: Option<&str>) {
        let mut diag = diagnostic.clone();
        
        // Add filename if provided
        if let Some(fname) = filename {
            diag.filename = Some(fname.to_string());
        }
        
        // If we have source code but no location, try to find a reasonable line to highlight
        if let Some(source) = source_code {
            if diag.line.is_none() && diag.source_line.is_none() {
                // For now, highlight the first non-empty line as a fallback
                let lines: Vec<&str> = source.lines().collect();
                if !lines.is_empty() {
                    let first_non_empty = lines.iter().enumerate()
                        .find(|(_, line)| !line.trim().is_empty())
                        .map(|(i, line)| (i + 1, *line))
                        .unwrap_or((1, lines[0]));
                    
                    diag.line = Some(first_non_empty.0);
                    diag.column = Some(1);
                    diag.source_line = Some(first_non_empty.1.to_string());
                }
            }
        }
        
        Self::render_diagnostic(&diag);
    }
    
    pub fn render_diagnostic(diagnostic: &Diagnostic) {
        let level_str = match diagnostic.level {
            DiagnosticLevel::Error => "error".red().bold(),
            DiagnosticLevel::Warning => "warning".yellow().bold(),
            DiagnosticLevel::Note => "note".blue().bold(),
            DiagnosticLevel::Help => "help".green().bold(),
        };

        // Print the main diagnostic line
        if let Some(code) = &diagnostic.code {
            print!("{}: {}: ", level_str, code.bold());
        } else {
            print!("{}: ", level_str);
        }
        println!("{}", diagnostic.message.bold());

        // Print location if available
        if let (Some(filename), Some(line), Some(column)) = (&diagnostic.filename, diagnostic.line, diagnostic.column) {
            println!("{} {}:{}:{}", 
                "-->".blue().bold(), 
                filename.bold(), 
                line.to_string().blue().bold(), 
                column.to_string().blue().bold()
            );
        } else if let (Some(line), Some(column)) = (diagnostic.line, diagnostic.column) {
            println!("{} line {}:{}", 
                "-->".blue().bold(), 
                line.to_string().blue().bold(), 
                column.to_string().blue().bold()
            );
        }

        // Print source line with context if available
        if let (Some(source_line), Some(line), Some(column)) = (&diagnostic.source_line, diagnostic.line, diagnostic.column) {
            let line_num_width = line.to_string().len();
            let padding = " ".repeat(line_num_width + 1);
            
            println!("{} {}", padding, "|".blue().bold());
            println!("{} {} {}", 
                line.to_string().blue().bold(), 
                "|".blue().bold(), 
                source_line
            );
            
            // Print the caret indicator
            let spaces_before_caret = " ".repeat(column.saturating_sub(1));
            println!("{} {} {}{}", 
                padding, 
                "|".blue().bold(), 
                spaces_before_caret,
                "^".red().bold()
            );
        }

        // Print help if available
        if let Some(help) = &diagnostic.help {
            println!("{} {}", "help:".green().bold(), help);
        }

        println!(); // Empty line for separation
    }

    pub fn render_summary(errors: usize, warnings: usize) {
        if errors > 0 || warnings > 0 {
            let mut parts = Vec::new();
            
            if errors > 0 {
                let error_text = if errors == 1 { "error" } else { "errors" };
                parts.push(format!("{} {}", errors, error_text).red().bold().to_string());
            }
            
            if warnings > 0 {
                let warning_text = if warnings == 1 { "warning" } else { "warnings" };
                parts.push(format!("{} {}", warnings, warning_text).yellow().bold().to_string());
            }

            if errors > 0 {
                println!("{}: {}", "aborting due to previous error(s)".red().bold(), parts.join(", "));
            } else {
                println!("{}: {}", "completed with".yellow().bold(), parts.join(", "));
            }
        }
    }
}

pub fn parse_lint_message(message: &str) -> Diagnostic {
    if message.starts_with("error:") {
        Diagnostic::error(message[6..].trim().to_string())
            .with_code("E0001".to_string())
    } else if message.starts_with("warning:") {
        Diagnostic::warning(message[8..].trim().to_string())
            .with_code("W0001".to_string())
    } else {
        // Fallback
        if message.contains("error") {
            Diagnostic::error(message.to_string())
        } else {
            Diagnostic::warning(message.to_string())
        }
    }
}
