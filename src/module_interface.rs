//! Public source-module metadata shared by diagnostics and editor tooling.

use crate::ast::{Parameter, Stmt};
use crate::lexer::Lexer;
use crate::parser::Parser;

#[derive(Debug, Clone, PartialEq)]
pub enum ExportKind {
    Value,
    Function { parameters: Vec<Parameter> },
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModuleExport {
    pub name: String,
    pub kind: ExportKind,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ModuleInterface {
    pub exports: Vec<ModuleExport>,
}

impl ModuleInterface {
    pub fn parse(source: &str) -> Result<Self, String> {
        let mut lexer = Lexer::new(source);
        let tokens = lexer
            .tokenize_with_positions()
            .into_iter()
            .map(|token| token.token)
            .collect();
        let statements = Parser::new_simple(tokens)
            .parse()
            .map_err(|error| error.message)?;
        Ok(Self::from_statements(&statements))
    }

    pub fn from_statements(statements: &[Stmt]) -> Self {
        let exports = statements
            .iter()
            .filter_map(|statement| match statement {
                Stmt::Export { name, .. } => Some(ModuleExport {
                    name: name.clone(),
                    kind: ExportKind::Value,
                }),
                Stmt::ExportFunction { name, params, .. } => Some(ModuleExport {
                    name: name.clone(),
                    kind: ExportKind::Function {
                        parameters: params.clone(),
                    },
                }),
                _ => None,
            })
            .collect();
        Self { exports }
    }

    pub fn export(&self, name: &str) -> Option<&ModuleExport> {
        self.exports.iter().find(|export| export.name == name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_only_public_api_and_preserves_defaults() {
        let interface = ModuleInterface::parse(
            r#"
let private = 1
export VERSION = "1.0"
fn helper(value)
    return value
end
export fn greet(name, punctuation="!")
    return name + punctuation
end
"#,
        )
        .unwrap();
        assert_eq!(interface.exports.len(), 2);
        assert!(matches!(
            interface.export("VERSION").unwrap().kind,
            ExportKind::Value
        ));
        let ExportKind::Function { parameters } = &interface.export("greet").unwrap().kind else {
            panic!("greet should be a function");
        };
        assert_eq!(parameters.len(), 2);
        assert!(parameters[1].default_value.is_some());
    }
}
