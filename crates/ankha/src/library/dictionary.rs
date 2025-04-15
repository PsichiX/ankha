use crate::{
    library::option::{AnkhaAsyncOption, AnkhaOption},
    script::AnkhaLiteral,
};
use intuicio_core::{
    IntuicioStruct,
    registry::Registry,
    transformer::{DynamicManagedValueTransformer, ValueTransformer},
};
use intuicio_derive::{IntuicioStruct, intuicio_method, intuicio_methods};
use std::collections::HashMap;

pub fn install(registry: &mut Registry) {
    registry.add_type(Dictionary::define_struct(registry));
    registry.add_function(Dictionary::is_empty__define_function(registry));
    registry.add_function(Dictionary::size__define_function(registry));
    registry.add_function(Dictionary::exists__define_function(registry));
    registry.add_function(Dictionary::is_valid__define_function(registry));
    registry.add_function(Dictionary::get__define_function(registry));
    registry.add_function(Dictionary::get_mut__define_function(registry));
    registry.add_function(Dictionary::get_lazy__define_function(registry));
    registry.add_function(Dictionary::insert__define_function(registry));
    registry.add_function(Dictionary::remove__define_function(registry));
    registry.add_function(Dictionary::clear__define_function(registry));
    registry.add_type(AsyncDictionary::define_struct(registry));
    registry.add_function(AsyncDictionary::is_empty__define_function(registry));
    registry.add_function(AsyncDictionary::size__define_function(registry));
    registry.add_function(AsyncDictionary::exists__define_function(registry));
    registry.add_function(AsyncDictionary::is_valid__define_function(registry));
    registry.add_function(AsyncDictionary::get__define_function(registry));
    registry.add_function(AsyncDictionary::get_mut__define_function(registry));
    registry.add_function(AsyncDictionary::get_lazy__define_function(registry));
    registry.add_function(AsyncDictionary::insert__define_function(registry));
    registry.add_function(AsyncDictionary::remove__define_function(registry));
    registry.add_function(AsyncDictionary::clear__define_function(registry));
}

#[derive(IntuicioStruct, Default)]
#[intuicio(name = "Dictionary", module_name = "dictionary")]
pub struct Dictionary {
    #[intuicio(ignore)]
    items: HashMap<AnkhaLiteral, AnkhaOption>,
}

impl Dictionary {
    pub fn inner(&self) -> &HashMap<AnkhaLiteral, AnkhaOption> {
        &self.items
    }

    pub fn inner_mut(&mut self) -> &mut HashMap<AnkhaLiteral, AnkhaOption> {
        &mut self.items
    }

    pub fn iter(&self) -> impl Iterator<Item = (&AnkhaLiteral, &AnkhaOption)> {
        self.items.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&AnkhaLiteral, &mut AnkhaOption)> {
        self.items.iter_mut()
    }

    pub fn keys(&self) -> impl Iterator<Item = &AnkhaLiteral> {
        self.items.keys()
    }

    pub fn into_keys(self) -> impl Iterator<Item = AnkhaLiteral> {
        self.items.into_keys()
    }

    pub fn values(&self) -> impl Iterator<Item = &AnkhaOption> {
        self.items.values()
    }

    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut AnkhaOption> {
        self.items.values_mut()
    }

    pub fn into_values(self) -> impl Iterator<Item = AnkhaOption> {
        self.items.into_values()
    }
}

impl IntoIterator for Dictionary {
    type Item = (AnkhaLiteral, AnkhaOption);
    type IntoIter = std::collections::hash_map::IntoIter<AnkhaLiteral, AnkhaOption>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}

#[intuicio_methods(module_name = "dictionary")]
impl Dictionary {
    #[intuicio_method(transformer = "DynamicManagedValueTransformer")]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    #[intuicio_method(transformer = "DynamicManagedValueTransformer")]
    pub fn size(&self) -> usize {
        self.items.len()
    }

