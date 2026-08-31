//! Ject Language Server Protocol implementation.
//!
//! This server deliberately uses the compiler's lexer, parser, and linter instead
//! of parsing rendered CLI output. Editors therefore receive stable structured
//! diagnostics for the exact in-memory text the user is editing.

use crate::diagnostic::{self, DiagnosticLevel, SourceSpan};
use crate::lexer::{Lexer, Token};
use crate::linter::Linter;
use crate::parser::Parser;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tokio::sync::RwLock;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

const KEYWORDS: &[&str] = &[
    "let", "fn", "if", "elseif", "else", "while", "for", "in", "return", "true", "false", "nil",
    "end", "print", "import", "export", "from", "as", "struct", "new", "try", "catch", "throw",
    "break", "continue", "match",
];

const BUILTINS: &[&str] = &[
    "type_of",
    "to_int",
    "to_float",
    "to_string",
    "to_bool",
    "len",
    "range",
    "push",
    "pop",
    "sum",
    "contains",
    "index_of",
    "first",
    "last",
    "slice",
    "sort",
    "reverse",
    "unique",
    "map",
    "filter",
    "reduce",
    "abs",
    "sqrt",
    "pow",
    "min",
    "max",
    "print",
    "input",
    "read_file",
    "write_file",
    "file_exists",
];

struct Backend {
    client: Client,
    documents: RwLock<HashMap<Url, String>>,
    indexes: RwLock<HashMap<Url, DocumentIndex>>,
}

#[derive(Clone)]
struct Occurrence {
    name: String,
    range: Range,
    definition: bool,
    kind: SymbolKind,
    detail: Option<String>,
}

#[derive(Clone, Default)]
struct DocumentIndex {
    occurrences: Vec<Occurrence>,
}

