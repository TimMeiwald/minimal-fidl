use std::fmt::Debug;

use minimal_fidl_collect::fidl_file::FileError;
use minimal_fidl_collect::{
    enumeration::Enumeration, fidl_file::FidlFile, interface::Interface, method::Method,
};
use minimal_fidl_parser::BasicPublisher;
mod codegen_js;
mod codegen_py;
mod codegen_rust;
mod codegen_trait;
mod indented_string;
use codegen_rust::RustCodeGen;
use codegen_trait::CodeGenerator;
use indented_string::FidlType;
use indented_string::IndentedString;

#[cfg(test)]
mod tests {
    use minimal_fidl_collect::{FidlFile, FidlProject};
    use minimal_fidl_parser::{
        BasicContext, BasicPublisher, Context, Key, Rules, Source, _var_name, grammar, RULES_SIZE,
    };
    use std::{
        cell::RefCell,
        fs::remove_dir_all,
        io::{stdout, Write},
        path::{Path, PathBuf},
    };

    use crate::{codegen_py::PythonCodeGen, CodeGenerator, RustCodeGen};

    // pub fn parse(input: &str) -> Option<BasicPublisher> {
    //     let string = input.to_string();
    //     let src_len = string.len() as u32;
    //     let source = Source::new(&string);
    //     let position: u32 = 0;
    //     let result: (bool, u32);
    //     let context = RefCell::new(BasicContext::new(src_len as usize, RULES_SIZE as usize));
    //     {
    //         let executor = _var_name(Rules::Grammar, &context, grammar);
    //         result = executor(Key(0), &source, position);
    //     }
    //     if result != (true, src_len) {
    //         println!("Failed with : {:?}", result);
    //         return None;
    //     }

    //     let publisher = context.into_inner().get_publisher().clear_false();
    //     Some(publisher)
    // }

    // #[test]
    // fn test_generator_1() {
    //     let src = "package org.javaohjavawhyareyouso
    // interface endOfPlaylist { }
    // interface endOfPlaylist2 { }	";
    //     let file = FidlProject::generate_file_from_string(src.to_string()).unwrap();
    //     let mut codegen = PythonCodeGen::new();
    //     codegen.generate_file(PathBuf::new(), file).unwrap();
    //     println!("{:?}", codegen);
    // }
    // #[test]
    // fn test_generator_2() {
    //     let src = "package org.javaohjavawhyareyouso
    // interface endOfPlaylist { }	";
    //     let file = FidlProject::generate_file_from_string(src.to_string()).unwrap();
    //     let mut codegen = PythonCodeGen::new();
    //     codegen.generate_file(PathBuf::new(), file).unwrap();
    //     println!("{:?}", codegen);
    // }
    // #[test]
    // fn test_generator_3() {
    //     let src = r#"package org.javaohjavawhyareyouso
    //     <** @Annotation: block **>
    //     interface endOfPlaylist {
    //         version {
    //             major 25
    //             minor 60
    //         }
    //         <** @Annotation: block
    //             @WegotsMore: of these annations **>

    //         struct thing {
    //             <** @Annotation: block **>

    //             p1 p1
    //             p2 p2
    //         }
    //         <** @Annotation: block **>

    //         attribute uint8 thing
    //         <** @Annotation: block **>

    //         method thing {
    //             <** @Annotation: block **>

    //             in {
    //                 <** @Annotation: block **>

    //                 Param param
    //             }
    //             <** @Annotation: block **>

    //             out {

    //                 Param2 param2
    //                 <** @Annotation: block **>
    //                 org.param3 param3
    //             }
    //         }
    //         <** @Annotation: block **>

    //         method thing2 {
    //             <** @Annotation: block **>

    //             in {
    //                 param2 param2

    //                 param param
    //             }
    //             <** @Annotation: block **>

    //             out {
    //                 param2 param2
    //                 org.param3 param3
    //             }
    //         }
    //         <** @Annotation: block **>

    //         typedef aTypedef is Int16
    //         <** @Annotation: block **>

    //         enumeration aEnum {
    //             A = 3
    //             B
    //             C
    //             D
    //             E = 10
    //         }

    //     }
    //     <** @Annotation: block **>

    //     typeCollection MustHaveName {

    //         typedef aTypedef is Int16
    //         enumeration aEnum {
    //             A = 3
    //             B
    //             C
    //             D
    //             E = 10
    //         }

    //         struct thing {
    //             p1 p1
    //             p2 p2
    //         }
    //     }"#;
    //     let file = FidlProject::generate_file_from_string(src.to_string()).unwrap();
    //     let mut codegen = PythonCodeGen::new();
    //     codegen.generate_file(PathBuf::new(), file).unwrap();
    //     println!("{:?}", codegen);
    // }

    // #[test]
    // fn test_generator_4() {
    //     let src = r#"
    //     package org.reference
    //     <**
    //         @description:
    //             This reference type collection uses all kinds of type definitions
    //             which can be done within one type collection.
    //     **>
    //     typeCollection MyTypeCollection10 {

    //         // struct with all basic types
    //         struct MyStruct01 {
    //             Int8 se01
    //             UInt8 se02
    //             Int16 se03
    //             UInt16 se04
    //             Int32 se05
    //             UInt32 se06
    //             Int64 se07
    //             UInt64 se08
    //             Boolean se09
    //             String se10
    //             ByteBuffer se11
    //         }

