use std::collections::HashSet;

use my_tags::{
    ast::Item, parse, query_dsl, resolve, NumericValue,
    resolved::Value,
};

fn resolve_source(source: &str) -> my_tags::resolved::ResolvedModel {
    let program = parse("numeric-types.mt", source).expect("source should parse");
    resolve(&program, "numeric-types.mt", source).expect("source should resolve")
}

fn query_subject_names(source: &str) -> HashSet<String> {
    let program = parse("numeric-query.mt", source).expect("query should parse");
    let model = resolve(&program, "numeric-query.mt", source).expect("query should resolve");
    let Item::ShowQuery(query) = program.items.last().expect("query item should exist") else {
        panic!("last item should be a query");
    };
    query_dsl::evaluate(&model, &query.expression)
        .expect("query should evaluate")
        .into_iter()
        .map(|id| model.instance_by_id(id).unwrap().name.clone())
        .collect()
}

#[test]
fn resolves_numeric_literals_and_implicit_widening() {
    let source = r#"
def Experiment: Subject
def Measurement: Tag {
  as_float: Float,
  as_int: Int,
  as_nat: Nat
}
let experiment: Experiment
let measurement: Measurement
stmt experiment has measurement {
  as_float: 0.125,
  as_int: 25,
  as_nat: 3
}
stmt experiment has measurement {
  as_float: -2,
  as_int: -7,
  as_nat: 0
}
"#;
    let model = resolve_source(source);

    let first = &model.statements[0].properties;
    assert!(matches!(first[0].value, Value::Number(NumericValue::Float(value)) if value == 0.125));
    assert!(matches!(first[1].value, Value::Number(NumericValue::Int(25))));
    assert!(matches!(first[2].value, Value::Number(NumericValue::Nat(3))));

    let second = &model.statements[1].properties;
    assert!(matches!(second[0].value, Value::Number(NumericValue::Float(value)) if value == -2.0));
    assert!(matches!(second[1].value, Value::Number(NumericValue::Int(-7))));
    assert!(matches!(second[2].value, Value::Number(NumericValue::Nat(0))));
}

#[test]
fn accepts_numeric_type_boundaries() {
    let source = r#"
def Experiment: Subject
def Measurement: Tag {
  largest_nat: Nat,
  smallest_int: Int,
  largest_finite_float: Float
}
let experiment: Experiment
let measurement: Measurement
stmt experiment has measurement {
  largest_nat: 18446744073709551615,
  smallest_int: -170141183460469231731687303715884105728,
  largest_finite_float: 1.7976931348623157e308
}
"#;
    let model = resolve_source(source);
    let values = &model.statements[0].properties;

    assert!(matches!(
        values[0].value,
        Value::Number(NumericValue::Nat(u64::MAX))
    ));
    assert!(matches!(
        values[1].value,
        Value::Number(NumericValue::Int(i128::MIN))
    ));
    assert!(matches!(
        values[2].value,
        Value::Number(NumericValue::Float(value)) if value.is_finite() && value > 1.0e308
    ));
}

#[test]
fn parses_decimal_and_exponent_float_literals() {
    let source = r#"
def Experiment: Subject
def Measurement: Tag {
  leading_dot: Float,
  exponent: Float,
  negative_exponent: Float,
  negative_zero: Float
}
let experiment: Experiment
let measurement: Measurement
stmt experiment has measurement {
  leading_dot: .75,
  exponent: 1e3,
  negative_exponent: 1.5e-2,
  negative_zero: -0.0
}
"#;
    let model = resolve_source(source);
    let values = &model.statements[0].properties;

    assert!(matches!(values[0].value, Value::Number(NumericValue::Float(value)) if value == 0.75));
    assert!(matches!(values[1].value, Value::Number(NumericValue::Float(value)) if value == 1000.0));
    assert!(matches!(values[2].value, Value::Number(NumericValue::Float(value)) if (value - 0.015).abs() < 1e-15));
    assert!(matches!(values[3].value, Value::Number(NumericValue::Float(value)) if value == 0.0 && value.is_sign_negative()));
}

#[test]
fn rejects_numeric_literals_outside_supported_ranges() {
    let cases = [
        (
            "Nat overflow",
            "def Experiment: Subject\ndef Measurement: Tag { value: Nat }\nlet experiment: Experiment\nlet measurement: Measurement\nstmt experiment has measurement { value: 18446744073709551616 }",
        ),
        (
            "Int underflow",
            "def Experiment: Subject\ndef Measurement: Tag { value: Int }\nlet experiment: Experiment\nlet measurement: Measurement\nstmt experiment has measurement { value: -170141183460469231731687303715884105729 }",
        ),
        (
            "Float overflow",
            "def Experiment: Subject\ndef Measurement: Tag { value: Float }\nlet experiment: Experiment\nlet measurement: Measurement\nstmt experiment has measurement { value: 1e999 }",
        ),
        (
            "negative Float overflow",
            "def Experiment: Subject\ndef Measurement: Tag { value: Float }\nlet experiment: Experiment\nlet measurement: Measurement\nstmt experiment has measurement { value: -1e999 }",
        ),
    ];

    for (label, source) in cases {
        assert!(parse("overflow.mt", source).is_err(), "{label} should be rejected");
    }
}

