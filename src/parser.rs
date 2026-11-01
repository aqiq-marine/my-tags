use crate::{
    ast::{
        InstanceDefinition, Item, Program, PropertyAssignment, PropertyDefinition,
        Statement, TypeDefinition, TypeExpression, ValueExpression,
    },
    diagnostic::FrontendError,
    lexer::{lex, Span, Token, TokenKind},
};

pub fn parse(source_name: &str, source: &str) -> Result<Program, FrontendError> {
    let tokens = lex(source_name, source)?;
    Parser {
        source_name,
        source,
        tokens,
        cursor: 0,
    }
    .parse_program()
}

struct Parser<'a> {
    source_name: &'a str,
    source: &'a str,
    tokens: Vec<Token>,
    cursor: usize,
}

impl Parser<'_> {
    fn parse_program(mut self) -> Result<Program, FrontendError> {
        let mut items = Vec::new();

        while let Some(kind) = self.current().map(|token| token.kind.clone()) {
            let item = match kind {
                TokenKind::Def => Item::TypeDefinition(self.parse_type_definition()?),
                TokenKind::Let => Item::InstanceDefinition(self.parse_instance_definition()?),
                TokenKind::Stmt => Item::Statement(self.parse_statement()?),
                _ => {
                    return Err(self.error_here(
                        "expected a top-level declaration (`def`, `let`, or `stmt`)",
                    ));
                }
            };
            items.push(item);
        }

        Ok(Program { items })
    }

    fn parse_type_definition(&mut self) -> Result<TypeDefinition, FrontendError> {
        let start = self.expect_simple(|kind| matches!(kind, TokenKind::Def), "`def`")?.span;
        let (name, name_span) = self.expect_identifier()?;

        let properties = if self.check(|kind| matches!(kind, TokenKind::LeftBrace)) {
            self.parse_property_definitions()?
        } else {
            Vec::new()
        };

        let parent = if self.consume_simple(|kind| matches!(kind, TokenKind::Colon)).is_some() {
            Some(self.expect_identifier()?.0)
        } else {
            None
        };

        let end = self.previous_span().unwrap_or(name_span);
        Ok(TypeDefinition {
            name,
            properties,
            parent,
            span: Span { start: start.start, end: end.end },
        })
    }

    fn parse_property_definitions(&mut self) -> Result<Vec<PropertyDefinition>, FrontendError> {
        self.expect_simple(|kind| matches!(kind, TokenKind::LeftBrace), "`{`")?;
        let mut properties = Vec::new();

        while !self.check(|kind| matches!(kind, TokenKind::RightBrace)) {
            if self.current().is_none() {
                return Err(self.error_here("unterminated property block; expected `}`"));
            }
            let (name, name_span) = self.expect_identifier()?;
            self.expect_simple(|kind| matches!(kind, TokenKind::Colon), "`:`")?;
            let value_type = self.parse_type_expression()?;
            let end = value_type.span;
            properties.push(PropertyDefinition {
                name,
                value_type,
                span: Span { start: name_span.start, end: end.end },
            });
            self.consume_simple(|kind| matches!(kind, TokenKind::Comma));
        }

        self.expect_simple(|kind| matches!(kind, TokenKind::RightBrace), "`}`")?;
        Ok(properties)
    }

    fn parse_type_expression(&mut self) -> Result<TypeExpression, FrontendError> {
        let (first, first_span) = self.expect_identifier()?;
        let mut terms = vec![first];
        let mut end = first_span;

        while self.consume_simple(|kind| matches!(kind, TokenKind::Plus)).is_some() {
            let (term, span) = self.expect_identifier()?;
            terms.push(term);
            end = span;
        }

        Ok(TypeExpression {
            terms,
            span: Span { start: first_span.start, end: end.end },
        })
    }

    fn parse_instance_definition(&mut self) -> Result<InstanceDefinition, FrontendError> {
        let start = self.expect_simple(|kind| matches!(kind, TokenKind::Let), "`let`")?.span;
        let (name, _) = self.expect_identifier()?;
        self.expect_simple(|kind| matches!(kind, TokenKind::Colon), "`:`")?;
        let instance_type = self.parse_type_expression()?;
        Ok(InstanceDefinition {
            name,
            span: Span { start: start.start, end: instance_type.span.end },
            instance_type,
        })
    }

    fn parse_statement(&mut self) -> Result<Statement, FrontendError> {
        let start = self.expect_simple(|kind| matches!(kind, TokenKind::Stmt), "`stmt`")?.span;
        let (subject, _) = self.expect_identifier()?;
        self.expect_simple(|kind| matches!(kind, TokenKind::Has), "`has`")?;
        let (tag, tag_span) = self.expect_identifier()?;

        let properties = if self.check(|kind| matches!(kind, TokenKind::LeftBrace)) {
            self.parse_property_assignments()?
        } else {
            Vec::new()
        };

        let end = self.previous_span().unwrap_or(tag_span);
        Ok(Statement {
            subject,
            tag,
            properties,
            span: Span { start: start.start, end: end.end },
        })
    }

    fn parse_property_assignments(&mut self) -> Result<Vec<PropertyAssignment>, FrontendError> {
        self.expect_simple(|kind| matches!(kind, TokenKind::LeftBrace), "`{`")?;
        let mut properties = Vec::new();

        while !self.check(|kind| matches!(kind, TokenKind::RightBrace)) {
            if self.current().is_none() {
                return Err(self.error_here("unterminated property block; expected `}`"));
            }
            let (name, name_span) = self.expect_identifier()?;
            self.expect_simple(|kind| matches!(kind, TokenKind::Colon), "`:`")?;
            let value = self.parse_value_expression()?;
            let end = value.span();
            properties.push(PropertyAssignment {
                name,
                value,
                span: Span { start: name_span.start, end: end.end },
            });
            self.consume_simple(|kind| matches!(kind, TokenKind::Comma));
        }

        self.expect_simple(|kind| matches!(kind, TokenKind::RightBrace), "`}`")?;
        Ok(properties)
    }

    fn parse_value_expression(&mut self) -> Result<ValueExpression, FrontendError> {
        if let Some(Token { kind: TokenKind::Number(raw), span }) = self.current().cloned() {
            self.cursor += 1;
            let value = raw.parse::<f64>().map_err(|_| {
                self.error_at("invalid numeric literal", span)
            })?;
            return Ok(ValueExpression::Number { value, span });
        }

        let (name, name_span) = self.expect_identifier()?;
        if self.check(|kind| matches!(kind, TokenKind::LeftBrace)) {
            let properties = self.parse_property_assignments()?;
            let end = self.previous_span().unwrap_or(name_span);
            Ok(ValueExpression::AnonymousInstance {
                type_name: name,
                properties,
                span: Span { start: name_span.start, end: end.end },
            })
        } else {
            Ok(ValueExpression::Reference { name, span: name_span })
        }
    }

    fn current(&self) -> Option<&Token> {
        self.tokens.get(self.cursor)
    }

    fn previous_span(&self) -> Option<Span> {
        self.cursor
            .checked_sub(1)
            .and_then(|index| self.tokens.get(index))
            .map(|token| token.span)
    }

    fn check(&self, predicate: impl FnOnce(&TokenKind) -> bool) -> bool {
        self.current().is_some_and(|token| predicate(&token.kind))
    }

    fn consume_simple(&mut self, predicate: impl FnOnce(&TokenKind) -> bool) -> Option<Token> {
        if self.check(predicate) {
            let token = self.tokens.get(self.cursor).cloned();
            if token.is_some() {
                self.cursor += 1;
            }
            token
        } else {
            None
        }
    }

    fn expect_simple(
        &mut self,
        predicate: impl FnOnce(&TokenKind) -> bool,
        expected: &str,
    ) -> Result<Token, FrontendError> {
        self.consume_simple(predicate)
            .ok_or_else(|| self.error_here(format!("expected {expected}")))
    }

    fn expect_identifier(&mut self) -> Result<(String, Span), FrontendError> {
        match self.current().cloned() {
            Some(Token { kind: TokenKind::Identifier(name), span }) => {
                self.cursor += 1;
                Ok((name, span))
            }
            _ => Err(self.error_here("expected an identifier")),
        }
    }

    fn error_here(&self, message: impl Into<String>) -> FrontendError {
        let span = self.current().map(|token| token.span).unwrap_or(Span {
            start: self.source.len(),
            end: self.source.len(),
        });
        self.error_at(message, span)
    }

    fn error_at(&self, message: impl Into<String>, span: Span) -> FrontendError {
        FrontendError::new(self.source_name, self.source, message, span)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE: &str = r#"
def Video
def Appearance
def Color
def SolidColor {
  color: Color
}: Appearance
def Animal {
  appearance: Appearance
}
let white: Color
let black: Color
let cat: Tag + Animal
let video_a: Video
stmt video_a has cat {
  appearance: SolidColor {
    color: white
  }
}
stmt video_a has cat {
  appearance: SolidColor {
    color: black
  }
}
"#;

    #[test]
    fn parses_the_documented_example() {
        let program = parse("example.mt", EXAMPLE).unwrap();
        assert_eq!(program.items.len(), 11);
        assert!(matches!(program.items[0], Item::TypeDefinition(_)));
        assert!(matches!(program.items[9], Item::Statement(_)));
        assert!(matches!(program.items[10], Item::Statement(_)));
    }

    #[test]
    fn parses_numeric_property_values() {
        let program = parse(
            "number.mt",
            "def Video\ndef Taggable: Tag\nlet video: Video\nlet tag: Taggable\nstmt video has tag { score: 2.5 }",
        )
        .unwrap();
        let Item::Statement(statement) = program.items.last().unwrap() else {
            panic!("expected statement");
        };
        assert!(matches!(statement.properties[0].value, ValueExpression::Number { value, .. } if value == 2.5));
    }

    #[test]
    fn reports_unclosed_blocks() {
        let error = parse("bad.mt", "def Color { value: Number").unwrap_err();
        assert!(error.to_string().contains("unterminated"));
    }
}
