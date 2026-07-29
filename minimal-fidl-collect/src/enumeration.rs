use crate::{
    annotation::{annotation_constructor, Annotation},
    enum_value::EnumValue,
    fidl_file::FileError,
    node::{impl_ast_node, sorted_children, Comment, MemberBuilder, MemberEnum, NodeMeta},
};
use minimal_fidl_parser::{BasicPublisher, Node, Rules};

/// An ordered child of an [`Enumeration`].
#[derive(Debug, Clone)]
pub enum EnumMember {
    Value(EnumValue),
    Comment(Comment),
}

impl MemberEnum for EnumMember {
    fn from_comment(comment: Comment) -> Self {
        EnumMember::Comment(comment)
    }
}

#[derive(Debug, Clone)]
pub struct Enumeration {
    pub meta: NodeMeta,
    pub annotations: Vec<Annotation>,
    pub name: String,
    /// Values and comments, in source order.
    pub members: Vec<EnumMember>,
}

impl_ast_node!(Enumeration);

impl Enumeration {
    crate::member_accessors!(
        EnumMember,
        Value,
        EnumValue,
        values,
        values_mut,
        value,
        value_mut
    );

    crate::container_ops!(EnumMember);
    crate::member_mutators!(EnumMember, Value, EnumValue, add_value, remove_value, EnumValueAlreadyExists, value);

    pub fn new(source: &str, publisher: &BasicPublisher, node: &Node) -> Result<Self, FileError> {
        debug_assert_eq!(node.rule, Rules::enumeration);
        let mut name: Result<String, FileError> = Err(FileError::InternalLogicError(
            "Uninitialized value: name in Enumeration::new".to_string(),
        ));
        let mut annotations: Vec<Annotation> = Vec::new();
        let mut builder: MemberBuilder<EnumMember> =
            MemberBuilder::new(source, node.start_position, false);

        for child in sorted_children(publisher, node) {
            match child.rule {
                Rules::comment | Rules::multiline_comment => builder.comment(child),
                Rules::open_bracket => builder.open_body(child),
                Rules::close_bracket => {}
                Rules::annotation_block => {
                    annotations = annotation_constructor(source, publisher, child)?;
                }
                Rules::type_dec => {
                    name = Ok(child.get_string(source));
                }
                Rules::enum_value => {
                    let enum_val = EnumValue::new(source, publisher, child)?;
                    builder.member(child, enum_val, EnumMember::Value);
                }
                rule => {
                    return Err(FileError::UnexpectedNode(
                        rule,
                        "Enumeration::new".to_string(),
                    ));
                }
            }
        }
        let (members, header_comments) = builder.finish();
        let mut meta = NodeMeta::from_cst(node);
        meta.header_comments = header_comments;
        let enumeration = Self {
            meta,
            name: name?,
            annotations,
            members,
        };
        Ok(enumeration)
    }


    pub fn push_if_not_exists_else_err(
        self,
        enumerations: &mut Vec<Enumeration>,
    ) -> Result<(), FileError> {
        for s in &mut *enumerations {
            if s.name == self.name {
                return Err(FileError::EnumerationAlreadyExists(s.clone(), self.clone()));
            }
        }
        enumerations.push(self);
        Ok(())
    }
}
