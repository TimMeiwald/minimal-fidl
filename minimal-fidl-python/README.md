# Python Franca IDL tooling.
N.B Not everything is implemented. Feel free to raise an issue if you need something. I'll mark this package as unmaintained if I stop maintaining it. 

For now it supports parsing FIDL 0.12 files that use  
Enumerations  
Interfaces  
Type Collections  
Typedefs  
Attributes  
Methods  
Hex/Dec/Binary Values  
Annotation blocks  

It does not support most keywords like extend, fire and forget etc. Though they can be easily added so ask if needed.

Nor does it support state machines or mathematical expressions. 

It does not support FDEPL files as of now. 

# Basic Usage
```python
from franca_idl import FidlFile, load_fidl_project
from pathlib import Path

# To get and parse all .fidl files in a directory
result: list[FidlFile] = load_fidl_project(Path("<path_to_directory_with_fidl_files>"))

# To get and parse one fidl file
fidl_file: FidlFile = FidlFile("<path_to_fidl_file>")

# Get the name and version of each interface in a given file.
for i in fidl_file.interfaces:
    print(f"{i.name}, {i.version}")
    print(f"Type: {type(i)}")
```

