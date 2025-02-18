pub mod import_model;
pub mod import_namespace;
pub mod interface;
pub mod package;
pub mod structure;
pub mod symbol_table;
pub mod symbol_table_builder;
pub mod type_collection;
pub mod variable_declaration;
pub mod version;
pub mod attribute;
pub mod type_def;
use import_model::ImportModel;
use import_namespace::ImportNamespace;
use interface::Interface;
use package::Package;
use type_def::TypeDef;
use attribute::Attribute;
use structure::Structure;
use type_collection::TypeCollection;
use variable_declaration::VariableDeclaration;
use version::Version;

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
    #[test]
    fn test_symbol_table_4() {
        let src = "package org.javaohjavawhyareyouso
	interface endOfPlaylist {  version {major 25 minor 60}}";
        let publisher = parse(src).unwrap();
        publisher.print(Key(0), Some(true));
        let fmt = symbol_table_builder::SymbolTableBuilder::new(src, &publisher);
        let output = fmt.create_symbol_table();
        println!("Formatted:\n\n{:#?}", output.unwrap());
    }
    #[test]
    fn test_symbol_table_5() {
        let src = "package org.javaohjavawhyareyouso
	interface endOfPlaylist {  version {major 25 minor 60}}
    interface endOfPlaylist {  version {major 23 minor 40}}";
        let publisher = parse(src).unwrap();
        publisher.print(Key(0), Some(true));
        let fmt = symbol_table_builder::SymbolTableBuilder::new(src, &publisher);
        let output = fmt.create_symbol_table();
        let err = output.unwrap_err();
        println!("Err: {:?}", err)
    }

    #[test]
    fn test_symbol_table_6() {
        let src = "package org.javaohjavawhyareyouso
	interface endOfPlaylist {  version {major 25 minor 60}struct thing{p1 p1 p2 p2}
}   ";
        let publisher = parse(src).unwrap();
        publisher.print(Key(0), Some(true));
        let fmt = symbol_table_builder::SymbolTableBuilder::new(src, &publisher);
        let output = fmt.create_symbol_table();
        println!("Formatted:\n\n{:#?}", output.unwrap());
    }
    #[test]
    fn test_symbol_table_7() {
        let src = "package org.javaohjavawhyareyouso
	interface endOfPlaylist {  version {major 25 minor 60}struct thing{p1 p1 p2 p1}
}   ";
        let publisher = parse(src).unwrap();
        publisher.print(Key(0), Some(true));
        let fmt = symbol_table_builder::SymbolTableBuilder::new(src, &publisher);
        let output = fmt.create_symbol_table();
        println!("Formatted:\n\n{:#?}", output.unwrap_err());
    }

    #[test]
    fn test_symbol_table_8() {
        let src = "package org.javaohjavawhyareyouso
	interface endOfPlaylist {  version {major 25 minor 60}struct thing{p1 p1 p2 p2}struct thing{}
}   ";
        let publisher = parse(src).unwrap();
        publisher.print(Key(0), Some(true));
        let fmt = symbol_table_builder::SymbolTableBuilder::new(src, &publisher);
        let output = fmt.create_symbol_table();
        println!("Formatted:\n\n{}", output.unwrap_err());
    }

    #[test]
    fn test_symbol_table_9() {
        let src = "package org.javaohjavawhyareyouso
	interface endOfPlaylist {  version {major 25 minor 60}struct thing{p1 p1 p2 p2}struct thing2{}attribute uint8 thing
}   ";
        let publisher = parse(src).unwrap();
        publisher.print(Key(0), Some(true));
        let fmt = symbol_table_builder::SymbolTableBuilder::new(src, &publisher);
        let output = fmt.create_symbol_table();
        println!("Formatted:\n\n{:#?}", output.unwrap());
    }

    #[test]
    fn test_symbol_table_10() {
        let src = "package org.javaohjavawhyareyouso
	interface endOfPlaylist {  version {major 25 minor 60}struct thing{p1 p1 p2 p2}struct thing2{}attribute uint8 thing
attribute uint16 thing2}   ";
        let publisher = parse(src).unwrap();
        publisher.print(Key(0), Some(true));
        let fmt = symbol_table_builder::SymbolTableBuilder::new(src, &publisher);
        let output = fmt.create_symbol_table();
        println!("Formatted:\n\n{:#?}", output.unwrap());
    }


    #[test]
    fn test_symbol_table_11() {
        let src = "package org.javaohjavawhyareyouso
	interface endOfPlaylist {  version {major 25 minor 60}struct thing{p1 p1 p2 p2}struct thing2{}attribute uint8 thing
attribute uint16 thing}   ";
        let publisher = parse(src).unwrap();
        publisher.print(Key(0), Some(true));
        let fmt = symbol_table_builder::SymbolTableBuilder::new(src, &publisher);
        let output = fmt.create_symbol_table();
        println!("Formatted:\n\n{}", output.unwrap_err());
    }


    #[test]
    fn test_symbol_table_12() {
        let src = "package org.javaohjavawhyareyouso
	interface endOfPlaylist {  version {major 25 minor 60}typedef aTypedef is Int16
    struct thing{p1 p1 p2 p2}attribute uint8 thing\n method thing 
    {in {param param}  out {param2 param2 org.param3 param3}}method thing {in {param param} 
    out {param2 param2 org.param3 param3}} 	
}	";
        let publisher = parse(src).unwrap();
        publisher.print(Key(0), Some(true));
        let fmt = symbol_table_builder::SymbolTableBuilder::new(src, &publisher);
        let output = fmt.create_symbol_table();
        println!("Formatted:\n\n{:?}", output.unwrap());
    }

    #[test]
    fn test_symbol_table_16() {
        let src = "package org.javaohjavawhyareyouso
	interface endOfPlaylist {  version {major 25 minor 60}struct thing{p1 p1 p2 p2}attribute uint8 thing\n method thing 
    {in {param param}  out {param2 param2 org.param3 param3}}method thing {in {param param} 
    out {param2 param2 org.param3 param3}} 	typedef aTypedef is Int16
}	";
        let publisher = parse(src).unwrap();
        publisher.print(Key(0), Some(true));
        let fmt = symbol_table_builder::SymbolTableBuilder::new(src, &publisher);
        let output = fmt.create_symbol_table();
        println!("Formatted:\n\n{:?}", output.unwrap());
    }
}
