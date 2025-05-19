use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use crate::{annotation::{annotation_constructor, Annotation}, enum_value::EnumValue, fidl_file::FileError, VariableDeclaration};
use minimal_fidl_parser::{BasicPublisher, Key, Node, Rules};
#[derive(Debug, Clone)]
pub struct Enumeration {
    start_position: u32,
    end_position: u32,
    pub annotations: Vec<Annotation>,
    pub name: String,
    pub values: Vec<EnumValue>,
}
impl Enumeration {
    pub fn new(source: &str, publisher: &BasicPublisher, node: &Node) -> Result<Self, FileError> {
        debug_assert_eq!(node.rule, Rules::enumeration);
        let mut name: Result<String, FileError> = Err(FileError::InternalLogicError(
            "Uninitialized value: name in Enumeration::new".to_string(),
        ));
        let mut annotations: Vec<Annotation> = Vec::new();

        let mut values: Vec<EnumValue> = Vec::new();
        for child in node.get_children() {
            let child = publisher.get_node(*child);
            match child.rule {
                Rules::comment
                | Rules::multiline_comment
                | Rules::open_bracket
                | Rules::close_bracket => {}
                Rules::annotation_block => {
                    annotations = annotation_constructor(source, publisher, child)?;
                }
                Rules::type_dec => {
                    name = Ok(child.get_string(source));
                }
                Rules::enum_value => {
                    let enum_val = EnumValue::new(source, publisher, child)?;
                    enum_val.push_if_not_exists_else_err(&mut values)?;
                }

                rule => {
                    return Err(FileError::UnexpectedNode(
                        rule,
                        "Enumeration::new".to_string(),
                    ));
                }
            }
        }
        Ok(Self {
            name: name?,
            values,
            annotations,
            start_position: node.start_position,
            end_position: node.end_position,
        })
    }

    pub fn push_if_not_exists_else_err(
        self,
        Enumerations: &mut Vec<Enumeration>,
    ) -> Result<(), FileError> {
        for s in &mut *Enumerations {
            if s.name == self.name {
                return Err(FileError::EnumerationAlreadyExists(s.clone(), self.clone()));
            }
        }
        Enumerations.push(self);
        Ok(())
    }
}
