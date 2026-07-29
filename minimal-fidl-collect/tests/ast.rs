//! Phase 1 acceptance: ordered members, derived accessors, comment capture,
//! blank-line preservation, and node identity.

use minimal_fidl_collect::{AstNode, FidlFile, FidlProject, FileMember, InterfaceMember, NodeRef};

fn parse(src: &str) -> FidlFile {
    FidlProject::generate_file_from_string(src.to_string()).expect("test source should parse")
}

const INTERLEAVED: &str = r#"package org.test
interface Greeter {
    version {major 1 minor 0}

    // what play does
    method play {in {UInt32 track}}

    attribute Boolean muted

    // a free floating note

    struct Info {UInt32 a String b}

    method stop {}
}
"#;

#[test]
fn members_keep_source_order() {
    let file = parse(INTERLEAVED);
    let iface = file.interface("Greeter").expect("interface Greeter");

    let kinds: Vec<&str> = iface
        .members
        .iter()
        .map(|m| match m {
            InterfaceMember::Method(_) => "method",
            InterfaceMember::Attribute(_) => "attribute",
            InterfaceMember::Structure(_) => "struct",
            InterfaceMember::Enumeration(_) => "enum",
            InterfaceMember::TypeDef(_) => "typedef",
            InterfaceMember::Comment(_) => "comment",
        })
        .collect();

    // The free-floating comment keeps its slot; the doc comment above `play`
    // binds to the method and so does not appear as a member.
    assert_eq!(
        kinds,
        vec!["method", "attribute", "comment", "struct", "method"]
    );
}

#[test]
fn accessors_filter_in_order() {
    let file = parse(INTERLEAVED);
    let iface = file.interface("Greeter").unwrap();

    let method_names: Vec<&str> = iface.methods().map(|m| m.name.as_str()).collect();
    assert_eq!(method_names, vec!["play", "stop"]);

    let attr_names: Vec<&str> = iface.attributes().map(|a| a.name.as_str()).collect();
    assert_eq!(attr_names, vec!["muted"]);

    assert!(iface.method("play").is_some());
    assert!(iface.method("nonexistent").is_none());
    assert_eq!(iface.structure("Info").unwrap().name, "Info");
}

#[test]
fn accessors_are_derived_not_duplicated() {
    // Removing a member from the ordered list must be reflected by the accessor,
    // which is the whole point of deriving rather than storing separately.
    let mut file = parse(INTERLEAVED);
    let iface = file.interface_mut("Greeter").unwrap();
    assert_eq!(iface.methods().count(), 2);

    iface
        .members
        .retain(|m| !matches!(m, InterfaceMember::Method(x) if x.name == "stop"));

    assert_eq!(iface.methods().count(), 1);
    assert!(iface.method("stop").is_none());
}

#[test]
fn mutable_accessors_write_through() {
    let mut file = parse(INTERLEAVED);
    let iface = file.interface_mut("Greeter").unwrap();
    iface.method_mut("play").unwrap().name = "resume".to_string();
    assert!(iface.method("play").is_none());
    assert_eq!(iface.method("resume").unwrap().name, "resume");
}

#[test]
fn leading_comments_bind_to_the_following_member() {
    let file = parse(INTERLEAVED);
    let iface = file.interface("Greeter").unwrap();

    let play = iface.method("play").unwrap();
    let leading: Vec<&str> = play
        .leading_comments()
        .iter()
        .map(|c| c.text.as_str())
        .collect();
    assert_eq!(leading, vec![" what play does"]);

    // `stop` has no comment above it.
    assert!(iface.method("stop").unwrap().leading_comments().is_empty());
}

#[test]
fn free_floating_comments_keep_their_position() {
    let file = parse(INTERLEAVED);
    let iface = file.interface("Greeter").unwrap();

    let comment = iface
        .members
        .iter()
        .find_map(|m| match m {
            InterfaceMember::Comment(c) => Some(c),
            _ => None,
        })
        .expect("the free-floating comment survives");
    assert_eq!(comment.text, " a free floating note");
}

