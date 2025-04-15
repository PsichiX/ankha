use crate::script::*;
use intuicio_core::{meta::*, prelude::*};
use intuicio_parser::{
    ParserHandle, ParserNoValue, ParserOutput, ParserRegistry,
    generator::Generator,
    shorthand::{
        alt, ext_exchange, inject, lit, map, map_err, number_float, number_int, number_int_pos, oc,
        opt, prefix, regex, seq, seq_del, string, suffix, zom,
    },
};
use std::{error::Error, str::FromStr};

pub struct AnkhaContentParser(ParserRegistry);

impl Default for AnkhaContentParser {
    fn default() -> Self {
        let mut registry = ParserRegistry::default();
        install(&mut registry);
        Self(registry)
    }
}

impl AnkhaContentParser {
    pub fn with_dialect(mut self, generator: &Generator) -> Result<Self, Box<dyn Error>> {
        self.dialect(generator)?;
        Ok(self)
    }

    pub fn with_setup(mut self, f: impl FnOnce(&mut ParserRegistry)) -> Self {
        self.setup(f);
        self
    }

    pub fn dialect(&mut self, generator: &Generator) -> Result<(), Box<dyn Error>> {
        generator.install(&mut self.0)
    }

    pub fn setup(&mut self, f: impl FnOnce(&mut ParserRegistry)) {
        f(&mut self.0);
    }

    pub fn parse_file_content(&self, content: &str) -> Result<AnkhaFile, Box<dyn Error>> {
        let (_, result) = self.0.parse("ENTRY", content)?;
        Ok(result.consume::<AnkhaFile>().ok().unwrap())
    }
}

impl BytesContentParser<AnkhaFile> for AnkhaContentParser {
    fn parse(&self, bytes: Vec<u8>) -> Result<AnkhaFile, Box<dyn Error>> {
        self.parse_file_content(&String::from_utf8(bytes)?)
    }
}

pub fn install(registry: &mut ParserRegistry) {
    let file = file();
    registry.add_parser("ENTRY", ext_exchange(file.clone()));
    registry.add_parser("ankha/file", file.clone());
    registry.add_parser("ankha/use", dependency());
    registry.add_parser("ankha/mod", module());
    registry.add_parser("ankha/meta", meta());
    registry.add_parser("ankha/meta_inner", meta_inner());
    registry.add_parser("ankha/vis", visibility());
    registry.add_parser("ankha/kind_any", value_kind_any());
    registry.add_parser("ankha/kind", value_kind());
    registry.add_parser("ankha/field", field());
    registry.add_parser("ankha/struct", struct_type());
    registry.add_parser("ankha/var", variant());
    registry.add_parser("ankha/enum", enum_type());
    registry.add_parser("ankha/input", param("in", true));
    registry.add_parser("ankha/output", param("out", false));
    registry.add_parser("ankha/fn", function());
    registry.add_parser("ankha/operation", operation());
    registry.add_parser("ankha/body", body());
    registry
        .extend("ENTRY", file)
        .expect("Could not extend `ENTRY` rule");
}

fn comment() -> ParserHandle {
    map(
        regex(r"(\s*/\*[^\*/]+\*/\s*|\s*//[^\r\n]+[\r\n]\s*)+"),
        |_: String| ParserNoValue,
    )
}

fn ws() -> ParserHandle {
    alt([comment(), intuicio_parser::shorthand::ws()])
}

fn ows() -> ParserHandle {
    alt([comment(), intuicio_parser::shorthand::ows()])
}

fn sentence(inner: ParserHandle) -> ParserHandle {
    oc(inner, suffix(lit("("), ows()), prefix(lit(")"), ows()))
}

fn sentence_list(header: ParserHandle, repeat: ParserHandle) -> ParserHandle {
    oc(
        prefix(zom(prefix(repeat, ws())), header),
        suffix(lit("("), ows()),
        prefix(lit(")"), ows()),
    )
}

fn lit_bool() -> ParserHandle {
    map(alt([lit("true"), lit("false")]), |value: String| {
        value.parse::<bool>().unwrap()
    })
}

fn lit_integer<T>() -> ParserHandle
where
    T: FromStr + 'static,
    T::Err: std::fmt::Debug,
{
    map(number_int(), |value: String| value.parse::<T>().unwrap())
}

fn lit_unsigned_integer<T>() -> ParserHandle
where
    T: FromStr + 'static,
    T::Err: std::fmt::Debug,
{
    map(number_int_pos(), |value: String| {
        value.parse::<T>().unwrap()
    })
}

fn lit_float<T>() -> ParserHandle
where
    T: FromStr + 'static,
    T::Err: std::fmt::Debug,
{
    map(number_float(), |value: String| value.parse::<T>().unwrap())
}

fn lit_char() -> ParserHandle {
    map(string("'", "'"), |value: String| {
        value.parse::<char>().unwrap()
    })
}

fn lit_string() -> ParserHandle {
    string("\"", "\"")
}

fn file() -> ParserHandle {
    map_err(
        map(
            sentence_list(lit("file"), alt([inject("ankha/use"), inject("ankha/mod")])),
            |items: Vec<ParserOutput>| {
                let mut result = AnkhaFile::default();
                for item in items {
                    if item.is::<String>() {
                        let name = item.consume::<String>().ok().unwrap();
                        result.dependencies.push(name);
                    } else if item.is::<AnkhaModule>() {
                        let module = item.consume::<AnkhaModule>().ok().unwrap();
                        result.modules.push(module);
                    } else {
                        unreachable!();
                    }
                }
                result
            },
        ),
        |error| format!("Expected `ankha/file` | {}", error).into(),
    )
}

fn dependency() -> ParserHandle {
    map_err(
        sentence(prefix(lit_string(), suffix(lit("use"), ws()))),
        |error| format!("Expected `ankha/use` | {}", error).into(),
    )
}

fn module() -> ParserHandle {
    map_err(
        map(
            sentence_list(
                lit("mod"),
                alt([
                    inject("ankha/struct"),
                    inject("ankha/enum"),
                    inject("ankha/fn"),
                    lit_string(),
                ]),
            ),
            |values: Vec<ParserOutput>| {
                let mut name = None;
                let mut structs = vec![];
                let mut enums = vec![];
                let mut functions = vec![];
                for value in values {
                    if value.is::<String>() {
                        name = Some(value.consume::<String>().ok().unwrap());
                    } else if value.is::<AnkhaStruct>() {
                        structs.push(value.consume::<AnkhaStruct>().ok().unwrap());
                    } else if value.is::<AnkhaEnum>() {
                        enums.push(value.consume::<AnkhaEnum>().ok().unwrap());
                    } else if value.is::<AnkhaFunction>() {
                        functions.push(value.consume::<AnkhaFunction>().ok().unwrap());
                    } else {
                        unreachable!();
                    }
                }
                AnkhaModule {
                    name: name.expect("Missing module name"),
                    structs,
                    enums,
                    functions,
                }
            },
        ),
        |error| format!("Expected `ankha/mod` | {}", error).into(),
    )
}

fn meta() -> ParserHandle {
    map_err(
        sentence(prefix(meta_inner(), suffix(lit("meta"), ws()))),
        |error| format!("Expected `ankha/meta` | {}", error).into(),
    )
}

fn meta_inner() -> ParserHandle {
    map_err(
        alt([meta_id(), meta_value(), meta_array(), meta_map()]),
        |error| format!("Expected `ankha/meta` inner | {}", error).into(),
    )
}

fn meta_id() -> ParserHandle {
    map_err(
        map(
            sentence(seq_del(ws(), [lit("id"), lit_string()])),
            |mut values: Vec<ParserOutput>| {
                Meta::Identifier(values.remove(1).consume::<String>().ok().unwrap())
            },
        ),
        |error| format!("Expected `ankha/meta` id | {}", error).into(),
    )
}

fn meta_value() -> ParserHandle {
    map_err(
        map(
            sentence(seq_del(
                ws(),
                [
                    lit("value"),
                    alt([
                        map(prefix(lit_bool(), seq([lit("b"), ws()])), |value: bool| {
                            MetaValue::Bool(value)
                        }),
                        map(
                            prefix(lit_float::<f64>(), seq([lit("f"), ws()])),
                            |value: f64| MetaValue::Float(value),
                        ),
                        map(
                            prefix(lit_integer::<i64>(), seq([lit("i"), ws()])),
                            |value: i64| MetaValue::Integer(value),
                        ),
                        map(
                            prefix(lit_string(), seq([lit("s"), ws()])),
                            |value: String| MetaValue::String(value),
                        ),
                    ]),
                ],
            )),
            |mut values: Vec<ParserOutput>| {
                Meta::Value(values.remove(1).consume::<MetaValue>().ok().unwrap())
            },
        ),
        |error| format!("Expected `ankha/meta` value | {}", error).into(),
    )
}

fn meta_array() -> ParserHandle {
    map_err(
        map(
            sentence_list(lit("array"), inject("ankha/meta_inner")),
            |values: Vec<ParserOutput>| {
                Meta::Array(
                    values
                        .into_iter()
                        .map(|item: ParserOutput| item.consume::<Meta>().ok().unwrap())
                        .collect(),
                )
            },
        ),
        |error| format!("Expected `ankha/meta` array | {}", error).into(),
    )
}

fn meta_map() -> ParserHandle {
    map_err(
        map(
            sentence_list(
                lit("map"),
                sentence(seq_del(ws(), [lit_string(), inject("ankha/meta_inner")])),
            ),
            |values: Vec<ParserOutput>| {
                Meta::Map(
                    values
                        .into_iter()
                        .map(|items: ParserOutput| {
                            let mut items = items.consume::<Vec<ParserOutput>>().ok().unwrap();
                            let value = items.remove(1).consume::<Meta>().ok().unwrap();
                            let key = items.remove(0).consume::<String>().ok().unwrap();
                            (key, value)
                        })
                        .collect(),
                )
            },
        ),
        |error| format!("Expected `ankha/meta` map | {}", error).into(),
    )
}

