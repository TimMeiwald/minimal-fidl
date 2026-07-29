"""Reading the tree as a tree: ids, paths, traversal, and output modes."""

import pytest

from franca_idl import (
    FidlAnnotation,
    FidlComment,
    FidlFile,
    FidlInterface,
    FidlMethod,
    FidlNodePath,
    FidlNodes,
    FidlParamList,
    FidlVersion,
    NewComment,
    NewMethod,
    NewParameter,
    StaleNodeError,
)

SRC = """package org.test
import model "other.fidl"
import org.x.* from "base.fidl"

// a free-floating comment
<** @description: the greeter **>
interface Greeter {
    version {
        major 1
        minor 0
    }

    // documents play
    method play {
        in {
            <** @description: which track **>
            UInt32 track
        }
        out {
            Boolean ok
        }
    }

    attribute Boolean muted
    struct Info {
        UInt32 a
        String b
    }
    enumeration State {
        STOPPED
        PLAYING = 5
    }
    typedef Duration is UInt32
}

typeCollection Types {
    typedef Id is UInt32
}
"""


def file() -> FidlFile:
    return FidlFile.new_from_string(SRC)


def test_traversal_reaches_every_kind_of_node():
    f = file()
    kinds = {node.kind for node in f.nodes()}
    for expected in [
        "file",
        "package",
        "import_model",
        "import_namespace",
        "comment",
        "annotation",
        "interface",
        "version",
        "method",
        "param_list",
        "variable_declaration",
        "attribute",
        "struct",
        "enum",
        "enum_value",
        "typedef",
        "type_collection",
    ]:
        assert expected in kinds, f"traversal missed {expected}"


def test_nodes_starts_with_the_file_and_descendants_does_not():
    f = file()
    everything = list(f.nodes())
    beneath = list(f.descendants())

    assert everything[0].kind == "file"
    assert isinstance(everything[0], FidlFile)
    assert len(everything) == len(beneath) + 1
    assert all(node.kind != "file" for node in beneath)


def test_a_traversal_is_a_lazy_iterator_with_a_length():
    f = file()
    nodes = f.descendants()
    assert isinstance(nodes, FidlNodes)
    assert iter(nodes) is nodes
    total = len(nodes)
    assert total > 20

    first = next(iter(nodes))
    assert first.kind == "package"
    # Iteration is stateful: the rest follows the one already taken.
    assert len(list(nodes)) == total - 1


def test_handles_from_a_traversal_are_of_the_matching_class():
    f = file()
    by_kind = {}
    for node in f.nodes():
        by_kind.setdefault(node.kind, node)

    assert isinstance(by_kind["interface"], FidlInterface)
    assert isinstance(by_kind["method"], FidlMethod)
    assert isinstance(by_kind["param_list"], FidlParamList)
    assert isinstance(by_kind["annotation"], FidlAnnotation)
    assert isinstance(by_kind["comment"], FidlComment)
    assert isinstance(by_kind["version"], FidlVersion)


def test_descendants_of_a_node_stay_beneath_it():
    f = file()
    iface = f.interfaces[0]
    kinds = [node.kind for node in iface.descendants()]

    assert "package" not in kinds
    assert "type_collection" not in kinds
    assert kinds.count("method") == 1
    # Trivia first, then annotations, then the version, then members in order.
    # The comment above the annotation block binds to the interface as leading
    # trivia, so it is the interface's own child rather than a file member.
    assert kinds[:3] == ["comment", "annotation", "version"]


def test_every_id_resolves_and_ids_are_unique():
    f = file()
    ids = [node.id for node in f.nodes()]
    assert len(set(ids)) == len(ids)

    for node in f.nodes():
        resolved = f.get(node.id)
        assert resolved is not None
        assert resolved.id == node.id
        assert resolved.kind == node.kind


def test_an_unknown_id_resolves_to_none():
    f = file()
    highest = max(node.id for node in f.nodes())
    assert f.get(highest + 1) is None


def test_node_count_matches_the_tree():
    f = file()
    assert f.node_count == len(list(f.nodes()))


def test_paths_round_trip_through_the_tree():
    f = file()
    for node in f.nodes():
        path = f.path_of(node)
        assert path is not None, f"no path for {node.kind}"
        found = f.at_path(path)
        assert found is not None, f"path {path} did not resolve back"
        assert found.id == node.id, f"path {path} resolved to a different node"


def test_paths_read_sensibly_and_carry_their_segments():
    f = file()
    play = f.interfaces[0].methods[0]
    path = play.node_path

    assert isinstance(path, FidlNodePath)
    assert str(path) == "interface(Greeter)/method(play)"
    assert len(path) == 2
    assert [(s.kind, s.name) for s in path.segments] == [
        ("interface", "Greeter"),
        ("method", "play"),
    ]
    assert path.segments[0].index is None

    # The file root is the empty path.
    assert str(f.node_path) == "<file>"
    assert len(f.node_path) == 0


