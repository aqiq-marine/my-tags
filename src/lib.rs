pub mod ast;
pub mod cli;
pub mod diagnostic;
pub mod lexer;
pub mod parser;
pub mod semantic;

pub use ast::Program;
pub use parser::parse;
pub use semantic::analyze;
