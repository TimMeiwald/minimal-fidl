//! Phase 4 acceptance: builders, structural mutation, dirty tracking, validation.

use minimal_fidl_collect::{
    Annotated, AstNode, Attribute, Comment, EnumValue, Enumeration, FidlFile, FidlProject,
    ImportModel, ImportNamespace, Interface, InterfaceMember, Method, NodeRef, Package, Severity,
    Structure, TypeDef, VariableDeclaration,
};

fn parse(src: &str) -> FidlFile {
    FidlProject::generate_file_from_string(src.to_string()).expect("test source should parse")
}

const BASE: &str = r#"package org.test
interface Greeter {
    version {major 1 minor 0}
    method play {in {UInt32 track}}
    attribute Boolean muted
    struct Info {UInt32 a String b}
}
"#;

#[test]
fn add_and_remove_methods() {
    let mut file = parse(BASE);
    let iface = file.interface_mut("Greeter").unwrap();

    iface
        .add_method(
            Method::builder("stop")
                .input("force", "Boolean")
                .output("ok", "Boolean")
                .build(),
        )
        .expect("stop is a new name");

    let names: Vec<&str> = iface.methods().map(|m| m.name.as_str()).collect();
    assert_eq!(names, vec!["play", "stop"]);

    let removed = iface.remove_method("play").expect("play was present");
    assert_eq!(removed.name, "play");
    assert_eq!(iface.methods().count(), 1);
    assert!(iface.remove_method("play").is_none());
}

#[test]
fn adding_a_duplicate_name_is_rejected() {
    let mut file = parse(BASE);
    let iface = file.interface_mut("Greeter").unwrap();

    let err = iface
        .add_method(Method::builder("play").build())
        .expect_err("play already exists");
    assert!(err.to_string().contains("Method"));
    // The rejected node is not left in the tree.
    assert_eq!(iface.methods().count(), 1);
}

#[test]
fn removing_a_member_takes_its_leading_comments() {
    let src = r#"package org.test
interface A {
    // documents play
    method play {}
    method stop {}
}
"#;
    let mut file = parse(src);
    let iface = file.interface_mut("A").unwrap();

    let removed = iface.remove_method("play").unwrap();
    assert_eq!(removed.leading_comments().len(), 1);
    assert_eq!(removed.leading_comments()[0].text, " documents play");

    // The comment left with the method rather than orphaning into the interface.
    let orphaned = file
        .nodes()
        .filter(|n| matches!(n, NodeRef::Comment(_)))
        .count();
    assert_eq!(orphaned, 0);
}

#[test]
fn add_remove_across_every_container() {
    let mut file = parse(BASE);
    let iface = file.interface_mut("Greeter").unwrap();

    iface.add_attribute(Attribute::create("volume", "UInt8")).unwrap();
    iface.add_typedef(TypeDef::create("Duration", "UInt32")).unwrap();
    iface
        .add_structure(Structure::builder("Track").field("id", "UInt32").build())
        .unwrap();
    iface
        .add_enumeration(Enumeration::builder("State").value("STOPPED").build())
        .unwrap();

    assert_eq!(iface.attributes().count(), 2);
    assert_eq!(iface.typedefs().count(), 1);
    assert_eq!(iface.structures().count(), 2);
    assert_eq!(iface.enumerations().count(), 1);

    assert!(iface.remove_attribute("volume").is_some());
    assert!(iface.remove_typedef("Duration").is_some());
    assert!(iface.remove_structure("Track").is_some());
    assert!(iface.remove_enumeration("State").is_some());

    assert_eq!(iface.attributes().count(), 1);
    assert_eq!(iface.typedefs().count(), 0);
}

