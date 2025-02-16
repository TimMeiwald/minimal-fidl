use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use crate::symbol_table::SymbolTableError;
use minimal_fidl_parser::{BasicPublisher, Key, Node, Rules};
#[derive(Debug)]
pub struct Interface {
    name: String,
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

        for child in node.get_children() {
            let child = publisher.get_node(*child);
            match child.rule {
                Rules::variable_name => {
                    let name_str = Self::variable_name(source, publisher, child);
                    name = Ok(name_str);
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
        Ok(Self { name: name? })
    }

    fn variable_name(source: &str, publisher: &BasicPublisher, node: &Node) -> String {
        node.get_string(source)
    }
}
