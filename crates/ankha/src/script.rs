use crate::library::reflection::{Function, Type};
use intuicio_core::{
    IntuicioVersion, Visibility,
    context::Context,
    crate_version,
    function::{FunctionQuery, FunctionQueryParameter},
    meta::Meta,
    object::Object,
    registry::Registry,
    script::{
        ScriptContentProvider, ScriptEnum, ScriptEnumVariant, ScriptExpression, ScriptFunction,
        ScriptFunctionParameter, ScriptFunctionSignature, ScriptHandle, ScriptModule,
        ScriptOperation, ScriptPackage, ScriptStruct, ScriptStructField,
    },
    types::{StructFieldQuery, TypeQuery},
};
use intuicio_data::{
    lifetime::Lifetime,
    managed::{DynamicManaged, DynamicManagedLazy, DynamicManagedRef, DynamicManagedRefMut},
    managed_box::DynamicManagedBox,
    type_hash::TypeHash,
};
use serde::{Deserialize, Serialize};
use std::{
    alloc::dealloc,
    collections::HashMap,
    error::Error,
    hash::{Hash, Hasher},
};

pub type AnkhaScript = Vec<AnkhaOperation>;

pub fn frontend_ankha_version() -> IntuicioVersion {
    crate_version!()
}

pub(crate) fn stack_managed_variant<R>(
    context: &mut Context,
    owned_variant: impl FnOnce(&mut Context, DynamicManaged) -> R,
    ref_variant: impl FnOnce(&mut Context, DynamicManagedRef) -> R,
    ref_mut_variant: impl FnOnce(&mut Context, DynamicManagedRefMut) -> R,
    lazy_variant: impl FnOnce(&mut Context, DynamicManagedLazy) -> R,
    box_variant: impl FnOnce(&mut Context, DynamicManagedBox) -> R,
) -> R {
    let type_hash = context
        .stack()
        .peek()
        .expect("Could not pop value from stack to borrow!");
    if type_hash == TypeHash::of::<DynamicManaged>() {
        let value = context
            .stack()
            .pop::<DynamicManaged>()
            .expect("Could not pop owned value from stack!");
        owned_variant(context, value)
    } else if type_hash == TypeHash::of::<DynamicManagedRef>() {
        let value = context
            .stack()
            .pop::<DynamicManagedRef>()
            .expect("Could not pop ref value from stack!");
        ref_variant(context, value)
    } else if type_hash == TypeHash::of::<DynamicManagedRefMut>() {
        let value = context
            .stack()
            .pop::<DynamicManagedRefMut>()
            .expect("Could not pop ref mut value from stack!");
        ref_mut_variant(context, value)
    } else if type_hash == TypeHash::of::<DynamicManagedLazy>() {
        let value = context
            .stack()
            .pop::<DynamicManagedLazy>()
            .expect("Could not pop lazy value from stack!");
        lazy_variant(context, value)
    } else if type_hash == TypeHash::of::<DynamicManagedBox>() {
        let value = context
            .stack()
            .pop::<DynamicManagedBox>()
            .expect("Could not pop box value from stack!");
        box_variant(context, value)
    } else {
        panic!("Value on stack is not managed!");
    }
}

fn register_managed_variant<R>(
    context: &mut Context,
    index: usize,
    owned_variant: impl FnOnce(&mut DynamicManaged) -> R,
    ref_variant: impl FnOnce(&mut DynamicManagedRef) -> R,
    ref_mut_variant: impl FnOnce(&mut DynamicManagedRefMut) -> R,
    lazy_variant: impl FnOnce(&mut DynamicManagedLazy) -> R,
    box_variant: impl FnOnce(&mut DynamicManagedBox) -> R,
) -> R {
    let index = context.absolute_register_index(index);
    let mut register = context
        .registers()
        .access_register(index)
        .unwrap_or_else(|| panic!("Could not access non-existent register: {}", index));
    let type_hash = register.type_hash();
    if type_hash == TypeHash::of::<DynamicManaged>() {
        let value = register
            .write::<DynamicManaged>()
            .unwrap_or_else(|| panic!("Could write register: {} with no value!", index));
        owned_variant(value)
    } else if type_hash == TypeHash::of::<DynamicManagedRef>() {
        let value = register
            .write::<DynamicManagedRef>()
            .unwrap_or_else(|| panic!("Could write register: {} with no value!", index));
        ref_variant(value)
    } else if type_hash == TypeHash::of::<DynamicManagedRefMut>() {
        let value = register
            .write::<DynamicManagedRefMut>()
            .unwrap_or_else(|| panic!("Could write register: {} with no value!", index));
        ref_mut_variant(value)
    } else if type_hash == TypeHash::of::<DynamicManagedLazy>() {
        let value = register
            .write::<DynamicManagedLazy>()
            .unwrap_or_else(|| panic!("Could write register: {} with no value!", index));
        lazy_variant(value)
    } else if type_hash == TypeHash::of::<DynamicManagedBox>() {
        let value = register
            .write::<DynamicManagedBox>()
            .unwrap_or_else(|| panic!("Could write register: {} with no value!", index));
        box_variant(value)
    } else {
        panic!("Register: {} on stack is not managed!", index);
    }
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnkhaTypeQuery {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub module_name: Option<String>,
    #[serde(skip)]
    pub type_hash: Option<TypeHash>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub type_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visibility: Option<Visibility>,
}

impl AnkhaTypeQuery {
    pub fn compile(&self) -> TypeQuery<'_> {
        TypeQuery {
            name: self.name.as_deref().map(|v| v.into()),
            module_name: self.module_name.as_deref().map(|v| v.into()),
            type_hash: self.type_hash,
            type_name: self.type_name.as_deref().map(|v| v.into()),
            visibility: self.visibility,
            ..Default::default()
        }
    }
}

impl std::fmt::Display for AnkhaTypeQuery {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "type ")?;
        if let Some(visibility) = self.visibility {
            write!(f, " {:?}", visibility)?;
        }
        if let Some(module_name) = self.module_name.as_deref() {
            write!(f, " {} ::", module_name)?;
        }
        if let Some(name) = self.name.as_deref() {
            write!(f, " {}", name)?;
        }
        if let Some(type_name) = self.type_name.as_deref() {
            write!(f, " <{}>", type_name)?;
        }
        if let Some(type_hash) = self.type_hash {
            write!(f, " [{:?}]", type_hash)?;
        }
        Ok(())
    }
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnkhaFieldQuery {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub type_query: Option<AnkhaTypeQuery>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visibility: Option<Visibility>,
}

impl AnkhaFieldQuery {
    pub fn compile(&self) -> StructFieldQuery<'_> {
        StructFieldQuery {
            name: Some(self.name.as_str().into()),
            type_query: self.type_query.as_ref().map(|v| v.compile()),
            visibility: self.visibility,
            ..Default::default()
        }
    }
}

impl std::fmt::Display for AnkhaFieldQuery {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "field ")?;
        if let Some(visibility) = self.visibility {
            write!(f, " {:?}", visibility)?;
        }
        write!(f, " {}", self.name)?;
        if let Some(type_query) = self.type_query.as_ref() {
            write!(f, " ({})", type_query)?;
        }
        Ok(())
    }
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnkhaFunctionQuery {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub module_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub type_query: Option<AnkhaTypeQuery>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visibility: Option<Visibility>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub inputs: Vec<AnkhaFunctionQueryParam>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub outputs: Vec<AnkhaFunctionQueryParam>,
}

impl AnkhaFunctionQuery {
    pub fn compile(&self) -> FunctionQuery<'_> {
        FunctionQuery {
            name: self.name.as_deref().map(|v| v.into()),
            module_name: self.module_name.to_owned().map(|v| v.into()),
            type_query: self.type_query.as_ref().map(|v| v.compile()),
            visibility: self.visibility,
            inputs: self.inputs.iter().map(|v| v.compile()).collect(),
            outputs: self.outputs.iter().map(|v| v.compile()).collect(),
            ..Default::default()
        }
    }
}

impl std::fmt::Display for AnkhaFunctionQuery {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "param ")?;
        if let Some(visibility) = self.visibility {
            write!(f, " {:?}", visibility)?;
        }
        if let Some(module_name) = self.module_name.as_deref() {
            write!(f, " {} ::", module_name)?;
        }
        if let Some(type_query) = self.type_query.as_ref() {
            write!(f, " {} ::", type_query)?;
        }
        if let Some(name) = self.name.as_deref() {
            write!(f, " {}", name)?;
        }
        write!(f, " (")?;
        for (i, param) in self.inputs.iter().enumerate() {
            if i > 0 {
                write!(f, " ,")?;
            }
            write!(f, " {}", param)?;
        }
        write!(f, " ) -> (")?;
        for (i, param) in self.outputs.iter().enumerate() {
            if i > 0 {
                write!(f, " ,")?;
            }
            write!(f, " {}", param)?;
        }
        write!(f, " )")?;
        Ok(())
    }
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnkhaFunctionQueryParam {
    pub name: Option<String>,
    pub type_query: Option<AnkhaTypeQuery>,
}

impl AnkhaFunctionQueryParam {
    pub fn compile(&self) -> FunctionQueryParameter<'_> {
        FunctionQueryParameter {
            name: self.name.as_deref().map(|v| v.into()),
            type_query: self.type_query.as_ref().map(|v| v.compile()),
            ..Default::default()
        }
    }
}

impl std::fmt::Display for AnkhaFunctionQueryParam {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "param ")?;
        if let Some(name) = self.name.as_deref() {
            write!(f, " {}", name)?;
        }
        if let Some(type_query) = self.type_query.as_ref() {
            write!(f, " ({})", type_query)?;
        }
        Ok(())
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnkhaValueKind {
    #[default]
    Any,
    Owned,
    Ref,
    RefMut,
    Lazy,
    Box,
}

impl AnkhaValueKind {
    pub fn from_type_hash(type_hash: TypeHash) -> Self {
        if type_hash == TypeHash::of::<DynamicManaged>() {
            Self::Owned
        } else if type_hash == TypeHash::of::<DynamicManagedRef>() {
            Self::Ref
        } else if type_hash == TypeHash::of::<DynamicManagedRefMut>() {
            Self::RefMut
        } else if type_hash == TypeHash::of::<DynamicManagedLazy>() {
            Self::Lazy
        } else if type_hash == TypeHash::of::<DynamicManagedBox>() {
            Self::Box
        } else {
            Self::Any
        }
    }

    pub fn is_any(&self) -> bool {
        matches!(self, Self::Any)
    }

