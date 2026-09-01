//! Scope-aware semantic index shared by diagnostics and editor tooling.

use crate::diagnostic::SourceSpan;
use crate::lexer::{Lexer, LocatedToken, Token};
use std::collections::HashMap;

pub type ScopeId = usize;
pub type SymbolId = usize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolKind {
    Variable,
    Parameter,
    Function,
    Struct,
    Module,
    Import,
}

#[derive(Debug, Clone)]
pub struct Symbol {
    pub id: SymbolId,
    pub name: String,
    pub kind: SymbolKind,
    pub span: SourceSpan,
    pub scope: ScopeId,
}

#[derive(Debug, Clone)]
pub struct Reference {
    pub name: String,
    pub span: SourceSpan,
    pub scope: ScopeId,
    pub resolved: Option<SymbolId>,
}

#[derive(Debug, Clone)]
pub struct Scope {
    pub id: ScopeId,
    pub parent: Option<ScopeId>,
    pub start: (usize, usize),
    pub end: Option<(usize, usize)>,
}

#[derive(Debug, Clone, Default)]
pub struct SemanticIndex {
    pub scopes: Vec<Scope>,
    pub symbols: Vec<Symbol>,
    pub references: Vec<Reference>,
}

impl SemanticIndex {
    pub fn symbol_at(&self, line: usize, column: usize) -> Option<SymbolId> {
        self.symbols
            .iter()
            .find(|symbol| contains(symbol.span, line, column))
            .map(|symbol| symbol.id)
            .or_else(|| {
                self.references
                    .iter()
                    .find(|reference| contains(reference.span, line, column))
                    .and_then(|reference| reference.resolved)
            })
    }

    pub fn spans_for(&self, symbol: SymbolId) -> Vec<SourceSpan> {
        let mut spans = self
            .symbols
            .iter()
            .filter(|item| item.id == symbol)
            .map(|item| item.span)
            .collect::<Vec<_>>();
        spans.extend(
            self.references
                .iter()
                .filter(|item| item.resolved == Some(symbol))
                .map(|item| item.span),
        );
        spans
    }

    pub fn visible_symbols(&self, line: usize, column: usize) -> Vec<SymbolId> {
        let position = (line, column);
        let mut scope = self
            .scopes
            .iter()
            .filter(|scope| scope.start <= position && scope.end.is_none_or(|end| position <= end))
            .max_by_key(|scope| scope_depth(&self.scopes, scope.id))
            .map(|scope| scope.id)
            .unwrap_or(0);
        let mut scopes = Vec::new();
        loop {
            scopes.push(scope);
            let Some(parent) = self.scopes.get(scope).and_then(|item| item.parent) else {
                break;
            };
            scope = parent;
        }
        let mut names = HashMap::new();
        let mut result = Vec::new();
        for scope in scopes {
            for symbol in self.symbols.iter().rev().filter(|symbol| {
                symbol.scope == scope && (symbol.span.line, symbol.span.column) <= position
            }) {
                if names.insert(symbol.name.clone(), ()).is_none() {
                    result.push(symbol.id);
                }
            }
        }
        result
    }
}

pub fn analyze(source: &str) -> SemanticIndex {
    let mut lexer = Lexer::new(source);
    analyze_tokens(&lexer.tokenize_with_positions())
}

