use std::path::PathBuf;

use minimal_fidl_collect::ImportNamespace;
use pyo3::prelude::*;

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
