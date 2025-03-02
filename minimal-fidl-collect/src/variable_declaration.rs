use std::{path::{Path, PathBuf}, str::FromStr};

use crate::fidl_file::FileError;
use minimal_fidl_parser::{BasicPublisher, Key, Node, Rules};
#[derive(Debug, Clone)]
pub struct VariableDeclaration {
    start_position: u32,
    end_position: u32,
    pub type_n: String,
    pub name: String,

}
impl VariableDeclaration {
    pub fn new(
        source: &str,
        publisher: &BasicPublisher,
        node: &Node,
    ) -> Result<Self, FileError> {
        debug_assert_eq!(node.rule, Rules::variable_declaration);
        let mut type_n: Result<String, FileError> = Err(
            FileError::InternalLogicError("Uninitialized value: type_n in VariableDeclaration::new".to_string()),
        );
        let mut name: Result<String, FileError> = Err(
            FileError::InternalLogicError("Uninitialized value: name in VariableDeclaration::new".to_string()),
        );

        for child in node.get_children() {
            let child = publisher.get_node(*child);
            match child.rule {
                Rules::comment | Rules::multiline_comment | Rules::annotation_block => {},
                Rules::type_ref => {
                    type_n = Ok(child.get_string(source));
                }
                Rules::variable_name => {
                    name = Ok(child.get_string(source));
                }
                rule => {
                    return Err(FileError::UnexpectedNode(
                        rule,
                        "VariableDeclaration::new".to_string(),
                    ));
                }
            }
        }
        Ok(Self { name: name?, type_n: type_n?, start_position: node.start_position, end_position: node.end_position})
    }

    pub fn push_if_not_exists_else_err(self, var_decs: &mut Vec<VariableDeclaration>) -> Result<(), FileError> {
        let res: u32 = var_decs
            .iter()
            .map(|intfc| intfc.name == self.name)
            .fold(0, |mut acc, result| {
                acc += result as u32;
                acc
            });
        if res == 0{
            var_decs.push(self);
            Ok(())
        }
        else{
            Err(FileError::FieldAlreadyExists(self.name))
        }
    }
}
