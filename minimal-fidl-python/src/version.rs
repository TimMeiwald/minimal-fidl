use minimal_fidl_collect::Version;
use pyo3::prelude::*;

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
}
impl From<&Version> for FidlVersion {
    fn from(item: &Version) -> Self {
        FidlVersion {
            major: item.major,
            minor: item.minor,
        }
    }
}
