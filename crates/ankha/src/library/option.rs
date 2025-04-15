use crate::script::{AnkhaLiteral, stack_managed_variant};
use intuicio_core::{
    IntuicioEnum,
    context::Context,
    registry::Registry,
    transformer::{DynamicManagedValueTransformer, ValueTransformer},
};
use intuicio_data::{
    managed::{DynamicManaged, DynamicManagedLazy, DynamicManagedRef, DynamicManagedRefMut},
    managed_box::DynamicManagedBox,
    type_hash::TypeHash,
};
use intuicio_derive::{IntuicioEnum, intuicio_method, intuicio_methods};

pub fn install(registry: &mut Registry) {
    registry.add_type(AnkhaOption::define_enum(registry));
    registry.add_function(AnkhaOption::script_wrap__define_function(registry));
    registry.add_function(AnkhaOption::script_unwrap__define_function(registry));
    registry.add_function(AnkhaOption::script_none__define_function(registry));
    registry.add_function(AnkhaOption::script_some__define_function(registry));
    registry.add_function(AnkhaOption::script_some_ref__define_function(registry));
    registry.add_function(AnkhaOption::script_some_ref_mut__define_function(registry));
    registry.add_function(AnkhaOption::script_some_lazy__define_function(registry));
    registry.add_function(AnkhaOption::script_some_box__define_function(registry));
    registry.add_function(AnkhaOption::script_is_none__define_function(registry));
    registry.add_function(AnkhaOption::script_is_some__define_function(registry));
    registry.add_function(AnkhaOption::script_is_owned__define_function(registry));
    registry.add_function(AnkhaOption::script_is_ref__define_function(registry));
    registry.add_function(AnkhaOption::script_is_ref_mut__define_function(registry));
    registry.add_function(AnkhaOption::script_is_lazy__define_function(registry));
    registry.add_function(AnkhaOption::script_is_box__define_function(registry));
    registry.add_function(AnkhaOption::script_borrow__define_function(registry));
    registry.add_function(AnkhaOption::script_borrow_mut__define_function(registry));
    registry.add_function(AnkhaOption::script_lazy__define_function(registry));
    registry.add_function(AnkhaOption::script_duplicate_box__define_function(registry));
    registry.add_function(AnkhaOption::script_take__define_function(registry));
    registry.add_function(AnkhaOption::script_take_ref__define_function(registry));
    registry.add_function(AnkhaOption::script_take_ref_mut__define_function(registry));
    registry.add_function(AnkhaOption::script_take_lazy__define_function(registry));
    registry.add_function(AnkhaOption::script_take_box__define_function(registry));
    registry.add_type(AnkhaAsyncOption::define_enum(registry));
    registry.add_function(AnkhaAsyncOption::script_wrap__define_function(registry));
    registry.add_function(AnkhaAsyncOption::script_unwrap__define_function(registry));
    registry.add_function(AnkhaAsyncOption::script_none__define_function(registry));
    registry.add_function(AnkhaAsyncOption::script_some__define_function(registry));
    registry.add_function(AnkhaAsyncOption::script_some_ref__define_function(registry));
    registry.add_function(AnkhaAsyncOption::script_some_ref_mut__define_function(
        registry,
    ));
    registry.add_function(AnkhaAsyncOption::script_some_lazy__define_function(
        registry,
    ));
    registry.add_function(AnkhaAsyncOption::script_is_none__define_function(registry));
    registry.add_function(AnkhaAsyncOption::script_is_some__define_function(registry));
    registry.add_function(AnkhaAsyncOption::script_is_owned__define_function(registry));
    registry.add_function(AnkhaAsyncOption::script_is_ref__define_function(registry));
    registry.add_function(AnkhaAsyncOption::script_is_ref_mut__define_function(
        registry,
    ));
    registry.add_function(AnkhaAsyncOption::script_is_lazy__define_function(registry));
    registry.add_function(AnkhaAsyncOption::script_borrow__define_function(registry));
    registry.add_function(AnkhaAsyncOption::script_borrow_mut__define_function(
        registry,
    ));
    registry.add_function(AnkhaAsyncOption::script_lazy__define_function(registry));
    registry.add_function(AnkhaAsyncOption::script_take__define_function(registry));
    registry.add_function(AnkhaAsyncOption::script_take_ref__define_function(registry));
    registry.add_function(AnkhaAsyncOption::script_take_ref_mut__define_function(
        registry,
    ));
    registry.add_function(AnkhaAsyncOption::script_take_lazy__define_function(
        registry,
    ));
}

