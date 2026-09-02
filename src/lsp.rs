//! Ject Language Server Protocol implementation.
//!
//! This server deliberately uses the compiler's lexer, parser, and linter instead
//! of parsing rendered CLI output. Editors therefore receive stable structured
//! diagnostics for the exact in-memory text the user is editing.

use crate::diagnostic::{self, DiagnosticLevel, SourceSpan};
use crate::lexer::{Lexer, Token};
use crate::linter::Linter;
use crate::module_interface::{ExportKind, ModuleInterface};
use crate::parser::Parser;
use std::collections::BTreeMap;
use std::collections::{HashMap, HashSet};
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
    graph_cache: RwLock<
        HashMap<
            Url,
            std::result::Result<
                crate::module_graph::ModuleGraph,
                crate::module_graph::ModuleGraphError,
            >,
        >,
    >,
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
    semantic: crate::semantic::SemanticIndex,
    exports: HashMap<String, crate::semantic::SymbolId>,
    selective_imports: HashMap<crate::semantic::SymbolId, (String, String)>,
    module_aliases: HashMap<String, String>,
    imports: Vec<String>,
    glob_imports: Vec<String>,
    reexport_references: Vec<(SourceSpan, String)>,
}

impl Backend {
    async fn publish(&self, uri: Url, text: String, version: Option<i32>) {
        let index = index_document(&text);
        self.documents.write().await.insert(uri.clone(), text);
        self.indexes.write().await.insert(uri.clone(), index);
        let dependents = {
            let indexes = self.indexes.read().await;
            dependent_documents(&indexes, &uri)
        };
        {
            let mut cache = self.graph_cache.write().await;
            cache.remove(&uri);
            for dependent in &dependents {
                cache.remove(dependent);
            }
        }
        let diagnostics = self.diagnostics_for(&uri).await;
        self.client
            .publish_diagnostics(uri.clone(), diagnostics, version)
            .await;

        for dependent in dependents {
            let diagnostics = self.diagnostics_for(&dependent).await;
            self.client
                .publish_diagnostics(dependent, diagnostics, None)
                .await;
        }
    }

    async fn diagnostics_for(&self, uri: &Url) -> Vec<Diagnostic> {
        let documents = self.documents.read().await;
        let Some(text) = documents.get(uri).cloned() else {
            return Vec::new();
        };
        let sources = documents
            .iter()
            .filter_map(|(uri, source)| {
                let path = uri.to_file_path().ok()?;
                let canonical = path.canonicalize().ok()?;
                Some((canonical, source.clone()))
            })
            .collect();
        drop(documents);
        let Some(path) = uri.to_file_path().ok() else {
            return analyze(&text);
        };
        let mut diagnostics = analyze_at_with_sources_without_graph(&text, &path, &sources);
        if path.is_file() {
            let cached = self.graph_cache.read().await.get(uri).cloned();
            let graph = match cached {
                Some(graph) => graph,
                None => {
                    let graph = crate::module_graph::ModuleGraph::build_sources(
                        &path,
                        text.clone(),
                        &sources,
                    );
                    self.graph_cache
                        .write()
                        .await
                        .insert(uri.clone(), graph.clone());
                    graph
                }
            };
            if let Ok(graph) = &graph {
                let discovered = graph_documents(graph, &sources);
                let mut documents = self.documents.write().await;
                let mut indexes = self.indexes.write().await;
                for (module_uri, module_source) in discovered {
                    indexes
                        .entry(module_uri.clone())
                        .or_insert_with(|| index_document(&module_source));
                    documents.entry(module_uri).or_insert(module_source);
                }
            }
            append_graph_diagnostic(&mut diagnostics, &graph, Some(&path));
        }
        diagnostics
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
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec![".".into(), "\"".into()]),
                    ..CompletionOptions::default()
                }),
                signature_help_provider: Some(SignatureHelpOptions {
                    trigger_characters: Some(vec!["(".into(), ",".into()]),
                    retrigger_characters: Some(vec![",".into()]),
                    ..SignatureHelpOptions::default()
                }),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                definition_provider: Some(OneOf::Left(true)),
                references_provider: Some(OneOf::Left(true)),
                document_highlight_provider: Some(OneOf::Left(true)),
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
        let text = match params.text {
            Some(text) => Some(text),
            None => self
                .documents
                .read()
                .await
                .get(&params.text_document.uri)
                .cloned(),
        };
        if let Some(text) = text {
            self.publish(params.text_document.uri, text, None).await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri;
        if let Ok(path) = uri.to_file_path() {
            if let Ok(text) = fs::read_to_string(path) {
                self.publish(uri.clone(), text, None).await;
            } else {
                let dependents = {
                    let indexes = self.indexes.read().await;
                    dependent_documents(&indexes, &uri)
                };
                self.documents.write().await.remove(&uri);
                self.indexes.write().await.remove(&uri);
                self.graph_cache.write().await.remove(&uri);
                {
                    let mut cache = self.graph_cache.write().await;
                    for dependent in &dependents {
                        cache.remove(dependent);
                    }
                }
                for dependent in dependents {
                    let diagnostics = self.diagnostics_for(&dependent).await;
                    self.client
                        .publish_diagnostics(dependent, diagnostics, None)
                        .await;
                }
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
                let dependents = {
                    let indexes = self.indexes.read().await;
                    dependent_documents(&indexes, &change.uri)
                };
                self.documents.write().await.remove(&change.uri);
                self.indexes.write().await.remove(&change.uri);
                self.graph_cache.write().await.remove(&change.uri);
                {
                    let mut cache = self.graph_cache.write().await;
                    for dependent in &dependents {
                        cache.remove(dependent);
                    }
                }
                self.client
                    .publish_diagnostics(change.uri, Vec::new(), None)
                    .await;
                for dependent in dependents {
                    let diagnostics = self.diagnostics_for(&dependent).await;
                    self.client
                        .publish_diagnostics(dependent, diagnostics, None)
                        .await;
                }
            } else if matches!(
                path.extension().and_then(|ext| ext.to_str()),
                Some("ject" | "jt")
            ) {
                if let Ok(text) = fs::read_to_string(path) {
                    self.publish(change.uri, text, None).await;
                }
            }
        }
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let indexes = self.indexes.read().await;
        let documents = self.documents.read().await;
        let uri = &params.text_document_position.text_document.uri;
        let items = completion_items(
            &indexes,
            &documents,
            uri,
            params.text_document_position.position,
        );
        Ok(Some(CompletionResponse::Array(items)))
    }

    async fn signature_help(&self, params: SignatureHelpParams) -> Result<Option<SignatureHelp>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;
        let indexes = self.indexes.read().await;
        let documents = self.documents.read().await;
        Ok(signature_help_at(&indexes, &documents, uri, position))
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
            let Some(identity) = symbol_identity_at(&indexes, &documents, uri, position) else {
                return Ok(None);
            };
            let Some((definition_uri, symbol)) = definition_occurrence(&indexes, &identity) else {
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
        let documents = self.documents.read().await;
        let Some((target_uri, target_id)) =
            symbol_identity_at(&indexes, &documents, &uri, position)
        else {
            return Ok(None);
        };
        let location = indexes
            .get(&target_uri)
            .and_then(|index| index.semantic.symbols.get(target_id))
            .map(|symbol| Location::new(target_uri, range_from_span(symbol.span)));
        Ok(location.map(GotoDefinitionResponse::Scalar))
    }

    async fn references(&self, params: ReferenceParams) -> Result<Option<Vec<Location>>> {
        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;
        let indexes = self.indexes.read().await;
        let documents = self.documents.read().await;
        let Some(identity) = symbol_identity_at(&indexes, &documents, &uri, position) else {
            return Ok(None);
        };
        let locations = locations_for_identity(&indexes, &documents, &identity)
            .into_iter()
            .filter(|(_, declaration)| params.context.include_declaration || !declaration)
            .map(|(location, _)| location)
            .collect();
        Ok(Some(locations))
    }

    async fn document_highlight(
        &self,
        params: DocumentHighlightParams,
    ) -> Result<Option<Vec<DocumentHighlight>>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;
        let indexes = self.indexes.read().await;
        let documents = self.documents.read().await;
        let Some(identity) = symbol_identity_at(&indexes, &documents, &uri, position) else {
            return Ok(None);
        };
        Ok(Some(document_highlights_for_identity(
            &indexes, &documents, &uri, &identity,
        )))
    }

    async fn prepare_rename(
        &self,
        params: TextDocumentPositionParams,
    ) -> Result<Option<PrepareRenameResponse>> {
        let indexes = self.indexes.read().await;
        let documents = self.documents.read().await;
        let Some(index) = indexes.get(&params.text_document.uri) else {
            return Ok(None);
        };
        if symbol_identity_at(
            &indexes,
            &documents,
            &params.text_document.uri,
            params.position,
        )
        .and_then(|identity| definition_occurrence(&indexes, &identity))
        .is_none()
        {
            return Ok(None);
        }
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
        let documents = self.documents.read().await;
        let Some(identity) = symbol_identity_at(
            &indexes,
            &documents,
            &uri,
            params.text_document_position.position,
        ) else {
            return Ok(None);
        };
        if let Some(message) = rename_conflict(&indexes, &documents, &identity, &params.new_name) {
            return Err(tower_lsp::jsonrpc::Error::invalid_params(message));
        }
        let mut changes = HashMap::new();
        for (location, _) in locations_for_identity(&indexes, &documents, &identity) {
            changes
                .entry(location.uri)
                .or_insert_with(Vec::new)
                .push(TextEdit::new(location.range, params.new_name.clone()));
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
        let mut symbols = indexes
            .iter()
            .flat_map(|(uri, index)| {
                let container = symbol_container(uri);
                index
                    .occurrences
                    .iter()
                    .filter(|item| item.definition && item.name.to_lowercase().contains(&query))
                    .map(move |item| SymbolInformation {
                        name: item.name.clone(),
                        kind: item.kind,
                        tags: None,
                        deprecated: None,
                        location: Location::new(uri.clone(), item.range),
                        container_name: container.clone(),
                    })
            })
            .collect::<Vec<_>>();
        symbols.sort_by(|left, right| {
            left.name
                .cmp(&right.name)
                .then_with(|| left.container_name.cmp(&right.container_name))
                .then_with(|| left.location.uri.as_str().cmp(right.location.uri.as_str()))
                .then_with(|| {
                    left.location
                        .range
                        .start
                        .line
                        .cmp(&right.location.range.start.line)
                })
        });
        Ok(Some(symbols))
    }
}

