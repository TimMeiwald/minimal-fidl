//! Uniform read-only traversal over the AST.
//!
//! [`NodeRef`] is the object-safe replacement for the abandoned `Ordered` trait:
//! an enum can hold heterogeneous children, `impl Trait` in a trait method cannot.
//! See `DESIGN.md` §5.

use crate::{
    enumeration::EnumMember,
    fidl_file::FileMember,
    interface::InterfaceMember,
    method::{ParamList, ParamMember},
    node::{AstNode, Comment, NodeId, NodeMeta, Span},
    structure::StructMember,
    type_collection::TypeCollectionMember,
    Annotation, Attribute, EnumValue, Enumeration, FidlFile, ImportModel, ImportNamespace,
    Interface, Method, Package, Structure, TypeCollection, TypeDef, VariableDeclaration, Version,
};

/// A borrowed reference to any node in the tree.
#[derive(Debug, Clone, Copy)]
pub enum NodeRef<'a> {
    File(&'a FidlFile),
    Package(&'a Package),
    ImportNamespace(&'a ImportNamespace),
    ImportModel(&'a ImportModel),
    Interface(&'a Interface),
    TypeCollection(&'a TypeCollection),
    Version(&'a Version),
    Method(&'a Method),
    ParamList(&'a ParamList),
    Attribute(&'a Attribute),
    Structure(&'a Structure),
    Enumeration(&'a Enumeration),
    EnumValue(&'a EnumValue),
    TypeDef(&'a TypeDef),
    VariableDeclaration(&'a VariableDeclaration),
    Annotation(&'a Annotation),
    Comment(&'a Comment),
}

/// Applies the same expression to every variant that carries a [`NodeMeta`].
///
/// Comments are excluded: they hold `id` and `span` directly rather than through
/// a `NodeMeta`, so they have no `meta()` method to call.
macro_rules! dispatch_meta {
    ($self:expr, $n:ident => $body:expr) => {
        match $self {
            NodeRef::File($n) => $body,
            NodeRef::Package($n) => $body,
            NodeRef::ImportNamespace($n) => $body,
            NodeRef::ImportModel($n) => $body,
            NodeRef::Interface($n) => $body,
            NodeRef::TypeCollection($n) => $body,
            NodeRef::Version($n) => $body,
            NodeRef::Method($n) => $body,
            NodeRef::ParamList($n) => $body,
            NodeRef::Attribute($n) => $body,
            NodeRef::Structure($n) => $body,
            NodeRef::Enumeration($n) => $body,
            NodeRef::EnumValue($n) => $body,
            NodeRef::TypeDef($n) => $body,
            NodeRef::VariableDeclaration($n) => $body,
            NodeRef::Annotation($n) => $body,
            NodeRef::Comment(_) => unreachable!("comments are handled by the caller"),
        }
    };
}

impl<'a> NodeRef<'a> {
    pub fn id(&self) -> NodeId {
        match self {
            NodeRef::Comment(c) => c.id,
            other => dispatch_meta!(other, n => n.meta().id),
        }
    }

    pub fn span(&self) -> Option<Span> {
        match self {
            NodeRef::Comment(c) => c.span,
            other => dispatch_meta!(other, n => n.meta().span),
        }
    }

    /// `None` for comments, which have no [`NodeMeta`].
    pub fn meta(&self) -> Option<&'a NodeMeta> {
        match self {
            NodeRef::Comment(_) => None,
            other => Some(dispatch_meta!(other, n => n.meta())),
        }
    }

    /// The node's declared name, where it has one.
    pub fn name(&self) -> Option<&'a str> {
        match self {
            NodeRef::Interface(n) => Some(&n.name),
            NodeRef::TypeCollection(n) => Some(&n.name),
            NodeRef::Method(n) => Some(&n.name),
            NodeRef::Attribute(n) => Some(&n.name),
            NodeRef::Structure(n) => Some(&n.name),
            NodeRef::Enumeration(n) => Some(&n.name),
            NodeRef::EnumValue(n) => Some(&n.name),
            NodeRef::TypeDef(n) => Some(&n.name),
            NodeRef::VariableDeclaration(n) => Some(&n.name),
            NodeRef::Annotation(n) => Some(&n.name),
            _ => None,
        }
    }

    pub fn kind_name(&self) -> &'static str {
        match self {
            NodeRef::File(_) => "file",
            NodeRef::Package(_) => "package",
            NodeRef::ImportNamespace(_) => "import_namespace",
            NodeRef::ImportModel(_) => "import_model",
            NodeRef::Interface(_) => "interface",
            NodeRef::TypeCollection(_) => "type_collection",
            NodeRef::Version(_) => "version",
            NodeRef::Method(_) => "method",
            NodeRef::ParamList(_) => "param_list",
            NodeRef::Attribute(_) => "attribute",
            NodeRef::Structure(_) => "struct",
            NodeRef::Enumeration(_) => "enum",
            NodeRef::EnumValue(_) => "enum_value",
            NodeRef::TypeDef(_) => "typedef",
            NodeRef::VariableDeclaration(_) => "variable_declaration",
            NodeRef::Annotation(_) => "annotation",
            NodeRef::Comment(_) => "comment",
        }
    }

    /// The node's annotations, or an empty slice for nodes that cannot carry them.
    pub fn annotations(&self) -> &'a [Annotation] {
        match self {
            NodeRef::Interface(n) => &n.annotations,
            NodeRef::TypeCollection(n) => &n.annotations,
            NodeRef::Method(n) => &n.annotations,
            NodeRef::ParamList(n) => &n.annotations,
            NodeRef::Attribute(n) => &n.annotations,
            NodeRef::Structure(n) => &n.annotations,
            NodeRef::Enumeration(n) => &n.annotations,
            NodeRef::EnumValue(n) => &n.annotations,
            NodeRef::TypeDef(n) => &n.annotations,
            NodeRef::VariableDeclaration(n) => &n.annotations,
            _ => &[],
        }
    }

    /// Direct children in traversal order: trivia, then annotations, then
    /// structural children.
    ///
    /// Returns a `Vec` rather than an iterator: children come from several
    /// differently-typed fields, so they have to be gathered anyway, and `.fidl`
    /// nodes have few children.
    pub fn children(&self) -> Vec<NodeRef<'a>> {
        let mut out: Vec<NodeRef<'a>> = Vec::new();

        if let Some(meta) = self.meta() {
            out.extend(meta.leading_comments.iter().map(NodeRef::Comment));
            out.extend(meta.header_comments.iter().map(NodeRef::Comment));
            out.extend(meta.trailing_comments.iter().map(NodeRef::Comment));
        }
        out.extend(self.annotations().iter().map(NodeRef::Annotation));

        match self {
            NodeRef::File(f) => {
                out.extend(f.members.iter().map(|m| match m {
                    FileMember::Package(x) => NodeRef::Package(x),
                    FileMember::ImportNamespace(x) => NodeRef::ImportNamespace(x),
                    FileMember::ImportModel(x) => NodeRef::ImportModel(x),
                    FileMember::Interface(x) => NodeRef::Interface(x),
                    FileMember::TypeCollection(x) => NodeRef::TypeCollection(x),
                    FileMember::Comment(x) => NodeRef::Comment(x),
                }));
            }
            NodeRef::Interface(i) => {
                out.extend(i.version.iter().map(NodeRef::Version));
                out.extend(i.members.iter().map(|m| match m {
                    InterfaceMember::Method(x) => NodeRef::Method(x),
                    InterfaceMember::Attribute(x) => NodeRef::Attribute(x),
                    InterfaceMember::Structure(x) => NodeRef::Structure(x),
                    InterfaceMember::Enumeration(x) => NodeRef::Enumeration(x),
                    InterfaceMember::TypeDef(x) => NodeRef::TypeDef(x),
                    InterfaceMember::Comment(x) => NodeRef::Comment(x),
                }));
            }
            NodeRef::TypeCollection(t) => {
                out.extend(t.version.iter().map(NodeRef::Version));
                out.extend(t.members.iter().map(|m| match m {
                    TypeCollectionMember::TypeDef(x) => NodeRef::TypeDef(x),
                    TypeCollectionMember::Structure(x) => NodeRef::Structure(x),
                    TypeCollectionMember::Enumeration(x) => NodeRef::Enumeration(x),
                    TypeCollectionMember::Comment(x) => NodeRef::Comment(x),
                }));
            }
            NodeRef::Method(m) => {
                out.push(NodeRef::ParamList(&m.inputs));
                out.push(NodeRef::ParamList(&m.outputs));
            }
            NodeRef::ParamList(p) => {
                out.extend(p.members.iter().map(|m| match m {
                    ParamMember::Param(x) => NodeRef::VariableDeclaration(x),
                    ParamMember::Comment(x) => NodeRef::Comment(x),
                }));
            }
            NodeRef::Structure(s) => {
                out.extend(s.members.iter().map(|m| match m {
                    StructMember::Field(x) => NodeRef::VariableDeclaration(x),
                    StructMember::Comment(x) => NodeRef::Comment(x),
                }));
            }
            NodeRef::Enumeration(e) => {
                out.extend(e.members.iter().map(|m| match m {
                    EnumMember::Value(x) => NodeRef::EnumValue(x),
                    EnumMember::Comment(x) => NodeRef::Comment(x),
                }));
            }
            // Leaves: any trivia and annotations were already pushed above.
            NodeRef::Package(_)
            | NodeRef::ImportNamespace(_)
            | NodeRef::ImportModel(_)
            | NodeRef::Version(_)
            | NodeRef::Attribute(_)
            | NodeRef::EnumValue(_)
            | NodeRef::TypeDef(_)
            | NodeRef::VariableDeclaration(_)
            | NodeRef::Annotation(_)
            | NodeRef::Comment(_) => {}
        }
        out
    }

    /// Every node beneath this one, pre-order, excluding this node.
    pub fn descendants(&self) -> Descendants<'a> {
        let mut stack = self.children();
        stack.reverse();
        Descendants { stack }
    }

    /// This node followed by every node beneath it, pre-order.
    pub fn self_and_descendants(&self) -> Descendants<'a> {
        Descendants { stack: vec![*self] }
    }
}

/// Pre-order iterator produced by [`NodeRef::descendants`].
pub struct Descendants<'a> {
    stack: Vec<NodeRef<'a>>,
}

impl<'a> Iterator for Descendants<'a> {
    type Item = NodeRef<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.stack.pop()?;
        let mut children = node.children();
        children.reverse();
        self.stack.extend(children);
        Some(node)
    }
}

impl FidlFile {
    pub fn as_node(&self) -> NodeRef<'_> {
        NodeRef::File(self)
    }

    /// Every node in the file, pre-order, starting with the file itself.
    pub fn nodes(&self) -> Descendants<'_> {
        self.as_node().self_and_descendants()
    }

    /// Resolve a [`NodeId`] to the node it names.
    ///
    /// A linear scan. `.fidl` files hold hundreds of nodes, so this is cheap; if
    /// profiling ever says otherwise, `DESIGN.md` §6 describes the index to add.
    /// Returns `None` if the node has been removed from the tree.
    pub fn get(&self, id: NodeId) -> Option<NodeRef<'_>> {
        self.nodes().find(|n| n.id() == id)
    }
}
