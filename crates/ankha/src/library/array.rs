use crate::library::option::{AnkhaAsyncOption, AnkhaOption};
use intuicio_core::{
    IntuicioStruct,
    registry::Registry,
    transformer::{DynamicManagedValueTransformer, ValueTransformer},
};
use intuicio_derive::{IntuicioStruct, intuicio_method, intuicio_methods};

pub fn install(registry: &mut Registry) {
    registry.add_type(Array::define_struct(registry));
    registry.add_function(Array::with_capacity__define_function(registry));
    registry.add_function(Array::reserve__define_function(registry));
    registry.add_function(Array::resize__define_function(registry));
    registry.add_function(Array::is_empty__define_function(registry));
    registry.add_function(Array::size__define_function(registry));
    registry.add_function(Array::exists__define_function(registry));
    registry.add_function(Array::is_valid__define_function(registry));
    registry.add_function(Array::get__define_function(registry));
    registry.add_function(Array::get_mut__define_function(registry));
    registry.add_function(Array::get_lazy__define_function(registry));
    registry.add_function(Array::push__define_function(registry));
    registry.add_function(Array::pop__define_function(registry));
    registry.add_function(Array::insert__define_function(registry));
    registry.add_function(Array::remove__define_function(registry));
    registry.add_function(Array::clear__define_function(registry));
    registry.add_function(Array::swap_remove__define_function(registry));
    registry.add_function(Array::swap__define_function(registry));
    registry.add_type(AsyncArray::define_struct(registry));
    registry.add_function(AsyncArray::with_capacity__define_function(registry));
    registry.add_function(AsyncArray::reserve__define_function(registry));
    registry.add_function(AsyncArray::resize__define_function(registry));
    registry.add_function(AsyncArray::is_empty__define_function(registry));
    registry.add_function(AsyncArray::size__define_function(registry));
    registry.add_function(AsyncArray::exists__define_function(registry));
    registry.add_function(AsyncArray::is_valid__define_function(registry));
    registry.add_function(AsyncArray::get__define_function(registry));
    registry.add_function(AsyncArray::get_mut__define_function(registry));
    registry.add_function(AsyncArray::get_lazy__define_function(registry));
    registry.add_function(AsyncArray::push__define_function(registry));
    registry.add_function(AsyncArray::pop__define_function(registry));
    registry.add_function(AsyncArray::insert__define_function(registry));
    registry.add_function(AsyncArray::remove__define_function(registry));
    registry.add_function(AsyncArray::clear__define_function(registry));
    registry.add_function(AsyncArray::swap_remove__define_function(registry));
    registry.add_function(AsyncArray::swap__define_function(registry));
}

#[derive(IntuicioStruct, Default)]
#[intuicio(name = "Array", module_name = "array")]
pub struct Array {
    #[intuicio(ignore)]
    items: Vec<AnkhaOption>,
}

impl Array {
    pub fn inner(&self) -> &Vec<AnkhaOption> {
        &self.items
    }

    pub fn inner_mut(&mut self) -> &mut Vec<AnkhaOption> {
        &mut self.items
    }

    pub fn into_inner(self) -> Vec<AnkhaOption> {
        self.items
    }

    pub fn as_slice(&self) -> &[AnkhaOption] {
        self.items.as_slice()
    }

    pub fn as_mut_slice(&mut self) -> &mut [AnkhaOption] {
        self.items.as_mut_slice()
    }

