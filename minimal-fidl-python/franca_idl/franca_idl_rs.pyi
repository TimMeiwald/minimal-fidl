# Should provide typestub for franca_idl_rs
from typing import Optional


def _respond_42() -> int:
    """
    Responds with 42

    This is solely a test function for the package to ensure
    
    Basic Rust-Python functionality. 

    :return: Returns 42
    """

class FidlAnnotation:
    name: str
    contents: str

class FidlVersion:
    major: Optional[int]
    minor: Optional[int]

class FidlInterface:
    name: str
    version: Optional[FidlVersion]

class FidlFile:
    interfaces: list[FidlInterface]

    def __init__(self, filepath: str) -> None:
        '''Parses a Fidl file at filepath'''