fn analyze(source: &str) -> Vec<Diagnostic> {
    let base = std::env::current_dir().unwrap_or_default();
    analyze_at(source, &base)
}

fn analyze_at(source: &str, path: &Path) -> Vec<Diagnostic> {
    analyze_at_with_sources(source, path, &HashMap::new())
}

fn analyze_at_with_sources(
    source: &str,
    path: &Path,
    sources: &HashMap<PathBuf, String>,
) -> Vec<Diagnostic> {
    let mut diagnostics = analyze_at_with_sources_without_graph(source, path, sources);
    if path.is_file() {
        let graph =
            crate::module_graph::ModuleGraph::build_sources(path, source.to_string(), sources);
        append_graph_diagnostic(&mut diagnostics, &graph, Some(path));
    }
    diagnostics
}

fn analyze_at_with_sources_without_graph(
    source: &str,
    path: &Path,
    sources: &HashMap<PathBuf, String>,
) -> Vec<Diagnostic> {
    let mut lexer = Lexer::new(source);
    let positioned: Vec<_> = lexer
        .tokenize_with_positions()
        .into_iter()
        .map(|token| (token.token, token.position))
        .collect();
    let mut parser = Parser::new(positioned.clone());
    match parser.parse() {
        Ok(statements) => {
            let mut linter = Linter::new()
                .with_tokens_and_source(positioned, source.into())
                .with_source_path(path)
                .with_source_overrides(sources.clone());
            let (diagnostics, _) = linter.lint(&statements);
            diagnostics
                .into_iter()
                .map(to_lsp_diagnostic)
                .collect::<Vec<_>>()
        }
        Err(error) => vec![to_lsp_diagnostic(diagnostic::parse_diagnostic(
            &error.message,
            error.line,
            error.column,
        ))],
    }
}

fn append_graph_diagnostic(
    diagnostics: &mut Vec<Diagnostic>,
    graph: &std::result::Result<
        crate::module_graph::ModuleGraph,
        crate::module_graph::ModuleGraphError,
    >,
    current_path: Option<&Path>,
) {
    let Err(error) = graph else {
        return;
    };
    let report = match error {
        crate::module_graph::ModuleGraphError::Cycle { .. } => Some((
            "E3102",
            "break the cycle by moving shared code into a third module",
        )),
        crate::module_graph::ModuleGraphError::Load { chain, .. } if chain.len() > 2 => {
            Some(("E3101", "check every module in the displayed import chain"))
        }
        _ => None,
    };
    if let Some((code, help)) = report {
        let mut diagnostic = to_lsp_diagnostic(
            diagnostic::Diagnostic::error(error.to_string())
                .with_code(code.into())
                .with_help(help.into()),
        );
        if let Some(site) = error.import_site() {
            if let ject::module_resolver::ModuleIdentity::File(path) = &site.module {
                let range = Range::new(
                    Position::new(
                        site.line.saturating_sub(1) as u32,
                        site.column.saturating_sub(1) as u32,
                    ),
                    Position::new(
                        site.line.saturating_sub(1) as u32,
                        (site.column.saturating_sub(1) + site.length) as u32,
                    ),
                );
                let is_current = current_path.is_some_and(|current| {
                    fs::canonicalize(current).ok() == fs::canonicalize(path).ok()
                });
                if is_current {
                    diagnostic.range = range;
                } else if let Ok(uri) = Url::from_file_path(path) {
                    diagnostic.related_information = Some(vec![DiagnosticRelatedInformation {
                        location: Location::new(uri, range),
                        message: format!("import of `{}` occurs here", site.specifier),
                    }]);
                }
            }
        }
        diagnostics.push(diagnostic);
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
    let semantic = crate::semantic::analyze(source);
    let inferred_types = infer_symbol_types(&tokens, &semantic);
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
                    Some(
                        Token::Let
                            | Token::Fn
                            | Token::Struct
                            | Token::Catch
                            | Token::As
                            | Token::Export
                    )
                ) || expect_for_binding;
                let kind = match previous {
                    Some(Token::Fn) => SymbolKind::FUNCTION,
                    Some(Token::Struct) => SymbolKind::STRUCT,
                    _ => SymbolKind::VARIABLE,
                };
                let detail = if definition {
                    match previous {
                        Some(Token::Fn) => {
                            Some(function_signature(source, &tokens, token_index, name))
                        }
                        Some(Token::Struct) => Some(format!("struct {name}")),
                        Some(Token::Let | Token::Export) => semantic
                            .symbol_at(located.position.line, located.position.column)
                            .and_then(|id| inferred_types.get(&id))
                            .map(|kind| format!("let {name}: {kind}"))
                            .or_else(|| Some(format!("let {name}: unknown"))),
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
            } else {
                let start = Position::new(
                    (located.position.line.saturating_sub(1)) as u32,
                    (located.position.column.saturating_sub(1)) as u32,
                );
                occurrences.push(Occurrence {
                    name: name.clone(),
                    range: Range::new(
                        start,
                        Position::new(
                            start.line,
                            start.character + name.encode_utf16().count() as u32,
                        ),
                    ),
                    definition: false,
                    kind: SymbolKind::FIELD,
                    detail: None,
                });
            }
            expect_for_binding = false;
        }
        if !matches!(located.token, Token::Newline) {
            previous = Some(&located.token);
        }
    }
    let (exports, selective_imports, module_aliases, imports, glob_imports, reexport_references) =
        module_metadata(&tokens, &semantic);
    DocumentIndex {
        occurrences,
        semantic,
        exports,
        selective_imports,
        module_aliases,
        imports,
        glob_imports,
        reexport_references,
    }
}

fn function_signature(
    source: &str,
    tokens: &[crate::lexer::LocatedToken],
    name_index: usize,
    name: &str,
) -> String {
    let mut depth = 0usize;
    let mut left = None;
    let mut right = None;
    for located in tokens.iter().skip(name_index + 1) {
        match &located.token {
            Token::LeftParen => {
                if depth == 0 {
                    left = Some(located.position.position);
                }
                depth += 1;
            }
            Token::RightParen if depth == 1 => {
                right = Some(located.position.position);
                break;
            }
            Token::RightParen => depth = depth.saturating_sub(1),
            _ => {}
        }
    }
    let chars = source.chars().collect::<Vec<_>>();
    let params = left
        .zip(right)
        .filter(|(left, right)| left < right && *right <= chars.len())
        .map(|(left, right)| chars[left + 1..right].iter().collect::<String>())
        .map(|raw| normalize_parameter_list(&raw))
        .unwrap_or_default();
    format!("fn {name}({params})")
}

fn normalize_parameter_list(raw: &str) -> String {
    split_top_level(raw)
        .into_iter()
        .map(|parameter| parameter.trim().replace(['\n', '\r'], " "))
        .filter(|parameter| !parameter.is_empty())
        .collect::<Vec<_>>()
        .join(", ")
}

type ModuleMetadata = (
    HashMap<String, crate::semantic::SymbolId>,
    HashMap<crate::semantic::SymbolId, (String, String)>,
    HashMap<String, String>,
    Vec<String>,
    Vec<String>,
    Vec<(SourceSpan, String)>,
);

fn module_metadata(
    tokens: &[crate::lexer::LocatedToken],
    semantic: &crate::semantic::SemanticIndex,
) -> ModuleMetadata {
    let mut exports = HashMap::new();
    let mut selective_imports = HashMap::new();
    let mut module_aliases = HashMap::new();
    let mut imports = Vec::new();
    let mut glob_imports = Vec::new();
    let mut reexport_references = Vec::new();
    let mut index = 0usize;
    while index < tokens.len() {
        match &tokens[index].token {
            Token::Export => {
                let mut name_index = index + 1;
                if matches!(
                    tokens.get(name_index).map(|item| &item.token),
                    Some(Token::Fn)
                ) {
                    name_index += 1;
                }
                if let Some(
                    located @ crate::lexer::LocatedToken {
                        token: Token::Identifier(name),
                        ..
                    },
                ) = tokens.get(name_index)
                {
                    if let Some(id) =
                        semantic.symbol_at(located.position.line, located.position.column)
                    {
                        exports.insert(name.clone(), id);
                    }
                    if !matches!(
                        tokens.get(index + 1).map(|item| &item.token),
                        Some(Token::Fn)
                    ) {
                        let equals = name_index + 1;
                        if matches!(
                            tokens.get(equals).map(|item| &item.token),
                            Some(Token::Equal)
                        ) {
                            if let Some(crate::lexer::LocatedToken {
                                token: Token::Identifier(source_name),
                                position,
                            }) = tokens.get(equals + 1)
                            {
                                reexport_references.push((
                                    SourceSpan::new(
                                        position.line,
                                        position.column,
                                        source_name.chars().count(),
                                    ),
                                    source_name.clone(),
                                ));
                            }
                        }
                    }
                }
            }
            Token::Import => {
                if matches!(
                    tokens.get(index + 1).map(|item| &item.token),
                    Some(Token::LeftBrace)
                ) {
                    let mut imported = Vec::new();
                    let mut cursor = index + 2;
                    while cursor < tokens.len()
                        && !matches!(tokens[cursor].token, Token::RightBrace)
                    {
                        if let Token::Identifier(name) = &tokens[cursor].token {
                            imported.push((name.clone(), tokens[cursor].position.clone()));
                        }
                        cursor += 1;
                    }
                    cursor += 1;
                    if matches!(
                        tokens.get(cursor).map(|item| &item.token),
                        Some(Token::From)
                    ) {
                        cursor += 1;
                    }
                    if let Some(crate::lexer::LocatedToken {
                        token: Token::String(module),
                        ..
                    }) = tokens.get(cursor)
                    {
                        imports.push(module.clone());
                        for (name, position) in imported {
                            if let Some(id) = semantic.symbol_at(position.line, position.column) {
                                selective_imports.insert(id, (module.clone(), name));
                            }
                        }
                    }
                } else if let Some(crate::lexer::LocatedToken {
                    token: Token::String(module),
                    ..
                }) = tokens.get(index + 1)
                {
                    imports.push(module.clone());
                    if matches!(
                        tokens.get(index + 2).map(|item| &item.token),
                        Some(Token::As)
                    ) {
                        if let Some(crate::lexer::LocatedToken {
                            token: Token::Identifier(alias),
                            ..
                        }) = tokens.get(index + 3)
                        {
                            module_aliases.insert(alias.clone(), module.clone());
                        }
                    } else {
                        glob_imports.push(module.clone());
                    }
                }
            }
            _ => {}
        }
        index += 1;
    }
    (
        exports,
        selective_imports,
        module_aliases,
        imports,
        glob_imports,
        reexport_references,
    )
}

