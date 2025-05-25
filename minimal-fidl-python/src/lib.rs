use pyo3::prelude::*;

/// A Python module implemented in Rust. The name of this function must match
/// the `lib.name` setting in the `Cargo.toml`, else Python will not be able to
/// import the module.
#[pymodule]
mod franca_idl_rs {
    use minimal_fidl_collect::fidl_file::{FidlFile, FileError};
    use minimal_fidl_collect::{FidlProject, Interface, Version};
    use pyo3::exceptions::PyValueError;
    use pyo3::prelude::*;
    #[pyfunction]
    fn _respond_42() -> u8 {
        42
    }

    struct PyFileError(FileError);

    impl From<PyFileError> for PyErr {
        fn from(error: PyFileError) -> Self {
            PyValueError::new_err(error.0.to_string())
        }
    }

    impl From<FileError> for PyFileError {
        fn from(other: FileError) -> Self {
            Self(other)
        }
    }

    #[pyclass(name = "FidlFile", frozen)] // We need to rename it so it's not PyFidlFile but we can't use that since
    #[derive(Debug, Clone)] // The rust type is also FidlFile
    struct PyFidlFile {
        // #[pyo3(get)]
        // pub source: String,
        // #[pyo3(get)]
        // pub package: Option<PyPackage>,
        // #[pyo3(get)]
        // pub namespaces: Vec<PyImportNamespace>,
        // #[pyo3(get)]
        // pub import_models: Vec<PyImportModel>,
        #[pyo3(get)]
        pub interfaces: Vec<PyInterface>,
        // #[pyo3(get)]
        // pub type_collections: Vec<PyTypeCollection>,
    }
    impl From<FidlFile> for PyFidlFile {
        fn from(item: FidlFile) -> Self {
            PyFidlFile {
                interfaces: item
                    .interfaces
                    .iter()
                    .map(|iface| PyInterface::from(iface))
                    .collect(),
            }
        }
    }
    #[pymethods]
    impl PyFidlFile {
        #[new]
        fn new(file_path: String) -> Result<Self, PyFileError> {
            let result = FidlProject::generate_file(file_path)?;
            Ok(PyFidlFile::from(result))
        }

        #[staticmethod]
        fn new_from_string(file_string: String) -> Result<Self, PyFileError> {
            let result = FidlProject::generate_file_from_string(file_string)?;
            Ok(PyFidlFile::from(result))
        }

        fn __str__(&self) -> String {
            format!("{:#?}", self)
        }
    }

    #[pyclass(name = "FidlInterface", frozen)]
    #[derive(Clone, Debug)]
    struct PyInterface {
        // start_position: u32,
        // end_position: u32,
        // pub annotations: Vec<Annotation>,
        #[pyo3(get)]
        pub name: String,
        #[pyo3(get)]
        pub version: Option<PyVersion>,
        // pub attributes: Vec<Attribute>,
        // pub structures: Vec<Structure>,
        // pub typedefs: Vec<TypeDef>,
        // pub methods: Vec<Method>,
        // pub enumerations: Vec<Enumeration>,
    }
    impl From<&Interface> for PyInterface {
        fn from(iface: &Interface) -> Self {
            let version = match &iface.version{
                None => None,
                Some(version) => {Some(PyVersion::from(version))}
            };
            PyInterface {
                name: iface.name.clone(),
                version,
            }
        }
    }

    #[pyclass(name = "FidlVersion", frozen)]
    #[derive(Clone, Debug)]
    struct PyVersion {
        #[pyo3(get)]
        pub major: Option<u32>,
        #[pyo3(get)]
        pub minor: Option<u32>,
    }
    impl From<&Version> for PyVersion {
        fn from(item: &Version) -> Self {
            PyVersion {
                major: item.major,
                minor: item.minor,
            }
        }
    }
}
