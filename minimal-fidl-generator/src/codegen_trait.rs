use std::fmt::Debug;

use minimal_fidl_collect::{
    attribute::{self, Attribute}, enumeration::Enumeration, fidl_file::FidlFile, interface::Interface, method::Method, structure::Structure, type_def::TypeDef, version::Version
};
use crate::indented_string::IndentedString;
pub trait CodeGenerator {
    fn new() -> Self;
    fn file(&self, file: &FidlFile) -> Vec<IndentedString>;
    fn interface(&self, interface: &Interface) -> Vec<IndentedString>;
    fn method(&self, method: &Method) -> Vec<IndentedString>;
    fn enumeration(&self, enumeration: &Enumeration) -> Vec<IndentedString>;
    fn attribute(&self, attribute: &Attribute) -> Vec<IndentedString>;
    fn structure(&self, structure: &Structure) -> Vec<IndentedString>;
    fn typedef(&self, typedef: &TypeDef) -> Vec<IndentedString>;
    fn version(&self, version: &Option<Version>) -> Vec<IndentedString>;

}

