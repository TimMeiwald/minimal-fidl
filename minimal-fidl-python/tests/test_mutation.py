"""Mutation through handles: resolution by id, insertion, and id assignment.

The regression at the top is the reason `get_mut` exists. Everything below it
depends on the same property: a handle names one node, and mutating through it
must reach that node and no other.
"""

import pytest

from franca_idl import (
    FidlAnnotation,
    FidlAttribute,
    FidlComment,
    FidlEnumeration,
    FidlFile,
    FidlMethod,
    FidlStructure,
    FidlTypeCollection,
    FidlTypeDef,
    FidlVariableDeclaration,
    NewAnnotation,
    NewAttribute,
    NewComment,
    NewEnumeration,
    NewEnumValue,
    NewImportModel,
    NewImportNamespace,
    NewInterface,
    NewMethod,
    NewPackage,
    NewParameter,
    NewStructure,
    NewTypeCollection,
    NewTypeDef,
    NewVersion,
    StaleNodeError,
)

BASE = """package org.test
interface Greeter {
    version {major 1 minor 0}
    method play {in {UInt32 track} out {Boolean ok}}
    attribute Boolean muted
    struct Info {UInt32 a String b}
    enumeration State {STOPPED PLAYING = 5}
    typedef Duration is UInt32
}
"""

DUPLICATES = """package org.test
interface A {method play {} method keepme {}}
interface A {method play {}}
"""


def base() -> FidlFile:
    return FidlFile.new_from_string(BASE)


def every_node_resolves(f: FidlFile) -> None:
    """Every node in the tree has an id that `get` can resolve.

    A node inserted without ids being handed out afterwards exists in the tree
    and prints correctly but cannot be addressed at all, which is the failure
    mode this guards.
    """
    for node in f.nodes():
        assert node.id > 0, f"{node.kind} has no id"
        assert f.get(node.id) is not None, f"{node.kind} id {node.id} did not resolve"


# ---- the regression -----------------------------------------------------


def test_mutation_reaches_the_node_the_handle_names_not_the_first_of_that_name():
    # Duplicate names are a validate() diagnostic rather than a parse error, so a
    # file can hold two interfaces called A. The mutation used to look its own
    # name up again and get the first one back.
    f = FidlFile.new_from_string(DUPLICATES)
    first, second = f.interfaces

    assert second.remove_method("play") is True

    assert [m.name for m in first.methods] == ["play", "keepme"]
    assert [m.name for m in second.methods] == []


def test_the_quiet_variant_of_the_same_bug():
    # The mirror image: removing from the *first* of two same-named interfaces
    # used to succeed by accident, which hid the problem.
    f = FidlFile.new_from_string(DUPLICATES)
    first, second = f.interfaces

    assert first.remove_method("keepme") is True
    assert [m.name for m in first.methods] == ["play"]
    assert [m.name for m in second.methods] == ["play"]


def test_adding_reaches_the_right_duplicate_too():
    f = FidlFile.new_from_string(DUPLICATES)
    first, second = f.interfaces

    second.add_method(NewMethod("added"))

    assert [m.name for m in first.methods] == ["play", "keepme"]
    assert [m.name for m in second.methods] == ["play", "added"]


def test_mutating_a_removed_node_raises_rather_than_hitting_a_namesake():
    f = FidlFile.new_from_string(DUPLICATES)
    first, second = f.interfaces
    assert f.remove_interface("A") is True  # removes the first

    assert not first.is_valid()
    with pytest.raises(StaleNodeError):
        first.remove_method("play")
    with pytest.raises(StaleNodeError):
        first.add_method(NewMethod("nope"))
    # The survivor is untouched, and still reachable.
    assert second.is_valid()
    assert [m.name for m in second.methods] == ["play"]


# ---- insertion ----------------------------------------------------------


def test_add_method_returns_a_usable_handle():
    f = base()
    iface = f.interfaces[0]

    method = iface.add_method(
        NewMethod(
            "stop",
            inputs=[NewParameter("force", "Boolean")],
            outputs=[NewParameter("ok", "Boolean")],
            annotations=[NewAnnotation("description", " stops playback")],
        )
    )

    assert isinstance(method, FidlMethod)
    assert method.is_valid()
    assert method.name == "stop"
    assert [p.name for p in method.input_parameters] == ["force"]
    assert [p.name for p in method.output_parameters] == ["ok"]
    assert method.annotation("description").contents == " stops playback"
    assert [m.name for m in iface.methods] == ["play", "stop"]
    every_node_resolves(f)


