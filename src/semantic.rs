use std::collections::HashSet;

use indexmap::IndexMap;

use crate::{
    ast::{CompareOp, InstanceDefinition, Item, Program, PropertyAssignment, PropertyPredicate, QueryExpr, QueryValue, TypeDefinition, TypeExpression, ValueExpression},
    diagnostic::FrontendError,
    lexer::Span,
};

fn issue(name: &str, source: &str, message: impl Into<String>, span: Span) -> FrontendError {
    FrontendError::new(name, source, message, span)
}

pub fn analyze(program: &Program, name: &str, source: &str) -> Result<(), Vec<FrontendError>> {
    let mut errors = Vec::new();
    let mut types: IndexMap<String, TypeDefinition> = IndexMap::new();
    let mut instances: IndexMap<String, InstanceDefinition> = IndexMap::new();

    for item in &program.items {
        match item {
            Item::TypeDefinition(def) => {
                if is_builtin_type(&def.name) {
                    errors.push(issue(name, source, "built-in type cannot be redefined", def.span));
                } else if types.contains_key(&def.name) {
                    errors.push(issue(name, source, format!("duplicate type {}", def.name), def.span));
                } else {
                    types.insert(def.name.clone(), def.clone());
                }
            }
            Item::InstanceDefinition(def) => {
                if instances.contains_key(&def.name) {
                    errors.push(issue(name, source, format!("duplicate instance {}", def.name), def.span));
                } else {
                    instances.insert(def.name.clone(), def.clone());
                }
            }
            Item::Statement(_) | Item::ShowQuery(_) => {}
        }
    }

    for def in types.values() {
        if let Some(parent) = &def.parent
            && !known_type(parent, &types)
        {
            errors.push(issue(name, source, format!("unknown parent type {parent}"), def.span));
        }
        let mut properties = HashSet::new();
        for property in &def.properties {
            if !properties.insert(property.name.as_str()) {
                errors.push(issue(name, source, format!("duplicate property {}", property.name), property.span));
            }
            if def.parent.as_ref().is_some_and(|parent| find_property(parent, &property.name, &types).is_some()) {
                errors.push(issue(name, source, format!("cannot override inherited property {}", property.name), property.span));
            }
            for term in &property.value_type.terms {
                if !known_type(term, &types) {
                    errors.push(issue(name, source, format!("unknown property value type {term}"), property.value_type.span));
                }
            }
        }
    }

    for def in types.values() {
        let mut visited = HashSet::new();
        let mut current = Some(def.name.as_str());
        while let Some(type_name) = current {
            if !visited.insert(type_name) {
                errors.push(issue(name, source, format!("cyclic inheritance at {type_name}"), def.span));
                break;
            }
            current = types.get(type_name).and_then(|item| item.parent.as_deref());
        }
    }

    for instance in instances.values() {
        check_type_expr(&instance.instance_type, &types, name, source, &mut errors);
    }

    for item in &program.items {
        if let Item::Statement(stmt) = item {
            match instances.get(&stmt.subject) {
                Some(subject) if satisfies(&subject.instance_type, "Subject", &types) => {}
                Some(_) => errors.push(issue(name, source, "statement subject must satisfy Subject", stmt.span)),
                None => errors.push(issue(name, source, format!("unknown subject {}", stmt.subject), stmt.span)),
            }
            match instances.get(&stmt.tag) {
                Some(tag) if satisfies(&tag.instance_type, "Tag", &types) => {
                    check_assignments(&stmt.properties, &tag.instance_type, &types, &instances, name, source, &mut errors);
                }
                Some(_) => errors.push(issue(name, source, format!("{} is not Tag-eligible", stmt.tag), stmt.span)),
                None => errors.push(issue(name, source, format!("unknown tag {}", stmt.tag), stmt.span)),
            }
        }
    }

    for item in &program.items {
        if let Item::ShowQuery(query) = item {
            check_query_expr(&query.expression, &types, &instances, name, source, &mut errors);
        }
    }

    if errors.is_empty() { Ok(()) } else { Err(errors) }
}

fn check_query_expr(
    expression: &QueryExpr,
    types: &IndexMap<String, TypeDefinition>,
    instances: &IndexMap<String, InstanceDefinition>,
    name: &str,
    source: &str,
    errors: &mut Vec<FrontendError>,
) {
    match expression {
        QueryExpr::Type { name: type_name, conditions, span } => {
            if !known_type(type_name, types) {
                errors.push(issue(name, source, format!("unknown query type {type_name}"), *span));
                return;
            }
            check_query_conditions(type_name, conditions, types, instances, name, source, errors);
        }
        QueryExpr::And(left, right)
        | QueryExpr::Or(left, right)
        | QueryExpr::Difference(left, right) => {
            check_query_expr(left, types, instances, name, source, errors);
            check_query_expr(right, types, instances, name, source, errors);
        }
        QueryExpr::Not(inner) => check_query_expr(inner, types, instances, name, source, errors),
    }
}

