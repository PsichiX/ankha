use crate::library::{closure::AsyncClosure, option::AnkhaAsyncOption};
use intuicio_core::{
    context::Context,
    registry::Registry,
    transformer::{DynamicManagedValueTransformer, ValueTransformer},
    types::struct_type::NativeStructBuilder,
};
use intuicio_data::{lifetime::Lifetime, managed::DynamicManagedLazy};
use intuicio_derive::{intuicio_method, intuicio_methods};
use std::sync::{Arc, RwLock};

pub fn install(registry: &mut Registry) {
    registry.add_type(
        NativeStructBuilder::new_named_uninitialized::<PromiseResolver>("PromiseResolver")
            .module_name("promise")
            .build(),
    );
    registry.add_function(PromiseResolver::resolve__define_function(registry));
    registry.add_function(PromiseResolver::reject__define_function(registry));
    registry.add_type(
        NativeStructBuilder::new_named_uninitialized::<Promise>("Promise")
            .module_name("promise")
            .build(),
    );
    registry.add_function(Promise::new__define_function(registry));
    registry.add_function(Promise::execute__define_function(registry));
    registry.add_function(Promise::on_success__define_function(registry));
    registry.add_function(Promise::on_failure__define_function(registry));
    registry.add_function(Promise::resolved__define_function(registry));
    registry.add_function(Promise::rejected__define_function(registry));
}

#[derive(Default)]
enum PromiseEffect {
    #[default]
    None,
    Closure(AsyncClosure),
    Promise(Arc<RwLock<Promise>>),
}

#[derive(Default)]
pub struct PromiseResolver {
    on_resolved: PromiseEffect,
    on_rejected: PromiseEffect,
}

#[intuicio_methods(module_name = "promise")]
impl PromiseResolver {
    #[intuicio_method(
        use_context,
        use_registry,
        transformer = "DynamicManagedValueTransformer"
    )]
    pub fn resolve(&mut self, context: &mut Context, registry: &Registry, value: AnkhaAsyncOption) {
        match &mut self.on_resolved {
            PromiseEffect::None => {}
            PromiseEffect::Closure(closure) => {
                closure.call(context, registry, [value].into_iter().collect());
            }
            PromiseEffect::Promise(promise) => {
                if let Ok(mut promise) = promise.write() {
                    promise.execute(context, registry, value);
                }
            }
        }
    }

    #[intuicio_method(
        use_context,
        use_registry,
        transformer = "DynamicManagedValueTransformer"
    )]
    pub fn reject(&mut self, context: &mut Context, registry: &Registry, value: AnkhaAsyncOption) {
        match &mut self.on_rejected {
            PromiseEffect::None => {}
            PromiseEffect::Closure(closure) => {
                closure.call(context, registry, [value].into_iter().collect());
            }
            PromiseEffect::Promise(promise) => {
                if let Ok(mut promise) = promise.write() {
                    promise.execute(context, registry, value);
                }
            }
        }
    }
}

pub struct Promise {
    closure: AsyncClosure,
    resolver_lifetime: Lifetime,
    resolver: PromiseResolver,
}

#[intuicio_methods(module_name = "promise")]
impl Promise {
    #[intuicio_method(transformer = "DynamicManagedValueTransformer")]
    pub fn new(closure: AsyncClosure) -> Self {
        Self {
            closure,
            resolver_lifetime: Default::default(),
            resolver: Default::default(),
        }
    }

    #[intuicio_method(
        use_context,
        use_registry,
        transformer = "DynamicManagedValueTransformer"
    )]
    pub fn execute(&mut self, context: &mut Context, registry: &Registry, value: AnkhaAsyncOption) {
        self.closure.call(
            context,
            registry,
            [
                DynamicManagedLazy::new(&mut self.resolver, self.resolver_lifetime.lazy()).into(),
                value,
            ]
            .into_iter()
            .collect(),
        );
    }

    #[intuicio_method(transformer = "DynamicManagedValueTransformer")]
    pub fn on_success(mut self, closure: AsyncClosure) -> Self {
        self.resolver.on_resolved = PromiseEffect::Closure(closure);
        self
    }

    #[intuicio_method(transformer = "DynamicManagedValueTransformer")]
    pub fn on_failure(mut self, closure: AsyncClosure) -> Self {
        self.resolver.on_rejected = PromiseEffect::Closure(closure);
        self
    }

    #[intuicio_method(transformer = "DynamicManagedValueTransformer")]
    pub fn resolved(mut self, promise: Promise) -> Self {
        self.resolver.on_resolved = PromiseEffect::Promise(Arc::new(RwLock::new(promise)));
        self
    }

    #[intuicio_method(transformer = "DynamicManagedValueTransformer")]
    pub fn rejected(mut self, promise: Promise) -> Self {
        self.resolver.on_rejected = PromiseEffect::Promise(Arc::new(RwLock::new(promise)));
        self
    }
}