#[derive(IntuicioEnum, Default)]
#[intuicio(name = "Option", module_name = "option")]
#[repr(u8)]
pub enum AnkhaOption {
    #[default]
    #[intuicio(ignore)]
    None,
    #[intuicio(ignore)]
    Owned(DynamicManaged),
    #[intuicio(ignore)]
    Ref(DynamicManagedRef),
    #[intuicio(ignore)]
    RefMut(DynamicManagedRefMut),
    #[intuicio(ignore)]
    Lazy(DynamicManagedLazy),
    #[intuicio(ignore)]
    Box(DynamicManagedBox),
}

impl AnkhaOption {
    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }

    pub fn is_some(&self) -> bool {
        !self.is_none()
    }

    pub fn is_owned(&self) -> bool {
        matches!(self, Self::Owned(_))
    }

    pub fn is_ref(&self) -> bool {
        matches!(self, Self::Ref(_))
    }

    pub fn is_ref_mut(&self) -> bool {
        matches!(self, Self::RefMut(_))
    }

    pub fn is_lazy(&self) -> bool {
        matches!(self, Self::Lazy(_))
    }

    pub fn is_box(&self) -> bool {
        matches!(self, Self::Box(_))
    }

    pub fn type_hash(&self) -> Option<TypeHash> {
        match self {
            Self::None => None,
            Self::Owned(value) => Some(*value.type_hash()),
            Self::Ref(value) => Some(*value.type_hash()),
            Self::RefMut(value) => Some(*value.type_hash()),
            Self::Lazy(value) => Some(*value.type_hash()),
            Self::Box(value) => value.type_hash(),
        }
    }

    pub fn borrow(&self) -> Option<DynamicManagedRef> {
        match self {
            Self::None => None,
            Self::Owned(value) => value.borrow(),
            Self::Ref(value) => value.borrow(),
            Self::RefMut(value) => value.borrow(),
            Self::Lazy(value) => value.borrow(),
            Self::Box(value) => value.borrow(),
        }
    }

    pub fn borrow_mut(&mut self) -> Option<DynamicManagedRefMut> {
        match self {
            Self::None => None,
            Self::Owned(value) => value.borrow_mut(),
            Self::Ref(_) => None,
            Self::RefMut(value) => value.borrow_mut(),
            Self::Lazy(value) => value.borrow_mut(),
            Self::Box(value) => value.borrow_mut(),
        }
    }

    pub fn lazy(&mut self) -> Option<DynamicManagedLazy> {
        match self {
            Self::None => None,
            Self::Owned(value) => Some(value.lazy()),
            Self::Ref(_) => None,
            Self::RefMut(_) => None,
            Self::Lazy(value) => Some(value.clone()),
            Self::Box(value) => value.lazy(),
        }
    }

    pub fn duplicate_box(&self) -> Option<DynamicManagedBox> {
        match self {
            Self::None => None,
            Self::Owned(_) => None,
            Self::Ref(_) => None,
            Self::RefMut(_) => None,
            Self::Lazy(_) => None,
            Self::Box(value) => Some(value.clone()),
        }
    }

    pub fn take(&mut self) -> Option<DynamicManaged> {
        match std::mem::take(self) {
            Self::Owned(value) => Some(value),
            value => {
                *self = value;
                None
            }
        }
    }

    pub fn take_ref(&mut self) -> Option<DynamicManagedRef> {
        match std::mem::take(self) {
            Self::Ref(value) => Some(value),
            value => {
                *self = value;
                None
            }
        }
    }

    pub fn take_ref_mut(&mut self) -> Option<DynamicManagedRefMut> {
        match std::mem::take(self) {
            Self::RefMut(value) => Some(value),
            value => {
                *self = value;
                None
            }
        }
    }

    pub fn take_lazy(&mut self) -> Option<DynamicManagedLazy> {
        match std::mem::take(self) {
            Self::Lazy(value) => Some(value),
            value => {
                *self = value;
                None
            }
        }
    }

    pub fn take_box(&mut self) -> Option<DynamicManagedBox> {
        match std::mem::take(self) {
            Self::Box(value) => Some(value),
            value => {
                *self = value;
                None
            }
        }
    }

    pub fn from_literal(literal: AnkhaLiteral) -> Self {
        match literal {
            AnkhaLiteral::Unit => Self::Owned(DynamicManaged::new(()).ok().unwrap()),
            AnkhaLiteral::Bool(value) => Self::Owned(DynamicManaged::new(value).ok().unwrap()),
            AnkhaLiteral::I8(value) => Self::Owned(DynamicManaged::new(value).ok().unwrap()),
            AnkhaLiteral::I16(value) => Self::Owned(DynamicManaged::new(value).ok().unwrap()),
            AnkhaLiteral::I32(value) => Self::Owned(DynamicManaged::new(value).ok().unwrap()),
            AnkhaLiteral::I64(value) => Self::Owned(DynamicManaged::new(value).ok().unwrap()),
            AnkhaLiteral::I128(value) => Self::Owned(DynamicManaged::new(value).ok().unwrap()),
            AnkhaLiteral::Isize(value) => Self::Owned(DynamicManaged::new(value).ok().unwrap()),
            AnkhaLiteral::U8(value) => Self::Owned(DynamicManaged::new(value).ok().unwrap()),
            AnkhaLiteral::U16(value) => Self::Owned(DynamicManaged::new(value).ok().unwrap()),
            AnkhaLiteral::U32(value) => Self::Owned(DynamicManaged::new(value).ok().unwrap()),
            AnkhaLiteral::U64(value) => Self::Owned(DynamicManaged::new(value).ok().unwrap()),
            AnkhaLiteral::U128(value) => Self::Owned(DynamicManaged::new(value).ok().unwrap()),
            AnkhaLiteral::Usize(value) => Self::Owned(DynamicManaged::new(value).ok().unwrap()),
            AnkhaLiteral::F32(value) => Self::Owned(DynamicManaged::new(value).ok().unwrap()),
            AnkhaLiteral::F64(value) => Self::Owned(DynamicManaged::new(value).ok().unwrap()),
            AnkhaLiteral::Char(value) => Self::Owned(DynamicManaged::new(value).ok().unwrap()),
            AnkhaLiteral::String(value) => Self::Owned(DynamicManaged::new(value).ok().unwrap()),
        }
    }

    pub fn into_literal(self) -> Option<AnkhaLiteral> {
        if let Self::Owned(value) = self {
            let type_hash = *value.type_hash();
            if type_hash == TypeHash::of::<()>() {
                Some(AnkhaLiteral::Unit)
            } else if type_hash == TypeHash::of::<bool>() {
                Some(AnkhaLiteral::Bool(value.consume::<bool>().ok().unwrap()))
            } else if type_hash == TypeHash::of::<i8>() {
                Some(AnkhaLiteral::I8(value.consume::<i8>().ok().unwrap()))
            } else if type_hash == TypeHash::of::<i16>() {
                Some(AnkhaLiteral::I16(value.consume::<i16>().ok().unwrap()))
            } else if type_hash == TypeHash::of::<i32>() {
                Some(AnkhaLiteral::I32(value.consume::<i32>().ok().unwrap()))
            } else if type_hash == TypeHash::of::<i64>() {
                Some(AnkhaLiteral::I64(value.consume::<i64>().ok().unwrap()))
            } else if type_hash == TypeHash::of::<i128>() {
                Some(AnkhaLiteral::I128(value.consume::<i128>().ok().unwrap()))
            } else if type_hash == TypeHash::of::<isize>() {
                Some(AnkhaLiteral::Isize(value.consume::<isize>().ok().unwrap()))
            } else if type_hash == TypeHash::of::<u8>() {
                Some(AnkhaLiteral::U8(value.consume::<u8>().ok().unwrap()))
            } else if type_hash == TypeHash::of::<u16>() {
                Some(AnkhaLiteral::U16(value.consume::<u16>().ok().unwrap()))
            } else if type_hash == TypeHash::of::<u32>() {
                Some(AnkhaLiteral::U32(value.consume::<u32>().ok().unwrap()))
            } else if type_hash == TypeHash::of::<u64>() {
                Some(AnkhaLiteral::U64(value.consume::<u64>().ok().unwrap()))
            } else if type_hash == TypeHash::of::<u128>() {
                Some(AnkhaLiteral::U128(value.consume::<u128>().ok().unwrap()))
            } else if type_hash == TypeHash::of::<usize>() {
                Some(AnkhaLiteral::Usize(value.consume::<usize>().ok().unwrap()))
            } else if type_hash == TypeHash::of::<f32>() {
                Some(AnkhaLiteral::F32(value.consume::<f32>().ok().unwrap()))
            } else if type_hash == TypeHash::of::<f64>() {
                Some(AnkhaLiteral::F64(value.consume::<f64>().ok().unwrap()))
            } else if type_hash == TypeHash::of::<char>() {
                Some(AnkhaLiteral::Char(value.consume::<char>().ok().unwrap()))
            } else if type_hash == TypeHash::of::<String>() {
                Some(AnkhaLiteral::String(
                    value.consume::<String>().ok().unwrap(),
                ))
            } else {
                None
            }
        } else {
            None
        }
    }
}