    pub fn iter(&self) -> impl Iterator<Item = &AnkhaOption> {
        self.items.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut AnkhaOption> {
        self.items.iter_mut()
    }
}

impl IntoIterator for Array {
    type Item = AnkhaOption;
    type IntoIter = std::vec::IntoIter<AnkhaOption>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}

#[intuicio_methods(module_name = "array")]
impl Array {
    #[intuicio_method(transformer = "DynamicManagedValueTransformer")]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            items: Vec::with_capacity(capacity),
        }
    }

    #[intuicio_method(transformer = "DynamicManagedValueTransformer")]
    pub fn reserve(&mut self, additional: usize) {
        self.items.reserve_exact(additional);
    }

    #[intuicio_method(transformer = "DynamicManagedValueTransformer")]
    pub fn resize(&mut self, new_size: usize) {
        self.items.resize_with(new_size, || AnkhaOption::None);
    }

    #[intuicio_method(transformer = "DynamicManagedValueTransformer")]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    #[intuicio_method(transformer = "DynamicManagedValueTransformer")]
    pub fn size(&self) -> usize {
        self.items.len()
    }

    #[intuicio_method(transformer = "DynamicManagedValueTransformer")]
    pub fn exists(&self, index: usize) -> bool {
        index < self.size()
    }

    #[intuicio_method(transformer = "DynamicManagedValueTransformer")]
    pub fn is_valid(&self, index: usize) -> bool {
        self.items
            .get(index)
            .map(|item| item.is_some())
            .unwrap_or_default()
    }

    #[intuicio_method(transformer = "DynamicManagedValueTransformer")]
    pub fn get(&self, index: usize) -> AnkhaOption {
        self.items
            .get(index)
            .and_then(|item| item.borrow())
            .map(|item| item.into())
            .unwrap_or_default()
    }

    #[intuicio_method(transformer = "DynamicManagedValueTransformer")]
    pub fn get_mut(&mut self, index: usize) -> AnkhaOption {
        self.items
            .get_mut(index)
            .and_then(|item| item.borrow_mut())
            .map(|item| item.into())
            .unwrap_or_default()
    }

    #[intuicio_method(transformer = "DynamicManagedValueTransformer")]
    pub fn get_lazy(&mut self, index: usize) -> AnkhaOption {
        self.items
            .get_mut(index)
            .and_then(|item| item.lazy())
            .map(|item| item.into())
            .unwrap_or_default()
    }

    #[intuicio_method(transformer = "DynamicManagedValueTransformer")]
    pub fn push(&mut self, value: AnkhaOption) {
        self.items.push(value);
    }

    #[intuicio_method(transformer = "DynamicManagedValueTransformer")]
    pub fn pop(&mut self) -> AnkhaOption {
        self.items.pop().unwrap_or_default()
    }

    #[intuicio_method(transformer = "DynamicManagedValueTransformer")]
    pub fn insert(&mut self, index: usize, value: AnkhaOption) {
        self.items.insert(index, value);
    }

    #[intuicio_method(transformer = "DynamicManagedValueTransformer")]
    pub fn remove(&mut self, index: usize) -> AnkhaOption {
        if index < self.size() {
            self.items.remove(index)
        } else {
            AnkhaOption::None
        }
    }

    #[intuicio_method(transformer = "DynamicManagedValueTransformer")]
    pub fn swap_remove(&mut self, index: usize) -> AnkhaOption {
        if index < self.size() {
            self.items.swap_remove(index)
        } else {
            AnkhaOption::None
        }
    }

    #[intuicio_method(transformer = "DynamicManagedValueTransformer")]
    pub fn clear(&mut self) {
        self.items.clear();
    }

    #[intuicio_method(transformer = "DynamicManagedValueTransformer")]
    pub fn swap(&mut self, from: usize, to: usize) {
        if from < self.size() && to < self.size() {
            self.items.swap(from, to);
        }
    }
}

impl FromIterator<AnkhaOption> for Array {
    fn from_iter<T: IntoIterator<Item = AnkhaOption>>(iter: T) -> Self {
        Self {
            items: iter.into_iter().collect(),
        }
    }
}

impl From<Vec<AnkhaOption>> for Array {
    fn from(value: Vec<AnkhaOption>) -> Self {
        Self { items: value }
    }
}

#[derive(IntuicioStruct, Default)]
#[intuicio(name = "AsyncArray", module_name = "array")]
pub struct AsyncArray {
    #[intuicio(ignore)]
    items: Vec<AnkhaAsyncOption>,
}

impl AsyncArray {
    pub fn inner(&self) -> &Vec<AnkhaAsyncOption> {
        &self.items
    }

    pub fn inner_mut(&mut self) -> &mut Vec<AnkhaAsyncOption> {
        &mut self.items
    }

    pub fn into_inner(self) -> Vec<AnkhaAsyncOption> {
        self.items
    }

    pub fn as_slice(&self) -> &[AnkhaAsyncOption] {
        self.items.as_slice()
    }

    pub fn as_mut_slice(&mut self) -> &mut [AnkhaAsyncOption] {
        self.items.as_mut_slice()
    }

