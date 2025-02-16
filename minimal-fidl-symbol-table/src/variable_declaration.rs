use std::{path::{Path, PathBuf}, str::FromStr};

use crate::symbol_table::SymbolTableError;
use minimal_fidl_parser::{BasicPublisher, Key, Node, Rules};
#[derive(Debug)]
pub struct VariableDeclaration {
    type_n: String,
    pub name: String,

}
impl VariableDeclaration {
    pub fn new(
        source: &str,
        publisher: &BasicPublisher,
        node: &Node,
    ) -> Result<Self, SymbolTableError> {
        debug_assert_eq!(node.rule, Rules::variable_declaration);
        let mut type_n: Result<String, SymbolTableError> = Err(
            SymbolTableError::InternalLogicError("Uninitialized value: type_n in VariableDeclaration::new".to_string()),
        );
        let mut name: Result<String, SymbolTableError> = Err(
            SymbolTableError::InternalLogicError("Uninitialized value: name in VariableDeclaration::new".to_string()),
        );

        for child in node.get_children() {
            let child = publisher.get_node(*child);
            match child.rule {
                Rules::comment | Rules::multiline_comment => {},
                Rules::type_ref => {
                    type_n = Ok(child.get_string(source));
                }
                Rules::variable_name => {
                    name = Ok(child.get_string(source));
                }
                rule => {
                    return Err(SymbolTableError::UnexpectedNode(
                        rule,
                        "VariableDeclaration::new".to_string(),
                    ));
                }
            }
        }
        Ok(Self { name: name?, type_n: type_n?})
    }
}