#[intuicio_methods(module_name = "option")]
impl AnkhaOption {
    #[intuicio_method(name = "wrap", use_context)]
    fn script_wrap(context: &mut Context) {
        stack_managed_variant(
            context,
            |context, value| context.stack().push(Self::script_some(value)),
            |context, value| context.stack().push(Self::script_some_ref(value)),
            |context, value| context.stack().push(Self::script_some_ref_mut(value)),
            |context, value| context.stack().push(Self::script_some_lazy(value)),
            |context, value| context.stack().push(Self::script_some_box(value)),
        );
    }

    #[intuicio_method(name = "unwrap", use_context)]
    fn script_unwrap(self, context: &mut Context) {
        match self {
            Self::None => panic!("Cannot unwrap None!"),
            Self::Owned(value) => context.stack().push(value),
            Self::Ref(value) => context.stack().push(value),
            Self::RefMut(value) => context.stack().push(value),
            Self::Lazy(value) => context.stack().push(value),
            Self::Box(value) => context.stack().push(value),
        };
    }

    #[intuicio_method(name = "none")]
    fn script_none() -> DynamicManaged {
        DynamicManaged::new(Self::None).ok().unwrap()
    }

    #[intuicio_method(name = "some")]
    fn script_some(value: DynamicManaged) -> DynamicManaged {
        DynamicManaged::new(Self::Owned(value)).ok().unwrap()
    }

