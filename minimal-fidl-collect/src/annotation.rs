use minimal_fidl_parser::{BasicPublisher, Node, Rules};

use crate::{annotation, FileError};

#[derive(Debug, Clone)]
pub struct Annotation {
    pub name: String,
    pub contents: String,
}

impl Annotation {
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
            (Some(name), Some(contents)) => Ok(Some(Self { name, contents })),
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
    debug_assert_eq!(node.rule, Rules::annotation_block);
    for child in node.get_children() {
        let child = publisher.get_node(*child);
        match child.rule {
            Rules::annotation => {
                let annotation = Annotation::new(source, publisher, child)?;
                if let Some(annotation) = annotation {
                    annotations.push(annotation);
                }
            }
            Rules::comment
            | Rules::multiline_comment
            | Rules::open_bracket
            | Rules::close_bracket => {}
            rule => {
                return Err(FileError::UnexpectedNode(
                    rule,
                    "Interface::new".to_string(),
                ));
            }
        }
    }
    Ok(annotations)
}
