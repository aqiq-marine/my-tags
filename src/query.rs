use std::collections::HashSet;
use crate::resolved::{Instance, InstanceId, ResolvedModel, Statement, TypeId};

pub struct Query<'a> { model: &'a ResolvedModel, statements: Vec<&'a Statement> }
impl<'a> Query<'a> {
    pub fn new(model: &'a ResolvedModel) -> Self { Self { model, statements: model.statements.iter().collect() } }
    pub fn statements(self) -> Self { self }
    pub fn where_subject(mut self, subject: InstanceId) -> Self { self.statements.retain(|s| s.subject == subject); self }
    pub fn where_tag(mut self, tag: InstanceId) -> Self { self.statements.retain(|s| s.tag == tag); self }
    pub fn where_tag_is_subtype_of(mut self, expected: TypeId) -> Self {
        let model = self.model;
        self.statements.retain(|s| model.instance_has_type(s.tag, expected)); self
    }
    pub fn where_property(mut self, property: crate::resolved::PropertyId) -> Self {
        self.statements.retain(|s| s.properties.iter().any(|v| v.property == property)); self
    }
    pub fn collect(self) -> Vec<&'a Statement> { self.statements }
    pub fn subjects(self) -> Vec<&'a Instance> {
        let mut seen = HashSet::new();
        self.statements.into_iter().filter_map(|s| {
            if seen.insert(s.subject) { self.model.instance_by_id(s.subject) } else { None }
        }).collect()
    }
}
impl ResolvedModel { pub fn query(&self) -> Query<'_> { Query::new(self) } }
