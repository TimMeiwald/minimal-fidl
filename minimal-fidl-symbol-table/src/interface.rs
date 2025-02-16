use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use crate::{structure::Structure, symbol_table::SymbolTableError, Version};
use minimal_fidl_parser::{BasicPublisher, Key, Node, Rules};
#[derive(Debug)]
pub struct Interface {
    pub name: String,
    version: Version,
    structures: Vec<Structure>
}
impl Interface {
    pub fn new(
        source: &str,
        publisher: &BasicPublisher,
        node: &Node,
    ) -> Result<Self, SymbolTableError> {
        debug_assert_eq!(node.rule, Rules::interface);
        let mut name: Result<String, SymbolTableError> = Err(SymbolTableError::InternalLogicError(
            "Uninitialized value: 'name' in Interface::new".to_string(),
        ));
        let mut version: Result<Version, SymbolTableError> = Err(SymbolTableError::InternalLogicError(
            "Uninitialized value: 'version' in Interface::new".to_string(),
        ));
        let mut structures: Vec<Structure> = Vec::new();
        for child in node.get_children() {
            let child = publisher.get_node(*child);
            match child.rule {
                Rules::variable_name => {
                    let name_str = Self::variable_name(source, publisher, child);
                    name = Ok(name_str);
                }
                Rules::version => {
                    version = Version::new(source, publisher, child);
                }
                Rules::structure => {
                    let structure = Structure::new(source, publisher, child)?;
                    structures.push(structure);
                }
                Rules::comment
                | Rules::multiline_comment
                | Rules::open_bracket
                | Rules::close_bracket => {}
                rule => {
                    return Err(SymbolTableError::UnexpectedNode(
                        rule,
                        "Interface::new".to_string(),
                    ));
                }
            }
        }
        Ok(Self { name: name?, version: version?, structures })
    }

    fn variable_name(source: &str, publisher: &BasicPublisher, node: &Node) -> String {
        node.get_string(source)
    }
}
