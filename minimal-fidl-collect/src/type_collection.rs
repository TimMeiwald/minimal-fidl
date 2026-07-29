use crate::{
    annotation::{annotation_constructor, Annotation},
    enumeration::Enumeration,
    fidl_file::FileError,
    node::{impl_ast_node, sorted_children, Comment, MemberBuilder, MemberEnum, NodeMeta},
    structure::Structure,
    type_def::TypeDef,
    Version,
};
use minimal_fidl_parser::{BasicPublisher, Node, Rules};

/// An ordered child of a [`TypeCollection`].
#[derive(Debug, Clone)]
pub enum TypeCollectionMember {
    TypeDef(TypeDef),
    Structure(Structure),
    Enumeration(Enumeration),
    Comment(Comment),
}

impl MemberEnum for TypeCollectionMember {
    fn from_comment(comment: Comment) -> Self {
        TypeCollectionMember::Comment(comment)
    }
}

#[derive(Debug, Clone)]
pub struct TypeCollection {
    pub meta: NodeMeta,
    pub annotations: Vec<Annotation>,
    /// Empty for an anonymous `typeCollection { ... }`, which the grammar allows.
    /// Use [`TypeCollection::is_anonymous`] rather than testing for `""`.
    pub name: String,
    pub version: Option<Version>,
    /// Typedefs, structs, enums and comments, in source order.
    pub members: Vec<TypeCollectionMember>,
}

impl_ast_node!(TypeCollection);

impl TypeCollection {
    /// An unnamed `typeCollection { ... }`. Legal, but nothing can refer to it.
    pub fn is_anonymous(&self) -> bool {
        self.name.is_empty()
    }

    crate::member_accessors!(
        TypeCollectionMember,
        TypeDef,
        TypeDef,
        typedefs,
        typedefs_mut,
        typedef,
        typedef_mut
    );
    crate::member_accessors!(
        TypeCollectionMember,
        Structure,
        Structure,
        structures,
        structures_mut,
        structure,
        structure_mut
    );
    crate::member_accessors!(
        TypeCollectionMember,
        Enumeration,
        Enumeration,
        enumerations,
        enumerations_mut,
        enumeration,
        enumeration_mut
    );

    crate::container_ops!(TypeCollectionMember);
    crate::member_mutators!(TypeCollectionMember, TypeDef, TypeDef, add_typedef, remove_typedef, TypeDefAlreadyExists, typedef);
    crate::member_mutators!(TypeCollectionMember, Structure, Structure, add_structure, remove_structure, StructAlreadyExists, structure);
    crate::member_mutators!(TypeCollectionMember, Enumeration, Enumeration, add_enumeration, remove_enumeration, EnumerationAlreadyExists, enumeration);

    pub fn new(source: &str, publisher: &BasicPublisher, node: &Node) -> Result<Self, FileError> {
        debug_assert_eq!(node.rule, Rules::type_collection);
        // The grammar permits `typeCollection { ... }` with no name at all.
        let mut name: String = String::new();
        let mut version: Option<Version> = None;
        let mut annotations: Vec<Annotation> = Vec::new();
        let mut builder: MemberBuilder<TypeCollectionMember> =
            MemberBuilder::new(source, node.start_position, false);

        for child in sorted_children(publisher, node) {
            match child.rule {
                Rules::comment | Rules::multiline_comment => builder.comment(child),
                Rules::open_bracket => builder.open_body(child),
                Rules::close_bracket => {}
                Rules::variable_name => {
                    name = child.get_string(source);
                }
                Rules::annotation_block => {
                    annotations = annotation_constructor(source, publisher, child)?;
                }
                Rules::version => {
                    let ver = Version::new(source, publisher, child)?;
                    let ver = builder.attach(child, ver);
                    ver.push_if_not_exists_else_err(&mut version)?;
                }
                Rules::typedef => {
                    let typedef = TypeDef::new(source, publisher, child)?;
                    builder.member(child, typedef, TypeCollectionMember::TypeDef);
                }
                Rules::structure => {
                    let structure = Structure::new(source, publisher, child)?;
                    builder.member(child, structure, TypeCollectionMember::Structure);
                }
                Rules::enumeration => {
                    let enumeration = Enumeration::new(source, publisher, child)?;
                    builder.member(child, enumeration, TypeCollectionMember::Enumeration);
                }
                rule => {
                    return Err(FileError::UnexpectedNode(
                        rule,
                        "TypeCollection::new".to_string(),
                    ));
                }
            }
        }
        let (members, header_comments) = builder.finish();
        let mut meta = NodeMeta::from_cst(node);
        meta.header_comments = header_comments;
        let type_collection = Self {
            meta,
            annotations,
            name,
            version,
            members,
        };
        Ok(type_collection)
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