#[test]
fn struct_fields_and_method_params_mutate() {
    let mut file = parse(BASE);
    let iface = file.interface_mut("Greeter").unwrap();

    let info = iface.structure_mut("Info").unwrap();
    info.add_field(VariableDeclaration::create("c", "Boolean")).unwrap();
    assert_eq!(
        info.fields().map(|f| f.name.as_str()).collect::<Vec<_>>(),
        vec!["a", "b", "c"]
    );
    assert!(info.add_field(VariableDeclaration::create("a", "UInt32")).is_err());
    assert!(info.remove_field("b").is_some());

    let play = iface.method_mut("play").unwrap();
    play.inputs
        .add_param(VariableDeclaration::create("volume", "UInt8"))
        .unwrap();
    assert_eq!(play.input_parameters().count(), 2);
    assert!(play.inputs.remove_param("track").is_some());
    assert_eq!(play.input_parameters().count(), 1);
}

#[test]
fn interfaces_and_type_collections_mutate_at_file_level() {
    let mut file = parse(BASE);

    file.add_interface(
        Interface::builder("Player")
            .version(2, 0)
            .method(Method::builder("next").build())
            .build(),
    )
    .unwrap();
    assert_eq!(file.interfaces().count(), 2);

    assert!(file.add_interface(Interface::builder("Greeter").build()).is_err());

    let removed = file.remove_interface("Player").unwrap();
    assert_eq!(removed.name, "Player");
    assert_eq!(file.interfaces().count(), 1);
}

#[test]
fn ordering_operations_work_on_the_member_list() {
    let mut file = parse(BASE);
    let iface = file.interface_mut("Greeter").unwrap();

    // Insert a comment between the method and the attribute.
    iface.insert_member_at(1, InterfaceMember::Comment(Comment::line(" TODO")));
    let kinds: Vec<&str> = iface.members.iter().map(kind).collect();
    assert_eq!(kinds, vec!["method", "comment", "attribute", "struct"]);

    // Move the struct to the front. No sidecar indices to repair — see DESIGN §1.1.
    assert!(iface.move_member(3, 0));
    let kinds: Vec<&str> = iface.members.iter().map(kind).collect();
    assert_eq!(kinds, vec!["struct", "method", "comment", "attribute"]);

    // Accessors still agree with the reordered list.
    assert_eq!(iface.structures().next().unwrap().name, "Info");
    assert_eq!(iface.methods().next().unwrap().name, "play");

    assert!(!iface.move_member(0, 99), "out of range is a no-op");
    assert_eq!(iface.member_count(), 4);
}

fn kind(m: &InterfaceMember) -> &'static str {
    match m {
        InterfaceMember::Method(_) => "method",
        InterfaceMember::Attribute(_) => "attribute",
        InterfaceMember::Structure(_) => "struct",
        InterfaceMember::Enumeration(_) => "enum",
        InterfaceMember::TypeDef(_) => "typedef",
        InterfaceMember::Comment(_) => "comment",
    }
}

#[test]
fn inserted_nodes_get_ids_and_become_addressable() {
    let mut file = parse(BASE);

    file.edit(|f| {
        f.interface_mut("Greeter")
            .unwrap()
            .add_method(Method::builder("stop").input("force", "Boolean").build())
            .unwrap();
    });

    let stop = file.interface("Greeter").unwrap().method("stop").unwrap();
    assert!(stop.id().is_assigned(), "edit() must hand out an id");

    let path = file.path_of(stop.id()).expect("addressable by path");
    assert_eq!(path.to_string(), "interface(Greeter)/method(stop)");
    assert_eq!(file.get(stop.id()).unwrap().name(), Some("stop"));

    // Every node in the tree, including the new parameter, resolves.
    for node in file.nodes() {
        assert!(node.id().is_assigned(), "{} lacks an id", node.kind_name());
        assert!(file.get(node.id()).is_some());
    }
}