    #[intuicio_method(name = "some_ref")]
    fn script_some_ref(value: DynamicManagedRef) -> DynamicManaged {
        DynamicManaged::new(Self::Ref(value)).ok().unwrap()
    }

    #[intuicio_method(name = "some_ref_mut")]
    fn script_some_ref_mut(value: DynamicManagedRefMut) -> DynamicManaged {
        DynamicManaged::new(Self::RefMut(value)).ok().unwrap()
    }

    #[intuicio_method(name = "some_lazy")]
    fn script_some_lazy(value: DynamicManagedLazy) -> DynamicManaged {
        DynamicManaged::new(Self::Lazy(value)).ok().unwrap()
    }

    #[intuicio_method(name = "some_box")]
    fn script_some_box(value: DynamicManagedBox) -> DynamicManaged {
        DynamicManaged::new(Self::Box(value)).ok().unwrap()
    }

    #[intuicio_method(name = "is_none", transformer = "DynamicManagedValueTransformer")]
    fn script_is_none(&self) -> bool {
        self.is_none()
    }

    #[intuicio_method(name = "is_some", transformer = "DynamicManagedValueTransformer")]
    fn script_is_some(&self) -> bool {
        self.is_some()
    }

    #[intuicio_method(name = "is_owned", transformer = "DynamicManagedValueTransformer")]
    fn script_is_owned(&self) -> bool {
        self.is_owned()
    }

    #[intuicio_method(name = "is_ref", transformer = "DynamicManagedValueTransformer")]
    fn script_is_ref(&self) -> bool {
        self.is_ref()
    }

    #[intuicio_method(name = "is_ref_mut", transformer = "DynamicManagedValueTransformer")]
    fn script_is_ref_mut(&self) -> bool {
        self.is_ref_mut()
    }

