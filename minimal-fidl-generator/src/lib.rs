use std::fmt::Debug;


use minimal_fidl_collect::{enumeration::Enumeration, fidl_file::FidlFile, interface::Interface, method::Method};
use minimal_fidl_parser::BasicPublisher;
use minimal_fidl_collect::fidl_file::FileError;
mod codegen_trait;
mod codegen_js;
mod codegen_rust;
mod indented_string;
use indented_string::IndentedString;
use codegen_trait::CodeGenerator;
use codegen_rust::RustCodeGen;
use codegen_js::JSCodeGen;
use indented_string::FidlType;

struct FidlGenerator<'a>(FidlFile<'a>);
impl<'a> FidlGenerator<'a> {
    pub fn new(source: &'a str, publisher: &'a BasicPublisher, codegen: impl CodeGenerator) -> Result<Self, FileError> {

        let mut resp = FidlFile::new(source, publisher);
        match resp {
            Ok(resp) => {
                let res = codegen.file(&resp);
                let mut result: String = "".to_string();
                for line in res{
                    result += &line.to_string();
                }
                println!("{result}");

                let fidl_gen = FidlGenerator{0: resp};
                Ok(fidl_gen)

            },
            Err(err) => Err(err),
        }
    }



}
#[cfg(test)]
mod tests {
    use minimal_fidl_collect::fidl_file:: FidlFile;
    use minimal_fidl_parser::{
        BasicContext, BasicPublisher, Context, Key, Rules, Source, _var_name, grammar, RULES_SIZE,
    };
    use std::cell::RefCell;

    use crate::{codegen_js::JSCodeGen, CodeGenerator, FidlGenerator, RustCodeGen};

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
    fn test_generator_1() {
        let src = "package org.javaohjavawhyareyouso
	interface endOfPlaylist { }
    interface endOfPlaylist2 { }	";
        let publisher = parse(src).unwrap();
        //        publisher.print(Key(0), Some(true));
        let codegen = FidlGenerator::new(src, &publisher, RustCodeGen::new()).unwrap();
    }
    #[test]
    fn test_generator_2() {
        let src = "package org.javaohjavawhyareyouso
	interface endOfPlaylist { }	";
        let publisher = parse(src).unwrap();
        //        publisher.print(Key(0), Some(true));
        let codegen = FidlGenerator::new(src, &publisher, JSCodeGen::new()).unwrap();
    }
    #[test]
    fn test_generator_3() {
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

                    Param param
                }
                <** @Annotation: block **>

                out {
                    

                    Param2 param2
                    <** @Annotation: block **>
                    org.param3 param3
                }
            }
            <** @Annotation: block **>

            method thing2 {
                <** @Annotation: block **>

                in {
                    param2 param2

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
        let codegen = FidlGenerator::new(src, &publisher, RustCodeGen::new()).unwrap();
    }
}