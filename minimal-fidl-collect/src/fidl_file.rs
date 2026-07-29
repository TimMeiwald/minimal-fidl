use core::fmt;
use std::path::PathBuf;

use crate::attribute::Attribute;
use crate::enum_value::EnumValue;
use crate::enumeration::Enumeration;
use crate::method::Method;
use crate::node::{
    impl_ast_node, sorted_children, Comment, MemberBuilder, MemberEnum, NodeIdGen, NodeMeta,
};
use crate::structure::Structure;
use crate::type_def::TypeDef;
use crate::version::Version;
use crate::ImportModel;
use crate::ImportNamespace;
use crate::Interface;
use crate::Package;
use crate::TypeCollection;
use minimal_fidl_parser::{BasicPublisher, Key, Rules};
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
    #[error["The Type collection requires a name\n{0}"]]
    TypeCollectionRequiresAName(String),
}


/// An ordered child of a [`FidlFile`].
#[derive(Debug, Clone)]
pub enum FileMember {
    Package(Package),
    ImportNamespace(ImportNamespace),
    ImportModel(ImportModel),
    Interface(Interface),
    TypeCollection(TypeCollection),
    Comment(Comment),
}

impl MemberEnum for FileMember {
    fn from_comment(comment: Comment) -> Self {
        FileMember::Comment(comment)
    }
}

/// A parsed `.fidl` file.
///
/// Members are held in a single ordered list; the typed accessors below filter
/// it, so they can never disagree with source order. See `DESIGN.md` §4.
pub struct FidlFile {
    pub meta: NodeMeta,
    /// The source this file was parsed from. Goes stale once the tree is edited;
    /// spans are only meaningful against this string.
    pub source: String,
    /// Set by [`crate::FidlProject::generate_file`], used by `save()` later.
    pub path: Option<PathBuf>,
    pub members: Vec<FileMember>,
    ids: NodeIdGen,
}

impl_ast_node!(FidlFile);

impl FidlFile {
    crate::member_accessors!(
        FileMember,
        Interface,
        Interface,
        interfaces,
        interfaces_mut,
        interface,
        interface_mut
    );
    crate::member_accessors!(
        FileMember,
        TypeCollection,
        TypeCollection,
        type_collections,
        type_collections_mut,
        type_collection,
        type_collection_mut
    );

    crate::container_ops!(FileMember);
    crate::member_mutators!(FileMember, Interface, Interface, add_interface, remove_interface, InterfaceAlreadyExists, interface);
    crate::member_mutators!(FileMember, TypeCollection, TypeCollection, add_type_collection, remove_type_collection, TypeCollectionAlreadyExists, type_collection);

    pub fn package(&self) -> Option<&Package> {
        self.members.iter().find_map(|m| match m {
            FileMember::Package(p) => Some(p),
            _ => None,
        })
    }

    pub fn namespaces(&self) -> impl Iterator<Item = &ImportNamespace> {
        self.members.iter().filter_map(|m| match m {
            FileMember::ImportNamespace(n) => Some(n),
            _ => None,
        })
    }

    pub fn import_models(&self) -> impl Iterator<Item = &ImportModel> {
        self.members.iter().filter_map(|m| match m {
            FileMember::ImportModel(i) => Some(i),
            _ => None,
        })
    }

