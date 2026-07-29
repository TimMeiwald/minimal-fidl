//! Phase 7 acceptance: reading files in and writing them back out.

use minimal_fidl_collect::{FidlFile, Method, Mode, Project};
use std::str::FromStr;

const SRC: &str = "package org.test\ninterface Greeter {\n    attribute Boolean muted\n}\n";

fn scratch(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("minimal-fidl-io-{name}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("create scratch dir");
    dir
}

#[test]
fn from_source_and_from_str_agree() {
    let a = FidlFile::from_source(SRC).unwrap();
    let b = FidlFile::from_str(SRC).unwrap();
    assert_eq!(a.to_fidl(), b.to_fidl());
    assert!(a.path.is_none(), "a string has no path");
}

#[test]
fn a_file_round_trips_through_disk() {
    let dir = scratch("round-trip");
    let path = dir.join("greeter.fidl");
    std::fs::write(&path, SRC).unwrap();

    let file = FidlFile::from_path(&path).expect("reads");
    assert_eq!(file.path.as_deref(), Some(path.as_path()));
    assert_eq!(file.interfaces().count(), 1);

    file.save().expect("writes back");
    let reread = FidlFile::from_path(&path).expect("reads again");
    assert_eq!(reread.to_fidl(), file.to_fidl());
}

#[test]
fn saving_a_string_built_file_is_an_error_not_a_panic() {
    let file = FidlFile::from_source(SRC).unwrap();
    let err = file.save().expect_err("no path to save to");
    assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);
}

#[test]
fn write_to_does_not_change_the_remembered_path() {
    let dir = scratch("write-to");
    let original = dir.join("a.fidl");
    let elsewhere = dir.join("b.fidl");
    std::fs::write(&original, SRC).unwrap();

    let file = FidlFile::from_path(&original).unwrap();
    file.write_to(&elsewhere).unwrap();

    assert_eq!(file.path.as_deref(), Some(original.as_path()));
    assert!(elsewhere.exists());
    assert_eq!(
        FidlFile::from_path(&elsewhere).unwrap().to_fidl(),
        file.to_fidl()
    );
}

#[test]
fn edits_survive_a_save_and_reload() {
    let dir = scratch("edit");
    let path = dir.join("greeter.fidl");
    std::fs::write(&path, SRC).unwrap();

    let mut file = FidlFile::from_path(&path).unwrap();
    file.edit(|f| {
        f.interface_mut("Greeter")
            .unwrap()
            .add_method(Method::builder("play").input("track", "UInt32").build())
            .unwrap();
    });
    file.save().unwrap();

    let reloaded = FidlFile::from_path(&path).unwrap();
    let play = reloaded
        .interface("Greeter")
        .and_then(|i| i.method("play"))
        .expect("the added method persisted");
    assert_eq!(play.input_parameters().count(), 1);
}

#[test]
fn saving_preserved_keeps_untouched_text_byte_for_byte() {
    let dir = scratch("preserve");
    let path = dir.join("messy.fidl");
    let messy = "package org.test\ninterface A {   attribute UInt32 x\n}\ninterface B {   attribute UInt32 y\n}\n";
    std::fs::write(&path, messy).unwrap();

    let mut file = FidlFile::from_path(&path).unwrap();
    file.edit(|f| {
        f.interface_mut("A")
            .unwrap()
            .add_method(Method::builder("play").build())
            .unwrap();
    });
    file.save_preserving().unwrap();

    let written = std::fs::read_to_string(&path).unwrap();
    assert!(
        written.contains("interface B {   attribute UInt32 y\n}"),
        "untouched interface B kept its original text:\n{written}"
    );
    assert!(written.contains("method play {}"));
    // And the result is still parseable.
    assert!(FidlFile::from_path(&path).is_ok());
}

#[test]
fn a_project_loads_every_fidl_file_under_a_directory() {
    let dir = scratch("project");
    std::fs::create_dir_all(dir.join("nested")).unwrap();
    std::fs::write(dir.join("one.fidl"), SRC).unwrap();
    std::fs::write(
        dir.join("nested/two.fidl"),
        "package org.test2\ninterface Other {}\n",
    )
    .unwrap();
    // Not a .fidl file: must be ignored.
    std::fs::write(dir.join("notes.txt"), "ignore me").unwrap();

    let project = Project::load(&dir).expect("loads");
    assert_eq!(project.files.len(), 2);

    let names: Vec<String> = project
        .files
        .iter()
        .flat_map(|f| f.interfaces().map(|i| i.name.clone()))
        .collect();
    assert!(names.contains(&"Greeter".to_string()));
    assert!(names.contains(&"Other".to_string()));

    assert!(project.file(dir.join("one.fidl")).is_some());
    assert!(project.file(dir.join("missing.fidl")).is_none());
    assert_eq!(project.validate(), vec![]);
}

