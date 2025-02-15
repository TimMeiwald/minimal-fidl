use core::fmt;

use crate::Interface;
use crate::TypeCollection;
use crate::ImportModel;
use crate::ImportNamespace;
use crate::Package;
use minimal_fidl_parser::{BasicPublisher, Rules, Key};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum SymbolTableError {
    #[error("Unexpected Node: {0:?} in '{1}'!")]
    UnexpectedNode(Rules, String),
    // #[error("Could not parse `{0}` as an integer.")]
    // IntegerParseError(String),
}


pub struct SymbolTable<'a> {
    source: &'a str,
    publisher: &'a BasicPublisher,
    packages: Vec<Package>,
    namespaces: Vec<ImportNamespace>,
    import_models: Vec<ImportModel>,
    interfaces: Vec<Interface>,
    type_collections: Vec<TypeCollection>,
}

impl fmt::Debug for SymbolTable<'_>{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // The below is some kind of magic I don't fully understand but basically
        // it let's me print just specific fields(the ones deifned in SymbolTable below) and 
        // not print source or BasicPublisher
        #[derive(Debug)]
        struct SymbolTableRepr<'a>{
            packages: &'a Vec<Package>,
            namespaces: &'a Vec<ImportNamespace>,
            import_models: &'a Vec<ImportModel>,
            interfaces: &'a Vec<Interface>,
            type_collections: &'a Vec<TypeCollection>,
        }
        // Below somehow allows me to use the internals of SymbolTable without explicitly using namespace: self.namespace
        // In a SymbolTableRepr construction.
        let Self {
            source: _, publisher: _, packages, namespaces, import_models, interfaces, type_collections
        } = self;
        fmt::Debug::fmt(&SymbolTableRepr{packages, namespaces, import_models, interfaces, type_collections}, f)
    }
}


impl<'a> SymbolTable<'a>{
    pub fn new(source: &'a str, publisher: &'a BasicPublisher) -> Self {
        Self {
            source,
            publisher,
            packages: Vec::new(),
            namespaces: Vec::new(),
            import_models: Vec::new(),
            interfaces: Vec::new(),
            type_collections: Vec::new(),
        }
    }

    fn create_symbol_table(&self) -> Result<(), SymbolTableError>{
        let root_node = self.publisher.get_node(Key(0));
        debug_assert_eq!(root_node.rule, Rules::Grammar);
        let root_node_children = root_node.get_children();
        debug_assert_eq!(root_node_children.len(), 1);
        let grammar_node_key = root_node_children[0];
        let grammar_node = self.publisher.get_node(grammar_node_key);
        for child in grammar_node.get_children(){
            let child = self.publisher.get_node(*child);
            match child.rule { 
                Rules::package => {}, 
                Rules::import_namespace => {}
                Rules::import_model => {},
                Rules::interface => {},
                Rules::type_collection => {}
                rule => {
                    return Err(SymbolTableError::UnexpectedNode(rule, "create_symbol_table".to_string()));
                }

            }
        }
        Ok(())

    }
}

