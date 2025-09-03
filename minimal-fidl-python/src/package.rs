use minimal_fidl_collect::Package;
use pyo3::prelude::*;

#[pyclass(name = "FidlPackage", frozen)]
#[derive(Clone, Debug)]
pub struct FidlPackage {
    #[pyo3(get)]
    path: Vec<String>,
}
#[pymethods]
impl FidlPackage {
    fn __str__(&self) -> String {
        format!("{:#?}", self)
    }
}
impl From<&Package> for FidlPackage {
    fn from(item: &Package) -> Self {
        FidlPackage {
            path: item.path.clone(),
        }
    }
}