fn visibility() -> ParserHandle {
    map_err(
        sentence(prefix(
            alt([
                map(lit("private"), |_: String| Visibility::Private),
                map(lit("module"), |_: String| Visibility::Module),
                map(lit("public"), |_: String| Visibility::Public),
            ]),
            suffix(lit("vis"), ws()),
        )),
        |error| format!("Expected `ankha/vis` | {}", error).into(),
    )
}

fn value_kind_any() -> ParserHandle {
    map_err(
        sentence(prefix(
            alt([
                map(lit("any"), |_: String| AnkhaValueKind::Any),
                map(lit("owned"), |_: String| AnkhaValueKind::Owned),
                map(lit("refmut"), |_: String| AnkhaValueKind::RefMut),
                map(lit("ref"), |_: String| AnkhaValueKind::Ref),
                map(lit("lazy"), |_: String| AnkhaValueKind::Lazy),
                map(lit("box"), |_: String| AnkhaValueKind::Box),
            ]),
            suffix(lit("kind"), ws()),
        )),
        |error| format!("Expected `ankha/kind_any` | {}", error).into(),
    )
}

fn value_kind() -> ParserHandle {
    map_err(
        sentence(prefix(
            alt([
                map(lit("owned"), |_: String| AnkhaValueKind::Owned),
                map(lit("refmut"), |_: String| AnkhaValueKind::RefMut),
                map(lit("ref"), |_: String| AnkhaValueKind::Ref),
                map(lit("lazy"), |_: String| AnkhaValueKind::Lazy),
                map(lit("box"), |_: String| AnkhaValueKind::Box),
            ]),
            suffix(lit("kind"), ws()),
        )),
        |error| format!("Expected `ankha/kind` | {}", error).into(),
    )
}

fn field() -> ParserHandle {
    map_err(
        map(
            sentence_list(
                lit("field"),
                alt([
                    lit_string(),
                    inject("ankha/meta"),
                    inject("ankha/vis"),
                    inject("ankha/kind"),
                ]),
            ),
            |values: Vec<ParserOutput>| {
                let mut meta = None;
                let mut name = None;
                let mut visibility = Visibility::Public;
                let mut kind = AnkhaValueKind::Any;
                for value in values {
                    if value.is::<String>() {
                        name = Some(value.consume::<String>().ok().unwrap());
                    } else if value.is::<Meta>() {
                        meta = Some(value.consume::<Meta>().ok().unwrap())
                    } else if value.is::<Visibility>() {
                        visibility = value.consume::<Visibility>().ok().unwrap();
                    } else if value.is::<AnkhaValueKind>() {
                        kind = value.consume::<AnkhaValueKind>().ok().unwrap();
                    } else {
                        unreachable!();
                    }
                }
                if kind == AnkhaValueKind::Any {
                    panic!("Missing value kind");
                }
                AnkhaStructField {
                    meta,
                    name: name.expect("Missing field name"),
                    visibility,
                    kind,
                }
            },
        ),
        |error| format!("Expected `ankha/field` | {}", error).into(),
    )
}

fn struct_type() -> ParserHandle {
    map_err(
        map(
            sentence_list(
                lit("struct"),
                alt([
                    lit_string(),
                    inject("ankha/meta"),
                    inject("ankha/vis"),
                    inject("ankha/field"),
                ]),
            ),
            |values: Vec<ParserOutput>| {
                let mut meta = None;
                let mut name = None;
                let mut visibility = Visibility::Public;
                let mut fields = vec![];
                for value in values {
                    if value.is::<String>() {
                        name = Some(value.consume::<String>().ok().unwrap());
                    } else if value.is::<Meta>() {
                        meta = Some(value.consume::<Meta>().ok().unwrap())
                    } else if value.is::<Visibility>() {
                        visibility = value.consume::<Visibility>().ok().unwrap();
                    } else if value.is::<AnkhaStructField>() {
                        fields.push(value.consume::<AnkhaStructField>().ok().unwrap());
                    } else {
                        unreachable!();
                    }
                }
                AnkhaStruct {
                    meta,
                    name: name.expect("Missing struct name"),
                    visibility,
                    fields,
                }
            },
        ),
        |error| format!("Expected `ankha/struct` | {}", error).into(),
    )
}

fn variant() -> ParserHandle {
    map_err(
        map(
            sentence_list(
                lit("var"),
                alt([lit_string(), inject("ankha/meta"), inject("ankha/field")]),
            ),
            |values: Vec<ParserOutput>| {
                let mut meta = None;
                let mut name = None;
                let mut fields = vec![];
                for value in values {
                    if value.is::<String>() {
                        name = Some(value.consume::<String>().ok().unwrap());
                    } else if value.is::<Meta>() {
                        meta = Some(value.consume::<Meta>().ok().unwrap())
                    } else if value.is::<AnkhaStructField>() {
                        fields.push(value.consume::<AnkhaStructField>().ok().unwrap());
                    } else {
                        unreachable!();
                    }
                }
                AnkhaEnumVariant {
                    meta,
                    name: name.expect("Missing enum variant name"),
                    fields,
                    discriminant: None,
                }
            },
        ),
        |error| format!("Expected `ankha/var` | {}", error).into(),
    )
}

fn enum_type() -> ParserHandle {
    map_err(
        map(
            sentence_list(
                lit("enum"),
                alt([
                    lit_string(),
                    inject("ankha/meta"),
                    inject("ankha/vis"),
                    inject("ankha/var"),
                ]),
            ),
            |values: Vec<ParserOutput>| {
                let mut meta = None;
                let mut name = None;
                let mut visibility = Visibility::Public;
                let mut variants = vec![];
                for value in values {
                    if value.is::<String>() {
                        name = Some(value.consume::<String>().ok().unwrap());
                    } else if value.is::<Meta>() {
                        meta = Some(value.consume::<Meta>().ok().unwrap())
                    } else if value.is::<Visibility>() {
                        visibility = value.consume::<Visibility>().ok().unwrap();
                    } else if value.is::<AnkhaEnumVariant>() {
                        variants.push(value.consume::<AnkhaEnumVariant>().ok().unwrap());
                    } else {
                        unreachable!();
                    }
                }
                AnkhaEnum {
                    meta,
                    name: name.expect("Missing enum name"),
                    visibility,
                    variants,
                    default_variant: None,
                }
            },
        ),
        |error| format!("Expected `ankha/enum` | {}", error).into(),
    )
}

fn type_name_module() -> ParserHandle {
    map_err(
        map(
            sentence(seq([
                lit("type"),
                prefix(lit_string(), ws()),
                opt(prefix(lit_string(), ws())),
            ])),
            |mut values: Vec<ParserOutput>| {
                let module = values.remove(2);
                let module = if module.is::<String>() {
                    Some(module.consume::<String>().ok().unwrap())
                } else {
                    None
                };
                let name = values.remove(1).consume::<String>().ok().unwrap();
                (name, module)
            },
        ),
        |error| format!("Expected type name-module | {}", error).into(),
    )
}

fn type_query() -> ParserHandle {
    map_err(
        map(
            type_name_module(),
            |(name, module): (String, Option<String>)| AnkhaTypeQuery {
                name: Some(name),
                module_name: module,
                ..Default::default()
            },
        ),
        |error| format!("Expected type query | {}", error).into(),
    )
}

fn param(header: &'static str, input: bool) -> ParserHandle {
    map_err(
        map(
            sentence_list(
                lit(header),
                alt([lit_string(), inject("ankha/meta"), inject("ankha/kind")]),
            ),
            move |values: Vec<ParserOutput>| {
                let mut meta = None;
                let mut name = None;
                let mut kind = AnkhaValueKind::Any;
                for value in values {
                    if value.is::<String>() {
                        name = Some(value.consume::<String>().ok().unwrap());
                    } else if value.is::<Meta>() {
                        meta = Some(value.consume::<Meta>().ok().unwrap())
                    } else if value.is::<AnkhaValueKind>() {
                        kind = value.consume::<AnkhaValueKind>().ok().unwrap();
                    } else {
                        unreachable!();
                    }
                }
                if kind == AnkhaValueKind::Any {
                    panic!("Missing value kind");
                }
                (
                    AnkhaFunctionParameter {
                        meta,
                        name: name.expect("Missing parameter name"),
                        kind,
                    },
                    input,
                )
            },
        ),
        move |error| {
            if input {
                format!("Expected `ankha/input` | {}", error).into()
            } else {
                format!("Expected `ankha/output` | {}", error).into()
            }
        },
    )
}

fn function() -> ParserHandle {
    map_err(
        map(
            sentence_list(
                lit("fn"),
                alt([
                    lit_string(),
                    type_name_module(),
                    inject("ankha/vis"),
                    inject("ankha/input"),
                    inject("ankha/output"),
                    inject("ankha/body"),
                ]),
            ),
            |values: Vec<ParserOutput>| {
                let mut meta = None;
                let mut name = None;
                let mut type_name_module = None;
                let mut visibility = Visibility::Public;
                let mut inputs = vec![];
                let mut outputs = vec![];
                let mut script = vec![];
                for value in values {
                    if value.is::<String>() {
                        name = Some(value.consume::<String>().ok().unwrap());
                    } else if value.is::<(String, Option<String>)>() {
                        type_name_module =
                            Some(value.consume::<(String, Option<String>)>().ok().unwrap())
                    } else if value.is::<Meta>() {
                        meta = Some(value.consume::<Meta>().ok().unwrap())
                    } else if value.is::<Visibility>() {
                        visibility = value.consume::<Visibility>().ok().unwrap();
                    } else if value.is::<(AnkhaFunctionParameter, bool)>() {
                        let (param, input) = value
                            .consume::<(AnkhaFunctionParameter, bool)>()
                            .ok()
                            .unwrap();
                        if input {
                            inputs.push(param);
                        } else {
                            outputs.push(param);
                        }
                    } else if value.is::<AnkhaScript>() {
                        script.extend(value.consume::<AnkhaScript>().ok().unwrap());
                    } else {
                        unreachable!();
                    }
                }
                AnkhaFunction {
                    meta,
                    name: name.expect("Missing function name"),
                    type_name_module,
                    visibility,
                    inputs,
                    outputs,
                    script,
                }
            },
        ),
        |error| format!("Expected `ankha/fn` | {}", error).into(),
    )
}

