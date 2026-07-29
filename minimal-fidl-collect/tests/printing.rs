//! Phase 5 acceptance: the AST printer.
//!
//! The corpus here is the same 32 inputs the formatter is tested against, so the
//! two can be compared directly. Note that the formatter's own 32 tests assert
//! nothing — they print and `unwrap()` — so these properties, not those tests,
//! are the real oracle. See `DESIGN.md` §8.

#[path = "../../minimal-fidl-formatter/tests/corpus.rs"]
mod corpus;

use minimal_fidl_collect::{AstNode, FidlFile, FidlProject, Method, Mode, NodeRef};

fn parse(src: &str) -> FidlFile {
    FidlProject::generate_file_from_string(src.to_string())
        .unwrap_or_else(|e| panic!("input must parse: {e}"))
}

/// Every corpus input must build a tree. Duplicate names are a `validate()`
/// concern, not a parse error: a tool that edits `.fidl` files has to be able to
/// load a file in order to fix it.
#[test]
fn every_corpus_input_builds_a_tree() {
    let rejected: Vec<&str> = corpus::CASES
        .iter()
        .filter(|(_, src)| try_parse(src).is_none())
        .map(|(name, _)| *name)
        .collect();
    assert!(rejected.is_empty(), "these inputs failed to build: {rejected:?}");
}

fn buildable_cases() -> Vec<(&'static str, &'static str)> {
    corpus::CASES.to_vec()
}

fn try_parse(src: &str) -> Option<FidlFile> {
    FidlProject::generate_file_from_string(src.to_string()).ok()
}

#[test]
fn every_corpus_input_reparses_after_printing() {
    // The property the old CST formatter fails on test_formatter_30 and _31:
    // output that cannot be read back means running fmt destroys the file.
    let mut broken: Vec<&str> = Vec::new();
    for (name, src) in buildable_cases() {
        let printed = parse(src).to_fidl();
        if try_parse(&printed).is_none() {
            broken.push(name);
        }
    }
    assert!(
        broken.is_empty(),
        "printed output failed to reparse for: {broken:?}"
    );
}

#[test]
fn printing_is_idempotent() {
    // The property the old CST formatter fails on test_formatter_27.
    let mut broken: Vec<(&str, String, String)> = Vec::new();
    for (name, src) in buildable_cases() {
        let once = parse(src).to_fidl();
        let twice = parse(&once).to_fidl();
        if once != twice {
            broken.push((name, once, twice));
        }
    }
    assert!(
        broken.is_empty(),
        "printing is not idempotent for: {:?}",
        broken.iter().map(|(n, ..)| n).collect::<Vec<_>>()
    );
}

#[test]
fn no_comment_is_lost_in_a_round_trip() {
    for (name, src) in buildable_cases() {
        let before = parse(src);
        let mut original: Vec<String> = comments(&before);
        let printed = before.to_fidl();
        let after = parse(&printed);
        let mut survived = comments(&after);

        original.sort();
        survived.sort();
        assert_eq!(
            original, survived,
            "{name}: comments changed across a print/reparse round trip"
        );
    }
}

fn comments(file: &FidlFile) -> Vec<String> {
    file.nodes()
        .filter_map(|n| match n {
            NodeRef::Comment(c) => Some(c.text.trim().to_string()),
            _ => None,
        })
        .collect()
}

#[test]
fn structure_survives_a_round_trip() {
    // Names and shapes must be identical after printing and reparsing; only
    // layout may differ.
    for (name, src) in buildable_cases() {
        let before = parse(src);
        let after = parse(&before.to_fidl());

        assert_eq!(
            shape(&before),
            shape(&after),
            "{name}: tree shape changed across a round trip"
        );
    }
}

/// Kind and name of every node, in traversal order. Ignores ids and layout.
fn shape(file: &FidlFile) -> Vec<String> {
    file.nodes()
        .filter(|n| !matches!(n, NodeRef::Comment(_)))
        .map(|n| format!("{}:{}", n.kind_name(), n.name().unwrap_or("-")))
        .collect()
}

