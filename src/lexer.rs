use logos::Logos;

use crate::diagnostic::FrontendError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl From<std::ops::Range<usize>> for Span {
    fn from(range: std::ops::Range<usize>) -> Self {
        Self {
            start: range.start,
            end: range.end,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Logos)]
#[logos(skip r"[ \t\r\n]+")]
#[logos(skip r"//[^\n]*")]
pub enum TokenKind {
    #[token("def")]
    Def,
    #[token("let")]
    Let,
    #[token("stmt")]
    Stmt,
    #[token("has")]
    Has,
    #[token("{")]
    LeftBrace,
    #[token("}")]
    RightBrace,
    #[token(":")]
    Colon,
    #[token("+")]
    Plus,
    #[token(",")]
    Comma,
    #[regex(r"[0-9]+(?:\.[0-9]+)?", |lex| lex.slice().to_owned())]
    Number(String),
    #[regex(r"[A-Za-z_][A-Za-z0-9_]*", |lex| lex.slice().to_owned())]
    Identifier(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

pub fn lex(source_name: &str, source: &str) -> Result<Vec<Token>, FrontendError> {
    let lexer = TokenKind::lexer(source);
    let mut tokens = Vec::new();

    for (result, range) in lexer.spanned() {
        let span = Span::from(range);
        match result {
            Ok(kind) => tokens.push(Token { kind, span }),
            Err(()) => {
                return Err(FrontendError::new(
                    source_name,
                    source,
                    "unrecognized character",
                    span,
                ));
            }
        }
    }

    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenizes_keywords_identifiers_and_punctuation() {
        let tokens = lex("test.mt", "def Cat { color: Color }").unwrap();
        assert!(matches!(tokens[0].kind, TokenKind::Def));
        assert!(matches!(&tokens[1].kind, TokenKind::Identifier(name) if name == "Cat"));
        assert!(matches!(tokens[2].kind, TokenKind::LeftBrace));
        assert!(matches!(&tokens[3].kind, TokenKind::Identifier(name) if name == "color"));
        assert!(matches!(tokens[4].kind, TokenKind::Colon));
    }

    #[test]
    fn skips_line_comments() {
        let tokens = lex("test.mt", "def Cat // comment\nlet cat: Cat").unwrap();
        assert_eq!(tokens.len(), 6);
        assert!(matches!(tokens[4].kind, TokenKind::Colon));
    }

    #[test]
    fn recognizes_numeric_literals() {
        let tokens = lex("test.mt", "42 3.5").unwrap();
        assert!(matches!(&tokens[0].kind, TokenKind::Number(value) if value == "42"));
        assert!(matches!(&tokens[1].kind, TokenKind::Number(value) if value == "3.5"));
    }
}
