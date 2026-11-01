use std::collections::HashSet;
use std::fmt;

use crate::{
    ast::{CompareOp, PropertyCondition, PropertyPredicate, QueryExpr, QueryValue},
    resolved::{InstanceId, PropertyId, ResolvedModel, Statement, Value},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryExecutionError(pub String);

impl fmt::Display for QueryExecutionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for QueryExecutionError {}

/// Evaluate a query to the set of subjects satisfying it.
///
/// A type query matches a subject when at least one statement attached to that
/// subject has a tag of the requested type and that same statement satisfies
/// every property condition in the type-query block.
pub fn evaluate(
    model: &ResolvedModel,
    expression: &QueryExpr,
) -> Result<HashSet<InstanceId>, QueryExecutionError> {
    match expression {
        QueryExpr::Type { name, conditions, .. } => {
            let type_id = model.types.iter()
                .find(|ty| ty.name == *name)
                .map(|ty| ty.id)
                .ok_or_else(|| QueryExecutionError(format!("unknown query type {name}")))?;
            let properties = conditions.iter()
                .map(|condition| resolve_condition_property(model, type_id, condition))
                .collect::<Result<Vec<_>, _>>()?;

            let mut result = HashSet::new();
            for statement in &model.statements {
                if !model.instance_has_type(statement.tag, type_id) {
                    continue;
                }
                let matches = conditions.iter().zip(properties.iter()).all(|(condition, property)| {
                    evaluate_condition(condition, *property, statement, model)
                });
                if matches {
                    result.insert(statement.subject);
                }
            }
            Ok(result)
        }
        QueryExpr::And(left, right) => {
            let left = evaluate(model, left)?;
            let right = evaluate(model, right)?;
            Ok(left.intersection(&right).copied().collect())
        }
        QueryExpr::Or(left, right) => {
            let mut left = evaluate(model, left)?;
            left.extend(evaluate(model, right)?);
            Ok(left)
        }
        QueryExpr::Difference(left, right) => {
            let left = evaluate(model, left)?;
            let right = evaluate(model, right)?;
            Ok(left.difference(&right).copied().collect())
        }
        QueryExpr::Not(inner) => {
            let excluded = evaluate(model, inner)?;
            Ok(subject_universe(model).difference(&excluded).copied().collect())
        }
    }
}

fn subject_universe(model: &ResolvedModel) -> HashSet<InstanceId> {
    model.statements.iter().map(|statement| statement.subject).collect()
}

fn resolve_condition_property(
    model: &ResolvedModel,
    type_id: crate::resolved::TypeId,
    condition: &PropertyCondition,
) -> Result<PropertyId, QueryExecutionError> {
    let mut current = Some(type_id);
    while let Some(id) = current {
        let ty = model.type_by_id(id)
            .ok_or_else(|| QueryExecutionError(format!("unknown type id {id:?}")))?;
        if let Some(property) = ty.properties.iter().find(|property| property.name == condition.name) {
            return Ok(property.id);
        }
        current = ty.parent;
    }
    Err(QueryExecutionError(format!(
        "property {} is not defined on query type {}",
        condition.name,
        model.type_by_id(type_id).map_or("<unknown>", |ty| ty.name.as_str())
    )))
}

fn evaluate_condition(
    condition: &PropertyCondition,
    property: PropertyId,
    statement: &Statement,
    model: &ResolvedModel,
) -> bool {
    let value = statement.properties.iter()
        .find(|entry| entry.property == property)
        .map(|entry| &entry.value);
    evaluate_predicate(&condition.predicate, value, model)
}

fn evaluate_predicate(
    predicate: &PropertyPredicate,
    actual: Option<&Value>,
    model: &ResolvedModel,
) -> bool {
    match predicate {
        PropertyPredicate::None => actual.is_none(),
        PropertyPredicate::And(left, right) => {
            evaluate_predicate(left, actual, model)
                && evaluate_predicate(right, actual, model)
        }
        PropertyPredicate::Or(left, right) => {
            evaluate_predicate(left, actual, model)
                || evaluate_predicate(right, actual, model)
        }
        PropertyPredicate::Compare { op, value, .. } => {
            let Some(actual) = actual else { return false; };
            match (actual, value) {
                (Value::Number(actual), QueryValue::Number { value: expected, .. }) => {
                    compare_number(*op, *actual, *expected)
                }
                (Value::Reference(actual), QueryValue::Reference { name, .. }) => {
                    let expected = model.instances.iter()
                        .find(|instance| instance.name == *name)
                        .map(|instance| instance.id);
                    expected.is_some_and(|expected| compare_reference(*op, *actual, expected))
                }
                _ => false,
            }
        }
    }
}

fn compare_number(op: CompareOp, actual: i64, expected: i64) -> bool {
    match op {
        CompareOp::Eq => actual == expected,
        CompareOp::Ne => actual != expected,
        CompareOp::Gt => actual > expected,
        CompareOp::Ge => actual >= expected,
        CompareOp::Lt => actual < expected,
        CompareOp::Le => actual <= expected,
    }
}

fn compare_reference(op: CompareOp, actual: InstanceId, expected: InstanceId) -> bool {
    match op {
        CompareOp::Eq => actual == expected,
        CompareOp::Ne => actual != expected,
        CompareOp::Gt | CompareOp::Ge | CompareOp::Lt | CompareOp::Le => false,
    }
}
