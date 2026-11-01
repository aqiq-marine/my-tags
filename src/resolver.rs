use std::{collections::HashMap, fmt};

use crate::{
    analyze,
    ast::{Item, Program, PropertyAssignment, TypeExpression, ValueExpression},
    diagnostic::FrontendError,
    numeric::NumericValue,
    resolved::{Instance, InstanceId, PropertyDefinition, PropertyId, PropertyValue, ResolvedModel, Statement, StatementId, Type, TypeId, Value},
};

#[derive(Debug)]
pub enum ResolveError {
    Validation(Vec<FrontendError>),
    AmbiguousProperty(String),
    MissingSymbol(String),
}
impl fmt::Display for ResolveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Validation(errors) => write!(f, "semantic validation failed ({} errors)", errors.len()),
            Self::AmbiguousProperty(name) => write!(f, "property {name} is ambiguous across the instance types"),
            Self::MissingSymbol(name) => write!(f, "internal resolution error: missing symbol {name}"),
        }
    }
}
impl std::error::Error for ResolveError {}

pub fn resolve(program: &Program, source_name: &str, source: &str) -> Result<ResolvedModel, ResolveError> {
    analyze(program, source_name, source).map_err(ResolveError::Validation)?;
    let mut model = ResolvedModel { types: Vec::new(), instances: Vec::new(), statements: Vec::new() };
    let mut type_ids = HashMap::new();

    let builtins = [
        ("Subject", None),
        ("Tag", None),
        ("Float", None),
        ("Int", Some("Float")),
        ("Nat", Some("Int")),
    ];
    for (name, _) in builtins {
        let id = TypeId(model.types.len() as u32);
        type_ids.insert(name.to_owned(), id);
        model.types.push(Type { id, name: name.into(), parent: None, properties: Vec::new(), builtin: true });
    }
    for (name, parent) in builtins {
        if let Some(parent) = parent {
            model.types[type_ids[name].0 as usize].parent = Some(type_ids[parent]);
        }
    }

    for item in &program.items {
        if let Item::TypeDefinition(def) = item {
            let id = TypeId(model.types.len() as u32);
            type_ids.insert(def.name.clone(), id);
            model.types.push(Type { id, name: def.name.clone(), parent: None, properties: Vec::new(), builtin: false });
        }
    }
    for item in &program.items {
        if let Item::TypeDefinition(def) = item {
            let id = type_ids[&def.name];
            let parent = def.parent.as_ref().map(|name| type_ids[name]);
            let properties = def.properties.iter().enumerate().map(|(index, property)| PropertyDefinition {
                id: PropertyId { owner: id, index: index as u32 },
                name: property.name.clone(),
                value_type: resolve_type_expression(&property.value_type, &type_ids),
            }).collect();
            model.types[id.0 as usize].parent = parent;
            model.types[id.0 as usize].properties = properties;
        }
    }

    let mut instance_ids = HashMap::new();
    for item in &program.items {
        if let Item::InstanceDefinition(def) = item {
            let id = InstanceId(model.instances.len() as u32);
            instance_ids.insert(def.name.clone(), id);
            model.instances.push(Instance { id, name: def.name.clone(), types: resolve_type_expression(&def.instance_type, &type_ids) });
        }
    }
    for item in &program.items {
        if let Item::Statement(stmt) = item {
            let subject = instance_ids[&stmt.subject];
            let tag = instance_ids[&stmt.tag];
            let owner_types = model.instance_by_id(tag).expect("validated instance").types.clone();
            let properties = resolve_assignments(&stmt.properties, &owner_types, &model, &type_ids, &instance_ids)?;
            let id = StatementId(model.statements.len() as u32);
            model.statements.push(Statement { id, subject, tag, properties });
        }
    }
    Ok(model)
}

fn resolve_type_expression(expr: &TypeExpression, ids: &HashMap<String, TypeId>) -> Vec<TypeId> {
    expr.terms.iter().map(|name| ids[name]).collect()
}

fn find_property(model: &ResolvedModel, owner_types: &[TypeId], name: &str) -> Result<PropertyId, ResolveError> {
    let mut found = Vec::new();
    for owner in owner_types {
        let mut current = Some(*owner);
        while let Some(type_id) = current {
            let ty = model.type_by_id(type_id).expect("validated type id");
            if let Some(property) = ty.properties.iter().find(|property| property.name == name) {
                found.push(property.id);
                break;
            }
            current = ty.parent;
        }
    }
    found.sort_unstable();
    found.dedup();
    match found.as_slice() {
        [property] => Ok(*property),
        [] => Err(ResolveError::MissingSymbol(name.to_owned())),
        _ => Err(ResolveError::AmbiguousProperty(name.to_owned())),
    }
}

fn resolve_assignments(
    assignments: &[PropertyAssignment],
    owner_types: &[TypeId],
    model: &ResolvedModel,
    type_ids: &HashMap<String, TypeId>,
    instance_ids: &HashMap<String, InstanceId>,
) -> Result<Vec<PropertyValue>, ResolveError> {
    assignments.iter().map(|assignment| {
        let property = find_property(model, owner_types, &assignment.name)?;
        let definition = model.type_by_id(property.owner)
            .and_then(|ty| ty.properties.iter().find(|candidate| candidate.id == property))
            .expect("validated property definition");
        let value = resolve_value(&assignment.value, &definition.value_type, model, type_ids, instance_ids)?;
        Ok(PropertyValue { property, value })
    }).collect()
}

fn resolve_value(
    expression: &ValueExpression,
    expected_types: &[TypeId],
    model: &ResolvedModel,
    type_ids: &HashMap<String, TypeId>,
    instance_ids: &HashMap<String, InstanceId>,
) -> Result<Value, ResolveError> {
    match expression {
        ValueExpression::Reference { name, .. } => instance_ids.get(name).copied().map(Value::Reference).ok_or_else(|| ResolveError::MissingSymbol(name.clone())),
        ValueExpression::Number { value, .. } => {
            let expected_names: Vec<_> = expected_types.iter()
                .filter_map(|id| model.type_by_id(*id).map(|ty| ty.name.as_str()))
                .collect();
            let value = if expected_names.contains(&"Nat") {
                match value {
                    NumericValue::Nat(number) => NumericValue::Nat(*number),
                    _ => unreachable!("semantic validation rejects non-Nat values for Nat properties"),
                }
            } else if expected_names.contains(&"Int") {
                match value {
                    NumericValue::Nat(number) => NumericValue::Int(i128::from(*number)),
                    NumericValue::Int(number) => NumericValue::Int(*number),
                    NumericValue::Float(_) => unreachable!("semantic validation rejects Float-to-Int narrowing"),
                }
            } else if expected_names.contains(&"Float") {
                NumericValue::Float(value.as_f64())
            } else {
                *value
            };
            Ok(Value::Number(value))
        }
        ValueExpression::AnonymousInstance { type_name, properties, .. } => {
            let type_id = type_ids.get(type_name).copied().ok_or_else(|| ResolveError::MissingSymbol(type_name.clone()))?;
            let nested = resolve_assignments(properties, &[type_id], model, type_ids, instance_ids)?;
            Ok(Value::Anonymous { type_id, properties: nested })
        }
    }
}
