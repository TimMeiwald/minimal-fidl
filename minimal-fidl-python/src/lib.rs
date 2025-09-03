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

    use crate::file::FidlFile;
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
