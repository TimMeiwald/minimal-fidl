//! Python bindings.
//!
//! Every Python object is a **handle** — a shared pointer to the file plus a
//! `NodeId` — rather than a deep copy of its subtree. See `DESIGN.md` §11.
//!
//! The previous design gave each pyclass owned copies of its children with
//! `#[pyo3(get)]`, which meant `file.interfaces` cloned the entire subtree on
//! *every* attribute access. Here `file.interfaces` allocates N two-word handles.
//!
//! The pyclasses stay `frozen`: their own fields (`file`, `id`) never change, and
//! any mutation goes through the lock. Frozen classes are cheaper and need no GIL
//! for field access, so this is strictly better than unfreezing them.

use pyo3::prelude::*;

/// A Python module implemented in Rust. The name of this function must match
/// the `lib.name` setting in the `Cargo.toml`, else Python will not be able to
/// import the module.
#[pymodule]
mod franca_idl {
    use std::path::PathBuf;
    use std::sync::{Arc, RwLock};

    use minimal_fidl_collect::{
        Annotated, AstNode, FidlFile as AstFile, FileError, NodeId, NodeRef, Project,
    };
    use pyo3::create_exception;
    use pyo3::exceptions::{PyException, PyValueError};
    use pyo3::prelude::*;

    /// Shared ownership of one parsed file. Every handle holds a clone of this.
    type Shared = Arc<RwLock<AstFile>>;

    create_exception!(
        franca_idl,
        StaleNodeError,
        PyException,
        "The node this object referred to is no longer in the tree."
    );

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

    fn poisoned() -> PyErr {
        PyValueError::new_err("the file lock is poisoned")
    }

    fn stale(kind: &str, id: NodeId) -> PyErr {
        StaleNodeError::new_err(format!(
            "the {kind} this object referred to (id {}) is no longer in the tree",
            id.get()
        ))
    }

    fn write<R>(file: &Shared, f: impl FnOnce(&mut AstFile) -> R) -> PyResult<R> {
        match file.write() {
            Ok(mut guard) => Ok(f(&mut guard)),
            Err(_) => Err(poisoned()),
        }
    }

    fn read<R>(file: &Shared, f: impl FnOnce(&AstFile) -> R) -> PyResult<R> {
        match file.read() {
            Ok(guard) => Ok(f(&guard)),
            Err(_) => Err(poisoned()),
        }
    }

    #[pyfunction]
    fn _respond_42() -> u8 {
        42
    }

    #[pyfunction]
    fn load_fidl_project(dir: PathBuf) -> Result<Vec<FidlFile>, PyErr> {
        let project = Project::load(dir).map_err(FidlFileError::from)?;
        Ok(project
            .files
            .into_iter()
            .map(|file| FidlFile {
                inner: Arc::new(RwLock::new(file)),
            })
            .collect())
    }

    /// Declares a handle pyclass and the accessor that resolves it.
    ///
    /// `$variant` is the [`NodeRef`] variant the id must resolve to. Anything
    /// else means the node has been removed from the tree, which raises
    /// `StaleNodeError` rather than returning data from the wrong node.
    macro_rules! handle {
        ($py_name:literal, $rust:ident, $variant:ident, $ast:ty, $kind:literal, { $($body:tt)* }) => {
            #[pyclass(name = $py_name, frozen)]
            #[derive(Clone)]
            struct $rust {
                file: Shared,
                id: NodeId,
            }

            impl $rust {
                fn with<R>(&self, f: impl FnOnce(&$ast) -> R) -> PyResult<R> {
                    let guard = self.file.read().map_err(|_| poisoned())?;
                    match guard.get(self.id) {
                        Some(NodeRef::$variant(node)) => Ok(f(node)),
                        _ => Err(stale($kind, self.id)),
                    }
                }

                fn handle(file: &Shared, id: NodeId) -> Self {
                    Self {
                        file: file.clone(),
                        id,
                    }
                }
            }

            #[pymethods]
            impl $rust {
                /// The node's stable id within its file.
                #[getter]
                fn id(&self) -> u32 {
                    self.id.get()
                }

                /// False once the node has been removed from the tree.
                fn is_valid(&self) -> bool {
                    self.with(|_| ()).is_ok()
                }

                fn __repr__(&self) -> String {
                    format!("<{} id={}>", $py_name, self.id.get())
                }

                $($body)*
            }
        };
    }

    /// Handles for the annotations of any annotated node.
    fn annotation_handles(file: &Shared, node: &impl Annotated) -> Vec<FidlAnnotation> {
        node.annotations()
            .iter()
            .map(|a| FidlAnnotation::handle(file, a.id()))
            .collect()
    }