    pub fn type_hash(self) -> Option<TypeHash> {
        match self {
            Self::Any => None,
            Self::Owned => Some(TypeHash::of::<DynamicManaged>()),
            Self::Ref => Some(TypeHash::of::<DynamicManagedRef>()),
            Self::RefMut => Some(TypeHash::of::<DynamicManagedRefMut>()),
            Self::Lazy => Some(TypeHash::of::<DynamicManagedLazy>()),
            Self::Box => Some(TypeHash::of::<DynamicManagedBox>()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AnkhaLiteral {
    Unit,
    Bool(bool),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),
    Isize(isize),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    Usize(usize),
    F32(f32),
    F64(f64),
    Char(char),
    String(String),
}

impl AnkhaLiteral {
    fn evaluate(&self, context: &mut Context) {
        match self {
            Self::Unit => context.stack().push(
                DynamicManaged::new(()).expect("Could not create unit literal managed value"),
            ),
            Self::Bool(value) => {
                context
                    .stack()
                    .push(DynamicManaged::new(*value).unwrap_or_else(|_| {
                        panic!("Could not create {:?} literal managed value", value)
                    }))
            }
            Self::I8(value) => {
                context
                    .stack()
                    .push(DynamicManaged::new(*value).unwrap_or_else(|_| {
                        panic!("Could not create {:?} literal managed value", value)
                    }))
            }
            Self::I16(value) => {
                context
                    .stack()
                    .push(DynamicManaged::new(*value).unwrap_or_else(|_| {
                        panic!("Could not create {:?} literal managed value", value)
                    }))
            }
            Self::I32(value) => {
                context
                    .stack()
                    .push(DynamicManaged::new(*value).unwrap_or_else(|_| {
                        panic!("Could not create {:?} literal managed value", value)
                    }))
            }
            Self::I64(value) => {
                context
                    .stack()
                    .push(DynamicManaged::new(*value).unwrap_or_else(|_| {
                        panic!("Could not create {:?} literal managed value", value)
                    }))
            }
            Self::I128(value) => {
                context
                    .stack()
                    .push(DynamicManaged::new(*value).unwrap_or_else(|_| {
                        panic!("Could not create {:?} literal managed value", value)
                    }))
            }
            Self::Isize(value) => {
                context
                    .stack()
                    .push(DynamicManaged::new(*value).unwrap_or_else(|_| {
                        panic!("Could not create {:?} literal managed value", value)
                    }))
            }
            Self::U8(value) => {
                context
                    .stack()
                    .push(DynamicManaged::new(*value).unwrap_or_else(|_| {
                        panic!("Could not create {:?} literal managed value", value)
                    }))
            }
            Self::U16(value) => {
                context
                    .stack()
                    .push(DynamicManaged::new(*value).unwrap_or_else(|_| {
                        panic!("Could not create {:?} literal managed value", value)
                    }))
            }
            Self::U32(value) => {
                context
                    .stack()
                    .push(DynamicManaged::new(*value).unwrap_or_else(|_| {
                        panic!("Could not create {:?} literal managed value", value)
                    }))
            }
            Self::U64(value) => {
                context
                    .stack()
                    .push(DynamicManaged::new(*value).unwrap_or_else(|_| {
                        panic!("Could not create {:?} literal managed value", value)
                    }))
            }
            Self::U128(value) => {
                context
                    .stack()
                    .push(DynamicManaged::new(*value).unwrap_or_else(|_| {
                        panic!("Could not create {:?} literal managed value", value)
                    }))
            }
            Self::Usize(value) => {
                context
                    .stack()
                    .push(DynamicManaged::new(*value).unwrap_or_else(|_| {
                        panic!("Could not create {:?} literal managed value", value)
                    }))
            }
            Self::F32(value) => {
                context
                    .stack()
                    .push(DynamicManaged::new(*value).unwrap_or_else(|_| {
                        panic!("Could not create {:?} literal managed value", value)
                    }))
            }
            Self::F64(value) => {
                context
                    .stack()
                    .push(DynamicManaged::new(*value).unwrap_or_else(|_| {
                        panic!("Could not create {:?} literal managed value", value)
                    }))
            }
            Self::Char(value) => {
                context
                    .stack()
                    .push(DynamicManaged::new(*value).unwrap_or_else(|_| {
                        panic!("Could not create {:?} literal managed value", value)
                    }))
            }
            Self::String(value) => {
                context
                    .stack()
                    .push(DynamicManaged::new(value.to_owned()).unwrap_or_else(|_| {
                        panic!("Could not create {:?} literal managed value", value)
                    }))
            }
        };
    }
}

impl Hash for AnkhaLiteral {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Self::Unit => 0_u8.hash(state),
            Self::Bool(value) => value.hash(state),
            Self::I8(value) => value.hash(state),
            Self::I16(value) => value.hash(state),
            Self::I32(value) => value.hash(state),
            Self::I64(value) => value.hash(state),
            Self::I128(value) => value.hash(state),
            Self::Isize(value) => value.hash(state),
            Self::U8(value) => value.hash(state),
            Self::U16(value) => value.hash(state),
            Self::U32(value) => value.hash(state),
            Self::U64(value) => value.hash(state),
            Self::U128(value) => value.hash(state),
            Self::Usize(value) => value.hash(state),
            Self::F32(value) => value.to_be_bytes().hash(state),
            Self::F64(value) => value.to_be_bytes().hash(state),
            Self::Char(value) => value.hash(state),
            Self::String(value) => value.hash(state),
        }
    }
}

impl Eq for AnkhaLiteral {}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AnkhaExpression {
    // push owned value on stack.
    Literal(AnkhaLiteral),
    // drop top value from stack.
    StackDrop,
    // unwrap managed boolean into unmanaged boolean.
    StackUnwrapBoolean,
    // Borrows top stack value and pushes back borrowed and original.
    Borrow,
    // Borrows mutably top stack value and pushes back borrowed and original.
    BorrowMut,
    // Acquires lazy value from top stack value and pushes back lazy and original.
    Lazy,
    // Borrows specific managed field from ref value.
    BorrowField {
        name: String,
        #[serde(default, skip_serializing_if = "AnkhaValueKind::is_any")]
        kind: AnkhaValueKind,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        visibility: Option<Visibility>,
    },
    // Borrows mutably specific managed field from ref value.
    BorrowMutField {
        name: String,
        #[serde(default, skip_serializing_if = "AnkhaValueKind::is_any")]
        kind: AnkhaValueKind,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        visibility: Option<Visibility>,
    },
    // Borrows specific unmanaged field from ref value.
    BorrowUnmanagedField {
        query: AnkhaFieldQuery,
    },
    // Borrows mutably specific unmanaged field from ref value.
    BorrowMutUnmanagedField {
        query: AnkhaFieldQuery,
    },
    // Makes copy of top stack value content (if type is copy-able).
    CopyFrom,
    // Moves top stack value into object behind ref mut.
    MoveInto,
    // Swaps content in object behind ref mut with top stack value.
    SwapIn,
    // Consume and unpack that object managed fields.
    Destructure {
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        fields: Vec<String>,
    },
    // Consume top stack managed values into new value fields.
    Structure {
        type_query: AnkhaTypeQuery,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        fields: Vec<String>,
    },
    // Turns top stack owned value into boxed value.
    Box,
    // Turns top stack owned unmanaged value into managed value.
    Manage,
    // Turns top stack owned managed value into unmanaged value.
    Unmanage,
    // Copies top value (if type is copy-able).
    Copy,
    // Swaps two top values on stack.
    Swap,
    // Clones top stack box value and pushes back clone and original.
    DuplicateBox,
    // Checks if stack top managed value has certain type and panics if doesn't.
    EnsureStackType {
        type_query: AnkhaTypeQuery,
    },
    // Checks if N register managed value has certain type and panics if doesn't.
    EnsureRegisterType {
        type_query: AnkhaTypeQuery,
        index: usize,
    },
    // Checks if stack top managed value has certain kind and panics if doesn't.
    EnsureStackKind {
        kind: AnkhaValueKind,
    },
    // Checks if N register managed value has certain kind and panics if doesn't.
    EnsureRegisterKind {
        kind: AnkhaValueKind,
        index: usize,
    },
    // Checks type of top stack managed value and overrides it to function query.
    CallMethod {
        function_query: AnkhaFunctionQuery,
    },
    // Calls function by handle from stack top value.
    CallIndirect,
    // Finds type and puts it on stack.
    GetType {
        query: AnkhaTypeQuery,
    },
    // Finds function and puts it on stack.
    GetFunction {
        query: AnkhaFunctionQuery,
    },
}

impl AnkhaExpression {
    fn stack_unwrap_boolean(context: &mut Context) {
        stack_managed_variant(
            context,
            |context, value| {
                context.stack().push(
                    *value
                        .read::<bool>()
                        .expect("Owned value got from stack is not a boleean!"),
                );
            },
            |context, value| {
                context.stack().push(
                    *value
                        .read::<bool>()
                        .expect("Ref value got from stack is not a boleean!"),
                );
            },
            |context, value| {
                context.stack().push(
                    *value
                        .read::<bool>()
                        .expect("Ref mut value got from stack is not a boleean!"),
                );
            },
            |context, value| {
                context.stack().push(
                    *value
                        .read::<bool>()
                        .expect("Lazy value got from stack is not a boleean!"),
                );
            },
            |context, value| {
                context.stack().push(
                    *value
                        .read::<bool>()
                        .expect("Box value got from stack is not a boleean!"),
                );
            },
        );
    }

    fn borrow(context: &mut Context) {
        stack_managed_variant(
            context,
            |context, value| {
                let value_ref = value.borrow().expect("Owned value cannot be borrowed!");
                context.stack().push(value_ref);
                context.stack().push(value);
            },
            |context, value| {
                let value_ref = value.borrow().expect("Ref value cannot be borrowed!");
                context.stack().push(value_ref);
                context.stack().push(value);
            },
            |context, value| {
                let value_ref = value.borrow().expect("Ref mut value cannot be borrowed!");
                context.stack().push(value_ref);
                context.stack().push(value);
            },
            |context, value| {
                let value_ref = value.borrow().expect("Lazy value cannot be borrowed!");
                context.stack().push(value_ref);
                context.stack().push(value);
            },
            |context, value| {
                let value_ref = value.borrow().expect("Box value cannot be borrowed!");
                context.stack().push(value_ref);
                context.stack().push(value);
            },
        );
    }

    fn borrow_mut(context: &mut Context) {
        stack_managed_variant(
            context,
            |context, mut value| {
                let value_ref = value
                    .borrow_mut()
                    .expect("Owned value cannot be borrowed mutably!");
                context.stack().push(value_ref);
                context.stack().push(value);
            },
            |_, _| {
                panic!("Ref value cannot be borrowed mutably!");
            },
            |context, mut value| {
                let value_ref = value
                    .borrow_mut()
                    .expect("Ref mut value cannot be borrowed mutably!");
                context.stack().push(value_ref);
                context.stack().push(value);
            },
            |context, mut value| {
                let value_ref = value
                    .borrow_mut()
                    .expect("Lazy value cannot be borrowed mutably!");
                context.stack().push(value_ref);
                context.stack().push(value);
            },
            |context, mut value| {
                let value_ref = value
                    .borrow_mut()
                    .expect("Box value cannot be borrowed mutably!");
                context.stack().push(value_ref);
                context.stack().push(value);
            },
        );
    }

    fn lazy(context: &mut Context) {
        stack_managed_variant(
            context,
            |context, value| {
                let value_ref = value.lazy();
                context.stack().push(value_ref);
                context.stack().push(value);
            },
            |_, _| {
                panic!("Ref value cannot be borrowed lazily!");
            },
            |_, _| {
                panic!("Ref mut value cannot be borrowed lazily!");
            },
            |_, _| {
                panic!("Lazy value cannot be borrowed lazily!");
            },
            |context, value| {
                let value_ref = value.lazy().expect("Box value cannot be borrowed lazily!");
                context.stack().push(value_ref);
                context.stack().push(value);
            },
        );
    }