    pub fn new(source: String, publisher: &BasicPublisher) -> Result<Self, FileError> {
        let root_node = publisher.get_node(Key(0));
        debug_assert_eq!(root_node.rule, Rules::Grammar);
        let root_children = root_node.get_children();
        debug_assert_eq!(root_children.len(), 1);
        let grammar_node = publisher.get_node(root_children[0]);

        let mut builder: MemberBuilder<FileMember> =
            MemberBuilder::new(&source, grammar_node.start_position, true);

        for child in sorted_children(publisher, grammar_node) {
            match child.rule {
                Rules::comment | Rules::multiline_comment => builder.comment(child),
                Rules::open_bracket | Rules::close_bracket | Rules::annotation_block => {}
                Rules::package => {
                    let pkg = Package::new(&source, publisher, child)?;
                    builder.member(child, pkg, FileMember::Package);
                }
                Rules::import_namespace => {
                    let ns = ImportNamespace::new(&source, publisher, child)?;
                    builder.member(child, ns, FileMember::ImportNamespace);
                }
                Rules::import_model => {
                    let im = ImportModel::new(&source, publisher, child)?;
                    builder.member(child, im, FileMember::ImportModel);
                }
                Rules::interface => {
                    let iface = Interface::new(&source, publisher, child)?;
                    builder.member(child, iface, FileMember::Interface);
                }
                Rules::type_collection => {
                    let tc = TypeCollection::new(&source, publisher, child)?;
                    builder.member(child, tc, FileMember::TypeCollection);
                }
                rule => {
                    return Err(FileError::UnexpectedNode(
                        rule,
                        "FidlFile::new".to_string(),
                    ));
                }
            }
        }
        let (members, header_comments) = builder.finish();
        let mut meta = NodeMeta::from_cst(grammar_node);
        meta.header_comments = header_comments;

        let mut file = Self {
            meta,
            source,
            path: None,
            members,
            ids: NodeIdGen::new(),
        };
        file.assign_ids();
        Ok(file)
    }

}

impl fmt::Debug for FidlFile {
    /// Prints the tree without the `source` string, which would drown everything else.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FidlFile")
            .field("path", &self.path)
            .field("members", &self.members)
            .finish()
    }
}

/// Gives every node a fresh [`NodeId`].
///
/// Runs once after construction. Phase 3 replaced a hand-written 130-line walk
/// with this; adding a node type now only requires touching `visit.rs`.
struct AssignIds {
    ids: NodeIdGen,
    /// When set, nodes that already have an id keep it.
    only_unassigned: bool,
}

impl crate::visit::VisitMut for AssignIds {
    fn visit_comment(&mut self, node: &mut Comment) {
        if !self.only_unassigned || !node.id.is_assigned() {
            node.id = self.ids.next_id();
        }
    }

    fn visit_meta(&mut self, meta: &mut NodeMeta) {
        if !self.only_unassigned || !meta.id.is_assigned() {
            meta.id = self.ids.next_id();
        }
        crate::visit::walk_meta(self, meta);
    }
}

impl FidlFile {
    fn assign_ids(&mut self) {
        use crate::visit::VisitMut;
        let mut assigner = AssignIds {
            ids: std::mem::replace(&mut self.ids, NodeIdGen::new()),
            only_unassigned: false,
        };
        assigner.visit_file(self);
        self.ids = assigner.ids;
    }

    /// Give ids to nodes that do not have one yet.
    ///
    /// Builders produce nodes with [`NodeId::UNASSIGNED`], so anything inserted
    /// into the tree needs this before it can be addressed by id or path. Existing
    /// ids are left alone, so handles held across the call stay valid. Idempotent.
    pub fn assign_missing_ids(&mut self) {
        use crate::visit::VisitMut;
        let mut assigner = AssignIds {
            ids: std::mem::replace(&mut self.ids, NodeIdGen::new()),
            only_unassigned: true,
        };
        assigner.visit_file(self);
        self.ids = assigner.ids;
    }

    /// Run an edit and then hand out ids to whatever it inserted.
    ///
    /// Preferred over calling [`Self::assign_missing_ids`] by hand, which is easy
    /// to forget after a structural change.
    pub fn edit<R>(&mut self, f: impl FnOnce(&mut Self) -> R) -> R {
        let result = f(self);
        self.assign_missing_ids();
        result
    }

    /// Number of nodes that have been assigned an id.
    pub fn node_count(&self) -> u32 {
        self.ids.peek().get().saturating_sub(1)
    }
}