    pub fn iter(&self) -> impl Iterator<Item = &AnkhaAsyncOption> {
        self.items.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut AnkhaAsyncOption> {
        self.items.iter_mut()
    }
}

impl IntoIterator for AsyncArray {
    type Item = AnkhaAsyncOption;
    type IntoIter = std::vec::IntoIter<AnkhaAsyncOption>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}

#[intuicio_methods(module_name = "array")]
impl AsyncArray {
    #[intuicio_method(transformer = "DynamicManagedValueTransformer")]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            items: Vec::with_capacity(capacity),
        }
    }

    #[intuicio_method(transformer = "DynamicManagedValueTransformer")]
    pub fn reserve(&mut self, additional: usize) {
        self.items.reserve_exact(additional);
    }

    #[intuicio_method(transformer = "DynamicManagedValueTransformer")]
    pub fn resize(&mut self, new_size: usize) {
        self.items.resize_with(new_size, || AnkhaAsyncOption::None);
    }

    #[intuicio_method(transformer = "DynamicManagedValueTransformer")]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    #[intuicio_method(transformer = "DynamicManagedValueTransformer")]
    pub fn size(&self) -> usize {
        self.items.len()
    }

    #[intuicio_method(transformer = "DynamicManagedValueTransformer")]
    pub fn exists(&self, index: usize) -> bool {
        index < self.size()
    }

    #[intuicio_method(transformer = "DynamicManagedValueTransformer")]
    pub fn is_valid(&self, index: usize) -> bool {
        self.items
            .get(index)
            .map(|item| item.is_some())
            .unwrap_or_default()
    }

    #[intuicio_method(transformer = "DynamicManagedValueTransformer")]
    pub fn get(&self, index: usize) -> AnkhaAsyncOption {
        self.items
            .get(index)
            .and_then(|item| item.borrow())
            .map(|item| item.into())
            .unwrap_or_default()
    }

    #[intuicio_method(transformer = "DynamicManagedValueTransformer")]
    pub fn get_mut(&mut self, index: usize) -> AnkhaAsyncOption {
        self.items
            .get_mut(index)
            .and_then(|item| item.borrow_mut())
            .map(|item| item.into())
            .unwrap_or_default()
    }

    #[intuicio_method(transformer = "DynamicManagedValueTransformer")]
    pub fn get_lazy(&mut self, index: usize) -> AnkhaAsyncOption {
        self.items
            .get_mut(index)
            .and_then(|item| item.lazy())
            .map(|item| item.into())
            .unwrap_or_default()
    }

    #[intuicio_method(transformer = "DynamicManagedValueTransformer")]
    pub fn push(&mut self, value: AnkhaAsyncOption) {
        self.items.push(value);
    }

    #[intuicio_method(transformer = "DynamicManagedValueTransformer")]
    pub fn pop(&mut self) -> AnkhaAsyncOption {
        self.items.pop().unwrap_or_default()
    }

    #[intuicio_method(transformer = "DynamicManagedValueTransformer")]
    pub fn insert(&mut self, index: usize, value: AnkhaAsyncOption) {
        self.items.insert(index, value);
    }

    #[intuicio_method(transformer = "DynamicManagedValueTransformer")]
    pub fn remove(&mut self, index: usize) -> AnkhaAsyncOption {
        if index < self.size() {
            self.items.remove(index)
        } else {
            AnkhaAsyncOption::None
        }
    }

    #[intuicio_method(transformer = "DynamicManagedValueTransformer")]
    pub fn swap_remove(&mut self, index: usize) -> AnkhaAsyncOption {
        if index < self.size() {
            self.items.swap_remove(index)
        } else {
            AnkhaAsyncOption::None
        }
    }

    #[intuicio_method(transformer = "DynamicManagedValueTransformer")]
    pub fn clear(&mut self) {
        self.items.clear();
    }

    #[intuicio_method(transformer = "DynamicManagedValueTransformer")]
    pub fn swap(&mut self, from: usize, to: usize) {
        if from < self.size() && to < self.size() {
            self.items.swap(from, to);
        }
    }
}

impl FromIterator<AnkhaAsyncOption> for AsyncArray {
    fn from_iter<T: IntoIterator<Item = AnkhaAsyncOption>>(iter: T) -> Self {
        Self {
            items: iter.into_iter().collect(),
        }
    }
}

impl From<Vec<AnkhaAsyncOption>> for AsyncArray {
    fn from(value: Vec<AnkhaAsyncOption>) -> Self {
        Self { items: value }
    }
}
