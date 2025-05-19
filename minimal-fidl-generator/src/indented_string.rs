use std::fmt::{self};

use std::ops::AddAssign;

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum FidlType {
    File,
    Package,
    Interface,
    Enumeration,
    EnumValue,
    Method,
    Attribute,
    Structure,
    TypeCollection,
}

#[derive(PartialEq, Debug)]
pub struct IndentedString {
    indent_level: u8,
    str: String,
    with_newline: bool,
    fidl_type: FidlType,
}
impl IndentedString {
    pub fn new(indent_level: u8, fidl_type: FidlType, str: String) -> Self {
        Self {
            str,
            indent_level,
            with_newline: true,
            fidl_type: fidl_type,
        }
    }

    pub fn indent(mut self) -> Self {
        self.indent_level += 1;
        self
    }

    pub fn set_with_newline(mut self, with_newline: bool) -> Self {
        self.with_newline = with_newline;
        self
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
            fidl_type: self.fidl_type,
            str: self.str.clone() + &other.str,
            indent_level: self.indent_level,
            with_newline: self.with_newline,
        };
    }
}
