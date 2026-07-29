//! Phase 6 acceptance: structural diff.

#[path = "../../minimal-fidl-formatter/tests/corpus.rs"]
mod corpus;

use minimal_fidl_collect::{
    diff, Attribute, Change, DiffOptions, FidlFile, FidlProject, Method, Structure,
};

fn parse(src: &str) -> FidlFile {
    FidlProject::generate_file_from_string(src.to_string())
        .unwrap_or_else(|e| panic!("input must parse: {e}"))
}

const BASE: &str = r#"package org.test
interface Greeter {
    version {major 1 minor 0}
    method play {in {UInt32 track} out {Boolean ok}}
    attribute Boolean muted
    struct Info {UInt32 a String b}
}
"#;

#[test]
fn a_file_does_not_differ_from_itself() {
    let file = parse(BASE);
    assert_eq!(file.diff(&file), vec![]);
}

#[test]
fn every_corpus_file_is_identical_to_itself() {
    for (name, src) in corpus::CASES {
        let file = parse(src);
        assert_eq!(file.diff(&file), vec![], "{name} differed from itself");
    }
}

#[test]
fn reformatting_is_not_a_semantic_change() {
    // The payoff of storing layout in the tree: an unformatted file and its
    // formatted equivalent compare equal under semantic options.
    for (name, src) in corpus::CASES {
        let before = parse(src);
        let after = parse(&before.to_fidl());
        let changes = diff(&before, &after, &DiffOptions::semantic());
        assert!(
            changes.is_empty(),
            "{name}: formatting changed meaning:\n{}",
            changes
                .iter()
                .map(|c| c.to_string())
                .collect::<Vec<_>>()
                .join("\n")
        );
    }
}

#[test]
fn added_and_removed_members_are_reported() {
    let before = parse(BASE);
    let mut after = parse(BASE);
    after.edit(|f| {
        let iface = f.interface_mut("Greeter").unwrap();
        iface.add_method(Method::builder("stop").build()).unwrap();
        iface.remove_attribute("muted");
    });

    let changes = before.diff(&after);
    assert!(
        changes.iter().any(|c| matches!(
            c,
            Change::Added { kind: "method", name: Some(n), .. } if n == "stop"
        )),
        "{changes:?}"
    );
    assert!(
        changes.iter().any(|c| matches!(
            c,
            Change::Removed { kind: "attribute", name: Some(n), .. } if n == "muted"
        )),
        "{changes:?}"
    );
}

#[test]
fn a_changed_parameter_type_is_reported_as_a_modification() {
    let before = parse(BASE);
    let mut after = parse(BASE);
    after.edit(|f| {
        f.interface_mut("Greeter")
            .unwrap()
            .method_mut("play")
            .unwrap()
            .inputs
            .param_mut("track")
            .unwrap()
            .type_n = "UInt64".to_string();
    });

    let changes = before.diff(&after);
    let modified: Vec<&Change> = changes
        .iter()
        .filter(|c| matches!(c, Change::Modified { field: "type", .. }))
        .collect();
    assert_eq!(modified.len(), 1, "{changes:?}");
    match modified[0] {
        Change::Modified {
            before,
            after,
            path,
            ..
        } => {
            assert_eq!(before, "UInt32");
            assert_eq!(after, "UInt64");
            assert!(
                path.to_string().contains("method(play)"),
                "path should locate the change: {path}"
            );
        }
        other => panic!("unexpected {other:?}"),
    }
}

#[test]
fn renaming_reads_as_a_removal_plus_an_addition() {
    // Matching is by name, so a rename is not a modification. Documented rather
    // than clever: a rename and a remove+add are the same thing to a consumer.
    let before = parse(BASE);
    let mut after = parse(BASE);
    after.edit(|f| {
        f.interface_mut("Greeter").unwrap().method_mut("play").unwrap().name = "resume".to_string();
    });

    let changes = before.diff(&after);
    assert!(changes
        .iter()
        .any(|c| matches!(c, Change::Removed { name: Some(n), .. } if n == "play")));
    assert!(changes
        .iter()
        .any(|c| matches!(c, Change::Added { name: Some(n), .. } if n == "resume")));
}

#[test]
fn reordering_is_a_move_not_a_rewrite() {
    let before = parse(BASE);
    let mut after = parse(BASE);
    after.edit(|f| {
        let iface = f.interface_mut("Greeter").unwrap();
        // Swap the method and the attribute.
        iface.move_member(0, 1);
    });

    let changes = before.diff(&after);
    assert!(
        changes.iter().all(|c| matches!(c, Change::Moved { .. })),
        "a reorder must not read as add/remove: {changes:?}"
    );
    assert!(!changes.is_empty());

    // And is invisible when order does not matter.
    let ignoring = diff(
        &before,
        &after,
        &DiffOptions {
            ignore_order: true,
            ..DiffOptions::default()
        },
    );
    assert_eq!(ignoring, vec![]);
}