def test_an_inserted_subtree_is_addressable_all_the_way_down():
    # DESIGN §7: nodes from builders carry NodeId::UNASSIGNED and are invisible
    # to get() until ids are handed out. Insertion has to do that itself.
    f = base()
    iface = f.interfaces[0]
    method = iface.add_method(
        NewMethod("stop", inputs=[NewParameter("force", "Boolean")])
    )

    every_node_resolves(f)
    (parameter,) = method.input_parameters
    assert f.get(parameter.id).name == "force"
    assert str(parameter.node_path) == (
        "interface(Greeter)/method(stop)/param_list[0]/variable_declaration(force)"
    )


def test_add_every_kind_of_interface_member():
    f = base()
    iface = f.interfaces[0]

    attribute = iface.add_attribute(NewAttribute("volume", "UInt8"))
    typedef = iface.add_typedef(NewTypeDef("Ids", "UInt32", is_array=True))
    structure = iface.add_structure(
        NewStructure("Track", members=[NewParameter("id", "UInt32")])
    )
    enumeration = iface.add_enumeration(
        NewEnumeration("Mode", members=[NewEnumValue("OFF"), NewEnumValue("ON", 2)])
    )

    assert isinstance(attribute, FidlAttribute)
    assert isinstance(typedef, FidlTypeDef)
    assert isinstance(structure, FidlStructure)
    assert isinstance(enumeration, FidlEnumeration)

    assert (attribute.name, attribute.type_name) == ("volume", "UInt8")
    assert typedef.is_array is True
    assert [x.name for x in structure.fields] == ["id"]
    assert [(v.name, v.value) for v in enumeration.values] == [("OFF", None), ("ON", 2)]

    every_node_resolves(f)
    assert f.validate() == []
    assert FidlFile.new_from_string(f.to_fidl()).validate() == []


def test_add_struct_fields_and_method_parameters():
    f = base()
    iface = f.interfaces[0]

    info = iface.structures[0]
    field = info.add_field(NewParameter("c", "Boolean"))
    assert isinstance(field, FidlVariableDeclaration)
    assert [x.name for x in info.fields] == ["a", "b", "c"]
    assert info.remove_field("b") is True
    assert [x.name for x in info.fields] == ["a", "c"]

    play = iface.methods[0]
    play.add_input(NewParameter("volume", "UInt8"))
    play.add_output(NewParameter("started", "Boolean"))
    assert [p.name for p in play.input_parameters] == ["track", "volume"]
    assert [p.name for p in play.output_parameters] == ["ok", "started"]
    assert play.remove_input("track") is True
    assert [p.name for p in play.input_parameters] == ["volume"]

    every_node_resolves(f)


def test_parameter_lists_are_nodes_in_their_own_right():
    f = base()
    play = f.interfaces[0].methods[0]

    assert play.inputs.member_count == 1
    added = play.inputs.add_parameter(NewParameter("volume", "UInt8"))
    assert added.name == "volume"
    assert [p.name for p in play.inputs.parameters] == ["track", "volume"]
    assert play.inputs.remove_parameter("volume") is True
    # An annotation on `in { }` itself, which the grammar allows.
    play.inputs.set_annotation("description", " what to play")
    assert play.inputs.annotation("description") is not None
    assert "in {" in f.to_fidl()


def test_add_enum_values():
    f = base()
    state = f.interfaces[0].enumerations[0]

    added = state.add_value(NewEnumValue("PAUSED", 7))
    assert added.value == 7
    assert [v.name for v in state.values] == ["STOPPED", "PLAYING", "PAUSED"]
    assert state.remove_value("STOPPED") is True
    assert [v.name for v in state.values] == ["PLAYING", "PAUSED"]
    every_node_resolves(f)


def test_duplicate_names_are_rejected_at_the_point_of_insertion():
    f = base()
    iface = f.interfaces[0]

    with pytest.raises(ValueError):
        iface.add_method(NewMethod("play"))
    with pytest.raises(ValueError):
        iface.add_attribute(NewAttribute("muted", "Boolean"))
    with pytest.raises(ValueError):
        f.add_interface(NewInterface("Greeter"))

    # Nothing was left behind by the rejected insertions.
    assert [m.name for m in iface.methods] == ["play"]
    assert f.validate() == []


