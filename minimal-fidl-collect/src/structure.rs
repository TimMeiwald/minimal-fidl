use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use crate::{annotation::{annotation_constructor, Annotation}, fidl_file::FileError, VariableDeclaration};
use minimal_fidl_parser::{BasicPublisher, Key, Node, Rules};
#[derive(Debug, Clone)]
pub struct Structure {
    start_position: u32,
    end_position: u32,
    pub annotations: Vec<Annotation>,
    pub name: String,
    pub contents: Vec<VariableDeclaration>,
}
impl Structure {
    pub fn new(source: &str, publisher: &BasicPublisher, node: &Node) -> Result<Self, FileError> {
        debug_assert_eq!(node.rule, Rules::structure);
        let mut name: Result<String, FileError> = Err(FileError::InternalLogicError(
            "Uninitialized value: name in Structure::new".to_string(),
        ));
        let mut annotations: Vec<Annotation> = Vec::new();

        let mut contents: Vec<VariableDeclaration> = Vec::new();
        for child in node.get_children() {
            let child = publisher.get_node(*child);
            match child.rule {
                Rules::comment
                | Rules::multiline_comment
                | Rules::open_bracket
                | Rules::close_bracket => {},
                Rules::annotation_block => {
                    annotations = annotation_constructor(source, publisher, child)?;
                }
                Rules::type_dec => {
                    name = Ok(child.get_string(source));
                }
                Rules::variable_declaration => {
                    let var_dec = VariableDeclaration::new(source, publisher, child)?;
                    var_dec.push_if_not_exists_else_err(&mut contents)?;
                }

                rule => {
                    return Err(FileError::UnexpectedNode(
                        rule,
                        "Structure::new".to_string(),
                    ));
                }
            }
        }
        Ok(Self {
            name: name?,
            contents,
            annotations,
            start_position: node.start_position,
            end_position: node.end_position,
        })
    }

    pub fn push_if_not_exists_else_err(
        self,
        structures: &mut Vec<Structure>,
    ) -> Result<(), FileError> {
        for s in &mut *structures {
            if s.name == self.name {
                return Err(FileError::StructAlreadyExists(s.clone(), self.clone()));
            }
        }
        structures.push(self);
        Ok(())
    }
}
