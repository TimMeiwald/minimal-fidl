# Type stub for franca_idl.
#
# Kept in sync BY HAND. Nothing enforces it, so update it whenever the binding
# gains a class or a method.
#
# There are two families of class here, and the split is the whole API:
#
#   Fidl*  a *handle* on a node that is already in a tree: a shared reference to
#          the parsed file plus a node id. Attribute access resolves the id rather
#          than copying a subtree, so reading `file.interfaces` is cheap no matter
#          how large the file is. Handles cannot be constructed directly.
#
#   New*   a *description* of a node to create. Plain constructible data belonging
#          to no file. Insertion consumes one and returns the handle.
#
# A handle stops working if its node is removed from the tree; accessing one then
# raises StaleNodeError. Ids are not stable across a reparse — paths are, where
# the nodes involved have names.
from typing import Iterator, List, Optional, Union
from pathlib import Path

class StaleNodeError(Exception):
    """The node this object referred to is no longer in the tree."""

def _respond_42() -> int:
    """
    Responds with 42

    This is solely a test function for the package to ensure

    Basic Rust-Python functionality.

    :return: Returns 42
    """

def load_fidl_project(dir: Path) -> list[FidlFile]:
    """
    Parses every .fidl file under `dir`.

    Raises ValueError if a file cannot be read or parsed.
    """

# A dotted name, written either as "org.example" or as ["org", "example"].
DottedName = Union[str, List[str]]

# ---------------------------------------------------------------- new nodes ---

class NewAnnotation:
    """`<** @name: contents **>`"""

    name: str
    contents: str
    def __init__(self, name: str, contents: str = "") -> None: ...

class NewComment:
    """`// text`, or `/* text */` with `block=True`.

    `text` is the content only; delimiters are added when printing.
    """

    text: str
    is_block: bool
    def __init__(self, text: str, block: bool = False) -> None: ...

class NewVersion:
    """`version {major M minor N}`"""

    major: Optional[int]
    minor: Optional[int]
    def __init__(self, major: int, minor: int) -> None: ...

class NewParameter:
    """A `Type name` pair.

    Used for method parameters *and* struct fields — they are the same node in the
    grammar, so there is no separate `NewField`.
    """

    name: str
    type_name: str
    is_array: bool
    def __init__(
        self,
        name: str,
        type_name: str,
        is_array: bool = False,
        annotations: Optional[list[NewAnnotation]] = None,
        leading_comments: Optional[list[NewComment]] = None,
    ) -> None: ...

class NewAttribute:
    """`attribute Type name`"""

    name: str
    type_name: str
    def __init__(
        self,
        name: str,
        type_name: str,
        annotations: Optional[list[NewAnnotation]] = None,
        leading_comments: Optional[list[NewComment]] = None,
    ) -> None: ...

class NewTypeDef:
    """`typedef name is Type`"""

    name: str
    type_name: str
    is_array: bool
    def __init__(
        self,
        name: str,
        type_name: str,
        is_array: bool = False,
        annotations: Optional[list[NewAnnotation]] = None,
        leading_comments: Optional[list[NewComment]] = None,
    ) -> None: ...

class NewEnumValue:
    """One `NAME` or `NAME = 3` inside an enumeration."""

    name: str
    value: Optional[int]
    def __init__(
        self,
        name: str,
        value: Optional[int] = None,
        annotations: Optional[list[NewAnnotation]] = None,
        leading_comments: Optional[list[NewComment]] = None,
    ) -> None: ...

class NewEnumeration:
    """`enumeration name { ... }`"""

    name: str
    def __init__(
        self,
        name: str,
        members: Optional[list[Union[NewEnumValue, NewComment]]] = None,
        annotations: Optional[list[NewAnnotation]] = None,
        leading_comments: Optional[list[NewComment]] = None,
    ) -> None: ...

class NewStructure:
    """`struct name { ... }`"""

    name: str
    def __init__(
        self,
        name: str,
        members: Optional[list[Union[NewParameter, NewComment]]] = None,
        annotations: Optional[list[NewAnnotation]] = None,
        leading_comments: Optional[list[NewComment]] = None,
    ) -> None: ...

class NewMethod:
    """`method name { in { ... } out { ... } }`"""

    name: str
    def __init__(
        self,
        name: str,
        inputs: Optional[list[Union[NewParameter, NewComment]]] = None,
        outputs: Optional[list[Union[NewParameter, NewComment]]] = None,
        annotations: Optional[list[NewAnnotation]] = None,
        leading_comments: Optional[list[NewComment]] = None,
    ) -> None: ...

