use minimal_fidl_parser::{BasicPublisher, Rules, Key, Node};
use thiserror::Error;
#[derive(Error, Debug, PartialEq)]
pub enum FormatterError {
    // #[error("Dividing by Zero is undefined!")]
    // DivideByZero,
    // #[error("Could not parse `{0}` as an integer.")]
    // IntegerParseError(String),
}

pub struct Formatter<'a>{
    source: &'a str, 
    publisher: &'a BasicPublisher
}

impl<'a> Formatter<'a>{
    pub fn new(source: &'a str, publisher: &'a BasicPublisher) -> Self {
        Formatter { source, publisher }
    }

    pub fn format(&self) -> Result<String, FormatterError>{
        let root_node = self.publisher.get_node(Key(0));
        debug_assert_eq!(root_node.rule, Rules::Grammar);
        let root_node_children = root_node.get_children();
        debug_assert_eq!(root_node_children.len(), 1);
        let grammar_node_key = root_node_children[0];
        let grammar_node = self.publisher.get_node(grammar_node_key);
        for child in grammar_node.get_children() {
            let c = self.publisher.get_node(*child);
            match c.rule {
                Rules::package => {
                    return self.package(&c);
                }
                Rules::import_model => {
                    todo!()
                }
                Rules::import_namespace => {
                    todo!()
                }
                Rules::interface => {todo!()}
                Rules::type_collection => {todo!()}
                Rules::comment => {todo!()}
                Rules::multiline_comment => {todo!()}
                e => {
                    panic!("Rule: {:?} should not be the roots child.", e)
                }
            }
        }

        panic!("Should not get here");
    }

    pub fn package(&self, node: &Node) -> Result<String, FormatterError>{

    }
}