    // ---------------------------------------------------------------- file ---

    #[pyclass(name = "FidlFile", frozen)]
    #[derive(Clone)]
    struct FidlFile {
        inner: Shared,
    }

    #[pymethods]
    impl FidlFile {
        #[new]
        fn new(file_path: String) -> Result<Self, FidlFileError> {
            let file = AstFile::from_path(&file_path)?;
            Ok(Self {
                inner: Arc::new(RwLock::new(file)),
            })
        }

        #[staticmethod]
        fn new_from_string(file_string: String) -> Result<Self, FidlFileError> {
            let file = AstFile::from_source(&file_string)?;
            Ok(Self {
                inner: Arc::new(RwLock::new(file)),
            })
        }

        #[getter]
        fn file_path(&self) -> PyResult<Option<String>> {
            read(&self.inner, |f| {
                f.path.as_ref().map(|p| p.display().to_string())
            })
        }

        #[getter]
        fn package(&self) -> PyResult<Option<FidlPackage>> {
            read(&self.inner, |f| {
                f.package().map(|p| FidlPackage::handle(&self.inner, p.id()))
            })
        }

        #[getter]
        fn namespaces(&self) -> PyResult<Vec<FidlImportNamespace>> {
            read(&self.inner, |f| {
                f.namespaces()
                    .map(|n| FidlImportNamespace::handle(&self.inner, n.id()))
                    .collect()
            })
        }

        #[getter]
        fn import_models(&self) -> PyResult<Vec<FidlImportModel>> {
            read(&self.inner, |f| {
                f.import_models()
                    .map(|i| FidlImportModel::handle(&self.inner, i.id()))
                    .collect()
            })
        }

        #[getter]
        fn interfaces(&self) -> PyResult<Vec<FidlInterface>> {
            read(&self.inner, |f| {
                f.interfaces()
                    .map(|i| FidlInterface::handle(&self.inner, i.id()))
                    .collect()
            })
        }

        #[getter]
        fn type_collections(&self) -> PyResult<Vec<FidlTypeCollection>> {
            read(&self.inner, |f| {
                f.type_collections()
                    .map(|t| FidlTypeCollection::handle(&self.inner, t.id()))
                    .collect()
            })
        }

        /// The file rendered back to `.fidl` text.
        fn to_fidl(&self) -> PyResult<String> {
            read(&self.inner, |f| f.to_fidl())
        }

        /// Write the file back to where it was read from.
        fn save(&self) -> PyResult<()> {
            let guard = self.inner.read().map_err(|_| poisoned())?;
            guard
                .save()
                .map_err(|e| PyValueError::new_err(e.to_string()))
        }

        fn write_to(&self, path: PathBuf) -> PyResult<()> {
            let guard = self.inner.read().map_err(|_| poisoned())?;
            guard
                .write_to(path)
                .map_err(|e| PyValueError::new_err(e.to_string()))
        }

        /// Problems with the file, as human-readable strings. Empty means sound.
        fn validate(&self) -> PyResult<Vec<String>> {
            read(&self.inner, |f| {
                f.validate().iter().map(|d| d.to_string()).collect()
            })
        }

        fn __str__(&self) -> PyResult<String> {
            read(&self.inner, |f| format!("{f:#?}"))
        }

        fn __repr__(&self) -> PyResult<String> {
            read(&self.inner, |f| {
                format!(
                    "<FidlFile {} interfaces={}>",
                    f.path
                        .as_ref()
                        .map(|p| p.display().to_string())
                        .unwrap_or_else(|| "<string>".to_string()),
                    f.interfaces().count()
                )
            })
        }
    }

    // ----------------------------------------------------------- interface ---

