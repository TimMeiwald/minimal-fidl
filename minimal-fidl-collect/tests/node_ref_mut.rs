//! Mutable resolution by id: `NodeRefMut` and `FidlFile::get_mut`.
//!
//! The point of `get_mut` is that an id names *one* node. Looking a node up by
//! name instead cannot promise that: duplicate names are a `validate()`
//! diagnostic rather than a parse error (`DESIGN.md` §3), so a file can hold two
//! interfaces called `A` and a name lookup will always find the first.

use std::collections::BTreeMap;

use minimal_fidl_collect::{
    Annotated, AstNode, Comment, FidlFile, FidlProject, FileMember, Method, NodeId, NodeRef,
    NodeRefMut,
};

fn parse(src: &str) -> FidlFile {
    FidlProject::generate_file_from_string(src.to_string()).expect("test source should parse")
}

const DUPLICATES: &str = r#"package org.test
interface A {method play {} method keepme {}}
interface A {method play {}}
"#;

const RICH: &str = r#"package org.test
import model "other.fidl"
import org.x.* from "base.fidl"
// free floating
<** @description: the greeter **>
interface Greeter {
    version {major 1 minor 0}
    // documents play
    method play {in {<** @description: which **> UInt32 track} out {Boolean ok}}
    attribute Boolean muted
    struct Info {UInt32 a String b}
    enumeration State {STOPPED PLAYING = 5}
    typedef Duration is UInt32
}
typeCollection Types {
    typedef Id is UInt32
    struct Point {UInt32 x UInt32 y}
    enumeration Kind {ONE}
}
"#;

/// The regression this API exists for. `interface_mut(name)` returns the *first*
/// interface of that name, so mutating "the interface this id names" through a
/// name lookup can hit the wrong one.
#[test]
fn get_mut_resolves_the_id_not_the_first_node_of_that_name() {
    let mut file = parse(DUPLICATES);

    let ids: Vec<NodeId> = file.interfaces().map(|i| i.id()).collect();
    assert_eq!(ids.len(), 2, "both same-named interfaces are in the tree");
    let second = ids[1];

    match file.get_mut(second) {
        Some(NodeRefMut::Interface(iface)) => {
            assert!(iface.remove_method("play").is_some());
        }
        other => panic!("expected the second interface, got {:?}", other.is_none()),
    }

    // The second one lost `play`; the first kept both of its methods.
    let names: Vec<Vec<&str>> = file
        .interfaces()
        .map(|i| i.methods().map(|m| m.name.as_str()).collect())
        .collect();
    assert_eq!(names, vec![vec!["play", "keepme"], Vec::<&str>::new()]);

    // For contrast: the name lookup really does hit the first one.
    let mut other = parse(DUPLICATES);
    other.interface_mut("A").unwrap().remove_method("play");
    let names: Vec<Vec<&str>> = other
        .interfaces()
        .map(|i| i.methods().map(|m| m.name.as_str()).collect())
        .collect();
    assert_eq!(names, vec![vec!["keepme"], vec!["play"]]);
}