def test_build_a_whole_interface_from_nothing():
    f = base()
    player = f.add_interface(
        NewInterface(
            "Player",
            version=NewVersion(2, 1),
            annotations=[NewAnnotation("description", " the player")],
            leading_comments=[NewComment(" everything below is generated")],
            members=[
                NewMethod("next", outputs=[NewParameter("ok", "Boolean")]),
                NewComment(" and a comment between members"),
                NewAttribute("shuffle", "Boolean"),
                NewStructure("Entry", members=[NewParameter("id", "UInt32")]),
                NewEnumeration("Repeat", members=[NewEnumValue("NONE")]),
                NewTypeDef("Seconds", "UInt32"),
            ],
        )
    )

    assert player.name == "Player"
    assert (player.version.major, player.version.minor) == (2, 1)
    assert [m.name for m in player.methods] == ["next"]
    assert [a.name for a in player.attributes] == ["shuffle"]
    assert player.annotation("description").contents == " the player"
    assert [c.text for c in player.leading_comments] == [
        " everything below is generated"
    ]
    # Member order is intrinsic to the tree, so the comment stays where it was put.
    assert [n.kind for n in player.descendants()][:1] == ["comment"]

    every_node_resolves(f)
    assert f.validate() == []
    printed = f.to_fidl()
    assert "interface Player" in printed
    assert "// and a comment between members" in printed
    assert FidlFile.new_from_string(printed).validate() == []


def test_type_collections_and_their_members():
    f = base()
    types = f.add_type_collection(
        NewTypeCollection(
            "Types",
            version=NewVersion(1, 0),
            members=[NewTypeDef("Id", "UInt32")],
        )
    )
    assert isinstance(types, FidlTypeCollection)
    assert types.name == "Types"
    assert types.is_anonymous is False

    types.add_structure(NewStructure("Point", members=[NewParameter("x", "UInt32")]))
    types.add_enumeration(NewEnumeration("Kind", members=[NewEnumValue("ONE")]))
    assert [t.name for t in types.typedefs] == ["Id"]
    assert [s.name for s in types.structures] == ["Point"]
    assert [e.name for e in types.enumerations] == ["Kind"]

    assert types.remove_typedef("Id") is True
    assert types.remove_structure("Point") is True
    assert types.remove_enumeration("Kind") is True
    assert types.member_count == 0

    assert f.remove_type_collection("Types") is True
    assert f.type_collections == []
    every_node_resolves(f)


def test_an_anonymous_type_collection_warns_but_is_allowed():
    f = base()
    anonymous = f.add_type_collection(NewTypeCollection())
    assert anonymous.is_anonymous is True
    problems = f.validate()
    assert len(problems) == 1
    assert "no name" in problems[0]


# ---- file-level members -------------------------------------------------


def test_package_and_imports_can_be_added_and_removed():
    f = base()
    assert f.remove_package() is True
    assert f.package is None

    package = f.add_package(NewPackage("org.rebuilt"))
    assert package.path == ["org", "rebuilt"]
    with pytest.raises(ValueError):
        f.add_package(NewPackage("org.again"))

    model = f.add_import_model(NewImportModel("other.fidl"))
    namespace = f.add_import_namespace(NewImportNamespace("org.x", "base.fidl"))
    assert str(model.file_path) == "other.fidl"
    assert namespace.imports == ["org", "x"]
    assert namespace.wildcard is True

    printed = f.to_fidl()
    # The grammar is `package (import)* (interface | typeCollection)*` and it
    # enforces the order, so appending at the end would break the output.
    assert printed.index("import model") < printed.index("interface Greeter")
    FidlFile.new_from_string(printed)  # raises if the order is wrong

    assert f.remove_import_model("other.fidl") is True
    assert f.remove_import_namespace("base.fidl") is True
    assert f.import_models == []
    assert f.namespaces == []
    every_node_resolves(f)


def test_a_package_accepts_either_spelling_of_a_dotted_name():
    f = base()
    f.remove_package()
    assert f.add_package(NewPackage(["org", "listed"])).path == ["org", "listed"]


# ---- ordering and comments ---------------------------------------------


def test_insert_member_at_puts_a_member_where_asked():
    f = base()
    iface = f.interfaces[0]
    before = [n for n in (m.kind for m in iface.descendants())]
    assert iface.member_count == 5

    comment = iface.insert_member_at(1, NewComment(" TODO: revisit"))
    assert isinstance(comment, FidlComment)
    assert comment.text == " TODO: revisit"
    assert iface.member_count == 6

    printed = f.to_fidl()
    assert printed.index("// TODO: revisit") < printed.index("attribute Boolean muted")
    assert printed.index("method play") < printed.index("// TODO: revisit")
    assert before  # the tree was non-trivial to begin with
    every_node_resolves(f)


