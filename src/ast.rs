use crate::{lexer::Span, numeric::NumericValue};

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub items: Vec<Item>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Item {
    TypeDefinition(TypeDefinition),
    InstanceDefinition(InstanceDefinition),
    Statement(Statement),
    ShowQuery(Query),
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
    Number { value: NumericValue, span: Span },
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

#[derive(Debug, Clone, PartialEq)]
pub struct Query {
    pub expression: QueryExpr,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum QueryExpr {
    Type { name: String, conditions: Vec<PropertyCondition>, span: Span },
    And(Box<QueryExpr>, Box<QueryExpr>),
    Or(Box<QueryExpr>, Box<QueryExpr>),
    Difference(Box<QueryExpr>, Box<QueryExpr>),
    Not(Box<QueryExpr>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct PropertyCondition {
    pub name: String,
    pub predicate: PropertyPredicate,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompareOp { Eq, Ne, Gt, Ge, Lt, Le }

#[derive(Debug, Clone, PartialEq)]
pub enum QueryValue {
    Reference { name: String, span: Span },
    Number { value: NumericValue, span: Span },
}

#[derive(Debug, Clone, PartialEq)]
pub enum PropertyPredicate {
    None,
    Compare { op: CompareOp, value: QueryValue, span: Span },
    Anonymous { type_name: String, conditions: Vec<PropertyCondition>, span: Span },
    And(Box<PropertyPredicate>, Box<PropertyPredicate>),
    Or(Box<PropertyPredicate>, Box<PropertyPredicate>),
}
