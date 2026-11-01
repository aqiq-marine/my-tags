use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TypeId(pub u32);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct InstanceId(pub u32);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct StatementId(pub u32);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct PropertyId { pub owner: TypeId, pub index: u32 }

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolvedModel { pub types: Vec<Type>, pub instances: Vec<Instance>, pub statements: Vec<Statement> }
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Type { pub id: TypeId, pub name: String, pub parent: Option<TypeId>, pub properties: Vec<PropertyDefinition>, pub builtin: bool }
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PropertyDefinition { pub id: PropertyId, pub name: String, pub value_type: Vec<TypeId> }
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Instance { pub id: InstanceId, pub name: String, pub types: Vec<TypeId> }
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Statement { pub id: StatementId, pub subject: InstanceId, pub tag: InstanceId, pub properties: Vec<PropertyValue> }
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PropertyValue { pub property: PropertyId, pub value: Value }
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Value {
    Reference(InstanceId),
    Number(i64),
    Anonymous { type_id: TypeId, properties: Vec<PropertyValue> },
}

impl ResolvedModel {
    pub fn type_by_id(&self, id: TypeId) -> Option<&Type> {
        self.types.get(id.0 as usize).filter(|ty| ty.id == id)
    }
    pub fn instance_by_id(&self, id: InstanceId) -> Option<&Instance> {
        self.instances.get(id.0 as usize).filter(|instance| instance.id == id)
    }
    pub fn is_subtype_of(&self, candidate: TypeId, expected: TypeId) -> bool {
        let mut current = Some(candidate);
        let mut remaining = self.types.len() + 1;
        while let Some(id) = current {
            if id == expected { return true; }
            if remaining == 0 { return false; }
            remaining -= 1;
            current = self.type_by_id(id).and_then(|ty| ty.parent);
        }
        false
    }
    pub fn instance_has_type(&self, instance: InstanceId, expected: TypeId) -> bool {
        self.instance_by_id(instance).is_some_and(|value| value.types.iter().any(|id| self.is_subtype_of(*id, expected)))
    }
}
