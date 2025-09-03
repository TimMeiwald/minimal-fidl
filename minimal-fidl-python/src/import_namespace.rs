use std::path::PathBuf;

use minimal_fidl_collect::ImportNamespace;
use pyo3::prelude::*;

use crate::diff::FidlDiff;

#[pyclass(name = "FidlImportNamespace", frozen)]
#[derive(Clone, Debug)]
pub struct FidlImportNamespace {
    #[pyo3(get)]
    from_: PathBuf,
    #[pyo3(get)]
    imports: Vec<String>,
    #[pyo3(get)]
    wildcard: bool,
}
#[pymethods]
impl FidlImportNamespace {
    fn __str__(&self) -> String {
        format!("{:#?}", self)
    }
    fn diff(&self, other: &Self) -> FidlDiff {
        if self.from_ != other.from_ || self.imports == other.imports || self.wildcard == other.wildcard {
            FidlDiff::MAJOR
        } else {
            FidlDiff::IDENTICAL
        }
    }
}
impl From<&ImportNamespace> for FidlImportNamespace {
    fn from(item: &ImportNamespace) -> Self {
        FidlImportNamespace {
            imports: item.import.clone(),
            from_: item.from.clone(),
            wildcard: item.wildcard,
        }
    }
}
