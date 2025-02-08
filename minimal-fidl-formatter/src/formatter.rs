use crate::indented_string::IndentedString;
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
                    ret_string += &self.package(&c).to_string();
                }
                Rules::import_model => ret_string += &self.import_model(&c).to_string(),
                Rules::import_namespace => ret_string += &self.import_namespace(&c).to_string(),
                Rules::interface => {
                    let interface = self.interface(&c);
                    for line in interface {
                        ret_string += &line.to_string();
                    }
                }
                Rules::type_collection => {
                    let typecollection = self.type_collection(&c);
                    for line in typecollection {
                        ret_string += &line.to_string();
                    }
                }
                Rules::comment => ret_string += &self.comment(c, false).to_string(),
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

    fn import_namespace(&self, node: &Node) -> IndentedString {
        debug_assert!(node.rule == Rules::import_namespace);
        let mut ret_str: IndentedString = IndentedString::new(0, "".to_string());
        let mut type_ref = "".to_string();
        let mut wildcard = "".to_string();
        for child in node.get_children() {
            let child = self.publisher.get_node(*child);
            match child.rule {
                Rules::type_ref => {
                    type_ref = self.type_ref(child);
                }
                Rules::wildcard => wildcard = ".*".to_string(),
                Rules::file_path => {
                    let filepath = self.file_path(child);
                    let ret = format!("import {}{} from {}", type_ref, wildcard, filepath);
                    ret_str = IndentedString::new(0, ret);
                }
                Rules::comment => {
                    let comment = self.comment(child, true);
                    ret_str += comment;
                }
                e => {
                    panic!("Rule: {:?} should not be the import_namespace child.", e)
                }
            }
        }
        ret_str
    }

    fn import_model(&self, node: &Node) -> IndentedString {
        debug_assert!(node.rule == Rules::import_model);
        let mut ret_str: IndentedString = IndentedString::default();
        for child in node.get_children() {
            let child = self.publisher.get_node(*child);
            match child.rule {
                Rules::file_path => {
                    ret_str =
                        IndentedString::new(0, format!("import model {}", self.file_path(child)));
                }
                Rules::comment => ret_str += self.comment(child, true),

                e => {
                    panic!("Rule: {:?} should not be the import_model child.", e)
                }
            }
        }
        ret_str
    }

    fn file_path(&self, node: &Node) -> String {
        debug_assert!(node.rule == Rules::file_path);
        node.get_string(self.source)
    }

    fn type_collection(&self, node: &Node) -> Vec<IndentedString> {
        debug_assert!(node.rule == Rules::type_collection);
        let mut type_collection_name: Option<String> = None;
        let mut ret_vec: Vec<IndentedString> = Vec::new();
        for child in node.get_children() {
            let child = self.publisher.get_node(*child);
            match child.rule {
                Rules::variable_name => {
                    let tcn = Some(self.variable_name(child));
                    type_collection_name = tcn.clone();
                    let interface = format!(
                        "typeCollection {} {{",
                        tcn.expect("Interface Name should always exist")
                    );
                    let interface = IndentedString::new(0, interface.to_string());
                    ret_vec.push(interface);
                }
                Rules::annotation_block => {
                    for line in self.annotation_block(child) {
                        ret_vec.push(line);
                    }
                }

                Rules::typedef => {
                    match type_collection_name {
                        Some(..) => {}
                        None => {
                            let interface = format!("typeCollection {{\n",);
                            let interface = IndentedString::new(0, interface.to_string());
                            type_collection_name = Some("No Name Set".to_string());
                            ret_vec.push(interface);
                        }
                    }
                    for mut line in self.typedef(child) {
                        line.indent();
                        ret_vec.push(line);
                    }
                }
                Rules::structure => {
                    match type_collection_name {
                        Some(..) => {}
                        None => {
                            let interface = format!("typeCollection {{\n",);
                            let interface = IndentedString::new(0, interface.to_string());
                            type_collection_name = Some("No Name Set".to_string());
                            ret_vec.push(interface);
                        }
                    }
                    for mut line in self.structure(child) {
                        line.indent();
                        ret_vec.push(line);
                    }
                    ret_vec.push(IndentedString::new(0, "".to_string()))
                }
                Rules::enumeration => {
                    match type_collection_name {
                        Some(..) => {}
                        None => {
                            let interface = format!("typeCollection {{\n",);
                            let interface = IndentedString::new(0, interface.to_string());
                            type_collection_name = Some("No Name Set".to_string());
                            ret_vec.push(interface);
                        }
                    }
                    for mut line in self.enumeration(child) {
                        line.indent();
                        ret_vec.push(line);
                    }
                    ret_vec.push(IndentedString::new(0, "".to_string()))
                }
                e => {
                    panic!("Rule: {:?} should not be the type_collection child.", e)
                }
            }
        }
        if ret_vec.len() > 1 {
            let last_element = ret_vec.pop().unwrap();
            if last_element != IndentedString::new(0, "".to_string()) {
                ret_vec.push(last_element);
            }
        }

        if ret_vec.len() == 1 {
            let mut end_str = IndentedString::new(0, "}\n".to_string());
            end_str.set_with_newline(false);
            ret_vec[0] += end_str;
        } else {
            ret_vec.push(IndentedString::new(0, "}\n".to_string()));
        }

        ret_vec
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
                        "interface {} {{",
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
                    ret_vec.push(IndentedString::new(0, "".to_string()))
                }
                Rules::typedef => {
                    for mut line in self.typedef(child) {
                        line.indent();
                        ret_vec.push(line);
                    }
                    // ret_vec.push(IndentedString::new(0, "".to_string()))
                }
                Rules::method => {
                    for mut line in self.method(child) {
                        line.indent();
                        ret_vec.push(line);
                    }
                    ret_vec.push(IndentedString::new(0, "".to_string()))
                }
                Rules::attribute => {
                    for mut line in self.attribute(child) {
                        line.indent();
                        ret_vec.push(line);
                    }
                    ret_vec.push(IndentedString::new(0, "".to_string()))
                }
                Rules::structure => {
                    for mut line in self.structure(child) {
                        line.indent();
                        ret_vec.push(line);
                    }
                    ret_vec.push(IndentedString::new(0, "".to_string()))
                }
                Rules::enumeration => {
                    for mut line in self.enumeration(child) {
                        line.indent();
                        ret_vec.push(line);
                    }
                    ret_vec.push(IndentedString::new(0, "".to_string()))
                }
                Rules::annotation_block => {
                    for mut line in self.annotation_block(child) {
                        ret_vec.push(line);
                    }
                }
                e => {
                    panic!("Rule: {:?} should not be the interfaces child.", e)
                }
            }
        }
        if ret_vec.len() > 1 {
            let last_element = ret_vec.pop().unwrap();
            if last_element != IndentedString::new(0, "".to_string()) {
                ret_vec.push(last_element);
            }
        }
        if ret_vec.len() == 1 {
            let mut end_str = IndentedString::new(0, "}\n".to_string());
            end_str.set_with_newline(false);
            ret_vec[0] += end_str;
        } else {
            ret_vec.push(IndentedString::new(0, "}\n".to_string()));
        }

        ret_vec
    }

    fn annotation_name(&self, node: &Node) -> String {
        node.get_string(&self.source).trim_start().to_string()
    }
    fn annotation_content(&self, node: &Node) -> Vec<IndentedString> {
        let mut ret_vec: Vec<IndentedString> = Vec::new();
        let content = node
            .get_string(&self.source)
            .trim_start()
            .trim_end()
            .replace("\r", "");
        let content = content.split('\n');
        for line in content {
            let line = line.trim_start().trim_end();
            let line = IndentedString::new(0, line.to_string());
            ret_vec.push(line);
        }
        ret_vec
    }

    fn annotation(&self, node: &Node) -> Vec<IndentedString> {
        debug_assert!(node.rule == Rules::annotation);
        let mut ret_vec: Vec<IndentedString> = Vec::new();
        let mut name: String = "".to_string();
        for child in node.get_children() {
            let child = self.publisher.get_node(*child);
            match child.rule {
                Rules::annotation_name => name = self.annotation_name(child),
                Rules::annotation_content => {
                    let mut content = self.annotation_content(child);
                    if content.len() == 1 {
                        content[0].set_with_newline(false);
                        let ret_str = format!("@{name}: {}", content[0]);
                        let ret_str = IndentedString::new(0, ret_str);
                        ret_vec.push(ret_str);
                    } else {
                        let ret_str = format!("@{name}:");
                        ret_vec.push(IndentedString::new(0, ret_str));
                        for mut line in self.annotation_content(child) {
                            line.indent();
                            ret_vec.push(line);
                        }
                    }
                }
                e => {
                    panic!("Rule: {:?} should not be the annotation child.", e)
                }
            }
        }
        ret_vec
    }

    fn annotation_block(&self, node: &Node) -> Vec<IndentedString> {
        debug_assert!(node.rule == Rules::annotation_block);
        let mut ret_vec: Vec<IndentedString> = Vec::new();
        ret_vec.push(IndentedString::new(0, "<**".to_string()));
        for child in node.get_children() {
            let child = self.publisher.get_node(*child);
            match child.rule {
                Rules::annotation => {
                    for mut line in self.annotation(child) {
                        line.indent();
                        ret_vec.push(line);
                    }
                }
                e => {
                    panic!("Rule: {:?} should not be the annotation_block child.", e)
                }
            }
        }
        ret_vec.push(IndentedString::new(0, "**>".to_string()));
        ret_vec
    }

    fn enumeration(&self, node: &Node) -> Vec<IndentedString> {
        debug_assert!(node.rule == Rules::enumeration);
        let mut ret_vec: Vec<IndentedString> = Vec::new();
        let mut var_name: String = "".to_string();
        for child in node.get_children() {
            let child = self.publisher.get_node(*child);
            match child.rule {
                Rules::annotation_block => {
                    for line in self.annotation_block(child) {
                        ret_vec.push(line);
                    }
                }
                Rules::type_dec => {
                    var_name = self.type_dec(child);
                    ret_vec.push(IndentedString::new(0, format!("enumeration {var_name} {{")));
                }
                Rules::enum_value => {
                    for mut line in self.enum_value(child) {
                        line.indent();
                        ret_vec.push(line);
                    }
                }
                e => {
                    panic!("Rule: {:?} should not be the enumeration child.", e)
                }
            }
        }
        if ret_vec.len() == 1 {
            ret_vec[0] += IndentedString::new(0, "}\n".to_string());
        } else {
            ret_vec.push(IndentedString::new(0, "}\n".to_string()));
        }

        ret_vec
    }
    fn enum_value(&self, node: &Node) -> Vec<IndentedString> {
        debug_assert!(node.rule == Rules::enum_value);
        let mut ret_vec: Vec<IndentedString> = Vec::new();
        let mut var_name: String = "".to_string();
        let mut number: Option<String> = None;
        for child in node.get_children() {
            let child = self.publisher.get_node(*child);
            match child.rule {
                Rules::annotation_block => {
                    for line in self.annotation_block(child) {
                        ret_vec.push(line);
                    }
                }
                Rules::variable_name => var_name = self.variable_name(child),
                Rules::number => number = Some(self.number(child)),
                e => {
                    panic!("Rule: {:?} should not be the enum_value child.", e)
                }
            }
        }
        let res_string = match number {
            None => format!("{var_name}"),
            Some(number) => format!("{var_name} = {number}"),
        };
        let res_string = IndentedString::new(0, res_string);
        ret_vec.push(res_string);

        ret_vec
    }
    fn type_dec(&self, node: &Node) -> String {
        node.get_string(&self.source)
    }

    fn typedef(&self, node: &Node) -> Vec<IndentedString> {
        debug_assert!(node.rule == Rules::typedef);
        let mut type_dec = "".to_string();
        let mut ret_vec: Vec<IndentedString> = Vec::new();

        for child in node.get_children() {
            let child = self.publisher.get_node(*child);
            match child.rule {
                Rules::annotation_block => {
                    for line in self.annotation_block(child) {
                        ret_vec.push(line);
                    }
                }
                Rules::type_dec => type_dec = self.type_dec(child),
                Rules::type_ref => {
                    let type_ref = self.type_ref(child);
                    let result = format!("typedef {} is {}", type_dec, type_ref);
                    let result = IndentedString::new(0, result);
                    ret_vec.push(result);
                }
                e => {
                    panic!("Rule: {:?} should not be the typedefs child.", e)
                }
            }
        }
        ret_vec
    }

    fn structure(&self, node: &Node) -> Vec<IndentedString> {
        debug_assert!(node.rule == Rules::structure);

        let mut ret_vec: Vec<IndentedString> = Vec::new();
        for child in node.get_children() {
            let child = self.publisher.get_node(*child);
            match child.rule {
                Rules::annotation_block => {
                    for line in self.annotation_block(child) {
                        ret_vec.push(line);
                    }
                }
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
                Rules::annotation_block => {
                    for line in self.annotation_block(child) {
                        ret_vec.push(line);
                    }
                }
                Rules::type_ref => type_ref = self.type_ref(child),
                Rules::variable_name => {
                    var_name = self.variable_name(child);
                    let attr =
                        IndentedString::new(0, format!("attribute {} {}", type_ref, var_name));
                    ret_vec.push(attr);
                }
                Rules::comment => {
                    ret_vec.push(self.comment(child, true));
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
                Rules::annotation_block => {
                    for line in self.annotation_block(child) {
                        ret_vec.push(line);
                    }
                }
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
        let mut in_already_there = false;
        for child in node.get_children() {
            let child = self.publisher.get_node(*child);
            match child.rule {
                Rules::annotation_block => {
                    for line in self.annotation_block(child) {
                        ret_vec.push(line);
                    }
                }
                Rules::variable_declaration => {
                    match in_already_there {
                        false => {
                            ret_vec.push(IndentedString::new(0, "in {".to_owned()));
                            in_already_there = true
                        }
                        true => {}
                    }
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
        let mut out_already_there = false;
        for child in node.get_children() {
            let child = self.publisher.get_node(*child);
            match child.rule {
                Rules::annotation_block => {
                    for line in self.annotation_block(child) {
                        ret_vec.push(line);
                    }
                }
                Rules::variable_declaration => {
                    match out_already_there {
                        false => {
                            ret_vec.push(IndentedString::new(0, "in {".to_owned()));
                            out_already_there = true;
                        }
                        true => {}
                    }
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
                Rules::annotation_block => {
                    for line in self.annotation_block(child) {
                        ret_vec.push(line);
                    }
                }
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

    fn package(&self, node: &Node) -> IndentedString {
        debug_assert!(node.rule == Rules::package);

        let mut ret_str: IndentedString = IndentedString::default();
        for child in node.get_children() {
            let child = self.publisher.get_node(*child);
            match child.rule {
                Rules::type_ref => {
                    let s = format!("package {}", self.type_ref(child));
                    let s = IndentedString::new(0, s);
                    ret_str = s;
                }
                Rules::comment => ret_str += self.comment(child, true),

                e => {
                    panic!("Rule: {:?} should not be the packages child.", e)
                }
            }
        }
        ret_str
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
    fn number(&self, node: &Node) -> String {
        debug_assert!(node.rule == Rules::number);
        node.get_string(&self.source)
    }
    fn comment(&self, node: &Node, leading_space: bool) -> IndentedString {
        // type_ref is a terminal so we can just return the str slice
        debug_assert!(node.rule == Rules::comment, "{:?}", node);
        let comment_string = &node.get_string(&self.source)[2..];
        let comment_string = comment_string.trim_start().trim_end();
        match leading_space {
            true => IndentedString::new(0, " // ".to_owned() + comment_string),
            false => IndentedString::new(0, "// ".to_owned() + comment_string),
        }
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