    #[intuicio_method(name = "is_lazy", transformer = "DynamicManagedValueTransformer")]
    fn script_is_lazy(&self) -> bool {
        self.is_lazy()
    }

    #[intuicio_method(name = "is_box", transformer = "DynamicManagedValueTransformer")]
    fn script_is_box(&self) -> bool {
        self.is_box()
    }

    #[intuicio_method(name = "borrow")]
    fn script_borrow(this: DynamicManagedRef) -> DynamicManagedRef {
        this.read::<Self>().unwrap().borrow().unwrap()
    }

    #[intuicio_method(name = "borrow_mut")]
    fn script_borrow_mut(mut this: DynamicManagedRefMut) -> DynamicManagedRefMut {
        this.write::<Self>().unwrap().borrow_mut().unwrap()
    }

    #[intuicio_method(name = "lazy")]
    fn script_lazy(mut this: DynamicManagedRefMut) -> DynamicManagedLazy {
        this.write::<Self>().unwrap().lazy().unwrap()
    }

    #[intuicio_method(name = "duplicate_box")]
    fn script_duplicate_box(this: DynamicManagedRef) -> DynamicManagedBox {
        this.read::<Self>().unwrap().duplicate_box().unwrap()
    }

    #[intuicio_method(name = "take")]
    fn script_take(mut this: DynamicManagedRefMut) -> DynamicManaged {
        this.write::<Self>().unwrap().take().unwrap()
    }

    #[intuicio_method(name = "take_ref")]
    fn script_take_ref(mut this: DynamicManagedRefMut) -> DynamicManagedRef {
        this.write::<Self>().unwrap().take_ref().unwrap()
    }

    #[intuicio_method(name = "take_ref_mut")]
    fn script_take_ref_mut(mut this: DynamicManagedRefMut) -> DynamicManagedRefMut {
        this.write::<Self>().unwrap().take_ref_mut().unwrap()
    }

    #[intuicio_method(name = "take_lazy")]
    fn script_take_lazy(mut this: DynamicManagedRefMut) -> DynamicManagedLazy {
        this.write::<Self>().unwrap().take_lazy().unwrap()
    }

    #[intuicio_method(name = "take_box")]
    fn script_take_box(mut this: DynamicManagedRefMut) -> DynamicManagedBox {
        this.write::<Self>().unwrap().take_box().unwrap()
    }
}

impl From<DynamicManaged> for AnkhaOption {
    fn from(value: DynamicManaged) -> Self {
        Self::Owned(value)
    }
}

impl From<DynamicManagedRef> for AnkhaOption {
    fn from(value: DynamicManagedRef) -> Self {
        Self::Ref(value)
    }
}

impl From<DynamicManagedRefMut> for AnkhaOption {
    fn from(value: DynamicManagedRefMut) -> Self {
        Self::RefMut(value)
    }
}

impl From<DynamicManagedLazy> for AnkhaOption {
    fn from(value: DynamicManagedLazy) -> Self {
        Self::Lazy(value)
    }
}

impl From<DynamicManagedBox> for AnkhaOption {
    fn from(value: DynamicManagedBox) -> Self {
        Self::Box(value)
    }
}

#[derive(IntuicioEnum, Default)]
#[intuicio(name = "AsyncOption", module_name = "option")]
#[repr(u8)]
pub enum AnkhaAsyncOption {
    #[default]
    #[intuicio(ignore)]
    None,
    #[intuicio(ignore)]
    Owned(DynamicManaged),
    #[intuicio(ignore)]
    Ref(DynamicManagedRef),
    #[intuicio(ignore)]
    RefMut(DynamicManagedRefMut),
    #[intuicio(ignore)]
    Lazy(DynamicManagedLazy),
}

