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

    // The version defines the version so 
    // it always returns FidlDiff::IDENTICAL as 
    // it makes little sense to diff it. 
    fn diff(&self, _other: &Self) -> FidlDiff {
        FidlDiff::IDENTICAL
    }

    fn __eq__(&self, other: &Self) -> bool{
        self.major == other.major && self.minor == self.minor
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