fn body() -> ParserHandle {
    map_err(
        map(
            sentence_list(lit("body"), inject("ankha/operation")),
            |values: Vec<ParserOutput>| {
                values
                    .into_iter()
                    .map(|value| value.consume::<AnkhaOperation>().ok().unwrap())
                    .collect::<AnkhaScript>()
            },
        ),
        |error| format!("Expected `ankha/body` | {}", error).into(),
    )
}

fn operation() -> ParserHandle {
    map_err(
        alt([
            map(expression(), |value: AnkhaExpression| {
                AnkhaOperation::Expression(value)
            }),
            group(),
            group_reversed(),
            make_register(),
            drop_register(),
            push_from_register(),
            pop_to_register(),
            call_function(),
            branch_scope(),
            loop_scope(),
            push_scope(),
            pop_scope(),
            ensure_register_type_op(),
            ensure_register_kind_op(),
        ]),
        |error| format!("Expected `ankha/operation` | {}", error).into(),
    )
}

fn expression() -> ParserHandle {
    alt([
        map(literal(), |value: AnkhaLiteral| {
            AnkhaExpression::Literal(value)
        }),
        stack_drop(),
        stack_unwrap_boolean(),
        borrow(),
        borrow_mut(),
        lazy(),
        borrow_field(),
        borrow_mut_field(),
        borrow_unmanaged_field(),
        borrow_mut_unmanaged_field(),
        copy_from(),
        move_into(),
        swap_in(),
        destructure(),
        structure(),
        box_(),
        manage(),
        unmanage(),
        copy(),
        swap(),
        duplicate_box(),
        ensure_stack_type(),
        ensure_register_type(),
        ensure_stack_kind(),
        ensure_register_kind(),
        call_method(),
        call_indirect(),
    ])
}

fn literal() -> ParserHandle {
    map_err(
        alt([
            map(literal_unit(), |_: ()| AnkhaLiteral::Unit),
            map(literal_value("bool", lit_bool()), |value: bool| {
                AnkhaLiteral::Bool(value)
            }),
            map(literal_value("i8", lit_integer::<i8>()), |value: i8| {
                AnkhaLiteral::I8(value)
            }),
            map(literal_value("i16", lit_integer::<i16>()), |value: i16| {
                AnkhaLiteral::I16(value)
            }),
            map(literal_value("i32", lit_integer::<i32>()), |value: i32| {
                AnkhaLiteral::I32(value)
            }),
            map(literal_value("i64", lit_integer::<i64>()), |value: i64| {
                AnkhaLiteral::I64(value)
            }),
            map(
                literal_value("i128", lit_integer::<i128>()),
                |value: i128| AnkhaLiteral::I128(value),
            ),
            map(
                literal_value("isize", lit_integer::<isize>()),
                |value: isize| AnkhaLiteral::Isize(value),
            ),
            map(
                literal_value("u8", lit_unsigned_integer::<u8>()),
                |value: u8| AnkhaLiteral::U8(value),
            ),
            map(
                literal_value("u16", lit_unsigned_integer::<u16>()),
                |value: u16| AnkhaLiteral::U16(value),
            ),
            map(
                literal_value("u32", lit_unsigned_integer::<u32>()),
                |value: u32| AnkhaLiteral::U32(value),
            ),
            map(
                literal_value("u64", lit_unsigned_integer::<u64>()),
                |value: u64| AnkhaLiteral::U64(value),
            ),
            map(
                literal_value("u128", lit_unsigned_integer::<u128>()),
                |value: u128| AnkhaLiteral::U128(value),
            ),
            map(
                literal_value("usize", lit_unsigned_integer::<usize>()),
                |value: usize| AnkhaLiteral::Usize(value),
            ),
            map(literal_value("f32", lit_float::<f32>()), |value: f32| {
                AnkhaLiteral::F32(value)
            }),
            map(literal_value("f64", lit_float::<f64>()), |value: f64| {
                AnkhaLiteral::F64(value)
            }),
            map(literal_value("char", lit_char()), |value: char| {
                AnkhaLiteral::Char(value)
            }),
            map(literal_value("string", lit_string()), |value: String| {
                AnkhaLiteral::String(value)
            }),
        ]),
        |error| format!("Expected literal | {}", error).into(),
    )
}

fn literal_unit() -> ParserHandle {
    map_err(
        map(
            sentence(seq_del(ws(), [lit("lit"), lit("unit")])),
            |_: Vec<ParserOutput>| (),
        ),
        |error| format!("Expected literal unit | {}", error).into(),
    )
}

fn literal_value(name: &'static str, value: ParserHandle) -> ParserHandle {
    map_err(
        sentence(prefix(value, seq([lit("lit"), ws(), lit(name), ws()]))),
        move |error| format!("Expected literal {} | {}", name, error).into(),
    )
}

fn stack_drop() -> ParserHandle {
    map_err(
        map(sentence(lit("stack_drop")), |_: String| {
            AnkhaExpression::StackDrop
        }),
        |error| format!("Expected stack drop | {}", error).into(),
    )
}

fn stack_unwrap_boolean() -> ParserHandle {
    map_err(
        map(sentence(lit("stack_unwrap_boolean")), |_: String| {
            AnkhaExpression::StackUnwrapBoolean
        }),
        |error| format!("Expected stack unwrap boolean | {}", error).into(),
    )
}

fn borrow() -> ParserHandle {
    map_err(
        map(sentence(lit("borrow")), |_: String| AnkhaExpression::Borrow),
        |error| format!("Expected borrow | {}", error).into(),
    )
}

fn borrow_mut() -> ParserHandle {
    map_err(
        map(sentence(lit("borrow_mut")), |_: String| {
            AnkhaExpression::BorrowMut
        }),
        |error| format!("Expected borrow mut | {}", error).into(),
    )
}

fn lazy() -> ParserHandle {
    map_err(
        map(sentence(lit("lazy")), |_: String| AnkhaExpression::Lazy),
        |error| format!("Expected lazy | {}", error).into(),
    )
}

fn borrow_field() -> ParserHandle {
    map_err(
        map(
            sentence_list(
                lit("borrow_field"),
                alt([lit_string(), inject("ankha/kind_any"), inject("ankha/vis")]),
            ),
            |values: Vec<ParserOutput>| {
                let mut name = None;
                let mut kind = AnkhaValueKind::Any;
                let mut visibility = None;
                for value in values {
                    if value.is::<String>() {
                        name = Some(value.consume::<String>().ok().unwrap());
                    } else if value.is::<AnkhaValueKind>() {
                        kind = value.consume::<AnkhaValueKind>().ok().unwrap();
                    } else if value.is::<Visibility>() {
                        visibility = Some(value.consume::<Visibility>().ok().unwrap());
                    } else {
                        unreachable!()
                    }
                }
                AnkhaExpression::BorrowField {
                    name: name.expect("Missing field name"),
                    kind,
                    visibility,
                }
            },
        ),
        |error| format!("Expected borrow field | {}", error).into(),
    )
}

fn borrow_mut_field() -> ParserHandle {
    map_err(
        map(
            sentence_list(
                lit("borrow_mut_field"),
                alt([lit_string(), inject("ankha/kind_any"), inject("ankha/vis")]),
            ),
            |values: Vec<ParserOutput>| {
                let mut name = None;
                let mut kind = AnkhaValueKind::Any;
                let mut visibility = None;
                for value in values {
                    if value.is::<String>() {
                        name = Some(value.consume::<String>().ok().unwrap());
                    } else if value.is::<AnkhaValueKind>() {
                        kind = value.consume::<AnkhaValueKind>().ok().unwrap();
                    } else if value.is::<Visibility>() {
                        visibility = Some(value.consume::<Visibility>().ok().unwrap());
                    } else {
                        unreachable!()
                    }
                }
                AnkhaExpression::BorrowMutField {
                    name: name.expect("Missing field name"),
                    kind,
                    visibility,
                }
            },
        ),
        |error| format!("Expected borrow mut field | {}", error).into(),
    )
}

fn borrow_unmanaged_field() -> ParserHandle {
    map_err(
        map(
            sentence_list(
                lit("borrow_unmanaged_field"),
                alt([lit_string(), type_query(), inject("ankha/vis")]),
            ),
            |values: Vec<ParserOutput>| {
                let mut name = None;
                let mut type_query = None;
                let mut visibility = None;
                for value in values {
                    if value.is::<String>() {
                        name = Some(value.consume::<String>().ok().unwrap());
                    } else if value.is::<AnkhaTypeQuery>() {
                        type_query = Some(value.consume::<AnkhaTypeQuery>().ok().unwrap());
                    } else if value.is::<Visibility>() {
                        visibility = Some(value.consume::<Visibility>().ok().unwrap());
                    } else {
                        unreachable!()
                    }
                }
                AnkhaExpression::BorrowUnmanagedField {
                    query: AnkhaFieldQuery {
                        name: name.expect("Missing field name"),
                        type_query,
                        visibility,
                    },
                }
            },
        ),
        |error| format!("Expected borrow field | {}", error).into(),
    )
}

fn borrow_mut_unmanaged_field() -> ParserHandle {
    map_err(
        map(
            sentence_list(
                lit("borrow_mut_unmanaged_field"),
                alt([lit_string(), type_query(), inject("ankha/vis")]),
            ),
            |values: Vec<ParserOutput>| {
                let mut name = None;
                let mut type_query = None;
                let mut visibility = None;
                for value in values {
                    if value.is::<String>() {
                        name = Some(value.consume::<String>().ok().unwrap());
                    } else if value.is::<AnkhaTypeQuery>() {
                        type_query = Some(value.consume::<AnkhaTypeQuery>().ok().unwrap());
                    } else if value.is::<Visibility>() {
                        visibility = Some(value.consume::<Visibility>().ok().unwrap());
                    } else {
                        unreachable!()
                    }
                }
                AnkhaExpression::BorrowMutUnmanagedField {
                    query: AnkhaFieldQuery {
                        name: name.expect("Missing field name"),
                        type_query,
                        visibility,
                    },
                }
            },
        ),
        |error| format!("Expected borrow field | {}", error).into(),
    )
}