fn analyze_tokens(tokens: &[LocatedToken]) -> SemanticIndex {
    let mut index = SemanticIndex {
        scopes: vec![Scope {
            id: 0,
            parent: None,
            start: (1, 1),
            end: None,
        }],
        symbols: Vec::new(),
        references: Vec::new(),
    };
    let mut bindings: Vec<HashMap<String, SymbolId>> = vec![HashMap::new()];
    let mut scope_stack = vec![0usize];
    let mut pending: Option<SymbolKind> = None;
    let mut pending_function_scope = false;
    let mut parameter_depth: Option<usize> = None;
    let mut expect_parameter = false;
    let mut previous: Option<&Token> = None;
    let mut import_may_be_selective = false;
    let mut selective_import = false;

    for (position, located) in tokens.iter().enumerate() {
        match &located.token {
            Token::End => pop_scope(&mut index, &mut scope_stack, &located.position),
            Token::Let => pending = Some(SymbolKind::Variable),
            Token::Export => pending = Some(SymbolKind::Variable),
            Token::Import => import_may_be_selective = true,
            Token::Fn => {
                pending = Some(SymbolKind::Function);
                pending_function_scope = true;
            }
            Token::Struct => pending = Some(SymbolKind::Struct),
            Token::As => pending = Some(SymbolKind::Module),
            Token::For => {
                push_scope(
                    &mut index,
                    &mut bindings,
                    &mut scope_stack,
                    &located.position,
                );
                pending = Some(SymbolKind::Variable);
            }
            Token::Catch => pending = Some(SymbolKind::Variable),
            Token::If | Token::While | Token::Match | Token::Try => {
                push_scope(
                    &mut index,
                    &mut bindings,
                    &mut scope_stack,
                    &located.position,
                );
            }
            Token::LeftBrace if import_may_be_selective => {
                import_may_be_selective = false;
                selective_import = true;
            }
            Token::RightBrace if selective_import => selective_import = false,
            Token::String(_) if import_may_be_selective => import_may_be_selective = false,
            Token::LeftParen if pending_function_scope => {
                push_scope(
                    &mut index,
                    &mut bindings,
                    &mut scope_stack,
                    &located.position,
                );
                pending_function_scope = false;
                pending = None;
                parameter_depth = Some(1);
                expect_parameter = true;
            }
            Token::LeftParen => {
                if let Some(depth) = parameter_depth.as_mut() {
                    *depth += 1;
                }
            }
            Token::RightParen => {
                if let Some(depth) = parameter_depth.as_mut() {
                    *depth = depth.saturating_sub(1);
                    if *depth == 0 {
                        parameter_depth = None;
                    }
                }
            }
            Token::Comma if parameter_depth == Some(1) => expect_parameter = true,
            Token::Arrow if parameter_depth.is_none() && scope_stack.len() > 1 => {
                // Expression-bodied functions do not have a matching `end`.
                if matches!(previous, Some(Token::RightParen)) {
                    pop_scope(&mut index, &mut scope_stack, &located.position);
                }
            }
            Token::Identifier(name) => {
                let span = token_span(located, name);
                let kind = if selective_import {
                    Some(SymbolKind::Import)
                } else if parameter_depth == Some(1) && expect_parameter {
                    expect_parameter = false;
                    Some(SymbolKind::Parameter)
                } else {
                    pending.take()
                };
                if let Some(kind) = kind {
                    let scope = *scope_stack.last().unwrap_or(&0);
                    let id = index.symbols.len();
                    index.symbols.push(Symbol {
                        id,
                        name: name.clone(),
                        kind,
                        span,
                        scope,
                    });
                    bindings[scope].insert(name.clone(), id);
                    if kind == SymbolKind::Function && pending_function_scope {
                        // The function name belongs to its parent scope; its parameters
                        // belong to the child scope created at `(`.
                    }
                } else if !matches!(previous, Some(Token::Dot))
                    && !is_keyword_argument(tokens, position, parameter_depth)
                {
                    let scope = *scope_stack.last().unwrap_or(&0);
                    let resolved = resolve(name, &scope_stack, &bindings);
                    index.references.push(Reference {
                        name: name.clone(),
                        span,
                        scope,
                        resolved,
                    });
                }
            }
            _ => {}
        }
        if !matches!(located.token, Token::Newline) {
            previous = Some(&located.token);
        }
    }
    index
}

fn push_scope(
    index: &mut SemanticIndex,
    bindings: &mut Vec<HashMap<String, SymbolId>>,
    stack: &mut Vec<ScopeId>,
    position: &crate::lexer::SourcePosition,
) {
    let id = index.scopes.len();
    index.scopes.push(Scope {
        id,
        parent: stack.last().copied(),
        start: (position.line, position.column),
        end: None,
    });
    bindings.push(HashMap::new());
    stack.push(id);
}

