use std::{path::{Path, PathBuf}, str::FromStr};

use crate::{symbol_table::SymbolTableError, VariableDeclaration};
use minimal_fidl_parser::{BasicPublisher, Key, Node, Rules};
#[derive(Debug, Clone)]
pub struct Attribute {
    start_position: u32,
    end_position: u32,
    pub name: String,
    type_n: String

}
impl Attribute {
    pub fn new(
        source: &str,
        publisher: &BasicPublisher,
        node: &Node,
    ) -> Result<Self, SymbolTableError> {
        debug_assert_eq!(node.rule, Rules::attribute);
        let mut name: Result<String, SymbolTableError> = Err(
            SymbolTableError::InternalLogicError("Uninitialized value: name in Attribute::new".to_string()),
        );
        let mut type_n: Result<String, SymbolTableError> = Err(
            SymbolTableError::InternalLogicError("Uninitialized value: name in Attribute::new".to_string()),
        );
        for child in node.get_children() {
            let child = publisher.get_node(*child);
            match child.rule {
                Rules::comment
                | Rules::multiline_comment
                | Rules::annotation_block
=> {},
                Rules::type_ref => {
                    type_n = Ok(child.get_string(source));                    
                }
                Rules::variable_name => {
                    name = Ok(child.get_string(source))
                }

                rule => {
                    return Err(SymbolTableError::UnexpectedNode(
                        rule,
                        "Attribute::new".to_string(),
                    ));
                }
            }
        }
        Ok(Self { name: name?, type_n: type_n?, start_position: node.start_position, end_position: node.end_position})
    }
    pub fn push_if_not_exists_else_err(self, attributes: &mut Vec<Attribute>) -> Result<(), SymbolTableError> {
        for attr in &mut *attributes{
            if attr.name == self.name{
                return Err(SymbolTableError::AttributeAlreadyExists(attr.clone(), self.clone()));

            }
        }
        attributes.push(self);
        Ok(())

    }

}