NewInterfaceMember = Union[
    NewMethod, NewAttribute, NewStructure, NewEnumeration, NewTypeDef, NewComment
]

class NewInterface:
    """`interface name { ... }`

    `members` is one ordered list rather than a list per kind, because member
    order is intrinsic to this tree and per-kind lists would throw away the
    interleaving at construction time.
    """

    name: str
    def __init__(
        self,
        name: str,
        members: Optional[list[NewInterfaceMember]] = None,
        version: Optional[NewVersion] = None,
        annotations: Optional[list[NewAnnotation]] = None,
        leading_comments: Optional[list[NewComment]] = None,
    ) -> None: ...

NewTypeCollectionMember = Union[
    NewTypeDef, NewStructure, NewEnumeration, NewComment
]

class NewTypeCollection:
    """`typeCollection name { ... }`

    The name may be empty: the grammar allows an anonymous collection, though
    `validate()` warns that nothing can refer to it.
    """

    name: str
    def __init__(
        self,
        name: str = "",
        members: Optional[list[NewTypeCollectionMember]] = None,
        version: Optional[NewVersion] = None,
        annotations: Optional[list[NewAnnotation]] = None,
        leading_comments: Optional[list[NewComment]] = None,
    ) -> None: ...

class NewPackage:
    """`package org.example`"""

    path: list[str]
    def __init__(self, path: DottedName) -> None: ...

class NewImportModel:
    """`import model "path"`"""

    file_path: Path
    def __init__(self, file_path: Path) -> None: ...

class NewImportNamespace:
    """`import org.example.* from "path"`

    Always a wildcard import: the grammar requires the `.*`, so anything else
    would print text that cannot be read back.
    """

    namespace: list[str]
    from_: Path
    def __init__(self, namespace: DottedName, from_: Path) -> None: ...

# --------------------------------------------------------------- addressing ---

class FidlPathSegment:
    """One step of a path: a name where the node has one, an index where it does
    not."""

    kind: str
    name: Optional[str]
    """None for a node addressed by position."""
    index: Optional[int]
    """None for a node addressed by name."""

class FidlNodePath:
    """The route from the file root to a node.

    Unlike an id, a path survives a reparse — it addresses by name where names
    exist. Pass one to `FidlFile.at_path` to resolve it again. There is no parser
    for the printed form; keep the object if you want to resolve it.
    """

    segments: list[FidlPathSegment]
    def __len__(self) -> int: ...
    def __eq__(self, other: object) -> bool: ...
    def __hash__(self) -> int: ...

class FidlChange:
    """One difference between two files.

    A single flat class rather than one per variant, so that the case this exists
    for stays a one-liner:

        removed = [c for c in before.diff(after) if c.change_type == "removed"]

    Fields that do not apply to a given `change_type` are None.
    """

    change_type: str
    """"added", "removed", "modified" or "moved"."""
    path: FidlNodePath
    kind: str
    """The kind of node that changed, e.g. "method"."""
    name: Optional[str]
    """The node's name, for added and removed named nodes."""
    field: Optional[str]
    """Which field changed, for "modified"."""
    before: Optional[str]
    after: Optional[str]
    from_index: Optional[int]
    """Old sibling position, for "moved"."""
    to_index: Optional[int]

FidlNode = Union[
    "FidlFile",
    "FidlPackage",
    "FidlImportNamespace",
    "FidlImportModel",
    "FidlInterface",
    "FidlTypeCollection",
    "FidlVersion",
    "FidlMethod",
    "FidlParamList",
    "FidlAttribute",
    "FidlStructure",
    "FidlEnumeration",
    "FidlEnumValue",
    "FidlTypeDef",
    "FidlVariableDeclaration",
    "FidlAnnotation",
    "FidlComment",
]

class FidlNodes:
    """A lazy iterator over node handles.

    The ids are collected up front, under one lock; each handle is built as it is
    yielded. Mutating the tree mid-iteration is allowed and does not invalidate the
    iterator — a handle to a node that has since been removed simply raises
    StaleNodeError when it is read.
    """

    def __iter__(self) -> Iterator[FidlNode]: ...
    def __next__(self) -> FidlNode: ...
    def __len__(self) -> int: ...

