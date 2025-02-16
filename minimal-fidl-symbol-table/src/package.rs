use crate::symbol_table::SymbolTableError;
use minimal_fidl_parser::{BasicPublisher, Key, Node, Rules};
#[derive(Debug)]
pub struct Package {
    path: Vec<String>,
}
impl Package {
    pub fn new(
        source: &str,
        publisher: &BasicPublisher,
        node: &Node,
    ) -> Result<Self, SymbolTableError> {
        debug_assert_eq!(node.rule, Rules::package);
        let mut path: Result<Vec<String>, SymbolTableError> = Err(
            SymbolTableError::InternalLogicError("Uninitialized value in Package::new".to_string()),
        );
        for child in node.get_children() {
            let child = publisher.get_node(*child);
            match child.rule {
                Rules::comment | Rules::multiline_comment => {},
                Rules::type_ref => {
                    let res: String = child.get_string(source);
                    path = Ok(res.split(".").map(|string| {string.to_string()}).collect())
                }
                rule => {
                    return Err(SymbolTableError::UnexpectedNode(
                        rule,
                        "Package::new".to_string(),
                    ));
                }
            }
        }
        Ok(Self { path: path? })
    }
}
