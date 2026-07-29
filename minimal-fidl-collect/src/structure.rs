use crate::{
    annotation::{annotation_constructor, Annotation},
    fidl_file::FileError,
    node::{impl_ast_node, sorted_children, Comment, MemberBuilder, MemberEnum, NodeMeta},
    VariableDeclaration,
};
use minimal_fidl_parser::{BasicPublisher, Node, Rules};

/// An ordered child of a [`Structure`].
#[derive(Debug, Clone)]
pub enum StructMember {
    Field(VariableDeclaration),
    Comment(Comment),
}

impl MemberEnum for StructMember {
    fn from_comment(comment: Comment) -> Self {
        StructMember::Comment(comment)
    }
}

#[derive(Debug, Clone)]
pub struct Structure {
    pub meta: NodeMeta,
    pub annotations: Vec<Annotation>,
    pub name: String,
    /// Fields and comments, in source order.
    pub members: Vec<StructMember>,
}

impl_ast_node!(Structure);

impl Structure {
    crate::member_accessors!(
        StructMember,
        Field,
        VariableDeclaration,
        fields,
        fields_mut,
        field,
        field_mut
    );

    crate::container_ops!(StructMember);

    /// Append a field, erroring if the name is taken.
    pub fn add_field(
        &mut self,
        field: crate::VariableDeclaration,
    ) -> Result<&mut crate::VariableDeclaration, FileError> {
        if self.field(&field.name).is_some() {
            return Err(FileError::FieldAlreadyExists(field.name.clone()));
        }
        self.members.push(StructMember::Field(field));
        match self.members.last_mut() {
            Some(StructMember::Field(v)) => Ok(v),
            _ => unreachable!("just pushed this variant"),
        }
    }

    pub fn remove_field(&mut self, name: &str) -> Option<crate::VariableDeclaration> {
        let index = self.members.iter().position(|m| match m {
            StructMember::Field(v) => v.name == name,
            _ => false,
        })?;
        match self.members.remove(index) {
            StructMember::Field(v) => Some(v),
            _ => unreachable!("index came from this variant"),
        }
    }

    pub fn new(source: &str, publisher: &BasicPublisher, node: &Node) -> Result<Self, FileError> {
        debug_assert_eq!(node.rule, Rules::structure);
        let mut name: Result<String, FileError> = Err(FileError::InternalLogicError(
            "Uninitialized value: name in Structure::new".to_string(),
        ));
        let mut annotations: Vec<Annotation> = Vec::new();
        let mut builder: MemberBuilder<StructMember> =
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
                Rules::variable_declaration => {
                    let var_dec = VariableDeclaration::new(source, publisher, child)?;
                    builder.member(child, var_dec, StructMember::Field);
                }
                rule => {
                    return Err(FileError::UnexpectedNode(
                        rule,
                        "Structure::new".to_string(),
                    ));
                }
            }
        }
        let (members, header_comments) = builder.finish();
        let mut meta = NodeMeta::from_cst(node);
        meta.header_comments = header_comments;
        let structure = Self {
            meta,
            name: name?,
            annotations,
            members,
        };
        Ok(structure)
    }


    pub fn push_if_not_exists_else_err(
        self,
        structures: &mut Vec<Structure>,
    ) -> Result<(), FileError> {
        for s in &mut *structures {
            if s.name == self.name {
                return Err(FileError::StructAlreadyExists(s.clone(), self.clone()));
            }
        }
        structures.push(self);
        Ok(())
    }
}
