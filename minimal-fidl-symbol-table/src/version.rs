use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use crate::symbol_table::SymbolTableError;
use minimal_fidl_parser::{BasicPublisher, Key, Node, Rules};
#[derive(Debug, Clone)]
pub struct Version {
    major: Option<u32>,
    minor: Option<u32>
}
impl Version {
    pub fn new(
        source: &str,
        publisher: &BasicPublisher,
        node: &Node,
    ) -> Result<Self, SymbolTableError> {
        debug_assert_eq!(node.rule, Rules::version);
        let mut major: Option<u32> = None;
        let mut minor: Option<u32> = None;
        for child in node.get_children() {
            let child = publisher.get_node(*child);
            match child.rule {
                Rules::comment
                | Rules::multiline_comment
                | Rules::open_bracket
                | Rules::annotation_block
                | Rules::close_bracket => {},
                Rules::major => {
                   major = Some(Self::get_version_number(source, publisher, child));
                }
                Rules::minor => {
                    minor = Some(Self::get_version_number(source, publisher, child));
                 }

                rule => {
                    return Err(SymbolTableError::UnexpectedNode(
                        rule,
                        "Version::new".to_string(),
                    ));
                }
            }
        }
        Ok(Self {
            major,
            minor,
        })
    }

    fn get_version_number(source: &str,
        publisher: &BasicPublisher,
        node: &Node) -> u32{
            debug_assert!(node.rule == Rules::major || node.rule == Rules::minor);
            0
        }

    pub fn push_if_not_exists_else_err(self, version: &mut Option<Version>) -> Result<(), SymbolTableError>{
        // Set would be a more appropriate name but push is more consistent naming.
        // TODO: This never happens because the parser does not allow more than one version.
        // However, the error messages in that case would be far less useful so it might be worth modifying the
        // grammar to allow more than one version so we can print an appropriate error instead. 
        match version{
            None => {
                *version = Some(self);
                Ok(())

            }
            Some(version) => {
                Err(SymbolTableError::VersionAlreadyExists(version.clone()))

            }
        }
    }
}