impl Backend {
    async fn publish(&self, uri: Url, text: String, version: Option<i32>) {
        let diagnostics = analyze(&text);
        let index = index_document(&text);
        self.documents.write().await.insert(uri.clone(), text);
        self.indexes.write().await.insert(uri.clone(), index);
        self.client
            .publish_diagnostics(uri, diagnostics, version)
            .await;
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        let roots: Vec<PathBuf> = params
            .workspace_folders
            .unwrap_or_default()
            .into_iter()
            .filter_map(|folder| folder.uri.to_file_path().ok())
            .chain(params.root_uri.and_then(|uri| uri.to_file_path().ok()))
            .collect();
        let mut scan_roots = roots.clone();
        for root in &roots {
            if let Ok(project) = crate::package::discover(root) {
                if let Ok(dependencies) = crate::package::dependency_projects(&project) {
                    scan_roots.extend(dependencies.into_iter().map(|dependency| dependency.root));
                }
            }
        }
        scan_roots.sort();
        scan_roots.dedup();
        let mut documents = self.documents.write().await;
        let mut indexes = self.indexes.write().await;
        for root in scan_roots {
            for path in collect_ject_files(&root) {
                if let (Ok(text), Ok(uri)) = (fs::read_to_string(&path), Url::from_file_path(&path))
                {
                    indexes.insert(uri.clone(), index_document(&text));
                    documents.insert(uri, text);
                }
            }
        }
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                completion_provider: Some(CompletionOptions::default()),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                definition_provider: Some(OneOf::Left(true)),
                references_provider: Some(OneOf::Left(true)),
                rename_provider: Some(OneOf::Right(RenameOptions {
                    prepare_provider: Some(true),
                    work_done_progress_options: WorkDoneProgressOptions::default(),
                })),
                document_symbol_provider: Some(OneOf::Left(true)),
                workspace_symbol_provider: Some(OneOf::Left(true)),
                ..ServerCapabilities::default()
            },
            server_info: Some(ServerInfo {
                name: "ject-language-server".into(),
                version: Some(env!("CARGO_PKG_VERSION").into()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(
                MessageType::INFO,
                format!("Ject language server {} ready", env!("CARGO_PKG_VERSION")),
            )
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let doc = params.text_document;
        self.publish(doc.uri, doc.text, Some(doc.version)).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        if let Some(change) = params.content_changes.into_iter().last() {
            self.publish(
                params.text_document.uri,
                change.text,
                Some(params.text_document.version),
            )
            .await;
        }
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        if let Some(text) = params.text {
            self.publish(params.text_document.uri, text, None).await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri;
        if let Ok(path) = uri.to_file_path() {
            if let Ok(text) = fs::read_to_string(path) {
                self.documents
                    .write()
                    .await
                    .insert(uri.clone(), text.clone());
                self.indexes
                    .write()
                    .await
                    .insert(uri.clone(), index_document(&text));
            } else {
                self.documents.write().await.remove(&uri);
                self.indexes.write().await.remove(&uri);
            }
        }
        self.client.publish_diagnostics(uri, vec![], None).await;
    }

    async fn did_change_watched_files(&self, params: DidChangeWatchedFilesParams) {
        for change in params.changes {
            let Ok(path) = change.uri.to_file_path() else {
                continue;
            };
            if change.typ == FileChangeType::DELETED {
                self.documents.write().await.remove(&change.uri);
                self.indexes.write().await.remove(&change.uri);
            } else if matches!(
                path.extension().and_then(|ext| ext.to_str()),
                Some("ject" | "jt")
            ) {
                if let Ok(text) = fs::read_to_string(path) {
                    self.documents
                        .write()
                        .await
                        .insert(change.uri.clone(), text.clone());
                    self.indexes
                        .write()
                        .await
                        .insert(change.uri, index_document(&text));
                }
            }
        }
    }

    async fn completion(&self, _: CompletionParams) -> Result<Option<CompletionResponse>> {
        let mut items = Vec::with_capacity(KEYWORDS.len() + BUILTINS.len());
        items.extend(KEYWORDS.iter().map(|name| CompletionItem {
            label: (*name).into(),
            kind: Some(CompletionItemKind::KEYWORD),
            ..CompletionItem::default()
        }));
        items.extend(BUILTINS.iter().map(|name| CompletionItem {
            label: (*name).into(),
            kind: Some(CompletionItemKind::FUNCTION),
            detail: Some(builtin_signature(name).into()),
            ..CompletionItem::default()
        }));
        let indexes = self.indexes.read().await;
        let mut names = std::collections::BTreeMap::new();
        for occurrence in indexes
            .values()
            .flat_map(|index| &index.occurrences)
            .filter(|item| item.definition)
        {
            names
                .entry(occurrence.name.clone())
                .or_insert((occurrence.kind, occurrence.detail.clone()));
        }
        items.extend(
            names
                .into_iter()
                .map(|(name, (kind, detail))| CompletionItem {
                    label: name,
                    kind: Some(completion_kind(kind)),
                    detail,
                    ..CompletionItem::default()
                }),
        );
        Ok(Some(CompletionResponse::Array(items)))
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;
        let documents = self.documents.read().await;
        let Some(text) = documents.get(uri) else {
            return Ok(None);
        };
        let Some(word) = word_at(text, position) else {
            return Ok(None);
        };
        if KEYWORDS.contains(&word.as_str()) {
            return Ok(None);
        }
        let indexes = self.indexes.read().await;
        let description = if BUILTINS.contains(&word.as_str()) {
            format!(
                "```ject\n{}\n```\n\nJect built-in function",
                builtin_signature(&word)
            )
        } else {
            let local = indexes
                .get(uri)
                .into_iter()
                .flat_map(|index| &index.occurrences)
                .find(|item| item.definition && item.name == word)
                .map(|item| (uri, item));
            let Some((definition_uri, symbol)) = local.or_else(|| {
                indexes.iter().find_map(|(target, index)| {
                    index
                        .occurrences
                        .iter()
                        .find(|item| item.definition && item.name == word)
                        .map(|item| (target, item))
                })
            }) else {
                return Ok(None);
            };
            let detail = symbol
                .detail
                .clone()
                .unwrap_or_else(|| format!("{} {word}", symbol_kind_name(symbol.kind)));
            let location = definition_uri
                .to_file_path()
                .ok()
                .map(|path| path.display().to_string())
                .unwrap_or_else(|| definition_uri.to_string());
            format!(
                "```ject\n{detail}\n```\n\nDefined in `{location}:{}`",
                symbol.range.start.line + 1
            )
        };
        Ok(Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: description,
            }),
            range: None,
        }))
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let position = params.text_document_position_params.position;
        let uri = params.text_document_position_params.text_document.uri;
        let indexes = self.indexes.read().await;
        let Some(name) = symbol_at(indexes.get(&uri), position) else {
            return Ok(None);
        };
        let local = indexes
            .get(&uri)
            .into_iter()
            .flat_map(|index| &index.occurrences)
            .find(|item| item.definition && item.name == name)
            .map(|item| Location::new(uri.clone(), item.range));
        let location = local.or_else(|| {
            indexes
                .iter()
                .flat_map(|(target, index)| {
                    index
                        .occurrences
                        .iter()
                        .filter(|item| item.definition && item.name == name)
                        .map(|item| Location::new(target.clone(), item.range))
                })
                .next()
        });
        Ok(location.map(GotoDefinitionResponse::Scalar))
    }

    async fn references(&self, params: ReferenceParams) -> Result<Option<Vec<Location>>> {
        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;
        let indexes = self.indexes.read().await;
        let Some(name) = symbol_at(indexes.get(&uri), position) else {
            return Ok(None);
        };
        let locations = indexes
            .iter()
            .flat_map(|(target, index)| {
                index
                    .occurrences
                    .iter()
                    .filter(|item| {
                        item.name == name
                            && (params.context.include_declaration || !item.definition)
                    })
                    .map(|item| Location::new(target.clone(), item.range))
            })
            .collect();
        Ok(Some(locations))
    }

    async fn prepare_rename(
        &self,
        params: TextDocumentPositionParams,
    ) -> Result<Option<PrepareRenameResponse>> {
        let indexes = self.indexes.read().await;
        let Some(index) = indexes.get(&params.text_document.uri) else {
            return Ok(None);
        };
        Ok(index
            .occurrences
            .iter()
            .find(|item| contains(item.range, params.position))
            .map(|item| PrepareRenameResponse::RangeWithPlaceholder {
                range: item.range,
                placeholder: item.name.clone(),
            }))
    }

    async fn rename(&self, params: RenameParams) -> Result<Option<WorkspaceEdit>> {
        if !valid_identifier(&params.new_name) {
            return Err(tower_lsp::jsonrpc::Error::invalid_params(
                "new name is not a valid Ject identifier",
            ));
        }
        let uri = params.text_document_position.text_document.uri;
        let indexes = self.indexes.read().await;
        let Some(name) = symbol_at(indexes.get(&uri), params.text_document_position.position)
        else {
            return Ok(None);
        };
        let mut changes = HashMap::new();
        for (target, index) in indexes.iter() {
            let edits: Vec<_> = index
                .occurrences
                .iter()
                .filter(|item| item.name == name)
                .map(|item| TextEdit::new(item.range, params.new_name.clone()))
                .collect();
            if !edits.is_empty() {
                changes.insert(target.clone(), edits);
            }
        }
        Ok(Some(WorkspaceEdit {
            changes: Some(changes),
            ..WorkspaceEdit::default()
        }))
    }

    #[allow(deprecated)]
    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        let indexes = self.indexes.read().await;
        let symbols = indexes
            .get(&params.text_document.uri)
            .map(|index| {
                index
                    .occurrences
                    .iter()
                    .filter(|item| item.definition)
                    .map(|item| SymbolInformation {
                        name: item.name.clone(),
                        kind: item.kind,
                        tags: None,
                        deprecated: None,
                        location: Location::new(params.text_document.uri.clone(), item.range),
                        container_name: None,
                    })
                    .collect()
            })
            .unwrap_or_default();
        Ok(Some(DocumentSymbolResponse::Flat(symbols)))
    }

    #[allow(deprecated)]
    async fn symbol(
        &self,
        params: WorkspaceSymbolParams,
    ) -> Result<Option<Vec<SymbolInformation>>> {
        let query = params.query.to_lowercase();
        let indexes = self.indexes.read().await;
        Ok(Some(
            indexes
                .iter()
                .flat_map(|(uri, index)| {
                    index
                        .occurrences
                        .iter()
                        .filter(|item| item.definition && item.name.to_lowercase().contains(&query))
                        .map(|item| SymbolInformation {
                            name: item.name.clone(),
                            kind: item.kind,
                            tags: None,
                            deprecated: None,
                            location: Location::new(uri.clone(), item.range),
                            container_name: None,
                        })
                })
                .collect(),
        ))
    }
}

