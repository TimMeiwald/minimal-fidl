use std::{fmt::Debug, path::PathBuf};

use minimal_fidl_collect::{
    attribute::{self, Attribute}, enumeration::Enumeration, fidl_file::FidlFile, interface::Interface, method::Method, structure::Structure, type_collection::TypeCollection, type_def::TypeDef, version::Version, FidlProject, FileError
};
use crate::indented_string::IndentedString;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GeneratorError {
    #[error["This error means the program has a bug: {0}"]]
    InternalLogicError(String),
    #[error["Could not generate code. {:?}", 0]]
    CouldNotGeneratCodeForFile(FidlFile),
    #[error["{:?}", 0]]
    FidlFileError(#[from] FileError),
    #[error["{:?}", 0]]
    IoError(#[from] std::io::Error)



}
pub trait CodeGenerator {
    fn new() -> Self;
    // fn generate_file(&mut self, path: PathBuf, fidl: FidlFile) -> Result<(), GeneratorError>;
    /// Convenience function if you don't want to filter the files at all. Otherwise use FidlProject to generate each file manually then call
    /// CodeGenerator::generate_file instead for each you want to use. 
    fn generate_project(&mut self,  dir: PathBuf) -> Result<(), GeneratorError>;
    fn emit_project(&self, target_dir: PathBuf) -> Result<(), GeneratorError>;
}