    handle!(
        "FidlInterface",
        FidlInterface,
        Interface,
        minimal_fidl_collect::Interface,
        "interface",
        {

        #[getter]
        fn name(&self) -> PyResult<String> {
            self.with(|i| i.name.clone())
        }

        #[getter]
        fn version(&self) -> PyResult<Option<FidlVersion>> {
            self.with(|i| {
                i.version
                    .as_ref()
                    .map(|v| FidlVersion::handle(&self.file, v.id()))
            })
        }

        #[getter]
        fn annotations(&self) -> PyResult<Vec<FidlAnnotation>> {
            self.with(|i| annotation_handles(&self.file, i))
        }

        #[getter]
        fn methods(&self) -> PyResult<Vec<FidlMethod>> {
            self.with(|i| {
                i.methods()
                    .map(|m| FidlMethod::handle(&self.file, m.id()))
                    .collect()
            })
        }

        #[getter]
        fn attributes(&self) -> PyResult<Vec<FidlAttribute>> {
            self.with(|i| {
                i.attributes()
                    .map(|a| FidlAttribute::handle(&self.file, a.id()))
                    .collect()
            })
        }

        #[getter]
        fn structures(&self) -> PyResult<Vec<FidlStructure>> {
            self.with(|i| {
                i.structures()
                    .map(|s| FidlStructure::handle(&self.file, s.id()))
                    .collect()
            })
        }

        #[getter]
        fn typedefs(&self) -> PyResult<Vec<FidlTypeDef>> {
            self.with(|i| {
                i.typedefs()
                    .map(|t| FidlTypeDef::handle(&self.file, t.id()))
                    .collect()
            })
        }

        #[getter]
        fn enumerations(&self) -> PyResult<Vec<FidlEnumeration>> {
            self.with(|i| {
                i.enumerations()
                    .map(|e| FidlEnumeration::handle(&self.file, e.id()))
                    .collect()
            })
        }

        /// Remove a method by name. Returns whether one was there.
        ///
        /// Handles to the removed method become stale: reading one raises
        /// `StaleNodeError` rather than resolving to whatever node later
        /// occupies that slot.
        fn remove_method(&self, name: &str) -> PyResult<bool> {
            let own_name = self.with(|i| i.name.clone())?;
            write(&self.file, |file| match file.interface_mut(&own_name) {
                Some(iface) => iface.remove_method(name).is_some(),
                None => false,
            })
        }

        /// Remove an attribute by name. Returns whether one was there.
        fn remove_attribute(&self, name: &str) -> PyResult<bool> {
            let own_name = self.with(|i| i.name.clone())?;
            write(&self.file, |file| match file.interface_mut(&own_name) {
                Some(iface) => iface.remove_attribute(name).is_some(),
                None => false,
            })
        }

        fn __str__(&self) -> PyResult<String> {
            self.with(|i| format!("{i:#?}"))
        }
        }
    );

    // ----------------------------------------------------- type collection ---

    handle!(
        "FidlTypeCollection",
        FidlTypeCollection,
        TypeCollection,
        minimal_fidl_collect::TypeCollection,
        "type collection",
        {

        #[getter]
        fn name(&self) -> PyResult<String> {
            self.with(|t| t.name.clone())
        }

        #[getter]
        fn version(&self) -> PyResult<Option<FidlVersion>> {
            self.with(|t| {
                t.version
                    .as_ref()
                    .map(|v| FidlVersion::handle(&self.file, v.id()))
            })
        }

        #[getter]
        fn annotations(&self) -> PyResult<Vec<FidlAnnotation>> {
            self.with(|t| annotation_handles(&self.file, t))
        }

        #[getter]
        fn typedefs(&self) -> PyResult<Vec<FidlTypeDef>> {
            self.with(|t| {
                t.typedefs()
                    .map(|x| FidlTypeDef::handle(&self.file, x.id()))
                    .collect()
            })
        }

        #[getter]
        fn structures(&self) -> PyResult<Vec<FidlStructure>> {
            self.with(|t| {
                t.structures()
                    .map(|x| FidlStructure::handle(&self.file, x.id()))
                    .collect()
            })
        }

        #[getter]
        fn enumerations(&self) -> PyResult<Vec<FidlEnumeration>> {
            self.with(|t| {
                t.enumerations()
                    .map(|x| FidlEnumeration::handle(&self.file, x.id()))
                    .collect()
            })
        }

        fn __str__(&self) -> PyResult<String> {
            self.with(|t| format!("{t:#?}"))
        }
        }
    );

    // -------------------------------------------------------------- method ---

    handle!(
        "FidlMethod",
        FidlMethod,
        Method,
        minimal_fidl_collect::Method,
        "method",
        {

        #[getter]
        fn name(&self) -> PyResult<String> {
            self.with(|m| m.name.clone())
        }

        #[getter]
        fn annotations(&self) -> PyResult<Vec<FidlAnnotation>> {
            self.with(|m| annotation_handles(&self.file, m))
        }

        #[getter]
        fn input_parameters(&self) -> PyResult<Vec<FidlVariableDeclaration>> {
            self.with(|m| {
                m.input_parameters()
                    .map(|p| FidlVariableDeclaration::handle(&self.file, p.id()))
                    .collect()
            })
        }

        #[getter]
        fn output_parameters(&self) -> PyResult<Vec<FidlVariableDeclaration>> {
            self.with(|m| {
                m.output_parameters()
                    .map(|p| FidlVariableDeclaration::handle(&self.file, p.id()))
                    .collect()
            })
        }

        fn __str__(&self) -> PyResult<String> {
            self.with(|m| format!("{m:#?}"))
        }
        }
    );

