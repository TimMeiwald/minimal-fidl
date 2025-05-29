use pyo3::prelude::*;

/// A Python module implemented in Rust. The name of this function must match
/// the `lib.name` setting in the `Cargo.toml`, else Python will not be able to
/// import the module.
#[pymodule]
mod franca_idl {
    use std::path::PathBuf;

    use minimal_fidl_collect::{
        Annotation, Attribute, EnumValue, Enumeration, FidlFileRs, FidlProject, FileError,
        ImportModel, ImportNamespace, Interface, Method, Package, Structure, TypeCollection,
        TypeDef, VariableDeclaration, Version,
    };
    use pyo3::exceptions::PyValueError;
    use pyo3::prelude::*;
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

    struct FidlFileError(FileError);

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

    #[pyclass(name = "FidlFile", frozen)] // We need to rename it so it's not FidlFidlFile but we can't use that since
    #[derive(Debug, Clone)] // The rust type is also FidlFile
    struct FidlFile {
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
        fn new(file_path: String) -> Result<Self, FidlFileError> {
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
    #[pyclass(name = "FidlTypeCollection", frozen)]
    #[derive(Clone, Debug)]
    struct FidlTypeCollection {
        #[pyo3(get)]
        pub annotations: Vec<FidlAnnotation>,
        #[pyo3(get)]
        pub name: String,
        #[pyo3(get)]
        pub version: Option<FidlVersion>,
        #[pyo3(get)]
        pub typedefs: Vec<FidlTypeDef>,
        #[pyo3(get)]
        pub structures: Vec<FidlStructure>,
        #[pyo3(get)]
        pub enumerations: Vec<FidlEnumeration>,
    }
    #[pymethods]
    impl FidlTypeCollection {
        fn __str__(&self) -> String {
            format!("{:#?}", self)
        }
    }
    impl From<&TypeCollection> for FidlTypeCollection {
        fn from(iface: &TypeCollection) -> Self {
            let version = match &iface.version {
                None => None,
                Some(version) => Some(FidlVersion::from(version)),
            };
            let annotations = iface
                .annotations
                .iter()
                .map(|a| FidlAnnotation::from(a))
                .collect();
            FidlTypeCollection {
                name: iface.name.clone(),
                version,
                annotations,
                structures: iface
                    .structures
                    .iter()
                    .map(|a| FidlStructure::from(a))
                    .collect(),
                typedefs: iface
                    .typedefs
                    .iter()
                    .map(|a| FidlTypeDef::from(a))
                    .collect(),
                enumerations: iface
                    .enumerations
                    .iter()
                    .map(|a| FidlEnumeration::from(a))
                    .collect(),
            }
        }
    }

    #[pyclass(name = "FidlInterface", frozen)]
    #[derive(Clone, Debug)]
    struct FidlInterface {
        #[pyo3(get)]
        pub annotations: Vec<FidlAnnotation>,
        #[pyo3(get)]
        pub name: String,
        #[pyo3(get)]
        pub version: Option<FidlVersion>,
        #[pyo3(get)]
        pub attributes: Vec<FidlAttribute>,
        #[pyo3(get)]
        pub structures: Vec<FidlStructure>,
        #[pyo3(get)]
        pub typedefs: Vec<FidlTypeDef>,
        #[pyo3(get)]
        pub methods: Vec<FidlMethod>,
        #[pyo3(get)]
        pub enumerations: Vec<FidlEnumeration>,
    }
    #[pymethods]
    impl FidlInterface {
        fn __str__(&self) -> String {
            format!("{:#?}", self)
        }
    }
    impl From<&Interface> for FidlInterface {
        fn from(iface: &Interface) -> Self {
            let version = match &iface.version {
                None => None,
                Some(version) => Some(FidlVersion::from(version)),
            };
            let annotations = iface
                .annotations
                .iter()
                .map(|a| FidlAnnotation::from(a))
                .collect();
            FidlInterface {
                name: iface.name.clone(),
                version,
                annotations,
                attributes: iface
                    .attributes
                    .iter()
                    .map(|a| FidlAttribute::from(a))
                    .collect(),
                structures: iface
                    .structures
                    .iter()
                    .map(|a| FidlStructure::from(a))
                    .collect(),
                typedefs: iface
                    .typedefs
                    .iter()
                    .map(|a| FidlTypeDef::from(a))
                    .collect(),
                methods: iface.methods.iter().map(|a| FidlMethod::from(a)).collect(),
                enumerations: iface
                    .enumerations
                    .iter()
                    .map(|a| FidlEnumeration::from(a))
                    .collect(),
            }
        }
    }

    #[pyclass(name = "FidlVersion", frozen)]
    #[derive(Clone, Debug)]
    struct FidlVersion {
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

    #[pyclass(name = "FidlAnnotation", frozen)]
    #[derive(Clone, Debug)]
    struct FidlAnnotation {
        #[pyo3(get)]
        pub name: String,
        #[pyo3(get)]
        pub contents: String,
    }
    #[pymethods]
    impl FidlAnnotation {
        fn __str__(&self) -> String {
            format!("{:#?}", self)
        }
    }
    impl From<&Annotation> for FidlAnnotation {
        fn from(item: &Annotation) -> Self {
            FidlAnnotation {
                name: item.name.clone(),
                contents: item.contents.clone(),
            }
        }
    }

    #[pyclass(name = "FidlAttribute", frozen)]
    #[derive(Clone, Debug)]
    struct FidlAttribute {
        #[pyo3(get)]
        pub annotations: Vec<FidlAnnotation>,
        #[pyo3(get)]
        pub name: String,
        #[pyo3(get)]
        pub type_name: String,
    }
    #[pymethods]
    impl FidlAttribute {
        fn __str__(&self) -> String {
            format!("{:#?}", self)
        }
    }
    impl From<&Attribute> for FidlAttribute {
        fn from(item: &Attribute) -> Self {
            FidlAttribute {
                annotations: item
                    .annotations
                    .iter()
                    .map(|a| FidlAnnotation::from(a))
                    .collect(),
                name: item.name.clone(),
                type_name: item.type_n.clone(),
            }
        }
    }
    #[pyclass(name = "FidlStructure", frozen)]
    #[derive(Clone, Debug)]
    struct FidlStructure {
        #[pyo3(get)]
        pub annotations: Vec<FidlAnnotation>,
        #[pyo3(get)]
        pub name: String,
        #[pyo3(get)]
        pub contents: Vec<FidlVariableDeclaration>,
    }
    #[pymethods]
    impl FidlStructure {
        fn __str__(&self) -> String {
            format!("{:#?}", self)
        }
    }
    impl From<&Structure> for FidlStructure {
        fn from(item: &Structure) -> Self {
            FidlStructure {
                annotations: item
                    .annotations
                    .iter()
                    .map(|a| FidlAnnotation::from(a))
                    .collect(),
                name: item.name.clone(),
                contents: item
                    .contents
                    .iter()
                    .map(|a| FidlVariableDeclaration::from(a))
                    .collect(),
            }
        }
    }
    #[pyclass(name = "FidlVariableDeclaration", frozen)]
    #[derive(Clone, Debug)]
    struct FidlVariableDeclaration {
        #[pyo3(get)]
        pub annotations: Vec<FidlAnnotation>,
        #[pyo3(get)]
        pub name: String,
        #[pyo3(get)]
        pub type_name: String,
        #[pyo3(get)]
        pub is_array: bool,
    }
    #[pymethods]
    impl FidlVariableDeclaration {
        fn __str__(&self) -> String {
            format!("{:#?}", self)
        }
    }
    impl From<&VariableDeclaration> for FidlVariableDeclaration {
        fn from(item: &VariableDeclaration) -> Self {
            FidlVariableDeclaration {
                annotations: item
                    .annotations
                    .iter()
                    .map(|a| FidlAnnotation::from(a))
                    .collect(),
                name: item.name.clone(),
                type_name: item.type_n.clone(),
                is_array: item.is_array,
            }
        }
    }

    #[pyclass(name = "FidlTypeDef", frozen)]
    #[derive(Clone, Debug)]
    struct FidlTypeDef {
        #[pyo3(get)]
        pub annotations: Vec<FidlAnnotation>,
        #[pyo3(get)]
        pub name: String,
        #[pyo3(get)]
        pub type_name: String,
        #[pyo3(get)]
        pub is_array: bool,
    }
    #[pymethods]
    impl FidlTypeDef {
        fn __str__(&self) -> String {
            format!("{:#?}", self)
        }
    }
    impl From<&TypeDef> for FidlTypeDef {
        fn from(item: &TypeDef) -> Self {
            FidlTypeDef {
                annotations: item
                    .annotations
                    .iter()
                    .map(|a| FidlAnnotation::from(a))
                    .collect(),
                name: item.name.clone(),
                type_name: item.type_n.clone(),
                is_array: item.is_array,
            }
        }
    }

    #[pyclass(name = "FidlMethod", frozen)]
    #[derive(Clone, Debug)]
    struct FidlMethod {
        #[pyo3(get)]
        pub annotations: Vec<FidlAnnotation>,
        #[pyo3(get)]
        pub name: String,
        #[pyo3(get)]
        pub input_parameters: Vec<FidlVariableDeclaration>,
        #[pyo3(get)]
        pub output_parameters: Vec<FidlVariableDeclaration>,
    }
    #[pymethods]
    impl FidlMethod {
        fn __str__(&self) -> String {
            format!("{:#?}", self)
        }
    }
    impl From<&Method> for FidlMethod {
        fn from(item: &Method) -> Self {
            FidlMethod {
                annotations: item
                    .annotations
                    .iter()
                    .map(|a| FidlAnnotation::from(a))
                    .collect(),
                name: item.name.clone(),
                input_parameters: item
                    .input_parameters
                    .iter()
                    .map(|a| FidlVariableDeclaration::from(a))
                    .collect(),
                output_parameters: item
                    .output_parameters
                    .iter()
                    .map(|a| FidlVariableDeclaration::from(a))
                    .collect(),
            }
        }
    }
    #[pyclass(name = "FidlEnumeration", frozen)]
    #[derive(Clone, Debug)]
    struct FidlEnumeration {
        #[pyo3(get)]
        pub annotations: Vec<FidlAnnotation>,
        #[pyo3(get)]
        pub name: String,
        #[pyo3(get)]
        pub values: Vec<FidlEnumValue>,
    }
    #[pymethods]
    impl FidlEnumeration {
        fn __str__(&self) -> String {
            format!("{:#?}", self)
        }
    }
    impl From<&Enumeration> for FidlEnumeration {
        fn from(item: &Enumeration) -> Self {
            FidlEnumeration {
                annotations: item
                    .annotations
                    .iter()
                    .map(|a| FidlAnnotation::from(a))
                    .collect(),
                name: item.name.clone(),
                values: item.values.iter().map(|a| FidlEnumValue::from(a)).collect(),
            }
        }
    }
    #[pyclass(name = "FidlEnumValue", frozen)]
    #[derive(Clone, Debug)]
    struct FidlEnumValue {
        #[pyo3(get)]
        pub annotations: Vec<FidlAnnotation>,
        #[pyo3(get)]
        pub name: String,
        #[pyo3(get)]
        pub value: Option<u64>,
    }
    #[pymethods]
    impl FidlEnumValue {
        fn __str__(&self) -> String {
            format!("{:#?}", self)
        }
    }
    impl From<&EnumValue> for FidlEnumValue {
        fn from(item: &EnumValue) -> Self {
            FidlEnumValue {
                annotations: item
                    .annotations
                    .iter()
                    .map(|a| FidlAnnotation::from(a))
                    .collect(),
                name: item.name.clone(),
                value: item.value,
            }
        }
    }

    #[pyclass(name = "FidlImportModel", frozen)]
    #[derive(Clone, Debug)]
    struct FidlImportModel {
        #[pyo3(get)]
        file_path: PathBuf,
    }
    #[pymethods]
    impl FidlImportModel {
        fn __str__(&self) -> String {
            format!("{:#?}", self)
        }
    }
    impl From<&ImportModel> for FidlImportModel {
        fn from(item: &ImportModel) -> Self {
            FidlImportModel {
                file_path: item.file_path.clone(),
            }
        }
    }

    #[pyclass(name = "FidlImportNamespace", frozen)]
    #[derive(Clone, Debug)]
    struct FidlImportNamespace {
        #[pyo3(get)]
        from_: PathBuf,
        #[pyo3(get)]
        imports: Vec<String>,
        #[pyo3(get)]
        wildcard: bool,
    }
    #[pymethods]
    impl FidlImportNamespace {
        fn __str__(&self) -> String {
            format!("{:#?}", self)
        }
    }
    impl From<&ImportNamespace> for FidlImportNamespace {
        fn from(item: &ImportNamespace) -> Self {
            FidlImportNamespace {
                imports: item.import.clone(),
                from_: item.from.clone(),
                wildcard: item.wildcard,
            }
        }
    }
    #[pyclass(name = "FidlPackage", frozen)]
    #[derive(Clone, Debug)]
    struct FidlPackage {
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
}
