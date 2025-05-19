use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use crate::{
    attribute::{self, Attribute},
    enumeration::{self, Enumeration},
    fidl_file::FileError,
    method::Method,
    structure::Structure,
    type_def::TypeDef,
    Version,
};
use minimal_fidl_parser::{BasicPublisher, Key, Node, Rules};
#[derive(Debug, Clone)]
pub struct TypeCollection {
    start_position: u32,
    end_position: u32,
    pub name: String,
    pub version: Option<Version>,
    pub typedefs: Vec<TypeDef>,
    pub structures: Vec<Structure>,
    pub enumerations: Vec<Enumeration>,
}
impl TypeCollection {
    pub fn new(source: &str, publisher: &BasicPublisher, node: &Node) -> Result<Self, FileError> {
        debug_assert_eq!(node.rule, Rules::type_collection);
        let mut name: String = "".to_string(); // Cos the type collection name can be seemingly empty.
        let mut version: Option<Version> = None;
        let mut structures: Vec<Structure> = Vec::new();
        let mut typedefs: Vec<TypeDef> = Vec::new();
        let mut enumerations: Vec<Enumeration> = Vec::new();

        for child in node.get_children() {
            let child = publisher.get_node(*child);
            match child.rule {
                Rules::variable_name => {
                    let name_str = Self::variable_name(source, publisher, child);
                    name = name_str;
                }
                Rules::version => {
                    let ver = Version::new(source, publisher, child)?;
                    ver.push_if_not_exists_else_err(&mut version)?;
                }
                Rules::structure => {
                    let structure = Structure::new(source, publisher, child)?;
                    structure.push_if_not_exists_else_err(&mut structures)?;
                }
                Rules::typedef => {
                    let typedef = TypeDef::new(source, publisher, child)?;
                    typedef.push_if_not_exists_else_err(&mut typedefs)?;
                }
                Rules::enumeration => {
                    let enumeration = Enumeration::new(source, publisher, child)?;
                    enumeration.push_if_not_exists_else_err(&mut enumerations)?;
                }

                Rules::comment
                | Rules::multiline_comment
                | Rules::open_bracket
                | Rules::annotation_block
                | Rules::close_bracket => {}
                rule => {
                    return Err(FileError::UnexpectedNode(
                        rule,
                        "TypeCollection::new".to_string(),
                    ));
                }
            }
        }
        if name.len() == 0 {
            return Err(FileError::TypeCollectionRequiresAName);
        }
        Ok(Self {
            name,
            version,
            structures,
            typedefs,
            enumerations,
            start_position: node.start_position,
            end_position: node.end_position,
        })
    }

    fn variable_name(source: &str, _publisher: &BasicPublisher, node: &Node) -> String {
        node.get_string(source)
    }

    pub fn push_if_not_exists_else_err(
        self,
        type_collections: &mut Vec<TypeCollection>,
    ) -> Result<(), FileError> {
        for s in &mut *type_collections {
            if s.name == self.name {
                return Err(FileError::TypeCollectionAlreadyExists(
                    s.clone(),
                    self.clone(),
                ));
            }
        }
        type_collections.push(self);
        Ok(())
    }
}