# ------------------------------------------------------------------ handles ---

class _Node:
    """Common to every handle. Not instantiable directly."""

    id: int
    """The node's stable id within its file. Not stable across a reparse."""
    kind: str
    """What kind of node this is, e.g. "method"."""
    span: Optional[tuple[int, int]]
    """The node's original (start, end) byte range, or None if it was constructed
    rather than parsed. Goes stale once the node is edited."""
    node_path: Optional[FidlNodePath]
    """Where this node sits in the tree. Called `node_path` rather than `path`
    because a package's `path` is its dotted name."""

    def is_valid(self) -> bool:
        """False once the node has been removed from the tree."""

    def descendants(self) -> FidlNodes:
        """Every node beneath this one, pre-order."""

class _MetaNode(_Node):
    """A node that carries comments and layout. Not instantiable directly."""

    leading_comments: list[FidlComment]
    """Comments bound directly above this node. They travel with it: removing the
    node removes them too."""
    header_comments: list[FidlComment]
    """Comments inside the node's own header, as in `interface /* x */ Foo {`."""
    trailing_comments: list[FidlComment]
    """Comments inside the node that follow its content, usually on the same
    line."""
    blank_lines_before: int
    is_dirty: bool
    """True once the node has been touched, which is what makes
    `to_fidl(preserve=True)` re-format it instead of reusing the original text."""

    def add_leading_comment(self, comment: NewComment) -> FidlComment:
        """Add a comment directly above this node."""

class _AnnotatedNode(_MetaNode):
    """A node that can carry annotations. Not instantiable directly."""

    annotations: list[FidlAnnotation]

    def annotation(self, name: str) -> Optional[FidlAnnotation]:
        """The annotation of this name, if there is one."""

    def set_annotation(self, name: str, contents: str) -> FidlAnnotation:
        """Replace an annotation's contents, or add it if it is absent."""

    def remove_annotation(self, name: str) -> bool:
        """Remove an annotation by name. Returns whether one was there."""

class FidlComment(_Node):
    text: str
    is_block: bool
    """True for `/* ... */`, false for `// ...`."""

    def to_source(self) -> str:
        """The comment as it appears in source, delimiters included."""

class FidlAnnotation(_MetaNode):
    name: str
    contents: str

class FidlVersion(_MetaNode):
    major: Optional[int]
    minor: Optional[int]

class FidlPackage(_MetaNode):
    path: list[str]

class FidlImportNamespace(_MetaNode):
    from_: Path
    imports: list[str]
    wildcard: bool
    """Always true for an import that was parsed: the grammar requires the `.*`."""

class FidlImportModel(_MetaNode):
    file_path: Path

class FidlVariableDeclaration(_AnnotatedNode):
    name: str
    type_name: str
    is_array: bool

class FidlAttribute(_AnnotatedNode):
    name: str
    type_name: str

class FidlTypeDef(_AnnotatedNode):
    name: str
    type_name: str
    is_array: bool

class FidlEnumValue(_AnnotatedNode):
    name: str
    value: Optional[int]

class FidlEnumeration(_AnnotatedNode):
    name: str
    values: list[FidlEnumValue]
    member_count: int
    """How many members there are, comments included."""

    def add_value(self, value: NewEnumValue) -> FidlEnumValue:
        """Add a value. Raises ValueError if one of that name is already there."""

    def remove_value(self, name: str) -> bool:
        """Remove a value by name. Returns whether one was there."""

    def insert_member_at(
        self, index: int, member: Union[NewEnumValue, NewComment]
    ) -> FidlNode:
        """Insert a member at a position. The index is clamped to the end."""

    def remove_member_at(self, index: int) -> bool: ...
    def move_member(self, from_index: int, to_index: int) -> bool:
        """Move a member. Out-of-range indices are a no-op, reported as False."""

class FidlStructure(_AnnotatedNode):
    name: str
    fields: list[FidlVariableDeclaration]
    contents: list[FidlVariableDeclaration]
    """Deprecated alias for `fields`."""
    member_count: int

    def add_field(self, field: NewParameter) -> FidlVariableDeclaration:
        """Add a field. Raises ValueError if the name is taken."""

    def remove_field(self, name: str) -> bool:
        """Remove a field by name. Returns whether one was there."""

    def insert_member_at(
        self, index: int, member: Union[NewParameter, NewComment]
    ) -> FidlNode: ...
    def remove_member_at(self, index: int) -> bool: ...
    def move_member(self, from_index: int, to_index: int) -> bool: ...