    fn borrow_managed_field(
        context: &mut Context,
        registry: &Registry,
        name: &str,
        kind: AnkhaValueKind,
        visibility: Option<Visibility>,
    ) {
        stack_managed_variant(
            context,
            |_, _| {
                panic!("Cannot borrow field from owned stack value!");
            },
            |context, value| {
                let type_ = registry
                    .find_type(TypeQuery {
                        type_hash: Some(*value.type_hash()),
                        ..Default::default()
                    })
                    .expect("Could not find type of ref stack value!");
                let field = if let Some(type_) = type_.as_struct() {
                    type_
                        .find_field(StructFieldQuery {
                            name: Some(name.into()),
                            type_query: kind.type_hash().map(|type_hash| TypeQuery {
                                type_hash: Some(type_hash),
                                ..Default::default()
                            }),
                            visibility,
                            ..Default::default()
                        })
                        .unwrap_or_else(|| {
                            panic!(
                                "Could not find `{}` field in `{}` struct!",
                                name, type_.name
                            )
                        })
                } else {
                    panic!("`{}` is not a struct!", type_.name());
                };
                let pointer = unsafe {
                    value
                        .as_ptr_raw()
                        .expect("Could not get ref value content pointer!")
                        .add(field.address_offset())
                };
                let field_type_hash = field.type_handle().type_hash();
                let value_ref = if field_type_hash == TypeHash::of::<DynamicManaged>() {
                    unsafe {
                        (*pointer.cast::<DynamicManaged>())
                            .borrow()
                            .unwrap_or_else(|| {
                                panic!(
                                    "Could not borrow `{}` field in `{}` struct!",
                                    name,
                                    type_.name()
                                )
                            })
                    }
                } else if field_type_hash == TypeHash::of::<DynamicManagedRef>() {
                    unsafe {
                        (*pointer.cast::<DynamicManagedRef>())
                            .borrow()
                            .unwrap_or_else(|| {
                                panic!(
                                    "Could not borrow `{}` field in `{}` struct!",
                                    name,
                                    type_.name()
                                )
                            })
                    }
                } else if field_type_hash == TypeHash::of::<DynamicManagedRefMut>() {
                    unsafe {
                        (*pointer.cast::<DynamicManagedRefMut>())
                            .borrow()
                            .unwrap_or_else(|| {
                                panic!(
                                    "Could not borrow `{}` field in `{}` struct!",
                                    name,
                                    type_.name()
                                )
                            })
                    }
                } else if field_type_hash == TypeHash::of::<DynamicManagedLazy>() {
                    unsafe {
                        (*pointer.cast::<DynamicManagedLazy>())
                            .borrow()
                            .unwrap_or_else(|| {
                                panic!(
                                    "Could not borrow `{}` field in `{}` struct!",
                                    name,
                                    type_.name()
                                )
                            })
                    }
                } else if field_type_hash == TypeHash::of::<DynamicManagedBox>() {
                    unsafe {
                        (*pointer.cast::<DynamicManagedBox>())
                            .borrow()
                            .unwrap_or_else(|| {
                                panic!(
                                    "Could not borrow `{}` field in `{}` struct!",
                                    name,
                                    type_.name()
                                )
                            })
                    }
                } else {
                    panic!(
                        "Value `{}` field in `{}` struct is not managed!",
                        name,
                        type_.name()
                    );
                };
                context.stack().push(value_ref);
            },
            |context, value| {
                let type_ = registry
                    .find_type(TypeQuery {
                        type_hash: Some(*value.type_hash()),
                        ..Default::default()
                    })
                    .expect("Could not find type of ref mut stack value!");
                let field = if let Some(type_) = type_.as_struct() {
                    type_
                        .find_field(StructFieldQuery {
                            name: Some(name.into()),
                            type_query: kind.type_hash().map(|type_hash| TypeQuery {
                                type_hash: Some(type_hash),
                                ..Default::default()
                            }),
                            visibility,
                            ..Default::default()
                        })
                        .unwrap_or_else(|| {
                            panic!(
                                "Could not find `{}` field in `{}` struct!",
                                name, type_.name
                            )
                        })
                } else {
                    panic!("`{}` is not a struct!", type_.name());
                };
                let pointer = unsafe {
                    value
                        .as_ptr_raw()
                        .expect("Could not get ref mut value content pointer!")
                        .add(field.address_offset())
                };
                let field_type_hash = field.type_handle().type_hash();
                let value_ref = if field_type_hash == TypeHash::of::<DynamicManaged>() {
                    unsafe {
                        (*pointer.cast::<DynamicManaged>())
                            .borrow()
                            .unwrap_or_else(|| {
                                panic!(
                                    "Could not borrow `{}` field in `{}` struct!",
                                    name,
                                    type_.name()
                                )
                            })
                    }
                } else if field_type_hash == TypeHash::of::<DynamicManagedRefMut>() {
                    unsafe {
                        (*pointer.cast::<DynamicManagedRefMut>())
                            .borrow()
                            .unwrap_or_else(|| {
                                panic!(
                                    "Could not borrow `{}` field in `{}` struct!",
                                    name,
                                    type_.name()
                                )
                            })
                    }
                } else if field_type_hash == TypeHash::of::<DynamicManagedLazy>() {
                    unsafe {
                        (*pointer.cast::<DynamicManagedLazy>())
                            .borrow()
                            .unwrap_or_else(|| {
                                panic!(
                                    "Could not borrow `{}` field in `{}` struct!",
                                    name,
                                    type_.name()
                                )
                            })
                    }
                } else if field_type_hash == TypeHash::of::<DynamicManagedBox>() {
                    unsafe {
                        (*pointer.cast::<DynamicManagedBox>())
                            .borrow()
                            .unwrap_or_else(|| {
                                panic!(
                                    "Could not borrow `{}` field in `{}` struct!",
                                    name,
                                    type_.name()
                                )
                            })
                    }
                } else {
                    panic!(
                        "Value `{}` field in `{}` struct is not managed!",
                        name,
                        type_.name()
                    );
                };
                context.stack().push(value_ref);
            },
            |context, value| {
                let type_ = registry
                    .find_type(TypeQuery {
                        type_hash: Some(*value.type_hash()),
                        ..Default::default()
                    })
                    .expect("Could not find type of lazy stack value!");
                let field = if let Some(type_) = type_.as_struct() {
                    type_
                        .find_field(StructFieldQuery {
                            name: Some(name.into()),
                            type_query: kind.type_hash().map(|type_hash| TypeQuery {
                                type_hash: Some(type_hash),
                                ..Default::default()
                            }),
                            visibility,
                            ..Default::default()
                        })
                        .unwrap_or_else(|| {
                            panic!(
                                "Could not find `{}` field in `{}` struct!",
                                name, type_.name
                            )
                        })
                } else {
                    panic!("`{}` is not a struct!", type_.name());
                };
                let pointer = unsafe {
                    value
                        .as_ptr_raw()
                        .expect("Could not get lazy value content pointer!")
                        .add(field.address_offset())
                };
                let field_type_hash = field.type_handle().type_hash();
                let value_ref = if field_type_hash == TypeHash::of::<DynamicManaged>() {
                    unsafe {
                        (*pointer.cast::<DynamicManaged>())
                            .borrow()
                            .unwrap_or_else(|| {
                                panic!(
                                    "Could not borrow `{}` field in `{}` struct!",
                                    name,
                                    type_.name()
                                )
                            })
                    }
                } else if field_type_hash == TypeHash::of::<DynamicManagedRefMut>() {
                    unsafe {
                        (*pointer.cast::<DynamicManagedRefMut>())
                            .borrow()
                            .unwrap_or_else(|| {
                                panic!(
                                    "Could not borrow `{}` field in `{}` struct!",
                                    name,
                                    type_.name()
                                )
                            })
                    }
                } else if field_type_hash == TypeHash::of::<DynamicManagedLazy>() {
                    unsafe {
                        (*pointer.cast::<DynamicManagedLazy>())
                            .borrow()
                            .unwrap_or_else(|| {
                                panic!(
                                    "Could not borrow `{}` field in `{}` struct!",
                                    name,
                                    type_.name()
                                )
                            })
                    }
                } else if field_type_hash == TypeHash::of::<DynamicManagedBox>() {
                    unsafe {
                        (*pointer.cast::<DynamicManagedBox>())
                            .borrow()
                            .unwrap_or_else(|| {
                                panic!(
                                    "Could not borrow `{}` field in `{}` struct!",
                                    name,
                                    type_.name()
                                )
                            })
                    }
                } else {
                    panic!(
                        "Value `{}` field in `{}` struct is not managed!",
                        name,
                        type_.name()
                    );
                };
                context.stack().push(value_ref);
            },
            |context, value| {
                let type_ = registry
                    .find_type(TypeQuery {
                        type_hash: value.type_hash(),
                        ..Default::default()
                    })
                    .expect("Could not find type of box stack value!");
                let field = if let Some(type_) = type_.as_struct() {
                    type_
                        .find_field(StructFieldQuery {
                            name: Some(name.into()),
                            type_query: kind.type_hash().map(|type_hash| TypeQuery {
                                type_hash: Some(type_hash),
                                ..Default::default()
                            }),
                            visibility,
                            ..Default::default()
                        })
                        .unwrap_or_else(|| {
                            panic!(
                                "Could not find `{}` field in `{}` struct!",
                                name, type_.name
                            )
                        })
                } else {
                    panic!("`{}` is not a struct!", type_.name());
                };
                let pointer = unsafe {
                    value
                        .as_ptr_raw()
                        .expect("Could not get box value content pointer!")
                        .add(field.address_offset())
                };
                let field_type_hash = field.type_handle().type_hash();
                let value_ref = if field_type_hash == TypeHash::of::<DynamicManaged>() {
                    unsafe {
                        (*pointer.cast::<DynamicManaged>())
                            .borrow()
                            .unwrap_or_else(|| {
                                panic!(
                                    "Could not borrow `{}` field in `{}` struct!",
                                    name,
                                    type_.name()
                                )
                            })
                    }
                } else if field_type_hash == TypeHash::of::<DynamicManagedRefMut>() {
                    unsafe {
                        (*pointer.cast::<DynamicManagedRefMut>())
                            .borrow()
                            .unwrap_or_else(|| {
                                panic!(
                                    "Could not borrow `{}` field in `{}` struct!",
                                    name,
                                    type_.name()
                                )
                            })
                    }
                } else if field_type_hash == TypeHash::of::<DynamicManagedLazy>() {
                    unsafe {
                        (*pointer.cast::<DynamicManagedLazy>())
                            .borrow()
                            .unwrap_or_else(|| {
                                panic!(
                                    "Could not borrow `{}` field in `{}` struct!",
                                    name,
                                    type_.name()
                                )
                            })
                    }
                } else if field_type_hash == TypeHash::of::<DynamicManagedBox>() {
                    unsafe {
                        (*pointer.cast::<DynamicManagedBox>())
                            .borrow()
                            .unwrap_or_else(|| {
                                panic!(
                                    "Could not borrow `{}` field in `{}` struct!",
                                    name,
                                    type_.name()
                                )
                            })
                    }
                } else {
                    panic!(
                        "Value `{}` field in `{}` struct is not managed!",
                        name,
                        type_.name()
                    );
                };
                context.stack().push(value_ref);
            },
        );
    }

    fn borrow_mut_managed_field(
        context: &mut Context,
        registry: &Registry,
        name: &str,
        kind: AnkhaValueKind,
        visibility: Option<Visibility>,
    ) {
        stack_managed_variant(
            context,
            |_, _| {
                panic!("Cannot borrow field mutably from owned stack value!");
            },
            |_, _| {
                panic!("Cannot borrow field mutably from ref stack value!");
            },
            |context, mut value| {
                let type_ = registry
                    .find_type(TypeQuery {
                        type_hash: Some(*value.type_hash()),
                        ..Default::default()
                    })
                    .expect("Could not find type of ref mut stack value!");
                let field = if let Some(type_) = type_.as_struct() {
                    type_
                        .find_field(StructFieldQuery {
                            name: Some(name.into()),
                            type_query: kind.type_hash().map(|type_hash| TypeQuery {
                                type_hash: Some(type_hash),
                                ..Default::default()
                            }),
                            visibility,
                            ..Default::default()
                        })
                        .unwrap_or_else(|| {
                            panic!(
                                "Could not find `{}` field in `{}` struct!",
                                name, type_.name
                            )
                        })
                } else {
                    panic!("`{}` is not a struct!", type_.name());
                };
                let pointer = unsafe {
                    value
                        .as_mut_ptr_raw()
                        .expect("Could not get ref mut value content pointer!")
                        .add(field.address_offset())
                };
                let field_type_hash = field.type_handle().type_hash();
                let value_ref = if field_type_hash == TypeHash::of::<DynamicManaged>() {
                    unsafe {
                        (*pointer.cast::<DynamicManaged>())
                            .borrow_mut()
                            .unwrap_or_else(|| {
                                panic!(
                                    "Could not borrow mutably `{}` field in `{}` struct!",
                                    name,
                                    type_.name()
                                )
                            })
                    }
                } else if field_type_hash == TypeHash::of::<DynamicManagedRefMut>() {
                    unsafe {
                        (*pointer.cast::<DynamicManagedRefMut>())
                            .borrow_mut()
                            .unwrap_or_else(|| {
                                panic!(
                                    "Could not borrow mutably `{}` field in `{}` struct!",
                                    name,
                                    type_.name()
                                )
                            })
                    }
                } else if field_type_hash == TypeHash::of::<DynamicManagedLazy>() {
                    unsafe {
                        (*pointer.cast::<DynamicManagedLazy>())
                            .borrow_mut()
                            .unwrap_or_else(|| {
                                panic!(
                                    "Could not borrow mutably `{}` field in `{}` struct!",
                                    name,
                                    type_.name()
                                )
                            })
                    }
                } else if field_type_hash == TypeHash::of::<DynamicManagedBox>() {
                    unsafe {
                        (*pointer.cast::<DynamicManagedBox>())
                            .borrow_mut()
                            .unwrap_or_else(|| {
                                panic!(
                                    "Could not borrow mutably `{}` field in `{}` struct!",
                                    name,
                                    type_.name()
                                )
                            })
                    }
                } else {
                    panic!(
                        "Value `{}` field in `{}` struct is not managed!",
                        name,
                        type_.name()
                    );
                };
                context.stack().push(value_ref);
            },
            |context, mut value| {
                let type_ = registry
                    .find_type(TypeQuery {
                        type_hash: Some(*value.type_hash()),
                        ..Default::default()
                    })
                    .expect("Could not find type of lazy stack value!");
                let field = if let Some(type_) = type_.as_struct() {
                    type_
                        .find_field(StructFieldQuery {
                            name: Some(name.into()),
                            type_query: kind.type_hash().map(|type_hash| TypeQuery {
                                type_hash: Some(type_hash),
                                ..Default::default()
                            }),
                            visibility,
                            ..Default::default()
                        })
                        .unwrap_or_else(|| {
                            panic!(
                                "Could not find `{}` field in `{}` struct!",
                                name, type_.name
                            )
                        })
                } else {
                    panic!("`{}` is not a struct!", type_.name());
                };
                let pointer = unsafe {
                    value
                        .as_mut_ptr_raw()
                        .expect("Could not get lazy value content pointer!")
                        .add(field.address_offset())
                };
                let field_type_hash = field.type_handle().type_hash();
                let value_ref = if field_type_hash == TypeHash::of::<DynamicManaged>() {
                    unsafe {
                        (*pointer.cast::<DynamicManaged>())
                            .borrow_mut()
                            .unwrap_or_else(|| {
                                panic!(
                                    "Could not borrow mutably `{}` field in `{}` struct!",
                                    name,
                                    type_.name()
                                )
                            })
                    }
                } else if field_type_hash == TypeHash::of::<DynamicManagedRefMut>() {
                    unsafe {
                        (*pointer.cast::<DynamicManagedRefMut>())
                            .borrow_mut()
                            .unwrap_or_else(|| {
                                panic!(
                                    "Could not borrow mutably `{}` field in `{}` struct!",
                                    name,
                                    type_.name()
                                )
                            })
                    }
                } else if field_type_hash == TypeHash::of::<DynamicManagedLazy>() {
                    unsafe {
                        (*pointer.cast::<DynamicManagedLazy>())
                            .borrow_mut()
                            .unwrap_or_else(|| {
                                panic!(
                                    "Could not borrow mutably `{}` field in `{}` struct!",
                                    name,
                                    type_.name()
                                )
                            })
                    }
                } else if field_type_hash == TypeHash::of::<DynamicManagedBox>() {
                    unsafe {
                        (*pointer.cast::<DynamicManagedBox>())
                            .borrow_mut()
                            .unwrap_or_else(|| {
                                panic!(
                                    "Could not borrow mutably `{}` field in `{}` struct!",
                                    name,
                                    type_.name()
                                )
                            })
                    }
                } else {
                    panic!(
                        "Value `{}` field in `{}` struct is not managed!",
                        name,
                        type_.name()
                    );
                };
                context.stack().push(value_ref);
            },
            |context, mut value| {
                let type_ = registry
                    .find_type(TypeQuery {
                        type_hash: value.type_hash(),
                        ..Default::default()
                    })
                    .expect("Could not find type of box stack value!");
                let field = if let Some(type_) = type_.as_struct() {
                    type_
                        .find_field(StructFieldQuery {
                            name: Some(name.into()),
                            type_query: kind.type_hash().map(|type_hash| TypeQuery {
                                type_hash: Some(type_hash),
                                ..Default::default()
                            }),
                            visibility,
                            ..Default::default()
                        })
                        .unwrap_or_else(|| {
                            panic!(
                                "Could not find `{}` field in `{}` struct!",
                                name, type_.name
                            )
                        })
                } else {
                    panic!("`{}` is not a struct!", type_.name());
                };
                let pointer = unsafe {
                    value
                        .as_mut_ptr_raw()
                        .expect("Could not get box value content pointer!")
                        .add(field.address_offset())
                };
                let field_type_hash = field.type_handle().type_hash();
                let value_ref = if field_type_hash == TypeHash::of::<DynamicManaged>() {
                    unsafe {
                        (*pointer.cast::<DynamicManaged>())
                            .borrow_mut()
                            .unwrap_or_else(|| {
                                panic!(
                                    "Could not borrow mutably `{}` field in `{}` struct!",
                                    name,
                                    type_.name()
                                )
                            })
                    }
                } else if field_type_hash == TypeHash::of::<DynamicManagedRefMut>() {
                    unsafe {
                        (*pointer.cast::<DynamicManagedRefMut>())
                            .borrow_mut()
                            .unwrap_or_else(|| {
                                panic!(
                                    "Could not borrow mutably `{}` field in `{}` struct!",
                                    name,
                                    type_.name()
                                )
                            })
                    }
                } else if field_type_hash == TypeHash::of::<DynamicManagedLazy>() {
                    unsafe {
                        (*pointer.cast::<DynamicManagedLazy>())
                            .borrow_mut()
                            .unwrap_or_else(|| {
                                panic!(
                                    "Could not borrow mutably `{}` field in `{}` struct!",
                                    name,
                                    type_.name()
                                )
                            })
                    }
                } else if field_type_hash == TypeHash::of::<DynamicManagedBox>() {
                    unsafe {
                        (*pointer.cast::<DynamicManagedBox>())
                            .borrow_mut()
                            .unwrap_or_else(|| {
                                panic!(
                                    "Could not borrow mutably `{}` field in `{}` struct!",
                                    name,
                                    type_.name()
                                )
                            })
                    }
                } else {
                    panic!(
                        "Value `{}` field in `{}` struct is not managed!",
                        name,
                        type_.name()
                    );
                };
                context.stack().push(value_ref);
            },
        );
    }