    //         // struct for checking alignment/padding
    //         struct MyStruct02 {
    //             UInt8 se01
    //             UInt32 se02
    //             UInt8 se03
    //             UInt8 se04
    //             UInt32 se05
    //             UInt8 se06
    //             UInt8 se07
    //             UInt8 se08
    //             UInt32 se09
    //         }

    //         // struct of arrays
    //         struct MyStruct04 {
    //             MyArray05 se01
    //             MyArray20 se02
    //             MyArray30 se03
    //         }
    //     FidlGenerator::new(src, &publisher, JSCodeGen::new()).unwrap();
    //         // struct with elements of implicit array type
    //         struct MyStruct05 {
    //             UInt8[] se01
    //             String[] se02
    //             ByteBuffer[] se03
    //             MyArray01[] se10
    //             MyStruct02[] se11
    //             MyEnum03[] se12
    //         }

    //         // struct of enums
    //         struct MyStruct06 {
    //             MyEnum01 se01
    //             MyEnum02 se02
    //             MyEnum03 se03
    //             MyEnum10 se10
    //         }

    //         // struct of maps and typedefs
    //         struct MyStruct08 {
    //             MyMap05 se01
    //             MyMap08 se02
    //             MyType01 se03
    //             MyType03 se04
    //         }

    //         // empty enumeration
    //         enumeration MyEnum01 {
    //             ENUM00
    //         }

    //         // enumeration without values
    //         enumeration MyEnum02 {
    //             ENUM01
    //             ENUM02
    //             ENUM03
    //         }

    //         // enumeration with values
    //         enumeration MyEnum03 {
    //             ENUM01 = 1
    //             ENUM02
    //             ENUM03 = 10
    //             ENUM04 = 7
    //             ENUM05 = 20
    //             ENUM06 = 0x20
    //         }

    //         // typedefs from basic types
    //         typedef MyType01 is UInt16
    //         typedef MyType02 is String
    //         typedef MyType03 is Double
    //         typedef MyType04 is ByteBuffer
    //         // typedefs from user-defined types
    //         typedef MyType10 is MyArray10
    //         typedef MyType11 is MyStruct01
    //         typedef MyType12 is MyStruct10
    //         typedef MyType13 is MyUnion03
    //         // typedefs from other typedefs
    //         typedef MyType20 is MyType01
    //         typedef MyType21 is MyType04
    //         typedef MyType22 is MyType10
    //         typedef MyType23 is MyType12
    //     }"#;
    //     let file = FidlProject::generate_file_from_string(src.to_string()).unwrap();
    //     let mut codegen = PythonCodeGen::new();
    //     codegen.generate_file(PathBuf::new(), file).unwrap();
    //     println!("{:?}", codegen);
    // }

    // #[test]
    // fn test_generator_5() {
    //     let src = "package org.javaohjavawhyareyouso
    // interface MyInterface {
    //     typedef CustomDouble is Double
    //     attribute UInt8 some_value

    //     struct ThingStruct {
    //         UInt16 some_value
    //         Float some_value2
    //     }
    //     enumeration aEnum {
    //         A
    //         B
    //         C
    //         D
    //         E
    //     }
    //     method thing {
    //         in {
    //             ThingStruct param
    //         }
    //         out {

    //             CustomDouble param2
    //             Double param3
    //         }
    //     }

    //  }";
    //     let file = FidlProject::generate_file_from_string(src.to_string()).unwrap();
    //     let mut codegen = PythonCodeGen::new();
    //     codegen.generate_file(PathBuf::new(), file).unwrap();
    //     println!("{:?}", codegen);
    // }

    // #[test]
    // fn test_generator_6() {
    //     let src = "package org.javaohjavawhyareyouso
    // interface MyInterface {
    //     typedef CustomDouble is Double
    //     attribute UInt8 some_value

    //     struct ThingStruct {
    //         UInt16 some_value
    //         Float some_value2
    //     }
    //     enumeration aEnum {
    //         A
    //         B
    //         C
    //         D
    //         E
    //     }
    //     method thing {
    //         in {
    //             ThingStruct param
    //         }
    //         out {

    //             CustomDouble param2
    //             Double param3
    //         }
    //     }

    //  }";
    //     let file = FidlProject::generate_file_from_string(src.to_string()).unwrap();
    //     let mut codegen = PythonCodeGen::new();
    //     codegen.generate_file(PathBuf::new(), file).unwrap();
    //     println!(r#"{}"#, codegen.python_code[&PathBuf::new()][0]);
    // }

    #[test]
    fn test_generator_7() -> Result<(), std::io::Error> {
        let src = Path::new("tests/");
        let mut codegen = PythonCodeGen::new();
        codegen.generate_project(src.into()).unwrap();
        println!("{:?}", codegen);
        stdout().flush()?;
        //  println!(r#"{}"#, codegen.python_code[&PathBuf::new()][0]);
        let mut path = PathBuf::new();
        path.push("tests");
        path.push("test_python_code");
        remove_dir_all("tests/test_python_code")?;
        codegen.emit_project(path).unwrap();
        Ok(())
    }
}