fn infer_symbol_types(
    tokens: &[crate::lexer::LocatedToken],
    semantic: &crate::semantic::SemanticIndex,
) -> HashMap<crate::semantic::SymbolId, String> {
    let mut types = HashMap::new();
    for symbol in &semantic.symbols {
        if symbol.kind != crate::semantic::SymbolKind::Variable {
            continue;
        }
        let Some(name_index) = tokens.iter().position(|token| {
            token.position.line == symbol.span.line
                && token.position.column == symbol.span.column
                && matches!(&token.token, Token::Identifier(name) if name == &symbol.name)
        }) else {
            continue;
        };
        let Some(equal_index) = tokens
            .iter()
            .enumerate()
            .skip(name_index + 1)
            .take_while(|(_, token)| !matches!(token.token, Token::Newline | Token::Eof))
            .find_map(|(index, token)| matches!(token.token, Token::Equal).then_some(index))
        else {
            continue;
        };
        let expression = tokens
            .iter()
            .skip(equal_index + 1)
            .take_while(|token| !matches!(token.token, Token::Newline | Token::Eof))
            .collect::<Vec<_>>();
        types.insert(
            symbol.id,
            infer_expression_type(&expression, semantic, &types),
        );
    }
    types
}

fn infer_expression_type(
    expression: &[&crate::lexer::LocatedToken],
    semantic: &crate::semantic::SemanticIndex,
    types: &HashMap<crate::semantic::SymbolId, String>,
) -> String {
    let tokens = expression
        .iter()
        .map(|located| &located.token)
        .collect::<Vec<_>>();
    if tokens.iter().any(|token| {
        matches!(
            token,
            Token::EqualEqual
                | Token::BangEqual
                | Token::Less
                | Token::Greater
                | Token::LessEqual
                | Token::GreaterEqual
                | Token::And
                | Token::Or
                | Token::Bang
        )
    }) {
        return "boolean".into();
    }
    if let Some(Token::Identifier(name)) = tokens.first() {
        if matches!(tokens.get(1), Some(Token::LeftParen)) {
            if let Some(kind) = builtin_return_type(name) {
                return kind.into();
            }
        }
        if tokens.len() == 1 {
            if let Some(id) =
                semantic.symbol_at(expression[0].position.line, expression[0].position.column)
            {
                return types.get(&id).cloned().unwrap_or_else(|| "unknown".into());
            }
        }
    }
    if matches!(tokens.first(), Some(Token::New)) {
        if let Some(Token::Identifier(name)) = tokens.get(1) {
            return name.clone();
        }
    }
    if matches!(tokens.first(), Some(Token::LeftBracket)) {
        return "array".into();
    }
    if matches!(
        tokens.first(),
        Some(Token::LeftBrace | Token::LeftBracePipe)
    ) {
        return "collection".into();
    }
    if matches!(tokens.first(), Some(Token::Fn | Token::Lambda)) {
        return "function".into();
    }
    let referenced_types = expression
        .iter()
        .filter_map(|located| match located.token {
            Token::Identifier(_) => semantic
                .symbol_at(located.position.line, located.position.column)
                .and_then(|id| types.get(&id))
                .map(String::as_str),
            _ => None,
        })
        .collect::<Vec<_>>();
    if tokens
        .iter()
        .any(|token| matches!(token, Token::String(_) | Token::InterpolatedString(_)))
        || referenced_types.contains(&"string")
    {
        return "string".into();
    }
    if tokens.iter().any(|token| matches!(token, Token::Float(_)))
        || tokens.iter().any(|token| matches!(token, Token::Slash))
        || referenced_types.contains(&"float")
    {
        return "float".into();
    }
    if tokens
        .iter()
        .any(|token| matches!(token, Token::Integer(_)))
        || referenced_types.contains(&"integer")
    {
        return "integer".into();
    }
    if tokens
        .iter()
        .any(|token| matches!(token, Token::True | Token::False | Token::Bool(_)))
        || referenced_types.contains(&"boolean")
    {
        return "boolean".into();
    }
    if matches!(tokens.first(), Some(Token::Nil)) {
        return "nil".into();
    }
    "unknown".into()
}

fn builtin_return_type(name: &str) -> Option<&'static str> {
    Some(match name {
        "type_of" | "to_string" | "input" | "read_file" => "string",
        "to_int" | "len" | "index_of" => "integer",
        "to_float" | "sqrt" => "float",
        "to_bool" | "contains" | "file_exists" => "boolean",
        "range" | "push" | "pop" | "sort" | "reverse" | "unique" | "map" | "filter" => "array",
        "sum" | "pow" | "min" | "max" | "abs" => "number",
        "write_file" | "print" => "nil",
        _ => return None,
    })
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

fn semantic_symbol_at(
    index: &DocumentIndex,
    position: Position,
) -> Option<crate::semantic::SymbolId> {
    index
        .semantic
        .symbol_at(position.line as usize + 1, position.character as usize + 1)
}

fn range_from_span(span: SourceSpan) -> Range {
    let start = Position::new(
        span.line.saturating_sub(1) as u32,
        span.column.saturating_sub(1) as u32,
    );
    Range::new(
        start,
        Position::new(start.line, start.character + span.length as u32),
    )
}

type SymbolIdentity = (Url, crate::semantic::SymbolId);

fn symbol_identity_at(
    indexes: &HashMap<Url, DocumentIndex>,
    documents: &HashMap<Url, String>,
    uri: &Url,
    position: Position,
) -> Option<SymbolIdentity> {
    let index = indexes.get(uri)?;
    if let Some((_, name)) = index
        .reexport_references
        .iter()
        .find(|(span, _)| span_contains(*span, position))
    {
        if let Some(identity) = imported_name_identity(indexes, uri, index, name) {
            return Some(identity);
        }
    }
    if let Some(id) = semantic_symbol_at(index, position) {
        return canonical_identity(indexes, uri, id);
    }
    let source = documents.get(uri)?;
    if let Some(name) = word_at(source, position) {
        if let Some(identity) = glob_imported_identity(indexes, uri, index, &name) {
            return Some(identity);
        }
    }
    let (alias, member) = qualified_member_at(source, position)?;
    let module = index.module_aliases.get(&alias)?;
    exported_identity(indexes, uri, module, &member)
}

fn imported_name_identity(
    indexes: &HashMap<Url, DocumentIndex>,
    importer: &Url,
    index: &DocumentIndex,
    name: &str,
) -> Option<SymbolIdentity> {
    let selective = index
        .selective_imports
        .iter()
        .find_map(|(id, (module, imported))| {
            (imported == name
                && index
                    .semantic
                    .symbols
                    .get(*id)
                    .is_some_and(|symbol| symbol.name == name))
            .then(|| exported_identity(indexes, importer, module, imported))
            .flatten()
        });
    selective.or_else(|| glob_imported_identity(indexes, importer, index, name))
}

fn glob_imported_identity(
    indexes: &HashMap<Url, DocumentIndex>,
    importer: &Url,
    index: &DocumentIndex,
    name: &str,
) -> Option<SymbolIdentity> {
    let mut candidates = index
        .glob_imports
        .iter()
        .filter_map(|module| exported_identity(indexes, importer, module, name))
        .collect::<Vec<_>>();
    candidates.sort_by(|left, right| {
        left.0
            .as_str()
            .cmp(right.0.as_str())
            .then_with(|| left.1.cmp(&right.1))
    });
    candidates.dedup();
    (candidates.len() == 1).then(|| candidates.remove(0))
}

fn canonical_identity(
    indexes: &HashMap<Url, DocumentIndex>,
    uri: &Url,
    id: crate::semantic::SymbolId,
) -> Option<SymbolIdentity> {
    let index = indexes.get(uri)?;
    if let Some((module, imported_name)) = index.selective_imports.get(&id) {
        exported_identity(indexes, uri, module, imported_name)
    } else {
        Some((uri.clone(), id))
    }
}

fn exported_identity(
    indexes: &HashMap<Url, DocumentIndex>,
    importer: &Url,
    module: &str,
    name: &str,
) -> Option<SymbolIdentity> {
    if let Some(uri) = resolved_module_uri(importer, module) {
        if let Some(id) = indexes
            .get(&uri)
            .and_then(|index| index.exports.get(name))
            .copied()
        {
            return Some((uri, id));
        }
    }
    indexes.iter().find_map(|(uri, index)| {
        if module_matches(uri, module) {
            index.exports.get(name).copied().map(|id| (uri.clone(), id))
        } else {
            None
        }
    })
}