    fn borrow_unmanaged_field(context: &mut Context, registry: &Registry, query: &AnkhaFieldQuery) {
        stack_managed_variant(
            context,
            |_, _| {
                panic!("Cannot borrow field from owned stack value!");
            },
            |context, value| {
                let type_ = registry
                    .find_type(TypeQuery {
                        type_hash: Some(*value.type_hash()),
                        ..Default::default()
                    })
                    .expect("Could not find type of ref stack value!");
                let field = if let Some(type_) = type_.as_struct() {
                    type_.find_field(query.compile()).unwrap_or_else(|| {
                        panic!(
                            "Could not find `{}` field in `{}` struct!",
                            query, type_.name
                        )
                    })
                } else {
                    panic!("`{}` is not a struct!", type_.name());
                };
                let pointer = unsafe {
                    value
                        .as_ptr_raw()
                        .expect("Could not get ref value content pointer!")
                        .add(field.address_offset())
                };
                context.stack().push(unsafe {
                    DynamicManagedRef::new_raw(
                        field.type_handle().type_hash(),
                        value.lifetime().borrow().unwrap_or_else(|| {
                            panic!(
                                "Could not borrow unmanaged `{}` field in `{}` struct!",
                                query,
                                type_.name()
                            );
                        }),
                        pointer,
                    )
                    .unwrap_or_else(|| {
                        panic!(
                            "Could not create value ref to unmanaged `{}` field in `{}` struct!",
                            query,
                            type_.name()
                        )
                    })
                });
            },
            |context, value| {
                let type_ = registry
                    .find_type(TypeQuery {
                        type_hash: Some(*value.type_hash()),
                        ..Default::default()
                    })
                    .expect("Could not find type of ref mut stack value!");
                let field = if let Some(type_) = type_.as_struct() {
                    type_.find_field(query.compile()).unwrap_or_else(|| {
                        panic!(
                            "Could not find `{}` field in `{}` struct!",
                            query, type_.name
                        )
                    })
                } else {
                    panic!("`{}` is not a struct!", type_.name());
                };
                let pointer = unsafe {
                    value
                        .as_ptr_raw()
                        .expect("Could not get ref value content pointer!")
                        .add(field.address_offset())
                };
                context.stack().push(unsafe {
                    DynamicManagedRef::new_raw(
                        field.type_handle().type_hash(),
                        value.lifetime().borrow().unwrap_or_else(|| {
                            panic!(
                                "Could not borrow unmanaged `{}` field in `{}` struct!",
                                query,
                                type_.name()
                            );
                        }),
                        pointer,
                    )
                    .unwrap_or_else(|| {
                        panic!(
                            "Could not create value ref to unmanaged `{}` field in `{}` struct!",
                            query,
                            type_.name()
                        )
                    })
                });
            },
            |context, value| {
                let type_ = registry
                    .find_type(TypeQuery {
                        type_hash: Some(*value.type_hash()),
                        ..Default::default()
                    })
                    .expect("Could not find type of lazy stack value!");
                let field = if let Some(type_) = type_.as_struct() {
                    type_.find_field(query.compile()).unwrap_or_else(|| {
                        panic!(
                            "Could not find `{}` field in `{}` struct!",
                            query, type_.name
                        )
                    })
                } else {
                    panic!("`{}` is not a struct!", type_.name());
                };
                let pointer = unsafe {
                    value
                        .as_ptr_raw()
                        .expect("Could not get box value content pointer!")
                        .add(field.address_offset())
                };
                context.stack().push(unsafe {
                    DynamicManagedRef::new_raw(
                        field.type_handle().type_hash(),
                        value.lifetime().borrow().unwrap_or_else(|| {
                            panic!(
                                "Could not borrow unmanaged `{}` field in `{}` struct!",
                                query,
                                type_.name()
                            );
                        }),
                        pointer,
                    )
                    .unwrap_or_else(|| {
                        panic!(
                            "Could not create value ref to unmanaged `{}` field in `{}` struct!",
                            query,
                            type_.name()
                        )
                    })
                });
            },
            |context, value| {
                let type_ = registry
                    .find_type(TypeQuery {
                        type_hash: value.type_hash(),
                        ..Default::default()
                    })
                    .expect("Could not find type of box stack value!");
                let field = if let Some(type_) = type_.as_struct() {
                    type_.find_field(query.compile()).unwrap_or_else(|| {
                        panic!(
                            "Could not find `{}` field in `{}` struct!",
                            query, type_.name
                        )
                    })
                } else {
                    panic!("`{}` is not a struct!", type_.name());
                };
                let pointer = unsafe {
                    value
                        .as_ptr_raw()
                        .expect("Could not get box value content pointer!")
                        .add(field.address_offset())
                };
                context.stack().push(unsafe {
                    DynamicManagedRef::new_raw(
                        field.type_handle().type_hash(),
                        value.lifetime_borrow().unwrap_or_else(|| {
                            panic!(
                                "Could not borrow unmanaged `{}` field in `{}` struct!",
                                query,
                                type_.name()
                            );
                        }),
                        pointer,
                    )
                    .unwrap_or_else(|| {
                        panic!(
                            "Could not create value ref to unmanaged `{}` field in `{}` struct!",
                            query,
                            type_.name()
                        )
                    })
                });
            },
        );
    }

    fn borrow_mut_unmanaged_field(
        context: &mut Context,
        registry: &Registry,
        query: &AnkhaFieldQuery,
    ) {
        stack_managed_variant(
            context,
            |_, _| {
                panic!("Cannot borrow mutably field from owned stack value!");
            },
            |_, _| {
                panic!("Cannot borrow mutably field from ref stack value!");
            },
            |context, mut value| {
                let type_ = registry
                    .find_type(TypeQuery {
                        type_hash: Some(*value.type_hash()),
                        ..Default::default()
                    })
                    .expect("Could not find type of ref mut stack value!");
                let field = if let Some(type_) = type_.as_struct() {
                    type_.find_field(query.compile()).unwrap_or_else(|| {
                        panic!(
                            "Could not find `{}` field in `{}` struct!",
                            query, type_.name
                        )
                    })
                } else {
                    panic!("`{}` is not a struct!", type_.name());
                };
                let pointer = unsafe {
                    value
                        .as_mut_ptr_raw()
                        .expect("Could not get ref mut value content pointer!")
                        .add(field.address_offset())
                };
                context.stack().push(unsafe {
                    DynamicManagedRefMut::new_raw(
                        field.type_handle().type_hash(),
                        value.lifetime().borrow_mut().unwrap_or_else(|| {
                            panic!(
                                "Could not borrow mutably unmanaged `{}` field in `{}` struct!",
                                query,
                                type_.name()
                            );
                        }),
                        pointer,
                    )
                    .unwrap_or_else(|| {
                        panic!(
                            "Could not create value ref mut to unmanaged `{}` field in `{}` struct!",
                            query,
                            type_.name()
                        )
                    })
                });
            },
            |context, mut value| {
                let type_ = registry
                    .find_type(TypeQuery {
                        type_hash: Some(*value.type_hash()),
                        ..Default::default()
                    })
                    .expect("Could not find type of lazy stack value!");
                let field = if let Some(type_) = type_.as_struct() {
                    type_.find_field(query.compile()).unwrap_or_else(|| {
                        panic!(
                            "Could not find `{}` field in `{}` struct!",
                            query, type_.name
                        )
                    })
                } else {
                    panic!("`{}` is not a struct!", type_.name());
                };
                let pointer = unsafe {
                    value
                        .as_mut_ptr_raw()
                        .expect("Could not get lazy value content pointer!")
                        .add(field.address_offset())
                };
                context.stack().push(unsafe {
                    DynamicManagedRefMut::new_raw(
                        field.type_handle().type_hash(),
                        value.lifetime().borrow_mut().unwrap_or_else(|| {
                            panic!(
                                "Could not borrow mutably unmanaged `{}` field in `{}` struct!",
                                query,
                                type_.name()
                            );
                        }),
                        pointer,
                    )
                    .unwrap_or_else(|| {
                        panic!(
                            "Could not create value ref mut to unmanaged `{}` field in `{}` struct!",
                            query,
                            type_.name()
                        )
                    })
                });
            },
            |context, mut value| {
                let type_ = registry
                    .find_type(TypeQuery {
                        type_hash: value.type_hash(),
                        ..Default::default()
                    })
                    .expect("Could not find type of box stack value!");
                let field = if let Some(type_) = type_.as_struct() {
                    type_.find_field(query.compile()).unwrap_or_else(|| {
                        panic!(
                            "Could not find `{}` field in `{}` struct!",
                            query, type_.name
                        )
                    })
                } else {
                    panic!("`{}` is not a struct!", type_.name());
                };
                let pointer = unsafe {
                    value
                        .as_mut_ptr_raw()
                        .expect("Could not get box value content pointer!")
                        .add(field.address_offset())
                };
                context.stack().push(unsafe {
                    DynamicManagedRefMut::new_raw(
                        field.type_handle().type_hash(),
                        value.lifetime_borrow_mut().unwrap_or_else(|| {
                            panic!(
                                "Could not borrow mutably unmanaged `{}` field in `{}` struct!",
                                query,
                                type_.name()
                            );
                        }),
                        pointer,
                    )
                    .unwrap_or_else(|| {
                        panic!(
                            "Could not create value ref mut to unmanaged `{}` field in `{}` struct!",
                            query,
                            type_.name()
                        )
                    })
                });
            },
        );
    }