def test_anonymous_nodes_are_addressed_by_position():
    # Annotations do have names, so the anonymous case is a parameter list: there
    # is nothing to call it but its position.
    f = file()
    inputs = f.interfaces[0].methods[0].inputs
    segment = inputs.node_path.segments[-1]
    assert segment.kind == "param_list"
    assert segment.name is None
    # The index counts every child, trivia included, so it is a position in the
    # method rather than "the first parameter list".
    assert segment.index is not None
    assert f.at_path(inputs.node_path).id == inputs.id


def test_paths_compare_and_hash_by_value():
    f = file()
    play = f.interfaces[0].methods[0]
    assert play.node_path == f.path_of(play)
    assert len({play.node_path, f.path_of(play)}) == 1
    assert play.node_path != f.interfaces[0].node_path


def test_path_of_accepts_a_bare_id_too():
    f = file()
    play = f.interfaces[0].methods[0]
    assert str(f.path_of(play.id)) == "interface(Greeter)/method(play)"


def test_a_path_to_a_removed_node_stops_resolving():
    f = file()
    play = f.interfaces[0].methods[0]
    path = play.node_path
    assert f.at_path(path) is not None

    f.interfaces[0].remove_method("play")
    assert f.at_path(path) is None
    with pytest.raises(StaleNodeError):
        _ = play.name


def test_spans_point_into_the_original_source():
    f = file()
    iface = f.interfaces[0]
    start, end = iface.span
    # The span opens at the annotation block, which is part of the declaration.
    text = SRC[start:end]
    assert text.startswith("<** @description")
    assert "interface Greeter {" in text
    assert text.endswith("}")

    # A constructed node has no original text.
    added = iface.add_method(NewMethod("stop"))
    assert added.span is None


def test_nodes_start_clean_and_editing_marks_them():
    f = file()
    iface = f.interfaces[0]
    assert iface.is_dirty is False
    assert iface.methods[0].is_dirty is False

    iface.methods[0].name = "start"
    assert iface.methods[0].is_dirty is True
    assert f.interfaces[0].attributes[0].is_dirty is False


def test_comments_are_reachable_as_nodes():
    f = file()
    comments = [node for node in f.nodes() if node.kind == "comment"]
    texts = sorted(c.text for c in comments)
    assert texts == [" a free-floating comment", " documents play"]

    # And through the node they are bound to.
    play = f.interfaces[0].methods[0]
    assert [c.text for c in play.leading_comments] == [" documents play"]
    assert play.trailing_comments == []


def test_format_output_is_stable_and_reparses():
    f = file()
    printed = f.to_fidl()
    again = FidlFile.new_from_string(printed)
    assert again.to_fidl() == printed
    assert again.validate() == []


def test_preserve_returns_untouched_text_verbatim():
    f = file()
    # An unedited file comes back as it went in, edges aside — the printer owns
    # the trailing newline in either mode.
    assert f.to_fidl(preserve=True) == SRC


def test_preserve_only_reformats_what_changed():
    f = file()
    f.interfaces[0].add_method(NewMethod("stop", inputs=[NewParameter("f", "Boolean")]))
    preserved = f.to_fidl(preserve=True)

    # The edited interface is re-laid-out, so the new method is there...
    assert "method stop" in preserved
    # ...while an untouched sibling keeps its original text exactly.
    assert 'import org.x.* from "base.fidl"' in preserved
    assert "typeCollection Types {\n    typedef Id is UInt32\n}" in preserved
    assert FidlFile.new_from_string(preserved).validate() == []


def test_write_to_and_save_honour_the_mode(tmp_path):
    f = file()
    formatted = tmp_path / "formatted.fidl"
    preserved = tmp_path / "preserved.fidl"
    f.write_to(formatted)
    f.write_to(preserved, preserve=True)

    assert preserved.read_text() == SRC
    assert formatted.read_text() == f.to_fidl()

    reloaded = FidlFile(str(preserved))
    reloaded.interfaces[0].add_leading_comment(NewComment(" touched"))
    reloaded.save(preserve=True)
    assert "// touched" in preserved.read_text()


def test_the_file_answers_the_same_questions_as_any_other_node():
    # `nodes()` yields the file first, so a uniform walk has to work on it.
    f = file()
    assert f.kind == "file"
    assert f.id > 0
    assert f.is_valid() is True
    assert f.span is not None
    # package, two imports, the interface and the type collection. The comment is
    # the interface's leading trivia, not a member of the file.
    assert f.member_count == 5
