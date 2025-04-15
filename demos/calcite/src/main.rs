pub mod library;
pub mod parser;

use ankha::{library::AnkhaVmScope, parser::*, script::*};
use intuicio_core::prelude::*;
use intuicio_data::managed::DynamicManaged;
use std::time::Instant;

fn main() {
    let original_registry = Registry::default()
        .with_basic_types()
        .with_install(ankha::library::install)
        .with_install(crate::library::install);
    let mut context = Context::new(10240, 10240);
    loop {
        println!();
        println!("* Provide equation:");
        let mut line = String::default();
        if let Err(error) = std::io::stdin().read_line(&mut line) {
            println!("* Could not read line: {}", error);
        }
        let mut line = line.trim();
        if line.is_empty() || line.starts_with("exit") {
            return;
        }
        let mut ast = false;
        if line.starts_with("ast") {
            ast = true;
            line = &line[(b"ast".len())..];
        }
        let mut registry = original_registry.clone();
        let mut timer = Instant::now();
        let file = match AnkhaContentParser::default()
            .with_setup(crate::parser::install)
            .parse_file_content(line)
        {
            Ok(file) => file,
            Err(error) => {
                eprintln!("* Error: {}", error);
                continue;
            }
        };
        println!("* Parsing: {:?}", timer.elapsed());
        if ast {
            println!("* AST: {:#?}", file);
            continue;
        }
        let mut package = AnkhaPackage::default();
        package.files.insert("main.ankha".to_owned(), file);
        timer = Instant::now();
        package
            .compile()
            .install::<AnkhaVmScope>(&mut registry, None);
        println!("* Compiling: {:?}", timer.elapsed());
        timer = Instant::now();
        registry
            .find_function(FunctionQuery {
                name: Some("main".into()),
                module_name: Some("main".into()),
                ..Default::default()
            })
            .unwrap()
            .invoke(&mut context, &registry);
        println!("* Executing: {:?}", timer.elapsed());
        let result = context
            .stack()
            .pop::<DynamicManaged>()
            .unwrap()
            .consume::<f64>()
            .ok()
            .unwrap();
        println!("* Result: {}", result);
    }
}

#[cfg(test)]
mod tests {
    use ankha::library::AnkhaVmScope;

    use super::*;

    #[test]
    fn test_calcite() {
        const CONTENT: &str = "(3 + 4) * 2 - 1 / 5";

        let mut registry = Registry::default()
            .with_basic_types()
            .with_install(ankha::library::install)
            .with_install(crate::library::install);
        let file = AnkhaContentParser::default()
            .with_setup(crate::parser::install)
            .parse_file_content(CONTENT)
            .unwrap();
        let mut package = AnkhaPackage::default();
        package.files.insert("main.ankha".to_owned(), file);
        package
            .compile()
            .install::<AnkhaVmScope>(&mut registry, None);
        assert!(
            registry
                .find_function(FunctionQuery {
                    name: Some("main".into()),
                    module_name: Some("main".into()),
                    ..Default::default()
                })
                .is_some()
        );
        let mut host = Host::new(Context::new(10240, 10240), registry.into());
        let (result,) = host
            .call_function::<(DynamicManaged,), _>("main", "main", None)
            .unwrap()
            .run(());
        assert_eq!(result.consume::<f64>().ok().unwrap(), 13.8);
    }
}
