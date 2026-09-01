//! Resolver-backed dependency graph for Ject source modules.

use crate::ast::Stmt;
use crate::lexer::Lexer;
use crate::module_interface::ModuleInterface;
use crate::module_resolver::{ModuleIdentity, ModuleResolver, ResolvedModule};
use crate::parser::Parser;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct ModuleNode {
    pub identity: ModuleIdentity,
    pub interface: ModuleInterface,
    pub dependencies: Vec<ModuleIdentity>,
    pub edges: Vec<ModuleEdge>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportSite {
    pub module: ModuleIdentity,
    pub specifier: String,
    pub line: usize,
    pub column: usize,
    pub length: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleEdge {
    pub target: ModuleIdentity,
    pub site: ImportSite,
}

#[derive(Debug, Clone, Default)]
pub struct ModuleGraph {
    pub root: Option<ModuleIdentity>,
    pub nodes: HashMap<ModuleIdentity, ModuleNode>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModuleGraphError {
    Load {
        chain: Vec<String>,
        message: String,
        site: Option<ImportSite>,
    },
    Parse {
        chain: Vec<String>,
        message: String,
    },
    Cycle {
        chain: Vec<String>,
        site: ImportSite,
    },
}

impl std::fmt::Display for ModuleGraphError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Load { chain, message, .. } | Self::Parse { chain, message } => {
                write!(formatter, "{message}")?;
                if chain.len() > 1 {
                    write!(formatter, "\nimport chain: {}", chain.join(" -> "))?;
                }
                Ok(())
            }
            Self::Cycle { chain, .. } => {
                write!(formatter, "circular module import: {}", chain.join(" -> "))
            }
        }
    }
}

impl std::error::Error for ModuleGraphError {}

impl ModuleGraphError {
    pub fn import_site(&self) -> Option<&ImportSite> {
        match self {
            Self::Load { site, .. } => site.as_ref(),
            Self::Cycle { site, .. } => Some(site),
            Self::Parse { .. } => None,
        }
    }
}

impl ModuleGraph {
    pub fn build(entry: &Path) -> Result<Self, ModuleGraphError> {
        let canonical = entry
            .canonicalize()
            .map_err(|error| ModuleGraphError::Load {
                chain: vec![display_identity(&ModuleIdentity::File(entry.to_path_buf()))],
                message: format!("failed to load module '{}': {error}", entry.display()),
                site: None,
            })?;
        let source = fs::read_to_string(&canonical).map_err(|error| ModuleGraphError::Load {
            chain: vec![canonical.display().to_string()],
            message: format!("failed to load module '{}': {error}", canonical.display()),
            site: None,
        })?;
        Self::build_source(&canonical, source)
    }

    /// Build from an in-memory root while loading dependencies from the resolver.
    /// This keeps editor diagnostics correct for unsaved changes.
    pub fn build_source(entry: &Path, source: String) -> Result<Self, ModuleGraphError> {
        Self::build_sources(entry, source, &HashMap::new())
    }

    pub fn build_sources(
        entry: &Path,
        source: String,
        sources: &HashMap<std::path::PathBuf, String>,
    ) -> Result<Self, ModuleGraphError> {
        let canonical = entry
            .canonicalize()
            .map_err(|error| ModuleGraphError::Load {
                chain: vec![display_identity(&ModuleIdentity::File(entry.to_path_buf()))],
                message: format!("failed to load module '{}': {error}", entry.display()),
                site: None,
            })?;
        let root = ModuleIdentity::File(canonical.clone());
        let module = ResolvedModule {
            identity: root.clone(),
            source,
            directory: canonical.parent().unwrap_or(&canonical).to_path_buf(),
            package: None,
        };
        let resolver = ModuleResolver::for_path(&canonical);
        let mut graph = Self {
            root: Some(root),
            nodes: HashMap::new(),
        };
        let mut active = Vec::new();
        graph.visit(module, &resolver, sources, &mut active, None)?;
        Ok(graph)
    }

    fn visit(
        &mut self,
        module: ResolvedModule,
        resolver: &ModuleResolver,
        sources: &HashMap<std::path::PathBuf, String>,
        active: &mut Vec<ModuleIdentity>,
        incoming: Option<ImportSite>,
    ) -> Result<(), ModuleGraphError> {
        if let Some(start) = active
            .iter()
            .position(|identity| identity == &module.identity)
        {
            let mut cycle = active[start..]
                .iter()
                .map(display_identity)
                .collect::<Vec<_>>();
            cycle.push(display_identity(&module.identity));
            return Err(ModuleGraphError::Cycle {
                chain: cycle,
                site: incoming.expect("non-root cycle must have an import site"),
            });
        }
        if self.nodes.contains_key(&module.identity) {
            return Ok(());
        }

        active.push(module.identity.clone());
        let (statements, imports) =
            parse_module(&module).map_err(|message| ModuleGraphError::Parse {
                chain: active.iter().map(display_identity).collect(),
                message,
            })?;
        let interface = ModuleInterface::from_statements(&statements);
        let child_resolver = match &module.identity {
            ModuleIdentity::File(path) => ModuleResolver::for_path(path),
            ModuleIdentity::Embedded(_) => resolver.with_base(&module.directory),
        };
        let mut dependencies = Vec::new();
        let mut edges = Vec::new();
        for parsed in imports {
            let specifier = parsed.specifier;
            let site = ImportSite {
                module: module.identity.clone(),
                length: specifier.chars().count(),
                specifier: specifier.clone(),
                line: parsed.line,
                column: parsed.column,
            };
            let resolved = child_resolver
                .resolve_with_sources(&specifier, sources)
                .map_err(|error| {
                    let mut chain = active.iter().map(display_identity).collect::<Vec<_>>();
                    chain.push(specifier.clone());
                    ModuleGraphError::Load {
                        chain,
                        message: error.to_string(),
                        site: Some(site.clone()),
                    }
                })?;
            dependencies.push(resolved.identity.clone());
            edges.push(ModuleEdge {
                target: resolved.identity.clone(),
                site: site.clone(),
            });
            self.visit(resolved, &child_resolver, sources, active, Some(site))?;
        }
        active.pop();
        self.nodes.insert(
            module.identity.clone(),
            ModuleNode {
                identity: module.identity,
                interface,
                dependencies,
                edges,
            },
        );
        Ok(())
    }
}