fn resolved_module_uri(importer: &Url, module: &str) -> Option<Url> {
    let importer_path = importer.to_file_path().ok()?;
    let resolved = crate::module_resolver::ModuleResolver::for_path(&importer_path)
        .resolve(module)
        .ok()?;
    match resolved.identity {
        crate::module_resolver::ModuleIdentity::File(path) => Url::from_file_path(path).ok(),
        crate::module_resolver::ModuleIdentity::Embedded(_) => None,
    }
}

fn dependent_documents(indexes: &HashMap<Url, DocumentIndex>, changed: &Url) -> Vec<Url> {
    let mut affected = HashSet::from([changed.clone()]);
    let mut found = true;
    while found {
        found = false;
        for (uri, index) in indexes {
            if affected.contains(uri) {
                continue;
            }
            let depends_on_affected = index.imports.iter().any(|module| {
                let resolved = resolved_module_uri(uri, module);
                affected.iter().any(|target| {
                    resolved.as_ref() == Some(target) || module_matches(target, module)
                })
            });
            if depends_on_affected {
                affected.insert(uri.clone());
                found = true;
            }
        }
    }
    affected.remove(changed);
    let mut dependents = affected.into_iter().collect::<Vec<_>>();
    dependents.sort_by(|left, right| left.as_str().cmp(right.as_str()));
    dependents
}

fn graph_documents(
    graph: &crate::module_graph::ModuleGraph,
    sources: &HashMap<PathBuf, String>,
) -> Vec<(Url, String)> {
    let mut documents = graph
        .nodes
        .keys()
        .filter_map(|identity| {
            let ject::module_resolver::ModuleIdentity::File(path) = identity else {
                return None;
            };
            let source = sources
                .get(path)
                .cloned()
                .or_else(|| fs::read_to_string(path).ok())?;
            Some((Url::from_file_path(path).ok()?, source))
        })
        .collect::<Vec<_>>();
    documents.sort_by(|left, right| left.0.as_str().cmp(right.0.as_str()));
    documents
}

fn definition_occurrence<'a>(
    indexes: &'a HashMap<Url, DocumentIndex>,
    identity: &SymbolIdentity,
) -> Option<(&'a Url, &'a Occurrence)> {
    let (definition_uri, index) = indexes.get_key_value(&identity.0)?;
    let symbol = index.semantic.symbols.get(identity.1)?;
    let range = range_from_span(symbol.span);
    index
        .occurrences
        .iter()
        .find(|occurrence| occurrence.definition && occurrence.range == range)
        .map(|occurrence| (definition_uri, occurrence))
}

fn module_matches(uri: &Url, module: &str) -> bool {
    let Ok(path) = uri.to_file_path() else {
        return false;
    };
    if path.file_stem().and_then(|stem| stem.to_str()) == Some(module) {
        return true;
    }
    crate::package::discover(&path).ok().is_some_and(|project| {
        project.name == module
            && fs::canonicalize(&project.entry).ok() == fs::canonicalize(&path).ok()
    })
}

fn locations_for_identity(
    indexes: &HashMap<Url, DocumentIndex>,
    documents: &HashMap<Url, String>,
    identity: &SymbolIdentity,
) -> Vec<(Location, bool)> {
    let mut result = Vec::new();
    for (uri, index) in indexes {
        for symbol in &index.semantic.symbols {
            if canonical_identity(indexes, uri, symbol.id).as_ref() == Some(identity) {
                let declaration = uri == &identity.0 && symbol.id == identity.1;
                result.push((
                    Location::new(uri.clone(), range_from_span(symbol.span)),
                    declaration,
                ));
            }
        }
        for reference in &index.semantic.references {
            let reexport = index
                .reexport_references
                .iter()
                .find(|(span, _)| *span == reference.span)
                .and_then(|(_, name)| imported_name_identity(indexes, uri, index, name));
            let resolved = reexport.or_else(|| {
                reference
                    .resolved
                    .and_then(|id| canonical_identity(indexes, uri, id))
                    .or_else(|| glob_imported_identity(indexes, uri, index, &reference.name))
            });
            if resolved.as_ref() == Some(identity) {
                result.push((
                    Location::new(uri.clone(), range_from_span(reference.span)),
                    false,
                ));
            }
        }
        for occurrence in index
            .occurrences
            .iter()
            .filter(|item| item.kind == SymbolKind::FIELD)
        {
            if symbol_identity_at(indexes, documents, uri, occurrence.range.start).as_ref()
                == Some(identity)
            {
                result.push((Location::new(uri.clone(), occurrence.range), false));
            }
        }
    }
    result.sort_by(|left, right| {
        left.0
            .uri
            .as_str()
            .cmp(right.0.uri.as_str())
            .then_with(|| left.0.range.start.line.cmp(&right.0.range.start.line))
            .then_with(|| {
                left.0
                    .range
                    .start
                    .character
                    .cmp(&right.0.range.start.character)
            })
    });
    result.dedup_by(|left, right| left.0 == right.0);
    result
}

fn rename_conflict(
    indexes: &HashMap<Url, DocumentIndex>,
    documents: &HashMap<Url, String>,
    identity: &SymbolIdentity,
    new_name: &str,
) -> Option<String> {
    let definition = indexes.get(&identity.0)?.semantic.symbols.get(identity.1)?;
    if definition.name == new_name {
        return None;
    }
    if let Some(existing) = indexes
        .get(&identity.0)
        .and_then(|index| index.exports.get(new_name))
    {
        if existing != &identity.1 {
            return Some(format!(
                "cannot rename `{}` to `{new_name}`: the module already exports `{new_name}`",
                definition.name
            ));
        }
    }

    for (location, _) in locations_for_identity(indexes, documents, identity) {
        if documents
            .get(&location.uri)
            .and_then(|source| qualified_member_at(source, location.range.start))
            .is_some()
        {
            continue;
        }
        let Some(index) = indexes.get(&location.uri) else {
            continue;
        };
        let line = location.range.start.line as usize + 1;
        let column = location.range.start.character as usize + 1;
        for candidate in index.semantic.visible_symbols(line, column) {
            let Some(symbol) = index.semantic.symbols.get(candidate) else {
                continue;
            };
            if symbol.name == new_name
                && canonical_identity(indexes, &location.uri, candidate).as_ref() != Some(identity)
            {
                let path = location
                    .uri
                    .to_file_path()
                    .ok()
                    .map(|path| path.display().to_string())
                    .unwrap_or_else(|| location.uri.to_string());
                return Some(format!(
                    "cannot rename `{}` to `{new_name}`: `{new_name}` is already visible at {path}:{line}",
                    definition.name
                ));
            }
        }
    }
    None
}

fn document_highlights_for_identity(
    indexes: &HashMap<Url, DocumentIndex>,
    documents: &HashMap<Url, String>,
    uri: &Url,
    identity: &SymbolIdentity,
) -> Vec<DocumentHighlight> {
    let current_index = indexes.get(uri);
    locations_for_identity(indexes, documents, identity)
        .into_iter()
        .filter(|(location, _)| &location.uri == uri)
        .map(|(location, _)| DocumentHighlight {
            kind: Some(
                if current_index.is_some_and(|index| {
                    index
                        .semantic
                        .symbols
                        .iter()
                        .any(|symbol| range_from_span(symbol.span) == location.range)
                }) {
                    DocumentHighlightKind::WRITE
                } else {
                    DocumentHighlightKind::READ
                },
            ),
            range: location.range,
        })
        .collect()
}

fn qualified_member_at(source: &str, position: Position) -> Option<(String, String)> {
    let line = source.lines().nth(position.line as usize)?;
    let cursor = (position.character as usize).min(line.len());
    let bytes = line.as_bytes();
    let mut member_start = cursor;
    while member_start > 0
        && (bytes[member_start - 1].is_ascii_alphanumeric() || bytes[member_start - 1] == b'_')
    {
        member_start -= 1;
    }
    let mut member_end = cursor;
    while member_end < bytes.len()
        && (bytes[member_end].is_ascii_alphanumeric() || bytes[member_end] == b'_')
    {
        member_end += 1;
    }
    if member_start == 0 || bytes.get(member_start - 1) != Some(&b'.') {
        return None;
    }
    let mut alias_start = member_start - 1;
    while alias_start > 0
        && (bytes[alias_start - 1].is_ascii_alphanumeric() || bytes[alias_start - 1] == b'_')
    {
        alias_start -= 1;
    }
    if alias_start == member_start - 1 {
        return None;
    }
    Some((
        line[alias_start..member_start - 1].into(),
        line[member_start..member_end].into(),
    ))
}

fn contains(range: Range, position: Position) -> bool {
    position.line == range.start.line
        && position.character >= range.start.character
        && position.character <= range.end.character
}

fn span_contains(span: SourceSpan, position: Position) -> bool {
    position.line as usize + 1 == span.line
        && position.character as usize + 1 >= span.column
        && (position.character as usize) < span.column + span.length
}

fn valid_identifier(name: &str) -> bool {
    let mut chars = name.chars();
    chars.next().is_some_and(|c| c == '_' || c.is_alphabetic())
        && chars.all(|c| c == '_' || c.is_alphanumeric())
        && !KEYWORDS.contains(&name)
}

fn signature_help_at(
    indexes: &HashMap<Url, DocumentIndex>,
    documents: &HashMap<Url, String>,
    uri: &Url,
    position: Position,
) -> Option<SignatureHelp> {
    let source = documents.get(uri)?;
    let (callee, callee_position, active_parameter) = call_context(source, position)?;
    let label = if !callee.contains('.') && BUILTINS.contains(&callee.as_str()) {
        builtin_signature(&callee).to_owned()
    } else {
        let identity = symbol_identity_at(indexes, documents, uri, callee_position)?;
        definition_occurrence(indexes, &identity)?
            .1
            .detail
            .clone()?
    };
    let parameters = signature_parameters(&label)
        .into_iter()
        .map(|parameter| ParameterInformation {
            label: ParameterLabel::Simple(parameter),
            documentation: None,
        })
        .collect::<Vec<_>>();
    let active_parameter = active_parameter.min(parameters.len().saturating_sub(1)) as u32;
    Some(SignatureHelp {
        signatures: vec![SignatureInformation {
            label,
            documentation: None,
            parameters: Some(parameters),
            active_parameter: Some(active_parameter),
        }],
        active_signature: Some(0),
        active_parameter: Some(active_parameter),
    })
}

