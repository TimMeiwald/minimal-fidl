from franca_idl import FidlFile

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
    assert 0 == 1

def test_thing2():
    fidl_file: FidlFile = FidlFile("../minimal-fidl-parser/tests/grammar_test_files/05-CoverageInterface.fidl")
    print(f"{fidl_file}")
    for i in fidl_file.interfaces:
        for j in i.methods:
            print(f"{j.name}, {j}")
            print(f"Type: {type(j)}")
    assert 0 == 1

def test_project():
    pass