fn analyze(source: &str) -> Vec<Diagnostic> {
    let mut lexer = Lexer::new(source);
    let positioned: Vec<_> = lexer
        .tokenize_with_positions()
        .into_iter()
        .map(|token| (token.token, token.position))
        .collect();
    let mut parser = Parser::new(positioned.clone());
    match parser.parse() {
        Ok(statements) => {
            let mut linter = Linter::new().with_tokens_and_source(positioned, source.into());
            let (diagnostics, _) = linter.lint(&statements);
            diagnostics.into_iter().map(to_lsp_diagnostic).collect()
        }
        Err(error) => vec![to_lsp_diagnostic(diagnostic::parse_diagnostic(
            &error.message,
            error.line,
            error.column,
        ))],
    }
}

fn to_lsp_diagnostic(value: diagnostic::Diagnostic) -> Diagnostic {
    let span = value
        .labels
        .iter()
        .find(|label| label.primary)
        .map(|label| label.span)
        .or_else(|| {
            value
                .line
                .zip(value.column)
                .map(|(line, column)| SourceSpan::new(line, column, 1))
        })
        .unwrap_or_else(|| SourceSpan::new(1, 1, 1));
    let start = Position::new((span.line - 1) as u32, (span.column - 1) as u32);
    let end = Position::new(start.line, start.character + span.length.max(1) as u32);
    let mut message = value.message;
    for note in value.notes {
        message.push_str("\n\nnote: ");
        message.push_str(&note);
    }
    if let Some(help) = value.help {
        message.push_str("\n\nhelp: ");
        message.push_str(&help);
    }
    Diagnostic {
        range: Range::new(start, end),
        severity: Some(match value.level {
            DiagnosticLevel::Error => DiagnosticSeverity::ERROR,
            DiagnosticLevel::Warning => DiagnosticSeverity::WARNING,
            DiagnosticLevel::Note => DiagnosticSeverity::INFORMATION,
            DiagnosticLevel::Help => DiagnosticSeverity::HINT,
        }),
        code: value.code.map(NumberOrString::String),
        source: Some("ject".into()),
        message,
        ..Diagnostic::default()
    }
}

