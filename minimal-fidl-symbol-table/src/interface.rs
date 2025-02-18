use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use crate::{attribute::{self, Attribute}, structure::Structure, symbol_table::SymbolTableError, Version};
use minimal_fidl_parser::{BasicPublisher, Key, Node, Rules};
#[derive(Debug)]
pub struct Interface {
    pub name: String,
    version: Version,
    attributes: Vec<Attribute>,
    structures: Vec<Structure>,
    
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
        let mut attributes: Vec<Attribute> = Vec::new();

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
                    let _res = Self::add_structure(&mut structures, structure)?;
                }
                Rules::attribute => {
                    let attribute = Attribute::new(source, publisher, child)?;
                    let _res = Self::add_attribute(&mut attributes, attribute)?;
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
        Ok(Self { name: name?, version: version?, structures, attributes })
    }

    fn variable_name(source: &str, publisher: &BasicPublisher, node: &Node) -> String {
        node.get_string(source)
    }


    fn add_structure(structures: &mut Vec<Structure>, structure: Structure) -> Result<(), SymbolTableError> {
        let res: u32 = structures
            .iter()
            .map(|intfc| intfc.name == structure.name)
            .fold(0, |mut acc, result| {
                acc += result as u32;
                acc
            });
        if res == 0{
            structures.push(structure);
            Ok(())
        }
        else{
            for s in structures{
                if s.name == structure.name{
                    return Err(SymbolTableError::StructAlreadyExists(structure, s.clone()))

                }
            }
            Err(SymbolTableError::InternalLogicError("Must not reach here".to_string()))
        }
    }

    fn add_attribute(attributes: &mut Vec<Attribute>, attribute: Attribute) -> Result<(), SymbolTableError> {
        let res: u32 = attributes
            .iter()
            .map(|intfc| intfc.name == attribute.name)
            .fold(0, |mut acc, result| {
                acc += result as u32;
                acc
            });
        if res == 0{
            attributes.push(attribute);
            Ok(())
        }
        else{
            for s in attributes{
                if s.name == attribute.name{
                    return Err(SymbolTableError::AttributeAlreadyExists(attribute, s.clone()))

                }
            }
            Err(SymbolTableError::InternalLogicError("Must not reach here".to_string()))
        }
    }


}