#[test]
fn assigning_missing_ids_leaves_existing_ones_alone() {
    let mut file = parse(BASE);
    let before: Vec<_> = file.nodes().map(|n| n.id()).collect();

    file.edit(|f| {
        f.interface_mut("Greeter")
            .unwrap()
            .add_attribute(Attribute::create("volume", "UInt8"))
            .unwrap();
    });

    let after: Vec<_> = file.nodes().map(|n| n.id()).collect();
    for id in &before {
        assert!(after.contains(id), "existing id {id:?} was renumbered");
    }

    // Idempotent.
    let snapshot: Vec<_> = file.nodes().map(|n| n.id()).collect();
    file.assign_missing_ids();
    assert_eq!(file.nodes().map(|n| n.id()).collect::<Vec<_>>(), snapshot);
}

#[test]
fn ids_are_stable_across_sibling_insertion_and_removal() {
    // The property that makes NodeId usable as a Python handle (DESIGN §6).
    let mut file = parse(BASE);
    let play_id = file.interface("Greeter").unwrap().method("play").unwrap().id();

    file.edit(|f| {
        let iface = f.interface_mut("Greeter").unwrap();
        iface.insert_member_at(0, InterfaceMember::Comment(Comment::line(" first")));
        iface.add_method(Method::builder("stop").build()).unwrap();
        iface.remove_attribute("muted");
    });

    let play = file.get(play_id).expect("play survives sibling churn");
    assert_eq!(play.name(), Some("play"));
}

#[test]
fn builders_produce_synthesised_nodes_with_no_span() {
    let method = Method::builder("play")
        .input("track", "UInt32")
        .output("ok", "Boolean")
        .annotation("description", " plays a track")
        .doc(" what play does")
        .build();

    assert!(method.span().is_none(), "synthesised nodes have no span");
    assert!(method.is_dirty(), "synthesised nodes are dirty by construction");
    assert_eq!(method.name, "play");
    assert_eq!(method.input_parameters().count(), 1);
    assert_eq!(method.output_parameters().count(), 1);
    assert_eq!(
        method.annotation("description").unwrap().contents,
        " plays a track"
    );
    assert_eq!(method.leading_comments()[0].text, " what play does");
}

#[test]
fn parsed_nodes_start_clean_and_mutation_marks_them_dirty() {
    let mut file = parse(BASE);
    assert!(
        file.nodes().all(|n| n.meta().is_none_or(|m| !m.dirty)),
        "a freshly parsed tree has nothing dirty"
    );

    // Handing out a &mut is what marks it — conservative by design (DESIGN §8).
    let iface = file.interface_mut("Greeter").unwrap();
    assert!(iface.is_dirty());

    let play = iface.method_mut("play").unwrap();
    assert!(play.is_dirty());

    assert!(
        !file
            .interface("Greeter")
            .unwrap()
            .attribute("muted")
            .unwrap()
            .is_dirty(),
        "untouched siblings stay clean, which is what Mode::Preserve needs"
    );
}

#[test]
fn validate_accepts_a_sound_file() {
    let file = parse(BASE);
    assert_eq!(file.validate(), vec![]);
}

#[test]
fn validate_reports_duplicates_introduced_by_mutation() {
    let mut file = parse(BASE);

    // Bypass add_method's check by pushing straight onto the ordered list — the
    // case validate() exists for.
    file.edit(|f| {
        f.interface_mut("Greeter")
            .unwrap()
            .push_member(InterfaceMember::Method(Method::builder("play").build()));
    });

    let diagnostics = file.validate();
    assert_eq!(diagnostics.len(), 1, "{diagnostics:?}");
    assert_eq!(diagnostics[0].severity, Severity::Error);
    assert!(diagnostics[0].message.contains("duplicate method 'play'"));
    assert_eq!(diagnostics[0].path.to_string(), "interface(Greeter)/method(play)");
}

#[test]
fn validate_reports_empty_names() {
    let mut file = parse(BASE);
    file.edit(|f| {
        f.interface_mut("Greeter")
            .unwrap()
            .add_method(Method::builder("  ").build())
            .unwrap();
    });

    let diagnostics = file.validate();
    assert!(
        diagnostics.iter().any(|d| d.message.contains("empty name")),
        "{diagnostics:?}"
    );
}

