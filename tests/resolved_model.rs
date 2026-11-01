use my_tags::{parse, resolve, storage};
use my_tags::resolved::{PropertyId, Value};
use my_tags::query::{PropertyPath, PropertyPathError};\nuse my_tags::query_dsl;

const SOURCE: &str = r#"
def Video: Subject
def Appearance
def Color
def SolidColor { color: Color, opacity: Int }: Appearance
def Animal { appearance: Appearance, weight: Int }
def Mammal: Animal
let white: Color
let cat: Tag + Mammal
let video: Video
stmt video has cat { appearance: SolidColor { color: white, opacity: 80 }, weight: 12 }
stmt video has cat { appearance: SolidColor { color: white, opacity: 80 }, weight: -3 }
"#;

#[test]
fn resolves_float_int_and_nat_with_implicit_upcasts() {
    use my_tags::NumericValue;

    let source = r#"def Experiment: Subject
def Measurement {
  concentration: Float,
  count: Int,
  samples: Nat
}: Tag
let experiment: Experiment
let measurement: Measurement
stmt experiment has measurement {
  concentration: 0.125,
  count: 25,
  samples: 3
}"#;
    let ast = parse("numeric.mt", source).unwrap();
    let model = resolve(&ast, "numeric.mt", source).unwrap();
    let values = &model.statements[0].properties;
    assert!(matches!(values[0].value, Value::Number(NumericValue::Float(value)) if value == 0.125));
    assert!(matches!(values[1].value, Value::Number(NumericValue::Int(25))));
    assert!(matches!(values[2].value, Value::Number(NumericValue::Nat(3))));
    let measurement = model.types.iter().find(|ty| ty.name == "Measurement").unwrap();
    let concentration = measurement.properties.iter().find(|p| p.name == "concentration").unwrap().id;
    assert_eq!(model.query().where_property_number_gt(concentration, 0.1).collect().len(), 1);
}

#[test]
fn query_dsl_compares_float_and_integer_literals() {
    let source = r#"def Experiment: Subject
def Measurement: Tag {
  concentration: Float,
  count: Int
}
let experiment: Experiment
let measurement: Measurement
stmt experiment has measurement {
  concentration: 0.125,
  count: 25
}
#show query Measurement {
  concentration: (> 0.1)
  count: (>= 20)
}"#;
    let ast = parse("query-numeric.mt", source).unwrap();
    let model = resolve(&ast, "query-numeric.mt", source).unwrap();
    let query = ast.items.iter().find_map(|item| {
        if let my_tags::ast::Item::ShowQuery(query) = item {
            Some(&query.expression)
        } else {
            None
        }
    }).unwrap();
    let results = query_dsl::evaluate(&model, query).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(model.instance_by_id(*results.iter().next().unwrap()).unwrap().name, "experiment");
}

#[test]
fn resolves_ids_inherited_properties_and_independent_statements() {
    let ast = parse("test.mt", SOURCE).unwrap();
    let model = resolve(&ast, "test.mt", SOURCE).unwrap();
    assert_eq!(model.statements.len(), 2);
    assert_ne!(model.statements[0].id, model.statements[1].id);
    let animal = model.types.iter().find(|ty| ty.name == "Animal").unwrap();
    let appearance = animal.properties.iter().find(|property| property.name == "appearance").unwrap();
    assert_eq!(model.statements[0].properties[0].property, appearance.id);
    assert_eq!(appearance.id, PropertyId { owner: animal.id, index: 0 });
    assert!(matches!(&model.statements[0].properties[0].value, Value::Anonymous { .. }));
}

#[test]
fn query_preserves_statements_but_deduplicates_subjects_explicitly() {
    let ast = parse("test.mt", SOURCE).unwrap();
    let model = resolve(&ast, "test.mt", SOURCE).unwrap();
    let animal = model.types.iter().find(|ty| ty.name == "Animal").unwrap();
    assert_eq!(model.query().where_tag_is_subtype_of(animal.id).collect().len(), 2);
    assert_eq!(model.query().where_tag_is_subtype_of(animal.id).subjects().len(), 1);
}

#[test]
fn json_round_trip_preserves_resolved_model() {
    let ast = parse("test.mt", SOURCE).unwrap();
    let model = resolve(&ast, "test.mt", SOURCE).unwrap();
    let json = storage::to_json(&model).unwrap();
    assert_eq!(model, storage::from_json(&json).unwrap());
}


#[test]
fn query_supports_property_presence_nested_reference_and_integer_comparison() {
    let ast = parse("test.mt", SOURCE).unwrap();
    let model = resolve(&ast, "test.mt", SOURCE).unwrap();
    let animal = model.types.iter().find(|ty| ty.name == "Animal").unwrap();
    let appearance = animal.properties.iter().find(|p| p.name == "appearance").unwrap().id;
    let weight = animal.properties.iter().find(|p| p.name == "weight").unwrap().id;
    let solid = model.types.iter().find(|ty| ty.name == "SolidColor").unwrap();
    let color = solid.properties.iter().find(|p| p.name == "color").unwrap().id;
    let white = model.instances.iter().find(|i| i.name == "white").unwrap().id;

    assert_eq!(model.query().where_property_exists(weight).collect().len(), 2);
    assert_eq!(model.query().where_property_number_gt(weight, 0).collect().len(), 1);
    assert_eq!(model.query().where_property_number_le(weight, 0).collect().len(), 1);

    let path = PropertyPath::resolve(&model, appearance, &[color]).unwrap();
    assert_eq!(model.query().where_property_path_reference_equals(&path, white).collect().len(), 2);
    assert_eq!(model.query().where_property_path_exists(&path).collect().len(), 2);
}

#[test]
fn property_path_rejects_invalid_nested_property() {
    let ast = parse("test.mt", SOURCE).unwrap();
    let model = resolve(&ast, "test.mt", SOURCE).unwrap();
    let animal = model.types.iter().find(|ty| ty.name == "Animal").unwrap();
    let appearance = animal.properties.iter().find(|p| p.name == "appearance").unwrap().id;
    let weight = animal.properties.iter().find(|p| p.name == "weight").unwrap().id;
    assert!(matches!(
        PropertyPath::resolve(&model, appearance, &[weight]),
        Err(PropertyPathError::InvalidTransition { .. })
    ));
}
