//! Phase 3 acceptance: uniform traversal, annotation access, addressing, and
//! mutable visiting.

use minimal_fidl_collect::{
    Annotated, AstNode, FidlFile, FidlProject, NodePath, NodeRef, VisitMut,
};

fn parse(src: &str) -> FidlFile {
    FidlProject::generate_file_from_string(src.to_string()).expect("test source should parse")
}

const ANNOTATED: &str = r#"package org.test
<** @description: the greeter **>
interface Greeter {
    version {major 1 minor 0}

    <** @description: plays a track **>
    method play {in {<** @description: which track **> UInt32 track} out {Boolean ok}}

    <** @description: mute state **>
    attribute Boolean muted

    <** @description: track info **>
    struct Info {UInt32 a String b}

    <** @description: playback state **>
    enumeration State {STOPPED PLAYING}

    typedef Duration is UInt32
}
"#;

#[test]
fn every_annotation_is_reachable_in_one_pass() {
    let file = parse(ANNOTATED);

    let mut descriptions: Vec<&str> = file
        .nodes()
        .flat_map(|node| node.annotations())
        .filter(|a| a.name == "description")
        .map(|a| a.contents.trim())
        .collect();
    descriptions.sort();

    assert_eq!(
        descriptions,
        vec![
            "mute state",
            "playback state",
            "plays a track",
            "the greeter",
            "track info",
            "which track",
        ],
        "one pass over the tree must see every annotation, at every depth"
    );
}

#[test]
fn descendants_reach_every_node_kind() {
    let file = parse(ANNOTATED);
    let mut kinds: Vec<&str> = file.nodes().map(|n| n.kind_name()).collect();
    kinds.sort();
    kinds.dedup();

    for expected in [
        "annotation",
        "attribute",
        "enum",
        "enum_value",
        "file",
        "interface",
        "method",
        "package",
        "param_list",
        "struct",
        "typedef",
        "variable_declaration",
        "version",
    ] {
        assert!(kinds.contains(&expected), "traversal missed {expected}");
    }
}

#[test]
fn every_id_in_the_tree_resolves() {
    let file = parse(ANNOTATED);
    let ids: Vec<_> = file.nodes().map(|n| n.id()).collect();
    assert!(ids.len() > 20, "expected a non-trivial tree");

    for id in &ids {
        let resolved = file.get(*id).unwrap_or_else(|| panic!("id {id:?} did not resolve"));
        assert_eq!(resolved.id(), *id);
    }

    let mut unique = ids.clone();
    unique.sort();
    unique.dedup();
    assert_eq!(unique.len(), ids.len(), "ids must be unique across the tree");
}

#[test]
fn unknown_ids_do_not_resolve() {
    let file = parse(ANNOTATED);
    let highest = file.nodes().map(|n| n.id().get()).max().unwrap();
    // Ids are never reused, so one past the highest is guaranteed absent.
    let absent = file.nodes().find(|n| n.id().get() == highest + 1);
    assert!(absent.is_none());
}

#[test]
fn paths_round_trip_through_the_tree() {
    let file = parse(ANNOTATED);

    for node in file.nodes() {
        let path = file
            .path_of(node.id())
            .unwrap_or_else(|| panic!("no path for {}", node.kind_name()));
        let found = file
            .at_path(&path)
            .unwrap_or_else(|| panic!("path {path} did not resolve back"));
        assert_eq!(found.id(), node.id(), "path {path} resolved to a different node");
    }
}

#[test]
fn paths_read_sensibly() {
    let file = parse(ANNOTATED);
    let play = file.interface("Greeter").unwrap().method("play").unwrap();
    let path = file.path_of(play.id()).unwrap();
    assert_eq!(path.to_string(), "interface(Greeter)/method(play)");

    // The file root is the empty path.
    assert_eq!(file.path_of(file.id()).unwrap(), NodePath::default());
    assert_eq!(NodePath::default().to_string(), "<file>");
}

