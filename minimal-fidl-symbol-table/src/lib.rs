pub mod import_model;
pub mod import_namespace;
pub mod interface;
pub mod package;
pub mod symbol_table;
pub mod symbol_table_builder;
pub mod type_collection;
use import_model::ImportModel;
use import_namespace::ImportNamespace;
use interface::Interface;
use package::Package;
use type_collection::TypeCollection;

#[cfg(test)]
mod tests {
    use crate::symbol_table_builder;
    use minimal_fidl_parser::{
        BasicContext, BasicPublisher, Context, Key, Rules, Source, _var_name, grammar, RULES_SIZE,
    };
    use std::cell::RefCell;

    pub fn parse(input: &str) -> Option<BasicPublisher> {
        let string = input.to_string();
        let src_len = string.len() as u32;
        let source = Source::new(&string);
        let position: u32 = 0;
        let result: (bool, u32);
        let context = RefCell::new(BasicContext::new(src_len as usize, RULES_SIZE as usize));
        {
            let executor = _var_name(Rules::Grammar, &context, grammar);
            result = executor(Key(0), &source, position);
        }
        if result != (true, src_len) {
            println!("Failed with : {:?}", result);
            return None;
        }

        let publisher = context.into_inner().get_publisher().clear_false();
        Some(publisher)
    }

    #[test]
    fn test_symbol_table_1() {
        let src = "package org.javaohjavawhyareyouso
	interface endOfPlaylist { }	";
        let publisher = parse(src).unwrap();
        publisher.print(Key(0), Some(true));
        let fmt = symbol_table_builder::SymbolTableBuilder::new(src, &publisher);
        let output = fmt.create_symbol_table();
        println!("{:?}", output);
        println!(
            "Formatted:\n\n{:#?}",
            output.expect("We expect no symbol table errors")
        );
    }
    #[test]
    fn test_symbol_table_2() {
        let src = r#"package org.javaohjavawhyareyouso
        import org.franca.omgidl.* from "OMGIDLBase.fidl" //Also Comment

	interface endOfPlaylist { }	"#;
        let publisher = parse(src).unwrap();
        publisher.print(Key(0), Some(true));
        let fmt = symbol_table_builder::SymbolTableBuilder::new(src, &publisher);
        let output = fmt.create_symbol_table();
        println!("{:?}", output);
        println!(
            "Formatted:\n\n{:#?}",
            output.expect("We expect no symbol table errors")
        );
    }
    #[test]
    fn test_symbol_table_3() {
        let src = r#"package whatever 
        import model "csm_t.fidl""#;
        let publisher = parse(src).unwrap();
        publisher.print(Key(0), Some(true));
        let fmt = symbol_table_builder::SymbolTableBuilder::new(src, &publisher);
        let output = fmt.create_symbol_table();
        println!(
            "Formatted:\n\n{:#?}",
            output.expect("We expect no symbol table errors")
        );
    }
}
