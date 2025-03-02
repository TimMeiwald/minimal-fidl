use std::fmt::format;

use crate::codegen_trait::CodeGenerator;
use crate::indented_string::IndentedString;
use crate::FidlType;
use minimal_fidl_collect::{
    attribute::{self, Attribute}, enumeration::Enumeration, fidl_file::FidlFile, interface::Interface, method::Method, structure::Structure, type_def::TypeDef, variable_declaration::VariableDeclaration, version::Version
};




pub struct RustCodeGen();
impl CodeGenerator for RustCodeGen {
    fn new() -> Self {
        Self {}
    }
    fn file(&self, file: &FidlFile) -> Vec<IndentedString> {
        let mut res: Vec<IndentedString> = Vec::new();
        for interface in &file.interfaces {
            let x = self.interface(&interface);
            res.extend(x);
        }
        res
    }

    fn version(&self, version: &Option<Version>) -> Vec<IndentedString> {
        let mut res: Vec<IndentedString> = Vec::new();
        match version {
            Some(version) => {
                res.push(IndentedString::new(
                    1,
                    FidlType::File,
                    format!(
                        "pub const VERSION_MAJOR: u32 = {:?}",
                        version.major.expect("Should exist")
                    ),
                ));
                res.push(IndentedString::new(
                    1,
                    FidlType::File,
                    format!(
                        "pub const VERSION_MINOR: u32 = {:?}",
                        version.minor.expect("Should exist")
                    ),
                ));
            }
            None => {
                res.push(IndentedString::new(
                    1,
                    FidlType::File,
                    format!("pub const VERSION_MAJOR: u32 = {:?}", 0),
                ));
                res.push(IndentedString::new(
                    1,
                    FidlType::File,
                    format!("pub const VERSION_MINOR: u32 = {:?}", 0),
                ));
            }
        }
        res
    }

    fn interface(&self, interface: &Interface) -> Vec<IndentedString> {
        let mut res: Vec<IndentedString> = Vec::new();
        // An interface is equivalent to a Rust Module
        let module = IndentedString::new(
            0,
            FidlType::File,
            format!("pub module {} {{", interface.name),
        );
        res.push(module);
        res.extend(self.version(&interface.version));
        for typedef in &interface.typedefs {
            let typedef: Vec<IndentedString> = self
                .typedef(typedef)
                .into_iter()
                .map(|e| e.indent())
                .collect();
            res.extend(typedef)
        }
        for attribute in &interface.attributes {
            let attr: Vec<IndentedString> = self
                .attribute(attribute)
                .into_iter()
                .map(|e| e.indent())
                .collect();
            res.extend(attr);
        }
        for method in &interface.methods {
            let method: Vec<IndentedString> = self
                .method(method)
                .into_iter()
                .map(|e| e.indent())
                .collect();
            res.extend(method)
        }
        for structure in &interface.structures {
            let structure: Vec<IndentedString> = self
                .structure(structure)
                .into_iter()
                .map(|e| e.indent())
                .collect();
            res.extend(structure)
        }
        for enumeration in &interface.enumerations {
            let enumeration: Vec<IndentedString> = self
                .enumeration(enumeration)
                .into_iter()
                .map(|e| e.indent())
                .collect();
            res.extend(enumeration)
        }

        let end_bracket = IndentedString::new(1, FidlType::Interface, format!("}}"));
        res.push(end_bracket);
        let end_bracket = IndentedString::new(0, FidlType::Interface, format!("}}"));
        res.push(end_bracket);
        res
    }
    fn attribute(&self, attribute: &Attribute) -> Vec<IndentedString> {
        vec![IndentedString::new(
            0,
            FidlType::File,
            "attribute Stub".to_string(),
        )]
    }
    fn structure(&self, structure: &Structure) -> Vec<IndentedString> {
        vec![IndentedString::new(
            0,
            FidlType::File,
            "Structure Stub".to_string(),
        )]
    }

    fn typedef(&self, typedef: &TypeDef) -> Vec<IndentedString> {
        vec![IndentedString::new(
            0,
            FidlType::File,
            format!("use {} as {};", typedef.type_n, typedef.name),
        )]
    }


    
    fn method(&self, method: &Method) -> Vec<IndentedString> {
        let mut input_params = "".to_string();
        for param in &method.input_parameters{
            input_params += &param.name; 
            input_params += ": ";
            input_params += &param.type_n;
            input_params += ", "
        }
        if input_params.len() != 0{
            input_params = input_params[0..input_params.len()-2].to_string();
        }

        let mut output_params = "".to_string();
        match method.output_parameters.len(){
            0 => {
                output_params += "()";
            }
            1 => {
                let single_param = &method.output_parameters[0];
                output_params = format!("{}", single_param.type_n);
            }
            e => {
                output_params.push('(');
                for param in &method.output_parameters{
                    output_params += &param.type_n;
                    output_params += ", "
                }
                output_params = output_params[0..output_params.len()-2].to_string();
                output_params.push(')');

            }
        }
        let mut res: Vec<IndentedString> = Vec::new();
        res.push(IndentedString::new(
            0,
            FidlType::Method,
            format!("fn {}({}) -> {} {{", method.name, input_params, output_params).to_string(),
        ));


        res
    }
    fn enumeration(&self, enumeration: &Enumeration) -> Vec<IndentedString> {
        vec![IndentedString::new(
            0,
            FidlType::File,
            "enum Stub".to_string(),
        )]
    }
}