    // ----------------------------------------------------------- structure ---

    handle!(
        "FidlStructure",
        FidlStructure,
        Structure,
        minimal_fidl_collect::Structure,
        "struct",
        {

        #[getter]
        fn name(&self) -> PyResult<String> {
            self.with(|s| s.name.clone())
        }

        #[getter]
        fn annotations(&self) -> PyResult<Vec<FidlAnnotation>> {
            self.with(|s| annotation_handles(&self.file, s))
        }

        /// Kept as `contents` for compatibility; `fields` is the clearer name.
        #[getter]
        fn contents(&self) -> PyResult<Vec<FidlVariableDeclaration>> {
            self.fields()
        }

        #[getter]
        fn fields(&self) -> PyResult<Vec<FidlVariableDeclaration>> {
            self.with(|s| {
                s.fields()
                    .map(|f| FidlVariableDeclaration::handle(&self.file, f.id()))
                    .collect()
            })
        }

        fn __str__(&self) -> PyResult<String> {
            self.with(|s| format!("{s:#?}"))
        }
        }
    );

    // --------------------------------------------------------- enumeration ---

    handle!(
        "FidlEnumeration",
        FidlEnumeration,
        Enumeration,
        minimal_fidl_collect::Enumeration,
        "enumeration",
        {

        #[getter]
        fn name(&self) -> PyResult<String> {
            self.with(|e| e.name.clone())
        }

        #[getter]
        fn annotations(&self) -> PyResult<Vec<FidlAnnotation>> {
            self.with(|e| annotation_handles(&self.file, e))
        }

        #[getter]
        fn values(&self) -> PyResult<Vec<FidlEnumValue>> {
            self.with(|e| {
                e.values()
                    .map(|v| FidlEnumValue::handle(&self.file, v.id()))
                    .collect()
            })
        }

        fn __str__(&self) -> PyResult<String> {
            self.with(|e| format!("{e:#?}"))
        }
        }
    );

    handle!(
        "FidlEnumValue",
        FidlEnumValue,
        EnumValue,
        minimal_fidl_collect::EnumValue,
        "enum value",
        {

        #[getter]
        fn name(&self) -> PyResult<String> {
            self.with(|v| v.name.clone())
        }

        #[getter]
        fn value(&self) -> PyResult<Option<u64>> {
            self.with(|v| v.value)
        }

        #[getter]
        fn annotations(&self) -> PyResult<Vec<FidlAnnotation>> {
            self.with(|v| annotation_handles(&self.file, v))
        }

        fn __str__(&self) -> PyResult<String> {
            self.with(|v| format!("{v:#?}"))
        }
        }
    );

    // -------------------------------------------------------------- leaves ---

    handle!(
        "FidlAttribute",
        FidlAttribute,
        Attribute,
        minimal_fidl_collect::Attribute,
        "attribute",
        {

        #[getter]
        fn name(&self) -> PyResult<String> {
            self.with(|a| a.name.clone())
        }

        #[getter]
        fn type_name(&self) -> PyResult<String> {
            self.with(|a| a.type_n.clone())
        }

        #[getter]
        fn annotations(&self) -> PyResult<Vec<FidlAnnotation>> {
            self.with(|a| annotation_handles(&self.file, a))
        }

        fn __str__(&self) -> PyResult<String> {
            self.with(|a| format!("{a:#?}"))
        }
        }
    );

    handle!(
        "FidlTypeDef",
        FidlTypeDef,
        TypeDef,
        minimal_fidl_collect::TypeDef,
        "typedef",
        {

        #[getter]
        fn name(&self) -> PyResult<String> {
            self.with(|t| t.name.clone())
        }

        #[getter]
        fn type_name(&self) -> PyResult<String> {
            self.with(|t| t.type_n.clone())
        }

        #[getter]
        fn is_array(&self) -> PyResult<bool> {
            self.with(|t| t.is_array)
        }

        #[getter]
        fn annotations(&self) -> PyResult<Vec<FidlAnnotation>> {
            self.with(|t| annotation_handles(&self.file, t))
        }

        fn __str__(&self) -> PyResult<String> {
            self.with(|t| format!("{t:#?}"))
        }
        }
    );

