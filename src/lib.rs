pub mod ast;
pub mod diagnostic;
pub mod interpreter;
pub mod jgui;
pub mod jnum;
pub mod lexer;
pub mod linter;
pub mod module_interface;
pub mod module_resolver;
pub mod native;
pub mod package;
pub mod parser;
pub mod semantic;
pub mod stdlib;
pub mod value;

#[cfg(test)]
mod tests;

// Re-export the main types for easy access
pub use ast::*;
pub use interpreter::*;
pub use lexer::*;
pub use parser::*;
pub use stdlib::*;
pub use value::*;