fn word_at(text: &str, position: Position) -> Option<String> {
    let line = text.lines().nth(position.line as usize)?;
    let index = (position.character as usize).min(line.len());
    let bytes = line.as_bytes();
    let mut start = index;
    let mut end = index;
    while start > 0 && (bytes[start - 1].is_ascii_alphanumeric() || bytes[start - 1] == b'_') {
        start -= 1;
    }
    while end < bytes.len() && (bytes[end].is_ascii_alphanumeric() || bytes[end] == b'_') {
        end += 1;
    }
    (start < end).then(|| line[start..end].to_string())
}

fn index_document(source: &str) -> DocumentIndex {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize_with_positions();
    let mut occurrences = Vec::new();
    let mut previous: Option<&Token> = None;
    let mut expect_for_binding = false;
    for (token_index, located) in tokens.iter().enumerate() {
        if matches!(located.token, Token::For) {
            expect_for_binding = true;
        }
        if let Token::Identifier(name) = &located.token {
            let after_dot = matches!(previous, Some(Token::Dot));
            if !after_dot {
                let definition = matches!(
                    previous,
                    Some(Token::Let | Token::Fn | Token::Struct | Token::Catch | Token::As)
                ) || expect_for_binding;
                let kind = match previous {
                    Some(Token::Fn) => SymbolKind::FUNCTION,
                    Some(Token::Struct) => SymbolKind::STRUCT,
                    _ => SymbolKind::VARIABLE,
                };
                let detail = if definition {
                    match previous {
                        Some(Token::Fn) => Some(function_signature(&tokens, token_index, name)),
                        Some(Token::Struct) => Some(format!("struct {name}")),
                        Some(Token::Let) => Some(format!(
                            "let {name}: {}",
                            inferred_assignment_type(&tokens, token_index)
                        )),
                        Some(Token::As) => Some(format!("module {name}")),
                        Some(Token::Catch) => Some(format!("let {name}: error")),
                        _ if expect_for_binding => Some(format!("let {name}: unknown")),
                        _ => None,
                    }
                } else {
                    None
                };
                let start = Position::new(
                    (located.position.line.saturating_sub(1)) as u32,
                    (located.position.column.saturating_sub(1)) as u32,
                );
                let end = Position::new(
                    start.line,
                    start.character + name.encode_utf16().count() as u32,
                );
                occurrences.push(Occurrence {
                    name: name.clone(),
                    range: Range::new(start, end),
                    definition,
                    kind,
                    detail,
                });
            }
            expect_for_binding = false;
        }
        if !matches!(located.token, Token::Newline) {
            previous = Some(&located.token);
        }
    }
    DocumentIndex { occurrences }
}