#[test]
fn validate_collects_every_problem_rather_than_stopping_at_the_first() {
    let mut file = parse(BASE);
    file.edit(|f| {
        let iface = f.interface_mut("Greeter").unwrap();
        iface.push_member(InterfaceMember::Method(Method::builder("play").build()));
        iface.push_member(InterfaceMember::Attribute(Attribute::create("muted", "Boolean")));
        iface.push_member(InterfaceMember::TypeDef(TypeDef::create("", "UInt32")));
    });

    let diagnostics = file.validate();
    assert!(
        diagnostics.len() >= 3,
        "bulk edits need every problem at once, got {diagnostics:?}"
    );
}

#[test]
fn enum_values_build_and_mutate() {
    let mut file = parse(BASE);
    file.edit(|f| {
        f.interface_mut("Greeter")
            .unwrap()
            .add_enumeration(
                Enumeration::builder("State")
                    .value("STOPPED")
                    .value_with("PLAYING", 5)
                    .build(),
            )
            .unwrap();
    });

    let state = file
        .interface("Greeter")
        .unwrap()
        .enumeration("State")
        .unwrap();
    assert_eq!(
        state.values().map(|v| v.name.as_str()).collect::<Vec<_>>(),
        vec!["STOPPED", "PLAYING"]
    );
    assert_eq!(state.value("PLAYING").unwrap().value, Some(5));
    assert_eq!(state.value("STOPPED").unwrap().value, None);

    let iface = file.interface_mut("Greeter").unwrap();
    let state = iface.enumeration_mut("State").unwrap();
    state.add_value(EnumValue::create("PAUSED")).unwrap();
    assert!(state.add_value(EnumValue::create("PAUSED")).is_err());
    assert!(state.remove_value("STOPPED").is_some());
    assert_eq!(state.values().count(), 2);
}

#[test]
fn a_built_file_survives_validation() {
    // Build a whole interface from nothing and check the result is sound.
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

    assert_eq!(file.validate(), vec![]);

    let player = file.interface("Player").unwrap();
    assert_eq!(player.methods().count(), 1);
    assert_eq!(player.version.as_ref().unwrap().major, Some(1));
    assert_eq!(
        player.annotation("description").unwrap().contents,
        " the player"
    );
    for node in file.nodes() {
        assert!(node.id().is_assigned());
    }
}

// ---- file-level members: package and imports ----------------------------
//
// These were the gap `member_mutators!` could not fill: neither a package nor an
// import has a `name`, so adding one meant `push_member` with a hand-built
// struct. The order they print in is not cosmetic — see the reparse test below.

