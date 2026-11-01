use std::collections::HashSet;

use crate::numeric::NumericValue;
use crate::resolved::{
    Instance, InstanceId, PropertyId, PropertyValue, ResolvedModel, Statement, TypeId, Value,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PropertyPath {
    pub properties: Vec<PropertyId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PropertyPathError {
    Empty,
    UnknownProperty(PropertyId),
    InvalidTransition { from: PropertyId, to: PropertyId },
}

impl std::fmt::Display for PropertyPathError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => write!(f, "property path must not be empty"),
            Self::UnknownProperty(id) => write!(f, "unknown property {id:?}"),
            Self::InvalidTransition { from, to } => {
                write!(f, "property {to:?} is not available on a value type of {from:?}")
            }
        }
    }
}
impl std::error::Error for PropertyPathError {}

impl PropertyPath {
    pub fn new(root: PropertyId) -> Self {
        Self { properties: vec![root] }
    }

    pub fn then(mut self, property: PropertyId) -> Self {
        self.properties.push(property);
        self
    }

    pub fn resolve(
        model: &ResolvedModel,
        root: PropertyId,
        nested: &[PropertyId],
    ) -> Result<Self, PropertyPathError> {
        let path = Self {
            properties: std::iter::once(root).chain(nested.iter().copied()).collect(),
        };
        path.validate(model)?;
        Ok(path)
    }

    pub fn validate(&self, model: &ResolvedModel) -> Result<(), PropertyPathError> {
        let Some(first) = self.properties.first().copied() else {
            return Err(PropertyPathError::Empty);
        };
        if property_definition(model, first).is_none() {
            return Err(PropertyPathError::UnknownProperty(first));
        }

        for pair in self.properties.windows(2) {
            let from = pair[0];
            let to = pair[1];
            let definition = property_definition(model, from)
                .ok_or(PropertyPathError::UnknownProperty(from))?;
            let valid = definition.value_type.iter().any(|type_id| {
                model.is_subtype_of(to.owner, *type_id)
                    || model.is_subtype_of(*type_id, to.owner)
            }) && property_definition(model, to).is_some();
            if !valid {
                return Err(PropertyPathError::InvalidTransition { from, to });
            }
        }
        Ok(())
    }
}

fn property_definition(
    model: &ResolvedModel,
    id: PropertyId,
) -> Option<&crate::resolved::PropertyDefinition> {
    model.type_by_id(id.owner)?.properties.iter().find(|property| property.id == id)
}

fn value_at_path<'a>(values: &'a [PropertyValue], path: &[PropertyId]) -> Option<&'a Value> {
    let (first, rest) = path.split_first()?;
    let value = &values.iter().find(|entry| entry.property == *first)?.value;
    if rest.is_empty() {
        return Some(value);
    }
    match value {
        Value::Anonymous { properties, .. } => value_at_path(properties, rest),
        Value::Reference(_) | Value::Number(_) => None,
    }
}

#[derive(Clone, Copy)]
enum NumberComparison { Eq, Ne, Gt, Ge, Lt, Le }

impl NumberComparison {
    fn matches(self, actual: NumericValue, expected: NumericValue) -> bool {
        let Some(ordering) = actual.partial_cmp_numeric(expected) else { return false; };
        match self {
            Self::Eq => ordering.is_eq(),
            Self::Ne => !ordering.is_eq(),
            Self::Gt => ordering.is_gt(),
            Self::Ge => ordering.is_ge(),
            Self::Lt => ordering.is_lt(),
            Self::Le => ordering.is_le(),
        }
    }
}

pub struct Query<'a> {
    model: &'a ResolvedModel,
    statements: Vec<&'a Statement>,
}

impl<'a> Query<'a> {
    pub fn new(model: &'a ResolvedModel) -> Self {
        Self { model, statements: model.statements.iter().collect() }
    }

    pub fn statements(self) -> Self { self }

    pub fn where_subject(mut self, subject: InstanceId) -> Self {
        self.statements.retain(|statement| statement.subject == subject);
        self
    }

    pub fn where_tag(mut self, tag: InstanceId) -> Self {
        self.statements.retain(|statement| statement.tag == tag);
        self
    }

    pub fn where_tag_is_subtype_of(mut self, expected: TypeId) -> Self {
        let model = self.model;
        self.statements.retain(|statement| model.instance_has_type(statement.tag, expected));
        self
    }

    pub fn where_property_exists(self, property: PropertyId) -> Self {
        self.where_property_path_exists(&PropertyPath::new(property))
    }

    #[deprecated(note = "use where_property_exists")]
    pub fn where_property(self, property: PropertyId) -> Self {
        self.where_property_exists(property)
    }