impl AnkhaAsyncOption {
    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }

    pub fn is_some(&self) -> bool {
        !self.is_none()
    }

    pub fn is_owned(&self) -> bool {
        matches!(self, Self::Owned(_))
    }

    pub fn is_ref(&self) -> bool {
        matches!(self, Self::Ref(_))
    }

    pub fn is_ref_mut(&self) -> bool {
        matches!(self, Self::RefMut(_))
    }

    pub fn is_lazy(&self) -> bool {
        matches!(self, Self::Lazy(_))
    }

    pub fn type_hash(&self) -> Option<TypeHash> {
        match self {
            Self::None => None,
            Self::Owned(value) => Some(*value.type_hash()),
            Self::Ref(value) => Some(*value.type_hash()),
            Self::RefMut(value) => Some(*value.type_hash()),
            Self::Lazy(value) => Some(*value.type_hash()),
        }
    }

    pub fn borrow(&self) -> Option<DynamicManagedRef> {
        match self {
            Self::None => None,
            Self::Owned(value) => value.borrow(),
            Self::Ref(value) => value.borrow(),
            Self::RefMut(value) => value.borrow(),
            Self::Lazy(value) => value.borrow(),
        }
    }

    pub fn borrow_mut(&mut self) -> Option<DynamicManagedRefMut> {
        match self {
            Self::None => None,
            Self::Owned(value) => value.borrow_mut(),
            Self::Ref(_) => None,
            Self::RefMut(value) => value.borrow_mut(),
            Self::Lazy(value) => value.borrow_mut(),
        }
    }

    pub fn lazy(&mut self) -> Option<DynamicManagedLazy> {
        match self {
            Self::None => None,
            Self::Owned(value) => Some(value.lazy()),
            Self::Ref(_) => None,
            Self::RefMut(_) => None,
            Self::Lazy(value) => Some(value.clone()),
        }
    }

    pub fn take(&mut self) -> Option<DynamicManaged> {
        match std::mem::take(self) {
            Self::Owned(value) => Some(value),
            value => {
                *self = value;
                None
            }
        }
    }

    pub fn take_ref(&mut self) -> Option<DynamicManagedRef> {
        match std::mem::take(self) {
            Self::Ref(value) => Some(value),
            value => {
                *self = value;
                None
            }
        }
    }

    pub fn take_ref_mut(&mut self) -> Option<DynamicManagedRefMut> {
        match std::mem::take(self) {
            Self::RefMut(value) => Some(value),
            value => {
                *self = value;
                None
            }
        }
    }

    pub fn take_lazy(&mut self) -> Option<DynamicManagedLazy> {
        match std::mem::take(self) {
            Self::Lazy(value) => Some(value),
            value => {
                *self = value;
                None
            }
        }
    }

    pub fn from_literal(literal: AnkhaLiteral) -> Self {
        match literal {
            AnkhaLiteral::Unit => Self::Owned(DynamicManaged::new(()).ok().unwrap()),
            AnkhaLiteral::Bool(value) => Self::Owned(DynamicManaged::new(value).ok().unwrap()),
            AnkhaLiteral::I8(value) => Self::Owned(DynamicManaged::new(value).ok().unwrap()),
            AnkhaLiteral::I16(value) => Self::Owned(DynamicManaged::new(value).ok().unwrap()),
            AnkhaLiteral::I32(value) => Self::Owned(DynamicManaged::new(value).ok().unwrap()),
            AnkhaLiteral::I64(value) => Self::Owned(DynamicManaged::new(value).ok().unwrap()),
            AnkhaLiteral::I128(value) => Self::Owned(DynamicManaged::new(value).ok().unwrap()),
            AnkhaLiteral::Isize(value) => Self::Owned(DynamicManaged::new(value).ok().unwrap()),
            AnkhaLiteral::U8(value) => Self::Owned(DynamicManaged::new(value).ok().unwrap()),
            AnkhaLiteral::U16(value) => Self::Owned(DynamicManaged::new(value).ok().unwrap()),
            AnkhaLiteral::U32(value) => Self::Owned(DynamicManaged::new(value).ok().unwrap()),
            AnkhaLiteral::U64(value) => Self::Owned(DynamicManaged::new(value).ok().unwrap()),
            AnkhaLiteral::U128(value) => Self::Owned(DynamicManaged::new(value).ok().unwrap()),
            AnkhaLiteral::Usize(value) => Self::Owned(DynamicManaged::new(value).ok().unwrap()),
            AnkhaLiteral::F32(value) => Self::Owned(DynamicManaged::new(value).ok().unwrap()),
            AnkhaLiteral::F64(value) => Self::Owned(DynamicManaged::new(value).ok().unwrap()),
            AnkhaLiteral::Char(value) => Self::Owned(DynamicManaged::new(value).ok().unwrap()),
            AnkhaLiteral::String(value) => Self::Owned(DynamicManaged::new(value).ok().unwrap()),
        }
    }

    pub fn into_literal(self) -> Option<AnkhaLiteral> {
        if let Self::Owned(value) = self {
            let type_hash = *value.type_hash();
            if type_hash == TypeHash::of::<()>() {
                Some(AnkhaLiteral::Unit)
            } else if type_hash == TypeHash::of::<bool>() {
                Some(AnkhaLiteral::Bool(value.consume::<bool>().ok().unwrap()))
            } else if type_hash == TypeHash::of::<i8>() {
                Some(AnkhaLiteral::I8(value.consume::<i8>().ok().unwrap()))
            } else if type_hash == TypeHash::of::<i16>() {
                Some(AnkhaLiteral::I16(value.consume::<i16>().ok().unwrap()))
            } else if type_hash == TypeHash::of::<i32>() {
                Some(AnkhaLiteral::I32(value.consume::<i32>().ok().unwrap()))
            } else if type_hash == TypeHash::of::<i64>() {
                Some(AnkhaLiteral::I64(value.consume::<i64>().ok().unwrap()))
            } else if type_hash == TypeHash::of::<i128>() {
                Some(AnkhaLiteral::I128(value.consume::<i128>().ok().unwrap()))
            } else if type_hash == TypeHash::of::<isize>() {
                Some(AnkhaLiteral::Isize(value.consume::<isize>().ok().unwrap()))
            } else if type_hash == TypeHash::of::<u8>() {
                Some(AnkhaLiteral::U8(value.consume::<u8>().ok().unwrap()))
            } else if type_hash == TypeHash::of::<u16>() {
                Some(AnkhaLiteral::U16(value.consume::<u16>().ok().unwrap()))
            } else if type_hash == TypeHash::of::<u32>() {
                Some(AnkhaLiteral::U32(value.consume::<u32>().ok().unwrap()))
            } else if type_hash == TypeHash::of::<u64>() {
                Some(AnkhaLiteral::U64(value.consume::<u64>().ok().unwrap()))
            } else if type_hash == TypeHash::of::<u128>() {
                Some(AnkhaLiteral::U128(value.consume::<u128>().ok().unwrap()))
            } else if type_hash == TypeHash::of::<usize>() {
                Some(AnkhaLiteral::Usize(value.consume::<usize>().ok().unwrap()))
            } else if type_hash == TypeHash::of::<f32>() {
                Some(AnkhaLiteral::F32(value.consume::<f32>().ok().unwrap()))
            } else if type_hash == TypeHash::of::<f64>() {
                Some(AnkhaLiteral::F64(value.consume::<f64>().ok().unwrap()))
            } else if type_hash == TypeHash::of::<char>() {
                Some(AnkhaLiteral::Char(value.consume::<char>().ok().unwrap()))
            } else if type_hash == TypeHash::of::<String>() {
                Some(AnkhaLiteral::String(
                    value.consume::<String>().ok().unwrap(),
                ))
            } else {
                None
            }
        } else {
            None
        }
    }
}