    handle!(
        "FidlVariableDeclaration",
        FidlVariableDeclaration,
        VariableDeclaration,
        minimal_fidl_collect::VariableDeclaration,
        "variable declaration",
        {

        #[getter]
        fn name(&self) -> PyResult<String> {
            self.with(|v| v.name.clone())
        }

        #[getter]
        fn type_name(&self) -> PyResult<String> {
            self.with(|v| v.type_n.clone())
        }

        #[getter]
        fn is_array(&self) -> PyResult<bool> {
            self.with(|v| v.is_array)
        }

        #[getter]
        fn annotations(&self) -> PyResult<Vec<FidlAnnotation>> {
            self.with(|v| annotation_handles(&self.file, v))
        }

        fn __str__(&self) -> PyResult<String> {
            self.with(|v| format!("{v:#?}"))
        }
        }
    );

    handle!(
        "FidlVersion",
        FidlVersion,
        Version,
        minimal_fidl_collect::Version,
        "version",
        {

        #[getter]
        fn major(&self) -> PyResult<Option<u32>> {
            self.with(|v| v.major)
        }

        #[getter]
        fn minor(&self) -> PyResult<Option<u32>> {
            self.with(|v| v.minor)
        }

        fn __str__(&self) -> PyResult<String> {
            self.with(|v| format!("{v:#?}"))
        }
        }
    );

    handle!(
        "FidlAnnotation",
        FidlAnnotation,
        Annotation,
        minimal_fidl_collect::Annotation,
        "annotation",
        {

        #[getter]
        fn name(&self) -> PyResult<String> {
            self.with(|a| a.name.clone())
        }

        #[getter]
        fn contents(&self) -> PyResult<String> {
            self.with(|a| a.contents.clone())
        }

        fn __str__(&self) -> PyResult<String> {
            self.with(|a| format!("{a:#?}"))
        }
        }
    );

    handle!(
        "FidlPackage",
        FidlPackage,
        Package,
        minimal_fidl_collect::Package,
        "package",
        {

        #[getter]
        fn path(&self) -> PyResult<Vec<String>> {
            self.with(|p| p.path.clone())
        }

        fn __str__(&self) -> PyResult<String> {
            self.with(|p| format!("{p:#?}"))
        }
        }
    );

    handle!(
        "FidlImportModel",
        FidlImportModel,
        ImportModel,
        minimal_fidl_collect::ImportModel,
        "import model",
        {

        #[getter]
        fn file_path(&self) -> PyResult<PathBuf> {
            self.with(|i| i.file_path.clone())
        }

        fn __str__(&self) -> PyResult<String> {
            self.with(|i| format!("{i:#?}"))
        }
        }
    );

    handle!(
        "FidlImportNamespace",
        FidlImportNamespace,
        ImportNamespace,
        minimal_fidl_collect::ImportNamespace,
        "import namespace",
        {

        #[getter]
        fn from_(&self) -> PyResult<PathBuf> {
            self.with(|n| n.from.clone())
        }

        #[getter]
        fn imports(&self) -> PyResult<Vec<String>> {
            self.with(|n| n.import.clone())
        }

        #[getter]
        fn wildcard(&self) -> PyResult<bool> {
            self.with(|n| n.wildcard)
        }

        fn __str__(&self) -> PyResult<String> {
            self.with(|n| format!("{n:#?}"))
        }
        }
    );

    // Classes produced by `handle!` are not visible to the `#[pymodule]` macro —
    // it processes the module body before the macro expands — so they have to be
    // registered by hand. Without this they still work as return values but
    // cannot be imported or used with `isinstance`.
    #[pymodule_init]
    fn init(module: &Bound<'_, PyModule>) -> PyResult<()> {
        module.add("StaleNodeError", module.py().get_type::<StaleNodeError>())?;
        module.add_class::<FidlInterface>()?;
        module.add_class::<FidlTypeCollection>()?;
        module.add_class::<FidlMethod>()?;
        module.add_class::<FidlStructure>()?;
        module.add_class::<FidlEnumeration>()?;
        module.add_class::<FidlEnumValue>()?;
        module.add_class::<FidlAttribute>()?;
        module.add_class::<FidlTypeDef>()?;
        module.add_class::<FidlVariableDeclaration>()?;
        module.add_class::<FidlVersion>()?;
        module.add_class::<FidlAnnotation>()?;
        module.add_class::<FidlPackage>()?;
        module.add_class::<FidlImportModel>()?;
        module.add_class::<FidlImportNamespace>()?;
        Ok(())
    }
}
