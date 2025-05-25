use pyo3::prelude::*;

/// A Python module implemented in Rust. The name of this function must match
/// the `lib.name` setting in the `Cargo.toml`, else Python will not be able to
/// import the module.
#[pymodule]
mod franca_idl_rs {
    use minimal_fidl_collect::fidl_file::{FidlFile, FileError};
    use minimal_fidl_collect::{
        Annotation, Attribute, FidlProject, Interface, Structure, VariableDeclaration, Version,
    };
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
        #[pyo3(get)]
        pub annotations: Vec<PyAnnotation>,
        #[pyo3(get)]
        pub name: String,
        #[pyo3(get)]
        pub version: Option<PyVersion>,
        #[pyo3(get)]
        pub attributes: Vec<PyAttribute>,
        #[pyo3(get)]
        pub structures: Vec<PyStructure>,
        #[pyo3(get)]
        pub typedefs: Vec<PyTypeDef>,
        // #[pyo3(get)]
        // pub methods: Vec<PyMethod>,
        // #[pyo3(get)]
        // pub enumerations: Vec<PyEnumeration>,
    }
    impl From<&Interface> for PyInterface {
        fn from(iface: &Interface) -> Self {
            let version = match &iface.version {
                None => None,
                Some(version) => Some(PyVersion::from(version)),
            };
            let annotations = iface
                .annotations
                .iter()
                .map(|a| PyAnnotation::from(a))
                .collect();
            PyInterface {
                name: iface.name.clone(),
                version,
                annotations,
                attributes: iface
                    .attributes
                    .iter()
                    .map(|a| PyAttribute::from(a))
                    .collect(),
                structures: iface
                    .structures
                    .iter()
                    .map(|a| PyStructure::from(a))
                    .collect(),
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

    #[pyclass(name = "FidlAnnotation", frozen)]
    #[derive(Clone, Debug)]
    struct PyAnnotation {
        #[pyo3(get)]
        pub name: String,
        #[pyo3(get)]
        pub contents: String,
    }
    impl From<&Annotation> for PyAnnotation {
        fn from(item: &Annotation) -> Self {
            PyAnnotation {
                name: item.name.clone(),
                contents: item.contents.clone(),
            }
        }
    }

    #[pyclass(name = "FidlAttribute", frozen)]
    #[derive(Clone, Debug)]
    struct PyAttribute {
        #[pyo3(get)]
        pub annotations: Vec<PyAnnotation>,
        #[pyo3(get)]
        pub name: String,
        #[pyo3(get)]
        pub type_name: String,
    }
    impl From<&Attribute> for PyAttribute {
        fn from(item: &Attribute) -> Self {
            PyAttribute {
                annotations: item
                    .annotations
                    .iter()
                    .map(|a| PyAnnotation::from(a))
                    .collect(),
                name: item.name.clone(),
                type_name: item.type_n.clone(),
            }
        }
    }
    #[pyclass(name = "FidlStructure", frozen)]
    #[derive(Clone, Debug)]
    struct PyStructure {
        #[pyo3(get)]
        pub annotations: Vec<PyAnnotation>,
        #[pyo3(get)]
        pub name: String,
        #[pyo3(get)]
        pub contents: Vec<PyVariableDeclaration>,
    }
    impl From<&Structure> for PyStructure {
        fn from(item: &Structure) -> Self {
            PyStructure {
                annotations: item
                    .annotations
                    .iter()
                    .map(|a| PyAnnotation::from(a))
                    .collect(),
                name: item.name.clone(),
                contents: item
                    .contents
                    .iter()
                    .map(|a| PyVariableDeclaration::from(a))
                    .collect(),
            }
        }
    }
    #[pyclass(name = "FidlVariableDeclaration", frozen)]
    #[derive(Clone, Debug)]
    struct PyVariableDeclaration {
        #[pyo3(get)]
        pub annotations: Vec<PyAnnotation>,
        #[pyo3(get)]
        pub name: String,
        #[pyo3(get)]
        pub type_name: String,
        #[pyo3(get)]
        pub is_array: bool,

    }
    impl From<&VariableDeclaration> for PyVariableDeclaration {
        fn from(item: &VariableDeclaration) -> Self {
            PyVariableDeclaration {
                annotations: item
                    .annotations
                    .iter()
                    .map(|a| PyAnnotation::from(a))
                    .collect(),
                name: item.name.clone(),
                type_name: item.type_n.clone(),
                is_array: item.is_array
            }
        }
    }
}
