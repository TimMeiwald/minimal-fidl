use clap::builder::Str;
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
                Rules::interface => {
                    let interface = self.interface(&c);
                    for line in interface{
                        ret_string += &line;
                    }
                },
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

    fn interface(&self, node: &Node) -> Vec<String> {
        debug_assert!(node.rule == Rules::interface);
        let mut interface_name: Option<String> = None;
        let mut version: Option<Vec<String>> = None;
        let mut methods: Vec<Vec<String>> = Vec::new();
        let mut return_vec: Vec<String> = Vec::new();
        for child in node.get_children() {
            let child = self.publisher.get_node(*child);
            match child.rule {
                Rules::variable_name => interface_name = Some(self.variable_name(child)),
                Rules::version => version = Some(self.version(child)),
                Rules::method => {methods.push(self.method(child));}
                e => {
                    panic!("Rule: {:?} should not be the interfaces child.", e)
                }
            }
        }
        return_vec.push(format!("\ninterface {} {{", interface_name.expect("Interface Name should always exist")));
        match version{
            None => {}
            Some(version) =>{
                for line in version {
                    return_vec.push("\n    ".to_owned() + &line);
                }
                return_vec.push("\n".to_string());
            }
        }
        if methods.len() != 0{
            for method in methods{
                for line in method{
                    return_vec.push("\n    ".to_owned() + &line);    
                }
            }


        }
        if return_vec.len() == 1{
            return_vec[0] += "}";
        }
        else{
            return_vec.push("\n}".to_owned());
        }
        return return_vec;
    }

    fn version(&self, node: &Node) -> Vec<String> {
        let mut major: Option<String> = None;
        let mut minor: Option<String> = None;
        debug_assert!(node.rule == Rules::version);
        for child in node.get_children(){
            let child = self.publisher.get_node(*child);
            match  child.rule {
                Rules::major => major = Some(self.major(child)),
                Rules::minor => minor = Some(self.minor(child)),
                e => {
                    panic!("Rule: {:?} should not be the version child.", e)
                }
            }
        }
        let major = major.expect("Should have major");
        let minor = minor.expect("Should have minor");
        let major = "    ".to_owned() + &major;
        let minor = "    ".to_owned() + &minor;
        let mut return_vec: Vec<String> = Vec::new();
        return_vec.push("version {".to_string());
        return_vec.push(major);
        return_vec.push(minor);
        return_vec.push("}".to_string());
        return_vec
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



    fn method(&self, node: &Node) -> Vec<String> {
        debug_assert!(node.rule == Rules::method);
        let mut var_name: String = "".to_string();
        let mut input: Vec<String> = Vec::new();
        let mut output: Vec<String> = Vec::new();
        for child in node.get_children(){
            let child = self.publisher.get_node(*child);
            match  child.rule {
                Rules::variable_name => var_name = self.variable_name(child),
                Rules::input_params => input = self.input_params(child),
                Rules::output_params => output = self.output_params(child),
                e => {
                    panic!("Rule: {:?} should not be the method child.", e)
                }
            }
        };
        let mut return_vec: Vec<String> = Vec::new();
        return_vec.push(format!("method {} {{", var_name));

        for line in input{
            return_vec.push("    ".to_owned() + &line);
        }
        for line in output{
            return_vec.push("    ".to_owned() + &line);
        }


        if return_vec.len() == 1{
            return_vec[0] += "}";
        }
        else{
            return_vec.push("}".to_owned());
        }
        return_vec
    }

    fn input_params(&self, node: &Node) -> Vec<String> {
        debug_assert!(node.rule == Rules::input_params);
        let mut return_vec: Vec<String> = Vec::new();
        return_vec.push("in {".to_owned());
        for child in node.get_children(){
            let child = self.publisher.get_node(*child);
            match  child.rule {
                Rules::variable_declaration => return_vec.push("    ".to_owned() + &self.variable_declaration(child)),
                e => {
                    panic!("Rule: {:?} should not be the version child.", e)
                }
            }
        };
        return_vec.push("}".to_owned());

        return_vec
    }

    fn output_params(&self, node: &Node) -> Vec<String> {
        debug_assert!(node.rule == Rules::output_params);
        let mut return_vec: Vec<String> = Vec::new();
        return_vec.push("out {".to_owned());
        for child in node.get_children(){
            let child = self.publisher.get_node(*child);
            match  child.rule {
                Rules::variable_declaration => return_vec.push("    ".to_owned() + &self.variable_declaration(child)),
                e => {
                    panic!("Rule: {:?} should not be the version child.", e)
                }
            }
        };
        return_vec.push("}".to_owned());

        return_vec
    }


    
    fn variable_declaration(&self, node: &Node) -> String {
        debug_assert!(node.rule == Rules::variable_declaration);
        let mut type_ref: String = "".to_string();
        let mut var_name: String = "".to_string();
        for child in node.get_children(){
            let child = self.publisher.get_node(*child);
            match  child.rule {
                Rules::type_ref => type_ref = self.type_ref(child),
                Rules::variable_name => var_name = self.variable_name(child),
                e => {
                    panic!("Rule: {:?} should not be the version child.", e)
                }
            }
        };
        format!("{} {}", type_ref, var_name)
    }




    fn package(&self, node: &Node) -> String {
        debug_assert!(node.rule == Rules::package);

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
