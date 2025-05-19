use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use crate::{annotation::{annotation_constructor, Annotation}, fidl_file::FileError, VariableDeclaration};
use minimal_fidl_parser::{BasicPublisher, Key, Node, Rules};
#[derive(Debug, Clone)]
pub struct Attribute {
    start_position: u32,
    end_position: u32,
    pub annotations: Vec<Annotation>,
    pub name: String,
    pub type_n: String,
}
impl Attribute {
    pub fn new(source: &str, publisher: &BasicPublisher, node: &Node) -> Result<Self, FileError> {
        debug_assert_eq!(node.rule, Rules::attribute);
        let mut name: Result<String, FileError> = Err(FileError::InternalLogicError(
            "Uninitialized value: name in Attribute::new".to_string(),
        ));
        let mut type_n: Result<String, FileError> = Err(FileError::InternalLogicError(
            "Uninitialized value: name in Attribute::new".to_string(),
        ));
        let mut annotations: Vec<Annotation> = Vec::new();
        for child in node.get_children() {
            let child = publisher.get_node(*child);
            match child.rule {
                Rules::comment | Rules::multiline_comment => {}
                Rules::annotation_block => {
                    annotations = annotation_constructor(source, publisher, node)?;
                }
                Rules::type_ref => {
                    type_n = Ok(child.get_string(source));
                }
                Rules::variable_name => name = Ok(child.get_string(source)),

                rule => {
                    return Err(FileError::UnexpectedNode(
                        rule,
                        "Attribute::new".to_string(),
                    ));
                }
            }
        }
        Ok(Self {
            name: name?,
            type_n: type_n?,
            annotations,
            start_position: node.start_position,
            end_position: node.end_position,
        })
    }
    pub fn push_if_not_exists_else_err(
        self,
        attributes: &mut Vec<Attribute>,
    ) -> Result<(), FileError> {
        for attr in &mut *attributes {
            if attr.name == self.name {
                return Err(FileError::AttributeAlreadyExists(
                    attr.clone(),
                    self.clone(),
                ));
            }
        }
        attributes.push(self);
        Ok(())
    }
}
