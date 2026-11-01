use std::collections::HashSet;

use my_tags::{parse, query_dsl, resolve};
use my_tags::ast::Item;

const SOURCE: &str = r#"
def Video
def Piyo
def Reaction: Tag {
  yield: Number
}
def Asymmetric: Reaction {
  ee: Number
}
def Hoge {
  huga: Piyo
}

let p1: Piyo
let p2: Piyo
let p3: Piyo
let heck: Asymmetric + Hoge
let video_a: Video
let video_b: Video
let video_c: Video

stmt video_a has heck {
  yield: 90
  ee: 80
  huga: p1
}
stmt video_b has heck {
  yield: 90
  ee: 80
  huga: p3
}
stmt video_b has heck {
  yield: 70
  ee: 95
  huga: p1
}
stmt video_c has heck {
  ee: 95
  huga: p2
}

#show query Reaction {
  yield: (>= 80)
} & Hoge {
  huga: (== p1) || (== p2)
}
"#;

fn model() -> (my_tags::ast::Program, my_tags::resolved::ResolvedModel) {
    let program = parse("query.mt", SOURCE).unwrap();
    let model = resolve(&program, "query.mt", SOURCE).unwrap();
    (program, model)
}

fn names(ids: HashSet<my_tags::resolved::InstanceId>, model: &my_tags::resolved::ResolvedModel) -> HashSet<String> {
    ids.into_iter()
        .map(|id| model.instance_by_id(id).unwrap().name.clone())
        .collect()
}

#[test]
fn evaluates_intersection_and_property_or_on_subjects() {
    let (program, model) = model();
    let Item::ShowQuery(query) = &program.items[program.items.len() - 1] else {
        panic!("expected show query");
    };
    assert_eq!(
        names(query_dsl::evaluate(&model, &query.expression).unwrap(), &model),
        HashSet::from(["video_a".to_owned(), "video_b".to_owned()])
    );
}

#[test]
fn all_property_conditions_must_match_the_same_statement() {
    let source = r#"
def Video
def Reaction: Tag { yield: Number }
def Asymmetric: Reaction { ee: Number }
let heck: Asymmetric
let video: Video
stmt video has heck { yield: 90 ee: 80 }
stmt video has heck { yield: 70 ee: 95 }
#show query Asymmetric {
  yield: (>= 80)
  ee: (>= 90)
}
"#;
    let program = parse("same-statement.mt", source).unwrap();
    let model = resolve(&program, "same-statement.mt", source).unwrap();
    let Item::ShowQuery(query) = program.items.last().unwrap() else { panic!("expected query"); };
    assert!(query_dsl::evaluate(&model, &query.expression).unwrap().is_empty());
}

#[test]
fn none_matches_a_statement_without_the_property() {
    let source = r#"
def Video
def Reaction: Tag { yield: Number }
let heck: Reaction
let video: Video
stmt video has heck
#show query Reaction { yield: none }
"#;
    let program = parse("none.mt", source).unwrap();
    let model = resolve(&program, "none.mt", source).unwrap();
    let Item::ShowQuery(query) = program.items.last().unwrap() else { panic!("expected query"); };
    assert_eq!(
        names(query_dsl::evaluate(&model, &query.expression).unwrap(), &model),
        HashSet::from(["video".to_owned()])
    );
}

#[test]
fn difference_and_not_use_the_subject_universe() {
    let source = r#"
def Video
def Reaction: Tag { score: Number }
let tag: Reaction
let a: Video
let b: Video
stmt a has tag { score: 10 }
stmt b has tag { score: 1 }
#show query Reaction { score: (>= 5) } - Reaction { score: (>= 20) }
"#;
    let program = parse("set.mt", source).unwrap();
    let model = resolve(&program, "set.mt", source).unwrap();
    let Item::ShowQuery(query) = program.items.last().unwrap() else { panic!("expected query"); };
    assert_eq!(
        names(query_dsl::evaluate(&model, &query.expression).unwrap(), &model),
        HashSet::from(["a".to_owned()])
    );
}

#[test]
fn complement_selects_subjects_outside_the_inner_set() {
    let source = r#"
def Video
def Reaction: Tag { score: Number }
let tag: Reaction
let a: Video
let b: Video
stmt a has tag { score: 10 }
stmt b has tag { score: 1 }
#show query !Reaction { score: (>= 5) }
"#;
    let program = parse("not.mt", source).unwrap();
    let model = resolve(&program, "not.mt", source).unwrap();
    let Item::ShowQuery(query) = program.items.last().unwrap() else { panic!("expected query"); };
    assert_eq!(
        names(query_dsl::evaluate(&model, &query.expression).unwrap(), &model),
        HashSet::from(["b".to_owned()])
    );
}

#[test]
fn rejects_undefined_property_in_query_type_block() {
    let source = r#"
def Video
def Reaction: Tag { yield: Number }
let tag: Reaction
let video: Video
stmt video has tag { yield: 90 }
#show query Reaction { missing: none }
"#;
    let program = parse("invalid-query.mt", source).unwrap();
    let errors = my_tags::analyze(&program, "invalid-query.mt", source).unwrap_err();
    assert!(errors.iter().any(|error| error.to_string().contains("not defined on query type")));
}
