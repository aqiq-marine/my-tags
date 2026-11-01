use logos::Logos;

use crate::diagnostic::FrontendError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl From<std::ops::Range<usize>> for Span {
    fn from(range: std::ops::Range<usize>) -> Self {
        Self { start: range.start, end: range.end }
    }
}

#[derive(Debug, Clone, PartialEq, Logos)]
#[logos(skip r"[ \t\r\n]+")]
#[logos(skip r"//[^\n]*")]
pub enum TokenKind {
    #[token("def")] Def,
    #[token("let")] Let,
    #[token("stmt")] Stmt,
    #[token("has")] Has,
    #[token("show")] Show,
    #[token("query")] Query,
    #[token("#")] Hash,
    #[token("{")] LeftBrace,
    #[token("}")] RightBrace,
    #[token("(")] LeftParen,
    #[token(")")] RightParen,
    #[token(":")] Colon,
    #[token("+")] Plus,
    #[token("-")] Minus,
    #[token("&")] Ampersand,
    #[token("|")] Pipe,
    #[token("!")] Bang,
    #[token("&&")] AndAnd,
    #[token("||")] OrOr,
    #[token("==")] EqualEqual,
    #[token("!=")] NotEqual,
    #[token(">=")] GreaterEqual,
    #[token("<=")] LessEqual,
    #[token(">")] Greater,
    #[token("<")] Less,
    #[token(",")] Comma,
    #[regex(r"(?:[0-9]+\.[0-9]+|\.[0-9]+)(?:[eE][+-]?[0-9]+)?|[0-9]+[eE][+-]?[0-9]+", |lex| lex.slice().to_owned())]
    Float(String),
    #[regex(r"[0-9]+", |lex| lex.slice().to_owned())]
    Integer(String),
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
            Err(()) => return Err(FrontendError::new(source_name, source, "unrecognized character", span)),
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
    fn recognizes_integer_and_float_literals() {
        let tokens = lex("test.mt", "42 -3 0.25 -1.5e-2 .75").unwrap();
        assert!(matches!(&tokens[0].kind, TokenKind::Integer(value) if value == "42"));
        assert!(matches!(&tokens[2].kind, TokenKind::Integer(value) if value == "3"));
        assert!(matches!(&tokens[3].kind, TokenKind::Float(value) if value == "0.25"));
        assert!(matches!(&tokens[5].kind, TokenKind::Float(value) if value == "1.5e-2"));
        assert!(matches!(&tokens[6].kind, TokenKind::Float(value) if value == ".75"));
    }

    #[test]
    fn recognizes_query_operators() {
        let tokens = lex("test.mt", "#show query A {} & B {} || !C {}").unwrap();
        assert!(tokens.iter().any(|token| matches!(token.kind, TokenKind::Ampersand)));
        assert!(tokens.iter().any(|token| matches!(token.kind, TokenKind::OrOr)));
        assert!(tokens.iter().any(|token| matches!(token.kind, TokenKind::Bang)));
    }
}
