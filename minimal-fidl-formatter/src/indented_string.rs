use std::fmt::{self};

use std::ops::AddAssign;

use minimal_fidl_parser::Rules;

#[derive(PartialEq, Debug)]
pub struct IndentedString {
    indent_level: u8,
    str: String,
    with_newline: bool,
    rule: Rules,
}
impl IndentedString {
    pub fn new(indent_level: u8, str: String) -> Self {
        Self {
            str,
            indent_level,
            with_newline: true,
            rule: Rules::Grammar,
        }
    }

    pub fn indent(&mut self) {
        self.indent_level += 1
    }

    pub fn set_rule(&mut self, rule: Rules) {
        self.rule = rule;
    }
    pub fn get_rule(&self) -> Rules {
        self.rule
    }

    pub fn set_with_newline(&mut self, with_newline: bool) {
        self.with_newline = with_newline;
    }
}
impl fmt::Display for IndentedString {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let spacing = "    ";
        let spacing = spacing.repeat(self.indent_level as usize);
        if self.with_newline {
            write!(f, "\n{spacing}{}", self.str)
        } else {
            write!(f, "{spacing}{}", self.str)
        }
    }
}
impl AddAssign for IndentedString {
    fn add_assign(&mut self, other: Self) {
        *self = Self {
            rule: self.rule,
            str: self.str.clone() + &other.str,
            indent_level: self.indent_level,
            with_newline: self.with_newline,
        };
    }
}

impl Default for IndentedString {
    fn default() -> Self {
        Self {
            rule: Rules::Grammar,
            str: "".to_string(),
            indent_level: 0,
            with_newline: true,
        }
    }
}
// #[test]
// fn test_indented_string() {
//     let mut i = IndentedString::new(0, "str".to_string());
//     println!("{i}");
//     i.indent();
//     println!("{i}");
//     i.indent();
//     println!("{i}");
// }