    #[intuicio_method(transformer = "DynamicManagedValueTransformer")]
    pub fn exists(&self, key: AnkhaOption) -> bool {
        key.into_literal()
            .map(|key| self.items.contains_key(&key))
            .unwrap_or_default()
    }

    #[intuicio_method(transformer = "DynamicManagedValueTransformer")]
    pub fn is_valid(&self, key: AnkhaOption) -> bool {
        key.into_literal()
            .and_then(|key| self.items.get(&key))
            .map(|item| item.is_some())
            .unwrap_or_default()
    }

    #[intuicio_method(transformer = "DynamicManagedValueTransformer")]
    pub fn get(&self, key: AnkhaOption) -> AnkhaOption {
        key.into_literal()
            .and_then(|key| self.items.get(&key))
            .and_then(|item| item.borrow())
            .map(|item| item.into())
            .unwrap_or_default()
    }

    #[intuicio_method(transformer = "DynamicManagedValueTransformer")]
    pub fn get_mut(&mut self, key: AnkhaOption) -> AnkhaOption {
        key.into_literal()
            .and_then(|key| self.items.get_mut(&key))
            .and_then(|item| item.borrow_mut())
            .map(|item| item.into())
            .unwrap_or_default()
    }

    #[intuicio_method(transformer = "DynamicManagedValueTransformer")]
    pub fn get_lazy(&mut self, key: AnkhaOption) -> AnkhaOption {
        key.into_literal()
            .and_then(|key| self.items.get_mut(&key))
            .and_then(|item| item.lazy())
            .map(|item| item.into())
            .unwrap_or_default()
    }

    #[intuicio_method(transformer = "DynamicManagedValueTransformer")]
    pub fn insert(&mut self, key: AnkhaOption, value: AnkhaOption) -> AnkhaOption {
        key.into_literal()
            .and_then(|key| self.items.insert(key, value))
            .unwrap_or_default()
    }

    #[intuicio_method(transformer = "DynamicManagedValueTransformer")]
    pub fn remove(&mut self, key: AnkhaOption) -> AnkhaOption {
        key.into_literal()
            .and_then(|key| self.items.remove(&key))
            .unwrap_or_default()
    }

    #[intuicio_method(transformer = "DynamicManagedValueTransformer")]
    pub fn clear(&mut self) {
        self.items.clear();
    }
}

impl FromIterator<(AnkhaLiteral, AnkhaOption)> for Dictionary {
    fn from_iter<T: IntoIterator<Item = (AnkhaLiteral, AnkhaOption)>>(iter: T) -> Self {
        Self {
            items: iter.into_iter().collect(),
        }
    }
}

impl From<HashMap<AnkhaLiteral, AnkhaOption>> for Dictionary {
    fn from(value: HashMap<AnkhaLiteral, AnkhaOption>) -> Self {
        Self { items: value }
    }
}

#[derive(IntuicioStruct, Default)]
#[intuicio(name = "AsyncDictionary", module_name = "dictionary")]
pub struct AsyncDictionary {
    #[intuicio(ignore)]
    items: HashMap<AnkhaLiteral, AnkhaAsyncOption>,
}

impl AsyncDictionary {
    pub fn inner(&self) -> &HashMap<AnkhaLiteral, AnkhaAsyncOption> {
        &self.items
    }

    pub fn inner_mut(&mut self) -> &mut HashMap<AnkhaLiteral, AnkhaAsyncOption> {
        &mut self.items
    }

    pub fn iter(&self) -> impl Iterator<Item = (&AnkhaLiteral, &AnkhaAsyncOption)> {
        self.items.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&AnkhaLiteral, &mut AnkhaAsyncOption)> {
        self.items.iter_mut()
    }

    pub fn keys(&self) -> impl Iterator<Item = &AnkhaLiteral> {
        self.items.keys()
    }

    pub fn into_keys(self) -> impl Iterator<Item = AnkhaLiteral> {
        self.items.into_keys()
    }

