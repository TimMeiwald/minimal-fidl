//! The whole pipeline over the repository's real `.fidl` files.
//!
//! The formatter corpus (`tests/printing.rs`) is made of hand-written fragments
//! chosen to stress the *parser*. These are complete, realistic models, and they
//! are what `minimal-fidl-cli fmt` is actually pointed at. Every property the
//! synthetic corpus checks is rechecked here against real input.

use minimal_fidl_collect::{diff, DiffOptions, FidlFile, Mode, NodeRef, Project};
use std::path::{Path, PathBuf};

fn corpus_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../minimal-fidl-parser/tests/grammar_test_files")
}

fn corpus() -> Vec<(String, String)> {
    let dir = corpus_dir();
    let mut files: Vec<(String, String)> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("read {}: {e}", dir.display()))
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|p| p.extension().is_some_and(|e| e == "fidl"))
        .map(|p| {
            let name = p.file_name().unwrap().to_string_lossy().to_string();
            (name, std::fs::read_to_string(&p).expect("read fidl file"))
        })
        .collect();
    files.sort();
    assert!(files.len() >= 10, "expected a real corpus, found {}", files.len());
    files
}

fn parse(name: &str, src: &str) -> FidlFile {
    FidlFile::from_source(src).unwrap_or_else(|e| panic!("{name} failed to build a tree: {e}"))
}

#[test]
fn every_real_file_builds_a_tree() {
    for (name, src) in corpus() {
        let file = parse(&name, &src);
        assert!(
            file.node_count() > 0,
            "{name} produced an empty tree"
        );
    }
}

#[test]
fn every_real_file_reparses_after_printing() {
    for (name, src) in corpus() {
        let printed = parse(&name, &src).to_fidl();
        assert!(
            FidlFile::from_source(&printed).is_ok(),
            "{name}: formatted output no longer parses:\n{printed}"
        );
    }
}

#[test]
fn printing_real_files_is_idempotent() {
    for (name, src) in corpus() {
        let once = parse(&name, &src).to_fidl();
        let twice = parse(&name, &once).to_fidl();
        assert_eq!(once, twice, "{name}: formatting is not idempotent");
    }
}

#[test]
fn formatting_a_real_file_never_changes_its_meaning() {
    for (name, src) in corpus() {
        let before = parse(&name, &src);
        let after = parse(&name, &before.to_fidl());
        let changes = diff(&before, &after, &DiffOptions::semantic());
        assert!(
            changes.is_empty(),
            "{name}: formatting altered meaning:\n{}",
            changes
                .iter()
                .map(|c| c.to_string())
                .collect::<Vec<_>>()
                .join("\n")
        );
    }
}

#[test]
fn no_comment_is_lost_from_a_real_file() {
    for (name, src) in corpus() {
        let before = parse(&name, &src);
        let mut original = comments(&before);
        let after = parse(&name, &before.to_fidl());
        let mut survived = comments(&after);
        original.sort();
        survived.sort();
        assert_eq!(original, survived, "{name}: comments changed");
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
fn every_real_file_validates_cleanly() {
    // These are the models shipped with the repo; if any of them has a duplicate
    // name or an empty one, that is worth knowing.
    for (name, src) in corpus() {
        let diagnostics = parse(&name, &src).validate();
        let errors: Vec<String> = diagnostics
            .iter()
            .filter(|d| d.severity == minimal_fidl_collect::Severity::Error)
            .map(|d| d.to_string())
            .collect();
        assert!(errors.is_empty(), "{name}:\n{}", errors.join("\n"));
    }
}

#[test]
fn preserve_mode_returns_an_unedited_real_file_byte_for_byte() {
    // Nothing was touched, so every subtree comes back verbatim from its span.
    // The only differences permitted are at the two edges the printer owns
    // regardless of mode: blank lines before the first member, and exactly one
    // trailing newline. Everything between must match the source byte for byte.
    for (name, src) in corpus() {
        let preserved = parse(&name, &src).to_fidl_with(Mode::Preserve);
        assert_eq!(
            trim_edges(&preserved),
            trim_edges(&src),
            "{name}: Preserve altered an untouched file"
        );
    }
}

/// Strips leading blank lines and normalises the file to one trailing newline.
fn trim_edges(text: &str) -> String {
    let body = text.trim_start_matches(['\n', '\r']);
    format!("{}\n", body.trim_end())
}

#[test]
fn the_project_loader_reads_the_whole_directory() {
    let project = Project::load(corpus_dir()).expect("loads the corpus");
    assert!(project.files.len() >= 10);
    assert!(
        !project.has_errors(),
        "every file in the corpus loads: {:?}",
        project.errors
    );
    assert!(
        project.files.iter().all(|f| f.path.is_some()),
        "every file loaded from disk knows its path"
    );

    let errors: Vec<String> = project
        .validate()
        .into_iter()
        .filter(|(_, d)| d.severity == minimal_fidl_collect::Severity::Error)
        .map(|(path, d)| format!("{}: {d}", display(path.as_deref())))
        .collect();
    assert!(errors.is_empty(), "{}", errors.join("\n"));
}

fn display(path: Option<&Path>) -> String {
    path.map(|p| p.display().to_string())
        .unwrap_or_else(|| "<string>".to_string())
}