#[test]
fn rejects_numeric_narrowing_conversions() {
    let cases = [
        (
            "Float to Int",
            "def Experiment: Subject\ndef Measurement: Tag { value: Int }\nlet experiment: Experiment\nlet measurement: Measurement\nstmt experiment has measurement { value: 1.5 }",
        ),
        (
            "Int to Nat",
            "def Experiment: Subject\ndef Measurement: Tag { value: Nat }\nlet experiment: Experiment\nlet measurement: Measurement\nstmt experiment has measurement { value: -1 }",
        ),
    ];

    for (label, source) in cases {
        let program = parse("narrowing.mt", source).expect("narrowing examples should parse");
        assert!(
            my_tags::analyze(&program, "narrowing.mt", source).is_err(),
            "{label} should fail semantic validation"
        );
    }
}

#[test]
fn rejects_non_finite_float_spellings_as_numeric_values() {
    for literal in ["NaN", "Infinity"] {
        let source = format!(
            "def Experiment: Subject\\ndef Measurement: Tag {{ value: Float }}\\nlet experiment: Experiment\\nlet measurement: Measurement\\nstmt experiment has measurement {{ value: {literal} }}"
        );
        let program = parse("non-finite.mt", &source).expect("identifier-shaped input should parse");
        assert!(
            my_tags::analyze(&program, "non-finite.mt", &source).is_err(),
            "{literal} must not be accepted as a numeric value"
        );
    }

    let negative_infinity = "def Experiment: Subject\\ndef Measurement: Tag { value: Float }\\nlet experiment: Experiment\\nlet measurement: Measurement\\nstmt experiment has measurement { value: -Infinity }";
    assert!(parse("non-finite.mt", negative_infinity).is_err());
}

#[test]
fn query_dsl_supports_all_numeric_comparison_operators() {
    let base = r#"
def Experiment: Subject
def Metric: Tag { score: Float }
let a: Experiment
let b: Experiment
let c: Experiment
let metric: Metric
stmt a has metric { score: 1 }
stmt b has metric { score: 2.0 }
stmt c has metric { score: 3 }
"#;

    let cases: [(&str, &str, &[&str]); 6] = [
        ("==", "2", &["b"]),
        ("!=", "2", &["a", "c"]),
        (">", "2", &["c"]),
        (">=", "2", &["b", "c"]),
        ("<", "2", &["a"]),
        ("<=", "2", &["a", "b"]),
    ];

    for (operator, expected, names) in cases {
        let source = format!(
            "{base}\n#show query Metric {{ score: ({operator} {expected}) }}"
        );
        assert_eq!(
            query_subject_names(&source),
            names.iter().map(|name| (*name).to_owned()).collect(),
            "operator {operator} should compare mixed Nat/Float values"
        );
    }
}

#[test]
fn query_api_supports_all_numeric_comparison_operators() {
    let source = r#"
def Experiment: Subject
def Metric: Tag { score: Float }
let a: Experiment
let b: Experiment
let c: Experiment
let metric: Metric
stmt a has metric { score: 1 }
stmt b has metric { score: 2.0 }
stmt c has metric { score: 3 }
"#;
    let model = resolve_source(source);
    let metric = model.types.iter().find(|ty| ty.name == "Metric").unwrap();
    let score = metric.properties.iter().find(|property| property.name == "score").unwrap().id;

    let names = |query: my_tags::query::Query<'_>| -> HashSet<String> {
        query.subjects().into_iter().map(|instance| instance.name.clone()).collect()
    };

    assert_eq!(names(model.query().where_property_number_eq(score, 2)), HashSet::from(["b".to_owned()]));
    assert_eq!(names(model.query().where_property_number_ne(score, 2)), HashSet::from(["a".to_owned(), "c".to_owned()]));
    assert_eq!(names(model.query().where_property_number_gt(score, 2)), HashSet::from(["c".to_owned()]));
    assert_eq!(names(model.query().where_property_number_ge(score, 2)), HashSet::from(["b".to_owned(), "c".to_owned()]));
    assert_eq!(names(model.query().where_property_number_lt(score, 2)), HashSet::from(["a".to_owned()]));
    assert_eq!(names(model.query().where_property_number_le(score, 2)), HashSet::from(["a".to_owned(), "b".to_owned()]));
}

#[test]
fn numeric_comparison_preserves_integer_precision_against_floats() {
    let integer = 9_007_199_254_740_993_i128;
    let rounded_float = 9_007_199_254_740_992.0_f64;

    assert_eq!(
        NumericValue::Int(integer).partial_cmp_numeric(NumericValue::Float(rounded_float)),
        Some(std::cmp::Ordering::Greater)
    );
    assert_eq!(
        NumericValue::Int(-integer).partial_cmp_numeric(NumericValue::Float(-rounded_float)),
        Some(std::cmp::Ordering::Less)
    );
    assert_eq!(
        NumericValue::Nat(u64::MAX).partial_cmp_numeric(NumericValue::Float(18_446_744_073_709_551_616.0)),
        Some(std::cmp::Ordering::Less)
    );
}