fn check_query_conditions(
    owner_type: &str,
    conditions: &[crate::ast::PropertyCondition],
    types: &IndexMap<String, TypeDefinition>,
    instances: &IndexMap<String, InstanceDefinition>,
    name: &str,
    source: &str,
    errors: &mut Vec<FrontendError>,
) {
    for condition in conditions {
        let Some(range) = find_property(owner_type, &condition.name, types) else {
            errors.push(issue(
                name, source,
                format!("property {} is not defined on query type {owner_type}", condition.name),
                condition.span,
            ));
            continue;
        };
        check_query_predicate(&condition.predicate, range, types, instances, name, source, errors);
    }
}

fn check_query_predicate(
    predicate: &PropertyPredicate,
    range: &TypeExpression,
    types: &IndexMap<String, TypeDefinition>,
    instances: &IndexMap<String, InstanceDefinition>,
    name: &str,
    source: &str,
    errors: &mut Vec<FrontendError>,
) {
    match predicate {
        PropertyPredicate::None => {}
        PropertyPredicate::And(left, right) | PropertyPredicate::Or(left, right) => {
            check_query_predicate(left, range, types, instances, name, source, errors);
            check_query_predicate(right, range, types, instances, name, source, errors);
        }
        PropertyPredicate::Anonymous { type_name, conditions, span } => {
            let actual = TypeExpression { terms: vec![type_name.clone()], span: *span };
            if !known_type(type_name, types) {
                errors.push(issue(name, source, format!("unknown nested query type {type_name}"), *span));
            } else if !compatible(&actual, range, types) {
                errors.push(issue(name, source, format!("nested {type_name} has an incompatible type for this property"), *span));
            } else {
                check_query_conditions(type_name, conditions, types, instances, name, source, errors);
            }
        }
        PropertyPredicate::Compare { op, value, span } => match value {
            QueryValue::Number { value, .. } => {
                let actual = TypeExpression {
                    terms: vec![value.type_name().to_owned()],
                    span: *span,
                };
                if !range.terms.iter().all(|term| subtype(term, "Float", types)) {
                    errors.push(issue(name, source, "numeric comparison requires a numeric property", *span));
                } else if !compatible(&actual, range, types) {
                    errors.push(issue(name, source, format!("numeric literal of type {} is incompatible with this property", value.type_name()), *span));
                }
            }
            QueryValue::Reference { name: value_name, span: value_span } => {
                if !matches!(op, CompareOp::Eq | CompareOp::Ne) {
                    errors.push(issue(name, source, "ordering comparisons require a numeric value", *span));
                }
                match instances.get(value_name) {
                    Some(instance) if compatible(&instance.instance_type, range, types) => {}
                    Some(_) => errors.push(issue(name, source, format!("instance {value_name} has an incompatible type for this property"), *value_span)),
                    None => errors.push(issue(name, source, format!("unknown instance {value_name}"), *value_span)),
                }
            }
        },
    }
}

fn is_builtin_type(name: &str) -> bool {
    matches!(name, "Subject" | "Tag" | "Float" | "Int" | "Nat")
}

fn builtin_parent(name: &str) -> Option<&'static str> {
    match name {
        "Nat" => Some("Int"),
        "Int" => Some("Float"),
        _ => None,
    }
}

fn known_type(name: &str, types: &IndexMap<String, TypeDefinition>) -> bool {
    is_builtin_type(name) || types.contains_key(name)
}

fn check_type_expr(expr: &TypeExpression, types: &IndexMap<String, TypeDefinition>, name: &str, source: &str, errors: &mut Vec<FrontendError>) {
    let mut seen = HashSet::new();
    for term in &expr.terms {
        if !known_type(term, types) {
            errors.push(issue(name, source, format!("unknown type {term}"), expr.span));
        }
        if !seen.insert(term) {
            errors.push(issue(name, source, format!("duplicate type {term} in intersection"), expr.span));
        }
    }
}

fn subtype(candidate: &str, expected: &str, types: &IndexMap<String, TypeDefinition>) -> bool {
    let mut current = Some(candidate);
    let mut visited = HashSet::new();
    while let Some(name) = current {
        if name == expected { return true; }
        if !visited.insert(name) { return false; }
        current = types.get(name)
            .and_then(|def| def.parent.as_deref())
            .or_else(|| builtin_parent(name));
    }
    false
}

fn satisfies(expr: &TypeExpression, expected: &str, types: &IndexMap<String, TypeDefinition>) -> bool {
    expr.terms.iter().any(|term| subtype(term, expected, types))
}