#[test]
fn packages_and_imports_can_be_built_and_added() {
    // The grammar requires a package, so a package-less tree has to be made by
    // removing one rather than by parsing a file without one.
    let mut file = parse(BASE);
    file.remove_package().expect("BASE has a package");
    assert!(file.package().is_none());

    file.edit(|f| {
        f.add_package(Package::parse("org.rebuilt")).unwrap();
        f.add_import_model(ImportModel::create("other.fidl"));
        f.add_import_namespace(ImportNamespace::create(["org", "x"], "base.fidl"));
    });

    assert_eq!(file.package().unwrap().path, vec!["org", "rebuilt"]);
    assert_eq!(file.import_models().count(), 1);
    assert_eq!(file.namespaces().count(), 1);

    let printed = file.to_fidl();
    assert!(printed.contains("package org.rebuilt"), "{printed}");
    assert!(printed.contains(r#"import model "other.fidl""#), "{printed}");
    assert!(printed.contains(r#"import org.x.* from "base.fidl""#), "{printed}");
    FidlFile::from_source(&printed).expect("a rebuilt header still parses");
}

#[test]
fn a_second_package_is_rejected() {
    let mut file = parse(BASE);
    let err = file
        .add_package(Package::parse("org.other"))
        .expect_err("the grammar permits exactly one package");
    assert!(err.to_string().contains("Package"));
    assert_eq!(file.package().unwrap().path, vec!["org", "test"]);
}

#[test]
fn added_imports_print_where_the_grammar_wants_them() {
    // The grammar is `package (import)* (interface | typeCollection)*` and it
    // enforces that order, so appending an import to the end of the member list
    // would produce a file that no longer parses.
    let mut file = parse(BASE);
    file.edit(|f| {
        f.add_import_model(ImportModel::create("late.fidl"));
        f.add_import_namespace(ImportNamespace::create(["org", "z"], "late2.fidl"));
    });

    let printed = file.to_fidl();
    let import = printed.find("import model").expect("the import printed");
    let interface = printed.find("interface Greeter").expect("the interface printed");
    assert!(import < interface, "imports must precede interfaces:\n{printed}");

    FidlFile::from_source(&printed).expect("the edited file must still parse");
}

#[test]
fn a_leading_comment_block_stays_above_an_added_package() {
    // A licence header separated by a blank line is a free-floating comment
    // member of its own; a package added afterwards has to go below it, not above.
    let mut file = parse("// licence\n\npackage org.test\ninterface A {}\n");
    file.remove_package().expect("the package is there to start with");
    file.edit(|f| {
        f.add_package(Package::parse("org.rebuilt")).unwrap();
    });

    let printed = file.to_fidl();
    assert!(
        printed.starts_with("// licence\npackage org.rebuilt"),
        "{printed}"
    );
    FidlFile::from_source(&printed).expect("still parses");
}

#[test]
fn packages_and_imports_can_be_removed_and_found() {
    let src = r#"package org.test
import model "other.fidl"
import org.x.* from "base.fidl"
interface A {}
"#;
    let mut file = parse(src);

    assert!(file.import_model("other.fidl").is_some());
    assert!(file.import_model("absent.fidl").is_none());
    assert!(file.namespace("base.fidl").is_some());

    assert_eq!(
        file.remove_import_model("other.fidl").unwrap().file_path,
        std::path::PathBuf::from("other.fidl")
    );
    assert!(file.remove_import_model("other.fidl").is_none());
    assert!(file.remove_import_namespace("base.fidl").is_some());
    assert_eq!(file.remove_package().unwrap().path, vec!["org", "test"]);
    assert!(file.package().is_none());
    assert!(file.remove_package().is_none());

    assert_eq!(file.to_fidl().trim(), "interface A {}");
    // Nothing is left to reparse — the grammar requires a package — which is
    // exactly why remove_package() is a deliberate step and not a convenience.
    assert!(FidlFile::from_source(&file.to_fidl()).is_err());
}

#[test]
fn file_level_leaves_are_mutable_in_place() {
    let src = r#"package org.test
import model "other.fidl"
import org.x.* from "base.fidl"
interface A {}
"#;
    let mut file = parse(src);

    file.package_mut().unwrap().path = vec!["org".to_string(), "renamed".to_string()];
    for import in file.import_models_mut() {
        import.file_path = "moved.fidl".into();
    }
    for namespace in file.namespaces_mut() {
        namespace.import = vec!["org".to_string(), "y".to_string()];
    }

    let printed = file.to_fidl();
    assert!(printed.contains("package org.renamed"), "{printed}");
    assert!(printed.contains(r#"import model "moved.fidl""#), "{printed}");
    assert!(printed.contains(r#"import org.y.* from "base.fidl""#), "{printed}");
    FidlFile::from_source(&printed).expect("still parses");
}

#[test]
fn removing_a_member_stops_preserve_from_reprinting_it() {
    // Mode::Preserve reuses a clean node's original text. A container that just
    // lost a member still has a span covering it, so removal has to mark the
    // container dirty or the removed method comes back in the output.
    let mut file = parse(BASE);
    file.interface_mut("Greeter")
        .unwrap()
        .remove_method("play")
        .unwrap();

    let printed = file.to_fidl_with(minimal_fidl_collect::Mode::Preserve);
    assert!(!printed.contains("method play"), "{printed}");
}
