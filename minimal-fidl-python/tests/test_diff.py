"""Structural diff between two files.

The use case this exists for is breaking-change detection: filter the changes by
`change_type` and look at what left the interface.
"""

from franca_idl import FidlChange, FidlFile, FidlNodePath, NewMethod, NewParameter

BEFORE = """package org.test
interface Greeter {
    method play {in {UInt32 track}}
    method stop {}
    attribute Boolean muted
    enumeration State {STOPPED PLAYING}
}
"""

AFTER = """package org.test
interface Greeter {
    method stop {}
    method play {in {UInt64 track}}
    attribute Boolean silenced
    enumeration State {STOPPED PLAYING PAUSED}
}
"""


def files():
    return FidlFile.new_from_string(BEFORE), FidlFile.new_from_string(AFTER)


def test_a_file_does_not_differ_from_itself():
    before, _ = files()
    assert before.diff(before) == []
    # Including against a separately parsed copy of the same text — the sibling
    # match key counts occurrences, so same-named siblings do not collide.
    assert before.diff(FidlFile.new_from_string(BEFORE)) == []


def test_changes_are_typed_and_carry_a_path():
    before, after = files()
    changes = before.diff(after)
    assert changes
    assert all(isinstance(c, FidlChange) for c in changes)

    for change in changes:
        assert change.change_type in {"added", "removed", "modified", "moved"}
        assert isinstance(change.path, FidlNodePath)
        assert change.kind
        assert str(change)  # a readable one-liner


def test_the_breaking_change_query_works():
    # The reason this is a flat class with a `change_type` string rather than one
    # class per variant.
    before, after = files()
    changes = before.diff(after)

    removed = [c for c in changes if c.change_type == "removed"]
    assert [(c.kind, c.name) for c in removed] == [("attribute", "muted")]

    added = [c for c in changes if c.change_type == "added"]
    assert ("attribute", "silenced") in [(c.kind, c.name) for c in added]
    assert ("enum_value", "PAUSED") in [(c.kind, c.name) for c in added]


def test_a_modified_field_reports_before_and_after():
    before, after = files()
    (modified,) = [
        c
        for c in before.diff(after)
        if c.change_type == "modified" and c.kind == "variable_declaration"
    ]
    assert modified.field == "type"
    assert modified.before == "UInt32"
    assert modified.after == "UInt64"
    assert str(modified.path) == (
        "interface(Greeter)/method(play)/param_list[0]/variable_declaration(track)"
    )
    # The fields that do not apply to a modification are absent, not empty.
    assert modified.name is None
    assert modified.from_index is None
    assert modified.to_index is None


def test_reordering_reads_as_a_move():
    before, after = files()
    moves = [c for c in before.diff(after) if c.change_type == "moved"]
    assert moves
    play = next(c for c in moves if c.name is None and "play" in str(c.path))
    assert play.from_index != play.to_index
    assert play.before is None

    # ...unless ordering is explicitly not the point.
    assert not [
        c for c in before.diff(after, ignore_order=True) if c.change_type == "moved"
    ]


def test_comments_count_unless_they_are_told_not_to():
    plain = FidlFile.new_from_string("package org.test\ninterface A {}\n")
    commented = FidlFile.new_from_string(
        "package org.test\n// added a note\ninterface A {}\n"
    )

    changes = plain.diff(commented)
    assert [c.kind for c in changes] == ["comment"]
    assert plain.diff(commented, ignore_comments=True) == []


def test_layout_is_ignored_by_default():
    tight = FidlFile.new_from_string(
        "package org.test\ninterface A {attribute UInt32 x\nattribute UInt32 y}\n"
    )
    spaced = FidlFile.new_from_string(
        "package org.test\ninterface A {attribute UInt32 x\n\n\nattribute UInt32 y}\n"
    )

    assert tight.diff(spaced) == []
    layout = tight.diff(spaced, ignore_layout=False)
    assert [c.field for c in layout] == ["blank_lines_before"]


def test_a_diff_sees_an_edit_made_through_the_api():
    before, _ = files()
    after = FidlFile.new_from_string(BEFORE)
    after.interfaces[0].add_method(
        NewMethod("pause", inputs=[NewParameter("hard", "Boolean")])
    )
    after.interfaces[0].remove_method("stop")

    kinds = {(c.change_type, c.kind, c.name) for c in before.diff(after)}
    assert ("added", "method", "pause") in kinds
    assert ("removed", "method", "stop") in kinds


def test_formatting_is_not_a_semantic_change():
    # The strongest single property of the pair: reprinting a file must not alter
    # what it means.
    before, _ = files()
    reformatted = FidlFile.new_from_string(before.to_fidl())
    assert before.diff(reformatted, ignore_layout=True, ignore_order=True) == []
