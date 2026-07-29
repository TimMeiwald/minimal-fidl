//! Mutable whole-tree traversal. See `DESIGN.md` §5.
//!
//! There is no immutable `Visit` counterpart: [`crate::node_ref::NodeRef`] and
//! `descendants()` already cover reading, and a second trait would be twice the
//! surface for no gain. Mutation needs the trait because `&mut` references cannot
//! be gathered into a `Vec` the way `NodeRef` gathers shared ones.
//!
//! Override the `visit_*` methods you care about; each defaults to the matching
//! `walk_*`, which recurses. Call `walk_*` from an override to keep descending.
//!
//! Invariant: every node routes through `self.visit_meta(&mut node.meta)`, never
//! `walk_meta` directly. A leaf that calls `walk_meta` skips the override and is
//! silently missed by any visitor that hooks `visit_meta` — which is exactly how
//! id assignment lost eight nodes before `every_id_in_the_tree_resolves` caught it.

use crate::{
    enumeration::EnumMember,
    fidl_file::FileMember,
    interface::InterfaceMember,
    method::{ParamList, ParamMember},
    node::{Comment, NodeMeta},
    structure::StructMember,
    type_collection::TypeCollectionMember,
    Annotation, Attribute, EnumValue, Enumeration, FidlFile, ImportModel, ImportNamespace,
    Interface, Method, Package, Structure, TypeCollection, TypeDef, VariableDeclaration, Version,
};

#[allow(unused_variables)]
pub trait VisitMut: Sized {
    fn visit_file(&mut self, node: &mut FidlFile) {
        walk_file(self, node);
    }
    fn visit_package(&mut self, node: &mut Package) {
        self.visit_meta(&mut node.meta);
    }
    fn visit_import_namespace(&mut self, node: &mut ImportNamespace) {
        self.visit_meta(&mut node.meta);
    }
    fn visit_import_model(&mut self, node: &mut ImportModel) {
        self.visit_meta(&mut node.meta);
    }
    fn visit_interface(&mut self, node: &mut Interface) {
        walk_interface(self, node);
    }
    fn visit_type_collection(&mut self, node: &mut TypeCollection) {
        walk_type_collection(self, node);
    }
    fn visit_version(&mut self, node: &mut Version) {
        self.visit_meta(&mut node.meta);
    }
    fn visit_method(&mut self, node: &mut Method) {
        walk_method(self, node);
    }
    fn visit_param_list(&mut self, node: &mut ParamList) {
        walk_param_list(self, node);
    }
    fn visit_attribute(&mut self, node: &mut Attribute) {
        walk_annotated(self, &mut node.meta, &mut node.annotations);
    }
    fn visit_structure(&mut self, node: &mut Structure) {
        walk_structure(self, node);
    }
    fn visit_enumeration(&mut self, node: &mut Enumeration) {
        walk_enumeration(self, node);
    }
    fn visit_enum_value(&mut self, node: &mut EnumValue) {
        walk_annotated(self, &mut node.meta, &mut node.annotations);
    }
    fn visit_typedef(&mut self, node: &mut TypeDef) {
        walk_annotated(self, &mut node.meta, &mut node.annotations);
    }
    fn visit_variable_declaration(&mut self, node: &mut VariableDeclaration) {
        walk_annotated(self, &mut node.meta, &mut node.annotations);
    }
    fn visit_annotation(&mut self, node: &mut Annotation) {
        self.visit_meta(&mut node.meta);
    }
    fn visit_comment(&mut self, node: &mut Comment) {}
    /// Called for every node's metadata before its structural children.
    fn visit_meta(&mut self, meta: &mut NodeMeta) {
        walk_meta(self, meta);
    }
}

pub fn walk_meta<V: VisitMut>(v: &mut V, meta: &mut NodeMeta) {
    for comment in &mut meta.leading_comments {
        v.visit_comment(comment);
    }
    for comment in &mut meta.header_comments {
        v.visit_comment(comment);
    }
    for comment in &mut meta.trailing_comments {
        v.visit_comment(comment);
    }
}

/// Shared by nodes whose only children are trivia and annotations.
pub fn walk_annotated<V: VisitMut>(
    v: &mut V,
    meta: &mut NodeMeta,
    annotations: &mut Vec<Annotation>,
) {
    v.visit_meta(meta);
    for annotation in annotations {
        v.visit_annotation(annotation);
    }
}

pub fn walk_file<V: VisitMut>(v: &mut V, node: &mut FidlFile) {
    v.visit_meta(&mut node.meta);
    for member in &mut node.members {
        match member {
            FileMember::Package(x) => v.visit_package(x),
            FileMember::ImportNamespace(x) => v.visit_import_namespace(x),
            FileMember::ImportModel(x) => v.visit_import_model(x),
            FileMember::Interface(x) => v.visit_interface(x),
            FileMember::TypeCollection(x) => v.visit_type_collection(x),
            FileMember::Comment(x) => v.visit_comment(x),
        }
    }
}

pub fn walk_interface<V: VisitMut>(v: &mut V, node: &mut Interface) {
    walk_annotated(v, &mut node.meta, &mut node.annotations);
    if let Some(version) = &mut node.version {
        v.visit_version(version);
    }
    for member in &mut node.members {
        match member {
            InterfaceMember::Method(x) => v.visit_method(x),
            InterfaceMember::Attribute(x) => v.visit_attribute(x),
            InterfaceMember::Structure(x) => v.visit_structure(x),
            InterfaceMember::Enumeration(x) => v.visit_enumeration(x),
            InterfaceMember::TypeDef(x) => v.visit_typedef(x),
            InterfaceMember::Comment(x) => v.visit_comment(x),
        }
    }
}

pub fn walk_type_collection<V: VisitMut>(v: &mut V, node: &mut TypeCollection) {
    walk_annotated(v, &mut node.meta, &mut node.annotations);
    if let Some(version) = &mut node.version {
        v.visit_version(version);
    }
    for member in &mut node.members {
        match member {
            TypeCollectionMember::TypeDef(x) => v.visit_typedef(x),
            TypeCollectionMember::Structure(x) => v.visit_structure(x),
            TypeCollectionMember::Enumeration(x) => v.visit_enumeration(x),
            TypeCollectionMember::Comment(x) => v.visit_comment(x),
        }
    }
}

pub fn walk_method<V: VisitMut>(v: &mut V, node: &mut Method) {
    walk_annotated(v, &mut node.meta, &mut node.annotations);
    v.visit_param_list(&mut node.inputs);
    v.visit_param_list(&mut node.outputs);
}

pub fn walk_param_list<V: VisitMut>(v: &mut V, node: &mut ParamList) {
    walk_annotated(v, &mut node.meta, &mut node.annotations);
    for member in &mut node.members {
        match member {
            ParamMember::Param(x) => v.visit_variable_declaration(x),
            ParamMember::Comment(x) => v.visit_comment(x),
        }
    }
}

pub fn walk_structure<V: VisitMut>(v: &mut V, node: &mut Structure) {
    walk_annotated(v, &mut node.meta, &mut node.annotations);
    for member in &mut node.members {
        match member {
            StructMember::Field(x) => v.visit_variable_declaration(x),
            StructMember::Comment(x) => v.visit_comment(x),
        }
    }
}

pub fn walk_enumeration<V: VisitMut>(v: &mut V, node: &mut Enumeration) {
    walk_annotated(v, &mut node.meta, &mut node.annotations);
    for member in &mut node.members {
        match member {
            EnumMember::Value(x) => v.visit_enum_value(x),
            EnumMember::Comment(x) => v.visit_comment(x),
        }
    }
}