class FidlParamList(_AnnotatedNode):
    """A method's `in { }` or `out { }` list.

    A node in its own right so that comments and annotations inside it have an
    owner.
    """

    parameters: list[FidlVariableDeclaration]
    member_count: int

    def add_parameter(self, parameter: NewParameter) -> FidlVariableDeclaration:
        """Add a parameter. Raises ValueError if the name is taken."""

    def remove_parameter(self, name: str) -> bool:
        """Remove a parameter by name. Returns whether one was there."""

    def insert_member_at(
        self, index: int, member: Union[NewParameter, NewComment]
    ) -> FidlNode: ...
    def remove_member_at(self, index: int) -> bool: ...
    def move_member(self, from_index: int, to_index: int) -> bool: ...

class FidlMethod(_AnnotatedNode):
    name: str
    inputs: FidlParamList
    outputs: FidlParamList
    input_parameters: list[FidlVariableDeclaration]
    output_parameters: list[FidlVariableDeclaration]

    def add_input(self, parameter: NewParameter) -> FidlVariableDeclaration:
        """Add an input parameter. Raises ValueError if the name is taken."""

    def remove_input(self, name: str) -> bool: ...
    def add_output(self, parameter: NewParameter) -> FidlVariableDeclaration:
        """Add an output parameter. Raises ValueError if the name is taken."""

    def remove_output(self, name: str) -> bool: ...

class FidlTypeCollection(_AnnotatedNode):
    name: str
    is_anonymous: bool
    """True for an unnamed `typeCollection { ... }`. Legal, but nothing can refer
    to it, and `validate()` says so."""
    version: Optional[FidlVersion]
    typedefs: list[FidlTypeDef]
    structures: list[FidlStructure]
    enumerations: list[FidlEnumeration]
    member_count: int

    def set_version(self, version: NewVersion) -> FidlVersion:
        """Set the version, replacing any existing one."""

    def remove_version(self) -> bool:
        """Drop the version. Returns whether there was one."""

    def add_typedef(self, typedef: NewTypeDef) -> FidlTypeDef: ...
    def remove_typedef(self, name: str) -> bool: ...
    def add_structure(self, structure: NewStructure) -> FidlStructure: ...
    def remove_structure(self, name: str) -> bool: ...
    def add_enumeration(self, enumeration: NewEnumeration) -> FidlEnumeration: ...
    def remove_enumeration(self, name: str) -> bool: ...
    def insert_member_at(
        self, index: int, member: NewTypeCollectionMember
    ) -> FidlNode: ...
    def remove_member_at(self, index: int) -> bool: ...
    def move_member(self, from_index: int, to_index: int) -> bool: ...

class FidlInterface(_AnnotatedNode):
    name: str
    version: Optional[FidlVersion]
    attributes: list[FidlAttribute]
    structures: list[FidlStructure]
    typedefs: list[FidlTypeDef]
    methods: list[FidlMethod]
    enumerations: list[FidlEnumeration]
    member_count: int

    def set_version(self, version: NewVersion) -> FidlVersion:
        """Set the interface's version, replacing any existing one."""

    def remove_version(self) -> bool:
        """Drop the version. Returns whether there was one."""

    def add_method(self, method: NewMethod) -> FidlMethod:
        """Add a method. Raises ValueError if one of that name is already there."""

    def remove_method(self, name: str) -> bool:
        """Remove a method by name. Returns whether one was there.

        Handles to the removed method become stale: reading one raises
        StaleNodeError rather than resolving to whatever node later occupies that
        slot.
        """

    def add_attribute(self, attribute: NewAttribute) -> FidlAttribute: ...
    def remove_attribute(self, name: str) -> bool: ...
    def add_structure(self, structure: NewStructure) -> FidlStructure: ...
    def remove_structure(self, name: str) -> bool: ...
    def add_enumeration(self, enumeration: NewEnumeration) -> FidlEnumeration: ...
    def remove_enumeration(self, name: str) -> bool: ...
    def add_typedef(self, typedef: NewTypeDef) -> FidlTypeDef: ...
    def remove_typedef(self, name: str) -> bool: ...
    def insert_member_at(
        self, index: int, member: NewInterfaceMember
    ) -> FidlNode:
        """Insert a member at a position, rather than appending.

        The index is clamped to the end of the list.
        """

    def remove_member_at(self, index: int) -> bool: ...
    def move_member(self, from_index: int, to_index: int) -> bool: ...

