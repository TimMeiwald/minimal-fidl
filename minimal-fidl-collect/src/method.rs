use crate::{
    annotation::{annotation_constructor, Annotation},
    fidl_file::FileError,
    node::{impl_ast_node, sorted_children, Comment, MemberBuilder, MemberEnum, NodeMeta},
    VariableDeclaration,
};
use minimal_fidl_parser::{BasicPublisher, Node, Rules};

/// An ordered child of a parameter list.
#[derive(Debug, Clone)]
pub enum ParamMember {
    Param(VariableDeclaration),
    Comment(Comment),
}

impl MemberEnum for ParamMember {
    fn from_comment(comment: Comment) -> Self {
        ParamMember::Comment(comment)
    }
}

/// A method's `in` or `out` parameter list.
#[derive(Debug, Clone, Default)]
pub struct ParamList {
    pub meta: NodeMeta,
    /// The grammar allows an annotation block before `in` / `out`.
    pub annotations: Vec<Annotation>,
    /// Parameters and comments, in source order.
    pub members: Vec<ParamMember>,
}

impl_ast_node!(ParamList);

impl ParamList {
    crate::member_accessors!(
        ParamMember,
        Param,
        VariableDeclaration,
        params,
        params_mut,
        param,
        param_mut
    );

    crate::container_ops!(ParamMember);

    /// Append a parameter, erroring if the name is taken.
    pub fn add_param(
        &mut self,
        param: VariableDeclaration,
    ) -> Result<&mut VariableDeclaration, FileError> {
        use crate::node::AstNode as _;
        if self.param(&param.name).is_some() {
            return Err(FileError::FieldAlreadyExists(param.name.clone()));
        }
        self.mark_dirty();
        self.members.push(ParamMember::Param(param));
        match self.members.last_mut() {
            Some(ParamMember::Param(v)) => Ok(v),
            _ => unreachable!("just pushed this variant"),
        }
    }

    pub fn remove_param(&mut self, name: &str) -> Option<VariableDeclaration> {
        use crate::node::AstNode as _;
        let index = self.members.iter().position(|m| match m {
            ParamMember::Param(v) => v.name == name,
            _ => false,
        })?;
        self.mark_dirty();
        match self.members.remove(index) {
            ParamMember::Param(v) => Some(v),
            _ => unreachable!("index came from this variant"),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.members.is_empty()
    }

    fn new(source: &str, publisher: &BasicPublisher, node: &Node) -> Result<Self, FileError> {
        debug_assert!(node.rule == Rules::input_params || node.rule == Rules::output_params);
        let mut builder: MemberBuilder<ParamMember> =
            MemberBuilder::new(source, node.start_position, false);
        let mut annotations: Vec<Annotation> = Vec::new();

        for child in sorted_children(publisher, node) {
            match child.rule {
                Rules::comment | Rules::multiline_comment => builder.comment(child),
                Rules::open_bracket => builder.open_body(child),
                Rules::annotation_block => {
                    annotations = annotation_constructor(source, publisher, child)?;
                }
                Rules::close_bracket => {}
                Rules::variable_declaration => {
                    let var_dec = VariableDeclaration::new(source, publisher, child)?;
                    builder.member(child, var_dec, ParamMember::Param);
                }
                rule => {
                    return Err(FileError::UnexpectedNode(
                        rule,
                        "Method::params".to_string(),
                    ));
                }
            }
        }
        let (members, header_comments) = builder.finish();
        let mut meta = NodeMeta::from_cst(node);
        meta.header_comments = header_comments;
        Ok(Self {
            meta,
            annotations,
            members,
        })
    }

}

#[derive(Debug, Clone)]
pub struct Method {
    pub meta: NodeMeta,
    pub annotations: Vec<Annotation>,
    pub name: String,
    pub inputs: ParamList,
    pub outputs: ParamList,
}

impl_ast_node!(Method);

impl Method {
    pub fn input_parameters(&self) -> impl Iterator<Item = &VariableDeclaration> {
        self.inputs.params()
    }

    pub fn output_parameters(&self) -> impl Iterator<Item = &VariableDeclaration> {
        self.outputs.params()
    }

    pub fn new(source: &str, publisher: &BasicPublisher, node: &Node) -> Result<Self, FileError> {
        debug_assert_eq!(node.rule, Rules::method);
        let mut name: Result<String, FileError> = Err(FileError::InternalLogicError(
            "Uninitialized value: name in Method::new".to_string(),
        ));
        let mut annotations: Vec<Annotation> = Vec::new();
        let mut inputs = ParamList::default();
        let mut outputs = ParamList::default();
        // The method body holds only the two parameter lists, so a builder here
        // would have nothing to order. Comments inside the braces attach to
        // whichever list follows them, or to the method header.
        // Comments are placed by where they sit relative to the braces, so that
        // printing puts them back where they came from. Anything else makes
        // formatting non-idempotent: a comment printed outside the braces is
        // captured as the method's *leading* trivia on the next parse.
        let mut header_comments: Vec<Comment> = Vec::new();
        let mut tail_comments: Vec<Comment> = Vec::new();
        let mut pending: Vec<Comment> = Vec::new();
        let mut body_open = false;
        let mut body_closed = false;

        for child in sorted_children(publisher, node) {
            match child.rule {
                Rules::comment | Rules::multiline_comment => {
                    if let Some(comment) = Comment::from_cst(source, child) {
                        if body_closed {
                            tail_comments.push(comment);
                        } else if body_open {
                            // Belongs to whichever parameter list comes next.
                            pending.push(comment);
                        } else {
                            header_comments.push(comment);
                        }
                    }
                }
                Rules::open_bracket => body_open = true,
                Rules::close_bracket => body_closed = true,
                Rules::annotation_block => {
                    annotations = annotation_constructor(source, publisher, child)?;
                }
                Rules::variable_name => {
                    name = Ok(child.get_string(source));
                }
                Rules::input_params => {
                    inputs = ParamList::new(source, publisher, child)?;
                    inputs.meta.header_comments.splice(..0, pending.drain(..));
                }
                Rules::output_params => {
                    outputs = ParamList::new(source, publisher, child)?;
                    outputs.meta.header_comments.splice(..0, pending.drain(..));
                }
                rule => {
                    return Err(FileError::UnexpectedNode(rule, "Method::new".to_string()));
                }
            }
        }
        // Comments after the last parameter list but still inside the braces.
        outputs.meta.trailing_comments.append(&mut pending);

        let mut meta = NodeMeta::from_cst(node);
        meta.header_comments = header_comments;
        meta.trailing_comments = tail_comments;
        Ok(Self {
            meta,
            name: name?,
            annotations,
            inputs,
            outputs,
        })
    }

    pub fn push_if_not_exists_else_err(self, methods: &mut Vec<Method>) -> Result<(), FileError> {
        for s in &mut *methods {
            if s.name == self.name {
                return Err(FileError::MethodAlreadyExists(s.clone(), self.clone()));
            }
        }
        methods.push(self);
        Ok(())
    }
}
