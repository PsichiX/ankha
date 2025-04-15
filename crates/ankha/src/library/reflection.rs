use crate::library::option::AnkhaOption;
use intuicio_core::{
    function::{FunctionHandle, FunctionQuery},
    registry::Registry,
    transformer::{DynamicManagedValueTransformer, ValueTransformer},
    types::{TypeHandle, TypeQuery, struct_type::NativeStructBuilder},
};
use intuicio_data::managed::DynamicManaged;
use intuicio_derive::{intuicio_method, intuicio_methods};

pub fn install(registry: &mut Registry) {
    registry.add_type(
        NativeStructBuilder::new_named_uninitialized::<Type>("Type")
            .module_name("reflect")
            .build(),
    );
    registry.add_function(Type::by_name__define_function(registry));
    registry.add_function(Type::name__define_function(registry));
    registry.add_function(Type::module_name__define_function(registry));
    registry.add_function(Type::function__define_function(registry));
    registry.add_type(
        NativeStructBuilder::new_named_uninitialized::<Function>("Function")
            .module_name("reflect")
            .build(),
    );
    registry.add_function(Function::by_name__define_function(registry));
    registry.add_function(Function::name__define_function(registry));
    registry.add_function(Function::module_name__define_function(registry));
}

#[derive(Debug, Clone)]
pub struct Type(pub TypeHandle);

#[intuicio_methods(module_name = "reflect")]
impl Type {
    #[intuicio_method(use_registry, transformer = "DynamicManagedValueTransformer")]
    pub fn by_name(registry: &Registry, name: String, module_name: String) -> AnkhaOption {
        registry
            .find_type(TypeQuery {
                name: Some(name.into()),
                module_name: Some(module_name.into()),
                ..Default::default()
            })
            .map(|value| DynamicManaged::new(Self(value)).ok().unwrap().into())
            .unwrap_or_default()
    }

    #[intuicio_method(transformer = "DynamicManagedValueTransformer")]
    pub fn name(&self) -> String {
        self.0.name().to_owned()
    }

    #[intuicio_method(transformer = "DynamicManagedValueTransformer")]
    pub fn module_name(&self) -> String {
        self.0
            .module_name()
            .map(|value| value.to_owned())
            .unwrap_or_default()
    }

    #[intuicio_method(use_registry, transformer = "DynamicManagedValueTransformer")]
    pub fn function(&self, registry: &Registry, name: String) -> AnkhaOption {
        registry
            .find_function(FunctionQuery {
                name: Some(name.into()),
                type_query: Some(TypeQuery {
                    type_hash: Some(self.0.type_hash()),
                    ..Default::default()
                }),
                ..Default::default()
            })
            .map(|value| DynamicManaged::new(Function(value)).ok().unwrap().into())
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone)]
pub struct Function(pub FunctionHandle);

#[intuicio_methods(module_name = "reflect")]
impl Function {
    #[intuicio_method(use_registry, transformer = "DynamicManagedValueTransformer")]
    pub fn by_name(registry: &Registry, name: String, module_name: String) -> AnkhaOption {
        registry
            .find_function(FunctionQuery {
                name: Some(name.into()),
                module_name: Some(module_name.into()),
                ..Default::default()
            })
            .map(|value| DynamicManaged::new(Self(value)).ok().unwrap().into())
            .unwrap_or_default()
    }

    #[intuicio_method(transformer = "DynamicManagedValueTransformer")]
    pub fn name(&self) -> String {
        self.0.signature().name.to_owned()
    }

    #[intuicio_method(transformer = "DynamicManagedValueTransformer")]
    pub fn module_name(&self) -> String {
        self.0
            .signature()
            .module_name
            .as_ref()
            .map(|value| value.to_owned())
            .unwrap_or_default()
    }

    #[intuicio_method(name = "type", transformer = "DynamicManagedValueTransformer")]
    pub fn type_(&self) -> AnkhaOption {
        self.0
            .signature()
            .type_handle
            .as_ref()
            .map(|value| AnkhaOption::Owned(DynamicManaged::new(Type(value.clone())).ok().unwrap()))
            .unwrap_or_default()
    }
}