#[test]
fn output_style_is_stable() {
    let file = parse(
        "package org.test\ninterface Greeter {version {major 1 minor 0}\nmethod play {in {UInt32 track} out {Boolean ok}}\nattribute Boolean muted\nstruct Info {UInt32 a}\nenumeration State {IDLE PLAYING = 5}\ntypedef Duration is UInt32\n}\n",
    );
    assert_eq!(
        file.to_fidl(),
        r#"package org.test
interface Greeter {
    version {
        major 1
        minor 0
    }
    method play {
        in {
            UInt32 track
        }
        out {
            Boolean ok
        }
    }
    attribute Boolean muted
    struct Info {
        UInt32 a
    }
    enumeration State {
        IDLE
        PLAYING = 5
    }
    typedef Duration is UInt32
}
"#
    );
}

#[test]
fn blank_line_grouping_from_the_source_is_preserved() {
    let src = "package org.test\n\ninterface A {\n    attribute UInt32 x\n\n    attribute UInt32 y\n}\n";
    let printed = parse(src).to_fidl();
    assert_eq!(
        printed,
        "package org.test\n\ninterface A {\n    attribute UInt32 x\n\n    attribute UInt32 y\n}\n"
    );
}

#[test]
fn runs_of_blank_lines_collapse_to_one() {
    let src = "package org.test\n\n\n\n\ninterface A {}\n";
    assert_eq!(parse(src).to_fidl(), "package org.test\n\ninterface A {}\n");
}

#[test]
fn empty_containers_collapse_to_braces() {
    let file = parse("package org.test\ninterface A {}\ntypeCollection T {}\n");
    assert_eq!(
        file.to_fidl(),
        "package org.test\ninterface A {}\ntypeCollection T {}\n"
    );
}

#[test]
fn comments_keep_their_position_and_binding() {
    let src = r#"package org.test
// above the interface
interface A {
    // documents x
    attribute UInt32 x

    // free floating

    attribute UInt32 y
}
"#;
    assert_eq!(parse(src).to_fidl(), src);
}

#[test]
fn annotations_render_singly_and_in_blocks() {
    let src = "package org.test\n<** @description: a thing **>\ninterface A {}\n";
    assert_eq!(parse(src).to_fidl(), src);

    let multi = "package org.test\n<**\n    @description: a thing\n    @author: someone\n**>\ninterface A {}\n";
    assert_eq!(parse(multi).to_fidl(), multi);
}

#[test]
fn preserve_mode_emits_untouched_subtrees_verbatim() {
    // Deliberately ugly input: Format would reflow it, Preserve must not.
    let src = "package org.test\ninterface Greeter {   method play {in {UInt32 track}}\n}\n";
    let file = parse(src);

    let preserved = file.to_fidl_with(Mode::Preserve);
    assert!(
        preserved.contains("interface Greeter {   method play {in {UInt32 track}}\n}"),
        "untouched interface should come back byte-identical, got:\n{preserved}"
    );

    // Format normalises the same input.
    assert!(file.to_fidl().contains("    method play {"));
}

#[test]
fn preserve_mode_reformats_only_what_changed() {
    let src = "package org.test\ninterface A {   attribute UInt32 x\n}\ninterface B {   attribute UInt32 y\n}\n";
    let mut file = parse(src);

    file.edit(|f| {
        f.interface_mut("A")
            .unwrap()
            .add_method(Method::builder("play").build())
            .unwrap();
    });

    let out = file.to_fidl_with(Mode::Preserve);
    assert!(
        out.contains("interface B {   attribute UInt32 y\n}"),
        "untouched interface B must stay verbatim, got:\n{out}"
    );
    assert!(
        out.contains("method play {}"),
        "modified interface A must be reformatted, got:\n{out}"
    );
}

#[test]
fn preserve_mode_output_still_reparses() {
    for (name, src) in buildable_cases() {
        let printed = parse(src).to_fidl_with(Mode::Preserve);
        assert!(
            try_parse(&printed).is_some(),
            "{name}: Preserve output failed to reparse:\n{printed}"
        );
    }
}

