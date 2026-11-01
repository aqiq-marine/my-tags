use crate::{ast::Program, diagnostic::FrontendError};

pub fn analyze(
    _program: &Program,
    _source_name: &str,
    _source: &str,
) -> Result<(), Vec<FrontendError>> {
    Ok(())
}
