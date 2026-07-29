
use crate::{
    annotation::{annotation_constructor, Annotation},
    fidl_file::FileError,
};
use minimal_fidl_parser::{BasicPublisher, Node, Rules};
use crate::node::{impl_ast_node, trailing_comments, NodeMeta};

#[derive(Debug, Clone)]
pub struct Attribute {
    pub meta: NodeMeta,
    pub annotations: Vec<Annotation>,
    pub name: String,
    pub type_n: String,
}

impl Attribute {
    pub fn new(source: &str, publisher: &BasicPublisher, node: &Node) -> Result<Self, FileError> {
        debug_assert_eq!(node.rule, Rules::attribute);
        let mut name: Result<String, FileError> = Err(FileError::InternalLogicError(
            "Uninitialized value: name in Attribute::new".to_string(),
        ));
        let mut type_n: Result<String, FileError> = Err(FileError::InternalLogicError(
            "Uninitialized value: type_n in Attribute::new".to_string(),
        ));
        let mut annotations: Vec<Annotation> = Vec::new();
        for child in node.get_children() {
            let child = publisher.get_node(*child);
            match child.rule {
                Rules::comment | Rules::multiline_comment => {}
                Rules::annotation_block => {
                    annotations = annotation_constructor(source, publisher, child)?;
                }
                Rules::type_ref => {
                    type_n = Ok(child.get_string(source));
                }
                Rules::variable_name => name = Ok(child.get_string(source)),

                rule => {
                    return Err(FileError::UnexpectedNode(
                        rule,
                        "Attribute::new".to_string(),
                    ));
                }
            }
        }
        Ok(Self {
            name: name?,
            type_n: type_n?,
            annotations,
            meta: NodeMeta {
                trailing_comments: trailing_comments(source, publisher, node),
                ..NodeMeta::from_cst(node)
            },
        })
    }
    pub fn push_if_not_exists_else_err(
        self,
        attributes: &mut Vec<Attribute>,
    ) -> Result<(), FileError> {
        for attr in &mut *attributes {
            if attr.name == self.name {
                return Err(FileError::AttributeAlreadyExists(
                    attr.clone(),
                    self.clone(),
                ));
            }
        }
        attributes.push(self);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use minimal_fidl_parser::*;

    // Parsing is tested in parser
    use crate::{attribute, shared_test::shared, Attribute};
    use super::*;

    #[test]
    fn test_attribute_fields() {
        let (source, publisher, node_key) = shared(
            "attribute Duration remainingTrack",
            attribute::<BasicContext>,
            Rules::attribute,
        );
        let node = publisher.get_node(node_key);
        let x = Attribute::new(source, &publisher, node).expect("We expect success in this test");
        assert_eq!(x.name, "remainingTrack");
        assert_eq!(x.type_n, "Duration");
        assert!(x.annotations.is_empty());
    }
}

impl_ast_node!(Attribute);
