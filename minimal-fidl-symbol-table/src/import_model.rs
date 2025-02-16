use std::{path::{Path, PathBuf}, str::FromStr};

use crate::symbol_table::SymbolTableError;
use minimal_fidl_parser::{BasicPublisher, Key, Node, Rules};
#[derive(Debug)]
pub struct ImportModel {
    file_path: PathBuf,

}
impl ImportModel {
    pub fn new(
        source: &str,
        publisher: &BasicPublisher,
        node: &Node,
    ) -> Result<Self, SymbolTableError> {
        debug_assert_eq!(node.rule, Rules::import_model);
        let mut filepath: Result<PathBuf, SymbolTableError> = Err(
            SymbolTableError::InternalLogicError("Uninitialized value: filepath in ImportModel::new".to_string()),
        );

        for child in node.get_children() {
            let child = publisher.get_node(*child);
            match child.rule {
                Rules::comment | Rules::multiline_comment => {},
                Rules::file_path => {
                    let res = child.get_string(source);
                    filepath = Ok(PathBuf::from_str(&res[1..(res.len()-1)]).expect("Claims to be infallible"));
                }
                rule => {
                    return Err(SymbolTableError::UnexpectedNode(
                        rule,
                        "ImportModel::new".to_string(),
                    ));
                }
            }
        }
        Ok(Self { file_path: filepath?})
    }
}
