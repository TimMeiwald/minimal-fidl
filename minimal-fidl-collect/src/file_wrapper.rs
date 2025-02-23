use minimal_fidl_parser::{BasicPublisher, Key, Node, Rules};
use thiserror::Error;
use crate::fidl_file::FileError;
use crate::fidl_file::FidlFile;


pub struct FileWrapper<'a> {
    source: &'a str,
    publisher: &'a BasicPublisher,
}

impl<'a> FileWrapper<'a> {
    pub fn new(source: &'a str, publisher: &'a BasicPublisher) -> Self {
        Self { source, publisher }
        
    }

    pub fn load_file(&self) -> Result<FidlFile, FileError> {
        FidlFile::new(&self.source, &self.publisher)
    }
}