class FidlFile:
    file_path: Optional[str]
    id: int
    """The id of the file node itself. The file is a node like any other."""
    kind: str
    node_path: FidlNodePath
    """Always empty: the file is the root."""
    span: Optional[tuple[int, int]]
    node_count: int
    """How many nodes have been given an id."""
    package: Optional[FidlPackage]
    namespaces: list[FidlImportNamespace]
    import_models: list[FidlImportModel]
    interfaces: list[FidlInterface]
    type_collections: list[FidlTypeCollection]
    header_comments: list[FidlComment]
    """Comments at the top of the file, before anything is declared."""
    member_count: int

    def __init__(self, filepath: str) -> None:
        """Parses a Fidl file at filepath."""

    @staticmethod
    def new_from_string(file_string: str) -> FidlFile:
        """Parses Fidl source held in a string."""

    def is_valid(self) -> bool:
        """Always true. A file cannot be removed from itself."""

    # ---- reading the whole tree ----

    def nodes(self) -> FidlNodes:
        """Every node in the file, pre-order, starting with the file itself."""

    def descendants(self) -> FidlNodes:
        """Every node beneath the file, pre-order, excluding the file itself."""

    def get(self, id: int) -> Optional[FidlNode]:
        """The node with this id, or None if it is not in the tree."""

    def path_of(self, node: Union[FidlNode, int]) -> Optional[FidlNodePath]:
        """The path to a node. Accepts any handle, or a bare id."""

    def at_path(self, path: FidlNodePath) -> Optional[FidlNode]:
        """The node at this path, or None if nothing is there."""

    # ---- structural mutation ----

    def add_package(self, package: NewPackage) -> FidlPackage:
        """Add a package.

        Raises ValueError if the file already has one — the grammar permits
        exactly one.
        """

    def remove_package(self) -> bool:
        """Remove the package. Returns whether there was one.

        The grammar requires a package, so a file without one cannot be read
        back — remove it only in order to replace it.
        """

    def add_import_model(self, import_: NewImportModel) -> FidlImportModel:
        """Add an `import model "..."`, placed after the package and any existing
        imports so that the output still parses."""

    def remove_import_model(self, file_path: Path) -> bool: ...
    def add_import_namespace(
        self, namespace: NewImportNamespace
    ) -> FidlImportNamespace:
        """Add an `import a.b.* from "..."`, placed with the other imports."""

    def remove_import_namespace(self, from_: Path) -> bool: ...
    def add_interface(self, interface: NewInterface) -> FidlInterface:
        """Add an interface. Raises ValueError if one of that name is there."""

    def remove_interface(self, name: str) -> bool: ...
    def add_type_collection(
        self, type_collection: NewTypeCollection
    ) -> FidlTypeCollection:
        """Add a type collection. Raises ValueError if one of that name is
        there."""

    def remove_type_collection(self, name: str) -> bool: ...
    def push_comment(self, comment: NewComment) -> FidlComment:
        """Append a free-floating comment."""

    def remove_member_at(self, index: int) -> bool: ...
    def move_member(self, from_index: int, to_index: int) -> bool: ...

    # ---- output ----

    def to_fidl(self, preserve: bool = False) -> str:
        """The file rendered back to .fidl text.

        With `preserve=True`, subtrees that have not been touched come back as the
        exact bytes they were read from, so an edit shows up as a minimal diff.
        Anything constructed or modified is formatted either way.
        """

    def save(self, preserve: bool = False) -> None:
        """Writes the file back to where it was read from.

        Raises ValueError if the file was parsed from a string.
        """

    def write_to(self, path: Path, preserve: bool = False) -> None:
        """Writes the file to `path`."""

    def validate(self) -> list[str]:
        """Problems with the file, as readable strings. Empty means sound."""

    def diff(
        self,
        other: FidlFile,
        ignore_comments: bool = False,
        ignore_layout: bool = True,
        ignore_order: bool = False,
    ) -> list[FidlChange]:
        """What changed between this file and `other`.

        Defaults: comments and ordering count, blank-line layout does not. Pass
        all three as True for a meaning-only comparison.
        """
