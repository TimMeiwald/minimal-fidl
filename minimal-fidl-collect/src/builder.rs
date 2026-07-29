//! Constructing nodes from scratch. See `DESIGN.md` §7.
//!
//! Every node built here has `span: None` and `id: NodeId::UNASSIGNED`; it gets a
//! real id when [`crate::FidlFile::assign_missing_ids`] next runs. Synthesised
//! nodes are always formatted on output, since there is no original text to reuse.

use std::path::PathBuf;

use crate::{
    enumeration::EnumMember,
    interface::InterfaceMember,
    method::{ParamList, ParamMember},
    node::{Comment, NodeMeta},
    structure::StructMember,
    type_collection::TypeCollectionMember,
    Annotation, Attribute, EnumValue, Enumeration, ImportModel, ImportNamespace, Interface, Method,
    Package, Structure, TypeCollection, TypeDef, VariableDeclaration, Version,
};

/// Fresh metadata for a synthesised node.
fn new_meta() -> NodeMeta {
    NodeMeta {
        dirty: true,
        ..NodeMeta::default()
    }
}

impl Package {
    /// `package a.b.c`, from the dot-separated segments.
    pub fn create(path: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            meta: new_meta(),
            path: path.into_iter().map(Into::into).collect(),
        }
    }

    /// `package a.b.c` from `"a.b.c"`.
    pub fn parse(path: &str) -> Self {
        Self::create(path.split('.'))
    }
}

impl ImportModel {
    /// `import model "path"`.
    pub fn create(file_path: impl Into<PathBuf>) -> Self {
        Self {
            meta: new_meta(),
            file_path: file_path.into(),
        }
    }
}

impl ImportNamespace {
    /// `import a.b.* from "path"`.
    ///
    /// Always a wildcard: the grammar's `import_namespace` rule requires the `.*`
    /// — it is a mandatory element, not an option — so an import built without it
    /// would print text that cannot be read back. The `wildcard` field stays
    /// public for anyone who needs to model one anyway.
    pub fn create(
        import: impl IntoIterator<Item = impl Into<String>>,
        from: impl Into<PathBuf>,
    ) -> Self {
        Self {
            meta: new_meta(),
            from: from.into(),
            import: import.into_iter().map(Into::into).collect(),
            wildcard: true,
        }
    }
}

impl VariableDeclaration {
    /// A `Type name` pair, as used for struct fields and method parameters.
    pub fn create(name: impl Into<String>, type_n: impl Into<String>) -> Self {
        Self {
            meta: new_meta(),
            annotations: Vec::new(),
            type_n: type_n.into(),
            name: name.into(),
            is_array: false,
        }
    }

    pub fn array(name: impl Into<String>, type_n: impl Into<String>) -> Self {
        Self {
            is_array: true,
            ..Self::create(name, type_n)
        }
    }
}

impl Attribute {
    pub fn create(name: impl Into<String>, type_n: impl Into<String>) -> Self {
        Self {
            meta: new_meta(),
            annotations: Vec::new(),
            name: name.into(),
            type_n: type_n.into(),
        }
    }
}

impl TypeDef {
    pub fn create(name: impl Into<String>, type_n: impl Into<String>) -> Self {
        Self {
            meta: new_meta(),
            annotations: Vec::new(),
            name: name.into(),
            type_n: type_n.into(),
            is_array: false,
        }
    }

    /// `typedef name is Type[]`.
    pub fn array(name: impl Into<String>, type_n: impl Into<String>) -> Self {
        Self {
            is_array: true,
            ..Self::create(name, type_n)
        }
    }
}

impl EnumValue {
    pub fn create(name: impl Into<String>) -> Self {
        Self {
            meta: new_meta(),
            annotations: Vec::new(),
            name: name.into(),
            value: None,
        }
    }

    pub fn with_value(name: impl Into<String>, value: u64) -> Self {
        Self {
            value: Some(value),
            ..Self::create(name)
        }
    }
}

impl Version {
    pub fn create(major: u32, minor: u32) -> Self {
        Self {
            meta: new_meta(),
            major: Some(major),
            minor: Some(minor),
        }
    }
}

/// Builds a [`Method`] and its two parameter lists.
pub struct MethodBuilder {
    method: Method,
}

impl Method {
    pub fn builder(name: impl Into<String>) -> MethodBuilder {
        MethodBuilder {
            method: Method {
                meta: new_meta(),
                annotations: Vec::new(),
                name: name.into(),
                inputs: ParamList {
                    meta: new_meta(),
                    annotations: Vec::new(),
                    members: Vec::new(),
                },
                outputs: ParamList {
                    meta: new_meta(),
                    annotations: Vec::new(),
                    members: Vec::new(),
                },
            },
        }
    }
}

impl MethodBuilder {
    pub fn input(mut self, name: impl Into<String>, type_n: impl Into<String>) -> Self {
        self.method
            .inputs
            .members
            .push(ParamMember::Param(VariableDeclaration::create(name, type_n)));
        self
    }

    pub fn output(mut self, name: impl Into<String>, type_n: impl Into<String>) -> Self {
        self.method
            .outputs
            .members
            .push(ParamMember::Param(VariableDeclaration::create(name, type_n)));
        self
    }

    pub fn annotation(mut self, name: impl Into<String>, contents: impl Into<String>) -> Self {
        self.method
            .annotations
            .push(Annotation::create(name, contents));
        self
    }