#[test]
fn get_mut_reaches_every_node_in_the_tree() {
    let mut file = parse(RICH);
    let expected: Vec<(NodeId, &'static str)> =
        file.nodes().map(|n| (n.id(), n.kind_name())).collect();
    assert!(expected.len() > 25, "expected a non-trivial tree");

    for (id, kind) in expected {
        let node = file
            .get_mut(id)
            .unwrap_or_else(|| panic!("{kind} id {id:?} did not resolve mutably"));
        assert_eq!(node.id(), id);
        assert_eq!(node.kind_name(), kind);
    }
}

#[test]
fn get_mut_agrees_with_get_on_names_and_kinds() {
    let mut file = parse(RICH);
    let by_id: BTreeMap<NodeId, (String, Option<String>)> = file
        .nodes()
        .map(|n| {
            (
                n.id(),
                (
                    n.kind_name().to_string(),
                    n.name().map(str::to_string),
                ),
            )
        })
        .collect();

    for (id, (kind, name)) in by_id {
        let node = file.get_mut(id).expect("resolves");
        assert_eq!(node.kind_name(), kind);
        assert_eq!(node.name().map(str::to_string), name);
    }
}

#[test]
fn children_mut_matches_children() {
    // The two traversals have to agree, or a mutable walk would silently skip
    // nodes the read-only one sees. Comments and annotations are the easy ones to
    // lose, since they live on `meta` rather than in `members`.
    let mut file = parse(RICH);
    let expected: BTreeMap<NodeId, Vec<NodeId>> = file
        .nodes()
        .map(|n| (n.id(), n.children().iter().map(NodeRef::id).collect()))
        .collect();

    for (id, children) in expected {
        let actual: Vec<NodeId> = file
            .get_mut(id)
            .expect("resolves")
            .children_mut()
            .iter()
            .map(NodeRefMut::id)
            .collect();
        assert_eq!(actual, children, "children disagree under node {id:?}");
    }
}

#[test]
fn unknown_ids_do_not_resolve_mutably() {
    let mut file = parse(RICH);
    let highest = file.nodes().map(|n| n.id().get()).max().unwrap();
    // Ids are never reused, so one past the highest is guaranteed absent.
    assert!(file.get_mut(NodeId::from_raw(highest + 1)).is_none());
}

#[test]
fn get_mut_marks_the_node_it_returns_dirty() {
    // Conservative by design (DESIGN §8): handing out a `&mut` counts as a
    // modification. Ancestors are left alone — `Mode::Preserve` already refuses to
    // reuse a span whose subtree contains anything dirty.
    let mut file = parse(RICH);
    let attribute_id = file
        .interface("Greeter")
        .unwrap()
        .attribute("muted")
        .unwrap()
        .id();
    let struct_id = file
        .interface("Greeter")
        .unwrap()
        .structure("Info")
        .unwrap()
        .id();

    assert!(file.nodes().all(|n| n.meta().is_none_or(|m| !m.dirty)));

    file.get_mut(attribute_id).expect("resolves");

    assert!(file.get(attribute_id).unwrap().meta().unwrap().dirty);
    assert!(
        !file.get(struct_id).unwrap().meta().unwrap().dirty,
        "an unrelated sibling stays clean"
    );
    assert!(
        !file
            .get(file.interface("Greeter").unwrap().id())
            .unwrap()
            .meta()
            .unwrap()
            .dirty,
        "the parent is not marked; a dirty descendant is enough to stop span reuse"
    );
}

#[test]
fn mutating_through_get_mut_reaches_the_printed_file() {
    let mut file = parse(RICH);
    let play_id = file
        .interface("Greeter")
        .unwrap()
        .method("play")
        .unwrap()
        .id();

    file.edit(|f| match f.get_mut(play_id) {
        Some(NodeRefMut::Method(method)) => {
            method.name = "start".to_string();
            method
                .inputs
                .add_param(minimal_fidl_collect::VariableDeclaration::create(
                    "volume", "UInt8",
                ))
                .unwrap();
        }
        _ => panic!("play should resolve to a method"),
    });

    let printed = file.to_fidl();
    assert!(printed.contains("method start"), "{printed}");
    assert!(printed.contains("UInt8 volume"), "{printed}");
    assert!(!printed.contains("method play"));
    // Still readable back, and the id still names the same node.
    assert_eq!(file.get(play_id).unwrap().name(), Some("start"));
    assert!(FidlFile::from_source(&printed).is_ok());
}

#[test]
fn annotations_are_reachable_and_writable_through_a_node_ref_mut() {
    let mut file = parse(RICH);
    let greeter_id = file.interface("Greeter").unwrap().id();

    let mut node = file.get_mut(greeter_id).expect("resolves");
    let annotations = node
        .annotations_mut()
        .expect("an interface can carry annotations");
    assert_eq!(annotations.len(), 1);
    annotations[0].contents = " rewritten".to_string();

    assert_eq!(
        file.interface("Greeter")
            .unwrap()
            .annotation("description")
            .unwrap()
            .contents,
        " rewritten"
    );

    // Nodes that cannot carry annotations say so rather than pretending.
    let package_id = file.package().unwrap().id();
    assert!(file.get_mut(package_id).unwrap().annotations_mut().is_none());
}

#[test]
fn comments_are_reachable_through_a_node_ref_mut() {
    let mut file = parse(RICH);
    let comment_ids: Vec<NodeId> = file
        .nodes()
        .filter_map(|n| match n {
            NodeRef::Comment(c) => Some(c.id),
            _ => None,
        })
        .collect();
    assert_eq!(comment_ids.len(), 2, "one free-floating, one bound to play");

    for id in comment_ids {
        match file.get_mut(id) {
            Some(NodeRefMut::Comment(comment)) => comment.text = " rewritten".to_string(),
            _ => panic!("comment {id:?} did not resolve"),
        }
    }

    assert!(file
        .nodes()
        .filter_map(|n| match n {
            NodeRef::Comment(c) => Some(c.text.as_str()),
            _ => None,
        })
        .all(|text| text == " rewritten"));
}

#[test]
fn get_mut_finds_nodes_added_by_an_edit() {
    let mut file = parse(RICH);
    file.edit(|f| {
        f.interface_mut("Greeter")
            .unwrap()
            .add_method(Method::builder("stop").build())
            .unwrap();
    });

    let stop_id = file
        .interface("Greeter")
        .unwrap()
        .method("stop")
        .unwrap()
        .id();
    match file.get_mut(stop_id) {
        Some(NodeRefMut::Method(m)) => assert_eq!(m.name, "stop"),
        _ => panic!("a node inserted by edit() must be addressable"),
    }
}

#[test]
fn the_file_itself_resolves_mutably() {
    let mut file = parse(RICH);
    let file_id = file.id();
    match file.get_mut(file_id) {
        Some(NodeRefMut::File(f)) => {
            f.push_member(FileMember::Comment(Comment::line(" the end")))
        }
        _ => panic!("the file is a node too"),
    }
    assert!(file.to_fidl().contains("// the end"));
}
