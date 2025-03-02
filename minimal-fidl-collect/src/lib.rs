pub mod attribute;
pub mod enum_value;
pub mod enumeration;
pub mod fidl_file;
pub mod import_model;
pub mod import_namespace;
pub mod interface;
pub mod method;
pub mod package;
pub mod structure;
pub mod type_collection;
pub mod type_def;
pub mod variable_declaration;
pub mod version;
use fidl_file::FidlFile;
use attribute::Attribute;
use enum_value::EnumValue;
use enumeration::Enumeration;
use import_model::ImportModel;
use import_namespace::ImportNamespace;
use interface::Interface;
use method::Method;
use package::Package;
use structure::Structure;
use type_collection::TypeCollection;
use type_def::TypeDef;
use variable_declaration::VariableDeclaration;
use version::Version;

#[cfg(test)]
mod tests {
    use crate::{FidlFile};
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
        //        publisher.print(Key(0), Some(true));
        let fmt = FidlFile::new(src, &publisher);
        let output = fmt;
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
        //        publisher.print(Key(0), Some(true));
        let fmt = FidlFile::new(src, &publisher);
        let output = fmt;
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
        //        publisher.print(Key(0), Some(true));
        let fmt = FidlFile::new(src, &publisher);
        let output = fmt;
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
        //        publisher.print(Key(0), Some(true));
        let fmt = FidlFile::new(src, &publisher);
        let output = fmt;
        println!("Formatted:\n\n{:#?}", output.unwrap());
    }
    #[test]
    fn test_symbol_table_5() {
        let src = "package org.javaohjavawhyareyouso
	interface endOfPlaylist {  version {major 25 minor 60}}
    interface endOfPlaylist {  version {major 23 minor 40}}";
        let publisher = parse(src).unwrap();
        //        publisher.print(Key(0), Some(true));
        let fmt = FidlFile::new(src, &publisher);
        let output = fmt;
        let err = output.unwrap_err();
        println!("Err: {:?}", err)
    }

    #[test]
    fn test_symbol_table_6() {
        let src = "package org.javaohjavawhyareyouso
	interface endOfPlaylist {  version {major 25 minor 60}struct thing{p1 p1 p2 p2}
}   ";
        let publisher = parse(src).unwrap();
        //        publisher.print(Key(0), Some(true));
        let fmt = FidlFile::new(src, &publisher);
        let output = fmt;
        println!("Formatted:\n\n{:#?}", output.unwrap());
    }
    #[test]
    fn test_symbol_table_7() {
        let src = "package org.javaohjavawhyareyouso
	interface endOfPlaylist {  version {major 25 minor 60}struct thing{p1 p1 p2 p1}
}   ";
        let publisher = parse(src).unwrap();
        //        publisher.print(Key(0), Some(true));
        let fmt = FidlFile::new(src, &publisher);
        let output = fmt;
        println!("Formatted:\n\n{:#?}", output.unwrap_err());
    }

    #[test]
    fn test_symbol_table_8() {
        let src = "package org.javaohjavawhyareyouso
	interface endOfPlaylist {  version {major 25 minor 60}struct thing{p1 p1 p2 p2}struct thing{}
}   ";
        let publisher = parse(src).unwrap();
        //        publisher.print(Key(0), Some(true));
        let fmt = FidlFile::new(src, &publisher);
        let output = fmt;
        println!("Formatted:\n\n{}", output.unwrap_err());
    }

    #[test]
    fn test_symbol_table_9() {
        let src = "package org.javaohjavawhyareyouso
	interface endOfPlaylist {  version {major 25 minor 60}struct thing{p1 p1 p2 p2}struct thing2{}attribute uint8 thing
}   ";
        let publisher = parse(src).unwrap();
        //        publisher.print(Key(0), Some(true));
        let fmt = FidlFile::new(src, &publisher);
        let output = fmt;
        println!("Formatted:\n\n{:#?}", output.unwrap());
    }

    #[test]
    fn test_symbol_table_10() {
        let src = "package org.javaohjavawhyareyouso
	interface endOfPlaylist {  version {major 25 minor 60}struct thing{p1 p1 p2 p2}struct thing2{}attribute uint8 thing
attribute uint16 thing2}   ";
        let publisher = parse(src).unwrap();
        //        publisher.print(Key(0), Some(true));
        let fmt = FidlFile::new(src, &publisher);
        let output = fmt;
        println!("Formatted:\n\n{:#?}", output.unwrap());
    }

    #[test]
    fn test_symbol_table_11() {
        let src = "package org.javaohjavawhyareyouso
	interface endOfPlaylist {  version {major 25 minor 60}struct thing{p1 p1 p2 p2}struct thing2{}attribute uint8 thing
attribute uint16 thing}   ";
        let publisher = parse(src).unwrap();
        //        publisher.print(Key(0), Some(true));
        let fmt = FidlFile::new(src, &publisher);
        let output = fmt;
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
        //        publisher.print(Key(0), Some(true));
        let fmt = FidlFile::new(src, &publisher);
        let output = fmt;
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
        //        publisher.print(Key(0), Some(true));
        let fmt = FidlFile::new(src, &publisher);
        let output = fmt;
        println!("Formatted:\n\n{:#?}", output.unwrap_err());
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
        //        publisher.print(Key(0), Some(true));
        let fmt = FidlFile::new(src, &publisher);
        let output = fmt;
        println!("Formatted:\n\n{:#?}", output.unwrap());
    }
    #[test]
    fn test_symbol_table_19() {
        let src = "package org.javaohjavawhyareyouso
	interface endOfPlaylist {}
    interface endOfPlaylist {}
	";
        let publisher = parse(src).unwrap();
        //        publisher.print(Key(0), Some(true));
        let fmt = FidlFile::new(src, &publisher);
        let output = fmt;
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
        //        publisher.print(Key(0), Some(true));
        let fmt = FidlFile::new(src, &publisher);
        let output = fmt;
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

            method thing2 {
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
        //        publisher.print(Key(0), Some(true));
        let fmt = FidlFile::new(src, &publisher);
        let output = fmt;
        //println!("Formatted:\n\n{:#?}", output.unwrap());
        output.unwrap();
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
            method thing2 {//Comment
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
        //        publisher.print(Key(0), Some(true));
        let fmt = FidlFile::new(src, &publisher);
        let output = fmt;
        //println!("Formatted:\n\n{:#?}", output.unwrap());
        output.unwrap();
    }

    #[test]
    fn test_symbol_table_23() {
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
        //        publisher.print(Key(0), Some(true));
        let fmt = FidlFile::new(src, &publisher);
        let output = fmt;
        //println!("Formatted:\n\n{:?}", output.unwrap());
        output.unwrap();
    }
    #[test]
    fn test_symbol_table_24() {
        let src = r#"
        package org.javaohjavawhyareyouso
        interface name {
            enumeration aEnum {
                A = 3
                B = 0x004000
                C = 0b0101001
                D
                E = 10
            }
        }"#;
        let publisher = parse(src).unwrap();
        //        publisher.print(Key(0), Some(true));
        let fmt = FidlFile::new(src, &publisher);
        let output = fmt;
        println!("Formatted:\n\n{:#?}", output.unwrap());
    }

    #[test]
    fn test_symbol_table_25() {
        let src = r#"
        /**
    *****************************************************************************
    * Copyright (c) 2013 itemis AG (http://www.itemis.de).
    * All rights reserved. This program and the accompanying materials
    * are made available under the terms of the Eclipse Public License v1.0
    * which accompanies this distribution, and is available at
    * http://www.eclipse.org/legal/epl-v10.html
    *****************************************************************************
**/
package org.reference
<**
    @description:
        This reference type collection uses all kinds of type definitions
        which can be done within one type collection.
**>
typeCollection MyTypeCollection10 {

    // struct with all basic types
    struct MyStruct01 {
        Int8 se01
        UInt8 se02
        Int16 se03
        UInt16 se04
        Int32 se05
        UInt32 se06
        Int64 se07
        UInt64 se08
        Boolean se09
        String se10
        ByteBuffer se11
    }

    // struct for checking alignment/padding
    struct MyStruct02 {
        UInt8 se01
        UInt32 se02
        UInt8 se03
        UInt8 se04
        UInt32 se05
        UInt8 se06
        UInt8 se07
        UInt8 se08
        UInt32 se09
    }

    // struct of arrays
    struct MyStruct04 {
        MyArray05 se01
        MyArray20 se02
        MyArray30 se03
    }

    // struct with elements of implicit array type
    struct MyStruct05 {
        UInt8[] se01
        String[] se02
        ByteBuffer[] se03
        MyArray01[] se10
        MyStruct02[] se11
        MyEnum03[] se12
    }

    // struct of enums
    struct MyStruct06 {
        MyEnum01 se01
        MyEnum02 se02
        MyEnum03 se03
        MyEnum10 se10
    }

    // struct of maps and typedefs
    struct MyStruct08 {
        MyMap05 se01
        MyMap08 se02
        MyType01 se03
        MyType03 se04
    }

    // empty enumeration
    enumeration MyEnum01 {
        ENUM00
    }

    // enumeration without values
    enumeration MyEnum02 {
        ENUM01
        ENUM02
        ENUM03
    }

    // enumeration with values
    enumeration MyEnum03 {
        ENUM01 = 1
        ENUM02
        ENUM03 = 10
        ENUM04 = 7
        ENUM05 = 20
        ENUM06 = 0x20
    }

    // typedefs from basic types
    typedef MyType01 is UInt16
    typedef MyType02 is String
    typedef MyType03 is Double
    typedef MyType04 is ByteBuffer
    // typedefs from user-defined types
    typedef MyType10 is MyArray10
    typedef MyType11 is MyStruct01
    typedef MyType12 is MyStruct10
    typedef MyType13 is MyUnion03
    // typedefs from other typedefs
    typedef MyType20 is MyType01
    typedef MyType21 is MyType04
    typedef MyType22 is MyType10
    typedef MyType23 is MyType12
}"#;
        let publisher = parse(src).unwrap();
        //        publisher.print(Key(0), Some(true));
        let fmt = FidlFile::new(src, &publisher);
        let output = fmt;
        println!("Formatted:\n\n{:#?}", output.unwrap());
    }
}
