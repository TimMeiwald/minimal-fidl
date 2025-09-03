use minimal_fidl_collect::Version;
use pyo3::prelude::*;

use crate::diff::FidlDiff;

#[pyclass(name = "FidlVersion", frozen)]
#[derive(Clone, Debug)]
pub struct FidlVersion {
    #[pyo3(get)]
    pub major: Option<u32>,
    #[pyo3(get)]
    pub minor: Option<u32>,
}
#[pymethods]
impl FidlVersion {
    fn __str__(&self) -> String {
        format!("{:#?}", self)
    }

    fn diff(&self) -> Result<FidlDiff, ()>{
        Ok(FidlDiff::IDENTICAL)
    }
}
impl From<&Version> for FidlVersion {
    fn from(item: &Version) -> Self {
        FidlVersion {
            major: item.major,
            minor: item.minor,
        }
    }
}
