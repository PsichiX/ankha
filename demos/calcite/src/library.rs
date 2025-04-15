use intuicio_core::{
    registry::Registry,
    transformer::{DynamicManagedValueTransformer, ValueTransformer},
};
use intuicio_derive::intuicio_function;

#[intuicio_function(transformer = "DynamicManagedValueTransformer")]
fn add(a: f64, b: f64) -> f64 {
    a + b
}

#[intuicio_function(transformer = "DynamicManagedValueTransformer")]
fn sub(a: f64, b: f64) -> f64 {
    a - b
}

#[intuicio_function(transformer = "DynamicManagedValueTransformer")]
fn mul(a: f64, b: f64) -> f64 {
    a * b
}

#[intuicio_function(transformer = "DynamicManagedValueTransformer")]
fn div(a: f64, b: f64) -> f64 {
    a / b
}

#[intuicio_function(transformer = "DynamicManagedValueTransformer")]
fn floor(v: f64) -> f64 {
    v.floor()
}

#[intuicio_function(transformer = "DynamicManagedValueTransformer")]
fn fract(v: f64) -> f64 {
    v.fract()
}

pub fn install(registry: &mut Registry) {
    registry.add_function(add::define_function(registry));
    registry.add_function(sub::define_function(registry));
    registry.add_function(mul::define_function(registry));
    registry.add_function(div::define_function(registry));
    registry.add_function(floor::define_function(registry));
    registry.add_function(fract::define_function(registry));
}
