use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TypeId(pub usize);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FunctionType {
    pub params: Vec<TypeId>,
    pub return_type: TypeId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StructFieldType {
    pub name: String,
    pub type_id: TypeId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StructType {
    pub name: String,
    pub fields: Vec<StructFieldType>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum KiraType {
    Unit,
    Bool,
    Int,
    Float,
    String,
    Dynamic,
    Array(TypeId),
    Function(FunctionType),
    Struct(StructType),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TypeSystem {
    types: Vec<KiraType>,
    names: HashMap<String, TypeId>,
}

impl Default for TypeSystem {
    fn default() -> Self {
        let mut system = Self {
            types: Vec::new(),
            names: HashMap::new(),
        };

        system.register_builtin("unit", KiraType::Unit);
        system.register_builtin("bool", KiraType::Bool);
        system.register_builtin("int", KiraType::Int);
        system.register_builtin("float", KiraType::Float);
        system.register_builtin("string", KiraType::String);
        system.register_builtin("dynamic", KiraType::Dynamic);
        system
    }
}

impl TypeSystem {
    pub fn unit(&self) -> TypeId {
        self.names["unit"]
    }

    pub fn dynamic(&self) -> TypeId {
        self.names["dynamic"]
    }

    pub fn bool(&self) -> TypeId {
        self.names["bool"]
    }

    pub fn float(&self) -> TypeId {
        self.names["float"]
    }

    pub fn int(&self) -> TypeId {
        self.names["int"]
    }

    pub fn register_function(&mut self, params: Vec<TypeId>, return_type: TypeId) -> TypeId {
        self.intern(KiraType::Function(FunctionType {
            params,
            return_type,
        }))
    }

    pub fn register_array(&mut self, element: TypeId) -> TypeId {
        self.intern(KiraType::Array(element))
    }

    pub fn declare_struct(&mut self, name: &str) -> Result<TypeId, String> {
        if self.names.contains_key(name) {
            return Err(format!("type `{name}` is already defined"));
        }

        let id = TypeId(self.types.len());
        self.types.push(KiraType::Struct(StructType {
            name: name.to_string(),
            fields: Vec::new(),
        }));
        self.names.insert(name.to_string(), id);
        Ok(id)
    }

    pub fn define_struct(
        &mut self,
        name: &str,
        fields: Vec<(String, TypeId)>,
    ) -> Result<TypeId, String> {
        let Some(type_id) = self.names.get(name).copied() else {
            return Err(format!("type `{name}` is not declared"));
        };

        let KiraType::Struct(struct_type) = self
            .types
            .get_mut(type_id.0)
            .ok_or_else(|| format!("type `{name}` is not declared"))?
        else {
            return Err(format!("type `{name}` is not a struct"));
        };

        if !struct_type.fields.is_empty() {
            return Err(format!("struct `{name}` is already defined"));
        }

        let mut seen = HashMap::new();
        let mut resolved_fields = Vec::with_capacity(fields.len());
        for (field_name, field_type) in fields {
            if seen.insert(field_name.clone(), ()).is_some() {
                return Err(format!(
                    "struct `{name}` declares field `{field_name}` more than once"
                ));
            }
            resolved_fields.push(StructFieldType {
                name: field_name,
                type_id: field_type,
            });
        }

        struct_type.fields = resolved_fields;
        Ok(type_id)
    }

    pub fn ensure_named(&mut self, name: &str) -> Option<TypeId> {
        if let Some(id) = self.names.get(name).copied() {
            return Some(id);
        }

        let inner = name
            .strip_prefix('[')
            .and_then(|value| value.strip_suffix(']'))?;
        let element = self.ensure_named(inner)?;
        let id = self.register_array(element);
        self.names.insert(name.to_string(), id);
        Some(id)
    }

    pub fn resolve_named(&self, name: &str) -> Option<TypeId> {
        self.names.get(name).copied()
    }

    pub fn get(&self, id: TypeId) -> &KiraType {
        &self.types[id.0]
    }

    pub fn type_name(&self, id: TypeId) -> String {
        match self.get(id) {
            KiraType::Unit => "unit".to_string(),
            KiraType::Bool => "bool".to_string(),
            KiraType::Int => "int".to_string(),
            KiraType::Float => "float".to_string(),
            KiraType::String => "string".to_string(),
            KiraType::Dynamic => "dynamic".to_string(),
            KiraType::Array(element) => format!("[{}]", self.type_name(*element)),
            KiraType::Function(_) => "func".to_string(),
            KiraType::Struct(struct_type) => struct_type.name.clone(),
        }
    }

    pub fn struct_field(&self, struct_type: TypeId, field_name: &str) -> Option<(usize, TypeId)> {
        let KiraType::Struct(struct_type) = self.get(struct_type) else {
            return None;
        };

        struct_type
            .fields
            .iter()
            .enumerate()
            .find(|(_, field)| field.name == field_name)
            .map(|(index, field)| (index, field.type_id))
    }

    pub fn struct_fields(&self, struct_type: TypeId) -> Option<&[StructFieldType]> {
        let KiraType::Struct(struct_type) = self.get(struct_type) else {
            return None;
        };
        Some(&struct_type.fields)
    }

    pub fn is_assignable(&self, expected: TypeId, actual: TypeId) -> bool {
        expected == actual || expected == self.dynamic()
    }

    fn register_builtin(&mut self, name: &str, kind: KiraType) {
        let id = self.intern(kind);
        self.names.insert(name.to_string(), id);
    }

    fn intern(&mut self, kind: KiraType) -> TypeId {
        if let Some(index) = self.types.iter().position(|existing| existing == &kind) {
            return TypeId(index);
        }

        let id = TypeId(self.types.len());
        self.types.push(kind);
        id
    }
}
