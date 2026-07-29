"""The type stub has to match the module.

`franca_idl.pyi` is written by hand and nothing in the build checks it, so a new
method reaches Python with no type information and nobody notices. This walks the
stub with `ast` and compares it to the real classes in both directions: a stub
entry that does not exist is a lie, and a public member that is not in the stub is
a gap.
"""

import ast
from pathlib import Path

import pytest

import franca_idl

STUB = Path(__file__).resolve().parent.parent / "franca_idl.pyi"

# Inherited from `object` or supplied by PyO3 for every class; not worth stubbing.
IGNORED = {
    "__class__",
    "__delattr__",
    "__dict__",
    "__dir__",
    "__doc__",
    "__eq__",
    "__format__",
    "__ge__",
    "__getattribute__",
    "__getstate__",
    "__gt__",
    "__hash__",
    "__init__",
    "__init_subclass__",
    "__le__",
    "__lt__",
    "__module__",
    "__ne__",
    "__new__",
    "__reduce__",
    "__reduce_ex__",
    "__repr__",
    "__setattr__",
    "__sizeof__",
    "__str__",
    "__subclasshook__",
    "__iter__",
    "__next__",
    "__len__",
}


def stub_classes() -> dict[str, tuple[list[str], set[str]]]:
    """Every class in the stub, as `name -> (base names, member names)`."""
    tree = ast.parse(STUB.read_text())
    classes: dict[str, tuple[list[str], set[str]]] = {}
    for node in tree.body:
        if not isinstance(node, ast.ClassDef):
            continue
        bases = [b.id for b in node.bases if isinstance(b, ast.Name)]
        members: set[str] = set()
        for item in node.body:
            if isinstance(item, ast.AnnAssign) and isinstance(item.target, ast.Name):
                members.add(item.target.id)
            elif isinstance(item, (ast.FunctionDef, ast.AsyncFunctionDef)):
                members.add(item.name)
        classes[node.name] = (bases, members)
    return classes


def inherited_members(name: str, classes: dict) -> set[str]:
    """A stub class's own members plus everything it inherits."""
    bases, members = classes[name]
    total = set(members)
    for base in bases:
        if base in classes:
            total |= inherited_members(base, classes)
    return total


def documented_and_real() -> list[tuple[str, set[str], set[str]]]:
    classes = stub_classes()
    out = []
    for name in classes:
        if name.startswith("_"):
            continue  # the shared bases are a stub-only convenience
        runtime = getattr(franca_idl, name, None)
        if not isinstance(runtime, type):
            continue  # a type alias, not a class
        if issubclass(runtime, BaseException):
            continue  # its surface is Python's, not ours
        real = {
            attribute
            for attribute in dir(runtime)
            if attribute not in IGNORED and not attribute.startswith("_")
        }
        out.append((name, inherited_members(name, classes), real))
    return out


def test_the_stub_covers_every_class_the_module_exports():
    exported = {
        name
        for name, value in vars(franca_idl).items()
        if isinstance(value, type)
        and not name.startswith("_")
        and name.startswith(("Fidl", "New"))
    }
    missing = exported - set(stub_classes())
    assert not missing, f"classes exported but not in the stub: {sorted(missing)}"


@pytest.mark.parametrize("name,documented,real", documented_and_real(), ids=lambda x: x if isinstance(x, str) else "")
def test_the_stub_documents_exactly_what_exists(name, documented, real):
    invented = {m for m in documented if not m.startswith("__")} - real
    assert not invented, f"{name}: in the stub but not on the class: {sorted(invented)}"

    undocumented = real - documented
    assert not undocumented, f"{name}: on the class but not in the stub: {sorted(undocumented)}"