fn copy_from() -> ParserHandle {
    map_err(
        map(sentence(lit("copy_from")), |_: String| {
            AnkhaExpression::CopyFrom
        }),
        |error| format!("Expected copy from | {}", error).into(),
    )
}

fn move_into() -> ParserHandle {
    map_err(
        map(sentence(lit("move_into")), |_: String| {
            AnkhaExpression::MoveInto
        }),
        |error| format!("Expected move into | {}", error).into(),
    )
}

fn swap_in() -> ParserHandle {
    map_err(
        map(sentence(lit("swap_in")), |_: String| {
            AnkhaExpression::SwapIn
        }),
        |error| format!("Expected swap in | {}", error).into(),
    )
}

fn destructure() -> ParserHandle {
    map_err(
        map(
            sentence_list(lit("destructure"), lit_string()),
            |values: Vec<ParserOutput>| AnkhaExpression::Destructure {
                fields: values
                    .into_iter()
                    .map(|value| value.consume::<String>().ok().unwrap())
                    .collect(),
            },
        ),
        |error| format!("Expected destructure | {}", error).into(),
    )
}

fn structure() -> ParserHandle {
    map_err(
        map(
            sentence_list(lit("structure"), alt([type_query(), lit_string()])),
            |values: Vec<ParserOutput>| {
                let mut type_query = None;
                let mut fields = vec![];
                for value in values {
                    if value.is::<AnkhaTypeQuery>() {
                        type_query = Some(value.consume::<AnkhaTypeQuery>().ok().unwrap());
                    } else if value.is::<String>() {
                        fields.push(value.consume::<String>().ok().unwrap());
                    } else {
                        unreachable!()
                    }
                }
                AnkhaExpression::Structure {
                    type_query: type_query.expect("Expected type query"),
                    fields,
                }
            },
        ),
        |error| format!("Expected destructure | {}", error).into(),
    )
}

fn box_() -> ParserHandle {
    map_err(
        map(sentence(lit("box")), |_: String| AnkhaExpression::Box),
        |error| format!("Expected box | {}", error).into(),
    )
}

fn manage() -> ParserHandle {
    map_err(
        map(sentence(lit("manage")), |_: String| AnkhaExpression::Manage),
        |error| format!("Expected manage | {}", error).into(),
    )
}

fn unmanage() -> ParserHandle {
    map_err(
        map(sentence(lit("unmanage")), |_: String| {
            AnkhaExpression::Unmanage
        }),
        |error| format!("Expected unmanage | {}", error).into(),
    )
}

fn copy() -> ParserHandle {
    map_err(
        map(sentence(lit("copy")), |_: String| AnkhaExpression::Copy),
        |error| format!("Expected copy | {}", error).into(),
    )
}

fn swap() -> ParserHandle {
    map_err(
        map(sentence(lit("swap")), |_: String| AnkhaExpression::Swap),
        |error| format!("Expected swap | {}", error).into(),
    )
}

fn duplicate_box() -> ParserHandle {
    map_err(
        map(sentence(lit("duplicate_box")), |_: String| {
            AnkhaExpression::DuplicateBox
        }),
        |error| format!("Expected duplicate box | {}", error).into(),
    )
}

fn ensure_stack_type() -> ParserHandle {
    map_err(
        map(
            sentence(seq_del(ws(), [lit("ensure_stack_type"), type_query()])),
            |mut values: Vec<ParserOutput>| {
                let type_query = values.remove(1).consume::<AnkhaTypeQuery>().ok().unwrap();
                AnkhaExpression::EnsureStackType { type_query }
            },
        ),
        |error| format!("Expected ensure stack type | {}", error).into(),
    )
}

fn ensure_register_type() -> ParserHandle {
    map_err(
        map(
            sentence_list(
                lit("ensure_register_type"),
                alt([type_query(), lit_unsigned_integer::<usize>()]),
            ),
            |values: Vec<ParserOutput>| {
                let mut type_query = None;
                let mut index = None;
                for value in values {
                    if value.is::<AnkhaTypeQuery>() {
                        type_query = Some(value.consume::<AnkhaTypeQuery>().ok().unwrap());
                    } else if value.is::<usize>() {
                        index = Some(value.consume::<usize>().ok().unwrap());
                    } else {
                        unreachable!()
                    }
                }
                AnkhaExpression::EnsureRegisterType {
                    type_query: type_query.expect("Expected type query"),
                    index: index.expect("Missing register index"),
                }
            },
        ),
        |error| format!("Expected ensure register type | {}", error).into(),
    )
}

fn ensure_stack_kind() -> ParserHandle {
    map_err(
        map(
            sentence(seq_del(
                ws(),
                [lit("ensure_stack_kind"), inject("ankha/kind")],
            )),
            |mut values: Vec<ParserOutput>| {
                let kind = values.remove(1).consume::<AnkhaValueKind>().ok().unwrap();
                AnkhaExpression::EnsureStackKind { kind }
            },
        ),
        |error| format!("Expected ensure stack kind | {}", error).into(),
    )
}

fn ensure_register_kind() -> ParserHandle {
    map_err(
        map(
            sentence_list(
                lit("ensure_register_kind"),
                alt([inject("ankha/kind"), lit_unsigned_integer::<usize>()]),
            ),
            |values: Vec<ParserOutput>| {
                let mut kind = None;
                let mut index = None;
                for value in values {
                    if value.is::<AnkhaValueKind>() {
                        kind = Some(value.consume::<AnkhaValueKind>().ok().unwrap());
                    } else if value.is::<usize>() {
                        index = Some(value.consume::<usize>().ok().unwrap());
                    } else {
                        unreachable!()
                    }
                }
                AnkhaExpression::EnsureRegisterKind {
                    kind: kind.expect("Expected value kind"),
                    index: index.expect("Missing register index"),
                }
            },
        ),
        |error| format!("Expected ensure register kind | {}", error).into(),
    )
}

fn param_query(header: &'static str, input: bool) -> ParserHandle {
    map_err(
        map(
            sentence_list(lit(header), alt([lit_string(), type_query()])),
            move |values: Vec<ParserOutput>| {
                let mut name = None;
                let mut type_query = None;
                for value in values {
                    if value.is::<String>() {
                        name = Some(value.consume::<String>().ok().unwrap());
                    } else if value.is::<AnkhaTypeQuery>() {
                        type_query = Some(value.consume::<AnkhaTypeQuery>().ok().unwrap())
                    } else {
                        unreachable!();
                    }
                }
                (AnkhaFunctionQueryParam { name, type_query }, input)
            },
        ),
        move |error| {
            if input {
                format!("Expected input query | {}", error).into()
            } else {
                format!("Expected output query | {}", error).into()
            }
        },
    )
}

fn function_query_name_module() -> ParserHandle {
    map_err(
        map(
            seq([lit_string(), opt(prefix(lit_string(), ws()))]),
            |mut values: Vec<ParserOutput>| {
                let module = values.remove(1).consume::<String>().ok();
                let name = values.remove(0).consume::<String>().ok().unwrap();
                (name, module)
            },
        ),
        |error| format!("Expected function query name-module | {}", error).into(),
    )
}

fn function_query() -> ParserHandle {
    map_err(
        map(
            sentence_list(
                lit("fn"),
                alt([
                    function_query_name_module(),
                    type_query(),
                    inject("ankha/vis"),
                    param_query("in", true),
                    param_query("out", false),
                ]),
            ),
            |values: Vec<ParserOutput>| {
                let mut name = None;
                let mut module_name = None;
                let mut type_query = None;
                let mut visibility = None;
                let mut inputs = vec![];
                let mut outputs = vec![];
                for value in values {
                    if value.is::<(String, Option<String>)>() {
                        let (n, m) = value.consume::<(String, Option<String>)>().ok().unwrap();
                        name = Some(n);
                        module_name = m;
                    } else if value.is::<AnkhaTypeQuery>() {
                        type_query = Some(value.consume::<AnkhaTypeQuery>().ok().unwrap());
                    } else if value.is::<Visibility>() {
                        visibility = Some(value.consume::<Visibility>().ok().unwrap());
                    } else if value.is::<(AnkhaFunctionQueryParam, bool)>() {
                        let (param, input) = value
                            .consume::<(AnkhaFunctionQueryParam, bool)>()
                            .ok()
                            .unwrap();
                        if input {
                            inputs.push(param);
                        } else {
                            outputs.push(param);
                        }
                    } else {
                        unreachable!()
                    }
                }
                AnkhaFunctionQuery {
                    name,
                    module_name,
                    type_query,
                    visibility,
                    inputs,
                    outputs,
                }
            },
        ),
        |error| format!("Expected function query | {}", error).into(),
    )
}

fn call_method() -> ParserHandle {
    map_err(
        map(
            sentence(prefix(function_query(), suffix(lit("call_method"), ws()))),
            |value: AnkhaFunctionQuery| AnkhaExpression::CallMethod {
                function_query: value,
            },
        ),
        |error| format!("Expected call method | {}", error).into(),
    )
}

fn call_indirect() -> ParserHandle {
    map_err(
        map(sentence(lit("call_indirect")), |_: String| {
            AnkhaExpression::CallIndirect
        }),
        |error| format!("Expected call indirect | {}", error).into(),
    )
}

fn group() -> ParserHandle {
    map_err(
        map(
            sentence_list(lit("group"), inject("ankha/operation")),
            |values: Vec<ParserOutput>| {
                AnkhaOperation::Group(
                    values
                        .into_iter()
                        .map(|value| value.consume::<AnkhaOperation>().ok().unwrap())
                        .collect(),
                )
            },
        ),
        |error| format!("Expected group | {}", error).into(),
    )
}

fn group_reversed() -> ParserHandle {
    map_err(
        map(
            sentence_list(lit("group_reversed"), inject("ankha/operation")),
            |values: Vec<ParserOutput>| {
                AnkhaOperation::GroupReversed(
                    values
                        .into_iter()
                        .map(|value| value.consume::<AnkhaOperation>().ok().unwrap())
                        .collect(),
                )
            },
        ),
        |error| format!("Expected group reversed | {}", error).into(),
    )
}