#[test]
fn blank_lines_are_recorded() {
    let file = parse(INTERLEAVED);
    let iface = file.interface("Greeter").unwrap();

    // `play` is preceded by its doc comment, which is preceded by one blank line.
    // The member inherits the run's spacing.
    assert_eq!(iface.method("play").unwrap().blank_lines_before(), 1);
    assert_eq!(iface.attribute("muted").unwrap().blank_lines_before(), 1);
    assert_eq!(iface.structure("Info").unwrap().blank_lines_before(), 1);
}

#[test]
fn no_blank_line_is_recorded_as_zero() {
    let file = parse("package org.test\ninterface A {\n    attribute UInt32 x\n    attribute UInt32 y\n}\n");
    let iface = file.interface("A").unwrap();
    assert_eq!(iface.attribute("x").unwrap().blank_lines_before(), 0);
    assert_eq!(iface.attribute("y").unwrap().blank_lines_before(), 0);
}

#[test]
fn every_comment_in_the_source_survives_parsing() {
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
    let mut found: Vec<String> = file
        .nodes()
        .filter_map(|n| match n {
            NodeRef::Comment(c) => Some(c.text.clone()),
            _ => None,
        })
        .collect();
    found.sort();
    assert_eq!(
        found,
        vec![
            " four ".to_string(),
            " one".to_string(),
            " three".to_string(),
            " two".to_string(),
        ],
        "all four comments must be preserved somewhere in the tree"
    );
}

#[test]
fn node_ids_are_assigned_and_unique() {
    let file = parse(INTERLEAVED);
    let mut ids = vec![file.id()];
    let iface = file.interface("Greeter").unwrap();
    ids.push(iface.id());
    ids.extend(iface.methods().map(|m| m.id()));
    ids.extend(iface.attributes().map(|a| a.id()));
    ids.extend(iface.structures().map(|s| s.id()));
    ids.extend(iface.structures().flat_map(|s| s.fields()).map(|f| f.id()));

    assert!(ids.iter().all(|id| id.is_assigned()), "no unassigned ids");
    let mut sorted = ids.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(sorted.len(), ids.len(), "ids must be unique");
    assert!(file.node_count() >= ids.len() as u32);
}

#[test]
fn struct_fields_keep_order() {
    let file = parse(INTERLEAVED);
    let info = file.interface("Greeter").unwrap().structure("Info").unwrap();
    let names: Vec<&str> = info.fields().map(|f| f.name.as_str()).collect();
    assert_eq!(names, vec!["a", "b"]);
}

#[test]
fn method_parameters_keep_order() {
    let src = "package org.test\ninterface A {method m {in {UInt32 first String second} out {Boolean ok}}}\n";
    let file = parse(src);
    let m = file.interface("A").unwrap().method("m").unwrap();
    let ins: Vec<&str> = m.input_parameters().map(|p| p.name.as_str()).collect();
    let outs: Vec<&str> = m.output_parameters().map(|p| p.name.as_str()).collect();
    assert_eq!(ins, vec!["first", "second"]);
    assert_eq!(outs, vec!["ok"]);
}

#[test]
fn file_level_members_keep_order() {
    let src = r#"package org.test
import model "a.fidl"
interface A {}
typeCollection T {}
"#;
    let file = parse(src);
    let kinds: Vec<&str> = file
        .members
        .iter()
        .map(|m| match m {
            FileMember::Package(_) => "package",
            FileMember::ImportModel(_) => "import_model",
            FileMember::ImportNamespace(_) => "import_namespace",
            FileMember::Interface(_) => "interface",
            FileMember::TypeCollection(_) => "type_collection",
            FileMember::Comment(_) => "comment",
        })
        .collect();
    assert_eq!(
        kinds,
        vec!["package", "import_model", "interface", "type_collection"]
    );
    assert!(file.package().is_some());
}