fn compatible(actual: &TypeExpression, expected: &TypeExpression, types: &IndexMap<String, TypeDefinition>) -> bool {
    expected.terms.iter().all(|need| actual.terms.iter().any(|have| subtype(have, need, types)))
}

fn find_property<'a>(type_name: &str, property: &str, types: &'a IndexMap<String, TypeDefinition>) -> Option<&'a TypeExpression> {
    let mut current = Some(type_name);
    let mut visited = HashSet::new();
    while let Some(name) = current {
        if !visited.insert(name) { return None; }
        let def = types.get(name)?;
        if let Some(found) = def.properties.iter().find(|item| item.name == property) {
            return Some(&found.value_type);
        }
        current = def.parent.as_deref();
    }
    None
}

fn check_assignments(
    assignments: &[PropertyAssignment],
    owner: &TypeExpression,
    types: &IndexMap<String, TypeDefinition>,
    instances: &IndexMap<String, InstanceDefinition>,
    name: &str,
    source: &str,
    errors: &mut Vec<FrontendError>,
) {
    let mut seen = HashSet::new();
    for assignment in assignments {
        if !seen.insert(assignment.name.as_str()) {
            errors.push(issue(name, source, format!("property {} occurs more than once", assignment.name), assignment.span));
            continue;
        }
        let ranges: Vec<_> = owner.terms.iter().filter_map(|term| find_property(term, &assignment.name, types)).collect();
        if ranges.is_empty() {
            errors.push(issue(name, source, format!("property {} is not defined on this type", assignment.name), assignment.span));
            continue;
        }
        check_value(&assignment.value, &ranges, types, instances, name, source, errors);
    }
}

fn check_value(
    value: &ValueExpression,
    ranges: &[&TypeExpression],
    types: &IndexMap<String, TypeDefinition>,
    instances: &IndexMap<String, InstanceDefinition>,
    name: &str,
    source: &str,
    errors: &mut Vec<FrontendError>,
) {
    match value {
        ValueExpression::Reference { name: value_name, span } => match instances.get(value_name) {
            Some(instance) if ranges.iter().all(|range| compatible(&instance.instance_type, range, types)) => {}
            Some(_) => errors.push(issue(name, source, format!("instance {value_name} has an incompatible type"), *span)),
            None => errors.push(issue(name, source, format!("unknown instance {value_name}"), *span)),
        },
        ValueExpression::Number { value, span } => {
            let actual = TypeExpression {
                terms: vec![value.type_name().to_owned()],
                span: *span,
            };
            if ranges.iter().any(|range| !range.terms.iter().all(|term| subtype(term, "Float", types))) {
                errors.push(issue(name, source, "numeric literal requires a numeric property", *span));
            } else if ranges.iter().any(|range| !compatible(&actual, range, types)) {
                errors.push(issue(name, source, format!("numeric literal of type {} is incompatible with this property", value.type_name()), *span));
            }
        }
        ValueExpression::AnonymousInstance { type_name, properties, span } => {
            let actual = TypeExpression { terms: vec![type_name.clone()], span: *span };
            if !known_type(type_name, types) {
                errors.push(issue(name, source, format!("unknown anonymous type {type_name}"), *span));
            } else if ranges.iter().any(|range| !compatible(&actual, range, types)) {
                errors.push(issue(name, source, format!("anonymous {type_name} has an incompatible type"), *span));
            } else {
                check_assignments(properties, &actual, types, instances, name, source, errors);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse;

    const EXAMPLE: &str = r#"
def Video
def Appearance
def Color
def SolidColor { color: Color }: Appearance
def Animal { appearance: Appearance }
let white: Color
let black: Color
let cat: Tag + Animal
let video_a: Video
stmt video_a has cat { appearance: SolidColor { color: white } }
stmt video_a has cat { appearance: SolidColor { color: black } }
"#;

    #[test]
    fn accepts_nested_values_and_repeated_independent_statements() {
        let program = parse("valid.mt", EXAMPLE).unwrap();
        assert!(analyze(&program, "valid.mt", EXAMPLE).is_ok());
    }

    #[test]
    fn rejects_non_tag_statement_objects() {
        let source = "def Video
def Color
let video: Video
let white: Color
stmt video has white";
        let program = parse("invalid.mt", source).unwrap();
        let errors = analyze(&program, "invalid.mt", source).unwrap_err();
        assert!(errors.iter().any(|error| error.to_string().contains("not Tag-eligible")));
    }

    #[test]
    fn rejects_duplicate_statement_properties() {
        let source = "def Video
def Animal { color: Int }
let tag: Tag + Animal
let video: Video
stmt video has tag { color: 1, color: 2 }";
        let program = parse("duplicate.mt", source).unwrap();
        let errors = analyze(&program, "duplicate.mt", source).unwrap_err();
        assert!(errors.iter().any(|error| error.to_string().contains("more than once")));
    }
}
