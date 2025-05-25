# Should provide typestub for franca_idl_rs
from typing import Optional
from pathlib import Path

def _respond_42() -> int:
    """
    Responds with 42

    This is solely a test function for the package to ensure
    
    Basic Rust-Python functionality. 

    :return: Returns 42
    """

class FidlTypeCollection:
    annotations: list[FidlAnnotation]
    name: str
    version: Optional[FidlVersion]
    typedefs: list[FidlTypeDef]
    structures: list[FidlStructure]
    enumerations: list[FidlEnumeration]

class FidlEnumValue:
    annotations: list[FidlAnnotation]
    name: str
    value: Optional[int]

class FidlEnumeration:
    annotations: list[FidlAnnotation]
    name: str
    values: list[FidlEnumValue]

class FidlMethod:
    annotations: list[FidlAnnotation]
    name: str
    input_parameters: list[FidlVariableDeclaration]
    output_parameters: list[FidlVariableDeclaration]

class FidlTypeDef:
    annotations: list[FidlAnnotation]
    name: str
    type_name: str
    is_array: bool


class FidlVariableDeclaration:
    annotations: list[FidlAnnotation]
    name: str
    type_name: str
    is_array: bool

class FidlStructure:
    annotations: list[FidlAnnotation]
    name: str
    ontents: list[FidlVariableDeclaration]

class FidlAttribute:
    annotations: list[FidlAnnotation]
    name: str
    type_name: str

class FidlPackage:
    path: list[str]

class FidlImportNamespace:
    from_: Path
    imports: list[str]
    wildcard: bool

class FidlImportModel:
    file_path: Path

class FidlAnnotation:
    name: str
    contents: str

class FidlVersion:
    major: Optional[int]
    minor: Optional[int]

class FidlInterface:
    name: str
    version: Optional[FidlVersion]
    annotations: list[FidlAnnotation]
    attributes: list[FidlAttribute]
    structures: list[FidlStructure]
    typedefs: list[FidlTypeDef]
    methods: list[FidlMethod]
    enumerations: list[FidlEnumeration]

class FidlFile:
    package: Optional[FidlPackage]
    namespaces: list[FidlImportNamespace]
    import_models: list[FidlImportModel]
    interfaces: list[FidlInterface]
    type_collections: list[FidlTypeCollection]

    def __init__(self, filepath: str) -> None:
        '''Parses a Fidl file at filepath'''