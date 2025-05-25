use core::fmt;
use std::fs;
use std::path::PathBuf;

use crate::attribute::Attribute;
use crate::enum_value::EnumValue;
use crate::enumeration::Enumeration;
use crate::method::Method;
use crate::structure::Structure;
use crate::type_def::TypeDef;
use crate::version::Version;
use crate::ImportModel;
use crate::ImportNamespace;
use crate::Interface;
use crate::Package;
use crate::TypeCollection;
use minimal_fidl_parser::{
    BasicContext, Context, Source, _var_name, grammar, BasicPublisher, Key, Rules, RULES_SIZE,
};
use std::cell::RefCell;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FileError {
    #[error("Unexpected Node: {0:?} in '{1}'!")]
    UnexpectedNode(Rules, String),
    #[error("Could not parse file: {0:?}")]
    CouldNotParseFile(PathBuf),
    #[error("Could not parse source string: {0:?}")]
    CouldNotParseSourceString(String),
    #[error("Could not read file: {0:?}")]
    CouldNotReadFile(std::io::Error),
    // #[error("Could not parse `{0}` as an integer.")]
    // IntegerParseError(String),
    #[error["This error means the program has a bug: {0}"]]
    InternalLogicError(String),
    #[error["The Interface: 'TODO' already exists!\nFirst Interface\n{0:#?}\nSecond Interface\n{1:#?}"]]
    InterfaceAlreadyExists(Interface, Interface),
    #[error["The Field: '{0}' already exists!"]]
    FieldAlreadyExists(String),
    #[error["The Struct: 'TODO' already exists.\nFirst Struct\n{0:#?}\nSecond Struct\n{1:#?}"]]
    StructAlreadyExists(Structure, Structure),
    #[error["The attribute: 'TODO' already exists.\nFirst Attribute\n{0:#?}\nSecond Attribute\n{1:#?}"]]
    AttributeAlreadyExists(Attribute, Attribute),
    #[error["The typedef: 'TODO' already exists.\nFirst typedef\n{0:#?}\nSecond typedef\n{1:#?}"]]
    TypeDefAlreadyExists(TypeDef, TypeDef),
    #[error["The Version: 'TODO' already exists.\n{0:#?}"]]
    VersionAlreadyExists(Version),
    #[error["The Method: 'TODO' already exists.\nFirst Struct\n{0:#?}\nSecond Struct\n{1:#?}"]]
    MethodAlreadyExists(Method, Method),
    #[error["The Package: 'TODO' already exists.\n{0:#?}"]]
    PackageAlreadyExists(Package),
    #[error["The Enumeration: 'TODO' already exists.\nFirst Enum\n{0:#?}\nSecond Enum\n{1:#?}"]]
    EnumerationAlreadyExists(Enumeration, Enumeration),
    #[error["Could not convert '{0}' to an Integer."]]
    CouldNotConvertToInteger(String),
    #[error["The Enum Value: 'TODO' already exists.\nFirst Enum Value\n{0:#?}\nSecond Enum Value\n{1:#?}"]]
    EnumValueAlreadyExists(EnumValue, EnumValue),
    #[error["The Type Collection: 'TODO' already exists.\nFirst Type Collection\n{0:#?}\nSecond Type Collection\n{1:#?}"]]
    TypeCollectionAlreadyExists(TypeCollection, TypeCollection),
    #[error["The Type collection requires a name"]]
    TypeCollectionRequiresAName,
}

pub struct FidlFileRs {
    pub source: String,
    pub package: Option<Package>,
    pub namespaces: Vec<ImportNamespace>,
    pub import_models: Vec<ImportModel>,
    pub interfaces: Vec<Interface>,
    pub type_collections: Vec<TypeCollection>,
}

impl fmt::Debug for FidlFileRs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // The below is some kind of magic I don't fully understand but basically
        // it let's me print just specific fields(the ones deifned in SymbolTable below) and
        // not print source or BasicPublisher
        #[derive(Debug)]
        struct FidlFileRs<'a> {
            package: &'a Option<Package>,
            namespaces: &'a Vec<ImportNamespace>,
            import_models: &'a Vec<ImportModel>,
            interfaces: &'a Vec<Interface>,
            type_collections: &'a Vec<TypeCollection>,
        }
        // Below somehow allows me to use the internals of SymbolTable without explicitly using namespace: self.namespace
        // In a SymbolTableRepr construction.
        let Self {
            source: _,
            package,
            namespaces,
            import_models,
            interfaces,
            type_collections,
        } = self;
        fmt::Debug::fmt(
            &FidlFileRs {
                package,
                namespaces,
                import_models,
                interfaces,
                type_collections,
            },
            f,
        )
    }
}

impl FidlFileRs {

    pub fn new(source: String, publisher: &BasicPublisher) -> Result<Self, FileError> {
        let mut resp = Self {
            source,
            package: None,
            namespaces: Vec::new(),
            import_models: Vec::new(),
            interfaces: Vec::new(),
            type_collections: Vec::new(),
        };
        let result = resp.create_symbol_table(&publisher);
        match result {
            Ok(()) => Ok(resp),
            Err(err) => Err(err),
        }
    }

    fn create_symbol_table(&mut self, publisher: &BasicPublisher) -> Result<(), FileError> {
        let root_node = publisher.get_node(Key(0));
        debug_assert_eq!(root_node.rule, Rules::Grammar);
        let root_node_children = root_node.get_children();
        debug_assert_eq!(root_node_children.len(), 1);
        let grammar_node_key = root_node_children[0];
        let grammar_node = publisher.get_node(grammar_node_key);
        for child in grammar_node.get_children() {
            let child = publisher.get_node(*child);
            match child.rule {
                Rules::package => {
                    let package = Package::new(&self.source, &publisher, child)?;
                    package.push_if_not_exists_else_err(&mut self.package)?;
                }
                Rules::import_namespace => {
                    let import_namespace = ImportNamespace::new(&self.source, &publisher, child)?;
                    self.namespaces.push(import_namespace);
                }
                Rules::import_model => {
                    let import_model = ImportModel::new(&self.source, &publisher, child)?;
                    self.import_models.push(import_model);
                }
                Rules::interface => {
                    let interface = Interface::new(&self.source, &publisher, child)?;
                    interface.push_if_not_exists_else_err(&mut self.interfaces)?;
                }
                Rules::type_collection => {
                    let type_collection = TypeCollection::new(&self.source, &publisher, child)?;
                    type_collection.push_if_not_exists_else_err(&mut self.type_collections)?;
                }
                Rules::comment
                | Rules::multiline_comment
                | Rules::open_bracket
                | Rules::annotation_block
                | Rules::close_bracket => {}
                rule => {
                    return Err(FileError::UnexpectedNode(
                        rule,
                        "SymblTable::create_symbol_table".to_string(),
                    ));
                }
            }
        }
        Ok(())
    }
}
