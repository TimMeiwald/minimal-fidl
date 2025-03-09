use std::path::PathBuf;

use crate::indented_string::IndentedString;
use crate::{codegen_trait::CodeGenerator, indented_string::FidlType};
use minimal_fidl_collect::attribute::{self, Attribute};
use minimal_fidl_collect::structure::Structure;
use minimal_fidl_collect::type_collection::TypeCollection;
use minimal_fidl_collect::type_def::TypeDef;
use minimal_fidl_collect::version::Version;
use minimal_fidl_collect::{
    enumeration::Enumeration, fidl_file::FidlFile, interface::Interface, method::Method,
};

pub struct JSCodeGen();
impl CodeGenerator for JSCodeGen {
    fn new() -> Self {
        Self {}
    }
    fn project(&self, dir: &PathBuf) -> Vec<IndentedString> {
        let mut res: Vec<IndentedString> = Vec::new();
        res.push(IndentedString::new(0, FidlType::Method,"JS Project stub".to_string()));
        res
    }
    fn file(&self, file: &FidlFile) -> Vec<IndentedString> {
        vec![IndentedString::new(0, FidlType::File, "JS File Stub".to_string())]
    }
    fn interface(&self, interface: &Interface) -> Vec<IndentedString> {
        vec![IndentedString::new(0, FidlType::Interface, "JS Interface Stub".to_string())]
    }
    fn method(&self, method: &Method) -> Vec<IndentedString> {
        vec![IndentedString::new(0, FidlType::Method, "JS Method Stub".to_string())]
    }
    fn enumeration(&self, enumeration: &Enumeration, public: bool) -> Vec<IndentedString> {
        vec![IndentedString::new(0, FidlType::Enumeration, "JS Enumeration Stub".to_string())]
    }
    fn attribute(&self, attribute: &Attribute) -> Vec<IndentedString> {
        vec![IndentedString::new(0, FidlType::Attribute, "JS Enumeration Stub".to_string())]
    }
    fn structure(&self, structure: &Structure, public: bool) -> Vec<IndentedString> {
        vec![IndentedString::new(0, FidlType::Attribute, "JS Structure Stub".to_string())]

    }
    fn typedef(&self, structure: &TypeDef, public: bool) -> Vec<IndentedString> {
        vec![IndentedString::new(0, FidlType::Attribute, "JS Typedef Stub".to_string())]

    }
    fn version(&self, version: &Option<Version>) -> Vec<IndentedString> {
        vec![IndentedString::new(0, FidlType::Attribute, "JS Version Stub".to_string())]

    }
    fn type_collection(&self, type_collection: &TypeCollection) -> Vec<IndentedString>{
        vec![IndentedString::new(0, FidlType::TypeCollection, "JS TypeCollec Stub".to_string())]
    }

}
