use crate::{
    library::{
        array::{Array, AsyncArray},
        option::{AnkhaAsyncOption, AnkhaOption},
        reflection::Function,
    },
    script::stack_managed_variant,
};
use intuicio_core::{
    context::Context,
    registry::Registry,
    transformer::{DynamicManagedValueTransformer, ValueTransformer},
    types::struct_type::NativeStructBuilder,
};
use intuicio_derive::{intuicio_method, intuicio_methods};

pub fn install(registry: &mut Registry) {
    registry.add_type(
        NativeStructBuilder::new_named_uninitialized::<Closure>("Closure")
            .module_name("closure")
            .build(),
    );
    registry.add_function(Closure::new__define_function(registry));
    registry.add_function(Closure::from_function__define_function(registry));
    registry.add_function(Closure::call__define_function(registry));
    registry.add_type(
        NativeStructBuilder::new_named_uninitialized::<AsyncClosure>("AsyncClosure")
            .module_name("closure")
            .build(),
    );
    registry.add_function(AsyncClosure::new__define_function(registry));
    registry.add_function(AsyncClosure::from_function__define_function(registry));
    registry.add_function(AsyncClosure::call__define_function(registry));
}

pub struct Closure {
    function: Function,
    captured: Array,
}

#[intuicio_methods(module_name = "closure")]
impl Closure {
    #[intuicio_method(transformer = "DynamicManagedValueTransformer")]
    pub fn new(function: Function, captured: Array) -> Self {
        Self { function, captured }
    }

    #[intuicio_method(transformer = "DynamicManagedValueTransformer")]
    pub fn from_function(function: Function) -> Self {
        Self {
            function,
            captured: Default::default(),
        }
    }

    pub fn invoke(
        &mut self,
        context: &mut Context,
        registry: &Registry,
        arguments: Vec<AnkhaOption>,
    ) {
        for argument in arguments.into_iter().rev() {
            match argument {
                AnkhaOption::None => {
                    panic!("Cannot use none value as closure argument!");
                }
                AnkhaOption::Owned(value) => {
                    context.stack().push(value);
                }
                AnkhaOption::Ref(value) => {
                    context.stack().push(value);
                }
                AnkhaOption::RefMut(value) => {
                    context.stack().push(value);
                }
                AnkhaOption::Lazy(value) => {
                    context.stack().push(value);
                }
                AnkhaOption::Box(value) => {
                    context.stack().push(value);
                }
            }
        }
        for argument in self.captured.inner_mut().iter_mut().rev() {
            match argument {
                AnkhaOption::None => {
                    panic!("Cannot use none value as closure capture!");
                }
                AnkhaOption::Owned(_) => {
                    panic!("Cannot use owned value as closure capture!");
                }
                AnkhaOption::Ref(value) => {
                    context
                        .stack()
                        .push(value.borrow().expect("Could not borrow ref value!"));
                }
                AnkhaOption::RefMut(value) => {
                    context.stack().push(
                        value
                            .borrow_mut()
                            .expect("Could not borrow mutably ref mut value!"),
                    );
                }
                AnkhaOption::Lazy(value) => {
                    context.stack().push(value.clone());
                }
                AnkhaOption::Box(value) => {
                    context.stack().push(value.clone());
                }
            }
        }
        self.function.0.invoke(context, registry);
    }

    #[intuicio_method(
        use_context,
        use_registry,
        transformer = "DynamicManagedValueTransformer"
    )]
    pub fn call(
        &mut self,
        context: &mut Context,
        registry: &Registry,
        arguments: Array,
    ) -> AnkhaOption {
        self.invoke(context, registry, arguments.into_inner());
        if self.function.0.signature().outputs.len() == 1 {
            stack_managed_variant(
                context,
                |_, value| value.into(),
                |_, value| value.into(),
                |_, value| value.into(),
                |_, value| value.into(),
                |_, value| value.into(),
            )
        } else {
            AnkhaOption::None
        }
    }
}

pub struct AsyncClosure {
    function: Function,
    captured: AsyncArray,
}

#[intuicio_methods(module_name = "closure")]
impl AsyncClosure {
    #[intuicio_method(transformer = "DynamicManagedValueTransformer")]
    pub fn new(function: Function, captured: AsyncArray) -> Self {
        Self { function, captured }
    }

    #[intuicio_method(transformer = "DynamicManagedValueTransformer")]
    pub fn from_function(function: Function) -> Self {
        Self {
            function,
            captured: Default::default(),
        }
    }

    pub fn invoke(
        &mut self,
        context: &mut Context,
        registry: &Registry,
        arguments: Vec<AnkhaAsyncOption>,
    ) {
        for argument in arguments.into_iter().rev() {
            match argument {
                AnkhaAsyncOption::None => {
                    panic!("Cannot use none value as closure argument!");
                }
                AnkhaAsyncOption::Owned(value) => {
                    context.stack().push(value);
                }
                AnkhaAsyncOption::Ref(value) => {
                    context.stack().push(value);
                }
                AnkhaAsyncOption::RefMut(value) => {
                    context.stack().push(value);
                }
                AnkhaAsyncOption::Lazy(value) => {
                    context.stack().push(value);
                }
            }
        }
        for argument in self.captured.inner_mut().iter_mut().rev() {
            match argument {
                AnkhaAsyncOption::None => {
                    panic!("Cannot use none value as closure capture!");
                }
                AnkhaAsyncOption::Owned(_) => {
                    panic!("Cannot use owned value as closure capture!");
                }
                AnkhaAsyncOption::Ref(value) => {
                    context
                        .stack()
                        .push(value.borrow().expect("Could not borrow ref value!"));
                }
                AnkhaAsyncOption::RefMut(value) => {
                    context.stack().push(
                        value
                            .borrow_mut()
                            .expect("Could not borrow mutably ref mut value!"),
                    );
                }
                AnkhaAsyncOption::Lazy(value) => {
                    context.stack().push(value.clone());
                }
            }
        }
        self.function.0.invoke(context, registry);
    }

    #[intuicio_method(
        use_context,
        use_registry,
        transformer = "DynamicManagedValueTransformer"
    )]
    pub fn call(
        &mut self,
        context: &mut Context,
        registry: &Registry,
        arguments: AsyncArray,
    ) -> AnkhaAsyncOption {
        self.invoke(context, registry, arguments.into_inner());
        if self.function.0.signature().outputs.len() == 1 {
            stack_managed_variant(
                context,
                |_, value| value.into(),
                |_, value| value.into(),
                |_, value| value.into(),
                |_, value| value.into(),
                |_, _| panic!("Async closure cannot return box value!"),
            )
        } else {
            AnkhaAsyncOption::None
        }
    }
}