    pub fn where_property_path_exists(mut self, path: &PropertyPath) -> Self {
        self.statements.retain(|statement| {
            value_at_path(&statement.properties, &path.properties).is_some()
        });
        self
    }

    pub fn where_property_reference_equals(self, property: PropertyId, expected: InstanceId) -> Self {
        self.where_property_path_reference_equals(&PropertyPath::new(property), expected)
    }

    pub fn where_property_path_reference_equals(
        mut self,
        path: &PropertyPath,
        expected: InstanceId,
    ) -> Self {
        self.statements.retain(|statement| {
            matches!(
                value_at_path(&statement.properties, &path.properties),
                Some(Value::Reference(actual)) if *actual == expected
            )
        });
        self
    }

    pub fn where_property_number_eq(self, property: PropertyId, expected: impl Into<NumericValue>) -> Self {
        self.where_number(&PropertyPath::new(property), expected.into(), NumberComparison::Eq)
    }
    pub fn where_property_number_ne(self, property: PropertyId, expected: impl Into<NumericValue>) -> Self {
        self.where_number(&PropertyPath::new(property), expected.into(), NumberComparison::Ne)
    }
    pub fn where_property_number_gt(self, property: PropertyId, expected: impl Into<NumericValue>) -> Self {
        self.where_number(&PropertyPath::new(property), expected.into(), NumberComparison::Gt)
    }
    pub fn where_property_number_ge(self, property: PropertyId, expected: impl Into<NumericValue>) -> Self {
        self.where_number(&PropertyPath::new(property), expected.into(), NumberComparison::Ge)
    }
    pub fn where_property_number_lt(self, property: PropertyId, expected: impl Into<NumericValue>) -> Self {
        self.where_number(&PropertyPath::new(property), expected.into(), NumberComparison::Lt)
    }
    pub fn where_property_number_le(self, property: PropertyId, expected: impl Into<NumericValue>) -> Self {
        self.where_number(&PropertyPath::new(property), expected.into(), NumberComparison::Le)
    }

    pub fn where_property_path_number_eq(self, path: &PropertyPath, expected: impl Into<NumericValue>) -> Self {
        self.where_number(path, expected.into(), NumberComparison::Eq)
    }
    pub fn where_property_path_number_ne(self, path: &PropertyPath, expected: impl Into<NumericValue>) -> Self {
        self.where_number(path, expected.into(), NumberComparison::Ne)
    }
    pub fn where_property_path_number_gt(self, path: &PropertyPath, expected: impl Into<NumericValue>) -> Self {
        self.where_number(path, expected.into(), NumberComparison::Gt)
    }
    pub fn where_property_path_number_ge(self, path: &PropertyPath, expected: impl Into<NumericValue>) -> Self {
        self.where_number(path, expected.into(), NumberComparison::Ge)
    }
    pub fn where_property_path_number_lt(self, path: &PropertyPath, expected: impl Into<NumericValue>) -> Self {
        self.where_number(path, expected.into(), NumberComparison::Lt)
    }
    pub fn where_property_path_number_le(self, path: &PropertyPath, expected: impl Into<NumericValue>) -> Self {
        self.where_number(path, expected.into(), NumberComparison::Le)
    }

    fn where_number(mut self, path: &PropertyPath, expected: NumericValue, comparison: NumberComparison) -> Self {
        self.statements.retain(|statement| {
            matches!(
                value_at_path(&statement.properties, &path.properties),
                Some(Value::Number(actual)) if comparison.matches(*actual, expected)
            )
        });
        self
    }

    pub fn where_property_value_is_subtype_of(self, property: PropertyId, expected: TypeId) -> Self {
        self.where_property_path_value_is_subtype_of(&PropertyPath::new(property), expected)
    }

    pub fn where_property_path_value_is_subtype_of(
        mut self,
        path: &PropertyPath,
        expected: TypeId,
    ) -> Self {
        let model = self.model;
        self.statements.retain(|statement| {
            matches!(
                value_at_path(&statement.properties, &path.properties),
                Some(Value::Anonymous { type_id, .. }) if model.is_subtype_of(*type_id, expected)
            )
        });
        self
    }

    pub fn collect(self) -> Vec<&'a Statement> { self.statements }

    pub fn subjects(self) -> Vec<&'a Instance> {
        let mut seen = HashSet::new();
        self.statements.into_iter().filter_map(|statement| {
            if seen.insert(statement.subject) {
                self.model.instance_by_id(statement.subject)
            } else {
                None
            }
        }).collect()
    }
}

impl ResolvedModel {
    pub fn query(&self) -> Query<'_> { Query::new(self) }
}
