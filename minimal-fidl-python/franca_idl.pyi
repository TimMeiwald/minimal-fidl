# Type stub for franca_idl.
#
# Every object below is a *handle*: a shared reference to the parsed file plus a
# node id. Attribute access resolves the id rather than copying a subtree, so
# reading `file.interfaces` is cheap no matter how large the file is.
#
# A handle stops working if its node is removed from the tree; accessing one then
# raises StaleNodeError. Ids are not stable across a reparse.
from typing import Optional
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

class _Node:
    """Common to every handle. Not instantiable directly."""

    id: int
    """The node's stable id within its file. Not stable across a reparse."""

    def is_valid(self) -> bool:
        """False once the node has been removed from the tree."""

class FidlTypeCollection(_Node):
    annotations: list[FidlAnnotation]
    name: str
    version: Optional[FidlVersion]
    typedefs: list[FidlTypeDef]
    structures: list[FidlStructure]
    enumerations: list[FidlEnumeration]

class FidlEnumValue(_Node):
    annotations: list[FidlAnnotation]
    name: str
    value: Optional[int]

class FidlEnumeration(_Node):
    annotations: list[FidlAnnotation]
    name: str
    values: list[FidlEnumValue]

class FidlMethod(_Node):
    annotations: list[FidlAnnotation]
    name: str
    input_parameters: list[FidlVariableDeclaration]
    output_parameters: list[FidlVariableDeclaration]

class FidlTypeDef(_Node):
    annotations: list[FidlAnnotation]
    name: str
    type_name: str
    is_array: bool

class FidlVariableDeclaration(_Node):
    annotations: list[FidlAnnotation]
    name: str
    type_name: str
    is_array: bool

class FidlStructure(_Node):
    annotations: list[FidlAnnotation]
    name: str
    fields: list[FidlVariableDeclaration]
    contents: list[FidlVariableDeclaration]
    """Deprecated alias for `fields`."""

class FidlAttribute(_Node):
    annotations: list[FidlAnnotation]
    name: str
    type_name: str

class FidlPackage(_Node):
    path: list[str]

class FidlImportNamespace(_Node):
    from_: Path
    imports: list[str]
    wildcard: bool

class FidlImportModel(_Node):
    file_path: Path

class FidlAnnotation(_Node):
    name: str
    contents: str

class FidlVersion(_Node):
    major: Optional[int]
    minor: Optional[int]

class FidlInterface(_Node):
    name: str
    version: Optional[FidlVersion]
    annotations: list[FidlAnnotation]
    attributes: list[FidlAttribute]
    structures: list[FidlStructure]
    typedefs: list[FidlTypeDef]
    methods: list[FidlMethod]
    enumerations: list[FidlEnumeration]

class FidlFile:
    file_path: Optional[str]
    package: Optional[FidlPackage]
    namespaces: list[FidlImportNamespace]
    import_models: list[FidlImportModel]
    interfaces: list[FidlInterface]
    type_collections: list[FidlTypeCollection]

    def __init__(self, filepath: str) -> None:
        """Parses a Fidl file at filepath."""

    @staticmethod
    def new_from_string(file_string: str) -> FidlFile:
        """Parses Fidl source held in a string."""

    def to_fidl(self) -> str:
        """The file rendered back to .fidl text."""

    def save(self) -> None:
        """Writes the file back to where it was read from.

        Raises ValueError if the file was parsed from a string.
        """

    def write_to(self, path: Path) -> None:
        """Writes the file to `path`."""

    def validate(self) -> list[str]:
        """Problems with the file, as readable strings. Empty means sound."""
