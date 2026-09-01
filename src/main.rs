mod ast;
mod diagnostic;
mod interpreter;
mod jgui;
mod jnum;
mod lexer;
mod linter;
mod lsp;
mod module_resolver;
mod native;
mod package;
mod parser;
mod semantic;
mod stdlib;
mod value;

use diagnostic::{parse_diagnostic, runtime_diagnostic, DiagnosticRenderer};
use interpreter::Interpreter;
use lexer::Lexer;
use parser::Parser;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use std::path::Path;
use std::{env, fs};

#[derive(Clone, Copy)]
enum ExecutionMode {
    /// Parse, lint, and run the interpreter.
    Run,
    /// Parse and lint only (for editors / CI — never executes user code).
    CheckOnly,
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() {
        run_repl();
        return;
    }

    match args[0].as_str() {
        "--help" | "-h" => {
            print_help();
        }
        "--version" | "-V" => {
            println!("ject {}", env!("CARGO_PKG_VERSION"));
        }
        "--introspect" => {
            println!("{}", stdlib::introspect_native_kernel_json());
        }
        "lsp" => lsp::run(),
        "run" | "check" | "build" => {
            let project = current_project_or_exit();
            if args[0] == "build" {
                let release = args.iter().any(|arg| arg == "--release");
                match package::build_native_graph(&project, release) {
                    Ok(artifacts) => {
                        for artifact in artifacts {
                            println!("Built native artifact {}", artifact.display());
                        }
                    }
                    Err(error) => {
                        emit_cli_error(
                            "E4101",
                            error,
                            Some(
                                "fix the native build error shown above and run `ject build` again",
                            ),
                        );
                        std::process::exit(1);
                    }
                }
            }
            if !project.entry.is_file() {
                emit_cli_error(
                    "E4004",
                    format!(
                        "package '{}' entry does not exist: {}",
                        project.name,
                        project.entry.display()
                    ),
                    Some("set `[package].entry` in Ject.toml to an existing .ject file"),
                );
                std::process::exit(1);
            }
            let entry = project.entry.to_string_lossy().into_owned();
            if args[0] == "run" {
                run_file(&entry);
            } else {
                check_project_sources(&project);
                if args[0] == "build" {
                    println!("Built package '{}' (source checked)", project.name);
                }
            }
        }
        "test" => run_project_tests(),
        "install" => install_project(),
        "add" => add_dependency(&args[1..]),
        "remove" => remove_dependency(&args[1..]),
        "init" => {
            let library = args.iter().any(|arg| arg == "--lib");
            let native = args.iter().any(|arg| arg == "--native");
            let root = env::current_dir().unwrap_or_else(|e| {
                emit_cli_error("E4004", format!("cannot read current directory: {e}"), None);
                std::process::exit(1);
            });
            let name = root
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("ject_project");
            match package::init(&root, name, library, native) {
                Ok(project) => println!("Created package '{}'", project.name),
                Err(e) => {
                    emit_cli_error(
                        "E4005",
                        e,
                        Some("check the package name and directory permissions"),
                    );
                    std::process::exit(1);
                }
            }
        }
        "new" => create_project(&args[1..]),
        "--check" => {
            let files: Vec<&String> = args.iter().skip(1).collect();
            if files.is_empty() {
                emit_cli_error(
                    "E4002",
                    "--check requires at least one .ject file",
                    Some("try `ject --check path/to/file.ject`"),
                );
                std::process::exit(2);
            }
            for path in files {
                check_file(path);
            }
        }
        "--test" => {
            let files: Vec<&String> = args.iter().skip(1).collect();
            if files.is_empty() {
                emit_cli_error(
                    "E4002",
                    "--test requires at least one .ject file",
                    Some("try `ject --test path/to/test.ject`"),
                );
                std::process::exit(2);
            }
            for path in files {
                run_file(path);
            }
        }
        flag if flag.starts_with('-') => {
            emit_cli_error(
                "E4001",
                format!("unknown option `{flag}`"),
                Some("run `ject --help` to see supported commands and options"),
            );
            std::process::exit(2);
        }
        _ => {
            // Legacy: `ject script.ject`
            run_file(&args[0]);
        }
    }
}

