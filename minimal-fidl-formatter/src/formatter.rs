use minimal_fidl_parser::{BasicPublisher, Key, Node, Rules};
use num_traits::int;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum FormatterError {
    // #[error("Dividing by Zero is undefined!")]
    // DivideByZero,
    // #[error("Could not parse `{0}` as an integer.")]
    // IntegerParseError(String),
}

pub struct Formatter<'a> {
    source: &'a str,
    publisher: &'a BasicPublisher,
}

impl<'a> Formatter<'a> {
    pub fn new(source: &'a str, publisher: &'a BasicPublisher) -> Self {
        Formatter { source, publisher }
    }

    pub fn format(&self) -> Result<String, FormatterError> {
        let root_node = self.publisher.get_node(Key(0));
        debug_assert_eq!(root_node.rule, Rules::Grammar);
        let root_node_children = root_node.get_children();
        debug_assert_eq!(root_node_children.len(), 1);
        let grammar_node_key = root_node_children[0];
        let grammar_node = self.publisher.get_node(grammar_node_key);
        let mut ret_string: String = "".to_string();
        for child in grammar_node.get_children() {
            let c = self.publisher.get_node(*child);
            match c.rule {
                Rules::package => {
                    ret_string += &self.package(&c);
                }
                Rules::import_model => {
                    todo!()
                }
                Rules::import_namespace => {
                    todo!()
                }
                Rules::interface => ret_string += &self.interface(&c),
                Rules::type_collection => {
                    todo!()
                }
                Rules::comment => ret_string += &self.comment(c),
                Rules::multiline_comment => {
                    todo!()
                }
                e => {
                    panic!("Rule: {:?} should not be the roots child.", e)
                }
            }
        }
        return Ok(ret_string);
    }

    fn interface(&self, node: &Node) -> String {
        let mut interface_name: Option<String> = None;
        for child in node.get_children() {
            let child = self.publisher.get_node(*child);
            match child.rule {
                Rules::variable_name => interface_name = Some(self.variable_name(child)),
                e => {
                    panic!("Rule: {:?} should not be the interfaces child.", e)
                }
            }
        }

        let mut ret_string = format!("\ninterface {} {{", interface_name.expect("Interface Name should always exist"));
        ret_string += &"\n}";
        return ret_string;
    }

    fn package(&self, node: &Node) -> String {
        let mut ret_string: String = "package ".to_string();
        for child in node.get_children() {
            let child = self.publisher.get_node(*child);
            match child.rule {
                Rules::type_ref => ret_string += &self.type_ref(child),
                Rules::comment => {
                    ret_string = ret_string + &self.comment(child);
                }
                Rules::multiline_comment => {
                    todo!()
                }
                e => {
                    panic!("Rule: {:?} should not be the packages child.", e)
                }
            }
        }
        return ret_string;
    }

    fn type_ref(&self, node: &Node) -> String {
        // type_ref is a terminal so we can just return the str slice
        debug_assert!(node.rule == Rules::type_ref);
        node.get_string(&self.source)
    }
    fn variable_name(&self, node: &Node) -> String {
        // type_ref is a terminal so we can just return the str slice
        debug_assert!(node.rule == Rules::variable_name);
        node.get_string(&self.source)
    }
    fn comment(&self, node: &Node) -> String {
        // type_ref is a terminal so we can just return the str slice
        debug_assert!(node.rule == Rules::comment);
        " ".to_owned() + &node.get_string(&self.source)
    }
    fn multiline_comment(&self, node: &Node) -> String {
        debug_assert!(node.rule == Rules::multiline_comment);
        // type_ref is a terminal so we can just return the str slice
        "\n".to_owned() + &node.get_string(&self.source) + "\n"
    }
}