    pub fn values(&self) -> impl Iterator<Item = &AnkhaAsyncOption> {
        self.items.values()
    }

    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut AnkhaAsyncOption> {
        self.items.values_mut()
    }

    pub fn into_values(self) -> impl Iterator<Item = AnkhaAsyncOption> {
        self.items.into_values()
    }
}

impl IntoIterator for AsyncDictionary {
    type Item = (AnkhaLiteral, AnkhaAsyncOption);
    type IntoIter = std::collections::hash_map::IntoIter<AnkhaLiteral, AnkhaAsyncOption>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}

#[intuicio_methods(module_name = "dictionary")]
impl AsyncDictionary {
    #[intuicio_method(transformer = "DynamicManagedValueTransformer")]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    #[intuicio_method(transformer = "DynamicManagedValueTransformer")]
    pub fn size(&self) -> usize {
        self.items.len()
    }

    #[intuicio_method(transformer = "DynamicManagedValueTransformer")]
    pub fn exists(&self, key: AnkhaAsyncOption) -> bool {
        key.into_literal()
            .map(|key| self.items.contains_key(&key))
            .unwrap_or_default()
    }

    #[intuicio_method(transformer = "DynamicManagedValueTransformer")]
    pub fn is_valid(&self, key: AnkhaAsyncOption) -> bool {
        key.into_literal()
            .and_then(|key| self.items.get(&key))
            .map(|item| item.is_some())
            .unwrap_or_default()
    }

    #[intuicio_method(transformer = "DynamicManagedValueTransformer")]
    pub fn get(&self, key: AnkhaAsyncOption) -> AnkhaAsyncOption {
        key.into_literal()
            .and_then(|key| self.items.get(&key))
            .and_then(|item| item.borrow())
            .map(|item| item.into())
            .unwrap_or_default()
    }

    #[intuicio_method(transformer = "DynamicManagedValueTransformer")]
    pub fn get_mut(&mut self, key: AnkhaAsyncOption) -> AnkhaAsyncOption {
        key.into_literal()
            .and_then(|key| self.items.get_mut(&key))
            .and_then(|item| item.borrow_mut())
            .map(|item| item.into())
            .unwrap_or_default()
    }

    #[intuicio_method(transformer = "DynamicManagedValueTransformer")]
    pub fn get_lazy(&mut self, key: AnkhaAsyncOption) -> AnkhaAsyncOption {
        key.into_literal()
            .and_then(|key| self.items.get_mut(&key))
            .and_then(|item| item.lazy())
            .map(|item| item.into())
            .unwrap_or_default()
    }

    #[intuicio_method(transformer = "DynamicManagedValueTransformer")]
    pub fn insert(&mut self, key: AnkhaAsyncOption, value: AnkhaAsyncOption) -> AnkhaAsyncOption {
        key.into_literal()
            .and_then(|key| self.items.insert(key, value))
            .unwrap_or_default()
    }

    #[intuicio_method(transformer = "DynamicManagedValueTransformer")]
    pub fn remove(&mut self, key: AnkhaAsyncOption) -> AnkhaAsyncOption {
        key.into_literal()
            .and_then(|key| self.items.remove(&key))
            .unwrap_or_default()
    }

    #[intuicio_method(transformer = "DynamicManagedValueTransformer")]
    pub fn clear(&mut self) {
        self.items.clear();
    }
}

impl FromIterator<(AnkhaLiteral, AnkhaAsyncOption)> for AsyncDictionary {
    fn from_iter<T: IntoIterator<Item = (AnkhaLiteral, AnkhaAsyncOption)>>(iter: T) -> Self {
        Self {
            items: iter.into_iter().collect(),
        }
    }
}

impl From<HashMap<AnkhaLiteral, AnkhaAsyncOption>> for AsyncDictionary {
    fn from(value: HashMap<AnkhaLiteral, AnkhaAsyncOption>) -> Self {
        Self { items: value }
    }
}
