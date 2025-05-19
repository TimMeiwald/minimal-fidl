use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use crate::{annotation::{annotation_constructor, Annotation}, fidl_file::FileError, VariableDeclaration};
use minimal_fidl_parser::{BasicPublisher, Key, Node, Rules};
#[derive(Debug, Clone)]
pub struct Method {
    start_position: u32,
    end_position: u32,
    pub annotations: Vec<Annotation>,
    pub name: String,
    pub input_parameters: Vec<VariableDeclaration>,
    pub output_parameters: Vec<VariableDeclaration>,
}
impl Method {
    pub fn new(source: &str, publisher: &BasicPublisher, node: &Node) -> Result<Self, FileError> {
        debug_assert_eq!(node.rule, Rules::method);
        let mut name: Result<String, FileError> = Err(FileError::InternalLogicError(
            "Uninitialized value: name in Method::new".to_string(),
        ));
        let mut input_parameters: Vec<VariableDeclaration> = Vec::new();
        let mut output_parameters: Vec<VariableDeclaration> = Vec::new();
        let mut annotations: Vec<Annotation> = Vec::new();

        for child in node.get_children() {
            let child = publisher.get_node(*child);
            match child.rule {
                Rules::comment
                | Rules::multiline_comment
                | Rules::open_bracket
                | Rules::close_bracket => {}
                Rules::annotation_block => {
                    annotations = annotation_constructor(source, publisher, node)?;
                }
                Rules::variable_name => {
                    name = Ok(child.get_string(source));
                }
                Rules::type_dec => {
                    name = Ok(child.get_string(source));
                }
                Rules::input_params => {
                    Self::params(source, publisher, child, &mut input_parameters)?;
                }
                Rules::output_params => {
                    Self::params(source, publisher, child, &mut output_parameters)?;
                }
                rule => {
                    return Err(FileError::UnexpectedNode(rule, "Method::new".to_string()));
                }
            }
        }
        Ok(Self {
            name: name?,
            start_position: node.start_position,
            annotations,
            end_position: node.end_position,
            input_parameters,
            output_parameters,
        })
    }
    pub fn push_if_not_exists_else_err(self, methods: &mut Vec<Method>) -> Result<(), FileError> {
        for s in &mut *methods {
            if s.name == self.name {
                return Err(FileError::MethodAlreadyExists(s.clone(), self.clone()));
            }
        }
        methods.push(self);
        Ok(())
    }

    fn params(
        source: &str,
        publisher: &BasicPublisher,
        node: &Node,
        params: &mut Vec<VariableDeclaration>,
    ) -> Result<(), FileError> {
        for child in node.get_children() {
            let child = publisher.get_node(*child);
            match child.rule {
                Rules::variable_declaration => {
                    let var_dec = VariableDeclaration::new(source, publisher, child)?;
                    var_dec.push_if_not_exists_else_err(params)?;
                }
                Rules::comment
                | Rules::multiline_comment
                | Rules::open_bracket
                | Rules::annotation_block
                | Rules::close_bracket => {}
                rule => {
                    return Err(FileError::UnexpectedNode(
                        rule,
                        "Method::params".to_string(),
                    ));
                }
            }
        }
        Ok(())
    }
}