def test_members_can_be_moved_and_removed_by_position():
    f = base()
    iface = f.interfaces[0]

    assert iface.move_member(4, 0) is True  # the typedef to the front
    kinds = [m.kind for m in iface.descendants()]
    assert kinds[1] == "typedef", kinds
    assert iface.move_member(0, 99) is False, "out of range is a no-op"

    count = iface.member_count
    assert iface.remove_member_at(0) is True
    assert iface.member_count == count - 1
    assert iface.remove_member_at(99) is False


def test_comments_can_be_added_and_read_back():
    f = base()
    iface = f.interfaces[0]

    comment = iface.add_leading_comment(NewComment(" what this interface is for"))
    assert isinstance(comment, FidlComment)
    assert comment.is_block is False
    assert comment.to_source() == "// what this interface is for"
    assert [c.text for c in iface.leading_comments] == [
        " what this interface is for"
    ]

    block = iface.methods[0].add_leading_comment(NewComment(" doc ", block=True))
    assert block.is_block is True
    assert block.to_source() == "/* doc */"

    printed = f.to_fidl()
    assert "// what this interface is for" in printed
    assert "/* doc */" in printed
    every_node_resolves(f)


def test_comment_text_can_be_rewritten_in_place():
    f = FidlFile.new_from_string(
        "package org.test\n// before\ninterface A {}\n"
    )
    (comment,) = [n for n in f.nodes() if n.kind == "comment"]
    comment.text = " after"
    assert "// after" in f.to_fidl()
    assert "// before" not in f.to_fidl()


def test_the_file_takes_a_free_floating_comment():
    f = base()
    comment = f.push_comment(NewComment(" end of file"))
    assert comment.text == " end of file"
    assert f.to_fidl().rstrip().endswith("// end of file")


# ---- scalar fields ------------------------------------------------------


def test_names_and_types_can_be_rewritten():
    f = base()
    iface = f.interfaces[0]

    iface.name = "Greeter2"
    iface.methods[0].name = "start"
    iface.attributes[0].name = "silenced"
    iface.attributes[0].type_name = "UInt8"
    iface.structures[0].fields[0].type_name = "UInt64"
    iface.typedefs[0].is_array = True
    iface.enumerations[0].values[0].value = 3

    assert iface.name == "Greeter2"
    printed = f.to_fidl()
    assert "interface Greeter2" in printed
    assert "method start" in printed
    assert "attribute UInt8 silenced" in printed
    assert "UInt64 a" in printed
    assert "STOPPED = 3" in printed
    assert FidlFile.new_from_string(printed).validate() == []


def test_version_can_be_set_changed_and_dropped():
    f = base()
    iface = f.interfaces[0]

    iface.version.major = 4
    assert iface.version.major == 4

    replaced = iface.set_version(NewVersion(9, 9))
    assert (replaced.major, replaced.minor) == (9, 9)
    assert "major 9" in f.to_fidl()

    assert iface.remove_version() is True
    assert iface.version is None
    assert iface.remove_version() is False
    assert "version" not in f.to_fidl()


def test_annotations_can_be_read_set_and_removed_on_any_annotated_node():
    f = base()
    iface = f.interfaces[0]

    assert iface.annotations == []
    added = iface.set_annotation("description", " the greeter")
    assert isinstance(added, FidlAnnotation)
    assert added.name == "description"
    assert [a.name for a in iface.annotations] == ["description"]

    # Setting an existing name replaces rather than duplicating.
    iface.set_annotation("description", " replaced")
    assert len(iface.annotations) == 1
    assert iface.annotation("description").contents == " replaced"

    # Annotations are nodes: they can be edited through their own handle.
    iface.annotation("description").contents = " edited"
    assert iface.annotation("description").contents == " edited"

    assert iface.remove_annotation("description") is True
    assert iface.remove_annotation("description") is False
    assert iface.annotation("description") is None

    # And the same surface exists further down the tree.
    for node in (
        iface.methods[0],
        iface.attributes[0],
        iface.structures[0],
        iface.structures[0].fields[0],
        iface.enumerations[0],
        iface.enumerations[0].values[0],
        iface.typedefs[0],
    ):
        node.set_annotation("deprecated", " gone soon")
        assert node.annotation("deprecated").contents == " gone soon"

    every_node_resolves(f)
    assert "@deprecated" in f.to_fidl()
    assert FidlFile.new_from_string(f.to_fidl()).validate() == []
