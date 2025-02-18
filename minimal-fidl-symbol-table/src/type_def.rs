use std::{path::{Path, PathBuf}, str::FromStr};

use crate::{symbol_table::SymbolTableError, VariableDeclaration};
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
    ) -> Result<Self, SymbolTableError> {
        debug_assert_eq!(node.rule, Rules::typedef);
        let mut name: Result<String, SymbolTableError> = Err(
            SymbolTableError::InternalLogicError("Uninitialized value: name in TypeDef::new".to_string()),
        );
        let mut type_n: Result<String, SymbolTableError> = Err(
            SymbolTableError::InternalLogicError("Uninitialized value: name in TypeDef::new".to_string()),
        );
        for child in node.get_children() {
            let child = publisher.get_node(*child);
            match child.rule {
                Rules::comment
                | Rules::multiline_comment
                | Rules::open_bracket
                | Rules::close_bracket => {},
                Rules::type_dec => {
                    todo!("Need to actually do this stuff. Types need to be checked after reading file, 
                    symbol table is actually File type imo.");
                    name = Ok(child.get_string(source))
                }
                Rules::type_ref => {
                    type_n = Ok(child.get_string(source));                    
                }
                rule => {
                    return Err(SymbolTableError::UnexpectedNode(
                        rule,
                        "TypeDef::new".to_string(),
                    ));
                }
            }
        }
        Ok(Self { name: name?, type_n: type_n?, start_position: node.start_position, end_position: node.end_position})
    }

}