    fn copy_from(context: &mut Context, registry: &Registry) {
        stack_managed_variant(
            context,
            |_, _| {
                panic!("Cannot copy owned value!");
            },
            |context, value| {
                let type_ = registry
                    .find_type(TypeQuery {
                        type_hash: Some(*value.type_hash()),
                        ..Default::default()
                    })
                    .expect("Could not find type of ref stack value!");
                if type_.is_copy() {
                    unsafe {
                        let memory = Object::new_uninitialized(type_.clone())
                            .unwrap_or_else(|| {
                                panic!("Could not create object of `{}` type!", type_.name())
                            })
                            .into_inner()
                            .1;
                        memory.copy_from(
                            value
                                .as_ptr_raw()
                                .expect("Could not get ref value content pointer!"),
                            type_.layout().size(),
                        );
                        let result = DynamicManaged::new_raw(
                            *value.type_hash(),
                            Lifetime::default(),
                            memory,
                            *type_.layout(),
                            type_.finalizer(),
                        )
                        .unwrap_or_else(|| {
                            panic!(
                                "Could not create managed object of `{}` type!",
                                type_.name()
                            )
                        });
                        context.stack().push(result);
                    }
                } else {
                    panic!("Type `{}` is not made for copies!", type_.name());
                }
            },
            |context, value| {
                let type_ = registry
                    .find_type(TypeQuery {
                        type_hash: Some(*value.type_hash()),
                        ..Default::default()
                    })
                    .expect("Could not find type of ref mut stack value!");
                if type_.is_copy() {
                    unsafe {
                        let memory = Object::new_uninitialized(type_.clone())
                            .unwrap_or_else(|| {
                                panic!("Could not create object of `{}` type!", type_.name())
                            })
                            .into_inner()
                            .1;
                        memory.copy_from(
                            value
                                .as_ptr_raw()
                                .expect("Could not get ref mut value content pointer!"),
                            type_.layout().size(),
                        );
                        let result = DynamicManaged::new_raw(
                            *value.type_hash(),
                            Lifetime::default(),
                            memory,
                            *type_.layout(),
                            type_.finalizer(),
                        )
                        .unwrap_or_else(|| {
                            panic!(
                                "Could not create managed object of `{}` type!",
                                type_.name()
                            )
                        });
                        context.stack().push(result);
                    }
                } else {
                    panic!("Type `{}` is not made for copies!", type_.name());
                }
            },
            |context, value| {
                let type_ = registry
                    .find_type(TypeQuery {
                        type_hash: Some(*value.type_hash()),
                        ..Default::default()
                    })
                    .expect("Could not find type of lazy stack value!");
                if type_.is_copy() {
                    unsafe {
                        let memory = Object::new_uninitialized(type_.clone())
                            .unwrap_or_else(|| {
                                panic!("Could not create object of `{}` type!", type_.name())
                            })
                            .into_inner()
                            .1;
                        memory.copy_from(
                            value
                                .as_ptr_raw()
                                .expect("Could not get lazy value content pointer!"),
                            type_.layout().size(),
                        );
                        let result = DynamicManaged::new_raw(
                            *value.type_hash(),
                            Lifetime::default(),
                            memory,
                            *type_.layout(),
                            type_.finalizer(),
                        )
                        .unwrap_or_else(|| {
                            panic!(
                                "Could not create managed object of `{}` type!",
                                type_.name()
                            )
                        });
                        context.stack().push(result);
                    }
                } else {
                    panic!("Type `{}` is not made for copies!", type_.name());
                }
            },
            |context, value| {
                let type_ = registry
                    .find_type(TypeQuery {
                        type_hash: value.type_hash(),
                        ..Default::default()
                    })
                    .expect("Could not find type of box stack value!");
                if type_.is_copy() {
                    unsafe {
                        let memory = Object::new_uninitialized(type_.clone())
                            .unwrap_or_else(|| {
                                panic!("Could not create object of `{}` type!", type_.name())
                            })
                            .into_inner()
                            .1;
                        memory.copy_from(
                            value
                                .as_ptr_raw()
                                .expect("Could not get box value content pointer!"),
                            type_.layout().size(),
                        );
                        let result = DynamicManaged::new_raw(
                            value.type_hash().unwrap(),
                            Lifetime::default(),
                            memory,
                            *type_.layout(),
                            type_.finalizer(),
                        )
                        .unwrap_or_else(|| {
                            panic!(
                                "Could not create managed object of `{}` type!",
                                type_.name()
                            )
                        });
                        context.stack().push(result);
                    }
                } else {
                    panic!("Type `{}` is not made for copies!", type_.name());
                }
            },
        );
    }

    fn move_into(context: &mut Context) {
        let value = context
            .stack()
            .pop::<DynamicManaged>()
            .expect("Value on stack is not managed owned value!");
        let type_hash = context
            .stack()
            .peek()
            .expect("Could not pop value from stack to borrow!");
        if type_hash == TypeHash::of::<DynamicManagedRefMut>() {
            let target = context
                .stack()
                .pop::<DynamicManagedRefMut>()
                .expect("Could not pop ref mut value from stack!");
            value
                .move_into_ref(target)
                .ok()
                .expect("Could not move managed value content!");
        } else if type_hash == TypeHash::of::<DynamicManagedLazy>() {
            let target = context
                .stack()
                .pop::<DynamicManagedLazy>()
                .expect("Could not pop lazy value from stack!");
            value
                .move_into_lazy(target)
                .ok()
                .expect("Could not move managed value content!");
        } else {
            panic!("Value can be moved into ref mut or lazy!");
        }
    }

    fn swap_in(context: &mut Context) {
        let mut value = context
            .stack()
            .pop::<DynamicManaged>()
            .expect("Value on stack is not managed owned value!");

        let type_hash = context
            .stack()
            .peek()
            .expect("Could not pop value from stack to borrow!");
        if type_hash == TypeHash::of::<DynamicManagedRefMut>() {
            let mut target = context
                .stack()
                .pop::<DynamicManagedRefMut>()
                .expect("Could not pop ref mut value from stack!");
            if value.type_hash() == target.type_hash() {
                unsafe {
                    let mut target = target
                        .as_mut_ptr_raw()
                        .expect("Could not get ref mut value content pointer!");
                    for value in value.memory_mut() {
                        target.swap(value as *mut u8);
                        target = target.add(1);
                    }
                }
            } else {
                panic!("Value and target have different types!");
            }
        } else if type_hash == TypeHash::of::<DynamicManagedLazy>() {
            let mut target = context
                .stack()
                .pop::<DynamicManagedLazy>()
                .expect("Could not pop lazy value from stack!");
            if value.type_hash() == target.type_hash() {
                unsafe {
                    let mut target = target
                        .as_mut_ptr_raw()
                        .expect("Could not get ref mut value content pointer!");
                    for value in value.memory_mut() {
                        target.swap(value as *mut u8);
                        target = target.add(1);
                    }
                }
            } else {
                panic!("Value and target have different types!");
            }
        } else if type_hash == TypeHash::of::<DynamicManagedBox>() {
            let mut target = context
                .stack()
                .pop::<DynamicManagedBox>()
                .expect("Could not pop box value from stack!");
            if *value.type_hash() == target.type_hash().expect("Could not get box value type!") {
                unsafe {
                    let mut target = target
                        .as_mut_ptr_raw()
                        .expect("Could not get ref mut value content pointer!");
                    for value in value.memory_mut() {
                        target.swap(value as *mut u8);
                        target = target.add(1);
                    }
                }
            } else {
                panic!("Value and target have different types!");
            }
        } else {
            panic!("Value can be swapped in ref mut, lazy or box!");
        }
    }

    fn destructure(context: &mut Context, registry: &Registry, fields: &[String]) {
        stack_managed_variant(
            context,
            |context, value| {
                let type_ = registry
                    .find_type(TypeQuery {
                        type_hash: Some(*value.type_hash()),
                        ..Default::default()
                    })
                    .expect("Could not find type of owned value!");
                if let Some(struct_type) = type_.as_struct() {
                    let pointer = unsafe { value.as_ptr_raw().cast_mut() };
                    for field in fields.iter().rev() {
                        let field = struct_type
                            .find_field(StructFieldQuery {
                                name: Some(field.into()),
                                ..Default::default()
                            })
                            .unwrap_or_else(|| {
                                panic!(
                                    "Could not find `{}` field in `{}` type!",
                                    field, struct_type.name
                                )
                            });
                        let pointer = unsafe { pointer.add(field.address_offset()) };
                        let type_hash = field.type_handle().type_hash();
                        unsafe {
                            if type_hash == TypeHash::of::<DynamicManaged>() {
                                context.stack().push(
                                    pointer.cast::<DynamicManaged>().replace(
                                        DynamicManaged::new(())
                                            .expect("Could not create managed object!"),
                                    ),
                                );
                            } else if type_hash == TypeHash::of::<DynamicManagedRef>() {
                                let lifetime = Lifetime::default();
                                context.stack().push(
                                    pointer.cast::<DynamicManagedRef>().replace(
                                        DynamicManagedRef::new_raw(
                                            *value.type_hash(),
                                            lifetime.borrow().unwrap(),
                                            std::ptr::null(),
                                        )
                                        .expect("Could not create managed object ref!"),
                                    ),
                                );
                            } else if type_hash == TypeHash::of::<DynamicManagedRefMut>() {
                                let lifetime = Lifetime::default();
                                context.stack().push(
                                    pointer.cast::<DynamicManagedRefMut>().replace(
                                        DynamicManagedRefMut::new_raw(
                                            *value.type_hash(),
                                            lifetime.borrow_mut().unwrap(),
                                            std::ptr::null_mut(),
                                        )
                                        .expect("Could not create managed object ref mut!"),
                                    ),
                                );
                            } else if type_hash == TypeHash::of::<DynamicManagedLazy>() {
                                let lifetime = Lifetime::default();
                                context.stack().push(
                                    pointer.cast::<DynamicManagedLazy>().replace(
                                        DynamicManagedLazy::new_raw(
                                            *value.type_hash(),
                                            lifetime.lazy(),
                                            std::ptr::null_mut(),
                                        )
                                        .expect("Could not create managed object ref mut!"),
                                    ),
                                );
                            } else if type_hash == TypeHash::of::<DynamicManagedBox>() {
                                context.stack().push(
                                    pointer
                                        .cast::<DynamicManagedBox>()
                                        .replace(DynamicManagedBox::new(())),
                                );
                            }
                        }
                    }
                } else {
                    panic!("`{}` is not a struct!", type_.name());
                }
            },
            |_, _| {
                panic!("Cannot destructure ref value!");
            },
            |_, _| {
                panic!("Cannot destructure ref mut value!");
            },
            |_, _| {
                panic!("Cannot destructure lazy value!");
            },
            |_, _| {
                panic!("Cannot destructure box value!");
            },
        );
    }

    fn structure(
        context: &mut Context,
        registry: &Registry,
        type_query: &AnkhaTypeQuery,
        fields: &[String],
    ) {
        let type_ = registry
            .find_type(type_query.compile())
            .unwrap_or_else(|| panic!("Could not find `{}` type!", type_query));
        if let Some(struct_type) = type_.as_struct() {
            if !struct_type.can_initialize() || struct_type.is_runtime() {
                for field in struct_type.fields() {
                    if !fields.contains(&field.name) {
                        panic!(
                            "Field `{}` of `{}` type must be initialized!",
                            field.name, struct_type.name
                        );
                    }
                }
            }
            unsafe {
                let memory = if struct_type.can_initialize() {
                    Object::new(type_.clone())
                } else {
                    Object::new_uninitialized(type_.clone()).unwrap_or_else(|| {
                        panic!("Could not create object of `{}` type!", struct_type.name)
                    })
                }
                .into_inner()
                .1;
                for field in fields.iter() {
                    let field = struct_type
                        .find_field(StructFieldQuery {
                            name: Some(field.into()),
                            ..Default::default()
                        })
                        .unwrap_or_else(|| {
                            panic!(
                                "Could not find `{}` field in `{}` type!",
                                field, struct_type.name
                            )
                        });
                    let type_hash = field.type_handle().type_hash();
                    if type_hash == TypeHash::of::<DynamicManaged>() {
                        let value = context
                            .stack()
                            .pop::<DynamicManaged>()
                            .expect("Value on stack is not owned managed!");
                        let pointer = memory.add(field.address_offset()).cast::<DynamicManaged>();
                        if field.type_handle().can_initialize() {
                            pointer.replace(value);
                        } else {
                            pointer.write(value);
                        }
                    } else if type_hash == TypeHash::of::<DynamicManagedRef>() {
                        let value = context
                            .stack()
                            .pop::<DynamicManagedRef>()
                            .expect("Value on stack is not ref managed!");
                        let pointer = memory
                            .add(field.address_offset())
                            .cast::<DynamicManagedRef>();
                        if field.type_handle().can_initialize() {
                            pointer.replace(value);
                        } else {
                            pointer.write(value);
                        }
                    } else if type_hash == TypeHash::of::<DynamicManagedRefMut>() {
                        let value = context
                            .stack()
                            .pop::<DynamicManagedRefMut>()
                            .expect("Value on stack is not ref mut managed!");
                        let pointer = memory
                            .add(field.address_offset())
                            .cast::<DynamicManagedRefMut>();
                        if field.type_handle().can_initialize() {
                            pointer.replace(value);
                        } else {
                            pointer.write(value);
                        }
                    } else if type_hash == TypeHash::of::<DynamicManagedLazy>() {
                        let value = context
                            .stack()
                            .pop::<DynamicManagedLazy>()
                            .expect("Value on stack is not lazy managed!");
                        let pointer = memory
                            .add(field.address_offset())
                            .cast::<DynamicManagedLazy>();
                        if field.type_handle().can_initialize() {
                            pointer.replace(value);
                        } else {
                            pointer.write(value);
                        }
                    } else if type_hash == TypeHash::of::<DynamicManagedBox>() {
                        let value = context
                            .stack()
                            .pop::<DynamicManagedBox>()
                            .expect("Value on stack is not box managed!");
                        let pointer = memory
                            .add(field.address_offset())
                            .cast::<DynamicManagedBox>();
                        if field.type_handle().can_initialize() {
                            pointer.replace(value);
                        } else {
                            pointer.write(value);
                        }
                    }
                }
                let result = DynamicManaged::new_raw(
                    struct_type.type_hash(),
                    Lifetime::default(),
                    memory,
                    *struct_type.layout(),
                    struct_type.finalizer(),
                )
                .unwrap_or_else(|| {
                    panic!("Could not create managed object of `{}` type!", type_query)
                });
                context.stack().push(result);
            }
        } else {
            panic!("Type `{}` is not a struct!", type_query);
        }
    }