fn call_context(source: &str, position: Position) -> Option<(String, Position, usize)> {
    let cursor = byte_offset(source, position)?;
    let bytes = source.as_bytes();
    let mut delimiters = Vec::new();
    let mut quoted = false;
    let mut escaped = false;
    for (index, byte) in bytes.iter().copied().take(cursor).enumerate() {
        if quoted {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                quoted = false;
            }
            continue;
        }
        match byte {
            b'"' => quoted = true,
            b'(' | b'[' | b'{' => delimiters.push((byte, index)),
            b')' | b']' | b'}' => {
                delimiters.pop();
            }
            _ => {}
        }
    }
    let open = delimiters
        .iter()
        .rev()
        .find_map(|(delimiter, index)| (*delimiter == b'(').then_some(*index))?;
    let mut end = open;
    while end > 0 && bytes[end - 1].is_ascii_whitespace() {
        end -= 1;
    }
    let mut start = end;
    while start > 0
        && (bytes[start - 1].is_ascii_alphanumeric() || matches!(bytes[start - 1], b'_' | b'.'))
    {
        start -= 1;
    }
    if start == end {
        return None;
    }
    let callee = source[start..end].to_owned();
    let member_start = callee.rfind('.').map_or(start, |dot| start + dot + 1);
    let active_parameter = top_level_commas(&source[open + 1..cursor]);
    Some((
        callee,
        position_from_offset(source, member_start),
        active_parameter,
    ))
}

fn top_level_commas(arguments: &str) -> usize {
    let mut depth = 0usize;
    let mut quoted = false;
    let mut escaped = false;
    let mut commas = 0usize;
    for byte in arguments.bytes() {
        if quoted {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                quoted = false;
            }
            continue;
        }
        match byte {
            b'"' => quoted = true,
            b'(' | b'[' | b'{' => depth += 1,
            b')' | b']' | b'}' => depth = depth.saturating_sub(1),
            b',' if depth == 0 => commas += 1,
            _ => {}
        }
    }
    commas
}

fn signature_parameters(signature: &str) -> Vec<String> {
    let Some(start) = signature.find('(') else {
        return Vec::new();
    };
    let Some(end) = matching_parenthesis(signature, start) else {
        return Vec::new();
    };
    split_top_level(&signature[start + 1..end])
        .into_iter()
        .map(str::trim)
        .filter(|parameter| !parameter.is_empty())
        .map(str::to_owned)
        .collect()
}

fn matching_parenthesis(value: &str, open: usize) -> Option<usize> {
    let mut depth = 0usize;
    let mut quoted = false;
    let mut escaped = false;
    for (offset, character) in value[open..].char_indices() {
        if quoted {
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == '"' {
                quoted = false;
            }
            continue;
        }
        match character {
            '"' => quoted = true,
            '(' => depth += 1,
            ')' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some(open + offset);
                }
            }
            _ => {}
        }
    }
    None
}

fn split_top_level(value: &str) -> Vec<&str> {
    let mut result = Vec::new();
    let mut start = 0usize;
    let mut depth = 0usize;
    let mut quoted = false;
    let mut escaped = false;
    for (index, character) in value.char_indices() {
        if quoted {
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == '"' {
                quoted = false;
            }
            continue;
        }
        match character {
            '"' => quoted = true,
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => {
                result.push(&value[start..index]);
                start = index + character.len_utf8();
            }
            _ => {}
        }
    }
    result.push(&value[start..]);
    result
}

fn byte_offset(source: &str, position: Position) -> Option<usize> {
    let mut offset = 0usize;
    for (line_number, line) in source.split_inclusive('\n').enumerate() {
        if line_number == position.line as usize {
            return Some(
                offset + (position.character as usize).min(line.trim_end_matches('\n').len()),
            );
        }
        offset += line.len();
    }
    (position.line as usize == source.lines().count()).then_some(source.len())
}

fn position_from_offset(source: &str, offset: usize) -> Position {
    let prefix = &source[..offset.min(source.len())];
    let line = prefix.bytes().filter(|byte| *byte == b'\n').count() as u32;
    let character = prefix
        .rsplit_once('\n')
        .map_or(prefix.len(), |(_, tail)| tail.len()) as u32;
    Position::new(line, character)
}

fn completion_items(
    indexes: &HashMap<Url, DocumentIndex>,
    documents: &HashMap<Url, String>,
    uri: &Url,
    position: Position,
) -> Vec<CompletionItem> {
    let Some(source) = documents.get(uri) else {
        return Vec::new();
    };
    let line = source
        .lines()
        .nth(position.line as usize)
        .unwrap_or_default();
    let cursor = (position.character as usize).min(line.len());
    let prefix = &line[..cursor];

    if import_string_context(prefix) {
        let mut modules = BTreeMap::new();
        for (module_uri, index) in indexes {
            if !index.exports.is_empty() {
                if let Some(name) = module_name(module_uri) {
                    modules.insert(name, ());
                }
            }
        }
        return modules
            .into_keys()
            .map(|label| CompletionItem {
                label,
                kind: Some(CompletionItemKind::MODULE),
                detail: Some("Ject module".into()),
                ..CompletionItem::default()
            })
            .collect();
    }

    if let Some(alias) = qualifier_before_cursor(prefix) {
        let Some(index) = indexes.get(uri) else {
            return Vec::new();
        };
        let Some(module) = index.module_aliases.get(alias) else {
            return Vec::new();
        };
        return exported_completion_items(indexes, uri, module);
    }

    if let Some(module) = selective_import_module(line, cursor) {
        return exported_completion_items(indexes, uri, module);
    }

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
    let Some(index) = indexes.get(uri) else {
        return items;
    };
    let mut names = BTreeMap::new();
    let mut glob_names: BTreeMap<String, (SymbolKind, Option<String>, usize)> = BTreeMap::new();
    for module in &index.glob_imports {
        for item in exported_completion_items(indexes, uri, module) {
            let kind = match item.kind {
                Some(CompletionItemKind::FUNCTION) => SymbolKind::FUNCTION,
                Some(CompletionItemKind::STRUCT) => SymbolKind::STRUCT,
                Some(CompletionItemKind::MODULE) => SymbolKind::MODULE,
                _ => SymbolKind::VARIABLE,
            };
            glob_names
                .entry(item.label)
                .and_modify(|entry| entry.2 += 1)
                .or_insert((kind, item.detail, 1));
        }
    }
    for (name, (kind, detail, providers)) in glob_names {
        if providers == 1 {
            names.insert(name, (kind, detail));
        }
    }
    for id in index
        .semantic
        .visible_symbols(position.line as usize + 1, position.character as usize + 1)
    {
        let Some(symbol) = index.semantic.symbols.get(id) else {
            continue;
        };
        let local_occurrence = index.occurrences.iter().find(|occurrence| {
            occurrence.definition && occurrence.range == range_from_span(symbol.span)
        });
        let imported_occurrence = index
            .selective_imports
            .get(&id)
            .and_then(|(module, imported_name)| {
                exported_identity(indexes, uri, module, imported_name)
            })
            .and_then(|identity| definition_occurrence(indexes, &identity))
            .map(|(_, occurrence)| occurrence);
        let occurrence = imported_occurrence.or(local_occurrence);
        let kind = occurrence
            .map(|item| item.kind)
            .unwrap_or_else(|| semantic_completion_kind(symbol.kind));
        let detail = occurrence.and_then(|item| item.detail.clone()).or_else(|| {
            (symbol.kind == crate::semantic::SymbolKind::Module).then(|| "Ject module".into())
        });
        names.insert(symbol.name.clone(), (kind, detail));
    }
    items.extend(
        names
            .into_iter()
            .map(|(label, (kind, detail))| CompletionItem {
                label,
                kind: Some(completion_kind(kind)),
                detail,
                ..CompletionItem::default()
            }),
    );
    items
}

fn semantic_completion_kind(kind: crate::semantic::SymbolKind) -> SymbolKind {
    match kind {
        crate::semantic::SymbolKind::Function => SymbolKind::FUNCTION,
        crate::semantic::SymbolKind::Struct => SymbolKind::STRUCT,
        crate::semantic::SymbolKind::Module => SymbolKind::MODULE,
        _ => SymbolKind::VARIABLE,
    }
}

fn exported_completion_items(
    indexes: &HashMap<Url, DocumentIndex>,
    importer: &Url,
    module: &str,
) -> Vec<CompletionItem> {
    let mut items = BTreeMap::new();
    for (uri, index) in indexes {
        let resolved = resolved_module_uri(importer, module);
        if resolved.as_ref().is_some_and(|target| target != uri)
            || (resolved.is_none() && !module_matches(uri, module))
        {
            continue;
        }
        for (name, id) in &index.exports {
            let identity = (uri.clone(), *id);
            let occurrence = definition_occurrence(indexes, &identity);
            items.insert(
                name.clone(),
                CompletionItem {
                    label: name.clone(),
                    kind: Some(
                        occurrence
                            .map(|(_, item)| completion_kind(item.kind))
                            .unwrap_or(CompletionItemKind::VARIABLE),
                    ),
                    detail: occurrence.and_then(|(_, item)| item.detail.clone()),
                    ..CompletionItem::default()
                },
            );
        }
    }
    if items.is_empty() {
        let interface = importer
            .to_file_path()
            .ok()
            .and_then(|path| {
                crate::module_resolver::ModuleResolver::for_path(&path)
                    .resolve(module)
                    .ok()
            })
            .and_then(|resolved| ModuleInterface::parse(&resolved.source).ok());
        if let Some(interface) = interface {
            for export in interface.exports {
                let (kind, detail) = match export.kind {
                    ExportKind::Value => {
                        (CompletionItemKind::VARIABLE, Some("exported value".into()))
                    }
                    ExportKind::Function { parameters } => {
                        let parameters = parameters
                            .iter()
                            .map(|parameter| {
                                if parameter.default_value.is_some() {
                                    format!("{}=…", parameter.name)
                                } else {
                                    parameter.name.clone()
                                }
                            })
                            .collect::<Vec<_>>()
                            .join(", ");
                        (
                            CompletionItemKind::FUNCTION,
                            Some(format!("fn {}({parameters})", export.name)),
                        )
                    }
                };
                items.insert(
                    export.name.clone(),
                    CompletionItem {
                        label: export.name,
                        kind: Some(kind),
                        detail,
                        ..CompletionItem::default()
                    },
                );
            }
        }
    }
    items.into_values().collect()
}

