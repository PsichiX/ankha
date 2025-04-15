use crate::library::{array::AsyncArray, closure::AsyncClosure, option::AnkhaAsyncOption};
use intuicio_core::{
    context::Context,
    registry::Registry,
    transformer::{DynamicManagedValueTransformer, ValueTransformer},
    types::struct_type::NativeStructBuilder,
};
use intuicio_derive::{intuicio_method, intuicio_methods};
use std::thread::{JoinHandle, spawn as std_spawn};

pub fn install(registry: &mut Registry) {
    registry.add_type(
        NativeStructBuilder::new_named_uninitialized::<Thread>("Thread")
            .module_name("thread")
            .build(),
    );
    registry.add_function(Thread::new__define_function(registry));
    registry.add_function(Thread::is_finished__define_function(registry));
    registry.add_function(Thread::join__define_function(registry));
}

pub struct Thread {
    handle: JoinHandle<AnkhaAsyncOption>,
}

#[intuicio_methods(module_name = "thread")]
impl Thread {
    #[intuicio_method(
        use_context,
        use_registry,
        transformer = "DynamicManagedValueTransformer"
    )]
    pub fn new(
        context: &mut Context,
        registry: &Registry,
        mut closure: AsyncClosure,
        arguments: AsyncArray,
    ) -> Self {
        let mut context = context.fork();
        let registry = registry.clone();
        let handle = std_spawn(move || {
            closure.call(&mut context, &registry, arguments);
            AnkhaAsyncOption::None
        });
        Self { handle }
    }

    #[intuicio_method(transformer = "DynamicManagedValueTransformer")]
    pub fn is_finished(&self) -> bool {
        self.handle.is_finished()
    }

    #[intuicio_method(transformer = "DynamicManagedValueTransformer")]
    pub fn join(self) -> AnkhaAsyncOption {
        self.handle.join().ok().unwrap_or_default()
    }
}
