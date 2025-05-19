use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use crate::fidl_file::FileError;
use minimal_fidl_parser::{BasicPublisher, Key, Node, Rules};
#[derive(Debug, Clone)]
pub struct Version {
    start_position: u32,
    end_position: u32,
    pub major: Option<u32>,
    pub minor: Option<u32>,
}
impl Version {
    pub fn new(source: &str, publisher: &BasicPublisher, node: &Node) -> Result<Self, FileError> {
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
                | Rules::close_bracket => {}
                Rules::major => {
                    let major_version = Self::get_version_number(source, publisher, child)?;
                    major = Some(major_version as u32); // Will cause an issue if it overflows but cmon
                }
                Rules::minor => {
                    let minor_version = Self::get_version_number(source, publisher, child)?;
                    minor = Some(minor_version as u32); // Will cause an issue if it overflows but cmon
                }

                rule => {
                    return Err(FileError::UnexpectedNode(rule, "Version::new".to_string()));
                }
            }
        }
        Ok(Self {
            major,
            minor,
            start_position: node.start_position,
            end_position: node.end_position,
        })
    }

    fn get_version_number(
        source: &str,
        publisher: &BasicPublisher,
        node: &Node,
    ) -> Result<u64, FileError> {
        debug_assert!(node.rule == Rules::major || node.rule == Rules::minor);
        for child in node.get_children() {
            let child = publisher.get_node(*child);
            match child.rule {
                Rules::comment
                | Rules::multiline_comment
                | Rules::open_bracket
                | Rules::annotation_block
                | Rules::close_bracket => {}
                Rules::digits => {
                    return Self::convert_string_representation_of_number_to_value(
                        child.get_string(source),
                    );
                }
                rule => {
                    return Err(FileError::UnexpectedNode(rule, "Version::new".to_string()));
                }
            }
        }
        return Err(FileError::InternalLogicError(
            "If Version node exists it should have a major and minor version".to_string(),
        ));
    }

    fn convert_string_representation_of_number_to_value(input: String) -> Result<u64, FileError> {
        let value = input.parse::<u64>();

        match value {
            Ok(integer) => return Ok(integer),
            Err(e) => {}
        };
        let hex_input = input.strip_prefix("0x");
        match hex_input {
            Some(hex_input) => {
                let value = u64::from_str_radix(&hex_input, 16);
                match value {
                    Ok(integer) => return Ok(integer),
                    Err(_e) => {}
                };
            }
            None => {}
        }

        let bin_input = input.strip_prefix("0b");
        match bin_input {
            Some(bin_input) => {
                let value = u64::from_str_radix(&bin_input, 2);
                match value {
                    Ok(integer) => return Ok(integer),
                    Err(_e) => {}
                };
            }
            None => {}
        }
        Err(FileError::CouldNotConvertToInteger(input))
    }

    pub fn push_if_not_exists_else_err(
        self,
        version: &mut Option<Version>,
    ) -> Result<(), FileError> {
        // Set would be a more appropriate name but push is more consistent naming.
        // TODO: This never happens because the parser does not allow more than one version.
        // However, the error messages in that case would be far less useful so it might be worth modifying the
        // grammar to allow more than one version so we can print an appropriate error instead.
        match version {
            None => {
                *version = Some(self);
                Ok(())
            }
            Some(version) => Err(FileError::VersionAlreadyExists(version.clone())),
        }
    }
}
