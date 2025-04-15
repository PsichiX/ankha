use intuicio_parser::{
    ParserHandle, ParserRegistry,
    pratt::{PrattParserAssociativity, PrattParserRule},
    shorthand::{
        alt, inject, list, lit, map_err, number_float, oc, ows, pratt, prefix, suffix, template,
    },
};

fn file() -> ParserHandle {
    const CONTENT: &str =
        "(file (mod \"main\" (fn \"main\" (out \"result\" (kind owned)) (body @{}@))))";
    map_err(
        template(
            inject("calcite/expr"),
            Some("ankha/file".to_owned()),
            CONTENT,
        ),
        |error| format!("Expected `calcite/file` | {}", error).into(),
    )
}

fn number() -> ParserHandle {
    const CONTENT: &str = "(lit f64 @{}@)";
    map_err(template(number_float(), None, CONTENT), |error| {
        format!("Expected `calcite/number` | {}", error).into()
    })
}

fn op() -> ParserHandle {
    map_err(
        alt([lit("+"), lit("-"), lit("*"), lit("/"), lit("#"), lit("!")]),
        |error| format!("Expected operator | {}", error).into(),
    )
}

fn sub_expr() -> ParserHandle {
    map_err(
        oc(
            inject("calcite/expr"),
            suffix(lit("("), ows()),
            prefix(lit(")"), ows()),
        ),
        |error| format!("Expected sub-expression | {}", error).into(),
    )
}

fn item() -> ParserHandle {
    alt([
        inject("calcite/number"),
        inject("calcite/op"),
        inject("calcite/sub_expr"),
    ])
}

fn expr_tokenizer() -> ParserHandle {
    list(inject("calcite/item"), ows(), true)
}

fn expr() -> ParserHandle {
    pratt(
        inject("calcite/expr_tokenizer"),
        vec![
            vec![
                PrattParserRule::infix(
                    "+".to_owned(),
                    |lhs, rhs| {
                        format!(
                            "(group_reversed (call_function (fn \"add\")) {} {})",
                            lhs, rhs
                        )
                    },
                    PrattParserAssociativity::Left,
                ),
                PrattParserRule::infix(
                    "-".to_owned(),
                    |lhs, rhs| {
                        format!(
                            "(group_reversed (call_function (fn \"sub\")) {} {})",
                            lhs, rhs
                        )
                    },
                    PrattParserAssociativity::Left,
                ),
            ],
            vec![
                PrattParserRule::infix(
                    "*".to_owned(),
                    |lhs, rhs| {
                        format!(
                            "(group_reversed (call_function (fn \"mul\")) {} {})",
                            lhs, rhs
                        )
                    },
                    PrattParserAssociativity::Left,
                ),
                PrattParserRule::infix(
                    "/".to_owned(),
                    |lhs, rhs| {
                        format!(
                            "(group_reversed (call_function (fn \"div\")) {} {})",
                            lhs, rhs
                        )
                    },
                    PrattParserAssociativity::Left,
                ),
            ],
            vec![PrattParserRule::prefix("#", |value| {
                format!("(group_reversed (call_function (fn \"floor\")) {})", value)
            })],
            vec![PrattParserRule::postfix("!".to_owned(), |value| {
                format!("(group_reversed (call_function (fn \"fract\")) {})", value)
            })],
        ],
    )
}

pub fn install(registry: &mut ParserRegistry) {
    let file = file();
    registry.add_parser("calcite/number", number());
    registry.add_parser("calcite/op", op());
    registry.add_parser("calcite/sub_expr", sub_expr());
    registry.add_parser("calcite/item", item());
    registry.add_parser("calcite/expr_tokenizer", expr_tokenizer());
    registry.add_parser("calcite/expr", expr());
    registry.add_parser("calcite/file", file.clone());
    registry
        .extend("ENTRY", file)
        .expect("Could not extend `ENTRY` rule");
}

#[cfg(test)]
mod tests {
    use ankha::script::*;
    use intuicio_core::prelude::*;
    use intuicio_parser::{ParserOutput, ParserRegistry};

    #[test]
    fn test_parsers() {
        let mut registry = ParserRegistry::default();
        ankha::parser::install(&mut registry);
        crate::parser::install(&mut registry);

        let (rest, value) = registry.parse("calcite/number", "42").unwrap();
        assert_eq!(rest, "");
        assert_eq!(
            value.consume::<String>().ok().unwrap().as_str(),
            "(lit f64 42)"
        );

        let (rest, value) = registry.parse("calcite/op", "+").unwrap();
        assert_eq!(rest, "");
        assert_eq!(value.consume::<String>().ok().unwrap().as_str(), "+");

        let (rest, value) = registry.parse("calcite/expr_tokenizer", "42").unwrap();
        assert_eq!(rest, "");
        let result = value.consume::<Vec<ParserOutput>>().ok().unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].read::<String>().unwrap().as_str(), "(lit f64 42)");

        let (rest, value) = registry.parse("calcite/expr", "42").unwrap();
        assert_eq!(rest, "");
        assert_eq!(
            value.consume::<String>().ok().unwrap().as_str(),
            "(lit f64 42)"
        );

        let (rest, value) = registry
            .parse("calcite/expr", "(3 + 4) * 2 - 1 / 5")
            .unwrap();
        assert_eq!(rest, "");
        assert_eq!(
            value.consume::<String>().ok().unwrap().as_str(),
            "(group_reversed (call_function (fn \"sub\")) (group_reversed (call_function (fn \"mul\")) (group_reversed (call_function (fn \"add\")) (lit f64 3) (lit f64 4)) (lit f64 2)) (group_reversed (call_function (fn \"div\")) (lit f64 1) (lit f64 5)))"
        );

        let (rest, value) = registry.parse("calcite/file", "42").unwrap();
        assert_eq!(rest, "");
        assert_eq!(
            value.consume::<AnkhaFile>().ok().unwrap(),
            AnkhaFile {
                modules: vec![AnkhaModule {
                    name: "main".to_owned(),
                    structs: vec![],
                    enums: vec![],
                    functions: vec![AnkhaFunction {
                        meta: None,
                        name: "main".to_owned(),
                        type_name_module: None,
                        visibility: Visibility::Public,
                        inputs: vec![],
                        outputs: vec![AnkhaFunctionParameter {
                            meta: None,
                            name: "result".to_owned(),
                            kind: AnkhaValueKind::Owned
                        }],
                        script: vec![AnkhaOperation::Expression(AnkhaExpression::Literal(
                            AnkhaLiteral::F64(42.0)
                        ))]
                    }],
                }],
                ..Default::default()
            }
        );
    }
}