fn pop_scope(
    index: &mut SemanticIndex,
    stack: &mut Vec<ScopeId>,
    position: &crate::lexer::SourcePosition,
) {
    if stack.len() > 1 {
        if let Some(scope) = stack.pop() {
            index.scopes[scope].end = Some((position.line, position.column));
        }
    }
}

fn scope_depth(scopes: &[Scope], mut scope: ScopeId) -> usize {
    let mut depth = 0;
    while let Some(parent) = scopes.get(scope).and_then(|item| item.parent) {
        depth += 1;
        scope = parent;
    }
    depth
}

fn resolve(
    name: &str,
    stack: &[ScopeId],
    bindings: &[HashMap<String, SymbolId>],
) -> Option<SymbolId> {
    stack
        .iter()
        .rev()
        .find_map(|scope| bindings[*scope].get(name).copied())
}

fn token_span(token: &LocatedToken, name: &str) -> SourceSpan {
    SourceSpan::new(
        token.position.line,
        token.position.column,
        name.chars().count(),
    )
}

fn contains(span: SourceSpan, line: usize, column: usize) -> bool {
    span.line == line && column >= span.column && column <= span.column + span.length
}

fn is_keyword_argument(
    tokens: &[LocatedToken],
    position: usize,
    parameter_depth: Option<usize>,
) -> bool {
    parameter_depth.is_none()
        && matches!(
            tokens.get(position + 1).map(|token| &token.token),
            Some(Token::Equal)
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_shadowed_names_to_distinct_symbols() {
        let index =
            analyze("let value = 1\nif true\n    let value = 2\n    print value\nend\nprint value");
        let values: Vec<_> = index
            .symbols
            .iter()
            .filter(|symbol| symbol.name == "value")
            .collect();
        assert_eq!(values.len(), 2);
        assert_ne!(values[0].id, values[1].id);
        assert_eq!(
            index
                .references
                .iter()
                .filter(|reference| reference.name == "value")
                .map(|reference| reference.resolved)
                .collect::<Vec<_>>(),
            vec![Some(values[1].id), Some(values[0].id)]
        );
    }

    #[test]
    fn parameters_have_exact_spans_and_resolve_in_body() {
        let index = analyze("fn greet(name, punctuation=\"!\")\n    print name, punctuation\nend");
        let name = index
            .symbols
            .iter()
            .find(|symbol| symbol.name == "name")
            .unwrap();
        assert_eq!(name.kind, SymbolKind::Parameter);
        assert_eq!(name.span, SourceSpan::new(1, 10, 4));
        assert!(index
            .references
            .iter()
            .any(|reference| reference.name == "name" && reference.resolved == Some(name.id)));
    }

    #[test]
    fn visible_symbols_follow_scope_ranges_and_shadowing() {
        let index = analyze(
            "let outer = 1\nif true\n    let inner = 2\n    let outer = 3\n    print outer\nend\nprint outer",
        );
        let inside = index
            .visible_symbols(5, 10)
            .into_iter()
            .map(|id| &index.symbols[id])
            .collect::<Vec<_>>();
        assert!(inside.iter().any(|symbol| symbol.name == "inner"));
        assert_eq!(
            inside
                .iter()
                .filter(|symbol| symbol.name == "outer")
                .count(),
            1
        );
        assert_eq!(
            inside
                .iter()
                .find(|symbol| symbol.name == "outer")
                .unwrap()
                .span
                .line,
            4
        );

        let after = index
            .visible_symbols(7, 10)
            .into_iter()
            .map(|id| &index.symbols[id])
            .collect::<Vec<_>>();
        assert!(!after.iter().any(|symbol| symbol.name == "inner"));
        assert_eq!(
            after
                .iter()
                .find(|symbol| symbol.name == "outer")
                .unwrap()
                .span
                .line,
            1
        );
    }
}