#[test]
fn a_project_writes_every_file_back() {
    let dir = scratch("project-write");
    std::fs::write(dir.join("one.fidl"), SRC).unwrap();
    std::fs::write(dir.join("two.fidl"), "package org.t\ninterface B {}\n").unwrap();

    let mut project = Project::load(&dir).unwrap();
    for file in &mut project.files {
        file.edit(|f| {
            if let Some(iface) = f.interface_mut("Greeter") {
                iface.add_method(Method::builder("play").build()).unwrap();
            }
        });
    }
    project.write_all().unwrap();

    let reloaded = Project::load(&dir).unwrap();
    assert_eq!(reloaded.files.len(), 2);
    assert!(reloaded
        .files
        .iter()
        .any(|f| f.interface("Greeter").is_some_and(|i| i.method("play").is_some())));
}

#[test]
fn project_validation_reports_the_file_a_problem_came_from() {
    let dir = scratch("project-validate");
    let bad = dir.join("dupes.fidl");
    std::fs::write(
        &bad,
        "package org.test\ninterface A {}\ninterface A {}\n",
    )
    .unwrap();

    let project = Project::load(&dir).expect("a duplicate name still loads");
    let diagnostics = project.validate();
    assert_eq!(diagnostics.len(), 1, "{diagnostics:?}");
    assert_eq!(diagnostics[0].0.as_deref(), Some(bad.as_path()));
    assert!(diagnostics[0].1.message.contains("duplicate interface 'A'"));
}

// --- loading a directory reports every failure, and keeps what parsed -------
//
// One malformed file used to end the whole load, hiding both the other failures
// and every file that was fine.

/// Something the grammar genuinely rejects. Duplicate names are no longer a
/// parse failure — they are a `validate()` diagnostic — so they will not do.
const UNPARSEABLE: &str = "package org.test\ninterface {{{ not fidl at all\n";

#[test]
fn a_bad_file_does_not_hide_the_good_ones() {
    let dir = scratch("project-mixed");
    std::fs::write(dir.join("good.fidl"), SRC).unwrap();
    std::fs::write(dir.join("bad.fidl"), UNPARSEABLE).unwrap();

    let project = Project::load(&dir).expect("the directory itself is readable");

    assert_eq!(project.files.len(), 1);
    assert!(project.files[0].interface("Greeter").is_some());

    assert!(project.has_errors());
    assert_eq!(project.errors.len(), 1, "{:?}", project.errors);
    assert_eq!(project.errors[0].path, dir.join("bad.fidl"));
}

#[test]
fn every_bad_file_is_reported_not_just_the_first() {
    let dir = scratch("project-many-bad");
    std::fs::write(dir.join("a.fidl"), UNPARSEABLE).unwrap();
    std::fs::write(dir.join("b.fidl"), UNPARSEABLE).unwrap();
    std::fs::write(dir.join("c.fidl"), UNPARSEABLE).unwrap();

    let project = Project::load(&dir).unwrap();

    assert!(project.files.is_empty());
    let mut reported: Vec<_> = project.errors.iter().map(|e| e.path.clone()).collect();
    reported.sort();
    assert_eq!(
        reported,
        vec![dir.join("a.fidl"), dir.join("b.fidl"), dir.join("c.fidl")]
    );
}

#[test]
fn a_bad_file_does_not_stop_the_files_after_it() {
    let dir = scratch("project-bad-first");
    // The walk is sorted, so "a-bad" is loaded before either good file: this
    // fails if the loader still aborts on the first error.
    std::fs::write(dir.join("a-bad.fidl"), UNPARSEABLE).unwrap();
    std::fs::write(dir.join("b-good.fidl"), SRC).unwrap();
    std::fs::write(dir.join("c-good.fidl"), "package org.t\ninterface C {}\n").unwrap();

    let project = Project::load(&dir).unwrap();

    assert_eq!(project.errors.len(), 1);
    let loaded: Vec<_> = project
        .files
        .iter()
        .map(|f| f.path.clone().unwrap())
        .collect();
    assert_eq!(loaded, vec![dir.join("b-good.fidl"), dir.join("c-good.fidl")]);
}

