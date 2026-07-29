use std::{
    path::PathBuf,
    str::FromStr,
};

use crate::fidl_file::FileError;
use minimal_fidl_parser::{BasicPublisher, Node, Rules};
use crate::node::{impl_ast_node, trailing_comments, NodeMeta};
#[derive(Debug, Clone)]
pub struct ImportModel {
    pub meta: NodeMeta,
    pub file_path: PathBuf,
}
impl ImportModel {
    pub fn new(source: &str, publisher: &BasicPublisher, node: &Node) -> Result<Self, FileError> {
        debug_assert_eq!(node.rule, Rules::import_model);
        let mut filepath: Result<PathBuf, FileError> = Err(FileError::InternalLogicError(
            "Uninitialized value: filepath in ImportModel::new".to_string(),
        ));

        for child in node.get_children() {
            let child = publisher.get_node(*child);
            match child.rule {
                Rules::comment | Rules::multiline_comment => {}
                Rules::file_path => {
                    let res = child.get_string(source);
                    filepath = Ok(PathBuf::from_str(&res[1..(res.len() - 1)])
                        .expect("Claims to be infallible"));
                }
                rule => {
                    return Err(FileError::UnexpectedNode(
                        rule,
                        "ImportModel::new".to_string(),
                    ));
                }
            }
        }
        Ok(Self {
            meta: NodeMeta {
                trailing_comments: trailing_comments(source, publisher, node),
                ..NodeMeta::from_cst(node)
            },
            file_path: filepath?,
        })
    }
}

impl_ast_node!(ImportModel);