#[test]
fn a_synthesised_file_prints_and_reparses() {
    use minimal_fidl_collect::{Attribute, Enumeration, Interface, Structure, TypeDef};

    let mut file = parse("package org.test\ninterface Placeholder {}\n");
    file.edit(|f| {
        f.add_interface(
            Interface::builder("Player")
                .version(1, 2)
                .annotation("description", " the player")
                .method(
                    Method::builder("play")
                        .input("track", "UInt32")
                        .output("ok", "Boolean")
                        .build(),
                )
                .attribute(Attribute::create("muted", "Boolean"))
                .structure(Structure::builder("Info").field("id", "UInt32").build())
                .enumeration(Enumeration::builder("State").value("IDLE").build())
                .typedef(TypeDef::create("Duration", "UInt32"))
                .build(),
        )
        .unwrap();
    });

    let printed = file.to_fidl();
    let reparsed = try_parse(&printed).unwrap_or_else(|| panic!("did not reparse:\n{printed}"));
    let player = reparsed.interface("Player").expect("Player survives");
    assert_eq!(player.methods().count(), 1);
    assert_eq!(player.method("play").unwrap().input_parameters().count(), 1);
    assert_eq!(player.version.as_ref().unwrap().major, Some(1));
    assert!(player.span().is_some(), "reparsed nodes have spans again");
}


/// Ground truth for comment preservation: the parser knows exactly how many
/// comments a source contains, so compare that against the tree.
///
/// The earlier round-trip test compares tree-to-tree, which cannot see a comment
/// that was dropped during construction — it is already missing from both sides.
mod comment_capture {
    use super::*;
    use minimal_fidl_parser::{
        grammar, BasicContext, BasicPublisher, Context, Key, Rules, Source, _var_name, RULES_SIZE,
    };
    use std::cell::RefCell;

    fn publisher(src: &str) -> BasicPublisher {
        let src_len = src.len() as u32;
        let source = Source::new(src);
        let context = RefCell::new(BasicContext::new(src_len as usize, RULES_SIZE as usize));
        let result = {
            let executor = _var_name(Rules::Grammar, &context, grammar);
            executor(Key(0), &source, 0)
        };
        assert_eq!(result, (true, src_len), "corpus input must parse");
        context.into_inner().get_publisher().clear_false()
    }

    /// Every comment the parser found, keyed by source offset so a missing one
    /// can be located.
    pub fn cst_comments_spanned(src: &str) -> Vec<(u32, String)> {
        let publisher = publisher(src);
        let mut out = Vec::new();
        let mut stack = vec![Key(0)];
        while let Some(key) = stack.pop() {
            let node = publisher.get_node(key);
            if matches!(node.rule, Rules::comment | Rules::multiline_comment) {
                out.push((node.start_position, node.get_string(src).trim().to_string()));
            }
            stack.extend(node.get_children().iter().copied());
        }
        out.sort();
        out
    }

    fn cst_comments(src: &str) -> Vec<String> {
        let mut v: Vec<String> = cst_comments_spanned(src).into_iter().map(|(_, t)| t).collect();
        v.sort();
        v
    }

    fn ast_comments(file: &FidlFile) -> Vec<String> {
        let mut out: Vec<String> = file
            .nodes()
            .filter_map(|n| match n {
                NodeRef::Comment(c) => Some(c.to_source().trim().to_string()),
                _ => None,
            })
            .collect();
        out.sort();
        out
    }

    #[test]
    fn the_tree_holds_every_comment_the_parser_found() {
        let mut failures = Vec::new();
        for (name, src) in corpus::CASES {
            let expected = cst_comments(src);
            let actual = ast_comments(&parse(src));
            if expected != actual {
                let mut missing = expected.clone();
                for found in &actual {
                    if let Some(i) = missing.iter().position(|m| m == found) {
                        missing.remove(i);
                    }
                }
                failures.push(format!("{name}: missing {missing:?}"));
            }
        }
        assert!(failures.is_empty(), "comments dropped:\n{}", failures.join("\n"));
    }
}

