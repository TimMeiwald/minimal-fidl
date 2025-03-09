use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use crate::fidl_file::FileError;
use minimal_fidl_parser::{BasicPublisher, Key, Node, Rules};
#[derive(Debug, Clone)]
pub struct TypeRef {
    pub name: String,
    pub is_array: bool,
}
impl TypeRef {
    pub fn new(source: &str, publisher: &BasicPublisher, node: &Node) -> Result<Self, FileError> {
        debug_assert_eq!(node.rule, Rules::type_ref);
        let mut is_array: bool = false;
        let mut arr_start_position = 0;
        for child in node.get_children() {
            let child = publisher.get_node(*child);
            match child.rule {
                Rules::array => {
                    is_array = true;
                    arr_start_position = child.start_position
                }
                rule => {
                    return Err(FileError::UnexpectedNode(
                        rule,
                        "TypeDef::type_ref".to_string(),
                    ));
                }
            }
        }
        if !is_array {
            return Ok(Self {
                name: node.get_string(source),
                is_array: false,
            });
        } else {
            return Ok(Self {
                name: (node.get_string(source)
                    [0..(arr_start_position - node.start_position) as usize])
                    .to_string(),
                is_array: true,
            });
        }
    }
}
