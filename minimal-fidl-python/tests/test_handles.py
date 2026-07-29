"""Phase 7 acceptance for the handle model.

Every object is a shared pointer to the file plus a node id, not a copy of its
subtree. These tests pin the behaviour that follows from that: handles stay valid
across unrelated edits, and a handle to a removed node raises rather than
returning data from the wrong node.
"""

from pathlib import Path

import pytest

from franca_idl import (
    FidlAnnotation,
    FidlFile,
    FidlInterface,
    FidlMethod,
    StaleNodeError,
    load_fidl_project,
)

SRC = """package org.test
interface Greeter {
    version {major 1 minor 0}
    <** @description: plays a track **>
    method play {in {UInt32 track} out {Boolean ok}}
    attribute Boolean muted
    struct Info {UInt32 a String b}
    enumeration State {IDLE PLAYING = 5}
    typedef Duration is UInt32
}
"""


def file() -> FidlFile:
    return FidlFile.new_from_string(SRC)


def test_every_class_is_importable():
    # The handle classes are macro-generated, so registration is easy to lose.
    assert issubclass(StaleNodeError, Exception)
    f = file()
    assert isinstance(f.interfaces[0], FidlInterface)
    assert isinstance(f.interfaces[0].methods[0], FidlMethod)
    assert isinstance(f.interfaces[0].annotations, list)


def test_reading_the_whole_tree():
    f = file()
    (iface,) = f.interfaces
    assert iface.name == "Greeter"
    assert iface.version is not None
    assert (iface.version.major, iface.version.minor) == (1, 0)

    (method,) = iface.methods
    assert method.name == "play"
    assert [p.name for p in method.input_parameters] == ["track"]
    assert [p.type_name for p in method.input_parameters] == ["UInt32"]
    assert [p.name for p in method.output_parameters] == ["ok"]

    (annotation,) = method.annotations
    assert isinstance(annotation, FidlAnnotation)
    assert annotation.name == "description"
    assert annotation.contents.strip() == "plays a track"

    (attribute,) = iface.attributes
    assert (attribute.name, attribute.type_name) == ("muted", "Boolean")

    (struct,) = iface.structures
    assert [x.name for x in struct.fields] == ["a", "b"]
    assert [x.name for x in struct.contents] == ["a", "b"]  # deprecated alias

    (enum,) = iface.enumerations
    assert [(v.name, v.value) for v in enum.values] == [("IDLE", None), ("PLAYING", 5)]

    (typedef,) = iface.typedefs
    assert (typedef.name, typedef.type_name, typedef.is_array) == (
        "Duration",
        "UInt32",
        False,
    )


def test_handles_are_cheap_not_copies():
    # Two reads of the same attribute produce distinct handle objects that point
    # at the same node. If these were deep copies the ids would still match, but
    # the point is that nothing was copied to get them.
    f = file()
    first = f.interfaces[0]
    second = f.interfaces[0]
    assert first is not second
    assert first.id == second.id
    assert first.is_valid() and second.is_valid()


def test_a_handle_survives_unrelated_edits():
    f = file()
    method = f.interfaces[0].methods[0]
    assert method.name == "play"
    # Re-reading the file's interfaces does not disturb existing handles.
    _ = f.interfaces
    assert method.is_valid()
    assert method.name == "play"


def test_a_handle_to_a_removed_node_raises_rather_than_lying():
    # The safety story for handles. Without the guard a stale id could resolve to
    # whatever node later occupies that slot, silently returning wrong data.
    f = file()
    iface = f.interfaces[0]
    method = iface.methods[0]
    assert method.is_valid()
    assert method.name == "play"

    assert iface.remove_method("play") is True

    assert not method.is_valid()
    with pytest.raises(StaleNodeError):
        _ = method.name
    with pytest.raises(StaleNodeError):
        _ = method.input_parameters

    # Removing again reports that there was nothing to remove.
    assert iface.remove_method("play") is False


def test_unrelated_handles_stay_valid_when_a_sibling_is_removed():
    # Ids are stable across sibling removal, which is what makes a handle usable
    # across edits at all.
    f = file()
    iface = f.interfaces[0]
    attribute = iface.attributes[0]
    struct = iface.structures[0]

    assert iface.remove_method("play") is True

    assert attribute.is_valid()
    assert attribute.name == "muted"
    assert struct.is_valid()
    assert [x.name for x in struct.fields] == ["a", "b"]
    assert iface.is_valid()


def test_removal_is_reflected_in_the_printed_file():
    f = file()
    assert "method play" in f.to_fidl()
    f.interfaces[0].remove_method("play")
    assert "method play" not in f.to_fidl()
    # And the result is still valid Fidl.
    assert FidlFile.new_from_string(f.to_fidl()).validate() == []


def test_file_level_members():
    f = FidlFile.new_from_string(
        'package org.test\n'
        'import model "other.fidl"\n'
        'import org.x.* from "base.fidl"\n'
        'interface A {}\n'
        'typeCollection T {}\n'
    )
    assert f.package is not None
    assert f.package.path == ["org", "test"]
    assert [str(m.file_path) for m in f.import_models] == ["other.fidl"]
    (ns,) = f.namespaces
    assert ns.imports == ["org", "x"]
    assert ns.wildcard is True
    assert str(ns.from_) == "base.fidl"
    assert [i.name for i in f.interfaces] == ["A"]
    assert [t.name for t in f.type_collections] == ["T"]


def test_a_string_built_file_has_no_path():
    f = file()
    assert f.file_path is None
    with pytest.raises(ValueError):
        f.save()


def test_round_trip_through_text():
    f = file()
    printed = f.to_fidl()
    again = FidlFile.new_from_string(printed)
    assert again.to_fidl() == printed
    assert [i.name for i in again.interfaces] == ["Greeter"]


def test_write_to_and_reload(tmp_path: Path):
    f = file()
    target = tmp_path / "out.fidl"
    f.write_to(target)
    assert target.exists()
    reloaded = FidlFile(str(target))
    assert reloaded.file_path is not None
    assert [i.name for i in reloaded.interfaces] == ["Greeter"]
    reloaded.save()  # has a path now, so this works


def test_validate_reports_duplicates():
    sound = file()
    assert sound.validate() == []

    dupes = FidlFile.new_from_string(
        "package org.test\ninterface A {}\ninterface A {}\n"
    )
    problems = dupes.validate()
    assert len(problems) == 1
    assert "duplicate interface 'A'" in problems[0]


def test_load_fidl_project(tmp_path: Path):
    (tmp_path / "nested").mkdir()
    (tmp_path / "one.fidl").write_text(SRC)
    (tmp_path / "nested" / "two.fidl").write_text(
        "package org.other\ninterface Other {}\n"
    )
    (tmp_path / "ignored.txt").write_text("not fidl")

    files = load_fidl_project(tmp_path)
    assert len(files) == 2
    names = sorted(i.name for f in files for i in f.interfaces)
    assert names == ["Greeter", "Other"]
    assert all(f.file_path is not None for f in files)


def test_repr_is_informative():
    f = file()
    assert "FidlFile" in repr(f)
    iface = f.interfaces[0]
    assert repr(iface).startswith("<FidlInterface id=")
    assert iface.id > 0
