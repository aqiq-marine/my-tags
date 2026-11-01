use miette::{Diagnostic, NamedSource, SourceSpan};
use thiserror::Error;

use crate::lexer::Span;

#[derive(Debug, Error, Diagnostic)]
#[error("{message}")]
#[diagnostic(code(my_tags::frontend))]
pub struct FrontendError {
    message: String,
    #[label("{message}")]
    span: SourceSpan,
    #[source_code]
    source_code: NamedSource<String>,
}

impl FrontendError {
    pub fn new(
        source_name: impl Into<String>,
        source: &str,
        message: impl Into<String>,
        span: Span,
    ) -> Self {
        let message = message.into();
        Self {
            message,
            span: SourceSpan::new(
                span.start.into(),
                span.end.saturating_sub(span.start),
            ),
            source_code: NamedSource::new(source_name.into(), source.to_owned()),
        }
    }
}