#[intuicio_methods(module_name = "option")]
impl AnkhaAsyncOption {
    #[intuicio_method(name = "wrap", use_context)]
    fn script_wrap(context: &mut Context) {
        stack_managed_variant(
            context,
            |context, value| context.stack().push(Self::script_some(value)),
            |context, value| context.stack().push(Self::script_some_ref(value)),
            |context, value| context.stack().push(Self::script_some_ref_mut(value)),
            |context, value| context.stack().push(Self::script_some_lazy(value)),
            |_, _| panic!("Async option cannot be wrapped from box value!"),
        );
    }

    #[intuicio_method(name = "unwrap", use_context)]
    fn script_unwrap(self, context: &mut Context) {
        match self {
            Self::None => panic!("Cannot unwrap None!"),
            Self::Owned(value) => context.stack().push(value),
            Self::Ref(value) => context.stack().push(value),
            Self::RefMut(value) => context.stack().push(value),
            Self::Lazy(value) => context.stack().push(value),
        };
    }

    #[intuicio_method(name = "none")]
    fn script_none() -> DynamicManaged {
        DynamicManaged::new(Self::None).ok().unwrap()
    }

    #[intuicio_method(name = "some")]
    fn script_some(value: DynamicManaged) -> DynamicManaged {
        DynamicManaged::new(Self::Owned(value)).ok().unwrap()
    }

