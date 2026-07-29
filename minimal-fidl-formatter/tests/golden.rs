//! Golden output for the formatter.
//!
//! The 32 tests in `src/lib.rs` assert nothing — they print and `unwrap()`, so
//! they catch a panic and nothing else; they cannot detect an output change at
//! all. This file is the actual oracle: it pins
//! the exact bytes the formatter produces for every corpus input, so any change
//! in output shows up as a diff rather than passing silently.
//!
//! Regenerate after a *deliberate* formatting change:
//!
//! ```text
//! UPDATE_GOLDEN=1 cargo test -p minimal-fidl-formatter --test golden
//! ```
//!
//! Review the resulting diff. An unexplained change is a regression.

mod corpus;

use minimal_fidl_formatter::Formatter;
use minimal_fidl_parser::{
    grammar, BasicContext, BasicPublisher, Context, Key, Rules, Source, _var_name, RULES_SIZE,
};
use std::cell::RefCell;
use std::path::PathBuf;

fn parse(input: &str) -> Option<BasicPublisher> {
    let string = input.to_string();
    let src_len = string.len() as u32;
    let source = Source::new(&string);
    let context = RefCell::new(BasicContext::new(src_len as usize, RULES_SIZE as usize));
    let result = {
        let executor = _var_name(Rules::Grammar, &context, grammar);
        executor(Key(0), &source, 0)
    };
    if result != (true, src_len) {
        return None;
    }
    Some(context.into_inner().get_publisher().clear_false())
}

fn format(src: &str) -> String {
    let publisher = parse(src).expect("corpus input must parse");
    Formatter::new(src, &publisher)
        .format()
        .expect("corpus input must format")
}

fn golden_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/golden")
}

#[test]
fn formatter_output_matches_golden() {
    let dir = golden_dir();
    let updating = std::env::var("UPDATE_GOLDEN").is_ok();
    if updating {
        std::fs::create_dir_all(&dir).expect("create golden dir");
    }

    let mut mismatches: Vec<String> = Vec::new();

    for (name, src) in corpus::CASES {
        let actual = format(src);
        let path = dir.join(format!("{name}.txt"));

        if updating {
            std::fs::write(&path, &actual).expect("write golden");
            continue;
        }

        match std::fs::read_to_string(&path) {
            Ok(expected) if expected == actual => {}
            Ok(expected) => mismatches.push(format!(
                "--- {name} ---\nexpected:\n{expected}\n---\nactual:\n{actual}\n---"
            )),
            Err(_) => mismatches.push(format!(
                "--- {name} ---\nno golden file at {}; run with UPDATE_GOLDEN=1",
                path.display()
            )),
        }
    }

    assert!(
        mismatches.is_empty(),
        "{} of {} corpus outputs changed:\n\n{}",
        mismatches.len(),
        corpus::CASES.len(),
        mismatches.join("\n")
    );
}

/// Both were non-empty under the old CST formatter, which produced output that
/// failed to reparse for test_formatter_30 and _31 (running fmt destroyed those
/// files) and was not idempotent for _27. The AST printer fixes all three, so
/// these must stay empty.
const KNOWN_BROKEN_IDEMPOTENT: &[&str] = &[];
const KNOWN_BROKEN_REPARSE: &[&str] = &[];

#[test]
fn formatting_is_idempotent() {
    // Formatting already-formatted text must be a no-op. Style-independent, so it
    // stays valid across a deliberate formatting change.
    let mut broken: Vec<&str> = Vec::new();
    for (name, src) in corpus::CASES {
        let once = format(src);
        if parse(&once).is_none() {
            continue; // covered by formatter_output_reparses
        }
        if once != format(&once) {
            broken.push(name);
        }
    }
    assert_eq!(
        broken, KNOWN_BROKEN_IDEMPOTENT,
        "set of non-idempotent cases changed"
    );
}

#[test]
fn formatter_output_reparses() {
    // Output that cannot be parsed back is corrupt regardless of how it looks:
    // running the formatter over such a file destroys it.
    let mut broken: Vec<&str> = Vec::new();
    for (name, src) in corpus::CASES {
        if parse(&format(src)).is_none() {
            broken.push(name);
        }
    }
    assert_eq!(
        broken, KNOWN_BROKEN_REPARSE,
        "set of non-reparsing cases changed"
    );
}
