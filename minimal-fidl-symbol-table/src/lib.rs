pub mod import_model;
pub mod import_namespace;
pub mod interface;
pub mod package;
pub mod structure;
pub mod symbol_table;
pub mod method;
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
use method::Method;
use type_def::TypeDef;
use attribute::Attribute;
use structure::Structure;
use type_collection::TypeCollection;
use variable_declaration::VariableDeclaration;
use version::Version;

#[cfg(test)]
mod tests {
    use crate::{symbol_table::SymbolTable, symbol_table_builder};
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
    struct thing{p1 p1 p2 p2}attribute uint8 thing\n method thing2 
    {in {param param}  out {param2 param2 org.param3 param3}}method thing {in {param param} 
    out {param2 param2 org.param3 param3}} 	
}	";
        let publisher = parse(src).unwrap();
        publisher.print(Key(0), Some(true));
        let fmt = symbol_table_builder::SymbolTableBuilder::new(src, &publisher);
        let output = fmt.create_symbol_table();
        println!("Formatted:\n\n{:#?}", output.unwrap());
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
        println!("Formatted:\n\n{:?}", output.unwrap_err());
    }
    #[test]
    #[should_panic] // Temporary because parser will fail instead,
    fn test_symbol_table_17() {
        let src = "package org.javaohjavawhyareyouso
	interface endOfPlaylist {  version {major 25 minor 60}version{}}";
        let publisher = parse(src).unwrap();
        // publisher.print(Key(0), Some(true));
        // let fmt = symbol_table_builder::SymbolTableBuilder::new(src, &publisher);
        // let output = fmt.create_symbol_table();
        // println!("Formatted:\n\n{:#?}", output.unwrap());
    }
    #[test]
    fn test_symbol_table_18() {
        let src = "package org.javaohjavawhyareyouso
	interface endOfPlaylist {  version {major 25 minor 60}struct thing{p1 p1 p2 p2}attribute uint8 thing\n method thing 
    {in {param param}  out {param2 param2 org.param3 param3}}method thing2 {in {param param} 
    out {param2 param2 org.param3 param3}} 	typedef aTypedef is Int16
}	";
        let publisher = parse(src).unwrap();
        publisher.print(Key(0), Some(true));
        let fmt = symbol_table_builder::SymbolTableBuilder::new(src, &publisher);
        let output = fmt.create_symbol_table();
        println!("Formatted:\n\n{:#?}", output.unwrap());
    }
    #[test]
    fn test_symbol_table_19() {
        let src = "package org.javaohjavawhyareyouso
	interface endOfPlaylist {}
    interface endOfPlaylist {}
	";
        let publisher = parse(src).unwrap();
        publisher.print(Key(0), Some(true));
        let fmt = symbol_table_builder::SymbolTableBuilder::new(src, &publisher);
        let output = fmt.create_symbol_table();
        println!("Formatted:\n\n{:#?}", output.unwrap_err());
    }
    #[test]
    #[should_panic] // Temporary because parser will fail instead,
    fn test_symbol_table_20() {
        let src = "
        package org.javaohjavawhyareyouso
        package org.javaohjavawhyareyouso
        interface endOfPlaylist {}
        interface endOfPlaylist {}
	";
        let publisher = parse(src).unwrap();
        publisher.print(Key(0), Some(true));
        let fmt = symbol_table_builder::SymbolTableBuilder::new(src, &publisher);
        let output = fmt.create_symbol_table();
        println!("Formatted:\n\n{:#?}", output.unwrap_err());
    }
    #[test]
    fn test_formatter_21() {
        let src = r#"package org.javaohjavawhyareyouso
        <** @Annotation: block **>
        interface endOfPlaylist {

            version {
                major 25
                minor 60
            }
            <** @Annotation: block
                @WegotsMore: of these annations **>

            struct thing {
                <** @Annotation: block **>

                p1 p1
                p2 p2
            }
            <** @Annotation: block **>

            attribute uint8 thing
            <** @Annotation: block **>

            method thing {
                <** @Annotation: block **>

                in {
                    <** @Annotation: block **>

                    param param
                }
                <** @Annotation: block **>

                out {
                    

                    param2 param2
                    <** @Annotation: block **>
                    org.param3 param3
                }
            }
            <** @Annotation: block **>

            method thing {
                <** @Annotation: block **>

                in {
                    param param
                }
                <** @Annotation: block **>

                out {
                    param2 param2
                    org.param3 param3
                }
            }
            <** @Annotation: block **>

            typedef aTypedef is Int16
            <** @Annotation: block **>

            enumeration aEnum {
                A = 3
                B
                C
                D
                E = 10
            }
        
        }
        <** @Annotation: block **>

        typeCollection {
        
            typedef aTypedef is Int16
            enumeration aEnum {
                A = 3
                B
                C
                D
                E = 10
            }
        
        
            struct thing {
                p1 p1
                p2 p2
            }
        }"#;
        let publisher = parse(src).unwrap();
        publisher.print(Key(0), Some(true));
        let fmt = symbol_table_builder::SymbolTableBuilder::new(src, &publisher);
        let output = fmt.create_symbol_table();
        println!("Formatted:\n\n{:#?}", output.unwrap());
    }
    #[test]
    fn test_formatter_22() {
        let src = r#"package org.javaohjavawhyareyouso //Comment
        <** @Annotation: block **>//Comment
        //Comment
        interface endOfPlaylist {//Comment
            //Comment
            version {
                //Comment
                major 25//Comment
                minor 60//Comment
            }//Comment
            <** @Annotation: block//Comment
                @WegotsMore: of these annations **>//Comment
                //Comment
            struct thing {//Comment
                <** @Annotation: block **>//Comment
                //Comment
                p1 p1//Comment
                p2 p2//Comment
            }//Comment
            <** @Annotation: block **>//Comment
            //Comment
            attribute uint8 thing//Comment
            <** @Annotation: block **>//Comment
            //Comment
            method thing {//Comment
                <** @Annotation: block **>//Comment
                //Comment
                in {//Comment
                    <** @Annotation: block **>//Comment
                    //Comment
                    param param//Comment
                }//Comment
                <** @Annotation: block **>//Comment
                //Comment
                out {//Comment
                    
                    //Comment
                    param2 param2//Comment
                    <** @Annotation: block **>//Comment
                    org.param3 param3//Comment
                }//Comment
            }//Comment
            <** @Annotation: block **>//Comment
            //Comment
            method thing {//Comment
                <** @Annotation: block **>//Comment
                //Comment
                in {//Comment
                    param param//Comment
                }//Comment
                <** @Annotation: block **>//Comment
                //Comment
                out {//Comment
                    param2 param2//Comment
                    org.param3 param3//Comment
                }//Comment
            }//Comment
            <** @Annotation: block **>//Comment
            //Comment
            typedef aTypedef is Int16//Comment
            <** @Annotation: block **>//Comment
            //Comment
            enumeration aEnum {//Comment
                A = 3//Comment
                B//Comment
                //Comment
                C//Comment
                //Comment
                D//Comment
                E = 10//Comment
            }//Comment
            //Comment
        }//Comment
        <** @Annotation: block **>
        //Comment
        typeCollection {//Comment
            //Comment
            typedef aTypedef is Int16//Comment
            //Comment
            enumeration aEnum {
                A = 3//Comment
                B//Comment
                C

                //Comment
                D
                E = 10
            }
        
            //Comment
            struct thing {
                //Comment
                p1 p1//Comment
                //Comment
                p2 p2//Comment
            }
        }"#;
        let publisher = parse(src).unwrap();
        publisher.print(Key(0), Some(true));
        let fmt = symbol_table_builder::SymbolTableBuilder::new(src, &publisher);
        let output = fmt.create_symbol_table();
        println!("Formatted:\n\n{:#?}", output.unwrap());
    }


    #[test]
    fn test_formatter_23() {
        let src = r#"/** MultiLine Comment **/
        package org.javaohjavawhyareyouso /** MultiLine Comment **/
        <** @Annotation: block **>/** MultiLine Comment
        Tis a multi line comment indeed **/
        /** MultiLine Comment **/
        interface endOfPlaylist {/** MultiLine Comment **/
            /** MultiLine Comment **/
            version {
                /** MultiLine Comment **/
                major 25/** MultiLine Comment **/
                minor 60/** MultiLine Comment **/
            }/** MultiLine Comment **/
            <** @Annotation: block/** MultiLine Comment **/
                @WegotsMore: of these annations **>/** MultiLine Comment **/
                /** MultiLine Comment **/
            struct thing {/** MultiLine Comment **/
                <** @Annotation: block **>/** MultiLine Comment **/
                /** MultiLine Comment **/
                p1 p1/** MultiLine Comment **/
                p2 p2/** MultiLine Comment **/
            }/** MultiLine Comment **/
            <** @Annotation: block **>/** MultiLine Comment **/
            /** MultiLine Comment **/
            attribute uint8 thing/** MultiLine Comment **/
            <** @Annotation: block **>/** MultiLine Comment **/
            /** MultiLine Comment **/
            method thing2 {/** MultiLine Comment **/
                <** @Annotation: block **>/** MultiLine Comment **/
                /** MultiLine Comment **/
                in {/** MultiLine Comment **/
                    <** @Annotation: block **>/** MultiLine Comment **/
                    /** MultiLine Comment **/
                    param param/** MultiLine Comment **/
                }/** MultiLine Comment **/
                <** @Annotation: block **>/** MultiLine Comment **/
                /** MultiLine Comment **/
                out {/** MultiLine Comment **/
                    
                    /** MultiLine Comment **/
                    param2 param2/** MultiLine Comment **/
                    <** @Annotation: block **>/** MultiLine Comment **/
                    org.param3 param3/** MultiLine Comment **/
                }/** MultiLine Comment **/
            }/** MultiLine Comment **/
            <** @Annotation: block **>/** MultiLine Comment **/
            /** MultiLine Comment **/
            method thing {/** MultiLine Comment **/
                <** @Annotation: block **>/** MultiLine Comment **/
                /** MultiLine Comment **/
                in {/** MultiLine Comment **/
                    param param/** MultiLine Comment **/
                }/** MultiLine Comment **/
                <** @Annotation: block **>/** MultiLine Comment **/
                /** MultiLine Comment **/
                out {/** MultiLine Comment **/
                    param2 param2/** MultiLine Comment **/
                    org.param3 param3/** MultiLine Comment **/
                }/** MultiLine Comment **/
            }/** MultiLine Comment **/
            <** @Annotation: block **>/** MultiLine Comment **/
            /** MultiLine Comment **/
            typedef aTypedef is Int16/** MultiLine Comment **/
            <** @Annotation: block **>/** MultiLine Comment **/
            /** MultiLine Comment **/
            enumeration aEnum {/** MultiLine Comment **/
                A = 3/** MultiLine Comment **/
                B/** MultiLine Comment **/
                /** MultiLine Comment **/
                C/** MultiLine Comment **/
                /** MultiLine Comment **/
                D/** MultiLine Comment **/
                E = 10/** MultiLine Comment **/
            }/** MultiLine Comment **/
            /** MultiLine Comment **/
        }/** MultiLine Comment **/
        <** @Annotation: block **>
        /** MultiLine Comment **/
        typeCollection {/** MultiLine Comment **/
            /** MultiLine Comment **/
            typedef aTypedef is Int16/** MultiLine Comment **/
            /** MultiLine Comment **/
            enumeration aEnum {
                A = 3/** MultiLine Comment **/
                B/** MultiLine Comment **/
                C

                /** MultiLine Comment **/
                D
                E = 10
            }
        
            /** MultiLine Comment **/
            struct thing {
                /** MultiLine Comment **/
                p1 p1/** MultiLine Comment **/
                /** MultiLine Comment **/
                p2 p2/** MultiLine Comment **/
            }
        }"#;
        let publisher = parse(src).unwrap();
        publisher.print(Key(0), Some(true));
        let fmt = symbol_table_builder::SymbolTableBuilder::new(src, &publisher);
        let output = fmt.create_symbol_table();
        println!("Formatted:\n\n{:?}", output.unwrap());
    }


    fn test_formatter_24(){
        let src = r#"
        
        "#;
        let publisher = parse(src).unwrap();
        publisher.print(Key(0), Some(true));
        let fmt = symbol_table_builder::SymbolTableBuilder::new(src, &publisher);
        let output = fmt.create_symbol_table();
        println!("Formatted:\n\n{:?}", output.unwrap());
    }

}
