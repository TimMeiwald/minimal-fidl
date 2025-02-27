use std::{path::{Path, PathBuf}, str::FromStr};

use crate::{fidl_file::FileError, VariableDeclaration};
use minimal_fidl_parser::{BasicPublisher, Key, Node, Rules};
#[derive(Debug, Clone)]
pub struct TypeDef {
    start_position: u32,
    end_position: u32,
    pub name: String,
    type_n: String

}
impl TypeDef {
    pub fn new(
        source: &str,
        publisher: &BasicPublisher,
        node: &Node,
    ) -> Result<Self, FileError> {
        debug_assert_eq!(node.rule, Rules::typedef);
        let mut name: Result<String, FileError> = Err(
            FileError::InternalLogicError("Uninitialized value: name in TypeDef::new".to_string()),
        );
        let mut type_n: Result<String, FileError> = Err(
            FileError::InternalLogicError("Uninitialized value: name in TypeDef::new".to_string()),
        );
        for child in node.get_children() {
            let child = publisher.get_node(*child);
            match child.rule {
                Rules::comment
                | Rules::multiline_comment
                | Rules::annotation_block
=> {},
                Rules::type_dec => {
                    println!("Need to actually do this stuff. Types need to be checked for duplicates and whether they exist if using external import after reading file.");
                    name = Ok(child.get_string(source))
                }
                Rules::type_ref => {
                    type_n = Ok(child.get_string(source));                    
                }
                rule => {
                    return Err(FileError::UnexpectedNode(
                        rule,
                        "TypeDef::new".to_string(),
                    ));
                }
            }
        }
        Ok(Self { name: name?, type_n: type_n?, start_position: node.start_position, end_position: node.end_position})
    }

    pub fn push_if_not_exists_else_err(self, typedefs: &mut Vec<TypeDef>) -> Result<(), FileError> {
        for t in &mut *typedefs{
            if t.name == self.name{
                return Err(FileError::TypeDefAlreadyExists(t.clone(), self.clone()));

            }
        }
        typedefs.push(self);
        Ok(())

    }

}
