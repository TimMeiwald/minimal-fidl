from franca_idl import FidlFile

def test_thing():
    fidl_file: FidlFile = FidlFile("../minimal-fidl-parser/tests/grammar_test_files/05-CoverageInterface.fidl")
    print(f"{fidl_file}")
    for i in fidl_file.interfaces:
        print(f"{i.name}, {i.version}")
    assert 0 == 1
