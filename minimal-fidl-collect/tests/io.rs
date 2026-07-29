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

#[test]
fn display_renders_the_formatted_file() {
    let file = FidlFile::from_source(SRC).unwrap();
    assert_eq!(format!("{file}"), file.to_fidl());
    assert_eq!(file.to_fidl_with(Mode::Format), file.to_fidl());
}