#[derive(Debug)]
struct ParsedImport {
    specifier: String,
    line: usize,
    column: usize,
}

fn parse_module(module: &ResolvedModule) -> Result<(Vec<Stmt>, Vec<ParsedImport>), String> {
    let located = Lexer::new(&module.source).tokenize_with_positions();
    let imports = collect_imports(&located);
    let tokens = located.into_iter().map(|token| token.token).collect();
    let statements = Parser::new_simple(tokens).parse().map_err(|error| {
        format!(
            "cannot parse module '{}': {}",
            display_identity(&module.identity),
            error.message
        )
    })?;
    Ok((statements, imports))
}

fn collect_imports(tokens: &[crate::lexer::LocatedToken]) -> Vec<ParsedImport> {
    let mut imports = Vec::new();
    for (index, token) in tokens.iter().enumerate() {
        if !matches!(token.token, crate::lexer::Token::Import) {
            continue;
        }
        if let Some(crate::lexer::LocatedToken {
            token: crate::lexer::Token::String(specifier),
            position,
        }) = tokens[index + 1..]
            .iter()
            .find(|token| matches!(token.token, crate::lexer::Token::String(_)))
        {
            if !specifier.starts_with("@native/") {
                imports.push(ParsedImport {
                    specifier: specifier.clone(),
                    line: position.line,
                    column: position.column + 1,
                });
            }
        }
    }
    imports
}

fn display_identity(identity: &ModuleIdentity) -> String {
    match identity {
        ModuleIdentity::File(path) => path.display().to_string(),
        ModuleIdentity::Embedded(name) => name.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn fixture() -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("ject-module-graph-{nonce}"));
        fs::create_dir_all(&root).unwrap();
        root
    }

    #[test]
    fn builds_transitive_interfaces_once() {
        let root = fixture();
        fs::write(root.join("main.ject"), "import \"./middle\"\n").unwrap();
        fs::write(
            root.join("middle.ject"),
            "import \"./leaf\"\nexport fn middle(value)\nend\n",
        )
        .unwrap();
        fs::write(root.join("leaf.ject"), "export answer = 42\n").unwrap();
        let graph = ModuleGraph::build(&root.join("main.ject")).unwrap();
        assert_eq!(graph.nodes.len(), 3);
        assert!(graph
            .nodes
            .values()
            .any(|node| node.interface.export("answer").is_some()));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reports_cycles_with_the_complete_loop() {
        let root = fixture();
        fs::write(root.join("a.ject"), "import \"./b\"\n").unwrap();
        fs::write(root.join("b.ject"), "import \"./a\"\n").unwrap();
        let error = ModuleGraph::build(&root.join("a.ject")).unwrap_err();
        let ModuleGraphError::Cycle { chain, site } = error else {
            panic!("expected a cycle");
        };
        assert_eq!(chain.len(), 3);
        assert_eq!(chain.first(), chain.last());
        assert_eq!(site.specifier, "./a");
        assert_eq!((site.line, site.column), (1, 9));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reports_the_import_chain_for_transitive_failures() {
        let root = fixture();
        fs::write(root.join("main.ject"), "import \"./middle\"\n").unwrap();
        fs::write(root.join("middle.ject"), "import \"./missing\"\n").unwrap();
        let error = ModuleGraph::build(&root.join("main.ject")).unwrap_err();
        let ModuleGraphError::Load { chain, site, .. } = error else {
            panic!("expected a load failure");
        };
        assert_eq!(chain.len(), 3);
        assert!(chain.last().unwrap().contains("missing"));
        let site = site.unwrap();
        assert_eq!(site.specifier, "./missing");
        assert_eq!((site.line, site.column), (1, 9));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn follows_imports_nested_in_statement_bodies() {
        let root = fixture();
        fs::write(
            root.join("main.ject"),
            "fn load()\n    if true\n        import \"./nested\"\n    end\nend\n",
        )
        .unwrap();
        fs::write(root.join("nested.ject"), "export value = 1\n").unwrap();
        let graph = ModuleGraph::build(&root.join("main.ject")).unwrap();
        assert_eq!(graph.nodes.len(), 2);
        fs::remove_dir_all(root).unwrap();
    }
}
