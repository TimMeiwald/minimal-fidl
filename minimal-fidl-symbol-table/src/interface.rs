use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use crate::{
    attribute::{self, Attribute},
    method::Method,
    structure::Structure,
    symbol_table::SymbolTableError,
    type_def::TypeDef,
    Version,
};
use minimal_fidl_parser::{BasicPublisher, Key, Node, Rules};
#[derive(Debug, Clone)]
pub struct Interface {
    pub name: String,
    version: Option<Version>,
    attributes: Vec<Attribute>,
    structures: Vec<Structure>,
    typedefs: Vec<TypeDef>,
    methods: Vec<Method>,
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
        let mut version: Option<Version> = None;
        let mut structures: Vec<Structure> = Vec::new();
        let mut attributes: Vec<Attribute> = Vec::new();
        let mut typedefs: Vec<TypeDef> = Vec::new();
        let mut methods: Vec<Method> = Vec::new();

        for child in node.get_children() {
            let child = publisher.get_node(*child);
            match child.rule {
                Rules::variable_name => {
                    let name_str = Self::variable_name(source, publisher, child);
                    name = Ok(name_str);
                }
                Rules::version => {
                    let ver = Version::new(source, publisher, child)?;
                    ver.push_if_not_exists_else_err(&mut version)?;
                }
                Rules::structure => {
                    let structure = Structure::new(source, publisher, child)?;
                    structure.push_if_not_exists_else_err(&mut structures)?;
                }
                Rules::attribute => {
                    let attribute = Attribute::new(source, publisher, child)?;
                    attribute.push_if_not_exists_else_err(&mut attributes)?;
                }
                Rules::typedef => {
                    let typedef = TypeDef::new(source, publisher, child)?;
                    typedef.push_if_not_exists_else_err(&mut typedefs)?;
                }
                Rules::method => {
                    let method = Method::new(source, publisher, child)?;
                    method.push_if_not_exists_else_err(&mut methods)?;
                }

                Rules::comment
                | Rules::multiline_comment
                | Rules::open_bracket
                | Rules::annotation_block
                | Rules::close_bracket => {}
                rule => {
                    return Err(SymbolTableError::UnexpectedNode(
                        rule,
                        "Interface::new".to_string(),
                    ));
                }
            }
        }
        Ok(Self {
            name: name?,
            version,
            structures,
            attributes,
            typedefs,
            methods,
        })
    }

    fn variable_name(source: &str, _publisher: &BasicPublisher, node: &Node) -> String {
        node.get_string(source)
    }

    pub fn push_if_not_exists_else_err(self, interfaces: &mut Vec<Interface>) -> Result<(), SymbolTableError> {
        for s in &mut *interfaces{
            if s.name == self.name{
                return Err(SymbolTableError::InterfaceAlreadyExists(s.clone(), self.clone()));

            }
        }
        interfaces.push(self);
        Ok(())

    }
}