    #[intuicio_method(name = "some_ref")]
    fn script_some_ref(value: DynamicManagedRef) -> DynamicManaged {
        DynamicManaged::new(Self::Ref(value)).ok().unwrap()
    }

    #[intuicio_method(name = "some_ref_mut")]
    fn script_some_ref_mut(value: DynamicManagedRefMut) -> DynamicManaged {
        DynamicManaged::new(Self::RefMut(value)).ok().unwrap()
    }

    #[intuicio_method(name = "some_lazy")]
    fn script_some_lazy(value: DynamicManagedLazy) -> DynamicManaged {
        DynamicManaged::new(Self::Lazy(value)).ok().unwrap()
    }

    #[intuicio_method(name = "is_none", transformer = "DynamicManagedValueTransformer")]
    fn script_is_none(&self) -> bool {
        self.is_none()
    }

    #[intuicio_method(name = "is_some", transformer = "DynamicManagedValueTransformer")]
    fn script_is_some(&self) -> bool {
        self.is_some()
    }

    #[intuicio_method(name = "is_owned", transformer = "DynamicManagedValueTransformer")]
    fn script_is_owned(&self) -> bool {
        self.is_owned()
    }

    #[intuicio_method(name = "is_ref", transformer = "DynamicManagedValueTransformer")]
    fn script_is_ref(&self) -> bool {
        self.is_ref()
    }

    #[intuicio_method(name = "is_ref_mut", transformer = "DynamicManagedValueTransformer")]
    fn script_is_ref_mut(&self) -> bool {
        self.is_ref_mut()
    }

    #[intuicio_method(name = "is_lazy", transformer = "DynamicManagedValueTransformer")]
    fn script_is_lazy(&self) -> bool {
        self.is_lazy()
    }

    #[intuicio_method(name = "borrow")]
    fn script_borrow(this: DynamicManagedRef) -> DynamicManagedRef {
        this.read::<Self>().unwrap().borrow().unwrap()
    }

    #[intuicio_method(name = "borrow_mut")]
    fn script_borrow_mut(mut this: DynamicManagedRefMut) -> DynamicManagedRefMut {
        this.write::<Self>().unwrap().borrow_mut().unwrap()
    }

    #[intuicio_method(name = "lazy")]
    fn script_lazy(mut this: DynamicManagedRefMut) -> DynamicManagedLazy {
        this.write::<Self>().unwrap().lazy().unwrap()
    }

    #[intuicio_method(name = "take")]
    fn script_take(mut this: DynamicManagedRefMut) -> DynamicManaged {
        this.write::<Self>().unwrap().take().unwrap()
    }

    #[intuicio_method(name = "take_ref")]
    fn script_take_ref(mut this: DynamicManagedRefMut) -> DynamicManagedRef {
        this.write::<Self>().unwrap().take_ref().unwrap()
    }

    #[intuicio_method(name = "take_ref_mut")]
    fn script_take_ref_mut(mut this: DynamicManagedRefMut) -> DynamicManagedRefMut {
        this.write::<Self>().unwrap().take_ref_mut().unwrap()
    }

    #[intuicio_method(name = "take_lazy")]
    fn script_take_lazy(mut this: DynamicManagedRefMut) -> DynamicManagedLazy {
        this.write::<Self>().unwrap().take_lazy().unwrap()
    }
}

impl From<DynamicManaged> for AnkhaAsyncOption {
    fn from(value: DynamicManaged) -> Self {
        Self::Owned(value)
    }
}

impl From<DynamicManagedRef> for AnkhaAsyncOption {
    fn from(value: DynamicManagedRef) -> Self {
        Self::Ref(value)
    }
}

impl From<DynamicManagedRefMut> for AnkhaAsyncOption {
    fn from(value: DynamicManagedRefMut) -> Self {
        Self::RefMut(value)
    }
}

impl From<DynamicManagedLazy> for AnkhaAsyncOption {
    fn from(value: DynamicManagedLazy) -> Self {
        Self::Lazy(value)
    }
}
