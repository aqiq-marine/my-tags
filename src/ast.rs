use crate::lexer::Span;

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub items: Vec<Item>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Item {
    TypeDefinition(TypeDefinition),
    InstanceDefinition(InstanceDefinition),
    Statement(Statement),
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypeDefinition {
    pub name: String,
    pub properties: Vec<PropertyDefinition>,
    pub parent: Option<String>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PropertyDefinition {
    pub name: String,
    pub value_type: TypeExpression,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypeExpression {
    pub terms: Vec<String>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct InstanceDefinition {
    pub name: String,
    pub instance_type: TypeExpression,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Statement {
    pub subject: String,
    pub tag: String,
    pub properties: Vec<PropertyAssignment>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PropertyAssignment {
    pub name: String,
    pub value: ValueExpression,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ValueExpression {
    Reference { name: String, span: Span },
    Number { value: f64, span: Span },
    AnonymousInstance {
        type_name: String,
        properties: Vec<PropertyAssignment>,
        span: Span,
    },
}

impl ValueExpression {
    pub fn span(&self) -> Span {
        match self {
            Self::Reference { span, .. }
            | Self::Number { span, .. }
            | Self::AnonymousInstance { span, .. } => *span,
        }
    }
}