    fn box_(context: &mut Context) {
        let value = context
            .stack()
            .pop::<DynamicManaged>()
            .expect("Stack value is not owned managed!");
        let (type_hash, _, memory, layout, finalizer) = value.into_inner();
        let mut result = DynamicManagedBox::new_uninitialized(type_hash, layout, finalizer);
        unsafe {
            result
                .as_mut_ptr_raw()
                .expect("Could not get target box value content pointer!")
                .copy_from(memory, layout.size());
            dealloc(memory, layout);
        }
        context.stack().push(result);
    }

    fn manage(context: &mut Context) {
        unsafe {
            let (layout, type_hash, finalizer, bytes) = context
                .stack()
                .pop_raw()
                .expect("Could not pop stack value!");
            let result = DynamicManaged::from_bytes(
                type_hash,
                Lifetime::default(),
                bytes,
                layout,
                finalizer,
            );
            context.stack().push(result);
        }
    }

    fn unmanage(context: &mut Context) {
        let value = context
            .stack()
            .pop::<DynamicManaged>()
            .expect("Stack value is not owned managed!");
        let (type_hash, _, memory, layout, finalizer) = value.into_inner();
        unsafe {
            context.stack().push_raw(
                layout,
                type_hash,
                finalizer,
                std::slice::from_raw_parts_mut(memory, layout.size()),
            );
            dealloc(memory, layout);
        }
    }

    fn copy(context: &mut Context, registry: &Registry) {
        let value = context
            .stack()
            .pop::<DynamicManaged>()
            .expect("Could not pop owned value from stack!");
        let type_ = registry
            .find_type(TypeQuery {
                type_hash: Some(*value.type_hash()),
                ..Default::default()
            })
            .expect("Could not find type of owned stack value!");
        if type_.is_copy() {
            unsafe {
                let memory = Object::new_uninitialized(type_.clone())
                    .unwrap_or_else(|| {
                        panic!("Could not create object of `{}` type!", type_.name())
                    })
                    .into_inner()
                    .1;
                memory.copy_from(value.as_ptr_raw(), type_.layout().size());
                let result = DynamicManaged::new_raw(
                    *value.type_hash(),
                    Lifetime::default(),
                    memory,
                    *type_.layout(),
                    type_.finalizer(),
                )
                .unwrap_or_else(|| {
                    panic!(
                        "Could not create managed object of `{}` type!",
                        type_.name()
                    )
                });
                context.stack().push(value);
                context.stack().push(result);
            }
        } else {
            panic!("Type `{}` is not made for copies!", type_.name());
        }
    }

    fn swap(context: &mut Context) {
        unsafe {
            let (layout_a, type_hash_a, finalizer_a, memory_a) = context
                .stack()
                .pop_raw()
                .expect("Cannot pop first value from stack to swap!");
            let (layout_b, type_hash_b, finalizer_b, memory_b) = context
                .stack()
                .pop_raw()
                .expect("Cannot pop second value from stack to swap!");
            context
                .stack()
                .push_raw(layout_a, type_hash_a, finalizer_a, &memory_a);
            context
                .stack()
                .push_raw(layout_b, type_hash_b, finalizer_b, &memory_b);
        }
    }

    fn duplicate_box(context: &mut Context) {
        let value = context
            .stack()
            .pop::<DynamicManagedBox>()
            .expect("Stack value is not box managed!");
        context.stack().push(value.clone());
        context.stack().push(value);
    }

    fn ensure_stack_type(context: &mut Context, registry: &Registry, type_query: &AnkhaTypeQuery) {
        let expected = registry
            .find_type(type_query.compile())
            .unwrap_or_else(|| panic!("Could not find `{}` type!", type_query))
            .type_hash();
        stack_managed_variant(
            context,
            |context, value| {
                if *value.type_hash() != expected {
                    panic!("Top stack managed value is not type of: `{}`", type_query);
                }
                context.stack().push(value);
            },
            |context, value| {
                if *value.type_hash() != expected {
                    panic!("Top stack managed value is not type of: `{}`", type_query);
                }
                context.stack().push(value);
            },
            |context, value| {
                if *value.type_hash() != expected {
                    panic!("Top stack managed value is not type of: `{}`", type_query);
                }
                context.stack().push(value);
            },
            |context, value| {
                if *value.type_hash() != expected {
                    panic!("Top stack managed value is not type of: `{}`", type_query);
                }
                context.stack().push(value);
            },
            |context, value| {
                if value.type_hash().unwrap() != expected {
                    panic!("Top stack managed value is not type of: `{}`", type_query);
                }
                context.stack().push(value);
            },
        );
    }

    fn ensure_register_type(
        context: &mut Context,
        registry: &Registry,
        type_query: &AnkhaTypeQuery,
        index: usize,
    ) {
        let expected = registry
            .find_type(type_query.compile())
            .unwrap_or_else(|| panic!("Could not find `{}` type!", type_query))
            .type_hash();
        register_managed_variant(
            context,
            index,
            |value| {
                if *value.type_hash() != expected {
                    panic!(
                        "Register: {} managed value is not type of: `{}`",
                        index, type_query
                    );
                }
            },
            |value| {
                if *value.type_hash() != expected {
                    panic!(
                        "Register: {} managed value is not type of: `{}`",
                        index, type_query
                    );
                }
            },
            |value| {
                if *value.type_hash() != expected {
                    panic!(
                        "Register: {} managed value is not type of: `{}`",
                        index, type_query
                    );
                }
            },
            |value| {
                if *value.type_hash() != expected {
                    panic!(
                        "Register: {} managed value is not type of: `{}`",
                        index, type_query
                    );
                }
            },
            |value| {
                if value.type_hash().unwrap() != expected {
                    panic!(
                        "Register: {} managed value is not type of: `{}`",
                        index, type_query
                    );
                }
            },
        )
    }

    fn ensure_stack_kind(context: &mut Context, kind: AnkhaValueKind) {
        let type_hash = context
            .stack()
            .peek()
            .expect("Could not peek top stack value!");
        let provided = AnkhaValueKind::from_type_hash(type_hash);
        if provided != kind {
            panic!(
                "Expected {:?} top stack value kind - got: {:?}!",
                kind, provided
            );
        }
    }

    fn ensure_register_kind(context: &mut Context, kind: AnkhaValueKind, index: usize) {
        let index = context.absolute_register_index(index);
        let register = context
            .registers()
            .access_register(index)
            .unwrap_or_else(|| panic!("Could not access non-existent register: {}", index));
        let type_hash = register.type_hash();
        let provided = AnkhaValueKind::from_type_hash(type_hash);
        if provided != kind {
            panic!(
                "Expected {:?} register #{} value kind - got: {:?}!",
                kind, index, provided
            );
        }
    }

    fn call_method(
        context: &mut Context,
        registry: &Registry,
        function_query: &AnkhaFunctionQuery,
    ) {
        let type_hash = stack_managed_variant(
            context,
            |context, value| {
                let result = *value.type_hash();
                context.stack().push(value);
                result
            },
            |context, value| {
                let result = *value.type_hash();
                context.stack().push(value);
                result
            },
            |context, value| {
                let result = *value.type_hash();
                context.stack().push(value);
                result
            },
            |context, value| {
                let result = *value.type_hash();
                context.stack().push(value);
                result
            },
            |context, value| {
                let result = value.type_hash().unwrap();
                context.stack().push(value);
                result
            },
        );
        let mut query = function_query.compile();
        query.type_query = Some(TypeQuery {
            type_hash: Some(type_hash),
            ..Default::default()
        });
        let handle = registry
            .functions()
            .find(|handle| query.is_valid(handle.signature()))
            .unwrap_or_else(|| panic!("Could not call non-existent function: {:#?}", query));
        handle.invoke(context, registry);
    }

    fn call_indirect(context: &mut Context, registry: &Registry) {
        let handle = stack_managed_variant(
            context,
            |_, value| {
                value
                    .consume::<Function>()
                    .ok()
                    .expect("Stack value is not Function!")
                    .0
            },
            |_, value| {
                value
                    .read::<Function>()
                    .expect("Stack value is not Function!")
                    .0
                    .clone()
            },
            |_, value| {
                value
                    .read::<Function>()
                    .expect("Stack value is not Function!")
                    .0
                    .clone()
            },
            |_, value| {
                value
                    .read::<Function>()
                    .expect("Stack value is not Function!")
                    .0
                    .clone()
            },
            |_, value| {
                value
                    .read::<Function>()
                    .expect("Stack value is not Function!")
                    .0
                    .clone()
            },
        );
        handle.invoke(context, registry);
    }

    fn get_type(context: &mut Context, registry: &Registry, query: &AnkhaTypeQuery) {
        let handle = registry
            .find_type(query.compile())
            .unwrap_or_else(|| panic!("Could not find `{}` type!", query));
        context.stack().push(Type(handle));
    }

