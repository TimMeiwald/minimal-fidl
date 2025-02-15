use minimal_fidl_parser::{BasicPublisher, Key, Node, Rules};
use thiserror::Error;
use crate::symbol_table::SymbolTableError;
use crate::symbol_table::SymbolTable;


pub struct SymbolTableBuilder<'a> {
    source: &'a str,
    publisher: &'a BasicPublisher,
}

impl<'a> SymbolTableBuilder<'a> {
    pub fn new(source: &'a str, publisher: &'a BasicPublisher) -> Self {
        Self { source, publisher }
        
    }

    pub fn create_symbol_table(&self) -> Result<SymbolTable, SymbolTableError> {
        Ok(SymbolTable::new(&self.source, &self.publisher))
    }
}