fn function_signature(
    tokens: &[crate::lexer::LocatedToken],
    name_index: usize,
    name: &str,
) -> String {
    let mut params = Vec::new();
    let mut depth = 0usize;
    let mut expect_parameter = false;
    for located in tokens.iter().skip(name_index + 1) {
        match &located.token {
            Token::LeftParen => {
                depth += 1;
                if depth == 1 {
                    expect_parameter = true;
                }
            }
            Token::RightParen if depth == 1 => break,
            Token::RightParen => depth = depth.saturating_sub(1),
            Token::Comma if depth == 1 => expect_parameter = true,
            Token::Identifier(param) if depth == 1 && expect_parameter => {
                params.push(param.clone());
                expect_parameter = false;
            }
            _ => {}
        }
    }
    format!("fn {name}({})", params.join(", "))
}

fn inferred_assignment_type(
    tokens: &[crate::lexer::LocatedToken],
    name_index: usize,
) -> &'static str {
    let value = tokens
        .iter()
        .skip(name_index + 1)
        .skip_while(|item| !matches!(item.token, Token::Equal))
        .skip(1)
        .find(|item| !matches!(item.token, Token::Newline))
        .map(|item| &item.token);
    match value {
        Some(Token::Integer(_)) => "integer",
        Some(Token::Float(_)) => "float",
        Some(Token::String(_) | Token::InterpolatedString(_)) => "string",
        Some(Token::True | Token::False | Token::Bool(_)) => "boolean",
        Some(Token::Nil) => "nil",
        Some(Token::LeftBracket) => "array",
        Some(Token::LeftBrace | Token::LeftBracePipe) => "collection",
        Some(Token::Fn | Token::Lambda) => "function",
        _ => "unknown",
    }
}

fn builtin_signature(name: &str) -> &'static str {
    match name {
        "type_of" => "type_of(value) -> string",
        "to_int" => "to_int(value) -> integer",
        "to_float" => "to_float(value) -> float",
        "to_string" => "to_string(value) -> string",
        "to_bool" => "to_bool(value) -> boolean",
        "len" => "len(value) -> integer",
        "range" => "range(start, stop?, step?) -> array",
        "push" => "push(array, value) -> array",
        "pop" => "pop(array) -> array",
        "sum" => "sum(array) -> number",
        "contains" => "contains(collection, value) -> boolean",
        "index_of" => "index_of(array, value) -> integer",
        "first" => "first(array) -> any",
        "last" => "last(array) -> any",
        "slice" => "slice(array, start, end) -> array",
        "sort" => "sort(array) -> array",
        "reverse" => "reverse(array) -> array",
        "unique" => "unique(array) -> array",
        "map" => "map(array, function) -> array",
        "filter" => "filter(array, predicate) -> array",
        "reduce" => "reduce(array, function, initial) -> any",
        "sqrt" => "sqrt(number) -> float",
        "pow" => "pow(base, exponent) -> number",
        "print" => "print(...values)",
        "input" => "input(prompt?) -> string",
        "read_file" => "read_file(path) -> string",
        "write_file" => "write_file(path, content) -> nil",
        "file_exists" => "file_exists(path) -> boolean",
        _ => "builtin(...)",
    }
}

