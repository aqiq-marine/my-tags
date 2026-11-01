use my_tags::{parse, resolve, storage};
use my_tags::resolved::{PropertyId, Value};

const SOURCE: &str = r#"
def Video
def Appearance
def Color
def SolidColor { color: Color }: Appearance
def Animal { appearance: Appearance }
def Mammal: Animal
let white: Color
let cat: Tag + Mammal
let video: Video
stmt video has cat { appearance: SolidColor { color: white } }
stmt video has cat { appearance: SolidColor { color: white } }
"#;

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
