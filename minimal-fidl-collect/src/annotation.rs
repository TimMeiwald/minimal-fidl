use minimal_fidl_parser::{BasicPublisher, Node, Rules};

use crate::node::{impl_ast_node, trailing_comments, NodeMeta};
use crate::FileError;

#[derive(Debug, Clone)]
pub struct Annotation {
    pub meta: NodeMeta,
    pub name: String,
    pub contents: String,
}

impl_ast_node!(Annotation);

impl PartialEq for Annotation {
    /// Ignores identity and layout — see [`crate::node::LayoutEq`].
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.contents == other.contents
    }
}
impl Eq for Annotation {}

impl Annotation {
    pub fn create(name: impl Into<String>, contents: impl Into<String>) -> Self {
        Self {
            meta: NodeMeta::default(),
            name: name.into(),
            contents: contents.into(),
        }
    }

    fn new(
        source: &str,
        publisher: &BasicPublisher,
        node: &Node,
    ) -> Result<Option<Annotation>, FileError> {
        debug_assert_eq!(node.rule, Rules::annotation);
        let mut name: Option<String> = None;
        let mut contents: Option<String> = None;
        for child in node.get_children() {
            let child = publisher.get_node(*child);
            match child.rule {
                Rules::annotation_name => {
                    name = Some(child.get_string(source));
                }
                Rules::annotation_content => contents = Some(child.get_string(source)),
                rule => {
                    return Err(FileError::UnexpectedNode(
                        rule,
                        "Annotation::new".to_string(),
                    ));
                }
            }
        }
        match (name, contents) {
            (Some(name), Some(contents)) => Ok(Some(Self {
                meta: NodeMeta {
                    trailing_comments: trailing_comments(source, publisher, node),
                    ..NodeMeta::from_cst(node)
                },
                name,
                contents,
            })),
            (None, None) => Ok(None),
            (_, _) => {
                return Err(FileError::InternalLogicError(
                    "Name and Contents should exist together".to_string(),
                ))
            }
        }
    }
}

pub fn annotation_constructor(
    source: &str,
    publisher: &BasicPublisher,
    node: &Node,
) -> Result<Vec<Annotation>, FileError> {
    let mut annotations: Vec<Annotation> = Vec::new();
    let mut pending: Vec<crate::node::Comment> = Vec::new();
    debug_assert_eq!(node.rule, Rules::annotation_block);
    for child in node.get_children() {
        let child = publisher.get_node(*child);
        match child.rule {
            Rules::annotation => {
                let annotation = Annotation::new(source, publisher, child)?;
                if let Some(mut annotation) = annotation {
                    if !pending.is_empty() {
                        let mut held = std::mem::take(&mut pending);
                        held.append(&mut annotation.meta.leading_comments);
                        annotation.meta.leading_comments = held;
                    }
                    annotations.push(annotation);
                }
            }
            Rules::comment | Rules::multiline_comment => {
                if let Some(comment) = crate::node::Comment::from_cst(source, child) {
                    match annotations.last_mut() {
                        Some(previous) => previous.meta.trailing_comments.push(comment),
                        // Nothing to attach to yet; hold it for the first annotation.
                        None => pending.push(comment),
                    }
                }
            }
            Rules::open_bracket | Rules::close_bracket => {}
            rule => {
                return Err(FileError::UnexpectedNode(
                    rule,
                    "Interface::new".to_string(),
                ));
            }
        }
    }
    // A block whose only content was comments still has to keep them.
    if let (Some(first), false) = (annotations.first_mut(), pending.is_empty()) {
        first.meta.leading_comments.append(&mut pending);
    }
    Ok(annotations)
}

#[cfg(test)]
mod tests {
    use minimal_fidl_parser::*;

    // Parsing is tested in parser
    use super::*;
    use crate::shared_test::shared;

    pub fn get_node_constructor_args(input: &str) -> (&str, BasicPublisher, Key) {
        shared(input, annotation::<BasicContext>, Rules::annotation)
    }

    #[test]
    fn test_annotation_fields() {
        let (source, publisher, node_key) = get_node_constructor_args("@Annotation: block");
        let node = publisher.get_node(node_key);
        let x = Annotation::new(source, &publisher, node)
            .expect("We expect success in this test")
            .unwrap();
        assert_eq!(x.name, "Annotation");
        // NOTE: contents currently keeps the whitespace after the ':'. Phase 2 should
        // decide whether to trim it — doing so changes formatter output, so not now.
        assert_eq!(x.contents, " block");
    }
}
