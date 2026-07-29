use crate::{
    annotation::{annotation_constructor, Annotation},
    attribute::Attribute,
    enumeration::Enumeration,
    fidl_file::FileError,
    method::Method,
    node::{impl_ast_node, sorted_children, Comment, MemberBuilder, MemberEnum, NodeMeta},
    structure::Structure,
    type_def::TypeDef,
    Version,
};
use minimal_fidl_parser::{BasicPublisher, Node, Rules};

/// An ordered child of an [`Interface`].
#[derive(Debug, Clone)]
pub enum InterfaceMember {
    Method(Method),
    Attribute(Attribute),
    Structure(Structure),
    Enumeration(Enumeration),
    TypeDef(TypeDef),
    Comment(Comment),
}

impl MemberEnum for InterfaceMember {
    fn from_comment(comment: Comment) -> Self {
        InterfaceMember::Comment(comment)
    }
}

#[derive(Debug, Clone)]
pub struct Interface {
    pub meta: NodeMeta,
    pub annotations: Vec<Annotation>,
    pub name: String,
    /// The grammar permits at most one, in a fixed position before the members,
    /// so it is a field rather than a member variant.
    pub version: Option<Version>,
    /// Methods, attributes, structs, enums, typedefs and comments, in source order.
    pub members: Vec<InterfaceMember>,
}

impl_ast_node!(Interface);

impl Interface {
    crate::member_accessors!(
        InterfaceMember,
        Method,
        Method,
        methods,
        methods_mut,
        method,
        method_mut
    );
    crate::member_accessors!(
        InterfaceMember,
        Attribute,
        Attribute,
        attributes,
        attributes_mut,
        attribute,
        attribute_mut
    );
    crate::member_accessors!(
        InterfaceMember,
        Structure,
        Structure,
        structures,
        structures_mut,
        structure,
        structure_mut
    );
    crate::member_accessors!(
        InterfaceMember,
        Enumeration,
        Enumeration,
        enumerations,
        enumerations_mut,
        enumeration,
        enumeration_mut
    );
    crate::member_accessors!(
        InterfaceMember,
        TypeDef,
        TypeDef,
        typedefs,
        typedefs_mut,
        typedef,
        typedef_mut
    );

    crate::container_ops!(InterfaceMember);
    crate::member_mutators!(InterfaceMember, Method, Method, add_method, remove_method, MethodAlreadyExists, method);
    crate::member_mutators!(InterfaceMember, Attribute, Attribute, add_attribute, remove_attribute, AttributeAlreadyExists, attribute);
    crate::member_mutators!(InterfaceMember, Structure, Structure, add_structure, remove_structure, StructAlreadyExists, structure);
    crate::member_mutators!(InterfaceMember, Enumeration, Enumeration, add_enumeration, remove_enumeration, EnumerationAlreadyExists, enumeration);
    crate::member_mutators!(InterfaceMember, TypeDef, TypeDef, add_typedef, remove_typedef, TypeDefAlreadyExists, typedef);

    pub fn new(source: &str, publisher: &BasicPublisher, node: &Node) -> Result<Self, FileError> {
        debug_assert_eq!(node.rule, Rules::interface);
        let mut name: Result<String, FileError> = Err(FileError::InternalLogicError(
            "Uninitialized value: 'name' in Interface::new".to_string(),
        ));
        let mut version: Option<Version> = None;
        let mut annotations: Vec<Annotation> = Vec::new();
        let mut builder: MemberBuilder<InterfaceMember> =
            MemberBuilder::new(source, node.start_position, false);

        for child in sorted_children(publisher, node) {
            match child.rule {
                Rules::comment | Rules::multiline_comment => builder.comment(child),
                Rules::open_bracket => builder.open_body(child),
                Rules::close_bracket => {}
                Rules::variable_name => {
                    name = Ok(child.get_string(source));
                }
                Rules::annotation_block => {
                    annotations = annotation_constructor(source, publisher, child)?;
                }
                Rules::version => {
                    let ver = Version::new(source, publisher, child)?;
                    let ver = builder.attach(child, ver);
                    ver.push_if_not_exists_else_err(&mut version)?;
                }
                Rules::method => {
                    let method = Method::new(source, publisher, child)?;
                    builder.member(child, method, InterfaceMember::Method);
                }
                Rules::attribute => {
                    let attribute = Attribute::new(source, publisher, child)?;
                    builder.member(child, attribute, InterfaceMember::Attribute);
                }
                Rules::structure => {
                    let structure = Structure::new(source, publisher, child)?;
                    builder.member(child, structure, InterfaceMember::Structure);
                }
                Rules::enumeration => {
                    let enumeration = Enumeration::new(source, publisher, child)?;
                    builder.member(child, enumeration, InterfaceMember::Enumeration);
                }
                Rules::typedef => {
                    let typedef = TypeDef::new(source, publisher, child)?;
                    builder.member(child, typedef, InterfaceMember::TypeDef);
                }
                rule => {
                    return Err(FileError::UnexpectedNode(
                        rule,
                        "Interface::new".to_string(),
                    ));
                }
            }
        }
        let (members, header_comments) = builder.finish();
        let mut meta = NodeMeta::from_cst(node);
        meta.header_comments = header_comments;
        let interface = Self {
            meta,
            annotations,
            name: name?,
            version,
            members,
        };
        Ok(interface)
    }


    pub fn push_if_not_exists_else_err(
        self,
        interfaces: &mut Vec<Interface>,
    ) -> Result<(), FileError> {
        for s in &mut *interfaces {
            if s.name == self.name {
                return Err(FileError::InterfaceAlreadyExists(s.clone(), self.clone()));
            }
        }
        interfaces.push(self);
        Ok(())
    }
}