fn qualifier_before_cursor(prefix: &str) -> Option<&str> {
    let without_member = prefix.trim_end_matches(|c: char| c == '_' || c.is_alphanumeric());
    let before_dot = without_member.strip_suffix('.')?;
    let start = before_dot
        .rfind(|c: char| !(c == '_' || c.is_alphanumeric()))
        .map_or(0, |index| index + 1);
    (start < before_dot.len()).then(|| &before_dot[start..])
}

fn import_string_context(prefix: &str) -> bool {
    let Some(quote) = prefix.rfind('"') else {
        return false;
    };
    let before = prefix[..quote].trim_end();
    (before.ends_with("import") || before.ends_with("from")) && !prefix[quote + 1..].contains('"')
}

fn selective_import_module(line: &str, cursor: usize) -> Option<&str> {
    let before = &line[..cursor];
    if !before.contains("import {") || before.rfind('}').is_some() {
        return None;
    }
    let after = &line[cursor..];
    let from = after.find("from")?;
    let quoted = &after[from + 4..];
    let start = quoted.find('"')? + 1;
    let end = quoted[start..].find('"')? + start;
    Some(&quoted[start..end])
}

fn module_name(uri: &Url) -> Option<String> {
    let path = uri.to_file_path().ok()?;
    if let Ok(project) = crate::package::discover(&path) {
        if fs::canonicalize(&project.entry).ok() == fs::canonicalize(&path).ok() {
            return Some(project.name);
        }
    }
    path.file_stem()?.to_str().map(str::to_owned)
}

fn symbol_container(uri: &Url) -> Option<String> {
    let path = uri.to_file_path().ok()?;
    let project = crate::package::discover(&path).ok()?;
    if fs::canonicalize(&project.entry).ok() == fs::canonicalize(&path).ok() {
        return Some(project.name);
    }
    let relative = path.strip_prefix(&project.root).ok()?;
    let relative = relative.strip_prefix("src").unwrap_or(relative);
    let mut parts = relative
        .components()
        .filter_map(|component| component.as_os_str().to_str())
        .map(str::to_owned)
        .collect::<Vec<_>>();
    if let Some(last) = parts.last_mut() {
        if let Some(stem) = Path::new(last).file_stem().and_then(|stem| stem.to_str()) {
            *last = stem.to_string();
        }
    }
    Some(if parts.is_empty() {
        project.name
    } else {
        format!("{}::{}", project.name, parts.join("::"))
    })
}

