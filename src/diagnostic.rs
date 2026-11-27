use colored::Colorize;
use std::fmt::Write;
use std::io::{self, IsTerminal, Write as IoWrite};
use unicode_width::UnicodeWidthChar;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DiagnosticLevel {
    Error,
    Warning,
    Note,
    Help,
}

impl DiagnosticLevel {
    fn name(self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Warning => "warning",
            Self::Note => "note",
            Self::Help => "help",
        }
    }
    fn paint(self, text: &str, color: bool) -> String {
        if !color {
            return text.into();
        }
        match self {
            Self::Error => text.red().bold(),
            Self::Warning => text.yellow().bold(),
            Self::Note => text.cyan().bold(),
            Self::Help => text.green().bold(),
        }
        .to_string()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceSpan {
    pub line: usize,
    pub column: usize,
    pub length: usize,
}
impl SourceSpan {
    pub fn new(line: usize, column: usize, length: usize) -> Self {
        Self {
            line: line.max(1),
            column: column.max(1),
            length: length.max(1),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticLabel {
    pub span: SourceSpan,
    pub message: Option<String>,
    pub primary: bool,
}

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub level: DiagnosticLevel,
    pub message: String,
    pub code: Option<String>,
    // Compatibility fields for existing linter consumers.
    pub line: Option<usize>,
    pub column: Option<usize>,
    pub source_line: Option<String>,
    pub filename: Option<String>,
    pub help: Option<String>,
    pub labels: Vec<DiagnosticLabel>,
    pub notes: Vec<String>,
}

impl Diagnostic {
    fn new(level: DiagnosticLevel, message: String) -> Self {
        Self {
            level,
            message,
            code: None,
            line: None,
            column: None,
            source_line: None,
            filename: None,
            help: None,
            labels: vec![],
            notes: vec![],
        }
    }
    pub fn error(message: String) -> Self {
        Self::new(DiagnosticLevel::Error, message)
    }
    pub fn warning(message: String) -> Self {
        Self::new(DiagnosticLevel::Warning, message)
    }
    pub fn note(message: String) -> Self {
        Self::new(DiagnosticLevel::Note, message)
    }
    pub fn with_location(mut self, line: usize, column: usize) -> Self {
        self.line = Some(line.max(1));
        self.column = Some(column.max(1));
        self
    }
    pub fn with_span(mut self, line: usize, column: usize, length: usize) -> Self {
        let span = SourceSpan::new(line, column, length);
        self.line = Some(span.line);
        self.column = Some(span.column);
        self.labels.retain(|label| !label.primary);
        self.labels.push(DiagnosticLabel {
            span,
            message: None,
            primary: true,
        });
        self
    }
    pub fn with_primary_label(mut self, span: SourceSpan, message: impl Into<String>) -> Self {
        self.line = Some(span.line);
        self.column = Some(span.column);
        self.labels.retain(|label| !label.primary);
        self.labels.push(DiagnosticLabel {
            span,
            message: Some(message.into()),
            primary: true,
        });
        self
    }
    pub fn with_secondary_label(mut self, span: SourceSpan, message: impl Into<String>) -> Self {
        self.labels.push(DiagnosticLabel {
            span,
            message: Some(message.into()),
            primary: false,
        });
        self
    }
    pub fn with_source_line(mut self, value: String) -> Self {
        self.source_line = Some(value);
        self
    }
    pub fn with_filename(mut self, value: String) -> Self {
        self.filename = Some(value);
        self
    }
    pub fn with_code(mut self, value: String) -> Self {
        self.code = Some(value);
        self
    }
    pub fn with_help(mut self, value: String) -> Self {
        self.help = Some(value);
        self
    }
    pub fn with_note(mut self, value: impl Into<String>) -> Self {
        self.notes.push(value.into());
        self
    }
}

pub struct DiagnosticRenderer {
    color: bool,
}
impl Default for DiagnosticRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl DiagnosticRenderer {
    pub fn new() -> Self {
        Self {
            color: io::stderr().is_terminal(),
        }
    }
    pub fn plain() -> Self {
        Self { color: false }
    }
    pub fn render(&self, diagnostic: &Diagnostic, filename: Option<&str>, source: Option<&str>) {
        let rendered = self.render_to_string(diagnostic, filename, source);
        let _ = io::stderr().write_all(rendered.as_bytes());
    }
    pub fn render_diagnostic(diagnostic: &Diagnostic) {
        Self::new().render(
            diagnostic,
            diagnostic.filename.as_deref(),
            diagnostic.source_line.as_deref(),
        );
    }
    pub fn render_to_string(
        &self,
        diagnostic: &Diagnostic,
        filename: Option<&str>,
        source: Option<&str>,
    ) -> String {
        let mut out = String::new();
        let heading = diagnostic
            .code
            .as_ref()
            .map(|c| format!("{}[{c}]", diagnostic.level.name()))
            .unwrap_or_else(|| diagnostic.level.name().into());
        let _ = writeln!(
            out,
            "{}: {}",
            diagnostic.level.paint(&heading, self.color),
            diagnostic.message
        );
        let primary = diagnostic
            .labels
            .iter()
            .find(|l| l.primary)
            .map(|l| l.span)
            .or_else(|| {
                diagnostic
                    .line
                    .zip(diagnostic.column)
                    .map(|(l, c)| SourceSpan::new(l, c, 1))
            });
        let file = filename
            .or(diagnostic.filename.as_deref())
            .unwrap_or("<input>");
        let lines: Vec<_> = source
            .map(|text| text.split_terminator('\n').collect())
            .unwrap_or_default();
        if let Some(span) = primary {
            let _ = writeln!(
                out,
                " {} {file}:{}:{}",
                self.blue("-->"),
                span.line,
                span.column
            );
            let text = lines
                .get(span.line - 1)
                .copied()
                .or(diagnostic.source_line.as_deref());
            if let Some(text) = text {
                let label = diagnostic
                    .labels
                    .iter()
                    .find(|l| l.primary)
                    .and_then(|l| l.message.as_deref());
                self.render_excerpt(&mut out, span, text, diagnostic.level, "^", label);
            }
        }
        for label in diagnostic.labels.iter().filter(|l| !l.primary) {
            let message = label.message.as_deref().unwrap_or("related location");
            if let Some(text) = lines.get(label.span.line - 1) {
                let _ = writeln!(
                    out,
                    " {} {file}:{}:{}",
                    self.blue(":::"),
                    label.span.line,
                    label.span.column
                );
                self.render_excerpt(
                    &mut out,
                    label.span,
                    text,
                    DiagnosticLevel::Note,
                    "-",
                    Some(message),
                );
                continue;
            }
            let _ = writeln!(
                out,
                " {} {}:{}: {}",
                self.blue("="),
                label.span.line,
                label.span.column,
                message
            );
        }
        for note in &diagnostic.notes {
            let _ = writeln!(out, " {} {note}", self.blue("= note:"));
        }
        if let Some(help) = &diagnostic.help {
            let _ = writeln!(
                out,
                " {} {help}",
                DiagnosticLevel::Help.paint("= help:", self.color)
            );
        }
        out.push('\n');
        out
    }

    fn render_excerpt(
        &self,
        out: &mut String,
        span: SourceSpan,
        text: &str,
        level: DiagnosticLevel,
        marker: &str,
        label: Option<&str>,
    ) {
        let number = span.line.to_string();
        let gutter = " ".repeat(number.len());
        let start = span.column - 1;
        let stop = start.saturating_add(span.length);
        let mut visible = String::new();
        let mut cells = 0;
        let mut padding = 0;
        let mut width = 0;
        for (index, character) in text.trim_end_matches('\r').chars().enumerate() {
            let advance = match character {
                '\t' => {
                    let spaces = 4 - cells % 4;
                    visible.push_str(&" ".repeat(spaces));
                    spaces
                }
                character if character.is_control() => {
                    visible.push('�');
                    1
                }
                character => {
                    visible.push(character);
                    character.width().unwrap_or(0)
                }
            };
            cells += advance;
            if index < start {
                padding += advance;
            } else if index < stop {
                width += advance;
            }
        }
        let _ = writeln!(out, " {gutter} {}", self.blue("|"));
        let _ = writeln!(out, " {} {} {visible}", self.blue(&number), self.blue("|"));
        let pointer = level.paint(&marker.repeat(width.max(1)), self.color);
        let message = label.map(|label| format!(" {label}")).unwrap_or_default();
        let _ = writeln!(
            out,
            " {gutter} {} {}{pointer}{message}",
            self.blue("|"),
            " ".repeat(padding)
        );
    }
    fn blue(&self, text: &str) -> String {
        if self.color {
            text.blue().bold().to_string()
        } else {
            text.into()
        }
    }
    pub fn render_summary(errors: usize, warnings: usize) {
        if errors > 0 {
            eprintln!(
                "error: aborting due to {errors} previous error(s); {warnings} warning(s) emitted"
            );
        } else if warnings > 0 {
            eprintln!("warning: {warnings} warning(s) emitted");
        }
    }
}

pub fn parse_diagnostic(message: &str, line: Option<usize>, column: Option<usize>) -> Diagnostic {
    let lower = message.to_lowercase();
    let (code, help) = if lower.contains("unterminated string") {
        (
            "E1002",
            Some("add a closing `\"` before the end of the line or file"),
        )
    } else if lower.contains("unterminated multiline comment") {
        ("E1003", Some("close the multiline comment with `*#`"))
    } else if lower.contains("numeric literal") {
        (
            "E1004",
            Some("use a valid finite number; integer literals must fit in signed 64-bit storage"),
        )
    } else if lower.contains("unexpected character") {
        (
            "E1001",
            Some("remove this character or replace it with a valid Ject token"),
        )
    } else if lower.contains("expected 'end'") || lower.contains("expected end") {
        ("E1103", Some("close the block with `end`"))
    } else if lower.contains("expected") {
        (
            "E1101",
            Some("check this token and the syntax immediately before it"),
        )
    } else if lower.contains("unexpected") {
        (
            "E1102",
            Some("remove this token or complete the surrounding expression"),
        )
    } else {
        ("E1199", None)
    };
    let mut d = Diagnostic::error(message.into()).with_code(code.into());
    if let (Some(l), Some(c)) = (line, column) {
        d = d.with_span(l, c, 1);
    }
    if let Some(help) = help {
        d = d.with_help(help.into());
    }
    d
}

pub fn runtime_diagnostic(message: &str) -> Diagnostic {
    let mut lines = message.lines();
    let message = lines.next().unwrap_or(message);
    let lower = message.to_lowercase();
    let (code, help) = if lower.contains("undefined variable") {
        (
            "E3001",
            Some("check the spelling or declare the name with `let` before using it"),
        )
    } else if lower.contains("division by zero") {
        (
            "E3002",
            Some("ensure the denominator is not zero before dividing"),
        )
    } else if lower.contains("integer overflow") {
        ("E3008", Some("keep integer results within -9223372036854775808..9223372036854775807 or use floating-point arithmetic"))
    } else if lower.contains("immutable binding") {
        (
            "E3007",
            Some("declare a mutable binding with `let` if this value must be reassigned"),
        )
    } else if lower.contains("jgui::run") && lower.contains("window failed") {
        (
            "E3202",
            Some("run JGUI in a graphical desktop session and verify DISPLAY or WAYLAND_DISPLAY is available"),
        )
    } else if lower.contains("array index")
        || lower.contains("invalid index")
        || lower.contains("index out")
        || lower.contains("index must")
        || lower.contains("out of bounds")
    {
        ("E3003", Some("check the collection length and index value"))
    } else if lower.contains("argument") {
        (
            "E3004",
            Some("check the function signature and the number and types of arguments"),
        )
    } else if lower.contains("cannot call") || lower.contains("not callable") {
        (
            "E3005",
            Some("only functions, lambdas, and native functions can be called"),
        )
    } else if lower.contains("module") || lower.contains("import") || lower.contains("export") {
        (
            "E3101",
            Some("check the import path, package dependency, and exported name"),
        )
    } else if lower.contains("native") || lower.contains("abi") {
        (
            "E3201",
            Some("rebuild the native package and verify its Ject ABI version"),
        )
    } else if lower.contains("type") || lower.contains("operand") || lower.contains("expected") {
        ("E3006", Some("use values supported by this operation"))
    } else {
        ("E3999", None)
    };
    let parse_message = message.strip_prefix("Bootstrap parse error: ");
    let unlocated_check = message.strip_prefix("Bootstrap unlocated check error: ");
    let check_message = unlocated_check.or_else(|| message.strip_prefix("Bootstrap check error: "));
    let mut d = match parse_message {
        Some(message) => parse_diagnostic(message, None, None),
        None => match check_message {
            Some(message) => {
                let code = crate::linter::Linter::error_code(message);
                let diagnostic = Diagnostic::error(message.into()).with_code(code.into());
                if code == "E4102" {
                    diagnostic.with_help("check each path dependency in Ject.toml".into())
                } else if code == "E3101" {
                    diagnostic.with_help(
                        "check the import path, package dependency, and exported name".into(),
                    )
                } else if message.contains("immutable binding") {
                    diagnostic.with_help(
                        "declare a mutable binding with `let` if this value must be reassigned"
                            .into(),
                    )
                } else {
                    diagnostic
                }
            }
            None => Diagnostic::error(message.into()).with_code(code.into()),
        },
    };
    let mut check_help = None::<String>;
    for detail in lines {
        let detail = detail.trim();
        if let Some(chain) = detail
            .strip_prefix("note: import chain: ")
            .filter(|_| parse_message.is_some() || check_message.is_some())
        {
            d = d.with_note(format!("import chain: {chain}"));
            continue;
        }
        if unlocated_check.is_some() {
            continue;
        }
        if let Some(location) = detail.strip_prefix("-->") {
            let mut parts = location.trim().rsplitn(3, ':');
            if let (Some(column), Some(line)) = (parts.next(), parts.next()) {
                if let (Ok(line), Ok(column)) = (line.parse(), column.parse()) {
                    d = d.with_span(line, column, 1);
                    if let Some(filename) = parts.next() {
                        d = d.with_filename(filename.to_string());
                    }
                }
            }
        } else if parse_message.is_some() || check_message.is_some() {
            if let Some(help) = detail
                .strip_prefix("help: ")
                .filter(|_| check_message.is_some())
            {
                check_help = Some(help.to_owned());
            } else if detail.starts_with("- ") {
                if let Some(help) = &mut check_help {
                    help.push_str("\n    ");
                    help.push_str(detail);
                }
            }
        } else {
            if let Some(frame) = detail.strip_prefix("at ") {
                d = d.with_note(format!("at {frame}"));
            }
        }
    }
    if let Some(help) = check_help {
        d = d.with_help(help);
    }
    if let Some(help) = help.filter(|_| parse_message.is_none() && check_message.is_none()) {
        d = d.with_help(help.into());
    }
    d
}

pub fn parse_lint_message(message: &str) -> Diagnostic {
    if let Some(m) = message.strip_prefix("error:") {
        Diagnostic::error(m.trim().into()).with_code("E2001".into())
    } else if let Some(m) = message.strip_prefix("warning:") {
        Diagnostic::warning(m.trim().into()).with_code("W2001".into())
    } else if message.contains("error") {
        Diagnostic::error(message.into()).with_code("E2099".into())
    } else {
        Diagnostic::warning(message.into()).with_code("W2099".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn integer_overflow_has_a_specific_diagnostic() {
        let diagnostic = runtime_diagnostic("Integer overflow in +");
        assert_eq!(diagnostic.code.as_deref(), Some("E3008"));
        assert!(diagnostic.help.unwrap().contains("floating-point"));
    }

    #[test]
    fn renders_stable_rust_style_diagnostic() {
        let d = Diagnostic::error("unknown name `totla`".into())
            .with_code("E3001".into())
            .with_primary_label(SourceSpan::new(2, 7, 5), "not found in this scope")
            .with_note("names are case-sensitive")
            .with_help("did you mean `total`?".into());
        let out = DiagnosticRenderer::plain().render_to_string(
            &d,
            Some("main.ject"),
            Some("let total = 1\nprint totla\n"),
        );
        assert!(out.contains("error[E3001]: unknown name `totla`"));
        assert!(out.contains("--> main.ject:2:7"));
        assert!(out.contains("^^^^^ not found"));
        assert!(out.contains("= note:"));
        assert!(out.contains("= help:"));
    }

    #[test]
    fn terminal_spans_use_display_cells_and_include_related_source() {
        let diagnostic = Diagnostic::warning("duplicate binding".into())
            .with_span(2, 1, 1)
            .with_primary_label(SourceSpan::new(2, 3, 2), "new binding")
            .with_secondary_label(SourceSpan::new(1, 5, 2), "previous binding");
        let rendered = DiagnosticRenderer::plain().render_to_string(
            &diagnostic,
            Some("demo.ject"),
            Some("let 名字 = 1\n\t界名字 = 2\n"),
        );
        assert!(rendered.contains("--> demo.ject:2:3"));
        assert!(rendered.contains("2 |     界名字 = 2"));
        assert!(rendered.contains("|       ^^^^ new binding"));
        assert!(rendered.contains("::: demo.ject:1:5"));
        assert!(rendered.contains("|     ---- previous binding"));
    }
    #[test]
    fn runtime_codes_are_specific() {
        assert_eq!(
            runtime_diagnostic("Undefined variable: x").code.as_deref(),
            Some("E3001")
        );
        assert_eq!(
            runtime_diagnostic("Division by zero").code.as_deref(),
            Some("E3002")
        );
        let immutable = runtime_diagnostic("Cannot assign through immutable binding 'answer'");
        assert_eq!(immutable.code.as_deref(), Some("E3007"));
        assert!(immutable.help.is_some());
        assert_eq!(
            runtime_diagnostic("jgui::run: window failed in index.crates.io")
                .code
                .as_deref(),
            Some("E3202")
        );
        let headless_jgui_error =
            runtime_diagnostic("jgui::dispatch: document has no widget with ID 'missing'");
        assert_eq!(headless_jgui_error.code.as_deref(), Some("E3999"));
        assert!(headless_jgui_error.help.is_none());
        assert_eq!(
            runtime_diagnostic("window failed to save").code.as_deref(),
            Some("E3999")
        );
        assert_eq!(
            runtime_diagnostic(
                "module './missing' not found as a package, path, or standard module"
            )
            .code
            .as_deref(),
            Some("E3101")
        );
    }

    #[test]
    fn runtime_stack_frames_render_as_notes() {
        let diagnostic = runtime_diagnostic("Undefined variable: x\n  at inner\n  at outer");
        assert_eq!(diagnostic.message, "Undefined variable: x");
        assert_eq!(diagnostic.notes, vec!["at inner", "at outer"]);
        let rendered = DiagnosticRenderer::plain().render_to_string(&diagnostic, None, None);
        assert!(rendered.contains("= note: at inner"));
        assert!(rendered.contains("= note: at outer"));
    }

    #[test]
    fn runtime_locations_render_source_spans() {
        let diagnostic = runtime_diagnostic("Undefined variable 'x'.\n  --> 2:7\n  at main");
        let rendered = DiagnosticRenderer::plain().render_to_string(
            &diagnostic,
            Some("main.ject"),
            Some("let x = 1\nprint missing\n"),
        );
        assert!(rendered.contains("--> main.ject:2:7"));
        assert!(rendered.contains("= note: at main"));
    }

    #[test]
    fn runtime_locations_accept_filenames() {
        let diagnostic = runtime_diagnostic("Division by zero\n  --> src/math.ject:8:14");
        assert_eq!(diagnostic.filename.as_deref(), Some("src/math.ject"));
        assert_eq!(diagnostic.line, Some(8));
        assert_eq!(diagnostic.column, Some(14));
    }
}
