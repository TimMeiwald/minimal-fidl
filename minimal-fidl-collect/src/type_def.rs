use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use crate::{annotation::{annotation_constructor, Annotation}, fidl_file::FileError, type_ref::TypeRef, VariableDeclaration};
use minimal_fidl_parser::{BasicPublisher, Key, Node, Rules};
#[derive(Debug, Clone)]
pub struct TypeDef {
    start_position: u32,
    end_position: u32,
    pub annotations: Vec<Annotation>,
    pub name: String,
    pub type_n: String,
    pub is_array: bool,
}
impl TypeDef {
    pub fn new(source: &str, publisher: &BasicPublisher, node: &Node) -> Result<Self, FileError> {
        debug_assert_eq!(node.rule, Rules::typedef);
        let mut name: Result<String, FileError> = Err(FileError::InternalLogicError(
            "Uninitialized value: name in TypeDef::new".to_string(),
        ));
        let mut type_n: Result<String, FileError> = Err(FileError::InternalLogicError(
            "Uninitialized value: name in TypeDef::new".to_string(),
        ));
        let mut annotations: Vec<Annotation> = Vec::new();

        let mut is_array = false;
        for child in node.get_children() {
            let child = publisher.get_node(*child);
            match child.rule {
                Rules::comment | Rules::multiline_comment => {}
                Rules::annotation_block => {
                    annotations = annotation_constructor(source, publisher, child)?;
                }
                Rules::type_dec => {
                    println!("Need to actually do this stuff. Types need to be checked for duplicates and whether they exist if using external import after reading file.");
                    name = Ok(child.get_string(source))
                }
                Rules::type_ref => {
                    let res = TypeRef::new(source, publisher, child)?;
                    is_array = res.is_array;
                    type_n = Ok(res.name);
                }
                rule => {
                    return Err(FileError::UnexpectedNode(rule, "TypeDef::new".to_string()));
                }
            }
        }
        Ok(Self {
            name: name?,
            type_n: type_n?,
            is_array: is_array,
            annotations,
            start_position: node.start_position,
            end_position: node.end_position,
        })
    }

    pub fn push_if_not_exists_else_err(self, typedefs: &mut Vec<TypeDef>) -> Result<(), FileError> {
        for t in &mut *typedefs {
            if t.name == self.name {
                return Err(FileError::TypeDefAlreadyExists(t.clone(), self.clone()));
            }
        }
        typedefs.push(self);
        Ok(())
    }
}