fn completion_kind(kind: SymbolKind) -> CompletionItemKind {
    if kind == SymbolKind::FUNCTION {
        CompletionItemKind::FUNCTION
    } else if kind == SymbolKind::STRUCT {
        CompletionItemKind::STRUCT
    } else if kind == SymbolKind::MODULE {
        CompletionItemKind::MODULE
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
            graph_cache: RwLock::new(HashMap::new()),
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
    fn reports_unresolved_imports_with_the_import_error_code() {
        let path = std::env::temp_dir().join(format!(
            "ject-lsp-missing-import-{}-{:?}.ject",
            std::process::id(),
            std::thread::current().id()
        ));
        let diagnostics = analyze_at("import \"./not_here\" as nope", &path);
        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.code == Some(NumberOrString::String("E3101".into()))
                && diagnostic.range.start.line == 0
        }));
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
            && item.detail.as_deref() == Some("fn greet(name, prefix=make_prefix())")));
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
    fn propagates_types_through_aliases_and_expressions() {
        let index = index_document(
            "let x = 2\nlet a = x\nlet precise = 2.0\nlet total = precise + x\nlet ratio = x / 2\nlet check = total > 1\nlet text = to_string(a)\nlet items = range(3)",
        );
        let details = index
            .occurrences
            .iter()
            .filter(|item| item.definition)
            .filter_map(|item| {
                item.detail
                    .as_deref()
                    .map(|detail| (item.name.as_str(), detail))
            })
            .collect::<HashMap<_, _>>();
        assert_eq!(details["x"], "let x: integer");
        assert_eq!(details["a"], "let a: integer");
        assert_eq!(details["precise"], "let precise: float");
        assert_eq!(details["total"], "let total: float");
        assert_eq!(details["ratio"], "let ratio: float");
        assert_eq!(details["check"], "let check: boolean");
        assert_eq!(details["text"], "let text: string");
        assert_eq!(details["items"], "let items: array");
    }

    #[test]
    fn lsp_index_keeps_shadowed_renames_separate() {
        let index = index_document(
            "let value = 1\nif true\n    let value = 2\n    print value\nend\nprint value",
        );
        let inner = index.semantic.symbol_at(4, 11).unwrap();
        let outer = index.semantic.symbol_at(6, 7).unwrap();
        assert_ne!(inner, outer);
        assert_eq!(index.semantic.spans_for(inner).len(), 2);
        assert_eq!(index.semantic.spans_for(outer).len(), 2);
    }

    #[test]
    fn document_highlights_respect_shadowing() {
        let uri = Url::parse("file:///tmp/app.ject").unwrap();
        let source = "let value = 1\nif true\n    let value = 2\n    print value\nend\nprint value";
        let indexes = HashMap::from([(uri.clone(), index_document(source))]);
        let documents = HashMap::from([(uri.clone(), source.into())]);
        let identity =
            symbol_identity_at(&indexes, &documents, &uri, Position::new(3, 10)).unwrap();
        let highlights = document_highlights_for_identity(&indexes, &documents, &uri, &identity);
        assert_eq!(highlights.len(), 2);
        assert_eq!(
            highlights
                .iter()
                .filter(|highlight| highlight.kind == Some(DocumentHighlightKind::WRITE))
                .count(),
            1
        );
        assert!(highlights
            .iter()
            .all(|highlight| matches!(highlight.range.start.line, 2 | 3)));
    }

    #[test]
    fn resolves_selective_imports_to_exported_identity() {
        let library_uri = Url::parse("file:///tmp/colors.ject").unwrap();
        let app_uri = Url::parse("file:///tmp/app.ject").unwrap();
        let library = "export fn paint(text)\n    return text\nend";
        let app = "import {paint} from \"colors\"\npaint(\"hi\")";
        let indexes = HashMap::from([
            (library_uri.clone(), index_document(library)),
            (app_uri.clone(), index_document(app)),
        ]);
        let documents = HashMap::from([
            (library_uri.clone(), library.into()),
            (app_uri.clone(), app.into()),
        ]);
        let identity =
            symbol_identity_at(&indexes, &documents, &app_uri, Position::new(1, 1)).unwrap();
        assert_eq!(identity.0, library_uri);
        assert_eq!(
            locations_for_identity(&indexes, &documents, &identity).len(),
            3
        );
        let highlights =
            document_highlights_for_identity(&indexes, &documents, &app_uri, &identity);
        assert_eq!(highlights.len(), 2);
        assert_eq!(highlights[0].kind, Some(DocumentHighlightKind::WRITE));
        assert_eq!(highlights[1].kind, Some(DocumentHighlightKind::READ));
    }

    #[test]
    fn resolves_qualified_module_members() {
        let library_uri = Url::parse("file:///tmp/colors.ject").unwrap();
        let app_uri = Url::parse("file:///tmp/app.ject").unwrap();
        let library = "export fn paint(text)\n    return text\nend";
        let app = "import \"colors\" as c\nc.paint(\"hi\")";
        let indexes = HashMap::from([
            (library_uri.clone(), index_document(library)),
            (app_uri.clone(), index_document(app)),
        ]);
        let documents = HashMap::from([
            (library_uri.clone(), library.into()),
            (app_uri.clone(), app.into()),
        ]);
        let identity =
            symbol_identity_at(&indexes, &documents, &app_uri, Position::new(1, 3)).unwrap();
        assert_eq!(identity.0, library_uri);
        assert_eq!(
            locations_for_identity(&indexes, &documents, &identity).len(),
            2
        );
    }

    #[test]
    fn resolves_unaliased_import_names_to_their_export() {
        let library_uri = Url::parse("file:///tmp/colors.ject").unwrap();
        let app_uri = Url::parse("file:///tmp/app.ject").unwrap();
        let library = "export fn paint(text)\n    return text\nend";
        let app = "import \"colors\"\npaint(\"hi\")";
        let indexes = HashMap::from([
            (library_uri.clone(), index_document(library)),
            (app_uri.clone(), index_document(app)),
        ]);
        let documents = HashMap::from([
            (library_uri.clone(), library.into()),
            (app_uri.clone(), app.into()),
        ]);

        let identity =
            symbol_identity_at(&indexes, &documents, &app_uri, Position::new(1, 2)).unwrap();
        assert_eq!(identity.0, library_uri);
        assert_eq!(
            locations_for_identity(&indexes, &documents, &identity).len(),
            2
        );
        let completions = completion_items(&indexes, &documents, &app_uri, Position::new(1, 2));
        let paint = completions
            .iter()
            .find(|item| item.label == "paint")
            .unwrap();
        assert_eq!(paint.kind, Some(CompletionItemKind::FUNCTION));
        assert_eq!(paint.detail.as_deref(), Some("fn paint(text)"));
    }

    #[test]
    fn unaliased_import_names_stay_ambiguous_when_modules_collide() {
        let colors_uri = Url::parse("file:///tmp/colors.ject").unwrap();
        let canvas_uri = Url::parse("file:///tmp/canvas.ject").unwrap();
        let app_uri = Url::parse("file:///tmp/app.ject").unwrap();
        let library = "export fn paint()\nend";
        let app = "import \"colors\"\nimport \"canvas\"\npaint()";
        let indexes = HashMap::from([
            (colors_uri.clone(), index_document(library)),
            (canvas_uri, index_document(library)),
            (app_uri.clone(), index_document(app)),
        ]);
        let documents =
            HashMap::from([(colors_uri, library.into()), (app_uri.clone(), app.into())]);

        assert!(symbol_identity_at(&indexes, &documents, &app_uri, Position::new(2, 2)).is_none());

        let completions = completion_items(&indexes, &documents, &app_uri, Position::new(2, 5));
        assert!(!completions.iter().any(|item| item.label == "paint"));
    }

    #[test]
    fn same_named_reexports_keep_source_and_public_identities_separate() {
        let base_uri = Url::parse("file:///tmp/base.ject").unwrap();
        let facade_uri = Url::parse("file:///tmp/facade.ject").unwrap();
        let app_uri = Url::parse("file:///tmp/app.ject").unwrap();
        let base = "export fn paint()\nend";
        let facade = "import {paint} from \"base\"\nexport paint = paint";
        let app = "import {paint} from \"facade\"\npaint()";
        let indexes = HashMap::from([
            (base_uri.clone(), index_document(base)),
            (facade_uri.clone(), index_document(facade)),
            (app_uri.clone(), index_document(app)),
        ]);
        let documents = HashMap::from([
            (base_uri.clone(), base.into()),
            (facade_uri.clone(), facade.into()),
            (app_uri.clone(), app.into()),
        ]);

        let source_identity =
            symbol_identity_at(&indexes, &documents, &facade_uri, Position::new(1, 16)).unwrap();
        let public_identity =
            symbol_identity_at(&indexes, &documents, &facade_uri, Position::new(1, 8)).unwrap();
        assert_eq!(source_identity.0, base_uri);
        assert_eq!(public_identity.0, facade_uri);
        assert_ne!(source_identity, public_identity);
        assert_eq!(
            locations_for_identity(&indexes, &documents, &source_identity).len(),
            3
        );
        assert_eq!(
            locations_for_identity(&indexes, &documents, &public_identity).len(),
            3
        );
        assert!(
            locations_for_identity(&indexes, &documents, &public_identity)
                .iter()
                .all(|(location, _)| !(location.uri == facade_uri
                    && location.range.start.character == 15))
        );
    }

    #[test]
    fn imported_identity_ignores_same_named_exports() {
        let colors_uri = Url::parse("file:///tmp/colors.ject").unwrap();
        let drawing_uri = Url::parse("file:///tmp/drawing.ject").unwrap();
        let app_uri = Url::parse("file:///tmp/app.ject").unwrap();
        let colors = "export fn paint(text)\n    return text\nend";
        let drawing = "export fn paint(canvas)\n    return canvas\nend";
        let app = "import {paint} from \"colors\"\npaint(\"hi\")";
        let indexes = HashMap::from([
            (colors_uri.clone(), index_document(colors)),
            (drawing_uri.clone(), index_document(drawing)),
            (app_uri.clone(), index_document(app)),
        ]);
        let documents = HashMap::from([
            (colors_uri.clone(), colors.into()),
            (drawing_uri, drawing.into()),
            (app_uri.clone(), app.into()),
        ]);
        let identity =
            symbol_identity_at(&indexes, &documents, &app_uri, Position::new(1, 1)).unwrap();
        assert_eq!(identity.0, colors_uri);
        assert_eq!(
            locations_for_identity(&indexes, &documents, &identity).len(),
            3
        );
        let (definition_uri, occurrence) = definition_occurrence(&indexes, &identity).unwrap();
        assert_eq!(definition_uri, &colors_uri);
        assert_eq!(occurrence.detail.as_deref(), Some("fn paint(text)"));
    }

    #[test]
    fn rename_rejects_export_and_visible_scope_collisions() {
        let library_uri = Url::parse("file:///tmp/colors.ject").unwrap();
        let app_uri = Url::parse("file:///tmp/app.ject").unwrap();
        let library = "export fn paint()\nend\nexport fn draw()\nend";
        let app = "import {paint} from \"colors\"\nlet replacement = 1\npaint()";
        let indexes = HashMap::from([
            (library_uri.clone(), index_document(library)),
            (app_uri.clone(), index_document(app)),
        ]);
        let documents = HashMap::from([
            (library_uri.clone(), library.into()),
            (app_uri.clone(), app.into()),
        ]);
        let identity =
            symbol_identity_at(&indexes, &documents, &app_uri, Position::new(2, 1)).unwrap();

        assert!(rename_conflict(&indexes, &documents, &identity, "draw")
            .unwrap()
            .contains("already exports"));
        assert!(
            rename_conflict(&indexes, &documents, &identity, "replacement")
                .unwrap()
                .contains("already visible")
        );
        assert!(rename_conflict(&indexes, &documents, &identity, "render").is_none());
    }

    #[test]
    fn qualified_export_rename_ignores_unrelated_local_names() {
        let library_uri = Url::parse("file:///tmp/colors.ject").unwrap();
        let app_uri = Url::parse("file:///tmp/app.ject").unwrap();
        let library = "export fn paint()\nend";
        let app = "import \"colors\" as colors\nlet render = 1\ncolors.paint()";
        let indexes = HashMap::from([
            (library_uri.clone(), index_document(library)),
            (app_uri.clone(), index_document(app)),
        ]);
        let documents =
            HashMap::from([(library_uri, library.into()), (app_uri.clone(), app.into())]);
        let identity =
            symbol_identity_at(&indexes, &documents, &app_uri, Position::new(2, 9)).unwrap();

        assert!(rename_conflict(&indexes, &documents, &identity, "render").is_none());
    }

    #[test]
    fn workspace_symbols_use_package_qualified_containers() {
        let root =
            std::env::temp_dir().join(format!("ject-symbol-container-{}", std::process::id()));
        let module_path = root.join("src/tools/color.ject");
        std::fs::create_dir_all(module_path.parent().unwrap()).unwrap();
        std::fs::write(
            root.join("Ject.toml"),
            "[package]\nname = \"palette\"\nentry = \"src/main.ject\"\n",
        )
        .unwrap();
        std::fs::write(root.join("src/main.ject"), "print 1\n").unwrap();
        std::fs::write(&module_path, "export red = 1\n").unwrap();
        let entry_uri = Url::from_file_path(root.join("src/main.ject")).unwrap();
        let module_uri = Url::from_file_path(&module_path).unwrap();

        assert_eq!(symbol_container(&entry_uri).as_deref(), Some("palette"));
        assert_eq!(
            symbol_container(&module_uri).as_deref(),
            Some("palette::tools::color")
        );
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn imported_identity_uses_the_importers_real_relative_path() {
        let root = std::env::temp_dir().join(format!(
            "ject-lsp-resolver-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let app_dir = root.join("app");
        let other_dir = root.join("other");
        std::fs::create_dir_all(&app_dir).unwrap();
        std::fs::create_dir_all(&other_dir).unwrap();
        let app_path = app_dir.join("main.ject");
        let expected_path = app_dir.join("colors.ject");
        let unrelated_path = other_dir.join("colors.ject");
        let app = "import {paint} from \"./colors\"\npaint(\"hi\")";
        let expected = "export fn paint(text)\nend";
        let unrelated = "export fn paint(canvas, brush)\nend";
        std::fs::write(&app_path, app).unwrap();
        std::fs::write(&expected_path, expected).unwrap();
        std::fs::write(&unrelated_path, unrelated).unwrap();
        let app_uri = Url::from_file_path(app_path.canonicalize().unwrap()).unwrap();
        let expected_uri = Url::from_file_path(expected_path.canonicalize().unwrap()).unwrap();
        let unrelated_uri = Url::from_file_path(unrelated_path.canonicalize().unwrap()).unwrap();
        let indexes = HashMap::from([
            (app_uri.clone(), index_document(app)),
            (expected_uri.clone(), index_document(expected)),
            (unrelated_uri, index_document(unrelated)),
        ]);
        let documents = HashMap::from([(app_uri.clone(), app.into())]);

        let identity =
            symbol_identity_at(&indexes, &documents, &app_uri, Position::new(1, 1)).unwrap();
        assert_eq!(identity.0, expected_uri);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn completion_respects_module_and_import_boundaries() {
        let colors_uri = Url::parse("file:///tmp/colors.ject").unwrap();
        let drawing_uri = Url::parse("file:///tmp/drawing.ject").unwrap();
        let app_uri = Url::parse("file:///tmp/app.ject").unwrap();
        let colors = "export fn paint(text)\n    return text\nend\nfn private_helper()\nend";
        let drawing = "export fn sketch(canvas)\n    return canvas\nend";
        let app = "import \"colors\" as c\nlet local = 1\nc.pa";
        let indexes = HashMap::from([
            (colors_uri.clone(), index_document(colors)),
            (drawing_uri.clone(), index_document(drawing)),
            (app_uri.clone(), index_document(app)),
        ]);
        let documents = HashMap::from([
            (colors_uri, colors.into()),
            (drawing_uri, drawing.into()),
            (app_uri.clone(), app.into()),
        ]);

        let qualified = completion_items(&indexes, &documents, &app_uri, Position::new(2, 4));
        assert_eq!(
            qualified
                .iter()
                .map(|item| item.label.as_str())
                .collect::<Vec<_>>(),
            vec!["paint"]
        );
        assert_eq!(qualified[0].detail.as_deref(), Some("fn paint(text)"));

        let general = completion_items(&indexes, &documents, &app_uri, Position::new(1, 9));
        let labels = general
            .iter()
            .map(|item| item.label.as_str())
            .collect::<Vec<_>>();
        assert!(labels.contains(&"local"));
        assert!(labels.contains(&"c"));
        assert!(!labels.contains(&"sketch"));
        assert!(!labels.contains(&"private_helper"));
    }

    #[test]
    fn completion_only_offers_exports_in_selective_imports() {
        let colors_uri = Url::parse("file:///tmp/colors.ject").unwrap();
        let app_uri = Url::parse("file:///tmp/app.ject").unwrap();
        let colors = "export fn paint(text)\nend\nfn private_helper()\nend";
        let app = "import {pa} from \"colors\"";
        let indexes = HashMap::from([
            (colors_uri.clone(), index_document(colors)),
            (app_uri.clone(), index_document(app)),
        ]);
        let documents = HashMap::from([(colors_uri, colors.into()), (app_uri.clone(), app.into())]);
        let items = completion_items(&indexes, &documents, &app_uri, Position::new(0, 10));
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].label, "paint");
    }

    #[test]
    fn completion_loads_resolved_modules_outside_the_workspace_index() {
        let root = std::env::temp_dir().join(format!("ject-lsp-completion-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let app_path = root.join("app.ject");
        let dependency_path = root.join("colors.ject");
        let app = "import \"./colors.ject\" as colors\ncolors.pa";
        std::fs::write(&app_path, app).unwrap();
        std::fs::write(
            &dependency_path,
            "export fn paint(text, shade=\"blue\")\nend\nlet private = 1",
        )
        .unwrap();
        let app_uri = Url::from_file_path(&app_path).unwrap();
        let indexes = HashMap::from([(app_uri.clone(), index_document(app))]);
        let documents = HashMap::from([(app_uri.clone(), app.into())]);

        let items = completion_items(&indexes, &documents, &app_uri, Position::new(1, 9));
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].label, "paint");
        assert_eq!(items[0].kind, Some(CompletionItemKind::FUNCTION));
        assert_eq!(items[0].detail.as_deref(), Some("fn paint(text, shade=…)"));
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn diagnostics_report_source_import_cycles() {
        let root = std::env::temp_dir().join(format!("ject-lsp-cycle-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let app_path = root.join("app.ject");
        let other_path = root.join("other.ject");
        std::fs::write(&app_path, "print 1\n").unwrap();
        std::fs::write(&other_path, "import \"./app\"\n").unwrap();

        // The root import exists only in the editor buffer. Graph validation
        // must use it instead of the older contents currently on disk.
        let diagnostics = analyze_at("import \"./other\"\n", &app_path);
        let cycle = diagnostics
            .iter()
            .find(|diagnostic| {
                diagnostic.code == Some(NumberOrString::String("E3102".into()))
                    && diagnostic.message.contains("circular module import")
            })
            .unwrap();
        let related = cycle.related_information.as_ref().unwrap();
        assert_eq!(related.len(), 1);
        assert_eq!(
            related[0].location.uri,
            Url::from_file_path(other_path.canonicalize().unwrap()).unwrap()
        );
        assert_eq!(related[0].location.range.start, Position::new(0, 8));
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn dependency_invalidation_is_transitive_and_import_form_agnostic() {
        let root = std::env::temp_dir().join(format!("ject-lsp-deps-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let leaf_path = root.join("leaf.ject");
        let middle_path = root.join("middle.ject");
        let app_path = root.join("app.ject");
        std::fs::write(&leaf_path, "export value = 1\n").unwrap();
        std::fs::write(
            &middle_path,
            "import {value} from \"./leaf\"\nexport middle = value\n",
        )
        .unwrap();
        std::fs::write(&app_path, "import \"./middle\" as middle\n").unwrap();
        let leaf_uri = Url::from_file_path(&leaf_path).unwrap();
        let middle_uri = Url::from_file_path(&middle_path).unwrap();
        let app_uri = Url::from_file_path(&app_path).unwrap();
        let indexes = HashMap::from([
            (leaf_uri.clone(), index_document("export value = 1\n")),
            (
                middle_uri.clone(),
                index_document("import {value} from \"./leaf\"\nexport middle = value\n"),
            ),
            (
                app_uri.clone(),
                index_document("import \"./middle\" as middle\n"),
            ),
        ]);

        assert_eq!(
            dependent_documents(&indexes, &leaf_uri),
            vec![app_uri, middle_uri]
        );
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn diagnostics_use_unsaved_dependency_sources() {
        let root = std::env::temp_dir().join(format!("ject-lsp-overlay-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let app_path = root.join("app.ject");
        let dependency_path = root.join("dependency.ject");
        let app = "import {new_api} from \"./dependency\"\nprint new_api()\n";
        std::fs::write(&app_path, app).unwrap();
        std::fs::write(&dependency_path, "export old_api = 1\n").unwrap();

        let disk_diagnostics = analyze_at(app, &app_path);
        assert!(disk_diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.code == Some(NumberOrString::String("E3101".into())) }));
        let sources = HashMap::from([(
            dependency_path.canonicalize().unwrap(),
            "export fn new_api()\n    return 1\nend\n".into(),
        )]);
        let overlay_diagnostics = analyze_at_with_sources(app, &app_path, &sources);
        assert!(!overlay_diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.code == Some(NumberOrString::String("E3101".into())) }));
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn resolved_graph_modules_enable_navigation_without_workspace_scan() {
        let root =
            std::env::temp_dir().join(format!("ject-lsp-graph-index-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let app_path = root.join("app.ject");
        let dependency_path = root.join("dependency.ject");
        let app = "import \"./dependency\" as dep\ndep.paint()\n";
        let dependency = "export fn paint()\nend\n";
        std::fs::write(&app_path, app).unwrap();
        std::fs::write(&dependency_path, dependency).unwrap();
        let app_uri = Url::from_file_path(app_path.canonicalize().unwrap()).unwrap();
        let dependency_uri = Url::from_file_path(dependency_path.canonicalize().unwrap()).unwrap();
        let graph = crate::module_graph::ModuleGraph::build(&app_path).unwrap();
        let discovered = graph_documents(&graph, &HashMap::new());
        let mut indexes = HashMap::from([(app_uri.clone(), index_document(app))]);
        let mut documents = HashMap::from([(app_uri.clone(), app.to_string())]);
        for (uri, source) in discovered {
            indexes
                .entry(uri.clone())
                .or_insert_with(|| index_document(&source));
            documents.entry(uri).or_insert(source);
        }

        let identity =
            symbol_identity_at(&indexes, &documents, &app_uri, Position::new(1, 5)).unwrap();
        assert_eq!(identity.0, dependency_uri);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn completion_hides_symbols_from_closed_scopes() {
        let uri = Url::parse("file:///tmp/app.ject").unwrap();
        let source = "let outer = 1\nif true\n    let inner = 2\n    inn\nend\nout";
        let indexes = HashMap::from([(uri.clone(), index_document(source))]);
        let documents = HashMap::from([(uri.clone(), source.into())]);
        let inside = completion_items(&indexes, &documents, &uri, Position::new(3, 7));
        assert!(inside.iter().any(|item| item.label == "inner"));
        let after = completion_items(&indexes, &documents, &uri, Position::new(5, 3));
        assert!(after.iter().any(|item| item.label == "outer"));
        assert!(!after.iter().any(|item| item.label == "inner"));
    }

    #[test]
    fn signature_help_resolves_imported_qualified_functions() {
        let colors_uri = Url::parse("file:///tmp/colors.ject").unwrap();
        let app_uri = Url::parse("file:///tmp/app.ject").unwrap();
        let colors = "export fn mix(left, right, amount=0.5)\nend";
        let app = "import \"colors\" as c\nc.mix(\"red\", ";
        let indexes = HashMap::from([
            (colors_uri.clone(), index_document(colors)),
            (app_uri.clone(), index_document(app)),
        ]);
        let documents = HashMap::from([(colors_uri, colors.into()), (app_uri.clone(), app.into())]);
        let help = signature_help_at(&indexes, &documents, &app_uri, Position::new(1, 13))
            .expect("signature help");
        assert_eq!(help.signatures[0].label, "fn mix(left, right, amount=0.5)");
        assert_eq!(help.active_parameter, Some(1));
        assert_eq!(help.signatures[0].parameters.as_ref().unwrap().len(), 3);
    }

    #[test]
    fn signatures_preserve_complex_default_values() {
        let source =
            "fn configure(\n    color=rgb(10, 20, 30),\n    label=\"hello, world\"\n)\nend";
        let index = index_document(source);
        let function = index
            .occurrences
            .iter()
            .find(|item| item.name == "configure" && item.definition)
            .unwrap();
        assert_eq!(
            function.detail.as_deref(),
            Some("fn configure(color=rgb(10, 20, 30), label=\"hello, world\")")
        );
        assert_eq!(
            signature_parameters(function.detail.as_deref().unwrap()).len(),
            2
        );
    }

    #[test]
    fn signature_help_ignores_nested_argument_commas() {
        let uri = Url::parse("file:///tmp/app.ject").unwrap();
        let source = "map([pair(1, 2)], ";
        let indexes = HashMap::from([(uri.clone(), index_document(source))]);
        let documents = HashMap::from([(uri.clone(), source.into())]);
        let help = signature_help_at(&indexes, &documents, &uri, Position::new(0, 18))
            .expect("signature help");
        assert_eq!(help.signatures[0].label, "map(array, function) -> array");
        assert_eq!(help.active_parameter, Some(1));

        let string_source = "print(\"still (the first, argument\", ";
        let string_indexes = HashMap::from([(uri.clone(), index_document(string_source))]);
        let string_documents = HashMap::from([(uri.clone(), string_source.into())]);
        let help = signature_help_at(
            &string_indexes,
            &string_documents,
            &uri,
            Position::new(0, string_source.len() as u32),
        )
        .expect("signature help inside a call containing punctuation");
        assert_eq!(help.signatures[0].label, "print(...values)");
        assert_eq!(help.active_parameter, Some(0));
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
