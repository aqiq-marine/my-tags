use crate::{
    ast::{
        CompareOp, ImportDeclaration, InstanceDefinition, Item, Program, PropertyAssignment, PropertyCondition,
        PropertyDefinition, PropertyPredicate, Query, QueryExpr, QueryValue, Statement,
        TypeDefinition, TypeExpression, ValueExpression,
    },
    diagnostic::FrontendError,
    lexer::{lex, Span, Token, TokenKind},
    numeric::NumericValue,
};

pub fn parse(source_name: &str, source: &str) -> Result<Program, FrontendError> {
    let tokens = lex(source_name, source)?;
    Parser { source_name, source, tokens, cursor: 0 }.parse_program()
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
        let mut saw_non_import = false;
        while let Some(kind) = self.current().map(|token| token.kind.clone()) {
            let item = match kind {
                TokenKind::Import if !saw_non_import => Item::Import(self.parse_import()?),
                TokenKind::Import => return Err(self.error_here("imports must appear before declarations")),
                TokenKind::Def => { saw_non_import = true; Item::TypeDefinition(self.parse_type_definition()?) },
                TokenKind::Let => { saw_non_import = true; Item::InstanceDefinition(self.parse_instance_definition()?) },
                TokenKind::Stmt => { saw_non_import = true; Item::Statement(self.parse_statement()?) },
                TokenKind::Hash => { saw_non_import = true; Item::ShowQuery(self.parse_show_query()?) },
                _ => return Err(self.error_here("expected an import, declaration, statement, or #show query")),
            };
            items.push(item);
        }
        Ok(Program { items })
    }

    fn parse_import(&mut self) -> Result<ImportDeclaration, FrontendError> {
        let start = self.expect_simple(|kind| matches!(kind, TokenKind::Import), "import")?.span;
        let (path, path_span) = match self.current().cloned() {
            Some(Token { kind: TokenKind::String(path), span }) => { self.cursor += 1; (path, span) }
            _ => return Err(self.error_here("expected a quoted import path")),
        };
        self.expect_simple(|kind| matches!(kind, TokenKind::As), "as")?;
        let (alias, alias_span) = self.expect_identifier()?;
        if alias.contains("::") {
            return Err(self.error_at("import alias must be a simple identifier", alias_span));
        }
        Ok(ImportDeclaration {
            path,
            alias,
            span: Span { start: start.start, end: alias_span.end.max(path_span.end) },
        })
    }

    fn parse_type_definition(&mut self) -> Result<TypeDefinition, FrontendError> {
        let start = self.expect_simple(|kind| matches!(kind, TokenKind::Def), "def")?.span;
        let (name, name_span) = self.expect_local_identifier()?;

        // Accept both parent-before-properties and properties-before-parent forms.
        let parent_before = if self.consume_simple(|kind| matches!(kind, TokenKind::Colon)).is_some() {
            Some(self.expect_identifier()?.0)
        } else {
            None
        };
        let properties = if self.check(|kind| matches!(kind, TokenKind::LeftBrace)) {
            self.parse_property_definitions()?
        } else {
            Vec::new()
        };
        let parent_after = if parent_before.is_none()
            && self.consume_simple(|kind| matches!(kind, TokenKind::Colon)).is_some()
        {
            Some(self.expect_identifier()?.0)
        } else {
            None
        };
        let parent = parent_before.or(parent_after);
        let end = self.previous_span().unwrap_or(name_span);
        Ok(TypeDefinition {
            name, properties, parent,
            span: Span { start: start.start, end: end.end },
        })
    }

    fn parse_property_definitions(&mut self) -> Result<Vec<PropertyDefinition>, FrontendError> {
        self.expect_simple(|kind| matches!(kind, TokenKind::LeftBrace), "{")?;
        let mut properties = Vec::new();
        while !self.check(|kind| matches!(kind, TokenKind::RightBrace)) {
            if self.current().is_none() {
                return Err(self.error_here("unterminated property block; expected }"));
            }
            let (name, name_span) = self.expect_local_identifier()?;
            self.expect_simple(|kind| matches!(kind, TokenKind::Colon), ":")?;
            let value_type = self.parse_type_expression()?;
            let end = value_type.span;
            properties.push(PropertyDefinition {
                name, value_type,
                span: Span { start: name_span.start, end: end.end },
            });
            self.consume_simple(|kind| matches!(kind, TokenKind::Comma));
        }
        self.expect_simple(|kind| matches!(kind, TokenKind::RightBrace), "}")?;
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
        let start = self.expect_simple(|kind| matches!(kind, TokenKind::Let), "let")?.span;
        let (name, _) = self.expect_local_identifier()?;
        self.expect_simple(|kind| matches!(kind, TokenKind::Colon), ":")?;
        let instance_type = self.parse_type_expression()?;
        Ok(InstanceDefinition {
            name,
            span: Span { start: start.start, end: instance_type.span.end },
            instance_type,
        })
    }

    fn parse_statement(&mut self) -> Result<Statement, FrontendError> {
        let start = self.expect_simple(|kind| matches!(kind, TokenKind::Stmt), "stmt")?.span;
        let (subject, _) = self.expect_identifier()?;
        self.expect_simple(|kind| matches!(kind, TokenKind::Has), "has")?;
        let (tag, tag_span) = self.expect_identifier()?;
        let properties = if self.check(|kind| matches!(kind, TokenKind::LeftBrace)) {
            self.parse_property_assignments()?
        } else {
            Vec::new()
        };
        let end = self.previous_span().unwrap_or(tag_span);
        Ok(Statement { subject, tag, properties, span: Span { start: start.start, end: end.end } })
    }

    fn parse_property_assignments(&mut self) -> Result<Vec<PropertyAssignment>, FrontendError> {
        self.expect_simple(|kind| matches!(kind, TokenKind::LeftBrace), "{")?;
        let mut properties = Vec::new();
        while !self.check(|kind| matches!(kind, TokenKind::RightBrace)) {
            if self.current().is_none() {
                return Err(self.error_here("unterminated property block; expected }"));
            }
            let (name, name_span) = self.expect_local_identifier()?;
            self.expect_simple(|kind| matches!(kind, TokenKind::Colon), ":")?;
            let value = self.parse_value_expression()?;
            let end = value.span();
            properties.push(PropertyAssignment {
                name, value,
                span: Span { start: name_span.start, end: end.end },
            });
            self.consume_simple(|kind| matches!(kind, TokenKind::Comma));
        }
        self.expect_simple(|kind| matches!(kind, TokenKind::RightBrace), "}")?;
        Ok(properties)
    }

    fn parse_value_expression(&mut self) -> Result<ValueExpression, FrontendError> {
        if let Some((value, span)) = self.parse_signed_number()? {
            return Ok(ValueExpression::Number { value, span });
        }
        let (name, name_span) = self.expect_identifier()?;
        if self.check(|kind| matches!(kind, TokenKind::LeftBrace)) {
            let properties = self.parse_property_assignments()?;
            let end = self.previous_span().unwrap_or(name_span);
            Ok(ValueExpression::AnonymousInstance {
                type_name: name, properties,
                span: Span { start: name_span.start, end: end.end },
            })
        } else {
            Ok(ValueExpression::Reference { name, span: name_span })
        }
    }

    fn parse_show_query(&mut self) -> Result<Query, FrontendError> {
        let start = self.expect_simple(|kind| matches!(kind, TokenKind::Hash), "#")?.span;
        self.expect_simple(|kind| matches!(kind, TokenKind::Show), "show")?;
        self.expect_simple(|kind| matches!(kind, TokenKind::Query), "query")?;
        let expression = self.parse_query_or()?;
        let end = self.previous_span().unwrap_or(start);
        Ok(Query { expression, span: Span { start: start.start, end: end.end } })
    }

    // Set precedence: unary !, then & and -, then |. & and - associate left-to-right.
    fn parse_query_or(&mut self) -> Result<QueryExpr, FrontendError> {
        let mut expression = self.parse_query_intersection()?;
        while self.consume_simple(|kind| matches!(kind, TokenKind::Pipe)).is_some() {
            let right = self.parse_query_intersection()?;
            expression = QueryExpr::Or(Box::new(expression), Box::new(right));
        }
        Ok(expression)
    }

    fn parse_query_intersection(&mut self) -> Result<QueryExpr, FrontendError> {
        let mut expression = self.parse_query_unary()?;
        loop {
            if self.consume_simple(|kind| matches!(kind, TokenKind::Ampersand)).is_some() {
                let right = self.parse_query_unary()?;
                expression = QueryExpr::And(Box::new(expression), Box::new(right));
            } else if self.consume_simple(|kind| matches!(kind, TokenKind::Minus)).is_some() {
                let right = self.parse_query_unary()?;
                expression = QueryExpr::Difference(Box::new(expression), Box::new(right));
            } else {
                break;
            }
        }
        Ok(expression)
    }

    fn parse_query_unary(&mut self) -> Result<QueryExpr, FrontendError> {
        if self.consume_simple(|kind| matches!(kind, TokenKind::Bang)).is_some() {
            return Ok(QueryExpr::Not(Box::new(self.parse_query_unary()?)));
        }
        self.parse_query_primary()
    }

    fn parse_query_primary(&mut self) -> Result<QueryExpr, FrontendError> {
        if self.consume_simple(|kind| matches!(kind, TokenKind::LeftParen)).is_some() {
            let expression = self.parse_query_or()?;
            self.expect_simple(|kind| matches!(kind, TokenKind::RightParen), ")")?;
            return Ok(expression);
        }

        let (name, name_span) = self.expect_identifier()?;
        self.expect_simple(|kind| matches!(kind, TokenKind::LeftBrace), "{")?;
        let mut conditions = Vec::new();
        while !self.check(|kind| matches!(kind, TokenKind::RightBrace)) {
            if self.current().is_none() {
                return Err(self.error_here("unterminated query type block; expected }"));
            }
            let (property, property_span) = self.expect_local_identifier()?;
            self.expect_simple(|kind| matches!(kind, TokenKind::Colon), ":")?;
            let predicate = self.parse_query_predicate_or()?;
            let end = self.previous_span().unwrap_or(property_span);
            conditions.push(PropertyCondition {
                name: property, predicate,
                span: Span { start: property_span.start, end: end.end },
            });
            self.consume_simple(|kind| matches!(kind, TokenKind::Comma));
        }
        let end = self.expect_simple(|kind| matches!(kind, TokenKind::RightBrace), "}")?.span;
        Ok(QueryExpr::Type {
            name, conditions,
            span: Span { start: name_span.start, end: end.end },
        })
    }

    fn parse_query_predicate_or(&mut self) -> Result<PropertyPredicate, FrontendError> {
        let mut predicate = self.parse_query_predicate_and()?;
        while self.consume_simple(|kind| matches!(kind, TokenKind::OrOr)).is_some() {
            let right = self.parse_query_predicate_and()?;
            predicate = PropertyPredicate::Or(Box::new(predicate), Box::new(right));
        }
        Ok(predicate)
    }

    fn parse_query_predicate_and(&mut self) -> Result<PropertyPredicate, FrontendError> {
        let mut predicate = self.parse_query_predicate_atom()?;
        while self.consume_simple(|kind| matches!(kind, TokenKind::AndAnd)).is_some() {
            let right = self.parse_query_predicate_atom()?;
            predicate = PropertyPredicate::And(Box::new(predicate), Box::new(right));
        }
        Ok(predicate)
    }

    fn parse_query_predicate_atom(&mut self) -> Result<PropertyPredicate, FrontendError> {
        if matches!(self.current().map(|token| &token.kind), Some(TokenKind::Identifier(_))) {
            let saved = self.cursor;
            let (type_name, type_span) = self.expect_identifier()?;
            if self.check(|kind| matches!(kind, TokenKind::LeftBrace)) {
                self.expect_simple(|kind| matches!(kind, TokenKind::LeftBrace), "{")?;
                let mut conditions = Vec::new();
                while !self.check(|kind| matches!(kind, TokenKind::RightBrace)) {
                    if self.current().is_none() {
                        return Err(self.error_here("unterminated nested query block; expected }"));
                    }
                    let (property, property_span) = self.expect_local_identifier()?;
                    self.expect_simple(|kind| matches!(kind, TokenKind::Colon), ":")?;
                    let predicate = self.parse_query_predicate_or()?;
                    let end = self.previous_span().unwrap_or(property_span);
                    conditions.push(PropertyCondition {
                        name: property,
                        predicate,
                        span: Span { start: property_span.start, end: end.end },
                    });
                    self.consume_simple(|kind| matches!(kind, TokenKind::Comma));
                }
                let end = self.expect_simple(|kind| matches!(kind, TokenKind::RightBrace), "}")?.span;
                return Ok(PropertyPredicate::Anonymous {
                    type_name,
                    conditions,
                    span: Span { start: type_span.start, end: end.end },
                });
            }
            self.cursor = saved;
        }

        if matches!(self.current().map(|token| &token.kind), Some(TokenKind::Identifier(name)) if name != "none")
            && !matches!(self.tokens.get(self.cursor + 1).map(|token| &token.kind), Some(TokenKind::LeftParen))
        {
            let (name, span) = self.expect_identifier()?;
            return Ok(PropertyPredicate::Compare {
                op: CompareOp::Eq,
                value: QueryValue::Reference { name, span },
                span,
            });
        }

        self.parse_property_atom()
    }

    fn parse_property_atom(&mut self) -> Result<PropertyPredicate, FrontendError> {
        if let Some(Token { kind: TokenKind::Identifier(name), .. }) = self.current().cloned()
            && name == "none"
        {
            self.cursor += 1;
            return Ok(PropertyPredicate::None);
        }
        let start = self.expect_simple(|kind| matches!(kind, TokenKind::LeftParen), "(")?.span;
        let op = match self.current().map(|token| &token.kind) {
            Some(TokenKind::EqualEqual) => CompareOp::Eq,
            Some(TokenKind::NotEqual) => CompareOp::Ne,
            Some(TokenKind::Greater) => CompareOp::Gt,
            Some(TokenKind::GreaterEqual) => CompareOp::Ge,
            Some(TokenKind::Less) => CompareOp::Lt,
            Some(TokenKind::LessEqual) => CompareOp::Le,
            _ => return Err(self.error_here("expected a comparison operator")),
        };
        self.cursor += 1;
        let value = if let Some((value, span)) = self.parse_signed_number()? {
            QueryValue::Number { value, span }
        } else {
            let (name, span) = self.expect_identifier()?;
            QueryValue::Reference { name, span }
        };
        let end = self.expect_simple(|kind| matches!(kind, TokenKind::RightParen), ")")?.span;
        Ok(PropertyPredicate::Compare {
            op, value, span: Span { start: start.start, end: end.end },
        })
    }

    fn parse_signed_number(&mut self) -> Result<Option<(NumericValue, Span)>, FrontendError> {
        let minus = self.consume_simple(|kind| matches!(kind, TokenKind::Minus));
        if let Some(Token { kind: TokenKind::Float(raw), span }) = self.current().cloned() {
            self.cursor += 1;
            let mut value = raw.parse::<f64>().map_err(|_| {
                self.error_at("invalid floating-point literal", span)
            })?;
            if minus.is_some() { value = -value; }
            if !value.is_finite() {
                return Err(self.error_at("Float literals must be finite", span));
            }
            return Ok(Some((NumericValue::Float(value), Span {
                start: minus.map_or(span.start, |token| token.span.start),
                end: span.end,
            })));
        }
        if let Some(Token { kind: TokenKind::Integer(raw), span }) = self.current().cloned() {
            self.cursor += 1;
            let magnitude = raw.parse::<u128>().map_err(|_| {
                self.error_at("integer literal is outside the supported range", span)
            })?;
            let value = if minus.is_some() {
                if magnitude == (i128::MAX as u128) + 1 {
                    NumericValue::Int(i128::MIN)
                } else {
                    let positive = i128::try_from(magnitude).map_err(|_| {
                        self.error_at("negative integer literal is outside the i128 range", span)
                    })?;
                    NumericValue::Int(-positive)
                }
            } else {
                let natural = u64::try_from(magnitude).map_err(|_| {
                    self.error_at("Nat literal is outside the u64 range", span)
                })?;
                NumericValue::Nat(natural)
            };
            return Ok(Some((value, Span {
                start: minus.map_or(span.start, |token| token.span.start),
                end: span.end,
            })));
        }
        if let Some(token) = minus {
            return Err(self.error_at("expected a numeric literal after -", token.span));
        }
        Ok(None)
    }

    fn current(&self) -> Option<&Token> { self.tokens.get(self.cursor) }

    fn previous_span(&self) -> Option<Span> {
        self.cursor.checked_sub(1)
            .and_then(|index| self.tokens.get(index))
            .map(|token| token.span)
    }

    fn check(&self, predicate: impl FnOnce(&TokenKind) -> bool) -> bool {
        self.current().is_some_and(|token| predicate(&token.kind))
    }

    fn consume_simple(&mut self, predicate: impl FnOnce(&TokenKind) -> bool) -> Option<Token> {
        if self.check(predicate) {
            let token = self.tokens.get(self.cursor).cloned();
            if token.is_some() { self.cursor += 1; }
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

    fn expect_local_identifier(&mut self) -> Result<(String, Span), FrontendError> {
        let (name, span) = self.expect_identifier()?;
        if name.contains("::") {
            return Err(self.error_at("expected an unqualified identifier", span));
        }
        Ok((name, span))
    }

    fn expect_identifier(&mut self) -> Result<(String, Span), FrontendError> {
        let (mut name, first_span) = match self.current().cloned() {
            Some(Token { kind: TokenKind::Identifier(name), span }) => {
                self.cursor += 1;
                (name, span)
            }
            _ => return Err(self.error_here("expected an identifier")),
        };
        let mut end = first_span;
        while self.consume_simple(|kind| matches!(kind, TokenKind::ColonColon)).is_some() {
            match self.current().cloned() {
                Some(Token { kind: TokenKind::Identifier(part), span }) => {
                    self.cursor += 1;
                    name.push_str("::");
                    name.push_str(&part);
                    end = span;
                }
                _ => return Err(self.error_here("expected an identifier after ::")),
            }
        }
        Ok((name, Span { start: first_span.start, end: end.end }))
    }

    fn error_here(&self, message: impl Into<String>) -> FrontendError {
        let span = self.current().map(|token| token.span).unwrap_or(Span {
            start: self.source.len(), end: self.source.len(),
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
    fn parses_signed_integer_property_values() {
        let program = parse(
            "number.mt",
            "def Video\ndef Taggable: Tag\nlet video: Video\nlet tag: Taggable\nstmt video has tag { score: -25 }",
        ).unwrap();
        let Item::Statement(statement) = program.items.last().unwrap() else { panic!("expected statement"); };
        assert!(matches!(statement.properties[0].value, ValueExpression::Number { value, .. } if value == NumericValue::Int(-25)));
    }

    #[test]
    fn parses_i128_minimum_and_rejects_overflow() {
        let minimum = parse(
            "number.mt",
            "def Video\ndef Taggable: Tag\nlet video: Video\nlet tag: Taggable\nstmt video has tag { score: -170141183460469231731687303715884105728 }",
        ).unwrap();
        let Item::Statement(statement) = minimum.items.last().unwrap() else { panic!("expected statement"); };
        assert!(matches!(statement.properties[0].value, ValueExpression::Number { value: NumericValue::Int(i128::MIN), .. }));

        let error = parse(
            "number.mt",
            "def Video\ndef Taggable: Tag\nlet video: Video\nlet tag: Taggable\nstmt video has tag { score: -170141183460469231731687303715884105729 }",
        ).unwrap_err();
        assert!(error.to_string().contains("i128 range"));
    }

    #[test]
    fn rejects_non_finite_float_literals() {
        let error = parse(
            "float.mt",
            "def Video: Subject\ndef Taggable: Tag\nlet video: Video\nlet tag: Taggable\nstmt video has tag { score: 1e999 }",
        ).unwrap_err();
        assert!(error.to_string().contains("must be finite"));
    }

    #[test]
    fn parses_query_with_set_and_property_logic() {
        let source = r#"
#show query Reaction {
  yield: (>= 80) && (< 95)
  solvent: (== p1) || (== p2)
} & Hoge {
  huga: none
} - !Asymmetric {}
"#;
        let program = parse("query.mt", source).unwrap();
        assert!(matches!(program.items[0], Item::ShowQuery(_)));
    }

    #[test]
    fn accepts_parent_before_or_after_property_block() {
        assert!(parse("type.mt", "def Child: Parent { value: Int }").is_ok());
        assert!(parse("type.mt", "def Child { value: Int }: Parent").is_ok());
    }

    #[test]
    fn reports_unclosed_blocks() {
        let error = parse("bad.mt", "def Color { value: Int").unwrap_err();
        assert!(error.to_string().contains("unterminated"));
    }
    #[test]
    fn parses_imports_and_qualified_names() {
        let program = parse(
            "main.mt",
            "import \"chemistry.mt\" as chem\nlet sample: chem::Molecule\n#show query chem::Molecule { atom: chem::atom }",
        ).unwrap();
        assert!(matches!(&program.items[0], Item::Import(import) if import.path == "chemistry.mt" && import.alias == "chem"));
        let Item::InstanceDefinition(instance) = &program.items[1] else { panic!("expected instance"); };
        assert_eq!(instance.instance_type.terms, ["chem::Molecule"]);
        let Item::ShowQuery(query) = &program.items[2] else { panic!("expected query"); };
        assert!(matches!(&query.expression, QueryExpr::Type { name, .. } if name == "chem::Molecule"));
    }

    #[test]
    fn rejects_qualified_declaration_names() {
        assert!(parse("bad.mt", "def chemistry::Atom").is_err());
        assert!(parse("bad.mt", "let chemistry::atom: Atom").is_err());
    }

    #[test]
    fn requires_imports_to_precede_other_items() {
        let error = parse("late-import.mt", "def Atom\nimport \"chemistry.mt\" as chem").unwrap_err();
        assert!(error.to_string().contains("imports must appear before declarations"));
    }

}
