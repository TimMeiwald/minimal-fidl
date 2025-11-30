use std::path::PathBuf;

use minimal_fidl_collect::ImportModel;
use pyo3::prelude::*;

use crate::diff::FidlDiff;

#[pyclass(name = "FidlImportModel", frozen)]
#[derive(Clone, Debug)]
pub struct FidlImportModel {
    #[pyo3(get)]
    file_path: PathBuf,
}
#[pymethods]
impl FidlImportModel {
    fn __str__(&self) -> String {
        format!("{:#?}", self)
    }
    fn diff(&self, other: &Self) -> FidlDiff {
        if self.file_path != other.file_path {
            FidlDiff::MAJOR
        } else {
            FidlDiff::IDENTICAL
        }
    }
}
impl From<&ImportModel> for FidlImportModel {
    fn from(item: &ImportModel) -> Self {
        FidlImportModel {
            file_path: item.file_path.clone(),
        }
    }
}