fn make_register() -> ParserHandle {
    map_err(
        map(
            sentence_list(
                lit("make_register"),
                alt([lit_string(), inject("ankha/kind")]),
            ),
            |values: Vec<ParserOutput>| {
                let mut kind = AnkhaValueKind::Any;
                let mut name = None;
                for value in values {
                    if value.is::<String>() {
                        name = Some(value.consume::<String>().ok().unwrap());
                    } else if value.is::<AnkhaValueKind>() {
                        kind = value.consume::<AnkhaValueKind>().ok().unwrap();
                    } else {
                        unreachable!();
                    }
                }
                if kind == AnkhaValueKind::Any {
                    panic!("Missing value kind");
                }
                AnkhaOperation::MakeRegister { kind, name }
            },
        ),
        |error| format!("Expected make register | {}", error).into(),
    )
}

fn drop_register() -> ParserHandle {
    map_err(
        map(
            sentence_list(
                lit("drop_register"),
                alt([lit_string(), lit_unsigned_integer::<usize>()]),
            ),
            |values: Vec<ParserOutput>| {
                let mut address = None;
                for value in values {
                    if value.is::<String>() {
                        address = Some(AnkhaRegisterAddress::Name(
                            value.consume::<String>().ok().unwrap(),
                        ));
                    } else if value.is::<usize>() {
                        address = Some(AnkhaRegisterAddress::Index(
                            value.consume::<usize>().ok().unwrap(),
                        ));
                    } else {
                        unreachable!();
                    }
                }
                AnkhaOperation::DropRegister(address.expect("Missing register address"))
            },
        ),
        |error| format!("Expected drop register | {}", error).into(),
    )
}

fn push_from_register() -> ParserHandle {
    map_err(
        map(
            sentence_list(
                lit("push_from_register"),
                alt([lit_string(), lit_unsigned_integer::<usize>()]),
            ),
            |values: Vec<ParserOutput>| {
                let mut address = None;
                for value in values {
                    if value.is::<String>() {
                        address = Some(AnkhaRegisterAddress::Name(
                            value.consume::<String>().ok().unwrap(),
                        ));
                    } else if value.is::<usize>() {
                        address = Some(AnkhaRegisterAddress::Index(
                            value.consume::<usize>().ok().unwrap(),
                        ));
                    } else {
                        unreachable!();
                    }
                }
                AnkhaOperation::PushFromRegister(address.expect("Missing register address"))
            },
        ),
        |error| format!("Expected push from register | {}", error).into(),
    )
}

fn pop_to_register() -> ParserHandle {
    map_err(
        map(
            sentence_list(
                lit("pop_to_register"),
                alt([lit_string(), lit_unsigned_integer::<usize>()]),
            ),
            |values: Vec<ParserOutput>| {
                let mut address = None;
                for value in values {
                    if value.is::<String>() {
                        address = Some(AnkhaRegisterAddress::Name(
                            value.consume::<String>().ok().unwrap(),
                        ));
                    } else if value.is::<usize>() {
                        address = Some(AnkhaRegisterAddress::Index(
                            value.consume::<usize>().ok().unwrap(),
                        ));
                    } else {
                        unreachable!();
                    }
                }
                AnkhaOperation::PopToRegister(address.expect("Missing register address"))
            },
        ),
        |error| format!("Expected pop to register | {}", error).into(),
    )
}

fn branch_scope_success() -> ParserHandle {
    map_err(
        map(
            sentence_list(lit("success"), inject("ankha/operation")),
            |values: Vec<ParserOutput>| {
                (
                    values
                        .into_iter()
                        .map(|value| value.consume::<AnkhaOperation>().ok().unwrap())
                        .collect::<AnkhaScript>(),
                    true,
                )
            },
        ),
        |error| format!("Expected branch scope success | {}", error).into(),
    )
}

fn branch_scope_failure() -> ParserHandle {
    map_err(
        map(
            sentence_list(lit("failure"), inject("ankha/operation")),
            |values: Vec<ParserOutput>| {
                (
                    values
                        .into_iter()
                        .map(|value| value.consume::<AnkhaOperation>().ok().unwrap())
                        .collect::<AnkhaScript>(),
                    false,
                )
            },
        ),
        |error| format!("Expected branch scope failure | {}", error).into(),
    )
}

fn call_function() -> ParserHandle {
    map_err(
        map(
            sentence(prefix(function_query(), suffix(lit("call_function"), ws()))),
            |value: AnkhaFunctionQuery| AnkhaOperation::CallFunction(value),
        ),
        |error| format!("Expected call function | {}", error).into(),
    )
}

fn branch_scope() -> ParserHandle {
    map_err(
        map(
            sentence_list(
                lit("branch"),
                alt([branch_scope_success(), branch_scope_failure()]),
            ),
            |values: Vec<ParserOutput>| {
                let mut success = vec![];
                let mut failure = vec![];
                for value in values {
                    if value.is::<(AnkhaScript, bool)>() {
                        let (script, s) = value.consume::<(AnkhaScript, bool)>().ok().unwrap();
                        if s {
                            success.extend(script);
                        } else {
                            failure.extend(script);
                        }
                    } else {
                        unreachable!();
                    }
                }
                AnkhaOperation::BranchScope {
                    script_success: success,
                    script_failure: if failure.is_empty() {
                        None
                    } else {
                        Some(failure)
                    },
                }
            },
        ),
        |error| format!("Expected branch scope | {}", error).into(),
    )
}

fn loop_scope() -> ParserHandle {
    map_err(
        map(
            sentence_list(lit("loop"), inject("ankha/operation")),
            |values: Vec<ParserOutput>| AnkhaOperation::LoopScope {
                script: values
                    .into_iter()
                    .map(|value| value.consume::<AnkhaOperation>().ok().unwrap())
                    .collect(),
            },
        ),
        |error| format!("Expected loop scope | {}", error).into(),
    )
}

fn push_scope() -> ParserHandle {
    map_err(
        map(
            sentence_list(lit("push"), inject("ankha/operation")),
            |values: Vec<ParserOutput>| AnkhaOperation::PushScope {
                script: values
                    .into_iter()
                    .map(|value| value.consume::<AnkhaOperation>().ok().unwrap())
                    .collect(),
            },
        ),
        |error| format!("Expected push scope | {}", error).into(),
    )
}

fn pop_scope() -> ParserHandle {
    map_err(
        map(sentence(lit("pop")), |_: String| AnkhaOperation::PopScope),
        |error| format!("Expected pop scope | {}", error).into(),
    )
}

fn ensure_register_type_op() -> ParserHandle {
    map_err(
        map(
            sentence_list(
                lit("ensure_register_type"),
                alt([type_query(), lit_string()]),
            ),
            |values: Vec<ParserOutput>| {
                let mut type_query = None;
                let mut name = None;
                for value in values {
                    if value.is::<AnkhaTypeQuery>() {
                        type_query = Some(value.consume::<AnkhaTypeQuery>().ok().unwrap());
                    } else if value.is::<String>() {
                        name = Some(value.consume::<String>().ok().unwrap());
                    } else {
                        unreachable!()
                    }
                }
                AnkhaOperation::EnsureRegisterType {
                    type_query: type_query.expect("Expected type query"),
                    address: AnkhaRegisterAddress::Name(name.expect("Missing register name")),
                }
            },
        ),
        |error| format!("Expected ensure register type | {}", error).into(),
    )
}

