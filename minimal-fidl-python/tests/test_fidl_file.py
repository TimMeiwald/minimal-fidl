from franca_idl import FidlFile, load_fidl_project
from pathlib import Path
'''
Turn on full type checking with pylance
To check that everything has correct type hints. 
'''

def test_thing():
    fidl_file: FidlFile = FidlFile("../minimal-fidl-parser/tests/grammar_test_files/05-CoverageInterface.fidl")
    print(f"{fidl_file}")
    for i in fidl_file.interfaces:
        print(f"{i.name}, {i.version}")
        print(f"Type: {type(i)}")

def test_thing2():
    fidl_file: FidlFile = FidlFile("../minimal-fidl-parser/tests/grammar_test_files/05-CoverageInterface.fidl")
    for i in fidl_file.interfaces:
        for j in i.methods:
            print(f"Type: {type(j)}")
            print(j)

def test_project():
    result = load_fidl_project(Path("../minimal-fidl-python/tests/grammar_test_files/"))
    assert result != None
    print(f"Files Parsed: {len(result)}")
    for fidl_file in result:
        print(fidl_file.file_path)
        for iface in fidl_file.interfaces:
            print(f"Version: {iface.version}")


    for fidl_file in result:
        print(fidl_file.__str__() + "\n\n\n")
    assert 0 == 1