#[test]
fn a_file_that_parses_but_will_not_build_a_tree_is_reported_too() {
    let dir = scratch("project-bad-tree");
    // Grammatically fine — the enum literal is simply too large for the u64 the
    // value is collected into. A distinct failure from a parse error.
    let bad = dir.join("overflow.fidl");
    std::fs::write(
        &bad,
        "package org.test\ninterface A {\nenumeration E {\nBIG = 0xFFFFFFFFFFFFFFFFF\n}\n}\n",
    )
    .unwrap();
    std::fs::write(dir.join("good.fidl"), SRC).unwrap();

    let project = Project::load(&dir).unwrap();

    assert_eq!(project.files.len(), 1, "the good file still loaded");
    assert_eq!(project.errors.len(), 1, "{:?}", project.errors);
    assert_eq!(project.errors[0].path, bad);
    assert!(
        matches!(
            project.errors[0].error,
            minimal_fidl_collect::FileError::CouldNotConvertToInteger(_)
        ),
        "this file parsed, then failed to build: {:?}",
        project.errors[0].error
    );
}

#[test]
#[cfg(unix)]
fn an_unreadable_subdirectory_does_not_abort_the_walk() {
    use std::os::unix::fs::PermissionsExt;

    let dir = scratch("project-locked-subdir");
    std::fs::write(dir.join("good.fidl"), SRC).unwrap();
    let locked = dir.join("locked");
    std::fs::create_dir(&locked).unwrap();
    std::fs::write(locked.join("hidden.fidl"), SRC).unwrap();
    std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o000)).unwrap();

    // root ignores the permission bits, so there would be nothing to observe.
    let readable_anyway = std::fs::read_dir(&locked).is_ok();
    let project = Project::load(&dir).expect("the top-level directory is readable");

    if readable_anyway {
        eprintln!("skipping: this process can read a 0o000 directory (running as root?)");
    } else {
        assert_eq!(project.errors.len(), 1, "{:?}", project.errors);
        assert_eq!(project.errors[0].path, locked);
        assert_eq!(
            project.files.len(),
            1,
            "the readable half of the tree still loaded"
        );
    }

    // Leave the scratch directory removable.
    std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o755)).unwrap();
}

#[test]
fn an_all_good_directory_reports_no_errors() {
    let dir = scratch("project-all-good");
    std::fs::write(dir.join("one.fidl"), SRC).unwrap();
    std::fs::write(dir.join("two.fidl"), "package org.t\ninterface B {}\n").unwrap();

    let project = Project::load(&dir).unwrap();

    assert_eq!(project.files.len(), 2);
    assert!(!project.has_errors(), "{:?}", project.errors);
}

#[test]
fn a_missing_directory_is_an_error_not_an_empty_success() {
    // Regression test: the walk opened with `if path.is_dir()`, so a path that
    // did not exist fell straight through to `Ok(vec![])` — silence, not a
    // wrong value, which is why this needs a test of its own.
    let dir = scratch("project-missing").join("nope");
    let err = Project::load(&dir).expect_err("a missing directory cannot be loaded");
    assert_eq!(err.kind(), std::io::ErrorKind::NotFound);
}

#[test]
fn a_path_that_is_a_file_is_an_error_too() {
    let dir = scratch("project-not-a-dir");
    let file = dir.join("one.fidl");
    std::fs::write(&file, SRC).unwrap();
    Project::load(&file).expect_err("a file is not a project directory");
}

#[test]
fn an_empty_directory_loads_as_empty_and_stays_distinct_from_a_missing_one() {
    let dir = scratch("project-empty");
    let project = Project::load(&dir).expect("an empty directory is still a directory");
    assert!(project.files.is_empty());
    assert!(project.errors.is_empty());
}

#[test]
fn display_renders_the_formatted_file() {
    let file = FidlFile::from_source(SRC).unwrap();
    assert_eq!(format!("{file}"), file.to_fidl());
    assert_eq!(file.to_fidl_with(Mode::Format), file.to_fidl());
}