fn ensure_register_kind_op() -> ParserHandle {
    map_err(
        map(
            sentence_list(
                lit("ensure_register_kind"),
                alt([inject("ankha/kind"), lit_string()]),
            ),
            |values: Vec<ParserOutput>| {
                let mut kind = None;
                let mut name = None;
                for value in values {
                    if value.is::<AnkhaValueKind>() {
                        kind = Some(value.consume::<AnkhaValueKind>().ok().unwrap());
                    } else if value.is::<String>() {
                        name = Some(value.consume::<String>().ok().unwrap());
                    } else {
                        unreachable!()
                    }
                }
                AnkhaOperation::EnsureRegisterKind {
                    kind: kind.expect("Expected value kind"),
                    address: AnkhaRegisterAddress::Name(name.expect("Missing register name")),
                }
            },
        ),
        |error| format!("Expected ensure register kind | {}", error).into(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_meta() {
        let mut registry = ParserRegistry::default();
        install(&mut registry);

        let (rest, result) = registry.parse("ankha/meta_inner", "(id \"foo\")").unwrap();
        assert_eq!(rest, "");
        assert_eq!(
            result.consume::<Meta>().ok().unwrap(),
            Meta::Identifier("foo".to_owned())
        );

        let (rest, result) = registry
            .parse("ankha/meta_inner", "(value b true)")
            .unwrap();
        assert_eq!(rest, "");
        assert_eq!(
            result.consume::<Meta>().ok().unwrap(),
            Meta::Value(MetaValue::Bool(true))
        );

        let (rest, result) = registry
            .parse("ankha/meta_inner", "(value f 42.0)")
            .unwrap();
        assert_eq!(rest, "");
        assert_eq!(
            result.consume::<Meta>().ok().unwrap(),
            Meta::Value(MetaValue::Float(42.0))
        );

        let (rest, result) = registry.parse("ankha/meta_inner", "(value i 42)").unwrap();
        assert_eq!(rest, "");
        assert_eq!(
            result.consume::<Meta>().ok().unwrap(),
            Meta::Value(MetaValue::Integer(42))
        );

        let (rest, result) = registry
            .parse("ankha/meta_inner", "(value s \"foo\")")
            .unwrap();
        assert_eq!(rest, "");
        assert_eq!(
            result.consume::<Meta>().ok().unwrap(),
            Meta::Value(MetaValue::String("foo".to_owned()))
        );

        let input = "(array
            (value b true)
            (value i 42)
        )";
        let (rest, result) = registry.parse("ankha/meta_inner", input).unwrap();
        assert_eq!(rest, "");
        assert_eq!(
            result.consume::<Meta>().ok().unwrap(),
            Meta::Array(vec![
                Meta::Value(MetaValue::Bool(true)),
                Meta::Value(MetaValue::Integer(42))
            ])
        );

        let input = "(map
            (\"a\" (value b true))
            (\"b\" (value i 42))
        )";
        let (rest, result) = registry.parse("ankha/meta_inner", input).unwrap();
        assert_eq!(rest, "");
        assert_eq!(
            result.consume::<Meta>().ok().unwrap(),
            Meta::Map(
                [
                    ("a".to_owned(), Meta::Value(MetaValue::Bool(true))),
                    ("b".to_owned(), Meta::Value(MetaValue::Integer(42)))
                ]
                .into_iter()
                .collect()
            )
        );

        let input = "(meta (map
            (\"a\" (id \"identifier\"))
            (\"b\" (array
                (value b true)
                (value i 42)
                (value f 42.0)
                (value s \"Hello World!\")
            ))
        ))";
        let (rest, result) = registry.parse("ankha/meta", input).unwrap();
        assert_eq!(rest, "");
        assert_eq!(
            result.consume::<Meta>().ok().unwrap(),
            Meta::Map(
                [
                    ("a".to_owned(), Meta::Identifier("identifier".to_owned())),
                    (
                        "b".to_owned(),
                        Meta::Array(vec![
                            Meta::Value(MetaValue::Bool(true)),
                            Meta::Value(MetaValue::Integer(42)),
                            Meta::Value(MetaValue::Float(42.0)),
                            Meta::Value(MetaValue::String("Hello World!".to_owned())),
                        ])
                    )
                ]
                .into_iter()
                .collect()
            )
        );
    }

    #[test]
    fn test_file() {
        let mut registry = ParserRegistry::default();
        install(&mut registry);

        let input = "(file
            (use \"std\")
            (mod \"main\"
                (struct \"Foo\"
                    (meta (map
                        (\"a\" (id \"identifier\"))
                        (\"b\" (array
                            (value b true)
                            (value i 42)
                            (value f 42.0)
                            (value s \"Hello World!\")
                        ))
                    ))
                    (vis module)
                    (field \"a\" (kind owned))
                    (field \"b\" (kind ref))
                    (field \"c\" (kind refmut))
                    (field \"d\" (kind lazy))
                    (field \"e\" (kind box))
                )
                (enum \"Bar\"
                    (vis private)
                    (var \"A\")
                    (var \"B\" (field \"a\" (kind owned)))
                )
                (fn \"test\"
                    (vis public)
                    (type \"Foo\" \"main\")
                    (in \"a\" (kind owned))
                    (in \"b\" (kind owned))
                    (out \"result\" (kind owned))
                    (body
                        (group_reversed
                            (lit unit)
                            (lit bool true)
                            (lit f32 42.0)
                            (lit f64 42.0)
                            (lit char '@')
                            (lit string \"Hello World!\")
                        )
                        (group
                            (stack_drop)
                            (stack_unwrap_boolean)
                            (borrow)
                            (borrow_mut)
                            (lazy)
                            (borrow_field \"a\" (kind owned) (vis public))
                            (borrow_mut_field \"b\")
                            (borrow_unmanaged_field \"c\" (type \"Foo\") (vis public))
                            (borrow_mut_unmanaged_field \"d\")
                            (copy_from)
                            (move_into)
                            (swap_in)
                            (destructure \"a\" \"b\" \"c\")
                            (structure (type \"Foo\") \"a\" \"b\" \"c\")
                            (box)
                            (manage)
                            (unmanage)
                            (copy)
                            (swap)
                            (duplicate_box)
                            (ensure_stack_type (type \"Foo\"))
                            (ensure_register_type (type \"Foo\") 0)
                            (ensure_stack_kind (kind owned))
                            (ensure_register_kind (kind box) 0)
                            (call_method (fn \"add\" \"intrinsics\"
                                (vis public)
                                (in \"a\" (type \"Foo\"))
                                (out \"result\")
                            ))
                            (call_indirect)
                        )
                        (call_function (fn \"add\" \"intrinsics\"
                            (type \"Bar\")
                            (vis public)
                            (in \"a\" (type \"Foo\"))
                            (out \"result\")
                        ))
                        (branch
                            (success
                                (make_register \"a\" (kind owned))
                                (push_from_register \"a\")
                            )
                            (failure
                                (pop_to_register 0)
                                (drop_register 0)
                            )
                        )
                        (loop
                            (lit i8 42)
                            (lit i16 42)
                            (lit i32 42)
                            (lit i64 42)
                            (lit i128 42)
                            (lit isize 42)
                        )
                        (push
                            (lit u8 42)
                            (lit u16 42)
                            (lit u32 42)
                            (lit u64 42)
                            (lit u128 42)
                            (lit usize 42)
                        )
                        (pop)
                        (ensure_register_type (type \"Foo\") \"a\")
                        (ensure_register_kind (kind owned) \"a\")
                    )
                )
            )
        )";
        let (rest, result) = registry.parse("ankha/file", input).unwrap();
        assert_eq!(rest, "");
        assert!(result.consume::<AnkhaFile>().is_ok());
    }

    #[test]
    fn test_use() {
        let mut registry = ParserRegistry::default();
        install(&mut registry);

        let (rest, result) = registry.parse("ankha/use", "(use \"std\")").unwrap();
        assert_eq!(rest, "");
        assert_eq!(result.consume::<String>().ok().unwrap().as_str(), "std");
    }

    #[test]
    fn test_field() {
        let mut registry = ParserRegistry::default();
        install(&mut registry);

        let input = "(field \"a\" (vis private) (kind box))";
        let (rest, result) = registry.parse("ankha/field", input).unwrap();
        assert_eq!(rest, "");
        assert_eq!(
            result.consume::<AnkhaStructField>().ok().unwrap(),
            AnkhaStructField {
                meta: None,
                name: "a".to_owned(),
                visibility: Visibility::Private,
                kind: AnkhaValueKind::Box
            }
        );
    }

    #[test]
    fn test_struct() {
        let mut registry = ParserRegistry::default();
        install(&mut registry);

        let input = "(struct \"Foo\"
            (meta (map
                (\"a\" (id \"identifier\"))
                (\"b\" (array
                    (value b true)
                    (value i 42)
                    (value f 42.0)
                    (value s \"Hello World!\")
                ))
            ))
            (vis module)
            (field \"a\" (kind owned))
            (field \"b\" (kind ref))
            (field \"c\" (kind refmut))
            (field \"d\" (kind lazy))
            (field \"e\" (kind box))
        )";
        let (rest, result) = registry.parse("ankha/struct", input).unwrap();
        assert_eq!(rest, "");
        assert_eq!(
            result.consume::<AnkhaStruct>().ok().unwrap(),
            AnkhaStruct {
                meta: Some(Meta::Map(
                    [
                        ("a".to_owned(), Meta::Identifier("identifier".to_owned())),
                        (
                            "b".to_owned(),
                            Meta::Array(vec![
                                Meta::Value(MetaValue::Bool(true)),
                                Meta::Value(MetaValue::Integer(42)),
                                Meta::Value(MetaValue::Float(42.0)),
                                Meta::Value(MetaValue::String("Hello World!".to_owned())),
                            ])
                        )
                    ]
                    .into_iter()
                    .collect()
                )),
                name: "Foo".to_owned(),
                visibility: Visibility::Module,
                fields: vec![
                    AnkhaStructField {
                        meta: None,
                        name: "a".to_owned(),
                        visibility: Visibility::Public,
                        kind: AnkhaValueKind::Owned
                    },
                    AnkhaStructField {
                        meta: None,
                        name: "b".to_owned(),
                        visibility: Visibility::Public,
                        kind: AnkhaValueKind::Ref
                    },
                    AnkhaStructField {
                        meta: None,
                        name: "c".to_owned(),
                        visibility: Visibility::Public,
                        kind: AnkhaValueKind::RefMut
                    },
                    AnkhaStructField {
                        meta: None,
                        name: "d".to_owned(),
                        visibility: Visibility::Public,
                        kind: AnkhaValueKind::Lazy
                    },
                    AnkhaStructField {
                        meta: None,
                        name: "e".to_owned(),
                        visibility: Visibility::Public,
                        kind: AnkhaValueKind::Box
                    },
                ]
            }
        );
    }

    #[test]
    fn test_var() {
        let mut registry = ParserRegistry::default();
        install(&mut registry);

        let input = "(var \"B\" (field \"a\" (kind owned)))";
        let (rest, result) = registry.parse("ankha/var", input).unwrap();
        assert_eq!(rest, "");
        assert_eq!(
            result.consume::<AnkhaEnumVariant>().ok().unwrap(),
            AnkhaEnumVariant {
                meta: None,
                name: "B".to_owned(),
                fields: vec![AnkhaStructField {
                    meta: None,
                    name: "a".to_owned(),
                    visibility: Visibility::Public,
                    kind: AnkhaValueKind::Owned
                }],
                discriminant: None
            }
        );
    }

    #[test]
    fn test_enum() {
        let mut registry = ParserRegistry::default();
        install(&mut registry);

        let input = "(enum \"Bar\"
            (vis private)
            (var \"A\")
            (var \"B\" (field \"a\" (kind owned)))
        )";
        let (rest, result) = registry.parse("ankha/enum", input).unwrap();
        assert_eq!(rest, "");
        assert_eq!(
            result.consume::<AnkhaEnum>().ok().unwrap(),
            AnkhaEnum {
                meta: None,
                name: "Bar".to_owned(),
                visibility: Visibility::Private,
                variants: vec![
                    AnkhaEnumVariant {
                        meta: None,
                        name: "A".to_owned(),
                        fields: vec![],
                        discriminant: None
                    },
                    AnkhaEnumVariant {
                        meta: None,
                        name: "B".to_owned(),
                        fields: vec![AnkhaStructField {
                            meta: None,
                            name: "a".to_owned(),
                            visibility: Visibility::Public,
                            kind: AnkhaValueKind::Owned
                        }],
                        discriminant: None
                    },
                ],
                default_variant: None
            }
        );
    }

    #[test]
    fn test_param() {
        let mut registry = ParserRegistry::default();
        install(&mut registry);

        let input = "(in \"a\" (kind owned))";
        let (rest, result) = registry.parse("ankha/input", input).unwrap();
        assert_eq!(rest, "");
        assert_eq!(
            result
                .consume::<(AnkhaFunctionParameter, bool)>()
                .ok()
                .unwrap(),
            (
                AnkhaFunctionParameter {
                    meta: None,
                    name: "a".to_owned(),
                    kind: AnkhaValueKind::Owned
                },
                true
            )
        );

        let input = "(out \"result\" (kind owned))";
        let (rest, result) = registry.parse("ankha/output", input).unwrap();
        assert_eq!(rest, "");
        assert_eq!(
            result
                .consume::<(AnkhaFunctionParameter, bool)>()
                .ok()
                .unwrap(),
            (
                AnkhaFunctionParameter {
                    meta: None,
                    name: "result".to_owned(),
                    kind: AnkhaValueKind::Owned
                },
                false
            )
        );
    }

    #[test]
    fn test_function() {
        let mut registry = ParserRegistry::default();
        install(&mut registry);

        let input = "(fn \"test\"
            (vis public)
            (type \"Foo\" \"main\")
            (in \"a\" (kind owned))
            (in \"b\" (kind owned))
            (out \"result\" (kind owned))
            (body
                (lit i32 40)
                (lit i32 2)
                (call_function (fn \"add\"))
            )
        )";
        let (rest, result) = registry.parse("ankha/fn", input).unwrap();
        assert_eq!(rest, "");
        assert_eq!(
            result.consume::<AnkhaFunction>().ok().unwrap(),
            AnkhaFunction {
                meta: None,
                name: "test".to_owned(),
                type_name_module: Some(("Foo".to_owned(), Some("main".to_owned()))),
                visibility: Visibility::Public,
                inputs: vec![
                    AnkhaFunctionParameter {
                        meta: None,
                        name: "a".to_owned(),
                        kind: AnkhaValueKind::Owned
                    },
                    AnkhaFunctionParameter {
                        meta: None,
                        name: "b".to_owned(),
                        kind: AnkhaValueKind::Owned
                    }
                ],
                outputs: vec![AnkhaFunctionParameter {
                    meta: None,
                    name: "result".to_owned(),
                    kind: AnkhaValueKind::Owned
                }],
                script: vec![
                    AnkhaOperation::Expression(AnkhaExpression::Literal(AnkhaLiteral::I32(40))),
                    AnkhaOperation::Expression(AnkhaExpression::Literal(AnkhaLiteral::I32(2))),
                    AnkhaOperation::CallFunction(AnkhaFunctionQuery {
                        name: Some("add".to_owned()),
                        ..Default::default()
                    })
                ]
            }
        );
    }

    #[test]
    fn test_literal() {
        let mut registry = ParserRegistry::default();
        install(&mut registry);

        let (rest, result) = literal_unit().parse(&registry, "(lit unit)").unwrap();
        assert_eq!(rest, "");
        assert!(result.is::<()>());

        let (rest, result) = literal().parse(&registry, "(lit unit)").unwrap();
        assert_eq!(rest, "");
        assert_eq!(
            result.consume::<AnkhaLiteral>().ok().unwrap(),
            AnkhaLiteral::Unit
        );

        let (rest, result) = literal().parse(&registry, "(lit bool true)").unwrap();
        assert_eq!(rest, "");
        assert_eq!(
            result.consume::<AnkhaLiteral>().ok().unwrap(),
            AnkhaLiteral::Bool(true)
        );

        let (rest, result) = literal().parse(&registry, "(lit i8 42)").unwrap();
        assert_eq!(rest, "");
        assert_eq!(
            result.consume::<AnkhaLiteral>().ok().unwrap(),
            AnkhaLiteral::I8(42)
        );

        let (rest, result) = literal().parse(&registry, "(lit u8 42)").unwrap();
        assert_eq!(rest, "");
        assert_eq!(
            result.consume::<AnkhaLiteral>().ok().unwrap(),
            AnkhaLiteral::U8(42)
        );

        let (rest, result) = literal().parse(&registry, "(lit f32 42.0)").unwrap();
        assert_eq!(rest, "");
        assert_eq!(
            result.consume::<AnkhaLiteral>().ok().unwrap(),
            AnkhaLiteral::F32(42.0)
        );

        let (rest, result) = literal().parse(&registry, "(lit char '@')").unwrap();
        assert_eq!(rest, "");
        assert_eq!(
            result.consume::<AnkhaLiteral>().ok().unwrap(),
            AnkhaLiteral::Char('@')
        );

        let (rest, result) = literal()
            .parse(&registry, "(lit string \"Hello World!\")")
            .unwrap();
        assert_eq!(rest, "");
        assert_eq!(
            result.consume::<AnkhaLiteral>().ok().unwrap(),
            AnkhaLiteral::String("Hello World!".to_owned())
        );
    }

    #[test]
    fn test_expression() {
        let mut registry = ParserRegistry::default();
        install(&mut registry);

        let (rest, result) = expression().parse(&registry, "(lit unit)").unwrap();
        assert_eq!(rest, "");
        assert_eq!(
            result.consume::<AnkhaExpression>().ok().unwrap(),
            AnkhaExpression::Literal(AnkhaLiteral::Unit)
        );

        let (rest, result) = expression().parse(&registry, "(stack_drop)").unwrap();
        assert_eq!(rest, "");
        assert_eq!(
            result.consume::<AnkhaExpression>().ok().unwrap(),
            AnkhaExpression::StackDrop
        );

        let (rest, result) = expression()
            .parse(&registry, "(stack_unwrap_boolean)")
            .unwrap();
        assert_eq!(rest, "");
        assert_eq!(
            result.consume::<AnkhaExpression>().ok().unwrap(),
            AnkhaExpression::StackUnwrapBoolean
        );

        let (rest, result) = expression().parse(&registry, "(borrow)").unwrap();
        assert_eq!(rest, "");
        assert_eq!(
            result.consume::<AnkhaExpression>().ok().unwrap(),
            AnkhaExpression::Borrow
        );

        let (rest, result) = expression().parse(&registry, "(borrow_mut)").unwrap();
        assert_eq!(rest, "");
        assert_eq!(
            result.consume::<AnkhaExpression>().ok().unwrap(),
            AnkhaExpression::BorrowMut
        );

        let (rest, result) = expression().parse(&registry, "(lazy)").unwrap();
        assert_eq!(rest, "");
        assert_eq!(
            result.consume::<AnkhaExpression>().ok().unwrap(),
            AnkhaExpression::Lazy
        );

        let (rest, result) = expression()
            .parse(&registry, "(borrow_field \"a\" (kind owned) (vis public))")
            .unwrap();
        assert_eq!(rest, "");
        assert_eq!(
            result.consume::<AnkhaExpression>().ok().unwrap(),
            AnkhaExpression::BorrowField {
                name: "a".to_owned(),
                kind: AnkhaValueKind::Owned,
                visibility: Some(Visibility::Public)
            }
        );

        let (rest, result) = expression()
            .parse(&registry, "(borrow_mut_field \"b\")")
            .unwrap();
        assert_eq!(rest, "");
        assert_eq!(
            result.consume::<AnkhaExpression>().ok().unwrap(),
            AnkhaExpression::BorrowMutField {
                name: "b".to_owned(),
                kind: AnkhaValueKind::Any,
                visibility: None
            }
        );

        let (rest, result) = expression()
            .parse(
                &registry,
                "(borrow_unmanaged_field \"c\" (type \"Foo\") (vis public))",
            )
            .unwrap();
        assert_eq!(rest, "");
        assert_eq!(
            result.consume::<AnkhaExpression>().ok().unwrap(),
            AnkhaExpression::BorrowUnmanagedField {
                query: AnkhaFieldQuery {
                    name: "c".to_owned(),
                    type_query: Some(AnkhaTypeQuery {
                        name: Some("Foo".to_owned()),
                        ..Default::default()
                    }),
                    visibility: Some(Visibility::Public)
                }
            }
        );

        let (rest, result) = expression()
            .parse(&registry, "(borrow_mut_unmanaged_field \"d\")")
            .unwrap();
        assert_eq!(rest, "");
        assert_eq!(
            result.consume::<AnkhaExpression>().ok().unwrap(),
            AnkhaExpression::BorrowMutUnmanagedField {
                query: AnkhaFieldQuery {
                    name: "d".to_owned(),
                    type_query: None,
                    visibility: None
                }
            }
        );

        let (rest, result) = expression().parse(&registry, "(copy_from)").unwrap();
        assert_eq!(rest, "");
        assert_eq!(
            result.consume::<AnkhaExpression>().ok().unwrap(),
            AnkhaExpression::CopyFrom
        );

        let (rest, result) = expression().parse(&registry, "(move_into)").unwrap();
        assert_eq!(rest, "");
        assert_eq!(
            result.consume::<AnkhaExpression>().ok().unwrap(),
            AnkhaExpression::MoveInto
        );

        let (rest, result) = expression().parse(&registry, "(swap_in)").unwrap();
        assert_eq!(rest, "");
        assert_eq!(
            result.consume::<AnkhaExpression>().ok().unwrap(),
            AnkhaExpression::SwapIn
        );

        let (rest, result) = expression()
            .parse(&registry, "(destructure \"a\" \"b\" \"c\")")
            .unwrap();
        assert_eq!(rest, "");
        assert_eq!(
            result.consume::<AnkhaExpression>().ok().unwrap(),
            AnkhaExpression::Destructure {
                fields: vec!["a".to_owned(), "b".to_owned(), "c".to_owned()]
            }
        );

        let (rest, result) = expression()
            .parse(&registry, "(structure (type \"Foo\") \"a\" \"b\" \"c\")")
            .unwrap();
        assert_eq!(rest, "");
        assert_eq!(
            result.consume::<AnkhaExpression>().ok().unwrap(),
            AnkhaExpression::Structure {
                type_query: AnkhaTypeQuery {
                    name: Some("Foo".to_owned()),
                    ..Default::default()
                },
                fields: vec!["a".to_owned(), "b".to_owned(), "c".to_owned()]
            }
        );

        let (rest, result) = expression().parse(&registry, "(box)").unwrap();
        assert_eq!(rest, "");
        assert_eq!(
            result.consume::<AnkhaExpression>().ok().unwrap(),
            AnkhaExpression::Box
        );

        let (rest, result) = expression().parse(&registry, "(manage)").unwrap();
        assert_eq!(rest, "");
        assert_eq!(
            result.consume::<AnkhaExpression>().ok().unwrap(),
            AnkhaExpression::Manage
        );

        let (rest, result) = expression().parse(&registry, "(unmanage)").unwrap();
        assert_eq!(rest, "");
        assert_eq!(
            result.consume::<AnkhaExpression>().ok().unwrap(),
            AnkhaExpression::Unmanage
        );

        let (rest, result) = expression().parse(&registry, "(copy)").unwrap();
        assert_eq!(rest, "");
        assert_eq!(
            result.consume::<AnkhaExpression>().ok().unwrap(),
            AnkhaExpression::Copy
        );

        let (rest, result) = expression().parse(&registry, "(swap)").unwrap();
        assert_eq!(rest, "");
        assert_eq!(
            result.consume::<AnkhaExpression>().ok().unwrap(),
            AnkhaExpression::Swap
        );

        let (rest, result) = expression().parse(&registry, "(duplicate_box)").unwrap();
        assert_eq!(rest, "");
        assert_eq!(
            result.consume::<AnkhaExpression>().ok().unwrap(),
            AnkhaExpression::DuplicateBox
        );

        let (rest, result) = expression()
            .parse(&registry, "(ensure_stack_type (type \"Foo\"))")
            .unwrap();
        assert_eq!(rest, "");
        assert_eq!(
            result.consume::<AnkhaExpression>().ok().unwrap(),
            AnkhaExpression::EnsureStackType {
                type_query: AnkhaTypeQuery {
                    name: Some("Foo".to_owned()),
                    ..Default::default()
                }
            }
        );

        let (rest, result) = expression()
            .parse(&registry, "(ensure_register_type (type \"Foo\") 0)")
            .unwrap();
        assert_eq!(rest, "");
        assert_eq!(
            result.consume::<AnkhaExpression>().ok().unwrap(),
            AnkhaExpression::EnsureRegisterType {
                type_query: AnkhaTypeQuery {
                    name: Some("Foo".to_owned()),
                    ..Default::default()
                },
                index: 0
            }
        );

        let input = "(call_method (fn \"add\" \"intrinsics\"
            (vis public)
            (in \"a\" (type \"Foo\"))
            (out \"result\")
        ))";
        let (rest, result) = expression().parse(&registry, input).unwrap();
        assert_eq!(rest, "");
        assert_eq!(
            result.consume::<AnkhaExpression>().ok().unwrap(),
            AnkhaExpression::CallMethod {
                function_query: AnkhaFunctionQuery {
                    name: Some("add".to_owned()),
                    module_name: Some("intrinsics".to_owned()),
                    type_query: None,
                    visibility: Some(Visibility::Public),
                    inputs: vec![AnkhaFunctionQueryParam {
                        name: Some("a".to_owned()),
                        type_query: Some(AnkhaTypeQuery {
                            name: Some("Foo".to_owned()),
                            ..Default::default()
                        })
                    }],
                    outputs: vec![AnkhaFunctionQueryParam {
                        name: Some("result".to_owned()),
                        type_query: None
                    }]
                }
            }
        );

        let input = "(call_indirect)";
        let (rest, result) = expression().parse(&registry, input).unwrap();
        assert_eq!(rest, "");
        assert_eq!(
            result.consume::<AnkhaExpression>().ok().unwrap(),
            AnkhaExpression::CallIndirect
        );
    }

    #[test]
    fn test_operation() {
        let mut registry = ParserRegistry::default();
        install(&mut registry);

        let (rest, result) = operation().parse(&registry, "(lit unit)").unwrap();
        assert_eq!(rest, "");
        assert_eq!(
            result.consume::<AnkhaOperation>().ok().unwrap(),
            AnkhaOperation::Expression(AnkhaExpression::Literal(AnkhaLiteral::Unit))
        );

        let (rest, result) = operation().parse(&registry, "(group (lit unit))").unwrap();
        assert_eq!(rest, "");
        assert_eq!(
            result.consume::<AnkhaOperation>().ok().unwrap(),
            AnkhaOperation::Group(vec![AnkhaOperation::Expression(AnkhaExpression::Literal(
                AnkhaLiteral::Unit
            ))])
        );

        let (rest, result) = operation()
            .parse(&registry, "(group_reversed (lit unit))")
            .unwrap();
        assert_eq!(rest, "");
        assert_eq!(
            result.consume::<AnkhaOperation>().ok().unwrap(),
            AnkhaOperation::GroupReversed(vec![AnkhaOperation::Expression(
                AnkhaExpression::Literal(AnkhaLiteral::Unit)
            )])
        );

        let (rest, result) = operation()
            .parse(&registry, "(make_register (kind owned) \"a\")")
            .unwrap();
        assert_eq!(rest, "");
        assert_eq!(
            result.consume::<AnkhaOperation>().ok().unwrap(),
            AnkhaOperation::MakeRegister {
                kind: AnkhaValueKind::Owned,
                name: Some("a".to_owned())
            }
        );

        let (rest, result) = operation().parse(&registry, "(drop_register 0)").unwrap();
        assert_eq!(rest, "");
        assert_eq!(
            result.consume::<AnkhaOperation>().ok().unwrap(),
            AnkhaOperation::DropRegister(AnkhaRegisterAddress::Index(0))
        );

        let (rest, result) = operation()
            .parse(&registry, "(drop_register \"a\")")
            .unwrap();
        assert_eq!(rest, "");
        assert_eq!(
            result.consume::<AnkhaOperation>().ok().unwrap(),
            AnkhaOperation::DropRegister(AnkhaRegisterAddress::Name("a".to_owned()))
        );

        let (rest, result) = operation()
            .parse(&registry, "(push_from_register 0)")
            .unwrap();
        assert_eq!(rest, "");
        assert_eq!(
            result.consume::<AnkhaOperation>().ok().unwrap(),
            AnkhaOperation::PushFromRegister(AnkhaRegisterAddress::Index(0))
        );

        let (rest, result) = operation()
            .parse(&registry, "(push_from_register \"a\")")
            .unwrap();
        assert_eq!(rest, "");
        assert_eq!(
            result.consume::<AnkhaOperation>().ok().unwrap(),
            AnkhaOperation::PushFromRegister(AnkhaRegisterAddress::Name("a".to_owned()))
        );

        let (rest, result) = operation().parse(&registry, "(pop_to_register 0)").unwrap();
        assert_eq!(rest, "");
        assert_eq!(
            result.consume::<AnkhaOperation>().ok().unwrap(),
            AnkhaOperation::PopToRegister(AnkhaRegisterAddress::Index(0))
        );

        let (rest, result) = operation()
            .parse(&registry, "(pop_to_register \"a\")")
            .unwrap();
        assert_eq!(rest, "");
        assert_eq!(
            result.consume::<AnkhaOperation>().ok().unwrap(),
            AnkhaOperation::PopToRegister(AnkhaRegisterAddress::Name("a".to_owned()))
        );

        let input = "(call_function (fn \"add\" \"intrinsics\"
            (type \"Bar\")
            (vis public)
            (in \"a\" (type \"Foo\"))
            (out \"result\")
        ))";
        let (rest, result) = operation().parse(&registry, input).unwrap();
        assert_eq!(rest, "");
        assert_eq!(
            result.consume::<AnkhaOperation>().ok().unwrap(),
            AnkhaOperation::CallFunction(AnkhaFunctionQuery {
                name: Some("add".to_owned()),
                module_name: Some("intrinsics".to_owned()),
                type_query: Some(AnkhaTypeQuery {
                    name: Some("Bar".to_owned()),
                    ..Default::default()
                }),
                visibility: Some(Visibility::Public),
                inputs: vec![AnkhaFunctionQueryParam {
                    name: Some("a".to_owned()),
                    type_query: Some(AnkhaTypeQuery {
                        name: Some("Foo".to_owned()),
                        ..Default::default()
                    })
                }],
                outputs: vec![AnkhaFunctionQueryParam {
                    name: Some("result".to_owned()),
                    type_query: None
                }]
            })
        );

        let (rest, result) = operation().parse(&registry, "(loop (lit unit))").unwrap();
        assert_eq!(rest, "");
        assert_eq!(
            result.consume::<AnkhaOperation>().ok().unwrap(),
            AnkhaOperation::LoopScope {
                script: vec![AnkhaOperation::Expression(AnkhaExpression::Literal(
                    AnkhaLiteral::Unit
                ))]
            }
        );

        let (rest, result) = operation().parse(&registry, "(push (lit unit))").unwrap();
        assert_eq!(rest, "");
        assert_eq!(
            result.consume::<AnkhaOperation>().ok().unwrap(),
            AnkhaOperation::PushScope {
                script: vec![AnkhaOperation::Expression(AnkhaExpression::Literal(
                    AnkhaLiteral::Unit
                ))]
            }
        );

        let (rest, result) = operation().parse(&registry, "(pop)").unwrap();
        assert_eq!(rest, "");
        assert_eq!(
            result.consume::<AnkhaOperation>().ok().unwrap(),
            AnkhaOperation::PopScope
        );

        let (rest, result) = operation()
            .parse(&registry, "(ensure_register_type (type \"Foo\") \"a\")")
            .unwrap();
        assert_eq!(rest, "");
        assert_eq!(
            result.consume::<AnkhaOperation>().ok().unwrap(),
            AnkhaOperation::EnsureRegisterType {
                type_query: AnkhaTypeQuery {
                    name: Some("Foo".to_owned()),
                    ..Default::default()
                },
                address: AnkhaRegisterAddress::Name("a".to_owned())
            }
        );
    }
}
