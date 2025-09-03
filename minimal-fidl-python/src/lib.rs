use pyo3::prelude::*;
mod annotation;
mod attribute;
mod collection;
mod diff;
mod enum_value;
mod enumeration;
mod file;
mod import_model;
mod import_namespace;
mod interface;
mod method;
mod package;
mod structure;
mod type_def;
mod variable_declaration;
mod version;
/// A Python module implemented in Rust. The name of this function must match
/// the `lib.name` setting in the `Cargo.toml`, else Python will not be able to
/// import the module.
#[pymodule]
mod franca_idl {
    use std::path::PathBuf;

    use minimal_fidl_collect::{FidlProject, FileError};
    use pyo3::exceptions::PyValueError;
    use pyo3::prelude::*;

    #[pymodule_export]
    use crate::annotation::FidlAnnotation;
    #[pymodule_export]
    use crate::attribute::FidlAttribute;
    #[pymodule_export]
    use crate::collection::FidlTypeCollection;
    #[pymodule_export]
    use crate::diff::FidlDiff;
    #[pymodule_export]
    use crate::enum_value::FidlEnumValue;
    #[pymodule_export]
    use crate::enumeration::FidlEnumeration;
    #[pymodule_export]
    use crate::file::FidlFile;
    #[pymodule_export]
    use crate::import_model::FidlImportModel;
    #[pymodule_export]
    use crate::import_namespace::FidlImportNamespace;
    #[pymodule_export]
    use crate::interface::FidlInterface;
    #[pymodule_export]
    use crate::method::FidlMethod;
    #[pymodule_export]
    use crate::package::FidlPackage;
    #[pymodule_export]
    use crate::structure::FidlStructure;
    #[pymodule_export]
    use crate::type_def::FidlTypeDef;
    #[pymodule_export]
    use crate::variable_declaration::FidlVariableDeclaration;
    #[pymodule_export]
    use crate::version::FidlVersion;

    #[pyfunction]
    fn _respond_42() -> u8 {
        42
    }
    #[pyfunction]
    fn load_fidl_project(dir: PathBuf) -> Result<Vec<FidlFile>, PyErr> {
        match FidlProject::new(dir) {
            Err(e) => Err(PyValueError::new_err(e.to_string())),
            Ok(file_paths) => {
                let mut fidl_files: Vec<FidlFile> = Vec::new();
                for path in file_paths {
                    let fidl_file = FidlFile::new(path.as_os_str().to_string_lossy().to_string())?;
                    fidl_files.push(fidl_file);
                }
                Ok(fidl_files)
            }
        }
    }

    pub struct FidlFileError(FileError);

    impl From<FidlFileError> for PyErr {
        fn from(error: FidlFileError) -> Self {
            PyValueError::new_err(error.0.to_string())
        }
    }

    impl From<FileError> for FidlFileError {
        fn from(other: FileError) -> Self {
            Self(other)
        }
    }
}
