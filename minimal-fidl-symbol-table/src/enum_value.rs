use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use crate::symbol_table::SymbolTableError;
use minimal_fidl_parser::{BasicPublisher, Key, Node, Rules};
#[derive(Debug, Clone)]
pub struct EnumValue {
    start_position: u32,
    end_position: u32,
    pub name: String,
    pub value: Option<u64>,

}
impl EnumValue {
    pub fn new(
        source: &str,
        publisher: &BasicPublisher,
        node: &Node,
    ) -> Result<Self, SymbolTableError> {
        debug_assert_eq!(node.rule, Rules::enum_value);
        let mut value: Option<u64> = None;
        let mut name: Result<String, SymbolTableError> = Err(SymbolTableError::InternalLogicError(
            "Uninitialized value: name in EnumValue::new".to_string(),
        ));

        for child in node.get_children() {
            let child = publisher.get_node(*child);
            match child.rule {
                Rules::comment | Rules::multiline_comment | Rules::annotation_block => {}
                Rules::number => {
                    let res = child.get_string(source);
                    value = Some(Self::convert_string_representation_of_number_to_value(res)?);
                }
                Rules::variable_name => {
                    name = Ok(child.get_string(source));
                }

                rule => {
                    return Err(SymbolTableError::UnexpectedNode(
                        rule,
                        "EnumValue::new".to_string(),
                    ));
                }
            }
        }
        Ok(Self { name: name?, value , start_position: node.start_position, end_position: node.end_position})
    }

    pub fn push_if_not_exists_else_err(
        self,
        enum_values: &mut Vec<EnumValue>,
    ) -> Result<(), SymbolTableError> {
        for s in &mut *enum_values {
            if s.name == self.name {
                return Err(SymbolTableError::EnumValueAlreadyExists(
                    s.clone(),
                    self.clone(),
                ));
            }
        }
        enum_values.push(self);
        Ok(())
    }

    fn convert_string_representation_of_number_to_value(
        input: String,
    ) -> Result<u64, SymbolTableError> {
        let value = input.parse::<u64>();

        match value {
            Ok(integer) => return Ok(integer),
            Err(e) => {}
        };
        let hex_input = input.strip_prefix("0x");
        match hex_input {
            Some(hex_input) => {
                let value = u64::from_str_radix(&hex_input, 16);
                match value {
                    Ok(integer) => return Ok(integer),
                    Err(_e) => {}
                };
            }
            None => {}
        }

        let bin_input = input.strip_prefix("0b");
        match bin_input {
            Some(bin_input) => {
                let value = u64::from_str_radix(&bin_input, 2);
                match value {
                    Ok(integer) => return Ok(integer),
                    Err(_e) => {}
                };
            }
            None => {}
        }
        Err(SymbolTableError::CouldNotConvertToInteger(input))
    }
}

#[cfg(test)]
mod tests {
    use super::EnumValue;

    #[test]
    fn test() {
        let val =
            EnumValue::convert_string_representation_of_number_to_value("0x40000".to_string());
        val.unwrap();
    }
}
