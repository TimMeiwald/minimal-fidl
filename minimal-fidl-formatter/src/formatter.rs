use clap::builder::Str;
use minimal_fidl_parser::{BasicPublisher, Key, Node, Rules};
use num_traits::int;
use std::fmt::{self, format};
use std::ops::AddAssign;
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

struct IndentedString {
    indent_level: u8,
    str: String,
}
impl IndentedString {
    pub fn new(indent_level: u8, str: String) -> Self {
        IndentedString { str, indent_level }
    }

    pub fn indent(&mut self) {
        self.indent_level += 1
    }
}
impl fmt::Display for IndentedString {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let spacing = "    ";
        let spacing = spacing.repeat(self.indent_level as usize);
        write!(f, "\n{spacing}{}", self.str)
    }
}
impl AddAssign for IndentedString {
    fn add_assign(&mut self, other: Self) {
        *self = Self {
            str: self.str.clone() + &other.str,
            indent_level: self.indent_level,
        };
    }
}

#[test]
fn test_indented_string() {
    let mut i = IndentedString::new(0, "str".to_string());
    println!("{i}");
    i.indent();
    println!("{i}");
    i.indent();
    println!("{i}");
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
                    for line in &self.package(&c) {
                        ret_string += &line.to_string();
                    }
                }
                Rules::import_model => {
                    todo!()
                }
                Rules::import_namespace => {
                    todo!()
                }
                Rules::interface => {
                    let interface = self.interface(&c);
                    for line in interface {
                        ret_string += &line.to_string();
                    }
                }
                Rules::type_collection => {
                    todo!()
                }
                Rules::comment => ret_string += &self.comment(c).to_string(),
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

    fn interface(&self, node: &Node) -> Vec<IndentedString> {
        debug_assert!(node.rule == Rules::interface);
        let mut interface_name: Option<String> = None;
        // let mut version: Option<Vec<String>> = None;
        // let mut methods: Vec<Vec<String>> = Vec::new();
        // let mut attributes: Vec<String> = Vec::new();
        // let mut structures: Vec<Vec<String>> = Vec::new();

        let mut ret_vec: Vec<IndentedString> = Vec::new();
        for child in node.get_children() {
            let child = self.publisher.get_node(*child);
            match child.rule {
                Rules::variable_name => {
                    interface_name = Some(self.variable_name(child));
                    let interface = format!(
                        "\ninterface {} {{",
                        interface_name.expect("Interface Name should always exist")
                    );
                    let interface = IndentedString::new(0, interface.to_string());
                    ret_vec.push(interface);
                }
                Rules::version => {
                    for mut line in self.version(child) {
                        line.indent();
                        ret_vec.push(line);
                    }
                }
                Rules::typedef => todo!("Implement typedef in interface"),
                Rules::method => {
                    for mut line in self.method(child) {
                        line.indent();
                        ret_vec.push(line);
                    }
                }
                Rules::attribute => {
                    for mut line in self.attribute(child) {
                        line.indent();
                        ret_vec.push(line);
                    }
                }
                Rules::structure => {
                    for mut line in self.structure(child) {
                        line.indent();
                        ret_vec.push(line);
                    }
                }
                e => {
                    panic!("Rule: {:?} should not be the interfaces child.", e)
                }
            }
        }

        if ret_vec.len() == 1 {
            ret_vec[0] += IndentedString::new(0, "}".to_string());
        } else {
            ret_vec.push(IndentedString::new(0, "}".to_string()));
        }

        ret_vec
    }

    fn type_dec(&self, node: &Node) -> String {
        node.get_string(&self.source)
    }

    fn typedef(&self, node: &Node) -> Vec<String> {
        debug_assert!(node.rule == Rules::typedef);
        let mut type_dec = "".to_string();
        let mut type_ref = "".to_string();
        let mut ret_vec: Vec<String> = Vec::new();

        for child in node.get_children() {
            let child = self.publisher.get_node(*child);
            match child.rule {
                Rules::type_dec => type_dec = self.type_dec(child),
                Rules::type_ref => type_dec = self.type_ref(child),
                e => {
                    panic!("Rule: {:?} should not be the typedefs child.", e)
                }
            }
        }
        let result = format!("typedef {} is {}", type_dec, type_ref);
        ret_vec.push(result);
        ret_vec
    }

    fn structure(&self, node: &Node) -> Vec<IndentedString> {
        debug_assert!(node.rule == Rules::structure);

        let mut ret_vec: Vec<IndentedString> = Vec::new();
        for child in node.get_children() {
            let child = self.publisher.get_node(*child);
            match child.rule {
                Rules::type_dec => {
                    // We know this happens before the contents of struct.
                    let struct_name = self.type_dec(child);
                    let struct_name = IndentedString::new(0, format!("struct {} {{", struct_name));
                    ret_vec.push(struct_name);
                }
                Rules::variable_declaration => {
                    for mut line in self.variable_declaration(child) {
                        line.indent();
                        ret_vec.push(line);
                    }
                }
                e => {
                    panic!("Rule: {:?} should not be the structures child.", e)
                }
            }
        }
        if ret_vec.len() == 1 {
            ret_vec[0] += IndentedString::new(0, "}".to_string());
        } else {
            ret_vec.push(IndentedString::new(0, "}".to_string()));
        }

        ret_vec
    }

    fn attribute(&self, node: &Node) -> Vec<IndentedString> {
        debug_assert!(node.rule == Rules::attribute);
        let mut type_ref: String = "".to_string();
        let mut var_name: String = "".to_string();
        let mut ret_vec: Vec<IndentedString> = Vec::new();
        for child in node.get_children() {
            let child = self.publisher.get_node(*child);
            match child.rule {
                Rules::type_ref => type_ref = self.type_ref(child),
                Rules::variable_name => {
                    var_name = self.variable_name(child);
                    let attr = IndentedString::new(0, format!("attribute {} {}", type_ref, var_name));
                    ret_vec.push(attr);
                }, 
                Rules::comment => {
                    ret_vec.push(self.comment(child));
                }
                e => {
                    panic!("Rule: {:?} should not be the version child.", e)
                }
            }
        }
        ret_vec
    }

    fn version(&self, node: &Node) -> Vec<IndentedString> {
        debug_assert!(node.rule == Rules::version);
        let mut major: Option<String> = None;
        let mut minor: Option<String> = None;
        let mut ret_vec: Vec<IndentedString> = Vec::new();
        ret_vec.push(IndentedString::new(0, "version {".to_string()));
        for child in node.get_children() {
            let child = self.publisher.get_node(*child);
            match child.rule {
                Rules::major => ret_vec.push(IndentedString::new(1, self.major(child))),
                Rules::minor => ret_vec.push(IndentedString::new(1, self.minor(child))),
                e => {
                    panic!("Rule: {:?} should not be the version child.", e)
                }
            }
        }

        if ret_vec.len() == 1 {
            ret_vec[0] += IndentedString::new(0, "}".to_string());
        } else {
            ret_vec.push(IndentedString::new(0, "}".to_string()));
        }
        ret_vec
    }
    fn major(&self, node: &Node) -> String {
        debug_assert!(node.rule == Rules::major);
        let children = node.get_children();
        debug_assert_eq!(children.len(), 1);
        let child = self.publisher.get_node(children[0]);
        format!("major {}", self.digits(child))
    }

    fn minor(&self, node: &Node) -> String {
        debug_assert!(node.rule == Rules::minor);
        let children = node.get_children();
        debug_assert_eq!(children.len(), 1);
        let child = self.publisher.get_node(children[0]);
        format!("minor {}", self.digits(child))
    }

    fn digits(&self, node: &Node) -> String {
        debug_assert!(node.rule == Rules::digits);
        node.get_string(&self.source)
    }

    fn method(&self, node: &Node) -> Vec<IndentedString> {
        debug_assert!(node.rule == Rules::method);
        let mut var_name: String = "".to_string();
        let mut input: Vec<IndentedString> = Vec::new();
        let mut output: Vec<IndentedString> = Vec::new();
        let mut ret_vec: Vec<IndentedString> = Vec::new();

        for child in node.get_children() {
            let child = self.publisher.get_node(*child);
            match child.rule {
                Rules::variable_name => var_name = self.variable_name(child),
                Rules::input_params => input = self.input_params(child),
                Rules::output_params => output = self.output_params(child),
                e => {
                    panic!("Rule: {:?} should not be the method child.", e)
                }
            }
        }

        ret_vec.push(IndentedString::new(0, format!("method {} {{", var_name)));

        for mut line in input {
            line.indent();
            ret_vec.push(line);
        }
        for mut line in output {
            line.indent();
            ret_vec.push(line);
        }

        if ret_vec.len() == 1 {
            ret_vec[0] += IndentedString::new(0, "}".to_string());
        } else {
            ret_vec.push(IndentedString::new(0, "}".to_string()));
        }
        ret_vec
    }

    fn input_params(&self, node: &Node) -> Vec<IndentedString> {
        debug_assert!(node.rule == Rules::input_params);
        let mut ret_vec: Vec<IndentedString> = Vec::new();
        ret_vec.push(IndentedString::new(0, "in {".to_owned()));
        for child in node.get_children() {
            let child = self.publisher.get_node(*child);
            match child.rule {
                Rules::variable_declaration => {
                    for mut line in self.variable_declaration(child) {
                        line.indent();
                        ret_vec.push(line);
                    }
                }
                e => {
                    panic!("Rule: {:?} should not be the version child.", e)
                }
            }
        }
        if ret_vec.len() == 1 {
            ret_vec[0] += IndentedString::new(0, "}".to_string());
        } else {
            ret_vec.push(IndentedString::new(0, "}".to_owned()));
        }
        ret_vec
    }

    fn output_params(&self, node: &Node) -> Vec<IndentedString> {
        debug_assert!(node.rule == Rules::output_params);
        let mut ret_vec: Vec<IndentedString> = Vec::new();
        ret_vec.push(IndentedString::new(0, "out {".to_owned()));
        for child in node.get_children() {
            let child = self.publisher.get_node(*child);
            match child.rule {
                Rules::variable_declaration => {
                    for mut line in self.variable_declaration(child) {
                        line.indent();
                        ret_vec.push(line);
                    }
                }
                e => {
                    panic!("Rule: {:?} should not be the version child.", e)
                }
            }
        }
        if ret_vec.len() == 1 {
            ret_vec[0] += IndentedString::new(0, "}".to_string());
        } else {
            ret_vec.push(IndentedString::new(0, "}".to_owned()));
        }
        ret_vec
    }

    fn variable_declaration(&self, node: &Node) -> Vec<IndentedString> {
        debug_assert!(node.rule == Rules::variable_declaration);
        let mut type_ref: String = "".to_string();
        let mut var_name: String = "".to_string();
        let mut ret_vec: Vec<IndentedString> = Vec::new();
        for child in node.get_children() {
            let child = self.publisher.get_node(*child);
            match child.rule {
                Rules::type_ref => type_ref = self.type_ref(child),
                Rules::variable_name => {
                    var_name = self.variable_name(child);
                    let s = format!("{} {}", type_ref, var_name);
                    let s = IndentedString::new(0, s);
                    ret_vec.push(s);
                }
                e => {
                    panic!("Rule: {:?} should not be the version child.", e)
                }
            }
        }
        let vardec = format!("{} {}", type_ref, var_name);
        ret_vec
    }

    fn package(&self, node: &Node) -> Vec<IndentedString> {
        debug_assert!(node.rule == Rules::package);

        let mut ret_vec: Vec<IndentedString> = Vec::new();
        for child in node.get_children() {
            let child = self.publisher.get_node(*child);
            match child.rule {
                Rules::type_ref => {
                    let s = format!("package {}", self.type_ref(child));
                    let s = IndentedString::new(0, s);
                    ret_vec.push(s);
                }
                Rules::comment => ret_vec.push(self.comment(child)),
                Rules::multiline_comment => {
                    todo!()
                }
                e => {
                    panic!("Rule: {:?} should not be the packages child.", e)
                }
            }
        }
        ret_vec
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
    fn comment(&self, node: &Node) -> IndentedString {
        // type_ref is a terminal so we can just return the str slice
        debug_assert!(node.rule == Rules::comment);
        IndentedString::new(0, " ".to_owned() + &node.get_string(&self.source))
    }
    fn multiline_comment(&self, node: &Node) -> Vec<IndentedString> {
        debug_assert!(node.rule == Rules::multiline_comment);
        // type_ref is a terminal so we can just return the str slice
        // Still needs to be organized properly
        // Right now it just sticks the entire blob down
        // Without even the ticks possibly
        let mut ret_vec: Vec<IndentedString> = Vec::new();
        let ml = IndentedString::new(0, "\n".to_owned() + &node.get_string(&self.source) + "\n");
        ret_vec.push(ml);
        ret_vec
    }
}
