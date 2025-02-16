use std::{path::{Path, PathBuf}, str::FromStr};

use crate::symbol_table::SymbolTableError;
use minimal_fidl_parser::{BasicPublisher, Key, Node, Rules};
#[derive(Debug)]
pub struct ImportNamespace {
    import: Vec<String>,
    from: PathBuf,
    wildcard: bool,

}
impl ImportNamespace {
    pub fn new(
        source: &str,
        publisher: &BasicPublisher,
        node: &Node,
    ) -> Result<Self, SymbolTableError> {
        debug_assert_eq!(node.rule, Rules::import_namespace);
        let mut wildcard = false;
        let mut import: Result<Vec<String>, SymbolTableError> = Err(
            SymbolTableError::InternalLogicError("Uninitialized value: 'import' in ImportNamespace::new".to_string()),
        );
        let mut from: Result<PathBuf, SymbolTableError> = Err(
            SymbolTableError::InternalLogicError("Uninitialized value: 'from' in ImportNamespace::new".to_string()),
        );

        for child in node.get_children() {
            let child = publisher.get_node(*child);
            match child.rule {
                Rules::comment | Rules::multiline_comment => {},
                Rules::type_ref => {
                    let res: String = child.get_string(source);
                    import = Ok(res.split(".").map(|string| {string.to_string()}).collect())
                }
                Rules::wildcard => {
                    wildcard = true;
                }
                Rules::file_path => {
                    let res = child.get_string(source);
                    from = Ok(PathBuf::from_str(&res[1..(res.len()-1)]).expect("Claims to be infallible"));
                }
                rule => {
                    return Err(SymbolTableError::UnexpectedNode(
                        rule,
                        "ImportNamespace::new".to_string(),
                    ));
                }
            }
        }
        Ok(Self { import: import? , wildcard, from: from?})
    }
}