    /// A comment placed directly above the method.
    pub fn doc(mut self, text: impl Into<String>) -> Self {
        self.method.meta.leading_comments.push(Comment::line(text));
        self
    }

    pub fn build(self) -> Method {
        self.method
    }
}

/// Builds an [`Interface`].
pub struct InterfaceBuilder {
    interface: Interface,
}

impl Interface {
    pub fn builder(name: impl Into<String>) -> InterfaceBuilder {
        InterfaceBuilder {
            interface: Interface {
                meta: new_meta(),
                annotations: Vec::new(),
                name: name.into(),
                version: None,
                members: Vec::new(),
            },
        }
    }
}

impl InterfaceBuilder {
    pub fn version(mut self, major: u32, minor: u32) -> Self {
        self.interface.version = Some(Version::create(major, minor));
        self
    }

    pub fn annotation(mut self, name: impl Into<String>, contents: impl Into<String>) -> Self {
        self.interface
            .annotations
            .push(Annotation::create(name, contents));
        self
    }

    pub fn doc(mut self, text: impl Into<String>) -> Self {
        self.interface.meta.leading_comments.push(Comment::line(text));
        self
    }

    pub fn method(mut self, method: Method) -> Self {
        self.interface.members.push(InterfaceMember::Method(method));
        self
    }

    pub fn attribute(mut self, attribute: Attribute) -> Self {
        self.interface
            .members
            .push(InterfaceMember::Attribute(attribute));
        self
    }

    pub fn structure(mut self, structure: Structure) -> Self {
        self.interface
            .members
            .push(InterfaceMember::Structure(structure));
        self
    }

    pub fn enumeration(mut self, enumeration: Enumeration) -> Self {
        self.interface
            .members
            .push(InterfaceMember::Enumeration(enumeration));
        self
    }

    pub fn typedef(mut self, typedef: TypeDef) -> Self {
        self.interface
            .members
            .push(InterfaceMember::TypeDef(typedef));
        self
    }

    pub fn build(self) -> Interface {
        self.interface
    }
}

/// Builds a [`Structure`].
pub struct StructureBuilder {
    structure: Structure,
}

impl Structure {
    pub fn builder(name: impl Into<String>) -> StructureBuilder {
        StructureBuilder {
            structure: Structure {
                meta: new_meta(),
                annotations: Vec::new(),
                name: name.into(),
                members: Vec::new(),
            },
        }
    }
}

impl StructureBuilder {
    pub fn field(mut self, name: impl Into<String>, type_n: impl Into<String>) -> Self {
        self.structure
            .members
            .push(StructMember::Field(VariableDeclaration::create(
                name, type_n,
            )));
        self
    }

    pub fn annotation(mut self, name: impl Into<String>, contents: impl Into<String>) -> Self {
        self.structure
            .annotations
            .push(Annotation::create(name, contents));
        self
    }

    pub fn build(self) -> Structure {
        self.structure
    }
}

/// Builds an [`Enumeration`].
pub struct EnumerationBuilder {
    enumeration: Enumeration,
}

impl Enumeration {
    pub fn builder(name: impl Into<String>) -> EnumerationBuilder {
        EnumerationBuilder {
            enumeration: Enumeration {
                meta: new_meta(),
                annotations: Vec::new(),
                name: name.into(),
                members: Vec::new(),
            },
        }
    }
}

impl EnumerationBuilder {
    pub fn value(mut self, name: impl Into<String>) -> Self {
        self.enumeration
            .members
            .push(EnumMember::Value(EnumValue::create(name)));
        self
    }

    pub fn value_with(mut self, name: impl Into<String>, value: u64) -> Self {
        self.enumeration
            .members
            .push(EnumMember::Value(EnumValue::with_value(name, value)));
        self
    }

    pub fn annotation(mut self, name: impl Into<String>, contents: impl Into<String>) -> Self {
        self.enumeration
            .annotations
            .push(Annotation::create(name, contents));
        self
    }

    pub fn build(self) -> Enumeration {
        self.enumeration
    }
}

/// Builds a [`TypeCollection`].
pub struct TypeCollectionBuilder {
    type_collection: TypeCollection,
}

impl TypeCollection {
    pub fn builder(name: impl Into<String>) -> TypeCollectionBuilder {
        TypeCollectionBuilder {
            type_collection: TypeCollection {
                meta: new_meta(),
                annotations: Vec::new(),
                name: name.into(),
                version: None,
                members: Vec::new(),
            },
        }
    }
}

impl TypeCollectionBuilder {
    pub fn version(mut self, major: u32, minor: u32) -> Self {
        self.type_collection.version = Some(Version::create(major, minor));
        self
    }

    pub fn typedef(mut self, typedef: TypeDef) -> Self {
        self.type_collection
            .members
            .push(TypeCollectionMember::TypeDef(typedef));
        self
    }

    pub fn structure(mut self, structure: Structure) -> Self {
        self.type_collection
            .members
            .push(TypeCollectionMember::Structure(structure));
        self
    }

    pub fn enumeration(mut self, enumeration: Enumeration) -> Self {
        self.type_collection
            .members
            .push(TypeCollectionMember::Enumeration(enumeration));
        self
    }

    pub fn build(self) -> TypeCollection {
        self.type_collection
    }
}
