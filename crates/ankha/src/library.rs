use crate::script::AnkhaExpression;
use intuicio_backend_vm::{debugger::PrintDebugger, scope::VmScope};
use intuicio_core::{
    registry::Registry, script::ScriptBuilder, types::struct_type::NativeStructBuilder,
};
use intuicio_data::{
    managed::{DynamicManaged, DynamicManagedLazy, DynamicManagedRef, DynamicManagedRefMut},
    managed_box::DynamicManagedBox,
};

pub type AnkhaVmScope<'a> = VmScope<'a, AnkhaExpression>;
pub type AnkhaScriptBuilder<'a> = ScriptBuilder<'a, AnkhaExpression>;

pub fn install(registry: &mut Registry) {
    registry.add_type(NativeStructBuilder::new_uninitialized::<DynamicManaged>().build());
    registry.add_type(NativeStructBuilder::new_uninitialized::<DynamicManagedRef>().build());
    registry.add_type(NativeStructBuilder::new_uninitialized::<DynamicManagedRefMut>().build());
    registry.add_type(NativeStructBuilder::new_uninitialized::<DynamicManagedLazy>().build());
    registry.add_type(NativeStructBuilder::new_uninitialized::<DynamicManagedBox>().build());
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