    fn get_function(context: &mut Context, registry: &Registry, query: &AnkhaFunctionQuery) {
        let handle = registry
            .find_function(query.compile())
            .unwrap_or_else(|| panic!("Could not find `{}` type!", query));
        context.stack().push(Function(handle));
    }
}

impl ScriptExpression for AnkhaExpression {
    fn evaluate(&self, context: &mut Context, registry: &Registry) {
        match self {
            Self::Literal(literal) => {
                literal.evaluate(context);
            }
            Self::StackDrop => {
                context.stack().drop();
            }
            Self::StackUnwrapBoolean => {
                Self::stack_unwrap_boolean(context);
            }
            Self::Borrow => {
                Self::borrow(context);
            }
            Self::BorrowMut => {
                Self::borrow_mut(context);
            }
            Self::Lazy => {
                Self::lazy(context);
            }
            Self::BorrowField {
                name,
                kind,
                visibility,
            } => {
                Self::borrow_managed_field(context, registry, name, *kind, *visibility);
            }
            Self::BorrowMutField {
                name,
                kind,
                visibility,
            } => {
                Self::borrow_mut_managed_field(context, registry, name, *kind, *visibility);
            }
            Self::BorrowUnmanagedField { query } => {
                Self::borrow_unmanaged_field(context, registry, query);
            }
            Self::BorrowMutUnmanagedField { query } => {
                Self::borrow_mut_unmanaged_field(context, registry, query);
            }
            Self::CopyFrom => {
                Self::copy_from(context, registry);
            }
            Self::MoveInto => {
                Self::move_into(context);
            }
            Self::SwapIn => {
                Self::swap_in(context);
            }
            Self::Destructure { fields } => {
                Self::destructure(context, registry, fields);
            }
            Self::Structure { type_query, fields } => {
                Self::structure(context, registry, type_query, fields);
            }
            Self::Box => {
                Self::box_(context);
            }
            Self::Manage => {
                Self::manage(context);
            }
            Self::Unmanage => {
                Self::unmanage(context);
            }
            Self::Copy => {
                Self::copy(context, registry);
            }
            Self::Swap => {
                Self::swap(context);
            }
            Self::DuplicateBox => {
                Self::duplicate_box(context);
            }
            Self::EnsureStackType { type_query } => {
                Self::ensure_stack_type(context, registry, type_query);
            }
            Self::EnsureRegisterType { type_query, index } => {
                Self::ensure_register_type(context, registry, type_query, *index);
            }
            Self::EnsureStackKind { kind } => {
                Self::ensure_stack_kind(context, *kind);
            }
            Self::EnsureRegisterKind { kind, index } => {
                Self::ensure_register_kind(context, *kind, *index);
            }
            Self::CallMethod { function_query } => {
                Self::call_method(context, registry, function_query);
            }
            Self::CallIndirect => {
                Self::call_indirect(context, registry);
            }
            Self::GetType { query } => {
                Self::get_type(context, registry, query);
            }
            Self::GetFunction { query } => {
                Self::get_function(context, registry, query);
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AnkhaRegisterAddress {
    Index(usize),
    Name(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AnkhaOperation {
    Expression(AnkhaExpression),
    Group(AnkhaScript),
    GroupReversed(AnkhaScript),
    MakeRegister {
        kind: AnkhaValueKind,
        name: Option<String>,
    },
    DropRegister(AnkhaRegisterAddress),
    PushFromRegister(AnkhaRegisterAddress),
    PopToRegister(AnkhaRegisterAddress),
    CallFunction(AnkhaFunctionQuery),
    BranchScope {
        script_success: AnkhaScript,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        script_failure: Option<AnkhaScript>,
    },
    LoopScope {
        script: AnkhaScript,
    },
    PushScope {
        script: AnkhaScript,
    },
    PopScope,
    EnsureRegisterType {
        type_query: AnkhaTypeQuery,
        address: AnkhaRegisterAddress,
    },
    EnsureRegisterKind {
        kind: AnkhaValueKind,
        address: AnkhaRegisterAddress,
    },
}

fn write_operation(
    operation: &AnkhaOperation,
    registers: &mut Vec<Option<String>>,
    result: &mut Vec<ScriptOperation<'static, AnkhaExpression>>,
) {
    match operation {
        AnkhaOperation::Expression(expression) => {
            result.push(ScriptOperation::Expression {
                expression: expression.to_owned(),
            });
        }
        AnkhaOperation::Group(operations) => {
            for operation in operations {
                write_operation(operation, registers, result);
            }
        }
        AnkhaOperation::GroupReversed(operations) => {
            for operation in operations.iter().rev() {
                write_operation(operation, registers, result);
            }
        }
        AnkhaOperation::MakeRegister { kind, name } => {
            let type_hash = kind.type_hash();
            if type_hash.is_none() {
                panic!("Using any value kind is forbidden for defining registers!");
            }
            registers.push(name.to_owned());
            result.push(ScriptOperation::DefineRegister {
                query: TypeQuery {
                    type_hash,
                    ..Default::default()
                },
            });
        }
        AnkhaOperation::DropRegister(address) => {
            result.push(match address {
                AnkhaRegisterAddress::Index(index) => {
                    ScriptOperation::DropRegister { index: *index }
                }
                AnkhaRegisterAddress::Name(name) => {
                    let index = registers
                        .iter()
                        .position(|register| {
                            register
                                .as_deref()
                                .map(|register| register == name)
                                .unwrap_or_default()
                        })
                        .unwrap_or_else(|| {
                            panic!("There is no register with `{}` name in the scope!", name)
                        });
                    ScriptOperation::DropRegister { index }
                }
            });
        }
        AnkhaOperation::PushFromRegister(address) => {
            result.push(match address {
                AnkhaRegisterAddress::Index(index) => {
                    ScriptOperation::PushFromRegister { index: *index }
                }
                AnkhaRegisterAddress::Name(name) => {
                    let index = registers
                        .iter()
                        .position(|register| {
                            register
                                .as_deref()
                                .map(|register| register == name)
                                .unwrap_or_default()
                        })
                        .unwrap_or_else(|| {
                            panic!("There is no register with `{}` name in the scope!", name)
                        });
                    ScriptOperation::PushFromRegister { index }
                }
            });
        }
        AnkhaOperation::PopToRegister(address) => {
            result.push(match address {
                AnkhaRegisterAddress::Index(index) => {
                    ScriptOperation::PopToRegister { index: *index }
                }
                AnkhaRegisterAddress::Name(name) => {
                    let index = registers
                        .iter()
                        .position(|register| {
                            register
                                .as_deref()
                                .map(|register| register == name)
                                .unwrap_or_default()
                        })
                        .unwrap_or_else(|| {
                            panic!("There is no register with `{}` name in the scope!", name)
                        });
                    ScriptOperation::PopToRegister { index }
                }
            });
        }
        AnkhaOperation::CallFunction(function_query) => {
            result.push(ScriptOperation::CallFunction {
                query: function_query.compile().to_static(),
            });
        }
        AnkhaOperation::BranchScope {
            script_success: operations_success,
            script_failure: operations_failure,
        } => {
            result.push(ScriptOperation::BranchScope {
                scope_success: build_script(operations_success),
                scope_failure: operations_failure.as_ref().map(build_script),
            });
        }
        AnkhaOperation::LoopScope { script: operations } => {
            result.push(ScriptOperation::LoopScope {
                scope: build_script(operations),
            });
        }
        AnkhaOperation::PushScope { script: operations } => {
            result.push(ScriptOperation::PushScope {
                scope: build_script(operations),
            });
        }
        AnkhaOperation::PopScope => {
            result.push(ScriptOperation::PopScope);
        }
        AnkhaOperation::EnsureRegisterType {
            type_query,
            address,
        } => {
            result.push(match address {
                AnkhaRegisterAddress::Index(index) => ScriptOperation::Expression {
                    expression: AnkhaExpression::EnsureRegisterType {
                        type_query: type_query.to_owned(),
                        index: *index,
                    },
                },
                AnkhaRegisterAddress::Name(name) => {
                    let index = registers
                        .iter()
                        .position(|register| {
                            register
                                .as_deref()
                                .map(|register| register == name)
                                .unwrap_or_default()
                        })
                        .unwrap_or_else(|| {
                            panic!("There is no register with `{}` name in the scope!", name)
                        });
                    ScriptOperation::Expression {
                        expression: AnkhaExpression::EnsureRegisterType {
                            type_query: type_query.to_owned(),
                            index,
                        },
                    }
                }
            });
        }
        AnkhaOperation::EnsureRegisterKind { kind, address } => {
            result.push(match address {
                AnkhaRegisterAddress::Index(index) => ScriptOperation::Expression {
                    expression: AnkhaExpression::EnsureRegisterKind {
                        kind: *kind,
                        index: *index,
                    },
                },
                AnkhaRegisterAddress::Name(name) => {
                    let index = registers
                        .iter()
                        .position(|register| {
                            register
                                .as_deref()
                                .map(|register| register == name)
                                .unwrap_or_default()
                        })
                        .unwrap_or_else(|| {
                            panic!("There is no register with `{}` name in the scope!", name)
                        });
                    ScriptOperation::Expression {
                        expression: AnkhaExpression::EnsureRegisterKind { kind: *kind, index },
                    }
                }
            });
        }
    }
}

fn build_script(script: &AnkhaScript) -> ScriptHandle<'static, AnkhaExpression> {
    let mut registers = vec![];
    let mut result = vec![];
    for operation in script {
        write_operation(operation, &mut registers, &mut result);
    }
    ScriptHandle::new(result)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnkhaFunctionParameter {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub meta: Option<Meta>,
    pub name: String,
    #[serde(default, skip_serializing_if = "AnkhaValueKind::is_any")]
    pub kind: AnkhaValueKind,
}

impl AnkhaFunctionParameter {
    pub fn compile(&self) -> ScriptFunctionParameter<'static> {
        ScriptFunctionParameter {
            meta: self.meta.to_owned(),
            name: self.name.to_owned(),
            type_query: TypeQuery {
                type_hash: self.kind.type_hash(),
                ..Default::default()
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnkhaFunction {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub meta: Option<Meta>,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub type_name_module: Option<(String, Option<String>)>,
    #[serde(default, skip_serializing_if = "Visibility::is_public")]
    pub visibility: Visibility,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub inputs: Vec<AnkhaFunctionParameter>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub outputs: Vec<AnkhaFunctionParameter>,
    pub script: AnkhaScript,
}

impl AnkhaFunction {
    pub fn compile(&self, module_name: &str) -> ScriptFunction<'static, AnkhaExpression> {
        ScriptFunction {
            signature: ScriptFunctionSignature {
                meta: self.meta.to_owned(),
                name: self.name.to_owned(),
                module_name: Some(module_name.to_owned()),
                type_query: self
                    .type_name_module
                    .as_ref()
                    .map(|(type_name, module_name)| TypeQuery {
                        name: Some(type_name.to_owned().into()),
                        module_name: module_name.as_ref().map(|name| name.to_owned().into()),
                        ..Default::default()
                    }),
                visibility: self.visibility,
                inputs: self
                    .inputs
                    .iter()
                    .map(|parameter| parameter.compile())
                    .collect(),
                outputs: self
                    .outputs
                    .iter()
                    .map(|parameter| parameter.compile())
                    .collect(),
            },
            script: build_script(&self.script),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnkhaStructField {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub meta: Option<Meta>,
    pub name: String,
    #[serde(default, skip_serializing_if = "Visibility::is_public")]
    pub visibility: Visibility,
    #[serde(default, skip_serializing_if = "AnkhaValueKind::is_any")]
    pub kind: AnkhaValueKind,
}

impl AnkhaStructField {
    pub fn compile(&self) -> ScriptStructField<'static> {
        ScriptStructField {
            meta: self.meta.to_owned(),
            name: self.name.to_owned(),
            visibility: self.visibility,
            type_query: TypeQuery {
                type_hash: self.kind.type_hash(),
                ..Default::default()
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnkhaStruct {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub meta: Option<Meta>,
    pub name: String,
    #[serde(default, skip_serializing_if = "Visibility::is_public")]
    pub visibility: Visibility,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fields: Vec<AnkhaStructField>,
}

impl AnkhaStruct {
    pub fn compile(&self, module_name: &str) -> ScriptStruct<'static> {
        ScriptStruct {
            meta: self.meta.to_owned(),
            name: self.name.to_owned(),
            module_name: Some(module_name.to_owned()),
            visibility: self.visibility,
            fields: self.fields.iter().map(|field| field.compile()).collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnkhaEnumVariant {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub meta: Option<Meta>,
    pub name: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fields: Vec<AnkhaStructField>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub discriminant: Option<u8>,
}

impl AnkhaEnumVariant {
    pub fn compile(&self) -> ScriptEnumVariant<'static> {
        ScriptEnumVariant {
            meta: self.meta.to_owned(),
            name: self.name.to_owned(),
            fields: self.fields.iter().map(|field| field.compile()).collect(),
            discriminant: self.discriminant,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnkhaEnum {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub meta: Option<Meta>,
    pub name: String,
    #[serde(default, skip_serializing_if = "Visibility::is_public")]
    pub visibility: Visibility,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub variants: Vec<AnkhaEnumVariant>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_variant: Option<u8>,
}

impl AnkhaEnum {
    pub fn compile(&self, module_name: &str) -> ScriptEnum<'static> {
        ScriptEnum {
            meta: self.meta.to_owned(),
            name: self.name.to_owned(),
            module_name: Some(module_name.to_owned()),
            visibility: self.visibility,
            variants: self
                .variants
                .iter()
                .map(|variant| variant.compile())
                .collect(),
            default_variant: self.default_variant,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnkhaModule {
    pub name: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub structs: Vec<AnkhaStruct>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub enums: Vec<AnkhaEnum>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub functions: Vec<AnkhaFunction>,
}

impl AnkhaModule {
    pub fn compile(&self) -> ScriptModule<'static, AnkhaExpression> {
        ScriptModule {
            name: self.name.to_owned(),
            structs: self
                .structs
                .iter()
                .map(|struct_type| struct_type.compile(&self.name))
                .collect(),
            enums: self
                .enums
                .iter()
                .map(|enum_type| enum_type.compile(&self.name))
                .collect(),
            functions: self
                .functions
                .iter()
                .map(|function| function.compile(&self.name))
                .collect(),
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnkhaFile {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dependencies: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub modules: Vec<AnkhaModule>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct AnkhaPackage {
    pub files: HashMap<String, AnkhaFile>,
}

impl AnkhaPackage {
    pub fn new<CP>(path: &str, content_provider: &mut CP) -> Result<Self, Box<dyn Error>>
    where
        CP: ScriptContentProvider<AnkhaFile>,
    {
        let mut result = Self::default();
        result.load(path, content_provider)?;
        Ok(result)
    }

    pub fn load<CP>(&mut self, path: &str, content_provider: &mut CP) -> Result<(), Box<dyn Error>>
    where
        CP: ScriptContentProvider<AnkhaFile>,
    {
        let path = content_provider.sanitize_path(path)?;
        if self.files.contains_key(&path) {
            return Ok(());
        }
        for content in content_provider.unpack_load(&path)? {
            if let Some(file) = content.data? {
                let dependencies = file.dependencies.to_owned();
                self.files.insert(content.name, file);
                for relative in dependencies {
                    let path = content_provider.join_paths(&content.path, &relative)?;
                    self.load(&path, content_provider)?;
                }
            }
        }
        Ok(())
    }

    pub fn compile(&self) -> ScriptPackage<'static, AnkhaExpression> {
        ScriptPackage {
            modules: self
                .files
                .values()
                .flat_map(|file| file.modules.iter())
                .map(|module| module.compile())
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        library::{AnkhaScriptBuilder, AnkhaVmScope},
        parser::AnkhaContentParser,
    };
    use intuicio_backend_vm::prelude::*;
    use intuicio_core::prelude::*;
    use intuicio_data::prelude::*;
    use intuicio_derive::*;

    pub struct LexprContentParser;

    impl BytesContentParser<AnkhaFile> for LexprContentParser {
        fn parse(&self, bytes: Vec<u8>) -> Result<AnkhaFile, Box<dyn Error>> {
            let content = String::from_utf8(bytes)?;
            Ok(serde_lexpr::from_str::<AnkhaFile>(&content)?)
        }
    }

    #[intuicio_function(
        transformer = "DynamicManagedValueTransformer",
        module_name = "intrinsics"
    )]
    fn add(a: i32, b: i32) -> i32 {
        a + b
    }

    #[intuicio_function(
        transformer = "DynamicManagedValueTransformer",
        module_name = "intrinsics"
    )]
    fn sub(a: i32, b: i32) -> i32 {
        a - b
    }

    #[intuicio_function(
        transformer = "DynamicManagedValueTransformer",
        module_name = "intrinsics"
    )]
    fn mul(a: i32, b: i32) -> i32 {
        a * b
    }

    #[intuicio_function(
        transformer = "DynamicManagedValueTransformer",
        module_name = "intrinsics"
    )]
    fn eq(a: i32, b: i32) -> bool {
        a == b
    }

    #[intuicio_function(module_name = "intrinsics")]
    fn unmanaged_add(a: i32, b: i32) -> i32 {
        a + b
    }

    #[derive(IntuicioStruct)]
    struct Foo {
        pub a: DynamicManaged,
        pub b: DynamicManagedBox,
    }

    impl Default for Foo {
        fn default() -> Self {
            let a = DynamicManaged::new(40).unwrap();
            let b = DynamicManagedBox::new(2);
            Self { a, b }
        }
    }

    #[test]
    fn test_script() {
        let mut registry = Registry::default().with_basic_types();
        crate::library::install(&mut registry);
        registry.add_type(Foo::define_struct(&registry));
        registry.add_function(add::define_function(&registry));
        registry.add_function(unmanaged_add::define_function(&registry));
        let mut context = Context::new(10240, 10240);

        let script = AnkhaScriptBuilder::default()
            .expression(AnkhaExpression::Literal(AnkhaLiteral::I32(40)))
            .expression(AnkhaExpression::EnsureStackType {
                type_query: AnkhaTypeQuery {
                    type_hash: Some(TypeHash::of::<i32>()),
                    ..Default::default()
                },
            })
            .expression(AnkhaExpression::Literal(AnkhaLiteral::I32(2)))
            .expression(AnkhaExpression::EnsureStackType {
                type_query: AnkhaTypeQuery {
                    type_hash: Some(TypeHash::of::<i32>()),
                    ..Default::default()
                },
            })
            .call_function(FunctionQuery {
                name: Some("add".into()),
                ..Default::default()
            })
            .expression(AnkhaExpression::EnsureStackType {
                type_query: AnkhaTypeQuery {
                    type_hash: Some(TypeHash::of::<i32>()),
                    ..Default::default()
                },
            })
            .build();
        let mut vm = VmScope::new(script, Default::default());
        vm.run(&mut context, &registry);
        assert_eq!(
            context
                .stack()
                .pop::<DynamicManaged>()
                .unwrap()
                .consume::<i32>()
                .ok()
                .unwrap(),
            42
        );
        context.stack().restore(unsafe { DataStackToken::new(0) });

        let script = AnkhaScriptBuilder::default()
            .expression(AnkhaExpression::Literal(AnkhaLiteral::I32(40)))
            .expression(AnkhaExpression::EnsureStackType {
                type_query: AnkhaTypeQuery {
                    type_hash: Some(TypeHash::of::<i32>()),
                    ..Default::default()
                },
            })
            .expression(AnkhaExpression::Unmanage)
            .expression(AnkhaExpression::Literal(AnkhaLiteral::I32(2)))
            .expression(AnkhaExpression::EnsureStackType {
                type_query: AnkhaTypeQuery {
                    type_hash: Some(TypeHash::of::<i32>()),
                    ..Default::default()
                },
            })
            .expression(AnkhaExpression::Unmanage)
            .call_function(FunctionQuery {
                name: Some("unmanaged_add".into()),
                ..Default::default()
            })
            .expression(AnkhaExpression::Manage)
            .expression(AnkhaExpression::EnsureStackType {
                type_query: AnkhaTypeQuery {
                    type_hash: Some(TypeHash::of::<i32>()),
                    ..Default::default()
                },
            })
            .build();
        let mut vm = VmScope::new(script, Default::default());
        vm.run(&mut context, &registry);
        assert_eq!(
            context
                .stack()
                .pop::<DynamicManaged>()
                .unwrap()
                .consume::<i32>()
                .ok()
                .unwrap(),
            42
        );
        context.stack().restore(unsafe { DataStackToken::new(0) });

        let script = AnkhaScriptBuilder::default()
            .expression(AnkhaExpression::Structure {
                type_query: AnkhaTypeQuery {
                    type_hash: Some(TypeHash::of::<Foo>()),
                    ..Default::default()
                },
                fields: Default::default(),
            })
            .expression(AnkhaExpression::Borrow)
            .expression(AnkhaExpression::Swap)
            .expression(AnkhaExpression::BorrowField {
                name: "a".to_owned(),
                kind: AnkhaValueKind::Owned,
                visibility: None,
            })
            .expression(AnkhaExpression::CopyFrom)
            .expression(AnkhaExpression::EnsureStackType {
                type_query: AnkhaTypeQuery {
                    type_hash: Some(TypeHash::of::<i32>()),
                    ..Default::default()
                },
            })
            .expression(AnkhaExpression::Literal(AnkhaLiteral::I32(2)))
            .expression(AnkhaExpression::EnsureStackType {
                type_query: AnkhaTypeQuery {
                    type_hash: Some(TypeHash::of::<i32>()),
                    ..Default::default()
                },
            })
            .call_function(FunctionQuery {
                name: Some("add".into()),
                ..Default::default()
            })
            .expression(AnkhaExpression::EnsureStackType {
                type_query: AnkhaTypeQuery {
                    type_hash: Some(TypeHash::of::<i32>()),
                    ..Default::default()
                },
            })
            .build();
        let mut vm = VmScope::new(script, Default::default());
        vm.run(&mut context, &registry);
        assert_eq!(
            *context
                .stack()
                .pop::<DynamicManaged>()
                .unwrap()
                .read::<i32>()
                .unwrap(),
            42
        );
        context.stack().restore(unsafe { DataStackToken::new(0) });

        let script = AnkhaScriptBuilder::default()
            .define_register(TypeQuery {
                type_hash: Some(TypeHash::of::<DynamicManaged>()),
                ..Default::default()
            })
            .expression(AnkhaExpression::Structure {
                type_query: AnkhaTypeQuery {
                    type_hash: Some(TypeHash::of::<Foo>()),
                    ..Default::default()
                },
                fields: Default::default(),
            })
            .expression(AnkhaExpression::Borrow)
            .pop_to_register(0)
            .expression(AnkhaExpression::EnsureStackType {
                type_query: AnkhaTypeQuery {
                    type_hash: Some(TypeHash::of::<Foo>()),
                    ..Default::default()
                },
            })
            .expression(AnkhaExpression::BorrowField {
                name: "a".to_owned(),
                kind: AnkhaValueKind::Owned,
                visibility: None,
            })
            .expression(AnkhaExpression::CopyFrom)
            .expression(AnkhaExpression::EnsureStackType {
                type_query: AnkhaTypeQuery {
                    type_hash: Some(TypeHash::of::<i32>()),
                    ..Default::default()
                },
            })
            .expression(AnkhaExpression::Literal(AnkhaLiteral::I32(2)))
            .expression(AnkhaExpression::EnsureStackType {
                type_query: AnkhaTypeQuery {
                    type_hash: Some(TypeHash::of::<i32>()),
                    ..Default::default()
                },
            })
            .call_function(FunctionQuery {
                name: Some("add".into()),
                ..Default::default()
            })
            .push_from_register(0)
            .expression(AnkhaExpression::BorrowMut)
            .pop_to_register(0)
            .expression(AnkhaExpression::BorrowMutField {
                name: "a".to_owned(),
                kind: AnkhaValueKind::Owned,
                visibility: None,
            })
            .expression(AnkhaExpression::Swap)
            .expression(AnkhaExpression::MoveInto)
            .push_from_register(0)
            .build();
        let mut vm = VmScope::new(script, Default::default());
        vm.run(&mut context, &registry);
        let foo = context.stack().pop::<DynamicManaged>().unwrap();
        assert_eq!(*foo.read::<Foo>().unwrap().a.read::<i32>().unwrap(), 42);
        context
            .registers()
            .restore(unsafe { DataStackToken::new(0) });
        context.stack().restore(unsafe { DataStackToken::new(0) });
    }

    #[test]
    fn test_frontend_lexpr() {
        let mut registry = Registry::default().with_basic_types();
        crate::library::install(&mut registry);
        registry.add_function(add::define_function(&registry));
        let mut content_provider = FileContentProvider::new("lexpr", LexprContentParser);
        AnkhaPackage::new("../../resources/package.lexpr", &mut content_provider)
            .unwrap()
            .compile()
            .install::<AnkhaVmScope>(
                &mut registry,
                None,
                // Some(
                //     crate::library::printables(PrintDebugger::full())
                //         .basic_printables()
                //         // .step_through(false)
                //         .stack_bytes(false)
                //         .registers_bytes(false)
                //         .into_handle(),
                // ),
            );
        assert!(
            registry
                .find_function(FunctionQuery {
                    name: Some("main".into()),
                    module_name: Some("test".into()),
                    ..Default::default()
                })
                .is_some()
        );

        let mut host = Host::new(Context::new(10240, 10240), RegistryHandle::new(registry));
        let (result,) = host
            .call_function::<(DynamicManaged,), _>("main", "test", None)
            .unwrap()
            .run(());
        assert_eq!(*result.read::<i32>().unwrap(), 42);

        let (result,) = host
            .call_function::<(DynamicManaged,), _>("main2", "test", None)
            .unwrap()
            .run(());
        let struct_type = host
            .registry()
            .find_type(TypeQuery {
                type_hash: Some(*result.type_hash()),
                ..Default::default()
            })
            .unwrap();
        let field = struct_type
            .find_struct_field(StructFieldQuery {
                name: Some("a".into()),
                ..Default::default()
            })
            .unwrap();
        let result = unsafe {
            result
                .as_ptr_raw()
                .add(field.address_offset())
                .cast::<DynamicManaged>()
                .as_ref()
                .unwrap()
        };
        assert_eq!(*result.read::<i32>().unwrap(), 42);
    }

    #[test]
    fn test_frontend_ankha() {
        let mut registry = Registry::default().with_basic_types();
        crate::library::install(&mut registry);
        registry.add_function(add::define_function(&registry));
        registry.add_function(sub::define_function(&registry));
        registry.add_function(mul::define_function(&registry));
        registry.add_function(eq::define_function(&registry));
        let mut content_provider = FileContentProvider::new("ankha", AnkhaContentParser::default());
        AnkhaPackage::new("../../resources/package.ankha", &mut content_provider)
            .unwrap()
            .compile()
            .install::<AnkhaVmScope>(
                &mut registry,
                None,
                // Some(
                //     crate::library::printables(PrintDebugger::full())
                //         .basic_printables()
                //         // .step_through(false)
                //         .stack_bytes(false)
                //         .registers_bytes(false)
                //         .into_handle(),
                // ),
            );
        assert!(
            registry
                .find_function(FunctionQuery {
                    name: Some("main".into()),
                    module_name: Some("test".into()),
                    ..Default::default()
                })
                .is_some()
        );
        assert!(
            registry
                .find_function(FunctionQuery {
                    name: Some("factorial".into()),
                    module_name: Some("test".into()),
                    ..Default::default()
                })
                .is_some()
        );
        let mut host = Host::new(Context::new(10240, 10240), RegistryHandle::new(registry));

        let (result,) = host
            .call_function::<(DynamicManaged,), _>("main", "test", None)
            .unwrap()
            .run(());
        assert_eq!(*result.read::<i32>().unwrap(), 42);

        let (result,) = host
            .call_function::<(DynamicManaged,), _>("main2", "test", None)
            .unwrap()
            .run(());
        let struct_type = host
            .registry()
            .find_type(TypeQuery {
                type_hash: Some(*result.type_hash()),
                ..Default::default()
            })
            .unwrap();
        let field = struct_type
            .find_struct_field(StructFieldQuery {
                name: Some("a".into()),
                ..Default::default()
            })
            .unwrap();
        let result = unsafe {
            result
                .as_ptr_raw()
                .add(field.address_offset())
                .cast::<DynamicManaged>()
                .as_ref()
                .unwrap()
        };
        assert_eq!(*result.read::<i32>().unwrap(), 42);

        let (result,) = host
            .call_function::<(DynamicManaged,), _>("factorial", "test", None)
            .unwrap()
            .run((DynamicManaged::new(5).ok().unwrap(),));
        assert_eq!(*result.read::<i32>().unwrap(), 120);
    }
}
