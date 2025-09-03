use minimal_fidl_collect::{FidlFileRs, FidlProject};
use pyo3::prelude::*;

use crate::{
    collection::FidlTypeCollection, franca_idl::FidlFileError, import_model::FidlImportModel,
    import_namespace::FidlImportNamespace, interface::FidlInterface, package::FidlPackage,
};

#[pyclass(name = "FidlFile", frozen)]
#[derive(Debug, Clone)]
pub struct FidlFile {
    // #[pyo3(get)]
    // pub source: String,
    #[pyo3(get)]
    pub file_path: Option<String>,
    #[pyo3(get)]
    pub package: Option<FidlPackage>,
    #[pyo3(get)]
    pub namespaces: Vec<FidlImportNamespace>,
    #[pyo3(get)]
    pub import_models: Vec<FidlImportModel>,
    #[pyo3(get)]
    pub interfaces: Vec<FidlInterface>,
    #[pyo3(get)]
    pub type_collections: Vec<FidlTypeCollection>,
}
impl From<FidlFileRs> for FidlFile {
    fn from(item: FidlFileRs) -> Self {
        FidlFile {
            file_path: None,
            interfaces: item
                .interfaces
                .iter()
                .map(|iface| FidlInterface::from(iface))
                .collect(),

            type_collections: item
                .type_collections
                .iter()
                .map(|iface| FidlTypeCollection::from(iface))
                .collect(),
            import_models: item
                .import_models
                .iter()
                .map(|iface| FidlImportModel::from(iface))
                .collect(),
            namespaces: item
                .namespaces
                .iter()
                .map(|iface| FidlImportNamespace::from(iface))
                .collect(),
            package: item
                .package
                .and_then(|package| Some(FidlPackage::from(&package))),
        }
    }
}

#[pymethods]
impl FidlFile {
    #[new]
    pub fn new(file_path: String) -> Result<Self, FidlFileError> {
        let result = FidlProject::generate_file(file_path.clone())?;
        let mut fidl_file = FidlFile::from(result);
        fidl_file.file_path = Some(file_path);
        Ok(fidl_file)
    }

    #[staticmethod]
    fn new_from_string(file_string: String) -> Result<Self, FidlFileError> {
        let result = FidlProject::generate_file_from_string(file_string)?;
        Ok(FidlFile::from(result))
    }

    fn __str__(&self) -> String {
        format!("{:#?}", self)
    }
}