#[test]
fn comment_changes_are_reported_unless_ignored() {
    let before = parse("package org.test\ninterface A {\n    // old\n    attribute UInt32 x\n}\n");
    let after = parse("package org.test\ninterface A {\n    // new\n    attribute UInt32 x\n}\n");

    let changes = before.diff(&after);
    assert!(
        changes.iter().any(|c| c.kind() == "comment"),
        "{changes:?}"
    );

    let ignoring = diff(
        &before,
        &after,
        &DiffOptions {
            ignore_comments: true,
            ..DiffOptions::default()
        },
    );
    assert_eq!(ignoring, vec![]);
}

#[test]
fn layout_changes_are_ignored_by_default_and_visible_on_request() {
    let before = parse("package org.test\ninterface A {\n    attribute UInt32 x\n}\n");
    let after = parse("package org.test\ninterface A {\n\n    attribute UInt32 x\n}\n");

    assert_eq!(before.diff(&after), vec![], "spacing is not a change by default");

    let with_layout = diff(
        &before,
        &after,
        &DiffOptions {
            ignore_layout: false,
            ..DiffOptions::default()
        },
    );
    assert!(
        with_layout
            .iter()
            .any(|c| matches!(c, Change::Modified { field: "blank_lines_before", .. })),
        "{with_layout:?}"
    );
}

#[test]
fn unformatted_files_can_be_compared_directly() {
    // No normalisation pass first — the whole point of keeping layout in the tree.
    let tidy = parse(BASE);
    let messy = parse(
        "package org.test\ninterface Greeter {   version {major 1 minor 0}\n\n\n  method play {in {UInt32 track} out {Boolean ok}}\nattribute Boolean muted\n   struct Info {UInt32 a String b}}\n",
    );
    assert_eq!(
        diff(&tidy, &messy, &DiffOptions::semantic()),
        vec![],
        "whitespace alone must not register as a difference"
    );
}

#[test]
fn version_and_annotation_changes_are_reported() {
    let before = parse("package org.test\n<** @description: v1 **>\ninterface A {version {major 1 minor 0}}\n");
    let after = parse("package org.test\n<** @description: v2 **>\ninterface A {version {major 2 minor 0}}\n");

    let changes = before.diff(&after);
    assert!(
        changes.iter().any(
            |c| matches!(c, Change::Modified { kind: "version", field: "major", before, after, .. }
                if before == "1" && after == "2")
        ),
        "{changes:?}"
    );
    assert!(
        changes.iter().any(|c| matches!(
            c,
            Change::Modified { kind: "annotation", field: "contents", .. }
        )),
        "{changes:?}"
    );
}

#[test]
fn nested_structure_changes_are_located_precisely() {
    let before = parse(BASE);
    let mut after = parse(BASE);
    after.edit(|f| {
        f.interface_mut("Greeter")
            .unwrap()
            .structure_mut("Info")
            .unwrap()
            .remove_field("b");
    });

    let changes = before.diff(&after);
    assert_eq!(changes.len(), 1, "{changes:?}");
    assert_eq!(
        changes[0].path().to_string(),
        "interface(Greeter)/struct(Info)/variable_declaration(b)"
    );
}

/// The application the diff exists for: telling a consumer-breaking change from a
/// safe one. The classification is the caller's policy, not the differ's.
#[test]
fn breaking_changes_can_be_identified_from_the_result() {
    let before = parse(BASE);
    let mut after = parse(BASE);
    after.edit(|f| {
        let iface = f.interface_mut("Greeter").unwrap();
        iface.remove_method("play"); // breaking
        iface
            .add_method(Method::builder("next").build())
            .unwrap(); // additive
        iface
            .add_attribute(Attribute::create("volume", "UInt8"))
            .unwrap(); // additive
        iface
            .add_structure(Structure::builder("Extra").field("z", "UInt32").build())
            .unwrap(); // additive
    });

    let changes = before.diff(&after);
    let breaking: Vec<&Change> = changes
        .iter()
        .filter(|c| {
            matches!(
                c,
                Change::Removed { .. } | Change::Modified { field: "type", .. }
            )
        })
        .collect();

    assert_eq!(breaking.len(), 1, "only the removal breaks consumers: {changes:?}");
    assert!(matches!(
        breaking[0],
        Change::Removed { kind: "method", name: Some(n), .. } if n == "play"
    ));
}
