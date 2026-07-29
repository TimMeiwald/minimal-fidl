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

# Editing a file

Every object you read out of a file is a *handle* — a reference to the file plus a
node id — so reading is cheap and edits are visible through handles you already
hold. Nodes you want to *create* are described by a `New*` class:

```python
from franca_idl import FidlFile, NewMethod, NewParameter

f = FidlFile("player.fidl")
iface = f.interfaces[0]

play = iface.add_method(NewMethod("play",
                                 inputs=[NewParameter("track", "UInt32")],
                                 outputs=[NewParameter("ok", "Boolean")]))
play.set_annotation("description", " start playback")
iface.remove_method("stop")

print(f.validate())        # [] means the file is sound
f.save(preserve=True)      # untouched parts of the file come back byte for byte
```

`preserve=True` reprints only what you changed, so an edit shows up as a minimal
diff in a file people also hand-edit. `to_fidl()` gives you the text instead of
writing it.

# Comparing two revisions

```python
old = FidlFile("player.fidl")
new = FidlFile("player.new.fidl")

for change in old.diff(new):
    if change.change_type == "removed":
        print(f"breaking: {change}")
```

Each change carries a `change_type` (`"added"`, `"removed"`, `"modified"`,
`"moved"`), the `path` to the node, and whichever of `name`, `field`, `before`,
`after`, `from_index` and `to_index` apply.

# Walking the tree

```python
for node in f.nodes():
    if node.kind != "file" and node.annotation("deprecated"):
        print(node.node_path)
```

Comments and annotations are nodes too, so nothing in the file is invisible, and a
file read in and written back out keeps its comments and blank-line grouping.

