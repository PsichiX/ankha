pub mod array;
pub mod channel;
pub mod closure;
pub mod dictionary;
pub mod event;
pub mod option;
pub mod promise;
pub mod reflection;
pub mod thread;

use crate::script::AnkhaExpression;
use intuicio_backend_vm::{debugger::PrintDebugger, scope::VmScope};
use intuicio_core::{
    registry::Registry, script::ScriptBuilder, types::struct_type::NativeStructBuilder,
};
use intuicio_data::{
    data_stack::DataStack,
    managed::{
        DynamicManaged, DynamicManagedLazy, DynamicManagedRef, DynamicManagedRefMut, Managed,
        ManagedLazy, ManagedRef, ManagedRefMut,
    },
    managed_box::{DynamicManagedBox, ManagedBox},
};

pub type AnkhaVmScope<'a> = VmScope<'a, AnkhaExpression>;
pub type AnkhaScriptBuilder<'a> = ScriptBuilder<'a, AnkhaExpression>;

pub fn install(registry: &mut Registry) {
    registry.add_type(NativeStructBuilder::new_uninitialized::<DynamicManaged>().build());
    registry.add_type(NativeStructBuilder::new_uninitialized::<DynamicManagedRef>().build());
    registry.add_type(NativeStructBuilder::new_uninitialized::<DynamicManagedRefMut>().build());
    registry.add_type(NativeStructBuilder::new_uninitialized::<DynamicManagedLazy>().build());
    registry.add_type(NativeStructBuilder::new_uninitialized::<DynamicManagedBox>().build());
    crate::library::option::install(registry);
    crate::library::array::install(registry);
    crate::library::dictionary::install(registry);
    crate::library::reflection::install(registry);
    crate::library::channel::install(registry);
    crate::library::event::install(registry);
    crate::library::closure::install(registry);
    crate::library::promise::install(registry);
    crate::library::thread::install(registry);
}

pub fn printables(debugger: PrintDebugger) -> PrintDebugger {
    debugger
        .printable_custom::<DynamicManaged>(|debugger, value| {
            if let Some((type_name, value)) = debugger.display_raw(*value.type_hash(), unsafe {
                value.as_ptr_raw().cast::<()>()
            }) {
                format!("DynamicManaged <{}>:\n{}", type_name, value)
            } else {
                "DynamicManaged".to_owned()
            }
        })
        .printable_custom::<DynamicManagedRef>(|debugger, value| {
            if let Some((type_name, value)) = debugger.display_raw(*value.type_hash(), unsafe {
                value.as_ptr_raw().unwrap().cast::<()>()
            }) {
                format!("DynamicManagedRef <{}>:\n{}", type_name, value)
            } else {
                "DynamicManagedRef".to_owned()
            }
        })
        .printable_custom::<DynamicManagedRefMut>(|debugger, value| {
            if let Some((type_name, value)) = debugger.display_raw(*value.type_hash(), unsafe {
                value.as_ptr_raw().unwrap().cast::<()>()
            }) {
                format!("DynamicManagedRefMut <{}>:\n{}", type_name, value)
            } else {
                "DynamicManagedRefMut".to_owned()
            }
        })
        .printable_custom::<DynamicManagedLazy>(|debugger, value| {
            if let Some((type_name, value)) = debugger.display_raw(*value.type_hash(), unsafe {
                value.as_ptr_raw().unwrap().cast::<()>()
            }) {
                format!("DynamicManagedLazy <{}>:\n{}", type_name, value)
            } else {
                "DynamicManagedLazy".to_owned()
            }
        })
        .printable_custom::<DynamicManagedBox>(|debugger, value| {
            if let Some((type_name, value)) = debugger
                .display_raw(value.type_hash().unwrap(), unsafe {
                    value.as_ptr_raw().unwrap().cast::<()>()
                })
            {
                format!("DynamicManagedBox <{}>:\n{}", type_name, value)
            } else {
                "DynamicManagedBox".to_owned()
            }
        })
}

pub struct AnkhaManagedStack<'a> {
    stack: &'a mut DataStack,
}

impl<'a> AnkhaManagedStack<'a> {
    pub fn new(stack: &'a mut DataStack) -> Self {
        Self { stack }
    }

    pub fn push<T>(&mut self, data: Managed<T>) {
        self.stack.push(
            data.into_dynamic()
                .expect("Could not convert into dynamic owned value!"),
        );
    }

    pub fn push_ref<T>(&mut self, data: ManagedRef<T>) {
        self.stack.push(data.into_dynamic());
    }

    pub fn push_ref_mut<T>(&mut self, data: ManagedRefMut<T>) {
        self.stack.push(data.into_dynamic());
    }

    pub fn push_lazy<T>(&mut self, data: ManagedLazy<T>) {
        self.stack.push(data.into_dynamic());
    }

    pub fn push_box<T>(&mut self, data: ManagedBox<T>) {
        self.stack.push(data.into_dynamic());
    }

    pub fn pop<T>(&mut self) -> Option<Managed<T>> {
        self.stack.pop::<DynamicManaged>()?.into_typed().ok()
    }

    pub fn pop_ref<T>(&mut self) -> Option<ManagedRef<T>> {
        self.stack.pop::<DynamicManagedRef>()?.into_typed().ok()
    }

    pub fn pop_ref_mut<T>(&mut self) -> Option<ManagedRefMut<T>> {
        self.stack.pop::<DynamicManagedRefMut>()?.into_typed().ok()
    }

    pub fn pop_lazy<T>(&mut self) -> Option<ManagedLazy<T>> {
        self.stack.pop::<DynamicManagedLazy>()?.into_typed().ok()
    }

    pub fn pop_box<T>(&mut self) -> Option<ManagedBox<T>> {
        self.stack.pop::<DynamicManagedBox>()?.into_typed().ok()
    }
}