fn symbol_at(index: Option<&DocumentIndex>, position: Position) -> Option<String> {
    index?
        .occurrences
        .iter()
        .find(|item| contains(item.range, position))
        .map(|item| item.name.clone())
}

fn contains(range: Range, position: Position) -> bool {
    position.line == range.start.line
        && position.character >= range.start.character
        && position.character <= range.end.character
}

fn valid_identifier(name: &str) -> bool {
    let mut chars = name.chars();
    chars.next().is_some_and(|c| c == '_' || c.is_alphabetic())
        && chars.all(|c| c == '_' || c.is_alphanumeric())
        && !KEYWORDS.contains(&name)
}

fn completion_kind(kind: SymbolKind) -> CompletionItemKind {
    if kind == SymbolKind::FUNCTION {
        CompletionItemKind::FUNCTION
    } else if kind == SymbolKind::STRUCT {
        CompletionItemKind::STRUCT
    } else {
        CompletionItemKind::VARIABLE
    }
}

fn symbol_kind_name(kind: SymbolKind) -> &'static str {
    if kind == SymbolKind::FUNCTION {
        "function"
    } else if kind == SymbolKind::STRUCT {
        "struct"
    } else {
        "variable"
    }
}

fn collect_ject_files(root: &Path) -> Vec<PathBuf> {
    let mut result = Vec::new();
    let mut pending = vec![root.to_path_buf()];
    while let Some(directory) = pending.pop() {
        let Ok(entries) = fs::read_dir(directory) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let ignored = path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| {
                        matches!(name, ".git" | "target" | "node_modules" | ".ject")
                    });
                if !ignored {
                    pending.push(path);
                }
            } else if matches!(
                path.extension().and_then(|ext| ext.to_str()),
                Some("ject" | "jt")
            ) {
                result.push(path);
            }
        }
    }
    result
}

pub fn run() {
    let runtime =
        tokio::runtime::Runtime::new().expect("failed to start the Ject language server runtime");
    runtime.block_on(async {
        let stdin = tokio::io::stdin();
        let stdout = tokio::io::stdout();
        let (service, socket) = LspService::new(|client| Backend {
            client,
            documents: RwLock::new(HashMap::new()),
            indexes: RwLock::new(HashMap::new()),
        });
        Server::new(stdin, stdout, socket).serve(service).await;
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_parser_and_linter_diagnostics() {
        assert!(analyze("let unused = 1")
            .iter()
            .any(|d| d.severity == Some(DiagnosticSeverity::WARNING)));
        assert!(analyze("let =")
            .iter()
            .any(|d| d.severity == Some(DiagnosticSeverity::ERROR)));
    }

    #[test]
    fn exported_api_is_clean_in_lsp() {
        assert!(analyze("export VERSION = \"1.0\"").is_empty());
    }

    #[test]
    fn indexes_definitions_and_references() {
        let index = index_document(
            "let answer = 42\nprint answer\nfn greet(name, prefix=make_prefix())\n    print name\nend",
        );
        assert!(index.occurrences.iter().any(|item| item.name == "answer"
            && item.definition
            && item.detail.as_deref() == Some("let answer: integer")));
        assert!(index
            .occurrences
            .iter()
            .any(|item| item.name == "answer" && !item.definition));
        assert!(index.occurrences.iter().any(|item| item.name == "greet"
            && item.kind == SymbolKind::FUNCTION
            && item.detail.as_deref() == Some("fn greet(name, prefix)")));
    }

    #[test]
    fn infers_basic_hover_types() {
        for (source, expected) in [
            ("let value = 1.5", "let value: float"),
            ("let value = \"hello\"", "let value: string"),
            ("let value = [1, 2]", "let value: array"),
            ("let value = true", "let value: boolean"),
        ] {
            assert_eq!(
                index_document(source).occurrences[0].detail.as_deref(),
                Some(expected)
            );
        }
    }

    #[test]
    fn workspace_scan_ignores_build_outputs() {
        let files = collect_ject_files(Path::new(env!("CARGO_MANIFEST_DIR")));
        assert!(files.iter().any(|path| path.ends_with("stdlib/jnum.ject")));
        assert!(!files
            .iter()
            .any(|path| path.components().any(|part| part.as_os_str() == "target")));
    }
}
