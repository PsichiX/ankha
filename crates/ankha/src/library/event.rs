use crate::library::{
    array::{Array, AsyncArray},
    closure::{AsyncClosure, Closure},
    option::{AnkhaAsyncOption, AnkhaOption},
};
use intuicio_core::{
    IntuicioStruct,
    context::Context,
    registry::Registry,
    transformer::{DynamicManagedValueTransformer, ValueTransformer},
};
use intuicio_derive::{IntuicioStruct, intuicio_method, intuicio_methods};
use std::collections::HashMap;
use typid::ID;

pub fn install(registry: &mut Registry) {
    registry.add_type(EventHandle::define_struct(registry));
    registry.add_type(Event::define_struct(registry));
    registry.add_function(Event::is_bound__define_function(registry));
    registry.add_function(Event::subscribe__define_function(registry));
    registry.add_function(Event::unsubscribe__define_function(registry));
    registry.add_function(Event::clear__define_function(registry));
    registry.add_type(AsyncEvent::define_struct(registry));
    registry.add_function(AsyncEvent::is_bound__define_function(registry));
    registry.add_function(AsyncEvent::subscribe__define_function(registry));
    registry.add_function(AsyncEvent::unsubscribe__define_function(registry));
    registry.add_function(AsyncEvent::clear__define_function(registry));
}

#[derive(IntuicioStruct, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[intuicio(module_name = "event")]
pub struct EventHandle {
    #[intuicio(ignore)]
    id: ID<Event>,
}

#[derive(IntuicioStruct, Default)]
#[intuicio(module_name = "event")]
pub struct Event {
    #[intuicio(ignore)]
    callbacks: HashMap<EventHandle, Closure>,
}

#[intuicio_methods(module_name = "event")]
impl Event {
    #[intuicio_method(transformer = "DynamicManagedValueTransformer")]
    pub fn is_bound(&self) -> bool {
        !self.callbacks.is_empty()
    }

    #[intuicio_method(transformer = "DynamicManagedValueTransformer")]
    pub fn subscribe(&mut self, closure: Closure) -> EventHandle {
        let handle = EventHandle::default();
        self.callbacks.insert(handle, closure);
        handle
    }

    #[intuicio_method(transformer = "DynamicManagedValueTransformer")]
    pub fn unsubscribe(&mut self, handle: EventHandle) {
        self.callbacks.remove(&handle);
    }

    #[intuicio_method(transformer = "DynamicManagedValueTransformer")]
    pub fn clear(&mut self) {
        self.callbacks.clear();
    }

    #[intuicio_method(
        use_context,
        use_registry,
        transformer = "DynamicManagedValueTransformer"
    )]
    pub fn dispatch(&mut self, context: &mut Context, registry: &Registry, mut arguments: Array) {
        for callback in self.callbacks.values_mut() {
            callback.call(
                context,
                registry,
                arguments
                    .iter_mut()
                    .map(|item| match item {
                        AnkhaOption::None => panic!("Cannot dispatch event with none argument!"),
                        AnkhaOption::Owned(_) => {
                            panic!("Cannot dispatch event with owned argument!")
                        }
                        AnkhaOption::Ref(value) => {
                            AnkhaOption::Ref(value.borrow().expect("Could not borrow ref value!"))
                        }
                        AnkhaOption::RefMut(value) => AnkhaOption::RefMut(
                            value
                                .borrow_mut()
                                .expect("Could not borrow mutably ref mut value!"),
                        ),
                        AnkhaOption::Lazy(value) => AnkhaOption::Ref(
                            value.borrow().expect("Could not borrow lazily lazy value!"),
                        ),
                        AnkhaOption::Box(value) => AnkhaOption::Box(value.clone()),
                    })
                    .collect(),
            );
        }
    }
}

#[derive(IntuicioStruct, Default)]
#[intuicio(module_name = "event")]
pub struct AsyncEvent {
    #[intuicio(ignore)]
    callbacks: HashMap<EventHandle, AsyncClosure>,
}

#[intuicio_methods(module_name = "event")]
impl AsyncEvent {
    #[intuicio_method(transformer = "DynamicManagedValueTransformer")]
    pub fn is_bound(&self) -> bool {
        !self.callbacks.is_empty()
    }

    #[intuicio_method(transformer = "DynamicManagedValueTransformer")]
    pub fn subscribe(&mut self, closure: AsyncClosure) -> EventHandle {
        let handle = EventHandle::default();
        self.callbacks.insert(handle, closure);
        handle
    }

    #[intuicio_method(transformer = "DynamicManagedValueTransformer")]
    pub fn unsubscribe(&mut self, handle: EventHandle) {
        self.callbacks.remove(&handle);
    }

    #[intuicio_method(transformer = "DynamicManagedValueTransformer")]
    pub fn clear(&mut self) {
        self.callbacks.clear();
    }

    #[intuicio_method(
        use_context,
        use_registry,
        transformer = "DynamicManagedValueTransformer"
    )]
    pub fn dispatch(
        &mut self,
        context: &mut Context,
        registry: &Registry,
        mut arguments: AsyncArray,
    ) {
        for callback in self.callbacks.values_mut() {
            callback.call(
                context,
                registry,
                arguments
                    .iter_mut()
                    .map(|item| match item {
                        AnkhaAsyncOption::None => {
                            panic!("Cannot dispatch event with none argument!")
                        }
                        AnkhaAsyncOption::Owned(_) => {
                            panic!("Cannot dispatch event with owned argument!")
                        }
                        AnkhaAsyncOption::Ref(value) => AnkhaAsyncOption::Ref(
                            value.borrow().expect("Could not borrow ref value!"),
                        ),
                        AnkhaAsyncOption::RefMut(value) => AnkhaAsyncOption::RefMut(
                            value
                                .borrow_mut()
                                .expect("Could not borrow mutably ref mut value!"),
                        ),
                        AnkhaAsyncOption::Lazy(value) => AnkhaAsyncOption::Ref(
                            value.borrow().expect("Could not borrow lazily lazy value!"),
                        ),
                    })
                    .collect(),
            );
        }
    }
}
