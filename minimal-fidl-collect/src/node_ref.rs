//! Uniform traversal over the AST, shared ([`NodeRef`]) and mutable
//! ([`NodeRefMut`]).
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

/// A mutable reference to any node in the tree.
///
/// The mutable mirror of [`NodeRef`]. Two differences follow from `&mut` not being
/// shareable:
///
/// - it cannot be `Copy`, so [`Self::children_mut`] takes `self` by value;
/// - there is no mutable `Descendants`. Whole-tree rewrites go through
///   [`crate::VisitMut`] and single-node lookup through [`FidlFile::get_mut`];
///   between them nothing is missing, and a mutable iterator is the genuinely
///   awkward part.
///
/// Gathering a node's children into a `Vec` *is* possible, despite the usual
/// objection: they live in disjoint fields (`members`, `annotations`, the three
/// comment lists on `meta`), and disjoint `&mut` borrows may coexist.
pub enum NodeRefMut<'a> {
    File(&'a mut FidlFile),
    Package(&'a mut Package),
    ImportNamespace(&'a mut ImportNamespace),
    ImportModel(&'a mut ImportModel),
    Interface(&'a mut Interface),
    TypeCollection(&'a mut TypeCollection),
    Version(&'a mut Version),
    Method(&'a mut Method),
    ParamList(&'a mut ParamList),
    Attribute(&'a mut Attribute),
    Structure(&'a mut Structure),
    Enumeration(&'a mut Enumeration),
    EnumValue(&'a mut EnumValue),
    TypeDef(&'a mut TypeDef),
    VariableDeclaration(&'a mut VariableDeclaration),
    Annotation(&'a mut Annotation),
    Comment(&'a mut Comment),
}

/// [`dispatch_meta`] for [`NodeRefMut`].
macro_rules! dispatch_meta_mut {
    ($self:expr, $n:ident => $body:expr) => {
        match $self {
            NodeRefMut::File($n) => $body,
            NodeRefMut::Package($n) => $body,
            NodeRefMut::ImportNamespace($n) => $body,
            NodeRefMut::ImportModel($n) => $body,
            NodeRefMut::Interface($n) => $body,
            NodeRefMut::TypeCollection($n) => $body,
            NodeRefMut::Version($n) => $body,
            NodeRefMut::Method($n) => $body,
            NodeRefMut::ParamList($n) => $body,
            NodeRefMut::Attribute($n) => $body,
            NodeRefMut::Structure($n) => $body,
            NodeRefMut::Enumeration($n) => $body,
            NodeRefMut::EnumValue($n) => $body,
            NodeRefMut::TypeDef($n) => $body,
            NodeRefMut::VariableDeclaration($n) => $body,
            NodeRefMut::Annotation($n) => $body,
            NodeRefMut::Comment(_) => unreachable!("comments are handled by the caller"),
        }
    };
}

/// A node's own comments, in the same order [`NodeRef::children`] yields them.
fn meta_comments_mut(meta: &mut NodeMeta) -> impl Iterator<Item = NodeRefMut<'_>> {
    meta.leading_comments
        .iter_mut()
        .chain(meta.header_comments.iter_mut())
        .chain(meta.trailing_comments.iter_mut())
        .map(NodeRefMut::Comment)
}

fn annotations_mut_iter(annotations: &mut [Annotation]) -> impl Iterator<Item = NodeRefMut<'_>> {
    annotations.iter_mut().map(NodeRefMut::Annotation)
}

impl<'a> NodeRefMut<'a> {
    pub fn id(&self) -> NodeId {
        match self {
            NodeRefMut::Comment(c) => c.id,
            other => dispatch_meta_mut!(other, n => n.meta().id),
        }
    }

    pub fn span(&self) -> Option<Span> {
        match self {
            NodeRefMut::Comment(c) => c.span,
            other => dispatch_meta_mut!(other, n => n.meta().span),
        }
    }

    /// `None` for comments, which have no [`NodeMeta`].
    pub fn meta_mut(&mut self) -> Option<&mut NodeMeta> {
        match self {
            NodeRefMut::Comment(_) => None,
            other => Some(dispatch_meta_mut!(other, n => n.meta_mut())),
        }
    }

    /// Mark the node modified, so `Mode::Preserve` reformats it rather than
    /// reusing stale source text. A no-op for comments, which have no flag.
    pub fn mark_dirty(&mut self) {
        if let Some(meta) = self.meta_mut() {
            meta.dirty = true;
        }
    }

    pub fn kind_name(&self) -> &'static str {
        match self {
            NodeRefMut::File(_) => "file",
            NodeRefMut::Package(_) => "package",
            NodeRefMut::ImportNamespace(_) => "import_namespace",
            NodeRefMut::ImportModel(_) => "import_model",
            NodeRefMut::Interface(_) => "interface",
            NodeRefMut::TypeCollection(_) => "type_collection",
            NodeRefMut::Version(_) => "version",
            NodeRefMut::Method(_) => "method",
            NodeRefMut::ParamList(_) => "param_list",
            NodeRefMut::Attribute(_) => "attribute",
            NodeRefMut::Structure(_) => "struct",
            NodeRefMut::Enumeration(_) => "enum",
            NodeRefMut::EnumValue(_) => "enum_value",
            NodeRefMut::TypeDef(_) => "typedef",
            NodeRefMut::VariableDeclaration(_) => "variable_declaration",
            NodeRefMut::Annotation(_) => "annotation",
            NodeRefMut::Comment(_) => "comment",
        }
    }

    /// The node's declared name, where it has one.
    pub fn name(&self) -> Option<&str> {
        match self {
            NodeRefMut::Interface(n) => Some(&n.name),
            NodeRefMut::TypeCollection(n) => Some(&n.name),
            NodeRefMut::Method(n) => Some(&n.name),
            NodeRefMut::Attribute(n) => Some(&n.name),
            NodeRefMut::Structure(n) => Some(&n.name),
            NodeRefMut::Enumeration(n) => Some(&n.name),
            NodeRefMut::EnumValue(n) => Some(&n.name),
            NodeRefMut::TypeDef(n) => Some(&n.name),
            NodeRefMut::VariableDeclaration(n) => Some(&n.name),
            NodeRefMut::Annotation(n) => Some(&n.name),
            _ => None,
        }
    }

    /// The node's annotations, or `None` for nodes that cannot carry them.
    pub fn annotations_mut(&mut self) -> Option<&mut Vec<Annotation>> {
        match self {
            NodeRefMut::Interface(n) => Some(&mut n.annotations),
            NodeRefMut::TypeCollection(n) => Some(&mut n.annotations),
            NodeRefMut::Method(n) => Some(&mut n.annotations),
            NodeRefMut::ParamList(n) => Some(&mut n.annotations),
            NodeRefMut::Attribute(n) => Some(&mut n.annotations),
            NodeRefMut::Structure(n) => Some(&mut n.annotations),
            NodeRefMut::Enumeration(n) => Some(&mut n.annotations),
            NodeRefMut::EnumValue(n) => Some(&mut n.annotations),
            NodeRefMut::TypeDef(n) => Some(&mut n.annotations),
            NodeRefMut::VariableDeclaration(n) => Some(&mut n.annotations),
            _ => None,
        }
    }

    /// Direct children in the same order as [`NodeRef::children`].
    ///
    /// Takes `self` by value: the returned borrows are carved out of it, so it
    /// cannot survive the call.
    pub fn children_mut(self) -> Vec<NodeRefMut<'a>> {
        let mut out: Vec<NodeRefMut<'a>> = Vec::new();
        match self {
            NodeRefMut::File(f) => {
                out.extend(meta_comments_mut(&mut f.meta));
                out.extend(f.members.iter_mut().map(|m| match m {
                    FileMember::Package(x) => NodeRefMut::Package(x),
                    FileMember::ImportNamespace(x) => NodeRefMut::ImportNamespace(x),
                    FileMember::ImportModel(x) => NodeRefMut::ImportModel(x),
                    FileMember::Interface(x) => NodeRefMut::Interface(x),
                    FileMember::TypeCollection(x) => NodeRefMut::TypeCollection(x),
                    FileMember::Comment(x) => NodeRefMut::Comment(x),
                }));
            }
            NodeRefMut::Interface(i) => {
                out.extend(meta_comments_mut(&mut i.meta));
                out.extend(annotations_mut_iter(&mut i.annotations));
                out.extend(i.version.iter_mut().map(NodeRefMut::Version));
                out.extend(i.members.iter_mut().map(|m| match m {
                    InterfaceMember::Method(x) => NodeRefMut::Method(x),
                    InterfaceMember::Attribute(x) => NodeRefMut::Attribute(x),
                    InterfaceMember::Structure(x) => NodeRefMut::Structure(x),
                    InterfaceMember::Enumeration(x) => NodeRefMut::Enumeration(x),
                    InterfaceMember::TypeDef(x) => NodeRefMut::TypeDef(x),
                    InterfaceMember::Comment(x) => NodeRefMut::Comment(x),
                }));
            }
            NodeRefMut::TypeCollection(t) => {
                out.extend(meta_comments_mut(&mut t.meta));
                out.extend(annotations_mut_iter(&mut t.annotations));
                out.extend(t.version.iter_mut().map(NodeRefMut::Version));
                out.extend(t.members.iter_mut().map(|m| match m {
                    TypeCollectionMember::TypeDef(x) => NodeRefMut::TypeDef(x),
                    TypeCollectionMember::Structure(x) => NodeRefMut::Structure(x),
                    TypeCollectionMember::Enumeration(x) => NodeRefMut::Enumeration(x),
                    TypeCollectionMember::Comment(x) => NodeRefMut::Comment(x),
                }));
            }
            NodeRefMut::Method(m) => {
                out.extend(meta_comments_mut(&mut m.meta));
                out.extend(annotations_mut_iter(&mut m.annotations));
                out.push(NodeRefMut::ParamList(&mut m.inputs));
                out.push(NodeRefMut::ParamList(&mut m.outputs));
            }
            NodeRefMut::ParamList(p) => {
                out.extend(meta_comments_mut(&mut p.meta));
                out.extend(annotations_mut_iter(&mut p.annotations));
                out.extend(p.members.iter_mut().map(|m| match m {
                    ParamMember::Param(x) => NodeRefMut::VariableDeclaration(x),
                    ParamMember::Comment(x) => NodeRefMut::Comment(x),
                }));
            }
            NodeRefMut::Structure(s) => {
                out.extend(meta_comments_mut(&mut s.meta));
                out.extend(annotations_mut_iter(&mut s.annotations));
                out.extend(s.members.iter_mut().map(|m| match m {
                    StructMember::Field(x) => NodeRefMut::VariableDeclaration(x),
                    StructMember::Comment(x) => NodeRefMut::Comment(x),
                }));
            }
            NodeRefMut::Enumeration(e) => {
                out.extend(meta_comments_mut(&mut e.meta));
                out.extend(annotations_mut_iter(&mut e.annotations));
                out.extend(e.members.iter_mut().map(|m| match m {
                    EnumMember::Value(x) => NodeRefMut::EnumValue(x),
                    EnumMember::Comment(x) => NodeRefMut::Comment(x),
                }));
            }
            // Leaves: trivia and annotations only.
            NodeRefMut::Package(n) => out.extend(meta_comments_mut(&mut n.meta)),
            NodeRefMut::ImportNamespace(n) => out.extend(meta_comments_mut(&mut n.meta)),
            NodeRefMut::ImportModel(n) => out.extend(meta_comments_mut(&mut n.meta)),
            NodeRefMut::Version(n) => out.extend(meta_comments_mut(&mut n.meta)),
            NodeRefMut::Annotation(n) => out.extend(meta_comments_mut(&mut n.meta)),
            NodeRefMut::Attribute(n) => {
                out.extend(meta_comments_mut(&mut n.meta));
                out.extend(annotations_mut_iter(&mut n.annotations));
            }
            NodeRefMut::EnumValue(n) => {
                out.extend(meta_comments_mut(&mut n.meta));
                out.extend(annotations_mut_iter(&mut n.annotations));
            }
            NodeRefMut::TypeDef(n) => {
                out.extend(meta_comments_mut(&mut n.meta));
                out.extend(annotations_mut_iter(&mut n.annotations));
            }
            NodeRefMut::VariableDeclaration(n) => {
                out.extend(meta_comments_mut(&mut n.meta));
                out.extend(annotations_mut_iter(&mut n.annotations));
            }
            NodeRefMut::Comment(_) => {}
        }
        out
    }
}

impl FidlFile {
    pub fn as_node(&self) -> NodeRef<'_> {
        NodeRef::File(self)
    }

    pub fn as_node_mut(&mut self) -> NodeRefMut<'_> {
        NodeRefMut::File(self)
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

    /// Resolve a [`NodeId`] to the node it names, mutably.
    ///
    /// Recursive descent rather than an iterator scan: the borrow has to move
    /// down the tree and stop at the match, which is exactly what a mutable
    /// `Descendants` cannot express.
    ///
    /// The returned node is marked dirty, on the same conservative rule as the
    /// `*_mut()` accessors: handing out a `&mut` counts as a modification whether
    /// or not the caller writes through it (`DESIGN.md` §8). Use [`Self::get`]
    /// when only reading.
    ///
    /// Prefer this over looking a node up by name when you already hold its id:
    /// duplicate names are legal enough to parse, so a name lookup can resolve to
    /// a different node than the one the id names.
    pub fn get_mut(&mut self, id: NodeId) -> Option<NodeRefMut<'_>> {
        fn descend<'a>(node: NodeRefMut<'a>, target: NodeId) -> Option<NodeRefMut<'a>> {
            if node.id() == target {
                return Some(node);
            }
            for child in node.children_mut() {
                if let Some(found) = descend(child, target) {
                    return Some(found);
                }
            }
            None
        }

        let mut found = descend(self.as_node_mut(), id)?;
        found.mark_dirty();
        Some(found)
    }
}
