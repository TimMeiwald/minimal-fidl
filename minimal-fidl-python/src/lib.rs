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
//!
//! Two kinds of class live here, and the distinction is the whole API:
//!
//! - `Fidl*` — a **handle** on a node that is already in a tree. Cannot be
//!   constructed from Python; you get one by reading or by inserting.
//! - `New*` — a **description** of a node to create. Plain constructible data,
//!   belonging to no file. Insertion consumes one and returns the handle.
//!
//! Mutation always resolves the handle's own id through `get_mut`, never a name
//! lookup: duplicate names are a `validate()` diagnostic rather than a parse
//! error, so `interface_mut("A")` can hand back a different node than the one the
//! handle names.

use pyo3::prelude::*;

/// A Python module implemented in Rust. The name of this function must match
/// the `lib.name` setting in the `Cargo.toml`, else Python will not be able to
/// import the module.
#[pymodule]
mod franca_idl {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use std::path::PathBuf;
    use std::sync::{Arc, RwLock};

    use minimal_fidl_collect::{
        diff, Annotated, Annotation, AstNode, Attribute, Change, Comment, CommentKind, DiffOptions,
        EnumMember, EnumValue, Enumeration, FidlFile as AstFile, FileError, FileMember, ImportModel,
        ImportNamespace, Interface, InterfaceMember, Method, Mode, NodeId, NodePath, NodeRef,
        NodeRefMut, Package, ParamList, ParamMember, PathSegment, Project, StructMember, Structure,
        TypeCollection, TypeCollectionMember, TypeDef, VariableDeclaration, Version,
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

    /// A rejected mutation — a duplicate name, usually.
    fn rejected(error: FileError) -> PyErr {
        PyValueError::new_err(error.to_string())
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

    /// The id of any node handle, or a bare `int`.
    ///
    /// Duck-typed on purpose: `path_of` and friends take a node of any kind, and
    /// enumerating seventeen handle classes in a `FromPyObject` enum would add
    /// nothing but maintenance.
    fn node_id_of(node: &Bound<'_, PyAny>) -> PyResult<NodeId> {
        if let Ok(raw) = node.extract::<u32>() {
            return Ok(NodeId::from_raw(raw));
        }
        Ok(NodeId::from_raw(node.getattr("id")?.extract::<u32>()?))
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

    // ============================================================ New* ===
    //
    // A `New*` describes a node to build. It is not a handle: it belongs to no
    // file and has no id until it is inserted. Insertion clones the description,
    // so one `New*` can be inserted more than once.
    //
    // Keyword arguments were rejected for these (`DESIGN.md` §11): a nested
    // structure built from kwargs degenerates into tuples of tuples, and neither
    // an editor nor a type checker can say what belongs where.

    /// A dotted name, written either as `"org.example"` or as `["org", "example"]`.
    ///
    /// `Dotted` is tried first: PyO3 will not extract a `str` into a `Vec<String>`,
    /// but leading with the string keeps the intent obvious.
    #[derive(FromPyObject, Clone)]
    enum DottedName {
        Dotted(String),
        Segments(Vec<String>),
    }

    impl DottedName {
        fn segments(self) -> Vec<String> {
            match self {
                DottedName::Dotted(text) => text.split('.').map(str::to_string).collect(),
                DottedName::Segments(segments) => segments,
            }
        }
    }

    fn annotations_of(annotations: Option<Vec<NewAnnotation>>) -> Vec<Annotation> {
        annotations
            .unwrap_or_default()
            .iter()
            .map(NewAnnotation::build)
            .collect()
    }

    fn comments_of(comments: Option<Vec<NewComment>>) -> Vec<Comment> {
        comments
            .unwrap_or_default()
            .iter()
            .map(NewComment::build)
            .collect()
    }

    /// `<** @name: contents **>`
    #[pyclass(name = "NewAnnotation")]
    #[derive(Clone)]
    struct NewAnnotation {
        inner: Annotation,
    }

    #[pymethods]
    impl NewAnnotation {
        #[new]
        #[pyo3(signature = (name, contents=String::new()))]
        fn new(name: String, contents: String) -> Self {
            Self {
                inner: Annotation::create(name, contents),
            }
        }

        #[getter]
        fn name(&self) -> String {
            self.inner.name.clone()
        }

        #[getter]
        fn contents(&self) -> String {
            self.inner.contents.clone()
        }

        fn __repr__(&self) -> String {
            format!("NewAnnotation({:?}, {:?})", self.inner.name, self.inner.contents)
        }
    }

    impl NewAnnotation {
        fn build(&self) -> Annotation {
            self.inner.clone()
        }
    }

    /// `// text`, or `/* text */` with `block=True`.
    ///
    /// `text` is the content only; the delimiters are added when printing.
    #[pyclass(name = "NewComment")]
    #[derive(Clone)]
    struct NewComment {
        inner: Comment,
    }

    #[pymethods]
    impl NewComment {
        #[new]
        #[pyo3(signature = (text, block=false))]
        fn new(text: String, block: bool) -> Self {
            Self {
                inner: if block {
                    Comment::block(text)
                } else {
                    Comment::line(text)
                },
            }
        }

        #[getter]
        fn text(&self) -> String {
            self.inner.text.clone()
        }

        #[getter]
        fn is_block(&self) -> bool {
            self.inner.kind == CommentKind::Block
        }

        fn __repr__(&self) -> String {
            format!("NewComment({:?}, block={})", self.inner.text, self.is_block())
        }
    }

    impl NewComment {
        fn build(&self) -> Comment {
            self.inner.clone()
        }
    }

    /// `version {major M minor N}`
    #[pyclass(name = "NewVersion")]
    #[derive(Clone)]
    struct NewVersion {
        inner: Version,
    }

    #[pymethods]
    impl NewVersion {
        #[new]
        fn new(major: u32, minor: u32) -> Self {
            Self {
                inner: Version::create(major, minor),
            }
        }

        #[getter]
        fn major(&self) -> Option<u32> {
            self.inner.major
        }

        #[getter]
        fn minor(&self) -> Option<u32> {
            self.inner.minor
        }

        fn __repr__(&self) -> String {
            format!("NewVersion({:?}, {:?})", self.inner.major, self.inner.minor)
        }
    }

    impl NewVersion {
        fn build(&self) -> Version {
            self.inner.clone()
        }
    }

    /// A `Type name` pair.
    ///
    /// Used for method parameters *and* struct fields — they are the same node in
    /// the grammar, so there is no separate `NewField`.
    #[pyclass(name = "NewParameter")]
    #[derive(Clone)]
    struct NewParameter {
        inner: VariableDeclaration,
    }

    #[pymethods]
    impl NewParameter {
        #[new]
        #[pyo3(signature = (name, type_name, is_array=false, annotations=None, leading_comments=None))]
        fn new(
            name: String,
            type_name: String,
            is_array: bool,
            annotations: Option<Vec<NewAnnotation>>,
            leading_comments: Option<Vec<NewComment>>,
        ) -> Self {
            let mut inner = if is_array {
                VariableDeclaration::array(name, type_name)
            } else {
                VariableDeclaration::create(name, type_name)
            };
            inner.annotations = annotations_of(annotations);
            inner.meta.leading_comments = comments_of(leading_comments);
            Self { inner }
        }

        #[getter]
        fn name(&self) -> String {
            self.inner.name.clone()
        }

        #[getter]
        fn type_name(&self) -> String {
            self.inner.type_n.clone()
        }

        #[getter]
        fn is_array(&self) -> bool {
            self.inner.is_array
        }

        fn __repr__(&self) -> String {
            format!(
                "NewParameter({:?}, {:?}, is_array={})",
                self.inner.name, self.inner.type_n, self.inner.is_array
            )
        }
    }

    impl NewParameter {
        fn build(&self) -> VariableDeclaration {
            self.inner.clone()
        }
    }

    /// `attribute Type name`
    #[pyclass(name = "NewAttribute")]
    #[derive(Clone)]
    struct NewAttribute {
        inner: Attribute,
    }

    #[pymethods]
    impl NewAttribute {
        #[new]
        #[pyo3(signature = (name, type_name, annotations=None, leading_comments=None))]
        fn new(
            name: String,
            type_name: String,
            annotations: Option<Vec<NewAnnotation>>,
            leading_comments: Option<Vec<NewComment>>,
        ) -> Self {
            let mut inner = Attribute::create(name, type_name);
            inner.annotations = annotations_of(annotations);
            inner.meta.leading_comments = comments_of(leading_comments);
            Self { inner }
        }

        #[getter]
        fn name(&self) -> String {
            self.inner.name.clone()
        }

        #[getter]
        fn type_name(&self) -> String {
            self.inner.type_n.clone()
        }

        fn __repr__(&self) -> String {
            format!("NewAttribute({:?}, {:?})", self.inner.name, self.inner.type_n)
        }
    }

    impl NewAttribute {
        fn build(&self) -> Attribute {
            self.inner.clone()
        }
    }

    /// `typedef name is Type`
    #[pyclass(name = "NewTypeDef")]
    #[derive(Clone)]
    struct NewTypeDef {
        inner: TypeDef,
    }

    #[pymethods]
    impl NewTypeDef {
        #[new]
        #[pyo3(signature = (name, type_name, is_array=false, annotations=None, leading_comments=None))]
        fn new(
            name: String,
            type_name: String,
            is_array: bool,
            annotations: Option<Vec<NewAnnotation>>,
            leading_comments: Option<Vec<NewComment>>,
        ) -> Self {
            let mut inner = if is_array {
                TypeDef::array(name, type_name)
            } else {
                TypeDef::create(name, type_name)
            };
            inner.annotations = annotations_of(annotations);
            inner.meta.leading_comments = comments_of(leading_comments);
            Self { inner }
        }

        #[getter]
        fn name(&self) -> String {
            self.inner.name.clone()
        }

        #[getter]
        fn type_name(&self) -> String {
            self.inner.type_n.clone()
        }

        #[getter]
        fn is_array(&self) -> bool {
            self.inner.is_array
        }

        fn __repr__(&self) -> String {
            format!("NewTypeDef({:?}, {:?})", self.inner.name, self.inner.type_n)
        }
    }

    impl NewTypeDef {
        fn build(&self) -> TypeDef {
            self.inner.clone()
        }
    }

    /// One `NAME` or `NAME = 3` inside an enumeration.
    #[pyclass(name = "NewEnumValue")]
    #[derive(Clone)]
    struct NewEnumValue {
        inner: EnumValue,
    }

    #[pymethods]
    impl NewEnumValue {
        #[new]
        #[pyo3(signature = (name, value=None, annotations=None, leading_comments=None))]
        fn new(
            name: String,
            value: Option<u64>,
            annotations: Option<Vec<NewAnnotation>>,
            leading_comments: Option<Vec<NewComment>>,
        ) -> Self {
            let mut inner = match value {
                Some(value) => EnumValue::with_value(name, value),
                None => EnumValue::create(name),
            };
            inner.annotations = annotations_of(annotations);
            inner.meta.leading_comments = comments_of(leading_comments);
            Self { inner }
        }

        #[getter]
        fn name(&self) -> String {
            self.inner.name.clone()
        }

        #[getter]
        fn value(&self) -> Option<u64> {
            self.inner.value
        }

        fn __repr__(&self) -> String {
            format!("NewEnumValue({:?}, {:?})", self.inner.name, self.inner.value)
        }
    }

    impl NewEnumValue {
        fn build(&self) -> EnumValue {
            self.inner.clone()
        }
    }

    /// An ordered child of an enumeration.
    #[derive(FromPyObject, Clone)]
    enum NewEnumMember {
        Value(NewEnumValue),
        Comment(NewComment),
    }

    impl NewEnumMember {
        fn node_kind(&self) -> NodeKind {
            match self {
                NewEnumMember::Value(_) => NodeKind::EnumValue,
                NewEnumMember::Comment(_) => NodeKind::Comment,
            }
        }

        fn build(&self) -> EnumMember {
            match self {
                NewEnumMember::Value(v) => EnumMember::Value(v.build()),
                NewEnumMember::Comment(c) => EnumMember::Comment(c.build()),
            }
        }
    }

    /// `enumeration name { ... }`
    #[pyclass(name = "NewEnumeration")]
    #[derive(Clone)]
    struct NewEnumeration {
        inner: Enumeration,
    }

    #[pymethods]
    impl NewEnumeration {
        #[new]
        #[pyo3(signature = (name, members=None, annotations=None, leading_comments=None))]
        fn new(
            name: String,
            members: Option<Vec<NewEnumMember>>,
            annotations: Option<Vec<NewAnnotation>>,
            leading_comments: Option<Vec<NewComment>>,
        ) -> Self {
            let mut inner = Enumeration::builder(name).build();
            inner.annotations = annotations_of(annotations);
            inner.meta.leading_comments = comments_of(leading_comments);
            inner.members = members
                .unwrap_or_default()
                .iter()
                .map(NewEnumMember::build)
                .collect();
            Self { inner }
        }

        #[getter]
        fn name(&self) -> String {
            self.inner.name.clone()
        }

        fn __repr__(&self) -> String {
            format!(
                "NewEnumeration({:?}, {} members)",
                self.inner.name,
                self.inner.members.len()
            )
        }
    }

    impl NewEnumeration {
        fn build(&self) -> Enumeration {
            self.inner.clone()
        }
    }

    /// An ordered child of a struct.
    #[derive(FromPyObject, Clone)]
    enum NewStructMember {
        Field(NewParameter),
        Comment(NewComment),
    }

    impl NewStructMember {
        fn node_kind(&self) -> NodeKind {
            match self {
                NewStructMember::Field(_) => NodeKind::VariableDeclaration,
                NewStructMember::Comment(_) => NodeKind::Comment,
            }
        }

        fn build(&self) -> StructMember {
            match self {
                NewStructMember::Field(f) => StructMember::Field(f.build()),
                NewStructMember::Comment(c) => StructMember::Comment(c.build()),
            }
        }
    }

    /// `struct name { ... }`
    #[pyclass(name = "NewStructure")]
    #[derive(Clone)]
    struct NewStructure {
        inner: Structure,
    }

    #[pymethods]
    impl NewStructure {
        #[new]
        #[pyo3(signature = (name, members=None, annotations=None, leading_comments=None))]
        fn new(
            name: String,
            members: Option<Vec<NewStructMember>>,
            annotations: Option<Vec<NewAnnotation>>,
            leading_comments: Option<Vec<NewComment>>,
        ) -> Self {
            let mut inner = Structure::builder(name).build();
            inner.annotations = annotations_of(annotations);
            inner.meta.leading_comments = comments_of(leading_comments);
            inner.members = members
                .unwrap_or_default()
                .iter()
                .map(NewStructMember::build)
                .collect();
            Self { inner }
        }

        #[getter]
        fn name(&self) -> String {
            self.inner.name.clone()
        }

        fn __repr__(&self) -> String {
            format!(
                "NewStructure({:?}, {} members)",
                self.inner.name,
                self.inner.members.len()
            )
        }
    }

    impl NewStructure {
        fn build(&self) -> Structure {
            self.inner.clone()
        }
    }

    /// An ordered child of an `in { }` / `out { }` list.
    #[derive(FromPyObject, Clone)]
    enum NewParamMember {
        Param(NewParameter),
        Comment(NewComment),
    }

    impl NewParamMember {
        fn node_kind(&self) -> NodeKind {
            match self {
                NewParamMember::Param(_) => NodeKind::VariableDeclaration,
                NewParamMember::Comment(_) => NodeKind::Comment,
            }
        }

        fn build(&self) -> ParamMember {
            match self {
                NewParamMember::Param(p) => ParamMember::Param(p.build()),
                NewParamMember::Comment(c) => ParamMember::Comment(c.build()),
            }
        }
    }

    fn param_list(members: Option<Vec<NewParamMember>>) -> ParamList {
        let mut list = ParamList::default();
        list.meta.dirty = true;
        list.members = members
            .unwrap_or_default()
            .iter()
            .map(NewParamMember::build)
            .collect();
        list
    }

    /// `method name { in { ... } out { ... } }`
    #[pyclass(name = "NewMethod")]
    #[derive(Clone)]
    struct NewMethod {
        inner: Method,
    }

    #[pymethods]
    impl NewMethod {
        #[new]
        #[pyo3(signature = (name, inputs=None, outputs=None, annotations=None, leading_comments=None))]
        fn new(
            name: String,
            inputs: Option<Vec<NewParamMember>>,
            outputs: Option<Vec<NewParamMember>>,
            annotations: Option<Vec<NewAnnotation>>,
            leading_comments: Option<Vec<NewComment>>,
        ) -> Self {
            let mut inner = Method::builder(name).build();
            inner.annotations = annotations_of(annotations);
            inner.meta.leading_comments = comments_of(leading_comments);
            inner.inputs = param_list(inputs);
            inner.outputs = param_list(outputs);
            Self { inner }
        }

        #[getter]
        fn name(&self) -> String {
            self.inner.name.clone()
        }

        fn __repr__(&self) -> String {
            format!(
                "NewMethod({:?}, {} in, {} out)",
                self.inner.name,
                self.inner.inputs.members.len(),
                self.inner.outputs.members.len()
            )
        }
    }

    impl NewMethod {
        fn build(&self) -> Method {
            self.inner.clone()
        }
    }

    /// An ordered child of an interface.
    #[derive(FromPyObject, Clone)]
    enum NewInterfaceMember {
        Method(NewMethod),
        Attribute(NewAttribute),
        Structure(NewStructure),
        Enumeration(NewEnumeration),
        TypeDef(NewTypeDef),
        Comment(NewComment),
    }

    impl NewInterfaceMember {
        fn node_kind(&self) -> NodeKind {
            match self {
                NewInterfaceMember::Method(_) => NodeKind::Method,
                NewInterfaceMember::Attribute(_) => NodeKind::Attribute,
                NewInterfaceMember::Structure(_) => NodeKind::Structure,
                NewInterfaceMember::Enumeration(_) => NodeKind::Enumeration,
                NewInterfaceMember::TypeDef(_) => NodeKind::TypeDef,
                NewInterfaceMember::Comment(_) => NodeKind::Comment,
            }
        }

        fn build(&self) -> InterfaceMember {
            match self {
                NewInterfaceMember::Method(m) => InterfaceMember::Method(m.build()),
                NewInterfaceMember::Attribute(a) => InterfaceMember::Attribute(a.build()),
                NewInterfaceMember::Structure(s) => InterfaceMember::Structure(s.build()),
                NewInterfaceMember::Enumeration(e) => InterfaceMember::Enumeration(e.build()),
                NewInterfaceMember::TypeDef(t) => InterfaceMember::TypeDef(t.build()),
                NewInterfaceMember::Comment(c) => InterfaceMember::Comment(c.build()),
            }
        }
    }

    /// `interface name { ... }`
    ///
    /// `members` is one ordered list rather than a list per kind, because member
    /// order is intrinsic to this tree (`DESIGN.md` §1.1) and per-kind lists would
    /// throw away the interleaving at construction time.
    #[pyclass(name = "NewInterface")]
    #[derive(Clone)]
    struct NewInterface {
        inner: Interface,
    }

    #[pymethods]
    impl NewInterface {
        #[new]
        #[pyo3(signature = (name, members=None, version=None, annotations=None, leading_comments=None))]
        fn new(
            name: String,
            members: Option<Vec<NewInterfaceMember>>,
            version: Option<NewVersion>,
            annotations: Option<Vec<NewAnnotation>>,
            leading_comments: Option<Vec<NewComment>>,
        ) -> Self {
            let mut inner = Interface::builder(name).build();
            inner.version = version.map(|v| v.build());
            inner.annotations = annotations_of(annotations);
            inner.meta.leading_comments = comments_of(leading_comments);
            inner.members = members
                .unwrap_or_default()
                .iter()
                .map(NewInterfaceMember::build)
                .collect();
            Self { inner }
        }

        #[getter]
        fn name(&self) -> String {
            self.inner.name.clone()
        }

        fn __repr__(&self) -> String {
            format!(
                "NewInterface({:?}, {} members)",
                self.inner.name,
                self.inner.members.len()
            )
        }
    }

    impl NewInterface {
        fn build(&self) -> Interface {
            self.inner.clone()
        }
    }

    /// An ordered child of a type collection.
    #[derive(FromPyObject, Clone)]
    enum NewTypeCollectionMember {
        TypeDef(NewTypeDef),
        Structure(NewStructure),
        Enumeration(NewEnumeration),
        Comment(NewComment),
    }

    impl NewTypeCollectionMember {
        fn node_kind(&self) -> NodeKind {
            match self {
                NewTypeCollectionMember::TypeDef(_) => NodeKind::TypeDef,
                NewTypeCollectionMember::Structure(_) => NodeKind::Structure,
                NewTypeCollectionMember::Enumeration(_) => NodeKind::Enumeration,
                NewTypeCollectionMember::Comment(_) => NodeKind::Comment,
            }
        }

        fn build(&self) -> TypeCollectionMember {
            match self {
                NewTypeCollectionMember::TypeDef(t) => TypeCollectionMember::TypeDef(t.build()),
                NewTypeCollectionMember::Structure(s) => TypeCollectionMember::Structure(s.build()),
                NewTypeCollectionMember::Enumeration(e) => {
                    TypeCollectionMember::Enumeration(e.build())
                }
                NewTypeCollectionMember::Comment(c) => TypeCollectionMember::Comment(c.build()),
            }
        }
    }

    /// `typeCollection name { ... }`
    ///
    /// The name may be empty: the grammar allows an anonymous collection, though
    /// `validate()` warns that nothing can refer to it.
    #[pyclass(name = "NewTypeCollection")]
    #[derive(Clone)]
    struct NewTypeCollection {
        inner: TypeCollection,
    }

    #[pymethods]
    impl NewTypeCollection {
        #[new]
        #[pyo3(signature = (name=String::new(), members=None, version=None, annotations=None, leading_comments=None))]
        fn new(
            name: String,
            members: Option<Vec<NewTypeCollectionMember>>,
            version: Option<NewVersion>,
            annotations: Option<Vec<NewAnnotation>>,
            leading_comments: Option<Vec<NewComment>>,
        ) -> Self {
            let mut inner = TypeCollection::builder(name).build();
            inner.version = version.map(|v| v.build());
            inner.annotations = annotations_of(annotations);
            inner.meta.leading_comments = comments_of(leading_comments);
            inner.members = members
                .unwrap_or_default()
                .iter()
                .map(NewTypeCollectionMember::build)
                .collect();
            Self { inner }
        }

        #[getter]
        fn name(&self) -> String {
            self.inner.name.clone()
        }

        fn __repr__(&self) -> String {
            format!(
                "NewTypeCollection({:?}, {} members)",
                self.inner.name,
                self.inner.members.len()
            )
        }
    }

    impl NewTypeCollection {
        fn build(&self) -> TypeCollection {
            self.inner.clone()
        }
    }

    /// `package org.example`
    #[pyclass(name = "NewPackage")]
    #[derive(Clone)]
    struct NewPackage {
        inner: Package,
    }

    #[pymethods]
    impl NewPackage {
        #[new]
        fn new(path: DottedName) -> Self {
            Self {
                inner: Package::create(path.segments()),
            }
        }

        #[getter]
        fn path(&self) -> Vec<String> {
            self.inner.path.clone()
        }

        fn __repr__(&self) -> String {
            format!("NewPackage({:?})", self.inner.path.join("."))
        }
    }

    impl NewPackage {
        fn build(&self) -> Package {
            self.inner.clone()
        }
    }

    /// `import model "path"`
    #[pyclass(name = "NewImportModel")]
    #[derive(Clone)]
    struct NewImportModel {
        inner: ImportModel,
    }

    #[pymethods]
    impl NewImportModel {
        #[new]
        fn new(file_path: PathBuf) -> Self {
            Self {
                inner: ImportModel::create(file_path),
            }
        }

        #[getter]
        fn file_path(&self) -> PathBuf {
            self.inner.file_path.clone()
        }

        fn __repr__(&self) -> String {
            format!("NewImportModel({:?})", self.inner.file_path)
        }
    }

    impl NewImportModel {
        fn build(&self) -> ImportModel {
            self.inner.clone()
        }
    }

    /// `import org.example.* from "path"`
    ///
    /// Always a wildcard import: the grammar requires the `.*`, so anything else
    /// would print text that cannot be read back.
    #[pyclass(name = "NewImportNamespace")]
    #[derive(Clone)]
    struct NewImportNamespace {
        inner: ImportNamespace,
    }

    #[pymethods]
    impl NewImportNamespace {
        #[new]
        fn new(namespace: DottedName, from_: PathBuf) -> Self {
            Self {
                inner: ImportNamespace::create(namespace.segments(), from_),
            }
        }

        #[getter]
        fn namespace(&self) -> Vec<String> {
            self.inner.import.clone()
        }

        #[getter]
        fn from_(&self) -> PathBuf {
            self.inner.from.clone()
        }

        fn __repr__(&self) -> String {
            format!(
                "NewImportNamespace({:?}, {:?})",
                self.inner.import.join("."),
                self.inner.from
            )
        }
    }

    impl NewImportNamespace {
        fn build(&self) -> ImportNamespace {
            self.inner.clone()
        }
    }

    // ======================================================= addressing ===

    /// One step of a [`FidlNodePath`]: a name where the node has one, an index
    /// where it does not.
    #[pyclass(name = "FidlPathSegment", frozen)]
    #[derive(Clone)]
    struct FidlPathSegment {
        #[pyo3(get)]
        kind: String,
        /// `None` for a node addressed by position.
        #[pyo3(get)]
        name: Option<String>,
        /// `None` for a node addressed by name.
        #[pyo3(get)]
        index: Option<usize>,
    }

    #[pymethods]
    impl FidlPathSegment {
        fn __repr__(&self) -> String {
            match (&self.name, self.index) {
                (Some(name), _) => format!("{}({name})", self.kind),
                (None, Some(index)) => format!("{}[{index}]", self.kind),
                (None, None) => self.kind.clone(),
            }
        }

        fn __str__(&self) -> String {
            self.__repr__()
        }
    }

    impl FidlPathSegment {
        fn from_segment(segment: &PathSegment) -> Self {
            match segment {
                PathSegment::Named { kind, name } => Self {
                    kind: kind.to_string(),
                    name: Some(name.clone()),
                    index: None,
                },
                PathSegment::Indexed { kind, index } => Self {
                    kind: kind.to_string(),
                    name: None,
                    index: Some(*index),
                },
            }
        }
    }

    /// The route from the file root to a node.
    ///
    /// Unlike an id, a path survives a reparse — it addresses by name where names
    /// exist. Pass one to `FidlFile.at_path` to resolve it again. There is no
    /// parser for the printed form; keep the object if you want to resolve it.
    #[pyclass(name = "FidlNodePath", frozen)]
    #[derive(Clone)]
    struct FidlNodePath {
        inner: NodePath,
    }

    #[pymethods]
    impl FidlNodePath {
        #[getter]
        fn segments(&self) -> Vec<FidlPathSegment> {
            self.inner
                .segments()
                .iter()
                .map(FidlPathSegment::from_segment)
                .collect()
        }

        fn __len__(&self) -> usize {
            self.inner.segments().len()
        }

        fn __str__(&self) -> String {
            self.inner.to_string()
        }

        fn __repr__(&self) -> String {
            format!("<FidlNodePath {}>", self.inner)
        }

        fn __eq__(&self, other: &Self) -> bool {
            self.inner == other.inner
        }

        fn __hash__(&self) -> u64 {
            let mut hasher = DefaultHasher::new();
            self.inner.to_string().hash(&mut hasher);
            hasher.finish()
        }
    }

    // ============================================================ diff ===

    /// One difference between two files.
    ///
    /// A single flat class rather than one per variant, so that the case this
    /// exists for stays a one-liner:
    ///
    /// ```python
    /// removed = [c for c in before.diff(after) if c.change_type == "removed"]
    /// ```
    ///
    /// Fields that do not apply to a given `change_type` are `None`.
    #[pyclass(name = "FidlChange", frozen)]
    struct FidlChange {
        /// `"added"`, `"removed"`, `"modified"` or `"moved"`.
        #[pyo3(get)]
        change_type: &'static str,
        /// Where in the tree the change is.
        #[pyo3(get)]
        path: FidlNodePath,
        /// The kind of node that changed, e.g. `"method"`.
        #[pyo3(get)]
        kind: &'static str,
        /// The node's name, for added and removed named nodes.
        #[pyo3(get)]
        name: Option<String>,
        /// Which field changed, for `"modified"`.
        #[pyo3(get)]
        field: Option<&'static str>,
        #[pyo3(get)]
        before: Option<String>,
        #[pyo3(get)]
        after: Option<String>,
        /// Old and new sibling positions, for `"moved"`.
        #[pyo3(get)]
        from_index: Option<usize>,
        #[pyo3(get)]
        to_index: Option<usize>,
        rendered: String,
    }

    #[pymethods]
    impl FidlChange {
        fn __str__(&self) -> String {
            self.rendered.clone()
        }

        fn __repr__(&self) -> String {
            format!("<FidlChange {}>", self.rendered)
        }
    }

    impl FidlChange {
        fn from_change(change: &Change) -> Self {
            let base = Self {
                change_type: "",
                path: FidlNodePath {
                    inner: change.path().clone(),
                },
                kind: change.kind(),
                name: None,
                field: None,
                before: None,
                after: None,
                from_index: None,
                to_index: None,
                rendered: change.to_string(),
            };
            match change {
                Change::Added { name, .. } => Self {
                    change_type: "added",
                    name: name.clone(),
                    ..base
                },
                Change::Removed { name, .. } => Self {
                    change_type: "removed",
                    name: name.clone(),
                    ..base
                },
                Change::Modified {
                    field,
                    before,
                    after,
                    ..
                } => Self {
                    change_type: "modified",
                    field: Some(field),
                    before: Some(before.clone()),
                    after: Some(after.clone()),
                    ..base
                },
                Change::Moved { from, to, .. } => Self {
                    change_type: "moved",
                    from_index: Some(*from),
                    to_index: Some(*to),
                    ..base
                },
            }
        }
    }

    // ======================================================= traversal ===

    /// Which handle class a node needs. Recorded when a traversal is collected so
    /// that the handle itself can be built lazily.
    #[derive(Clone, Copy)]
    enum NodeKind {
        File,
        Package,
        ImportNamespace,
        ImportModel,
        Interface,
        TypeCollection,
        Version,
        Method,
        ParamList,
        Attribute,
        Structure,
        Enumeration,
        EnumValue,
        TypeDef,
        VariableDeclaration,
        Annotation,
        Comment,
    }

    impl NodeKind {
        fn of(node: &NodeRef<'_>) -> Self {
            match node {
                NodeRef::File(_) => NodeKind::File,
                NodeRef::Package(_) => NodeKind::Package,
                NodeRef::ImportNamespace(_) => NodeKind::ImportNamespace,
                NodeRef::ImportModel(_) => NodeKind::ImportModel,
                NodeRef::Interface(_) => NodeKind::Interface,
                NodeRef::TypeCollection(_) => NodeKind::TypeCollection,
                NodeRef::Version(_) => NodeKind::Version,
                NodeRef::Method(_) => NodeKind::Method,
                NodeRef::ParamList(_) => NodeKind::ParamList,
                NodeRef::Attribute(_) => NodeKind::Attribute,
                NodeRef::Structure(_) => NodeKind::Structure,
                NodeRef::Enumeration(_) => NodeKind::Enumeration,
                NodeRef::EnumValue(_) => NodeKind::EnumValue,
                NodeRef::TypeDef(_) => NodeKind::TypeDef,
                NodeRef::VariableDeclaration(_) => NodeKind::VariableDeclaration,
                NodeRef::Annotation(_) => NodeKind::Annotation,
                NodeRef::Comment(_) => NodeKind::Comment,
            }
        }
    }

    /// The handle for a node, of whichever class matches its kind.
    fn handle_for(
        py: Python<'_>,
        file: &Shared,
        kind: NodeKind,
        id: NodeId,
    ) -> PyResult<Py<PyAny>> {
        Ok(match kind {
            NodeKind::File => Py::new(
                py,
                FidlFile {
                    inner: file.clone(),
                },
            )?
            .into_any(),
            NodeKind::Package => Py::new(py, FidlPackage::handle(file, id))?.into_any(),
            NodeKind::ImportNamespace => {
                Py::new(py, FidlImportNamespace::handle(file, id))?.into_any()
            }
            NodeKind::ImportModel => Py::new(py, FidlImportModel::handle(file, id))?.into_any(),
            NodeKind::Interface => Py::new(py, FidlInterface::handle(file, id))?.into_any(),
            NodeKind::TypeCollection => {
                Py::new(py, FidlTypeCollection::handle(file, id))?.into_any()
            }
            NodeKind::Version => Py::new(py, FidlVersion::handle(file, id))?.into_any(),
            NodeKind::Method => Py::new(py, FidlMethod::handle(file, id))?.into_any(),
            NodeKind::ParamList => Py::new(py, FidlParamList::handle(file, id))?.into_any(),
            NodeKind::Attribute => Py::new(py, FidlAttribute::handle(file, id))?.into_any(),
            NodeKind::Structure => Py::new(py, FidlStructure::handle(file, id))?.into_any(),
            NodeKind::Enumeration => Py::new(py, FidlEnumeration::handle(file, id))?.into_any(),
            NodeKind::EnumValue => Py::new(py, FidlEnumValue::handle(file, id))?.into_any(),
            NodeKind::TypeDef => Py::new(py, FidlTypeDef::handle(file, id))?.into_any(),
            NodeKind::VariableDeclaration => {
                Py::new(py, FidlVariableDeclaration::handle(file, id))?.into_any()
            }
            NodeKind::Annotation => Py::new(py, FidlAnnotation::handle(file, id))?.into_any(),
            NodeKind::Comment => Py::new(py, FidlComment::handle(file, id))?.into_any(),
        })
    }

    fn collect_nodes<'a>(nodes: impl Iterator<Item = NodeRef<'a>>) -> Vec<(NodeKind, NodeId)> {
        nodes.map(|n| (NodeKind::of(&n), n.id())).collect()
    }

    /// A lazy iterator over node handles.
    ///
    /// The ids are collected up front, under one lock; each handle is built as it
    /// is yielded. Mutating the tree mid-iteration is allowed and does not
    /// invalidate the iterator — a handle to a node that has since been removed
    /// simply raises `StaleNodeError` when it is read.
    #[pyclass(name = "FidlNodes")]
    struct FidlNodes {
        file: Shared,
        nodes: Vec<(NodeKind, NodeId)>,
        index: usize,
    }

    #[pymethods]
    impl FidlNodes {
        fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
            slf
        }

        fn __next__(&mut self, py: Python<'_>) -> PyResult<Option<Py<PyAny>>> {
            match self.nodes.get(self.index) {
                None => Ok(None),
                Some(&(kind, id)) => {
                    self.index += 1;
                    Ok(Some(handle_for(py, &self.file, kind, id)?))
                }
            }
        }

        fn __len__(&self) -> usize {
            self.nodes.len()
        }
    }

    // ========================================================= handles ===

    /// Handles for the annotations of any annotated node.
    fn annotation_handles(file: &Shared, node: &impl Annotated) -> Vec<FidlAnnotation> {
        node.annotations()
            .iter()
            .map(|a| FidlAnnotation::handle(file, a.id()))
            .collect()
    }

    fn comment_handles(file: &Shared, comments: &[Comment]) -> Vec<FidlComment> {
        comments
            .iter()
            .map(|c| FidlComment::handle(file, c.id))
            .collect()
    }

    fn interface_member_id(member: &InterfaceMember) -> NodeId {
        match member {
            InterfaceMember::Method(x) => x.id(),
            InterfaceMember::Attribute(x) => x.id(),
            InterfaceMember::Structure(x) => x.id(),
            InterfaceMember::Enumeration(x) => x.id(),
            InterfaceMember::TypeDef(x) => x.id(),
            InterfaceMember::Comment(x) => x.id,
        }
    }

    fn type_collection_member_id(member: &TypeCollectionMember) -> NodeId {
        match member {
            TypeCollectionMember::TypeDef(x) => x.id(),
            TypeCollectionMember::Structure(x) => x.id(),
            TypeCollectionMember::Enumeration(x) => x.id(),
            TypeCollectionMember::Comment(x) => x.id,
        }
    }

    fn struct_member_id(member: &StructMember) -> NodeId {
        match member {
            StructMember::Field(x) => x.id(),
            StructMember::Comment(x) => x.id,
        }
    }

    fn enum_member_id(member: &EnumMember) -> NodeId {
        match member {
            EnumMember::Value(x) => x.id(),
            EnumMember::Comment(x) => x.id,
        }
    }

    fn param_member_id(member: &ParamMember) -> NodeId {
        match member {
            ParamMember::Param(x) => x.id(),
            ParamMember::Comment(x) => x.id,
        }
    }

    fn file_member_id(member: &FileMember) -> NodeId {
        match member {
            FileMember::Package(x) => x.id(),
            FileMember::ImportNamespace(x) => x.id(),
            FileMember::ImportModel(x) => x.id(),
            FileMember::Interface(x) => x.id(),
            FileMember::TypeCollection(x) => x.id(),
            FileMember::Comment(x) => x.id,
        }
    }

    /// Declares a handle pyclass, the accessors that resolve it, and the mutation
    /// plumbing every node shares.
    ///
    /// `$variant` is the [`NodeRef`] variant the id must resolve to. Anything else
    /// means the node has been removed from the tree, which raises
    /// `StaleNodeError` rather than returning data from the wrong node.
    ///
    /// The last positional argument says how much the node carries:
    ///
    /// - `annotated` — a `NodeMeta` and annotations (most nodes);
    /// - `meta` — a `NodeMeta` only (package, version, imports, annotation);
    /// - `bare` — neither (comments, which hold `id` and `span` directly).
    ///
    /// The arms delegate downwards, so each level's methods are written once. They
    /// have to be text-substituted rather than factored into a shared trait
    /// because PyO3 rejects a macro invocation inside `#[pymethods]`, which is
    /// also why `handle!` emits the whole block and takes the type's own methods
    /// as a token tree.
    macro_rules! handle {
        ($py_name:literal, $rust:ident, $variant:ident, $ast:ty, $kind:literal, annotated, { $($body:tt)* }) => {
            handle!($py_name, $rust, $variant, $ast, $kind, meta, {
                /// Every annotation on this node, in source order.
                #[getter]
                fn annotations(&self) -> PyResult<Vec<FidlAnnotation>> {
                    self.with(|n| annotation_handles(&self.file, n))
                }

                /// The annotation of this name, if there is one.
                fn annotation(&self, name: &str) -> PyResult<Option<FidlAnnotation>> {
                    self.with(|n| {
                        n.annotation(name)
                            .map(|a| FidlAnnotation::handle(&self.file, a.id()))
                    })
                }

                /// Replace an annotation's contents, or add it if it is absent.
                fn set_annotation(&self, name: &str, contents: &str) -> PyResult<FidlAnnotation> {
                    let id = self.edit_then(
                        |n| {
                            n.set_annotation(name, contents);
                            Ok(n.annotations()
                                .iter()
                                .position(|a| a.name == name)
                                .expect("set_annotation just added or replaced it"))
                        },
                        |n, index| n.annotations()[index].id(),
                    )?;
                    Ok(FidlAnnotation::handle(&self.file, id))
                }

                /// Remove an annotation by name. Returns whether one was there.
                fn remove_annotation(&self, name: &str) -> PyResult<bool> {
                    self.edit(|n| n.remove_annotation(name).is_some())
                }

                $($body)*
            });
        };

        ($py_name:literal, $rust:ident, $variant:ident, $ast:ty, $kind:literal, meta, { $($body:tt)* }) => {
            handle!($py_name, $rust, $variant, $ast, $kind, bare, {
                /// Comments bound directly above this node. They travel with it:
                /// removing the node removes them too.
                #[getter]
                fn leading_comments(&self) -> PyResult<Vec<FidlComment>> {
                    self.with(|n| comment_handles(&self.file, &n.meta().leading_comments))
                }

                /// Comments inside the node's own header, as in
                /// `interface /* x */ Foo {`.
                #[getter]
                fn header_comments(&self) -> PyResult<Vec<FidlComment>> {
                    self.with(|n| comment_handles(&self.file, &n.meta().header_comments))
                }

                /// Comments inside the node that follow its content, typically on
                /// the same line.
                #[getter]
                fn trailing_comments(&self) -> PyResult<Vec<FidlComment>> {
                    self.with(|n| comment_handles(&self.file, &n.meta().trailing_comments))
                }

                /// Add a comment directly above this node.
                fn add_leading_comment(&self, comment: NewComment) -> PyResult<FidlComment> {
                    let id = self.edit_then(
                        |n| {
                            n.meta_mut().leading_comments.push(comment.build());
                            Ok(n.meta().leading_comments.len() - 1)
                        },
                        |n, index| n.meta().leading_comments[index].id,
                    )?;
                    Ok(FidlComment::handle(&self.file, id))
                }

                /// Blank lines that preceded this node in the source.
                #[getter]
                fn blank_lines_before(&self) -> PyResult<u8> {
                    self.with(|n| n.meta().blank_lines_before)
                }

                /// True once the node has been touched, which is what makes
                /// `to_fidl(preserve=True)` re-format it instead of reusing the
                /// original text.
                #[getter]
                fn is_dirty(&self) -> PyResult<bool> {
                    self.with(|n| n.is_dirty())
                }

                $($body)*
            });
        };

        ($py_name:literal, $rust:ident, $variant:ident, $ast:ty, $kind:literal, bare, { $($body:tt)* }) => {
            #[pyclass(name = $py_name, frozen)]
            #[derive(Clone)]
            struct $rust {
                file: Shared,
                id: NodeId,
            }

            impl $rust {
                /// Read the node this handle names.
                fn with<R>(&self, f: impl FnOnce(&$ast) -> R) -> PyResult<R> {
                    let guard = self.file.read().map_err(|_| poisoned())?;
                    match guard.get(self.id) {
                        Some(NodeRef::$variant(node)) => Ok(f(node)),
                        _ => Err(stale($kind, self.id)),
                    }
                }

                /// Read the node together with the file, for whole-tree questions
                /// such as its path.
                fn with_node<R>(&self, f: impl FnOnce(&AstFile, NodeRef<'_>) -> R) -> PyResult<R> {
                    let guard = self.file.read().map_err(|_| poisoned())?;
                    match guard.get(self.id) {
                        Some(node @ NodeRef::$variant(_)) => Ok(f(&guard, node)),
                        _ => Err(stale($kind, self.id)),
                    }
                }

                /// Mutate the node this handle names, then hand out ids to
                /// anything the mutation inserted.
                ///
                /// Resolution is by id, never by name — that is the whole point of
                /// `get_mut`.
                fn edit<R>(&self, f: impl FnOnce(&mut $ast) -> R) -> PyResult<R> {
                    let mut guard = self.file.write().map_err(|_| poisoned())?;
                    let out = match guard.get_mut(self.id) {
                        Some(NodeRefMut::$variant(node)) => f(node),
                        _ => return Err(stale($kind, self.id)),
                    };
                    guard.assign_missing_ids();
                    Ok(out)
                }

                /// Mutate, assign ids, then read something back out of the same
                /// node — under a single lock.
                ///
                /// Insertion needs both halves: a node built by a `New*` carries
                /// `NodeId::UNASSIGNED` and is invisible to `get()` until ids are
                /// handed out, so its handle cannot be built until after the edit.
                /// `mutate` returns wherever it put the node (usually an index) and
                /// `locate` turns that into an id.
                ///
                /// Unused for comments, which have no children to insert.
                #[allow(dead_code)]
                fn edit_then<T, R>(
                    &self,
                    mutate: impl FnOnce(&mut $ast) -> PyResult<T>,
                    locate: impl FnOnce(&$ast, T) -> R,
                ) -> PyResult<R> {
                    let mut guard = self.file.write().map_err(|_| poisoned())?;
                    let found = match guard.get_mut(self.id) {
                        Some(NodeRefMut::$variant(node)) => mutate(node)?,
                        _ => return Err(stale($kind, self.id)),
                    };
                    guard.assign_missing_ids();
                    match guard.get(self.id) {
                        Some(NodeRef::$variant(node)) => Ok(locate(node, found)),
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

                /// What kind of node this is, e.g. `"method"`.
                #[getter]
                fn kind(&self) -> PyResult<&'static str> {
                    self.with_node(|_, node| node.kind_name())
                }

                /// False once the node has been removed from the tree.
                fn is_valid(&self) -> bool {
                    self.with(|_| ()).is_ok()
                }

                /// The node's original `(start, end)` byte range, or `None` if it
                /// was constructed rather than parsed. Goes stale once the node is
                /// edited.
                #[getter]
                fn span(&self) -> PyResult<Option<(u32, u32)>> {
                    self.with_node(|_, node| node.span().map(|s| (s.start, s.end)))
                }

                /// Where this node sits in the tree.
                ///
                /// Called `node_path` rather than `path` because a package's
                /// `path` is its dotted name, and that name was here first.
                #[getter]
                fn node_path(&self) -> PyResult<Option<FidlNodePath>> {
                    self.with_node(|file, node| {
                        file.path_of(node.id()).map(|inner| FidlNodePath { inner })
                    })
                }

                /// Every node beneath this one, pre-order.
                fn descendants(&self) -> PyResult<FidlNodes> {
                    let nodes = self.with_node(|_, node| collect_nodes(node.descendants()))?;
                    Ok(FidlNodes {
                        file: self.file.clone(),
                        nodes,
                        index: 0,
                    })
                }

                fn __repr__(&self) -> String {
                    format!("<{} id={}>", $py_name, self.id.get())
                }

                $($body)*
            }
        };
    }

    // ---------------------------------------------------------------- file ---

    #[pyclass(name = "FidlFile", frozen)]
    #[derive(Clone)]
    struct FidlFile {
        inner: Shared,
    }

    impl FidlFile {
        /// Mutate the file, then hand out ids to whatever was inserted.
        fn edit<R>(&self, f: impl FnOnce(&mut AstFile) -> R) -> PyResult<R> {
            write(&self.inner, |file| file.edit(f))
        }

        /// The file-level counterpart of the handle macro's `edit_then`.
        fn edit_then<T, R>(
            &self,
            mutate: impl FnOnce(&mut AstFile) -> PyResult<T>,
            locate: impl FnOnce(&AstFile, T) -> R,
        ) -> PyResult<R> {
            let mut guard = self.inner.write().map_err(|_| poisoned())?;
            let found = mutate(&mut guard)?;
            guard.assign_missing_ids();
            Ok(locate(&guard, found))
        }
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

        /// The id of the file node itself. The file is a node like any other.
        #[getter]
        fn id(&self) -> PyResult<u32> {
            read(&self.inner, |f| f.id().get())
        }

        #[getter]
        fn kind(&self) -> &'static str {
            "file"
        }

        // The file turns up as the first node of `nodes()`, so it answers the
        // same three questions every other handle does.

        /// The file root's path, which is always empty.
        #[getter]
        fn node_path(&self) -> FidlNodePath {
            FidlNodePath {
                inner: NodePath::default(),
            }
        }

        /// Always true. A file cannot be removed from itself.
        fn is_valid(&self) -> bool {
            true
        }

        #[getter]
        fn span(&self) -> PyResult<Option<(u32, u32)>> {
            read(&self.inner, |f| f.span().map(|s| (s.start, s.end)))
        }

        /// How many nodes have been given an id.
        #[getter]
        fn node_count(&self) -> PyResult<u32> {
            read(&self.inner, |f| f.node_count())
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

        /// Comments at the top of the file, before anything is declared.
        #[getter]
        fn header_comments(&self) -> PyResult<Vec<FidlComment>> {
            read(&self.inner, |f| {
                comment_handles(&self.inner, &f.meta().header_comments)
            })
        }

        // ---- reading the whole tree ----

        /// Every node in the file, pre-order, starting with the file itself.
        fn nodes(&self) -> PyResult<FidlNodes> {
            let nodes = read(&self.inner, |f| collect_nodes(f.nodes()))?;
            Ok(FidlNodes {
                file: self.inner.clone(),
                nodes,
                index: 0,
            })
        }

        /// Every node beneath the file, pre-order, excluding the file itself.
        fn descendants(&self) -> PyResult<FidlNodes> {
            let nodes = read(&self.inner, |f| collect_nodes(f.as_node().descendants()))?;
            Ok(FidlNodes {
                file: self.inner.clone(),
                nodes,
                index: 0,
            })
        }

        /// The node with this id, or `None` if it is not in the tree.
        fn get(&self, py: Python<'_>, id: u32) -> PyResult<Option<Py<PyAny>>> {
            let found = read(&self.inner, |f| {
                f.get(NodeId::from_raw(id))
                    .map(|n| (NodeKind::of(&n), n.id()))
            })?;
            match found {
                Some((kind, id)) => Ok(Some(handle_for(py, &self.inner, kind, id)?)),
                None => Ok(None),
            }
        }

        /// The path to a node. Accepts any handle, or a bare id.
        fn path_of(&self, node: &Bound<'_, PyAny>) -> PyResult<Option<FidlNodePath>> {
            let id = node_id_of(node)?;
            read(&self.inner, |f| {
                f.path_of(id).map(|inner| FidlNodePath { inner })
            })
        }

        /// The node at this path, or `None` if nothing is there.
        fn at_path(&self, py: Python<'_>, path: &FidlNodePath) -> PyResult<Option<Py<PyAny>>> {
            let found = read(&self.inner, |f| {
                f.at_path(&path.inner).map(|n| (NodeKind::of(&n), n.id()))
            })?;
            match found {
                Some((kind, id)) => Ok(Some(handle_for(py, &self.inner, kind, id)?)),
                None => Ok(None),
            }
        }

        // ---- structural mutation ----

        /// Add a package. Errors if the file already has one — the grammar
        /// permits exactly one.
        fn add_package(&self, package: NewPackage) -> PyResult<FidlPackage> {
            let id = self.edit_then(
                |f| {
                    f.add_package(package.build()).map_err(rejected)?;
                    Ok(())
                },
                |f, ()| f.package().expect("just added").id(),
            )?;
            Ok(FidlPackage::handle(&self.inner, id))
        }

        /// Remove the package. Returns whether there was one.
        ///
        /// The grammar requires a package, so a file without one cannot be read
        /// back — remove it only in order to replace it.
        fn remove_package(&self) -> PyResult<bool> {
            self.edit(|f| f.remove_package().is_some())
        }

        /// Add an `import model "..."`, placed after the package and any existing
        /// imports so that the output still parses.
        fn add_import_model(&self, import: NewImportModel) -> PyResult<FidlImportModel> {
            let id = self.edit_then(
                |f| {
                    let path = import.build().file_path;
                    f.add_import_model(import.build());
                    Ok(path)
                },
                |f, path| f.import_model(&path).expect("just added").id(),
            )?;
            Ok(FidlImportModel::handle(&self.inner, id))
        }

        /// Remove a model import by path. Returns whether one was there.
        fn remove_import_model(&self, file_path: PathBuf) -> PyResult<bool> {
            self.edit(|f| f.remove_import_model(&file_path).is_some())
        }

        /// Add an `import a.b.* from "..."`, placed with the other imports.
        fn add_import_namespace(
            &self,
            namespace: NewImportNamespace,
        ) -> PyResult<FidlImportNamespace> {
            let id = self.edit_then(
                |f| {
                    let from = namespace.build().from;
                    f.add_import_namespace(namespace.build());
                    Ok(from)
                },
                |f, from| f.namespace(&from).expect("just added").id(),
            )?;
            Ok(FidlImportNamespace::handle(&self.inner, id))
        }

        /// Remove a namespace import by the file it reads from. Returns whether
        /// one was there.
        fn remove_import_namespace(&self, from_: PathBuf) -> PyResult<bool> {
            self.edit(|f| f.remove_import_namespace(&from_).is_some())
        }

        /// Add an interface. Errors if one of that name is already there.
        fn add_interface(&self, interface: NewInterface) -> PyResult<FidlInterface> {
            let id = self.edit_then(
                |f| {
                    f.add_interface(interface.build()).map_err(rejected)?;
                    Ok(f.member_count() - 1)
                },
                |f, index| file_member_id(&f.members[index]),
            )?;
            Ok(FidlInterface::handle(&self.inner, id))
        }

        /// Remove an interface by name. Returns whether one was there.
        fn remove_interface(&self, name: &str) -> PyResult<bool> {
            self.edit(|f| f.remove_interface(name).is_some())
        }

        /// Add a type collection. Errors if one of that name is already there.
        fn add_type_collection(
            &self,
            type_collection: NewTypeCollection,
        ) -> PyResult<FidlTypeCollection> {
            let id = self.edit_then(
                |f| {
                    f.add_type_collection(type_collection.build())
                        .map_err(rejected)?;
                    Ok(f.member_count() - 1)
                },
                |f, index| file_member_id(&f.members[index]),
            )?;
            Ok(FidlTypeCollection::handle(&self.inner, id))
        }

        /// Remove a type collection by name. Returns whether one was there.
        fn remove_type_collection(&self, name: &str) -> PyResult<bool> {
            self.edit(|f| f.remove_type_collection(name).is_some())
        }

        /// Append a free-floating comment.
        fn push_comment(&self, comment: NewComment) -> PyResult<FidlComment> {
            let id = self.edit_then(
                |f| {
                    f.push_member(FileMember::Comment(comment.build()));
                    Ok(f.member_count() - 1)
                },
                |f, index| file_member_id(&f.members[index]),
            )?;
            Ok(FidlComment::handle(&self.inner, id))
        }

        // ---- the ordered member list ----

        /// How many members the file has, comments included.
        #[getter]
        fn member_count(&self) -> PyResult<usize> {
            read(&self.inner, |f| f.member_count())
        }

        /// Remove the member at this position. Returns whether there was one.
        fn remove_member_at(&self, index: usize) -> PyResult<bool> {
            self.edit(|f| f.remove_member_at(index).is_some())
        }

        /// Move a member. Out-of-range indices are a no-op, reported as `False`.
        fn move_member(&self, from_index: usize, to_index: usize) -> PyResult<bool> {
            self.edit(|f| f.move_member(from_index, to_index))
        }

        // ---- output ----

        /// The file rendered back to `.fidl` text.
        ///
        /// With `preserve=True`, subtrees that have not been touched come back as
        /// the exact bytes they were read from, so an edit shows up as a minimal
        /// diff. Anything constructed or modified is formatted either way.
        #[pyo3(signature = (preserve=false))]
        fn to_fidl(&self, preserve: bool) -> PyResult<String> {
            read(&self.inner, |f| f.to_fidl_with(mode(preserve)))
        }

        /// Write the file back to where it was read from.
        #[pyo3(signature = (preserve=false))]
        fn save(&self, preserve: bool) -> PyResult<()> {
            let guard = self.inner.read().map_err(|_| poisoned())?;
            let result = if preserve {
                guard.save_preserving()
            } else {
                guard.save()
            };
            result.map_err(|e| PyValueError::new_err(e.to_string()))
        }

        #[pyo3(signature = (path, preserve=false))]
        fn write_to(&self, path: PathBuf, preserve: bool) -> PyResult<()> {
            let guard = self.inner.read().map_err(|_| poisoned())?;
            guard
                .write_to_with(path, mode(preserve))
                .map_err(|e| PyValueError::new_err(e.to_string()))
        }

        /// Problems with the file, as human-readable strings. Empty means sound.
        fn validate(&self) -> PyResult<Vec<String>> {
            read(&self.inner, |f| {
                f.validate().iter().map(|d| d.to_string()).collect()
            })
        }

        /// What changed between this file and `other`.
        ///
        /// Defaults match the Rust `DiffOptions`: comments and ordering count,
        /// blank-line layout does not. Pass all three as `True` for a
        /// meaning-only comparison.
        #[pyo3(signature = (other, ignore_comments=false, ignore_layout=true, ignore_order=false))]
        fn diff(
            &self,
            other: &FidlFile,
            ignore_comments: bool,
            ignore_layout: bool,
            ignore_order: bool,
        ) -> PyResult<Vec<FidlChange>> {
            let options = DiffOptions {
                ignore_comments,
                ignore_layout,
                ignore_order,
            };
            let changes = if Arc::ptr_eq(&self.inner, &other.inner) {
                // One lock, taken once: a second read guard on the same lock in
                // the same thread is not something to rely on.
                read(&self.inner, |f| diff(f, f, &options))?
            } else {
                let mine = self.inner.read().map_err(|_| poisoned())?;
                let theirs = other.inner.read().map_err(|_| poisoned())?;
                diff(&mine, &theirs, &options)
            };
            Ok(changes.iter().map(FidlChange::from_change).collect())
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

    fn mode(preserve: bool) -> Mode {
        if preserve {
            Mode::Preserve
        } else {
            Mode::Format
        }
    }

    // ----------------------------------------------------------- interface ---

    handle!(
        "FidlInterface",
        FidlInterface,
        Interface,
        minimal_fidl_collect::Interface,
        "interface",
        annotated,
        {

        #[getter]
        fn name(&self) -> PyResult<String> {
            self.with(|i| i.name.clone())
        }

        #[setter]
        fn set_name(&self, name: String) -> PyResult<()> {
            self.edit(|i| i.name = name)
        }

        #[getter]
        fn version(&self) -> PyResult<Option<FidlVersion>> {
            self.with(|i| {
                i.version
                    .as_ref()
                    .map(|v| FidlVersion::handle(&self.file, v.id()))
            })
        }

        /// Set the interface's version, replacing any existing one.
        fn set_version(&self, version: NewVersion) -> PyResult<FidlVersion> {
            let id = self.edit_then(
                |i| {
                    i.version = Some(version.build());
                    Ok(())
                },
                |i, ()| i.version.as_ref().expect("just set").id(),
            )?;
            Ok(FidlVersion::handle(&self.file, id))
        }

        /// Drop the version. Returns whether there was one.
        fn remove_version(&self) -> PyResult<bool> {
            self.edit(|i| i.version.take().is_some())
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

        /// Add a method. Errors if one of that name is already there.
        fn add_method(&self, method: NewMethod) -> PyResult<FidlMethod> {
            let id = self.edit_then(
                |i| {
                    i.add_method(method.build()).map_err(rejected)?;
                    Ok(i.member_count() - 1)
                },
                |i, index| interface_member_id(&i.members[index]),
            )?;
            Ok(FidlMethod::handle(&self.file, id))
        }

        /// Remove a method by name. Returns whether one was there.
        ///
        /// Handles to the removed method become stale: reading one raises
        /// `StaleNodeError` rather than resolving to whatever node later
        /// occupies that slot.
        fn remove_method(&self, name: &str) -> PyResult<bool> {
            self.edit(|i| i.remove_method(name).is_some())
        }

        /// Add an attribute. Errors if one of that name is already there.
        fn add_attribute(&self, attribute: NewAttribute) -> PyResult<FidlAttribute> {
            let id = self.edit_then(
                |i| {
                    i.add_attribute(attribute.build()).map_err(rejected)?;
                    Ok(i.member_count() - 1)
                },
                |i, index| interface_member_id(&i.members[index]),
            )?;
            Ok(FidlAttribute::handle(&self.file, id))
        }

        /// Remove an attribute by name. Returns whether one was there.
        fn remove_attribute(&self, name: &str) -> PyResult<bool> {
            self.edit(|i| i.remove_attribute(name).is_some())
        }

        /// Add a struct. Errors if one of that name is already there.
        fn add_structure(&self, structure: NewStructure) -> PyResult<FidlStructure> {
            let id = self.edit_then(
                |i| {
                    i.add_structure(structure.build()).map_err(rejected)?;
                    Ok(i.member_count() - 1)
                },
                |i, index| interface_member_id(&i.members[index]),
            )?;
            Ok(FidlStructure::handle(&self.file, id))
        }

        /// Remove a struct by name. Returns whether one was there.
        fn remove_structure(&self, name: &str) -> PyResult<bool> {
            self.edit(|i| i.remove_structure(name).is_some())
        }

        /// Add an enumeration. Errors if one of that name is already there.
        fn add_enumeration(&self, enumeration: NewEnumeration) -> PyResult<FidlEnumeration> {
            let id = self.edit_then(
                |i| {
                    i.add_enumeration(enumeration.build()).map_err(rejected)?;
                    Ok(i.member_count() - 1)
                },
                |i, index| interface_member_id(&i.members[index]),
            )?;
            Ok(FidlEnumeration::handle(&self.file, id))
        }

        /// Remove an enumeration by name. Returns whether one was there.
        fn remove_enumeration(&self, name: &str) -> PyResult<bool> {
            self.edit(|i| i.remove_enumeration(name).is_some())
        }

        /// Add a typedef. Errors if one of that name is already there.
        fn add_typedef(&self, typedef: NewTypeDef) -> PyResult<FidlTypeDef> {
            let id = self.edit_then(
                |i| {
                    i.add_typedef(typedef.build()).map_err(rejected)?;
                    Ok(i.member_count() - 1)
                },
                |i, index| interface_member_id(&i.members[index]),
            )?;
            Ok(FidlTypeDef::handle(&self.file, id))
        }

        /// Remove a typedef by name. Returns whether one was there.
        fn remove_typedef(&self, name: &str) -> PyResult<bool> {
            self.edit(|i| i.remove_typedef(name).is_some())
        }

        /// How many members the interface has, comments included.
        #[getter]
        fn member_count(&self) -> PyResult<usize> {
            self.with(|i| i.member_count())
        }

        /// Insert a member at a position, rather than appending.
        ///
        /// The index is clamped to the end of the list.
        fn insert_member_at(
            &self,
            py: Python<'_>,
            index: usize,
            member: NewInterfaceMember,
        ) -> PyResult<Py<PyAny>> {
            let kind = member.node_kind();
            let id = self.edit_then(
                |i| {
                    let index = index.min(i.member_count());
                    i.insert_member_at(index, member.build());
                    Ok(index)
                },
                |i, index| interface_member_id(&i.members[index]),
            )?;
            handle_for(py, &self.file, kind, id)
        }

        /// Remove the member at this position. Returns whether there was one.
        fn remove_member_at(&self, index: usize) -> PyResult<bool> {
            self.edit(|i| i.remove_member_at(index).is_some())
        }

        /// Move a member. Out-of-range indices are a no-op, reported as `False`.
        fn move_member(&self, from_index: usize, to_index: usize) -> PyResult<bool> {
            self.edit(|i| i.move_member(from_index, to_index))
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
        annotated,
        {

        #[getter]
        fn name(&self) -> PyResult<String> {
            self.with(|t| t.name.clone())
        }

        #[setter]
        fn set_name(&self, name: String) -> PyResult<()> {
            self.edit(|t| t.name = name)
        }

        /// True for an unnamed `typeCollection { ... }`. Legal, but nothing can
        /// refer to it, and `validate()` says so.
        #[getter]
        fn is_anonymous(&self) -> PyResult<bool> {
            self.with(|t| t.is_anonymous())
        }

        #[getter]
        fn version(&self) -> PyResult<Option<FidlVersion>> {
            self.with(|t| {
                t.version
                    .as_ref()
                    .map(|v| FidlVersion::handle(&self.file, v.id()))
            })
        }

        /// Set the version, replacing any existing one.
        fn set_version(&self, version: NewVersion) -> PyResult<FidlVersion> {
            let id = self.edit_then(
                |t| {
                    t.version = Some(version.build());
                    Ok(())
                },
                |t, ()| t.version.as_ref().expect("just set").id(),
            )?;
            Ok(FidlVersion::handle(&self.file, id))
        }

        /// Drop the version. Returns whether there was one.
        fn remove_version(&self) -> PyResult<bool> {
            self.edit(|t| t.version.take().is_some())
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

        /// Add a typedef. Errors if one of that name is already there.
        fn add_typedef(&self, typedef: NewTypeDef) -> PyResult<FidlTypeDef> {
            let id = self.edit_then(
                |t| {
                    t.add_typedef(typedef.build()).map_err(rejected)?;
                    Ok(t.member_count() - 1)
                },
                |t, index| type_collection_member_id(&t.members[index]),
            )?;
            Ok(FidlTypeDef::handle(&self.file, id))
        }

        /// Remove a typedef by name. Returns whether one was there.
        fn remove_typedef(&self, name: &str) -> PyResult<bool> {
            self.edit(|t| t.remove_typedef(name).is_some())
        }

        /// Add a struct. Errors if one of that name is already there.
        fn add_structure(&self, structure: NewStructure) -> PyResult<FidlStructure> {
            let id = self.edit_then(
                |t| {
                    t.add_structure(structure.build()).map_err(rejected)?;
                    Ok(t.member_count() - 1)
                },
                |t, index| type_collection_member_id(&t.members[index]),
            )?;
            Ok(FidlStructure::handle(&self.file, id))
        }

        /// Remove a struct by name. Returns whether one was there.
        fn remove_structure(&self, name: &str) -> PyResult<bool> {
            self.edit(|t| t.remove_structure(name).is_some())
        }

        /// Add an enumeration. Errors if one of that name is already there.
        fn add_enumeration(&self, enumeration: NewEnumeration) -> PyResult<FidlEnumeration> {
            let id = self.edit_then(
                |t| {
                    t.add_enumeration(enumeration.build()).map_err(rejected)?;
                    Ok(t.member_count() - 1)
                },
                |t, index| type_collection_member_id(&t.members[index]),
            )?;
            Ok(FidlEnumeration::handle(&self.file, id))
        }

        /// Remove an enumeration by name. Returns whether one was there.
        fn remove_enumeration(&self, name: &str) -> PyResult<bool> {
            self.edit(|t| t.remove_enumeration(name).is_some())
        }

        #[getter]
        fn member_count(&self) -> PyResult<usize> {
            self.with(|t| t.member_count())
        }

        /// Insert a member at a position. The index is clamped to the end.
        fn insert_member_at(
            &self,
            py: Python<'_>,
            index: usize,
            member: NewTypeCollectionMember,
        ) -> PyResult<Py<PyAny>> {
            let kind = member.node_kind();
            let id = self.edit_then(
                |t| {
                    let index = index.min(t.member_count());
                    t.insert_member_at(index, member.build());
                    Ok(index)
                },
                |t, index| type_collection_member_id(&t.members[index]),
            )?;
            handle_for(py, &self.file, kind, id)
        }

        fn remove_member_at(&self, index: usize) -> PyResult<bool> {
            self.edit(|t| t.remove_member_at(index).is_some())
        }

        fn move_member(&self, from_index: usize, to_index: usize) -> PyResult<bool> {
            self.edit(|t| t.move_member(from_index, to_index))
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
        annotated,
        {

        #[getter]
        fn name(&self) -> PyResult<String> {
            self.with(|m| m.name.clone())
        }

        #[setter]
        fn set_name(&self, name: String) -> PyResult<()> {
            self.edit(|m| m.name = name)
        }

        /// The `in { }` list as a node of its own, so that comments and
        /// annotations inside it have an owner.
        #[getter]
        fn inputs(&self) -> PyResult<FidlParamList> {
            self.with(|m| FidlParamList::handle(&self.file, m.inputs.id()))
        }

        /// The `out { }` list.
        #[getter]
        fn outputs(&self) -> PyResult<FidlParamList> {
            self.with(|m| FidlParamList::handle(&self.file, m.outputs.id()))
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

        /// Add an input parameter. Errors if the name is taken.
        fn add_input(&self, parameter: NewParameter) -> PyResult<FidlVariableDeclaration> {
            let id = self.edit_then(
                |m| {
                    m.inputs.add_param(parameter.build()).map_err(rejected)?;
                    Ok(m.inputs.member_count() - 1)
                },
                |m, index| param_member_id(&m.inputs.members[index]),
            )?;
            Ok(FidlVariableDeclaration::handle(&self.file, id))
        }

        /// Remove an input parameter by name. Returns whether one was there.
        fn remove_input(&self, name: &str) -> PyResult<bool> {
            self.edit(|m| m.inputs.remove_param(name).is_some())
        }

        /// Add an output parameter. Errors if the name is taken.
        fn add_output(&self, parameter: NewParameter) -> PyResult<FidlVariableDeclaration> {
            let id = self.edit_then(
                |m| {
                    m.outputs.add_param(parameter.build()).map_err(rejected)?;
                    Ok(m.outputs.member_count() - 1)
                },
                |m, index| param_member_id(&m.outputs.members[index]),
            )?;
            Ok(FidlVariableDeclaration::handle(&self.file, id))
        }

        /// Remove an output parameter by name. Returns whether one was there.
        fn remove_output(&self, name: &str) -> PyResult<bool> {
            self.edit(|m| m.outputs.remove_param(name).is_some())
        }

        fn __str__(&self) -> PyResult<String> {
            self.with(|m| format!("{m:#?}"))
        }
        }
    );

    handle!(
        "FidlParamList",
        FidlParamList,
        ParamList,
        minimal_fidl_collect::ParamList,
        "parameter list",
        annotated,
        {

        #[getter]
        fn parameters(&self) -> PyResult<Vec<FidlVariableDeclaration>> {
            self.with(|p| {
                p.params()
                    .map(|v| FidlVariableDeclaration::handle(&self.file, v.id()))
                    .collect()
            })
        }

        /// Add a parameter. Errors if the name is taken.
        fn add_parameter(&self, parameter: NewParameter) -> PyResult<FidlVariableDeclaration> {
            let id = self.edit_then(
                |p| {
                    p.add_param(parameter.build()).map_err(rejected)?;
                    Ok(p.member_count() - 1)
                },
                |p, index| param_member_id(&p.members[index]),
            )?;
            Ok(FidlVariableDeclaration::handle(&self.file, id))
        }

        /// Remove a parameter by name. Returns whether one was there.
        fn remove_parameter(&self, name: &str) -> PyResult<bool> {
            self.edit(|p| p.remove_param(name).is_some())
        }

        #[getter]
        fn member_count(&self) -> PyResult<usize> {
            self.with(|p| p.member_count())
        }

        /// Insert a member at a position. The index is clamped to the end.
        fn insert_member_at(
            &self,
            py: Python<'_>,
            index: usize,
            member: NewParamMember,
        ) -> PyResult<Py<PyAny>> {
            let kind = member.node_kind();
            let id = self.edit_then(
                |p| {
                    let index = index.min(p.member_count());
                    p.insert_member_at(index, member.build());
                    Ok(index)
                },
                |p, index| param_member_id(&p.members[index]),
            )?;
            handle_for(py, &self.file, kind, id)
        }

        fn remove_member_at(&self, index: usize) -> PyResult<bool> {
            self.edit(|p| p.remove_member_at(index).is_some())
        }

        fn move_member(&self, from_index: usize, to_index: usize) -> PyResult<bool> {
            self.edit(|p| p.move_member(from_index, to_index))
        }

        fn __str__(&self) -> PyResult<String> {
            self.with(|p| format!("{p:#?}"))
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
        annotated,
        {

        #[getter]
        fn name(&self) -> PyResult<String> {
            self.with(|s| s.name.clone())
        }

        #[setter]
        fn set_name(&self, name: String) -> PyResult<()> {
            self.edit(|s| s.name = name)
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

        /// Add a field. Errors if the name is taken.
        fn add_field(&self, field: NewParameter) -> PyResult<FidlVariableDeclaration> {
            let id = self.edit_then(
                |s| {
                    s.add_field(field.build()).map_err(rejected)?;
                    Ok(s.member_count() - 1)
                },
                |s, index| struct_member_id(&s.members[index]),
            )?;
            Ok(FidlVariableDeclaration::handle(&self.file, id))
        }

        /// Remove a field by name. Returns whether one was there.
        fn remove_field(&self, name: &str) -> PyResult<bool> {
            self.edit(|s| s.remove_field(name).is_some())
        }

        #[getter]
        fn member_count(&self) -> PyResult<usize> {
            self.with(|s| s.member_count())
        }

        /// Insert a member at a position. The index is clamped to the end.
        fn insert_member_at(
            &self,
            py: Python<'_>,
            index: usize,
            member: NewStructMember,
        ) -> PyResult<Py<PyAny>> {
            let kind = member.node_kind();
            let id = self.edit_then(
                |s| {
                    let index = index.min(s.member_count());
                    s.insert_member_at(index, member.build());
                    Ok(index)
                },
                |s, index| struct_member_id(&s.members[index]),
            )?;
            handle_for(py, &self.file, kind, id)
        }

        fn remove_member_at(&self, index: usize) -> PyResult<bool> {
            self.edit(|s| s.remove_member_at(index).is_some())
        }

        fn move_member(&self, from_index: usize, to_index: usize) -> PyResult<bool> {
            self.edit(|s| s.move_member(from_index, to_index))
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
        annotated,
        {

        #[getter]
        fn name(&self) -> PyResult<String> {
            self.with(|e| e.name.clone())
        }

        #[setter]
        fn set_name(&self, name: String) -> PyResult<()> {
            self.edit(|e| e.name = name)
        }

        #[getter]
        fn values(&self) -> PyResult<Vec<FidlEnumValue>> {
            self.with(|e| {
                e.values()
                    .map(|v| FidlEnumValue::handle(&self.file, v.id()))
                    .collect()
            })
        }

        /// Add a value. Errors if one of that name is already there.
        fn add_value(&self, value: NewEnumValue) -> PyResult<FidlEnumValue> {
            let id = self.edit_then(
                |e| {
                    e.add_value(value.build()).map_err(rejected)?;
                    Ok(e.member_count() - 1)
                },
                |e, index| enum_member_id(&e.members[index]),
            )?;
            Ok(FidlEnumValue::handle(&self.file, id))
        }

        /// Remove a value by name. Returns whether one was there.
        fn remove_value(&self, name: &str) -> PyResult<bool> {
            self.edit(|e| e.remove_value(name).is_some())
        }

        #[getter]
        fn member_count(&self) -> PyResult<usize> {
            self.with(|e| e.member_count())
        }

        /// Insert a member at a position. The index is clamped to the end.
        fn insert_member_at(
            &self,
            py: Python<'_>,
            index: usize,
            member: NewEnumMember,
        ) -> PyResult<Py<PyAny>> {
            let kind = member.node_kind();
            let id = self.edit_then(
                |e| {
                    let index = index.min(e.member_count());
                    e.insert_member_at(index, member.build());
                    Ok(index)
                },
                |e, index| enum_member_id(&e.members[index]),
            )?;
            handle_for(py, &self.file, kind, id)
        }

        fn remove_member_at(&self, index: usize) -> PyResult<bool> {
            self.edit(|e| e.remove_member_at(index).is_some())
        }

        fn move_member(&self, from_index: usize, to_index: usize) -> PyResult<bool> {
            self.edit(|e| e.move_member(from_index, to_index))
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
        annotated,
        {

        #[getter]
        fn name(&self) -> PyResult<String> {
            self.with(|v| v.name.clone())
        }

        #[setter]
        fn set_name(&self, name: String) -> PyResult<()> {
            self.edit(|v| v.name = name)
        }

        #[getter]
        fn value(&self) -> PyResult<Option<u64>> {
            self.with(|v| v.value)
        }

        #[setter]
        fn set_value(&self, value: Option<u64>) -> PyResult<()> {
            self.edit(|v| v.value = value)
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
        annotated,
        {

        #[getter]
        fn name(&self) -> PyResult<String> {
            self.with(|a| a.name.clone())
        }

        #[setter]
        fn set_name(&self, name: String) -> PyResult<()> {
            self.edit(|a| a.name = name)
        }

        #[getter]
        fn type_name(&self) -> PyResult<String> {
            self.with(|a| a.type_n.clone())
        }

        #[setter]
        fn set_type_name(&self, type_name: String) -> PyResult<()> {
            self.edit(|a| a.type_n = type_name)
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
        annotated,
        {

        #[getter]
        fn name(&self) -> PyResult<String> {
            self.with(|t| t.name.clone())
        }

        #[setter]
        fn set_name(&self, name: String) -> PyResult<()> {
            self.edit(|t| t.name = name)
        }

        #[getter]
        fn type_name(&self) -> PyResult<String> {
            self.with(|t| t.type_n.clone())
        }

        #[setter]
        fn set_type_name(&self, type_name: String) -> PyResult<()> {
            self.edit(|t| t.type_n = type_name)
        }

        #[getter]
        fn is_array(&self) -> PyResult<bool> {
            self.with(|t| t.is_array)
        }

        #[setter]
        fn set_is_array(&self, is_array: bool) -> PyResult<()> {
            self.edit(|t| t.is_array = is_array)
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
        annotated,
        {

        #[getter]
        fn name(&self) -> PyResult<String> {
            self.with(|v| v.name.clone())
        }

        #[setter]
        fn set_name(&self, name: String) -> PyResult<()> {
            self.edit(|v| v.name = name)
        }

        #[getter]
        fn type_name(&self) -> PyResult<String> {
            self.with(|v| v.type_n.clone())
        }

        #[setter]
        fn set_type_name(&self, type_name: String) -> PyResult<()> {
            self.edit(|v| v.type_n = type_name)
        }

        #[getter]
        fn is_array(&self) -> PyResult<bool> {
            self.with(|v| v.is_array)
        }

        #[setter]
        fn set_is_array(&self, is_array: bool) -> PyResult<()> {
            self.edit(|v| v.is_array = is_array)
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
        meta,
        {

        #[getter]
        fn major(&self) -> PyResult<Option<u32>> {
            self.with(|v| v.major)
        }

        #[setter]
        fn set_major(&self, major: Option<u32>) -> PyResult<()> {
            self.edit(|v| v.major = major)
        }

        #[getter]
        fn minor(&self) -> PyResult<Option<u32>> {
            self.with(|v| v.minor)
        }

        #[setter]
        fn set_minor(&self, minor: Option<u32>) -> PyResult<()> {
            self.edit(|v| v.minor = minor)
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
        meta,
        {

        #[getter]
        fn name(&self) -> PyResult<String> {
            self.with(|a| a.name.clone())
        }

        #[setter]
        fn set_name(&self, name: String) -> PyResult<()> {
            self.edit(|a| a.name = name)
        }

        #[getter]
        fn contents(&self) -> PyResult<String> {
            self.with(|a| a.contents.clone())
        }

        #[setter]
        fn set_contents(&self, contents: String) -> PyResult<()> {
            self.edit(|a| a.contents = contents)
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
        meta,
        {

        #[getter]
        fn path(&self) -> PyResult<Vec<String>> {
            self.with(|p| p.path.clone())
        }

        #[setter]
        fn set_path(&self, path: DottedName) -> PyResult<()> {
            self.edit(|p| p.path = path.segments())
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
        meta,
        {

        #[getter]
        fn file_path(&self) -> PyResult<PathBuf> {
            self.with(|i| i.file_path.clone())
        }

        #[setter]
        fn set_file_path(&self, file_path: PathBuf) -> PyResult<()> {
            self.edit(|i| i.file_path = file_path)
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
        meta,
        {

        #[getter]
        fn from_(&self) -> PyResult<PathBuf> {
            self.with(|n| n.from.clone())
        }

        #[setter]
        fn set_from_(&self, from_: PathBuf) -> PyResult<()> {
            self.edit(|n| n.from = from_)
        }

        #[getter]
        fn imports(&self) -> PyResult<Vec<String>> {
            self.with(|n| n.import.clone())
        }

        #[setter]
        fn set_imports(&self, imports: DottedName) -> PyResult<()> {
            self.edit(|n| n.import = imports.segments())
        }

        /// Always true for an import that was parsed: the grammar requires the
        /// `.*`, so a namespace import without it cannot be read back.
        #[getter]
        fn wildcard(&self) -> PyResult<bool> {
            self.with(|n| n.wildcard)
        }

        fn __str__(&self) -> PyResult<String> {
            self.with(|n| format!("{n:#?}"))
        }
        }
    );

    handle!(
        "FidlComment",
        FidlComment,
        Comment,
        minimal_fidl_collect::Comment,
        "comment",
        bare,
        {

        /// The comment's content, delimiters excluded.
        #[getter]
        fn text(&self) -> PyResult<String> {
            self.with(|c| c.text.clone())
        }

        #[setter]
        fn set_text(&self, text: String) -> PyResult<()> {
            self.edit(|c| c.text = text)
        }

        /// True for `/* ... */`, false for `// ...`.
        #[getter]
        fn is_block(&self) -> PyResult<bool> {
            self.with(|c| c.kind == CommentKind::Block)
        }

        /// The comment as it appears in source, delimiters included.
        fn to_source(&self) -> PyResult<String> {
            self.with(|c| c.to_source())
        }

        fn __str__(&self) -> PyResult<String> {
            self.with(|c| c.to_source())
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
        module.add_class::<FidlParamList>()?;
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
        module.add_class::<FidlComment>()?;
        Ok(())
    }
}
