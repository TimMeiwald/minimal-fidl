//! Formatting entry point.
//!
//! Formatting is now `parse -> AST -> print`: the tree in `minimal-fidl-collect`
//! is the single description of a `.fidl` file, and `FidlFile::to_fidl` is the
//! single renderer. This crate is a thin adapter kept for its existing API.
//!
//! It replaces a 1119-line walker over the parse tree. That walker could not
//! print an edited file (it read the CST, which has no mutation API), produced
//! output that failed to reparse for two of its own test inputs, and was not
//! idempotent for a third. See `minimal-fidl-collect/DESIGN.md` §8.

use minimal_fidl_collect::{FidlFile, FileError, Mode};
use minimal_fidl_parser::BasicPublisher;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FormatterError {
    #[error("Could not build a syntax tree from the source: {0}")]
    CouldNotBuildTree(#[from] FileError),
}

pub struct Formatter<'a> {
    source: &'a str,
    publisher: &'a BasicPublisher,
}

impl<'a> Formatter<'a> {
    pub fn new(source: &'a str, publisher: &'a BasicPublisher) -> Self {
        Formatter { source, publisher }
    }

    /// Format the source, laying everything out from the tree.
    pub fn format(&self) -> Result<String, FormatterError> {
        Ok(self.tree()?.to_fidl())
    }

    /// Format, but emit any subtree that was not modified exactly as it was read.
    ///
    /// Formatting a freshly parsed file this way is close to a no-op, which is
    /// what makes it useful for editing tools that want a minimal diff.
    pub fn format_preserving(&self) -> Result<String, FormatterError> {
        Ok(self.tree()?.to_fidl_with(Mode::Preserve))
    }

    fn tree(&self) -> Result<FidlFile, FormatterError> {
        Ok(FidlFile::new(self.source.to_string(), self.publisher)?)
    }
}
