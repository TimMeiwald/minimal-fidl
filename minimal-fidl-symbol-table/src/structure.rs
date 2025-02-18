use std::{path::{Path, PathBuf}, str::FromStr};

use crate::{symbol_table::SymbolTableError, VariableDeclaration};
use minimal_fidl_parser::{BasicPublisher, Key, Node, Rules};
#[derive(Debug, Clone)]
pub struct Structure {
    start_position: u32,
    end_position: u32,
    pub name: String,
    contents: Vec<VariableDeclaration>

}
impl Structure {
    pub fn new(
        source: &str,
        publisher: &BasicPublisher,
        node: &Node,
    ) -> Result<Self, SymbolTableError> {
        debug_assert_eq!(node.rule, Rules::structure);
        let mut name: Result<String, SymbolTableError> = Err(
            SymbolTableError::InternalLogicError("Uninitialized value: name in Structure::new".to_string()),
        );
        let mut contents: Vec<VariableDeclaration> = Vec::new();
        for child in node.get_children() {
            let child = publisher.get_node(*child);
            match child.rule {
                Rules::comment
                | Rules::multiline_comment
                | Rules::open_bracket
                | Rules::close_bracket => {},
                Rules::type_dec => {
                    name = Ok(child.get_string(source));
                }
                Rules::variable_declaration => {
                    let var_dec= VariableDeclaration::new(source, publisher, child)?;
                    Self::add_variable_declaration(&mut contents, var_dec)?;
                }

                rule => {
                    return Err(SymbolTableError::UnexpectedNode(
                        rule,
                        "Structure::new".to_string(),
                    ));
                }
            }
        }
        Ok(Self { name: name?, contents, start_position: node.start_position, end_position: node.end_position})
    }

    fn add_variable_declaration(var_decs: &mut Vec<VariableDeclaration>, var_dec: VariableDeclaration) -> Result<(), SymbolTableError> {
        let res: u32 = var_decs
            .iter()
            .map(|intfc| intfc.name == var_dec.name)
            .fold(0, |mut acc, result| {
                acc += result as u32;
                acc
            });
        if res == 0{
            var_decs.push(var_dec);
            Ok(())
        }
        else{
            Err(SymbolTableError::FieldAlreadyExists(var_dec.name))
        }
    }

}