#[test]
fn annotated_trait_reads_and_writes() {
    let mut file = parse(ANNOTATED);
    let iface = file.interface_mut("Greeter").unwrap();

    assert_eq!(
        iface.annotation("description").unwrap().contents.trim(),
        "the greeter"
    );
    assert!(iface.has_annotation("description"));
    assert!(!iface.has_annotation("deprecated"));

    iface.set_annotation("deprecated", " use Greeter2 ");
    assert_eq!(
        iface.annotation("deprecated").unwrap().contents,
        " use Greeter2 "
    );

    // set_annotation on an existing name replaces rather than duplicating.
    iface.set_annotation("description", " replaced ");
    assert_eq!(
        iface
            .annotations()
            .iter()
            .filter(|a| a.name == "description")
            .count(),
        1
    );
    assert_eq!(iface.annotation("description").unwrap().contents, " replaced ");

    let removed = iface.remove_annotation("deprecated").expect("was present");
    assert_eq!(removed.name, "deprecated");
    assert!(!iface.has_annotation("deprecated"));
    assert!(iface.remove_annotation("deprecated").is_none());
}

#[test]
fn annotated_works_on_nested_nodes() {
    let file = parse(ANNOTATED);
    let iface = file.interface("Greeter").unwrap();

    let param = iface
        .method("play")
        .unwrap()
        .input_parameters()
        .next()
        .unwrap();
    assert_eq!(
        param.annotation("description").unwrap().contents.trim(),
        "which track"
    );

    let state = iface.enumeration("State").unwrap();
    assert_eq!(
        state.annotation("description").unwrap().contents.trim(),
        "playback state"
    );
}

/// A whole-tree rewrite: the case `descendants()` cannot serve because it hands
/// out shared references.
struct UppercaseAnnotations {
    rewritten: usize,
}

impl VisitMut for UppercaseAnnotations {
    fn visit_annotation(&mut self, node: &mut minimal_fidl_collect::Annotation) {
        node.contents = node.contents.to_uppercase();
        self.rewritten += 1;
    }
}

#[test]
fn visit_mut_rewrites_the_whole_tree() {
    let mut file = parse(ANNOTATED);
    let mut visitor = UppercaseAnnotations { rewritten: 0 };
    visitor.visit_file(&mut file);

    assert_eq!(visitor.rewritten, 6, "every annotation must be visited");
    for node in file.nodes() {
        for annotation in node.annotations() {
            assert_eq!(annotation.contents, annotation.contents.to_uppercase());
        }
    }
}

#[test]
fn visit_mut_reaches_comments_including_bound_trivia() {
    let src = r#"package org.test
// free floating

interface A {
    // bound to x
    attribute UInt32 x
}
"#;
    struct CountComments(usize);
    impl VisitMut for CountComments {
        fn visit_comment(&mut self, _node: &mut minimal_fidl_collect::Comment) {
            self.0 += 1;
        }
    }

    let mut file = parse(src);
    let mut counter = CountComments(0);
    counter.visit_file(&mut file);
    assert_eq!(counter.0, 2, "both free-floating and bound comments visited");
}

#[test]
fn traversal_sees_comments_bound_as_leading_trivia() {
    let src = r#"package org.test
// one
interface A {
    // two
    attribute UInt32 x

    // three

    attribute UInt32 y
    /* four */
}
"#;
    let file = parse(src);
    let mut comments: Vec<&str> = file
        .nodes()
        .filter_map(|n| match n {
            NodeRef::Comment(c) => Some(c.text.as_str()),
            _ => None,
        })
        .collect();
    comments.sort();
    assert_eq!(comments, vec![" four ", " one", " three", " two"]);
}

#[test]
fn children_are_returned_in_traversal_order() {
    let file = parse(ANNOTATED);
    let iface = file.interface("Greeter").unwrap();
    let kinds: Vec<&str> = NodeRef::Interface(iface)
        .children()
        .iter()
        .map(|c| c.kind_name())
        .collect();

    // Trivia and annotations first, then version, then members in source order.
    assert_eq!(
        kinds,
        vec![
            "annotation",
            "version",
            "method",
            "attribute",
            "struct",
            "enum",
            "typedef",
        ]
    );
}

#[test]
fn node_names_are_exposed_uniformly() {
    let file = parse(ANNOTATED);
    let named: Vec<(&str, &str)> = file
        .nodes()
        .filter_map(|n| n.name().map(|name| (n.kind_name(), name)))
        .collect();

    assert!(named.contains(&("interface", "Greeter")));
    assert!(named.contains(&("method", "play")));
    assert!(named.contains(&("enum_value", "STOPPED")));
    assert!(named.contains(&("typedef", "Duration")));
    // Nodes without a declared name report None rather than a placeholder.
    assert!(file
        .nodes()
        .any(|n| n.kind_name() == "version" && n.name().is_none()));
}