fn emit_cli_error(code: &str, message: impl Into<String>, help: Option<&str>) {
    let mut diagnostic = diagnostic::Diagnostic::error(message.into()).with_code(code.to_string());
    if let Some(help) = help {
        diagnostic = diagnostic.with_help(help.to_string());
    }
    DiagnosticRenderer::new().render(&diagnostic, None, None);
}

fn print_help() {
    println!(
        "\
Ject language interpreter

USAGE:
    ject                      Start REPL
    ject <file.ject>          Run a script
    ject new <name> [--lib|--native]  Create a package
    ject init [--lib|--native]        Create a package in the current directory
    ject run                  Run the current package
    ject check                Check the current package
    ject test                 Run tests/*.ject in the current package
    ject install              Resolve, lock, and build dependencies
    ject add <name> --path <path>  Add a local source or mixed library
    ject remove <name>        Remove a dependency
    ject build                Validate the current source package
    ject lsp                  Start the Language Server Protocol server over stdio
    ject --check <file> [...] Parse + lint only (no execution)
    ject --test <file> [...]  Run script(s); exit non-zero on failure
    ject --introspect         Print native kernel metadata (JSON)
    ject --help               Show this message"
    );
}

fn install_project() {
    let project = current_project_or_exit();
    match package::install(&project) {
        Ok(dependencies) => {
            println!("Locked {} package(s) in Ject.lock", dependencies.len() + 1);
            match package::build_native_graph(&project, false) {
                Ok(artifacts) => {
                    for artifact in artifacts {
                        println!("Built native artifact {}", artifact.display());
                    }
                }
                Err(error) => {
                    emit_cli_error(
                        "E4101",
                        error,
                        Some("fix the native build error and run `ject install` again"),
                    );
                    std::process::exit(1);
                }
            }
        }
        Err(error) => {
            emit_cli_error(
                "E4102",
                error,
                Some("check dependency names and paths in Ject.toml"),
            );
            std::process::exit(1);
        }
    }
}

fn add_dependency(args: &[String]) {
    let Some(name) = args.first().filter(|name| !name.starts_with('-')) else {
        emit_cli_error(
            "E4002",
            "ject add requires a package name",
            Some("try `ject add my_lib --path ../my_lib`"),
        );
        std::process::exit(2);
    };
    let path = args
        .windows(2)
        .find(|pair| pair[0] == "--path")
        .map(|pair| Path::new(&pair[1]));
    let Some(path) = path else {
        emit_cli_error(
            "E4002",
            "local installation requires `--path <path>`",
            Some("registry packages are not available yet; use `ject add name --path ../name`"),
        );
        std::process::exit(2);
    };
    let project = current_project_or_exit();
    match package::add_path_dependency(&project, name, path)
        .and_then(|updated| package::install(&updated).map(|_| updated))
    {
        Ok(_) => println!("Added {name} from {}", path.display()),
        Err(error) => {
            emit_cli_error(
                "E4103",
                error,
                Some("the path must contain a valid Ject.toml whose package name matches"),
            );
            std::process::exit(1);
        }
    }
}

fn remove_dependency(args: &[String]) {
    let Some(name) = args.first().filter(|name| !name.starts_with('-')) else {
        emit_cli_error(
            "E4002",
            "ject remove requires a package name",
            Some("try `ject remove my_lib`"),
        );
        std::process::exit(2);
    };
    let project = current_project_or_exit();
    match package::remove_dependency(&project, name)
        .and_then(|updated| package::install(&updated).map(|_| updated))
    {
        Ok(_) => println!("Removed {name}"),
        Err(error) => {
            emit_cli_error(
                "E4104",
                error,
                Some("run `ject install` after correcting Ject.toml"),
            );
            std::process::exit(1);
        }
    }
}

fn current_project_or_exit() -> package::Project {
    let cwd = env::current_dir().unwrap_or_else(|e| {
        emit_cli_error("E4004", format!("cannot read current directory: {e}"), None);
        std::process::exit(1);
    });
    package::discover(&cwd).unwrap_or_else(|e| {
        emit_cli_error(
            "E4003",
            e,
            Some("run this command inside a package containing Ject.toml"),
        );
        std::process::exit(1);
    })
}

fn create_project(args: &[String]) {
    let Some(name) = args.iter().find(|arg| !arg.starts_with('-')) else {
        emit_cli_error(
            "E4002",
            "ject new requires a package name",
            Some("try `ject new my_package`"),
        );
        std::process::exit(2);
    };
    if name.contains('/') || name.contains('\\') || name == "." || name == ".." {
        emit_cli_error(
            "E4002",
            "package name must be a single directory name",
            Some("use a name such as `my_package`, without `/`, `\\`, `.` or `..`"),
        );
        std::process::exit(2);
    }
    let library = args.iter().any(|arg| arg == "--lib");
    let native = args.iter().any(|arg| arg == "--native");
    let root = Path::new(name);
    if root.exists() {
        emit_cli_error(
            "E4005",
            format!("{} already exists", root.display()),
            Some("choose another package name or use `ject init` inside that directory"),
        );
        std::process::exit(1);
    }
    fs::create_dir(root).unwrap_or_else(|e| {
        emit_cli_error(
            "E4005",
            format!("failed to create {}: {e}", root.display()),
            Some("check the parent directory permissions"),
        );
        std::process::exit(1);
    });
    match package::init(root, name, library, native) {
        Ok(_) => println!("Created package '{}'", name),
        Err(e) => {
            emit_cli_error(
                "E4005",
                e,
                Some("check the package name and directory permissions"),
            );
            std::process::exit(1);
        }
    }
}

fn run_project_tests() {
    let project = current_project_or_exit();
    let tests_dir = project.root.join("tests");
    let mut tests = fs::read_dir(&tests_dir)
        .map(|entries| {
            entries
                .filter_map(Result::ok)
                .map(|entry| entry.path())
                .filter(|path| path.extension().and_then(|e| e.to_str()) == Some("ject"))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    tests.sort();
    if tests.is_empty() {
        println!("No tests found in {}", tests_dir.display());
        return;
    }
    for test in tests {
        println!("test {}", test.display());
        run_file(&test.to_string_lossy());
    }
}

fn run_file(filename: &str) {
    prepare_native_for(Path::new(filename));
    match fs::read_to_string(filename) {
        Ok(source) => {
            let mut interpreter = Interpreter::new();
            if let Some(dir) = Path::new(filename).parent() {
                if !dir.as_os_str().is_empty() {
                    interpreter.set_script_dir(dir.to_path_buf());
                }
            }
            if !execute_source(
                &source,
                &mut interpreter,
                Some(filename.to_string()),
                ExecutionMode::Run,
            ) {
                std::process::exit(1);
            }
        }
        Err(error) => {
            emit_cli_error(
                "E4004",
                format!("could not read `{filename}`: {error}"),
                Some("check that the path exists and is readable"),
            );
            std::process::exit(1);
        }
    }
}

fn check_file(filename: &str) {
    prepare_native_for(Path::new(filename));
    match fs::read_to_string(filename) {
        Ok(source) => {
            let mut interpreter = Interpreter::new();
            if let Some(dir) = Path::new(filename).parent() {
                if !dir.as_os_str().is_empty() {
                    interpreter.set_script_dir(dir.to_path_buf());
                }
            }
            if !execute_source(
                &source,
                &mut interpreter,
                Some(filename.to_string()),
                ExecutionMode::CheckOnly,
            ) {
                std::process::exit(1);
            }
        }
        Err(error) => {
            emit_cli_error(
                "E4004",
                format!("could not read `{filename}`: {error}"),
                Some("check that the path exists and is readable"),
            );
            std::process::exit(1);
        }
    }
}

fn check_project_sources(project: &package::Project) {
    prepare_native_for(&project.root);
    let mut projects = package::dependency_projects(project).unwrap_or_else(|error| {
        emit_cli_error(
            "E4102",
            error,
            Some("check each path dependency in Ject.toml"),
        );
        std::process::exit(1);
    });
    projects.push(project.clone());
    let mut valid = true;
    for package in projects {
        let files = package::source_files(&package).unwrap_or_else(|error| {
            emit_cli_error(
                "E4004",
                error,
                Some("check that package sources are readable"),
            );
            std::process::exit(1);
        });
        for path in files {
            let filename = path.to_string_lossy().into_owned();
            let source = fs::read_to_string(&path).unwrap_or_else(|error| {
                emit_cli_error(
                    "E4004",
                    format!("could not read `{filename}`: {error}"),
                    Some("check that the path exists and is readable"),
                );
                std::process::exit(1);
            });
            let mut interpreter = Interpreter::new();
            interpreter.set_script_dir(path.parent().unwrap_or(&path).to_path_buf());
            valid &= execute_source(
                &source,
                &mut interpreter,
                Some(filename),
                ExecutionMode::CheckOnly,
            );
        }
    }
    if !valid {
        std::process::exit(1);
    }
}

fn prepare_native_for(path: &Path) {
    let start = if path.is_dir() {
        path
    } else {
        path.parent().unwrap_or(path)
    };
    let Ok(project) = package::discover(start) else {
        return;
    };
    let mut projects = match package::dependency_projects(&project) {
        Ok(projects) => projects,
        Err(error) => {
            emit_cli_error(
                "E4102",
                error,
                Some("check each path dependency in Ject.toml"),
            );
            std::process::exit(1);
        }
    };
    projects.push(project);
    for project in projects {
        match package::find_native_artifact(&project) {
            Ok(Some(artifact)) => {
                if let Err(error) = native::register_dynamic(&artifact, Some(&project.name)) {
                    emit_cli_error("E4201", format!("failed to load native package '{}': {error}", project.name), Some("run `ject build` and ensure the native package uses the current ject-native ABI"));
                    std::process::exit(1);
                }
            }
            Ok(None) => {}
            Err(error) => {
                emit_cli_error(
                    "E4202",
                    error,
                    Some("rebuild the native package with `ject build`"),
                );
                std::process::exit(1);
            }
        }
    }
}

/// Heuristically decides whether `source` looks like an incomplete statement/block
/// that should keep accumulating more input lines, rather than a genuine syntax
/// error to show immediately. Used by the REPL to support multi-line input the way
/// Python's REPL does -- typing `fn foo(x)` and pressing enter waits for a matching
/// `end` instead of erroring immediately.
fn input_seems_incomplete(source: &str) -> bool {
    let mut lexer = Lexer::new(source);
    let tokens: Vec<lexer::Token> = lexer
        .tokenize_with_positions()
        .into_iter()
        .map(|lt| lt.token)
        .collect();

    let mut paren_depth: i32 = 0;
    let mut bracket_depth: i32 = 0;
    let mut brace_depth: i32 = 0;
    let mut block_depth: i32 = 0;

    let mut i = 0;
    while i < tokens.len() {
        match &tokens[i] {
            lexer::Token::LeftParen => paren_depth += 1,
            lexer::Token::RightParen => paren_depth -= 1,
            lexer::Token::LeftBracket => bracket_depth += 1,
            lexer::Token::RightBracket => bracket_depth -= 1,
            lexer::Token::LeftBrace | lexer::Token::LeftBracePipe => brace_depth += 1,
            lexer::Token::RightBrace | lexer::Token::RightPipeBrace => brace_depth -= 1,
            lexer::Token::If
            | lexer::Token::While
            | lexer::Token::For
            | lexer::Token::Try
            | lexer::Token::Match => {
                block_depth += 1;
            }
            lexer::Token::Fn => {
                // `fn(...) -> expr` / `fn name(...) -> expr` need no matching `end`;
                // `fn(...) ... end` / `fn name(...) ... end` do. Scan ahead past this
                // header (skipping the optional name and the parameter list) to see
                // which form it is.
                let mut j = i + 1;
                let mut header_paren_depth = 0i32;
                let mut seen_paren = false;
                let mut incomplete_header = false;
                loop {
                    if j >= tokens.len() {
                        incomplete_header = true;
                        break;
                    }
                    match &tokens[j] {
                        lexer::Token::LeftParen => {
                            header_paren_depth += 1;
                            seen_paren = true;
                        }
                        lexer::Token::RightParen => {
                            header_paren_depth -= 1;
                            if seen_paren && header_paren_depth <= 0 {
                                j += 1;
                                break;
                            }
                        }
                        _ => {}
                    }
                    j += 1;
                }
                if incomplete_header {
                    return true;
                }
                if tokens.get(j) != Some(&lexer::Token::Arrow) {
                    block_depth += 1;
                }
                i = j;
                continue;
            }
            lexer::Token::End => {
                block_depth -= 1;
            }
            _ => {}
        }
        i += 1;
    }

    if paren_depth > 0 || bracket_depth > 0 || brace_depth > 0 || block_depth > 0 {
        return true;
    }

    // A trailing token that clearly expects something to follow it (e.g. `let x =`,
    // `1 +`, a dangling `,`) means there's more coming even though nothing's unbalanced.
    let last_meaningful = tokens
        .iter()
        .rev()
        .find(|t| !matches!(t, lexer::Token::Newline | lexer::Token::Eof));
    if let Some(t) = last_meaningful {
        use lexer::Token::*;
        if matches!(
            t,
            Plus | Minus
                | Star
                | Slash
                | Percent
                | Equal
                | EqualEqual
                | BangEqual
                | Less
                | Greater
                | LessEqual
                | GreaterEqual
                | And
                | Or
                | Comma
                | Arrow
                | Dot
                | DotDot
                | Let
                | Import
                | From
                | As
                | Colon
        ) {
            return true;
        }
    }

    false
}

fn run_repl() {
    println!("Ject REPL - version {}", env!("CARGO_PKG_VERSION"));
    println!("Use arrow keys to access history");
    println!("Ctrl+C cancels the current line, or interrupts a running script; Ctrl+D exits");
    println!("'exit' to, well, exit\n");

    let mut interpreter = Interpreter::new();
    // A REPL started inside a package has the same dependency and native-module
    // context as `ject run`. Without this, manifest imports incorrectly fell
    // through to stdlib lookup.
    if let Ok(cwd) = env::current_dir() {
        prepare_native_for(&cwd);
        interpreter.set_script_dir(cwd);
    }
    let mut linter = linter::Linter::new(); // Persistent linter for REPL
    let mut rl = DefaultEditor::new().expect("Failed to create readline editor");

    // Try to load history from file
    let _ = rl.load_history(".ject_history");

    // Shared with a Ctrl+C handler below: lets a running script (e.g. an infinite
    // `while true do ... end`) be interrupted cleanly instead of only being able to
    // cancel line input that hasn't been submitted yet. If installing the handler
    // fails for any reason, the REPL still works exactly as before -- Ctrl+C just
    // won't be able to interrupt a script already running.
    let interrupt_flag = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    interpreter.set_interrupt_flag(interrupt_flag.clone());
    {
        let flag = interrupt_flag.clone();
        let _ = ctrlc::set_handler(move || {
            flag.store(true, std::sync::atomic::Ordering::Relaxed);
        });
    }

    // Accumulates lines while input looks incomplete (an unclosed `fn ... end`, an
    // open paren, a trailing `+`, etc.) -- mirrors Python's REPL waiting for more
    // input instead of erroring on the first line of a multi-line statement.
    let mut buffer = String::new();

    loop {
        let prompt = if buffer.is_empty() { ">> " } else { ".. " };
        match rl.readline(prompt) {
            Ok(line) => {
                if buffer.is_empty() {
                    let trimmed = line.trim();
                    if trimmed == "exit" {
                        println!("Goodbye!");
                        break;
                    }
                    if trimmed.is_empty() {
                        continue;
                    }
                }

                if !buffer.is_empty() {
                    buffer.push('\n');
                }
                buffer.push_str(&line);

                // An empty line while continuing a multi-line block means "run what
                // I've got" -- try executing even if the heuristic below would
                // otherwise still want more (it can't always tell an
                // intentionally-abandoned block from a genuinely unfinished one).
                if !line.trim().is_empty() && input_seems_incomplete(&buffer) {
                    continue;
                }

                let _ = rl.add_history_entry(buffer.trim());
                interrupt_flag.store(false, std::sync::atomic::Ordering::Relaxed);
                execute_source_repl(&buffer, &mut interpreter, &mut linter);
                interrupt_flag.store(false, std::sync::atomic::Ordering::Relaxed);
                buffer.clear();
            }
            Err(ReadlineError::Interrupted) => {
                println!("\n^C");
                buffer.clear(); // Cancel whatever multi-line input was being composed too.
                continue;
            }
            Err(ReadlineError::Eof) => {
                println!("\n^D");
                break;
            }
            Err(err) => {
                emit_cli_error(
                    "E4006",
                    format!("REPL input failed: {err}"),
                    Some("restart the REPL; remove .ject_history if the history file is corrupt"),
                );
                break;
            }
        }
    }

    // Save history to file
    let _ = rl.save_history(".ject_history");
}

fn execute_source(
    source: &str,
    interpreter: &mut Interpreter,
    filename: Option<String>,
    mode: ExecutionMode,
) -> bool {
    let mut lexer = Lexer::new(source);
    let located_tokens = lexer.tokenize_with_positions();
    let positioned_tokens: Vec<(lexer::Token, lexer::SourcePosition)> = located_tokens
        .into_iter()
        .map(|lt| (lt.token, lt.position))
        .collect();

    // Clone positioned tokens for linter before parser consumes them
    let positioned_tokens_for_linter = positioned_tokens.clone();
    let mut parser = Parser::new(positioned_tokens);

    match parser.parse() {
        Ok(statements) => {
            // Run linter to detect errors and warnings
            let mut linter = linter::Linter::new()
                .with_tokens_and_source(positioned_tokens_for_linter, source.to_string());
            if let Some(path) = filename.as_deref() {
                linter = linter.with_source_path(path);
            }
            let (diagnostics, has_errors) = linter.lint(&statements);

            // Create diagnostic renderer for beautiful output
            let renderer = DiagnosticRenderer::new();

            // Display all diagnostics with colorful formatting
            for diagnostic in &diagnostics {
                renderer.render(diagnostic, filename.as_deref(), Some(source));
            }
            let errors = diagnostics
                .iter()
                .filter(|d| d.level == diagnostic::DiagnosticLevel::Error)
                .count();
            let warnings = diagnostics
                .iter()
                .filter(|d| d.level == diagnostic::DiagnosticLevel::Warning)
                .count();
            DiagnosticRenderer::render_summary(errors, warnings);

            // Only run interpreter if no errors were found
            if !has_errors {
                match mode {
                    ExecutionMode::CheckOnly => return true,
                    ExecutionMode::Run => match interpreter.interpret(&statements) {
                        Ok(_) => return true,
                        Err(error) => {
                            let runtime_diagnostic = runtime_diagnostic(&error.message);
                            renderer.render(&runtime_diagnostic, filename.as_deref(), Some(source));
                            return false;
                        }
                    },
                }
            } else {
                return false;
            }
        }
        Err(error) => {
            // Create diagnostic renderer for parse errors
            let renderer = DiagnosticRenderer::new();
            let parse_diagnostic = parse_diagnostic(&error.message, error.line, error.column);

            renderer.render(&parse_diagnostic, filename.as_deref(), Some(source));

            return false;
        }
    }
}

fn execute_source_repl(source: &str, interpreter: &mut Interpreter, linter: &mut linter::Linter) {
    let mut lexer = Lexer::new(source);
    let located_tokens = lexer.tokenize_with_positions();
    let positioned_tokens: Vec<(lexer::Token, lexer::SourcePosition)> = located_tokens
        .into_iter()
        .map(|lt| (lt.token, lt.position))
        .collect();

    // Clone positioned tokens for linter before parser consumes them
    let positioned_tokens_for_linter = positioned_tokens.clone();
    let mut parser = Parser::new(positioned_tokens);

    match parser.parse() {
        Ok(statements) => {
            // Use REPL-aware linter that maintains state
            *linter = linter
                .clone()
                .with_tokens_and_source(positioned_tokens_for_linter, source.to_string());
            let (diagnostics, has_errors) = linter.lint_repl(&statements);

            // Create diagnostic renderer for beautiful output
            let renderer = DiagnosticRenderer::new();

            // In REPL mode, only show errors (not warnings) to reduce noise
            // Warnings are still collected but not displayed unless verbose mode
            for diagnostic in &diagnostics {
                if diagnostic.level == crate::diagnostic::DiagnosticLevel::Error {
                    renderer.render(diagnostic, None, Some(source));
                }
            }

            // Only run interpreter if no errors were found
            if !has_errors {
                match interpreter.interpret_repl(&statements) {
                    Ok(Some(result)) if !matches!(result, value::Value::Nil) => {
                        println!("{}", result.display());
                    }
                    Ok(_) => {}
                    Err(error) if error.message == "Interrupted" => {
                        println!("\n^C");
                    }
                    Err(error) => {
                        let runtime_diagnostic = runtime_diagnostic(&error.message);
                        let renderer = DiagnosticRenderer::new();
                        renderer.render(&runtime_diagnostic, None, Some(source));
                    }
                }
            }
        }
        Err(error) => {
            // Create diagnostic renderer for parse errors
            let renderer = DiagnosticRenderer::new();
            let parse_diagnostic = parse_diagnostic(&error.message, error.line, error.column);

            renderer.render(&parse_diagnostic, None, Some(source));
        }
    }
}
